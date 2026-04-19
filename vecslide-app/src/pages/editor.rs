use js_sys::{Array, Uint8Array};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use lucide_leptos::{ArrowLeft, AudioLines, ChevronLeft, ChevronRight, CircleAlert, Download, Mic, Pause, Play, Plus, Save, Trash2};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::spawn_local;

use crate::export::{export_html, trigger_download, trigger_download_binary};
use crate::import::{export_vecslide, import_vecslide_from_bytes};
use crate::tts::{SlideAudio, TtsState, ensure_model_loaded, list_voices, synthesize_slide};
use crate::tts::config::DEFAULT_VOICE;
use crate::typst_world::{FontAssets, compile_slide_to_svg};
use crate::{LoadedFile, ThemeColorsCtx};

/// Formats a Kokoro voice ID like `af_heart` into a human-friendly label
/// like "US F — Heart" — language code, gender, then the capitalised name part.
/// The original ID is kept as the `<option>` value.

// High-level playback state used by the Play/Pause toggle button.
#[derive(Debug, Clone, Copy, PartialEq)]
enum PlaybackState {
    Idle,
    Playing,
    Paused,
}

fn format_voice_label(id: &str) -> String {
    let mut chars = id.chars();
    let lang = chars.next().unwrap_or('a');
    let gender = chars.next().unwrap_or('f');
    let name_raw = id.split_once('_').map(|(_, n)| n).unwrap_or(id);
    let name = {
        let mut c = name_raw.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    };
    let lang_code = match lang {
        'a' => "US",
        'b' => "GB",
        'j' => "JP",
        'z' => "CN",
        'e' => "ES",
        'f' => "FR",
        'h' => "IN",
        'i' => "IT",
        'p' => "BR",
        _ => "??",
    };
    let gender_label = match gender {
        'f' => "F",
        'm' => "M",
        _ => "",
    };
    format!("{lang_code} {gender_label} — {name}")
}

/// Split-view Typst editor with live SVG preview.
#[component]
pub fn Editor() -> impl IntoView {
    // ── State ─────────────────────────────────────────────────────────────
    let source = RwSignal::new(String::new());

    let slide_count = Signal::derive(move || source.get().split("\n----\n").count());
    let current_slide = RwSignal::new(0usize);

    // One narration string per slide — kept in sync with slide_count.
    let narrations: RwSignal<Vec<String>> = RwSignal::new(vec![String::new()]);
    Effect::new(move |_| {
        let count = slide_count.get();
        narrations.update(|v| v.resize(count, String::new()));
    });

    // Per-slide synthesized audio (OGG Opus). Populated by the TTS pipeline
    // on-demand; `None` = slide has no audio yet. Length is kept aligned to
    // `narrations.len()` by the effect below.
    let slide_audios: RwSignal<Vec<Option<SlideAudio>>> = RwSignal::new(vec![None]);
    Effect::new(move |_| {
        let count = slide_count.get();
        slide_audios.update(|v| v.resize(count, None));
    });

    // Global TTS pipeline state (loader → synthesizer). Starts `Uninit`
    // until the first click on "Synthesize" kicks off the loader (steps 5-9
    // of the plan). While the mock is in place (step 3) we go straight to
    // `Synthesizing` and back to `Ready`.
    let tts_state: RwSignal<TtsState> = RwSignal::new(TtsState::Uninit);

    // Selected voice for TTS synthesis.
    let selected_voice: RwSignal<String> = RwSignal::new(DEFAULT_VOICE.to_string());
    // Available voices — populated immediately from kokoro-js metadata (no model download).
    let available_voices: RwSignal<Vec<String>> = {
        let voices = list_voices()
            .into_iter()
            .filter(|id| id.starts_with('a') || id.starts_with('b'))
            .collect();
        RwSignal::new(voices)
    };
    // Keep current_slide in bounds when source changes
    let current_slide_clamped = Signal::derive(move || {
        let max = slide_count.get().saturating_sub(1);
        current_slide.get().min(max)
    });

    // ── Audio playback state ─────────────────────────────────────────────────
    let active_audio: RwSignal<Option<web_sys::HtmlAudioElement>> = RwSignal::new(None);
    let active_object_url: RwSignal<Option<String>> = RwSignal::new(None);
    let playback_state: RwSignal<PlaybackState> = RwSignal::new(PlaybackState::Idle);
    // Hold JS closure references so Rust can drop them when replaced
    let on_ended_js: RwSignal<Option<JsValue>> = RwSignal::new(None);
    let on_data_js: RwSignal<Option<JsValue>> = RwSignal::new(None);
    // setTimeout handle for the 2-second auto-advance delay.
    let auto_advance_handle: RwSignal<Option<i32>> = RwSignal::new(None);
    // Set to true when the user clicks Play; auto-advance continues until
    // the last slide, a slide without audio, or the user toggles it off.
    // This is NOT reset by stop_audio — it's a user preference that persists
    // across slide changes.
    let auto_advance_enabled: RwSignal<bool> = RwSignal::new(false);
    // Signal used by the auto-advance timeout to request playback of the
    // next slide without capturing the `start_slide_playback` closure.
    let play_request: RwSignal<Option<usize>> = RwSignal::new(None);

    // ── Microphone recording state ──────────────────────────────────────
    // True while the microphone is actively recording for the current slide.
    let recording: RwSignal<bool> = RwSignal::new(false);
    // Holds the MediaRecorder + stream alive across the async stop operation.
    // `JsValue` carries a JS object `{ recorder, stream, chunks: Array }`.
    let recorder_handle: RwSignal<Option<JsValue>> = RwSignal::new(None);

    // ── Audio control closures ───────────────────────────────────────────────

    // Stops the current audio, cancels any pending auto-advance, and resets
    // all playback signals to Idle.  Safe to call at any time.
    let stop_audio = move || {
        // Cancel pending auto-advance timeout.
        if let Some(handle) = auto_advance_handle.get() {
            web_sys::window()
                .expect("no window")
                .clear_timeout_with_handle(handle);
            auto_advance_handle.set(None);
        }
        // Pause and release the audio element.
        if let Some(el) = active_audio.get() {
            let _ = el.pause();
            el.set_onended(None);
            el.set_src("");
            active_audio.set(None);
        }
        on_ended_js.set(None);
        // Revoke the object URL.
        if let Some(url) = active_object_url.get() {
            let _ = web_sys::Url::revoke_object_url(&url);
            active_object_url.set(None);
        }
        playback_state.set(PlaybackState::Idle);
    };

    // Starts playback of the OGG audio for slide `idx`.
    // Stops any previous audio first.  Sets up `onended` to revoke the
    // object URL and, when `auto_advance_enabled` is true, schedule the
    // 2-second auto-advance to the next slide.
    let start_slide_playback = move |idx: usize| {
        // Clean up any previous audio.
        stop_audio();

        let Some(Some(audio)) = slide_audios.get().get(idx).cloned() else {
            return;
        };
        let ogg_bytes = &audio.ogg_bytes;

        let u8 = Uint8Array::new_with_length(ogg_bytes.len() as u32);
        u8.copy_from(ogg_bytes);
        let parts = Array::new();
        parts.push(&u8.buffer());
        let opts = web_sys::BlobPropertyBag::new();
        opts.set_type("audio/ogg; codecs=opus");
        let Ok(blob) =
            web_sys::Blob::new_with_buffer_source_sequence_and_options(&parts, &opts)
        else {
            return;
        };
        let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) else { return };
        let Ok(el) = web_sys::HtmlAudioElement::new_with_src(&url) else {
            let _ = web_sys::Url::revoke_object_url(&url);
            return;
        };

        active_audio.set(Some(el.clone()));
        active_object_url.set(Some(url.clone()));
        playback_state.set(PlaybackState::Playing);

        // onended: revoke URL, reset state, schedule auto-advance if enabled.
        let url_cleanup = url.clone();
        let on_ended = wasm_bindgen::closure::Closure::<dyn FnMut(JsValue)>::new(move |_: JsValue| {
            let _ = web_sys::Url::revoke_object_url(&url_cleanup);
            active_audio.set(None);
            active_object_url.set(None);
            playback_state.set(PlaybackState::Idle);

            if auto_advance_enabled.get() {
                let next_idx = current_slide_clamped.get() + 1;
                let max = slide_count.get().saturating_sub(1);
                let next_has_audio = slide_audios
                    .get()
                    .get(next_idx)
                    .is_some_and(|o| o.is_some());

                if next_idx <= max && next_has_audio {
                    let cb = wasm_bindgen::closure::Closure::<dyn FnMut()>::new(move || {
                        auto_advance_handle.set(None);
                        current_slide.set(next_idx);
                        play_request.set(Some(next_idx));
                    });
                    let window = web_sys::window().expect("no window");
                    if let Ok(handle) = window
                        .set_timeout_with_callback_and_timeout_and_arguments_0(
                            cb.as_ref().unchecked_ref(),
                            2000,
                        )
                    {
                        cb.forget();
                        auto_advance_handle.set(Some(handle));
                    }
                } else {
                    auto_advance_enabled.set(false);
                }
            }
        });
        el.set_onended(Some(on_ended.as_ref().unchecked_ref()));
        on_ended_js.set(Some(on_ended.into_js_value()));

        let _ = el.play();
    };

    // ── Microphone recording ──────────────────────────────────────────────

    // Starts recording from the microphone for the current slide.
    // The resulting OGG/Opus bytes are stored in `slide_audios[idx]`.
    let start_recording = move || {
        let rec_sig = recording;
        let handle_sig = recorder_handle;

        spawn_local(async move {
            // Request microphone access.
            let window = web_sys::window().expect("no window");
            let navigator = window.navigator();
            let devices = navigator.media_devices().map_err(|_| "MediaDevices not available");
            let devices = match devices {
                Ok(d) => d,
                Err(_) => return,
            };

            let constraints = web_sys::MediaStreamConstraints::new();
            constraints.set_audio(&JsValue::from_bool(true));
            let Ok(promise) = devices.get_user_media_with_constraints(&constraints) else {
                return;
            };
            let stream_result = wasm_bindgen_futures::JsFuture::from(promise).await;

            let stream_js = match stream_result {
                Ok(s) => s,
                Err(_) => return, // user denied permission or no mic
            };
            let stream: web_sys::MediaStream = stream_js.clone().unchecked_into();

            // Create MediaRecorder with OGG/Opus mime type.
            let opts = web_sys::MediaRecorderOptions::new();
            opts.set_mime_type("audio/ogg; codecs=opus");
            let recorder = match web_sys::MediaRecorder::new_with_media_stream_and_media_recorder_options(
                &stream,
                &opts,
            ) {
                Ok(r) => r,
                Err(_) => {
                    // Fallback: try without specific mime type (browser default, usually WebM).
                    match web_sys::MediaRecorder::new_with_media_stream(&stream) {
                        Ok(r) => r,
                        Err(_) => {
                            // Stream will be GC'd; just bail out.
                            return;
                        }
                    }
                }
            };

            // Collect data chunks.
            let chunks = js_sys::Array::new();
            let chunks_clone = chunks.clone();
            let on_data = wasm_bindgen::closure::Closure::<dyn FnMut(JsValue)>::new(
                move |event: JsValue| {
                    let event: web_sys::BlobEvent = event.unchecked_into();
                    if let Some(blob) = event.data() {
                        chunks_clone.push(&blob);
                    }
                },
            );
            recorder
                .set_ondataavailable(Some(on_data.as_ref().unchecked_ref()));
            on_data_js.set(Some(on_data.into_js_value()));

            recorder.start_with_time_slice(100).ok(); // collect chunks every 100ms

            // Store handles so stop_recording can access them.
            let handle = js_sys::Object::new();
            js_sys::Reflect::set(
                &handle,
                &JsValue::from_str("recorder"),
                &recorder.clone(),
            )
            .ok();
            js_sys::Reflect::set(&handle, &JsValue::from_str("stream"), &stream_js).ok();
            js_sys::Reflect::set(&handle, &JsValue::from_str("chunks"), &chunks).ok();
            handle_sig.set(Some(handle.into()));
            rec_sig.set(true);
        });
    };

    // Stops the active recording, assembles the chunks into OGG bytes,
    // and stores the result as a `SlideAudio` for the current slide.
    let stop_recording = move || {
        let idx = current_slide_clamped.get();
        let sa = slide_audios;
        let rec_sig = recording;
        let handle_sig = recorder_handle;

        let Some(handle) = handle_sig.get() else {
            return;
        };
        handle_sig.set(None);
        rec_sig.set(false);
        on_data_js.set(None);

        let recorder: web_sys::MediaRecorder =
            js_sys::Reflect::get(&handle, &JsValue::from_str("recorder"))
                .unwrap()
                .unchecked_into();
        let stream: web_sys::MediaStream =
            js_sys::Reflect::get(&handle, &JsValue::from_str("stream"))
                .unwrap()
                .unchecked_into();
        let chunks: js_sys::Array =
            js_sys::Reflect::get(&handle, &JsValue::from_str("chunks"))
                .unwrap()
                .unchecked_into();

        // Stop recording and release microphone.
        recorder.stop().ok();
        if let Ok(arr) = stream.get_tracks().dyn_into::<js_sys::Array>() {
            for i in 0..arr.length() {
                if let Ok(track) = arr.get(i).dyn_into::<web_sys::MediaStreamTrack>() {
                    track.stop();
                }
            }
        }

        // Assemble all chunks into a single Blob → ArrayBuffer → Vec<u8>.
        let blob_opts = web_sys::BlobPropertyBag::new();
        blob_opts.set_type("audio/ogg; codecs=opus");
        let blob = web_sys::Blob::new_with_buffer_source_sequence_and_options(
            &chunks,
            &blob_opts,
        )
        .ok();

        spawn_local(async move {
            let Some(blob) = blob else { return };
            let buf = wasm_bindgen_futures::JsFuture::from(blob.array_buffer())
                .await
                .ok();
            let Some(buf) = buf else { return };
            let bytes = js_sys::Uint8Array::new(&buf).to_vec();
            if bytes.is_empty() {
                return;
            }

            // Estimate duration: assume 48kHz Opus, ~12 bytes/ms at 64kbps.
            // Better: let the browser figure it out during playback.
            let duration_ms = (bytes.len() as u64 * 8 * 1000) / 64_000;

            let text = narrations.get().get(idx).cloned().unwrap_or_default();
            sa.update(|v| {
                if idx < v.len() {
                    v[idx] = Some(SlideAudio {
                        ogg_bytes: bytes,
                        duration_ms,
                        generated_from: crate::tts::hash_text(&text),
                    });
                }
            });
        });
    };

    // Raw Opus/OGG audio bytes for the current presentation (None = no audio loaded).
    let audio_data: RwSignal<Option<Vec<u8>>> = RwSignal::new(None);

    // Word count for the current slide's narration text.
    let word_count = Signal::derive(move || {
        let idx = current_slide_clamped.get();
        narrations.get().get(idx).map_or(0, |t| t.split_whitespace().count())
    });

    // ── Import .vecslide on mount ─────────────────────────────────────────
    // If the user opened a .vecslide file in Home, the bytes are in LoadedFile context.
    // Consume them here, populate source + narrations, then clear the context.
    let LoadedFile(loaded_file) = use_context::<LoadedFile>().expect("LoadedFile context missing");
    Effect::new(move |_| {
        if let Some(bytes) = loaded_file.get_untracked() {
            loaded_file.set(None); // consume immediately to free memory
            match import_vecslide_from_bytes(bytes) {
                Ok(state) => {
                    source.set(state.source);
                    narrations.set(state.narrations);
                    audio_data.set(state.audio);
                    slide_audios.set(state.slide_audios);
                    current_slide.set(0);
                }
                Err(e) => leptos::logging::error!("Failed to import .vecslide: {e}"),
            }
        }
    });

    // ── Preview state ──────────────────────────────────────────────────────
    let preview_svg = RwSignal::new(Option::<String>::None);
    let preview_error = RwSignal::new(Option::<String>::None);
    let compiling = RwSignal::new(false);

    // Lazy: FontAssets (Library::default + fonts) built on first compile, not at mount.
    // Avoids allocating the entire Typst stdlib (~30 MB) before the page renders.
    let font_assets: StoredValue<Option<FontAssets>> = StoredValue::new(None);

    // Debounce source changes — compile 150 ms after the last keystroke
    let ThemeColorsCtx(theme_sig) = use_context::<ThemeColorsCtx>().expect("ThemeColorsCtx missing");
    let debounced_source = RwSignal::new(source.get_untracked());
    let debounce_handle: RwSignal<Option<i32>> = RwSignal::new(None);

    Effect::new(move |_| {
        let src = source.get();
        let old_handle = debounce_handle.get_untracked();
        if let Some(h) = old_handle {
            web_sys::window().unwrap().clear_timeout_with_handle(h);
        }
        let cb = wasm_bindgen::closure::Closure::<dyn FnMut()>::new(move || {
            debounce_handle.set(None);
        });
        let window = web_sys::window().unwrap();
        if let Ok(h) = window.set_timeout_with_callback_and_timeout_and_arguments_0(
            cb.as_ref().unchecked_ref(),
            150,
        ) {
            cb.forget();
            debounce_handle.set(Some(h));
            debounced_source.set(src);
        } else {
            cb.forget();
            debounced_source.set(src);
        }
    });

    // Recompile whenever debounced source, slide index, or theme changes
    Effect::new(move |_| {
        let src = debounced_source.get();
        let idx = current_slide_clamped.get();
        let theme = theme_sig.get();
        let slide_src = src
            .split("\n----\n")
            .nth(idx)
            .unwrap_or("")
            .trim()
            .to_string();

        if slide_src.is_empty() {
            preview_svg.set(None);
            preview_error.set(None);
            compiling.set(false);
            return;
        }

        compiling.set(true);
        font_assets.update_value(|opt| {
            if opt.is_none() {
                *opt = Some(FontAssets::build());
            }
        });
        font_assets.with_value(|opt| {
            if let Some(assets) = opt {
                match compile_slide_to_svg(&slide_src, &theme, assets) {
                    Ok(svg) => {
                        preview_svg.set(Some(svg));
                        preview_error.set(None);
                    }
                    Err(msg) => {
                        preview_error.set(Some(msg));
                    }
                }
            }
        });
        compiling.set(false);
    });

    // ── Textarea scroll-to-active-slide ───────────────────────────────────
    let source_ref    = NodeRef::new();
    let narration_ref = NodeRef::new();

    // Left sidebar: scroll source textarea so the current slide is vertically centered.
    Effect::new(move |_| {
        let idx = current_slide_clamped.get();                // tracked
        let src = source.get_untracked();                     // untracked — typing must not re-trigger

        let Some(el): Option<web_sys::HtmlTextAreaElement> = source_ref.get() else { return };

        let separator    = "\n----\n";
        let slide_start: usize = src.split(separator).take(idx).map(|p| p.len() + separator.len()).sum();
        let slide_end: usize   = slide_start + src.split(separator).nth(idx).map_or(0, |p| p.len());
        let slide_start        = slide_start.min(src.len());
        let slide_end          = slide_end.min(src.len());

        let total_lines = (src.chars().filter(|&c| c == '\n').count() + 1) as i32;
        let line_start  = src[..slide_start].chars().filter(|&c| c == '\n').count() as i32;
        let line_end    = src[..slide_end  ].chars().filter(|&c| c == '\n').count() as i32;
        let mid_line    = (line_start + line_end) / 2;

        let scroll_h    = el.scroll_height();
        let client_h    = el.client_height();
        let target      = ((mid_line * scroll_h / total_lines.max(1)) - client_h / 2).max(0);
        el.set_scroll_top(f64::from(target));
    });

    // Right sidebar: scroll narration textarea to top on slide change.
    Effect::new(move |_| {
        current_slide_clamped.get();
        if let Some(el) = narration_ref.get().map(|e: web_sys::HtmlTextAreaElement| e) {
            el.set_scroll_top(0.0);
        }
    });

    // ── Navigation ─────────────────────────────────────────────────────────
    let navigate = use_navigate();
    let on_back = move |_| navigate("/", Default::default());

    let prev_disabled = Signal::derive(move || current_slide_clamped.get() == 0);
    let next_disabled = Signal::derive(move || {
        current_slide_clamped.get() >= slide_count.get().saturating_sub(1)
    });
    let on_prev = move |_: leptos::ev::MouseEvent| {
        current_slide.update(|v| *v = v.saturating_sub(1));
    };
    let on_next = move |_: leptos::ev::MouseEvent| {
        let max = slide_count.get().saturating_sub(1);
        current_slide.update(|v| *v = (*v + 1).min(max));
    };

    // Stop audio whenever the current slide changes (buttons, keyboard, carousel).
    Effect::new(move |prev: Option<usize>| {
        let slide = current_slide.get();
        if prev.is_some() && prev != Some(slide) {
            stop_audio();
        }
        slide
    });

    // Deferred playback request (used by auto-advance timeout to avoid
    // capturing the `start_slide_playback` closure inside a leaked Closure).
    Effect::new(move |_| {
        if let Some(idx) = play_request.get() {
            play_request.set(None);
            start_slide_playback(idx);
        }
    });

    // ── Export ─────────────────────────────────────────────────────────────
    let exporting = RwSignal::new(false);
    let saving    = RwSignal::new(false);
    let export_error = RwSignal::new(Option::<String>::None);

    let on_export = move |_: leptos::ev::MouseEvent| {
        exporting.set(true);
        export_error.set(None);

        let src = source.get();
        let narr = narrations.get();
        let audio = audio_data.get();
        let theme = theme_sig.get();
        font_assets.update_value(|opt| { if opt.is_none() { *opt = Some(FontAssets::build()); } });
        font_assets.with_value(|opt| {
            if let Some(assets) = opt {
                match export_html(&src, &narr, audio, assets, &theme) {
                    Ok(html) => trigger_download(&html, "presentation.html", "text/html"),
                    Err(e) => export_error.set(Some(e)),
                }
            }
        });

        exporting.set(false);
    };

    let on_save_vecslide = move |_: leptos::ev::MouseEvent| {
        saving.set(true);
        export_error.set(None);

        let src = source.get();
        let narr = narrations.get();
        let audio = audio_data.get();
        let audios = slide_audios.get();
        let theme = theme_sig.get();
        font_assets.update_value(|opt| { if opt.is_none() { *opt = Some(FontAssets::build()); } });
        font_assets.with_value(|opt| {
            if let Some(assets) = opt {
                match export_vecslide(&src, &narr, &audios, audio, assets, &theme) {
                    Ok(bytes) => {
                        trigger_download_binary(&bytes, "presentation.vecslide", "application/zip");
                    }
                    Err(e) => export_error.set(Some(e)),
                }
            }
        });

        saving.set(false);
    };

    // ── Sidebar / panel resize ────────────────────────────────────────────
    let left_width     = RwSignal::new(600.0f64);
    let left_collapsed = RwSignal::new(false);
    let bottom_height  = RwSignal::new(160.0f64);
    // drag_target: 0=none, 1=left handle, 2=bottom panel top edge
    // drag_start_x reused as "drag start position" (x or y), drag_start_w as "drag start size"
    let drag_target  = RwSignal::new(0i8);
    let drag_start_x = RwSignal::new(0.0f64);
    let drag_start_w = RwSignal::new(0.0f64);

    let on_mouse_move = move |ev: leptos::ev::MouseEvent| {
        match drag_target.get() {
            1 => {
                let dx = ev.client_x() - drag_start_x.get();
                left_width.set((drag_start_w.get() + dx).clamp(160.0, 600.0));
            }
            2 => {
                let dy = drag_start_x.get() - ev.client_y();
                bottom_height.set((drag_start_w.get() + dy).clamp(80.0, 500.0));
            }
            _ => {}
        }
    };
    let on_mouse_up = move |_: leptos::ev::MouseEvent| { drag_target.set(0); };

    // Keyboard navigation handler
    let on_keydown = move |ev: leptos::ev::KeyboardEvent| {
        // Don't intercept arrow keys while the user is typing in a textarea or input.
        let in_text_field = ev.target().map(|t| {
            t.dyn_ref::<web_sys::HtmlTextAreaElement>().is_some()
            || t.dyn_ref::<web_sys::HtmlInputElement>().is_some()
        }).unwrap_or(false);
        if in_text_field { return; }

        let count = slide_count.get();
        let max_idx = count.saturating_sub(1);
        let current = current_slide.get();

        match ev.key().as_str() {
            "ArrowLeft" | "ArrowUp" => {
                if current > 0 {
                    current_slide.set(current - 1);
                }
                ev.prevent_default();
            }
            "ArrowRight" | "ArrowDown" => {
                if current < max_idx {
                    current_slide.set(current + 1);
                }
                ev.prevent_default();
            }
            "Home" => {
                current_slide.set(0);
                ev.prevent_default();
            }
            "End" => {
                current_slide.set(max_idx);
                ev.prevent_default();
            }
            _ => {}
        }
    };

    view! {
        <div
            class="min-h-[100dvh] flex flex-col bg-base-200 overflow-hidden outline-none"
            style:user-select=move || if drag_target.get() != 0 { "none" } else { "auto" }
            tabindex="0"
            on:keydown=on_keydown
            on:mousemove=on_mouse_move
            on:mouseup=on_mouse_up
        >

            // ── Top toolbar ───────────────────────────────────────────────
            <header class="navbar bg-base-100 border-b border-base-300 px-4 py-2 shrink-0">
                <div class="flex-1 flex items-center gap-3">
                    <button
                        class="btn btn-ghost btn-sm btn-square"
                        on:click=on_back
                        aria-label="Back to home"
                        title="Back to home"
                    >
                        <ArrowLeft size=16 />
                    </button>
                    <div class="divider divider-horizontal m-0 h-5"></div>
                    <img
                        src="/vecslide-icon-color.svg"
                        alt="VecSlide"
                        class="h-6 w-auto select-none"
                        draggable="false"
                    />
                    <span class="font-semibold text-sm">"Editor"</span>
                    <span class="badge badge-ghost font-mono tabular-nums">
                        {move || {
                            let n = slide_count.get();
                            if n == 1 { "1 slide".to_string() } else { format!("{n} slides") }
                        }}
                    </span>
                    {move || compiling.get().then(|| view! {
                        <span class="loading loading-spinner loading-xs text-primary" aria-label="Compiling"></span>
                    })}
                </div>
                <div class="flex-none flex items-center gap-2">
                    // Export error (shown inline next to the button)
                    {move || export_error.get().map(|e| view! {
                        <span
                            class="text-xs text-error/80 max-w-xs truncate"
                            title=e.clone()
                        >{e.clone()}</span>
                    })}

                    // ── Voice picker dropdown ─────────────────────────
                    <select
                        class="select select-sm select-bordered w-40 text-xs"
                        title="Select TTS voice"
                        aria-label="Select TTS voice"
                        prop:disabled=move || tts_state.get().is_busy()
                        on:change=move |ev| {
                            let target = ev
                                .target()
                                .and_then(|t| t.dyn_into::<web_sys::HtmlSelectElement>().ok());
                            if let Some(el) = target {
                                selected_voice.set(el.value());
                            }
                        }
                    >
                        {move || {
                            let voices = available_voices.get();
                            let current = selected_voice.get();
                            if voices.is_empty() {
                                // Fallback: kokoro-js module not loaded yet
                                view! {
                                    <option value=DEFAULT_VOICE selected=true>
                                        {format_voice_label(DEFAULT_VOICE)}
                                    </option>
                                }.into_any()
                            } else {
                                voices.iter().map(|v| {
                                    let selected = *v == current;
                                    let val = v.clone();
                                    let label = format_voice_label(v);
                                    view! {
                                        <option value=val prop:selected=selected>
                                            {label}
                                        </option>
                                    }
                                }).collect_view().into_any()
                            }
                        }}
                    </select>

                    // ── Synthesize All button ─────────────────────────
                    <button
                        class="btn btn-secondary btn-sm gap-2"
                        prop:disabled=move || {
                            let all_empty = narrations
                                .get()
                                .iter()
                                .all(|t| t.trim().is_empty());
                            all_empty || tts_state.get().is_busy()
                        }
                        title="Synthesize speech for all slides via Kokoro-82M (first use downloads ~92 MB model)"
                        aria-label="Synthesize speech for all slides"
                        on:click=move |_| {
                            let narr = narrations.get();
                            let voice = selected_voice.get();
                            // Collect indices of slides that have non-empty narrations.
                            let to_synth: Vec<(usize, String)> = narr
                                .iter()
                                .enumerate()
                                .filter(|(_, t)| !t.trim().is_empty())
                                .map(|(i, t)| (i, t.clone()))
                                .collect();
                            if to_synth.is_empty() { return; }
                            let total = to_synth.len();
                            spawn_local(async move {
                                // Phase 1: ensure the Kokoro model is loaded.
                                let on_state = move |s: TtsState| tts_state.set(s);
                                if let Err(e) = ensure_model_loaded(on_state).await {
                                    let msg = e.as_string()
                                        .unwrap_or_else(|| format!("{e:?}"));
                                    leptos::logging::error!("kokoro load failed: {msg}");
                                    tts_state.set(TtsState::Error(msg));
                                    return;
                                }

                                // Phase 2: synthesize each slide sequentially.
                                for (seq, (idx, text)) in to_synth.into_iter().enumerate() {
                                    tts_state.set(TtsState::SynthesizingAll {
                                        current: seq + 1,
                                        total,
                                    });
                                    match synthesize_slide(&text, &voice).await {
                                        Ok(audio) => {
                                            slide_audios.update(|v| {
                                                if idx < v.len() { v[idx] = Some(audio); }
                                            });
                                        }
                                        Err(e) => {
                                            let msg = e.as_string()
                                                .unwrap_or_else(|| format!("{e:?}"));
                                            leptos::logging::error!("synth slide {idx} failed: {msg}");
                                            tts_state.set(TtsState::Error(msg));
                                            return;
                                        }
                                    }
                                }
                                tts_state.set(TtsState::Ready);
                            });
                        }
                    >
                        {move || {
                            match tts_state.get() {
                                TtsState::SynthesizingAll { current, total } => {
                                    view! {
                                        <span class="loading loading-spinner loading-xs" aria-hidden="true"></span>
                                        {format!("Synthesizing {current}/{total}...")}
                                    }.into_any()
                                }
                                TtsState::LoadingWasm { .. } | TtsState::LoadingWeights { .. } => {
                                    view! {
                                        <span class="loading loading-spinner loading-xs" aria-hidden="true"></span>
                                        "Loading model..."
                                    }.into_any()
                                }
                                _ => {
                                    view! {
                                        <AudioLines size=16 />
                                        "Synthesize All"
                                    }.into_any()
                                }
                            }
                        }}
                    </button>

                    <div class="divider divider-horizontal m-0 h-5"></div>

                    <button
                        class="btn btn-ghost btn-sm gap-2 border border-base-300"
                        on:click=on_save_vecslide
                        prop:disabled=move || saving.get()
                        title="Save as .vecslide (re-openable in the editor)"
                        aria-label="Save presentation as .vecslide"
                    >
                        {move || if saving.get() {
                            view! {
                                <span class="loading loading-spinner loading-xs" aria-hidden="true"></span>
                                "Saving..."
                            }.into_any()
                        } else {
                            view! {
                                <Save size=16 />
                                "Save .vecslide"
                            }.into_any()
                        }}
                    </button>
                    <button
                        class="btn btn-primary btn-sm gap-2"
                        on:click=on_export
                        prop:disabled=move || exporting.get()
                        title="Export all slides as a standalone HTML viewer"
                        aria-label="Export presentation as HTML"
                    >
                        {move || if exporting.get() {
                            view! {
                                <span class="loading loading-spinner loading-xs" aria-hidden="true"></span>
                                "Exporting..."
                            }.into_any()
                        } else {
                            view! {
                                <Download size=16 />
                                "Export HTML"
                            }.into_any()
                        }}
                    </button>
                </div>
            </header>

            // ── Body: sidebar + right column (preview + bottom panel) ───────
            <div class="flex flex-1 overflow-hidden">

                // ── Left: Typst source (full height) ───────────────────
                <div
                    class="flex flex-col bg-base-100 shrink-0 overflow-hidden border-r border-base-300/30"
                    style:width=move || if left_collapsed.get() { "32px".to_string() } else { format!("{}px", left_width.get()) }
                >
                    // Collapse toggle — always visible
                    <button
                        class="shrink-0 w-full h-7 flex items-center justify-end px-2 border-b border-base-300/30 text-base-content/60 hover:text-base-content/70 hover:bg-base-200/60 transition-colors text-base font-mono"
                        title=move || if left_collapsed.get() { "Expand" } else { "Collapse" }
                        on:click=move |_| left_collapsed.update(|c| *c = !*c)
                    >
                        {move || if left_collapsed.get() { "›" } else { "‹" }}
                    </button>

                    // Full content — hidden when collapsed
                    <div class=move || if left_collapsed.get() { "hidden" } else { "flex flex-col flex-1 overflow-hidden" }>
                        <div class="flex items-center justify-between px-4 py-2 border-b border-base-300/30 shrink-0">
                            <span class="text-xs text-base-content/60 font-mono uppercase tracking-wider">
                                "source .typ"
                            </span>
                            <div class="flex items-center gap-1">
                                <kbd class="kbd kbd-xs">"----"</kbd>
                                <span class="text-xs text-base-content/50 ml-1">"separates slides"</span>
                            </div>
                        </div>
                        <textarea
                            node_ref=source_ref
                            class="code-textarea flex-1 w-full bg-transparent px-4 py-4 scrollbar-thin text-base-content/90"
                            spellcheck="false"
                            autocomplete="off"
                            placeholder="= Title\n\nSlide content\n\n$ formula $\n\n----\n\n= Slide 2"
                            aria-label="Typst source"
                            prop:value=move || source.get()
                            on:input=move |ev| {
                                let target = ev
                                    .target()
                                    .and_then(|t| t.dyn_into::<web_sys::HtmlTextAreaElement>().ok());
                                if let Some(el) = target {
                                    source.set(el.value());
                                }
                            }
                        ></textarea>
                    </div>
                </div>

                // ── Left resize handle ──────────────────────────────────
                <div
                    class=move || if left_collapsed.get() {
                        "w-px shrink-0 bg-base-300/40"
                    } else {
                        "w-1 shrink-0 cursor-col-resize hover:bg-primary/50 active:bg-primary/70 transition-colors"
                    }
                    on:mousedown=move |ev: leptos::ev::MouseEvent| {
                        if !left_collapsed.get() {
                            ev.prevent_default();
                            drag_target.set(1);
                            drag_start_x.set(ev.client_x());
                            drag_start_w.set(left_width.get());
                        }
                    }
                ></div>

                // ── Right column: preview + bottom panel ─────────────────
                <div class="flex-1 flex flex-col overflow-hidden">

                // ── Right: SVG preview ──────────────────────────────────
                <div class="flex-1 flex flex-col bg-base-200 overflow-hidden">

                    // SVG preview area — container-query context for aspect-ratio filling
                    <div class="slide-preview-area flex-1 flex items-center justify-center p-4 overflow-hidden bg-base-300/20">
                        {move || {
                            if let Some(error) = preview_error.get() {
                                view! {
                                    <div class="slide-preview card-surface rounded-2xl flex flex-col items-center justify-center gap-3 p-8 border border-error/30">
                                        <div class="w-10 h-10 text-error/60">
                                            <CircleAlert size=40 stroke_width=2 />
                                        </div>
                                        <p class="text-error/80 text-sm font-mono whitespace-pre-wrap text-center max-w-lg">
                                            {error}
                                        </p>
                                    </div>
                                }.into_any()
                            } else if let Some(svg) = preview_svg.get() {
                                view! {
                                    <div
                                        class="slide-preview rounded-2xl overflow-hidden bg-base-200"
                                        inner_html=svg
                                        aria-label=move || format!("Slide {} preview", current_slide_clamped.get() + 1)
                                    ></div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="slide-preview card-surface rounded-2xl flex flex-col items-center justify-center gap-3">
                                        <span class="loading loading-spinner loading-lg text-primary" aria-label="Compiling"></span>
                                        <p class="text-sm text-base-content/50">"Write Typst source to see a preview"</p>
                                    </div>
                                }.into_any()
                            }
                        }}
                    </div>

                    // ── Slide nav strip (bottom carousel) ──────────────────
                    // Each button: w-9 (36px) + gap-2 (8px) = 44px/item.
                    // Center of item i = i*44 + 18 from inner-flex left edge.
                    // Inner flex: left:50% → left edge at container center.
                    // translateX(-(active*44+18)) → active item centered.
                    <nav
                        class="flex items-center gap-2 px-4 py-2 border-t border-base-300/30 bg-base-100/40 shrink-0"
                        aria-label="Slide navigation"
                    >
                        <span class="text-xs text-base-content/60 font-mono uppercase tracking-wider shrink-0">
                            "preview"
                        </span>
                        <button
                            class="btn btn-ghost btn-xs btn-square shrink-0 hover:bg-primary/10"
                            prop:disabled=prev_disabled
                            on:click=on_prev
                            aria-label="Previous slide"
                            title="Previous"
                        >
                            <ChevronLeft size=16 />
                        </button>

                        // Carousel viewport
                        <div class="flex-1 relative overflow-hidden h-7">
                            <div
                                class="flex gap-2 items-center absolute inset-y-0"
                                style:left="50%"
                                style:transition="transform 220ms ease"
                                style:transform=move || {
                                    let offset = current_slide_clamped.get() * 44 + 18;
                                    format!("translateX(-{offset}px)")
                                }
                            >
                                {move || {
                                    let count = slide_count.get();
                                    let active = current_slide_clamped.get();
                                    (0..count)
                                        .map(|i| {
                                            let is_active = i == active;
                                            let class = if is_active {
                                                "slide-tab-active w-9 h-6 flex items-center justify-center rounded-lg text-base font-bold font-mono border shrink-0 transition-colors"
                                            } else {
                                                "w-9 h-6 flex items-center justify-center rounded-lg text-sm font-mono border border-base-300/40 shrink-0 transition-colors hover:border-primary/40 text-base-content/50"
                                            };
                                            view! {
                                                <button
                                                    class=class
                                                    on:click=move |_| current_slide.set(i)
                                                    aria-label=format!("Slide {}", i + 1)
                                                    aria-current=if is_active { "true" } else { "false" }
                                                >
                                                    {i + 1}
                                                </button>
                                            }
                                        })
                                        .collect_view()
                                }}
                            </div>
                        </div>

                        <button
                            class="btn btn-ghost btn-xs btn-square shrink-0 hover:bg-primary/10"
                            prop:disabled=next_disabled
                            on:click=on_next
                            aria-label="Next slide"
                            title="Next"
                        >
                            <ChevronRight size=16 />
                        </button>
                    </nav>
                </div>

                // ── Bottom panel: unified Notes + Audio ────────────────────
                <div
                    class="shrink-0 flex flex-col bg-base-100 border-t border-base-300/40"
                    style:height=move || format!("{}px", bottom_height.get())
                >
                    // Top drag handle for vertical resize
                    <div
                        class="h-1 shrink-0 cursor-row-resize hover:bg-primary/50 active:bg-primary/70 transition-colors"
                        on:mousedown=move |ev: leptos::ev::MouseEvent| {
                            ev.prevent_default();
                            drag_target.set(2);
                            drag_start_x.set(ev.client_y());
                            drag_start_w.set(bottom_height.get());
                        }
                    ></div>

                    // ── Status bar ────────────────────────────────────────
                    <div class="flex items-center gap-2 px-4 py-1.5 border-b border-base-300/30 shrink-0 text-xs font-mono text-base-content/60">
                        <span>{move || format!("Slide {}/{}", current_slide_clamped.get() + 1, slide_count.get())}</span>
                        <span>"·"</span>
                        <span>{move || format!("{} words", word_count.get())}</span>
                        <span>"·"</span>
                        {move || {
                            let idx = current_slide_clamped.get();
                            let has_audio = slide_audios
                                .get()
                                .get(idx)
                                .is_some_and(|o| o.is_some());
                            if has_audio {
                                let dur = slide_audios
                                    .get()
                                    .get(idx)
                                    .and_then(|o| o.as_ref().map(|a| a.duration_ms))
                                    .unwrap_or(0);
                                let secs = dur / 1000;
                                let m = secs / 60;
                                let s = secs % 60;
                                view! {
                                    <span class="text-primary/60">
                                        {format!("{m}:{s:02}")}
                                    </span>
                                }.into_any()
                            } else {
                                view! {
                                    <span class="text-base-content/50">"no audio"</span>
                                }.into_any()
                            }
                        }}
                        // Stale badge
                        <span class=move || {
                            let idx = current_slide_clamped.get();
                            let text = narrations.get().get(idx).cloned().unwrap_or_default();
                            let is_stale = slide_audios
                                .get()
                                .get(idx)
                                .and_then(|o| o.as_ref().map(|a| a.is_stale(&text)))
                                .unwrap_or(false);
                            if is_stale {
                                "badge badge-warning badge-xs gap-1"
                            } else {
                                "hidden"
                            }
                        }
                        title="Text changed since the audio was synthesized — regenerate to update">
                            "stale"
                        </span>
                    </div>

                    // ── Content: notes + audio stacked ────────────────────
                    <div class="flex-1 flex flex-col overflow-hidden min-h-0">

                        // Notes textarea (always visible, takes remaining space)
                        <textarea
                            node_ref=narration_ref
                            class="flex-1 w-full bg-transparent px-4 py-2 text-sm text-base-content/80 resize-none scrollbar-thin min-h-0"
                            placeholder="Spoken text for this slide..."
                            aria-label="Speaker notes for the current slide"
                            prop:value=move || {
                                let idx = current_slide_clamped.get();
                                narrations.get().get(idx).cloned().unwrap_or_default()
                            }
                            on:input=move |ev| {
                                let idx = current_slide_clamped.get();
                                let target = ev
                                    .target()
                                    .and_then(|t| t.dyn_into::<web_sys::HtmlTextAreaElement>().ok());
                                if let Some(el) = target {
                                    narrations.update(|v| {
                                        if idx < v.len() { v[idx] = el.value(); }
                                    });
                                }
                            }
                        ></textarea>

                        // ── Audio section (always visible, compact) ───────
                        <div class="shrink-0 border-t border-base-300/20 px-4 py-2">

                            // ── State A: Empty (no audio, not recording) ──
                            <div class=move || {
                                let idx = current_slide_clamped.get();
                                let has_audio = slide_audios.get().get(idx).is_some_and(|o| o.is_some());
                                if !has_audio && !recording.get() {
                                    "flex items-center gap-3"
                                } else {
                                    "hidden"
                                }
                            }>
                                <span class="text-base-content/60"><Mic size=16 /></span>
                                <button
                                    class="btn btn-outline btn-xs gap-1"
                                    on:click=move |_| {
                                        start_recording();
                                    }
                                >
                                    <Mic size=12 />
                                    "Record audio"
                                </button>
                                <span class="text-base-content/60">"or"</span>
                                <label class="btn btn-ghost btn-xs gap-1 cursor-pointer">
                                    <Plus size=12 />
                                    "Add from file"
                                    <input
                                        type="file"
                                        accept=".ogg,audio/ogg"
                                        class="hidden"
                                        on:change=move |ev| {
                                            use wasm_bindgen_futures::spawn_local;
                                            use js_sys::Uint8Array;
                                            let input = ev
                                                .target()
                                                .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok());
                                            if let Some(input) = input
                                                && let Some(file) = input.files().and_then(|f| f.get(0))
                                            {
                                                let idx = current_slide_clamped.get();
                                                let sa = slide_audios;
                                                let narr = narrations;
                                                spawn_local(async move {
                                                    let promise = file.array_buffer();
                                                    if let Ok(buf) = wasm_bindgen_futures::JsFuture::from(promise).await {
                                                        let bytes = Uint8Array::new(&buf).to_vec();
                                                        let duration_ms = (bytes.len() as u64 * 8 * 1000) / 64_000;
                                                        let text = narr.get().get(idx).cloned().unwrap_or_default();
                                                        sa.update(|v| {
                                                            if idx < v.len() {
                                                                v[idx] = Some(SlideAudio {
                                                                    ogg_bytes: bytes,
                                                                    duration_ms,
                                                                    generated_from: crate::tts::hash_text(&text),
                                                                });
                                                            }
                                                        });
                                                    }
                                                });
                                            }
                                        }
                                    />
                                </label>
                            </div>

                            // ── State B: Recording ─────────────────────────
                            <div class=move || if recording.get() {
                                "flex items-center gap-3"
                            } else {
                                "hidden"
                            }>
                                <div class="text-error animate-pulse">
                                    <Mic size=16 />
                                </div>
                                <span class="text-sm text-error font-medium">
                                    "Recording slide "{move || current_slide_clamped.get() + 1}"..."
                                </span>
                                <button
                                    class="btn btn-error btn-xs gap-1"
                                    on:click=move |_| stop_recording()
                                >
                                    "Stop"
                                </button>
                            </div>

                            // ── State C: Audio present (player + actions) ──
                            <div class=move || {
                                let idx = current_slide_clamped.get();
                                let has_audio = slide_audios.get().get(idx).is_some_and(|o| o.is_some());
                                if has_audio {
                                    "flex items-center gap-2"
                                } else {
                                    "hidden"
                                }
                            }>
                                // Play / Pause button
                                <button
                                    class="btn btn-ghost btn-xs btn-square"
                                    title=move || match playback_state.get() {
                                        PlaybackState::Playing => "Pause",
                                        PlaybackState::Paused => "Resume",
                                        PlaybackState::Idle => "Play",
                                    }
                                    on:click=move |_| {
                                        match playback_state.get() {
                                            PlaybackState::Idle => {
                                                let idx = current_slide_clamped.get();
                                                start_slide_playback(idx);
                                            }
                                            PlaybackState::Playing => {
                                                if let Some(el) = active_audio.get() {
                                                    let _ = el.pause();
                                                }
                                                playback_state.set(PlaybackState::Paused);
                                            }
                                            PlaybackState::Paused => {
                                                if let Some(el) = active_audio.get() {
                                                    let _ = el.play();
                                                }
                                                playback_state.set(PlaybackState::Playing);
                                            }
                                        }
                                    }
                                >
                                    {move || match playback_state.get() {
                                        PlaybackState::Playing => view! {
                                            <Pause size=14 />
                                        }.into_any(),
                                        _ => view! {
                                            <Play size=14 />
                                        }.into_any(),
                                    }}
                                </button>

                                // Mini progress bar
                                <div class="flex-1 h-2 bg-base-300/40 rounded overflow-hidden">
                                    <div
                                        class="h-full bg-primary/50 rounded transition-all"
                                        style:width=move || {
                                            let count = slide_count.get();
                                            if count == 0 { return "0%".to_string(); }
                                            format!("{:.1}%", ((current_slide_clamped.get() + 1) as f64 / count as f64) * 100.0)
                                        }
                                    ></div>
                                </div>

                                // Duration
                                <span class="text-xs font-mono text-base-content/60 shrink-0">
                                    {move || {
                                        let idx = current_slide_clamped.get();
                                        let dur = slide_audios
                                            .get()
                                            .get(idx)
                                            .and_then(|o| o.as_ref().map(|a| a.duration_ms))
                                            .unwrap_or(0);
                                        let secs = dur / 1000;
                                        let m = secs / 60;
                                        let s = secs % 60;
                                        format!("{m}:{s:02}")
                                    }}
                                </span>

                                // Remove
                                <button
                                    class="btn btn-ghost btn-xs gap-1 text-error/60 hover:text-error"
                                    title="Remove audio from this slide"
                                    on:click=move |_| {
                                        let idx = current_slide_clamped.get();
                                        stop_audio();
                                        slide_audios.update(|v| {
                                            if idx < v.len() { v[idx] = None; }
                                        });
                                    }
                                >
                                    <Trash2 size=12 />
                                    "Remove"
                                </button>

                                // Auto-advance toggle
                                <label
                                    class="label cursor-pointer gap-1 flex items-center"
                                    title="Auto-advance to next slide after audio ends"
                                >
                                    <input
                                        type="checkbox"
                                        class="toggle toggle-xs toggle-primary"
                                        prop:checked=move || auto_advance_enabled.get()
                                        on:change=move |ev| {
                                            let checked = ev
                                                .target()
                                                .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
                                                .map(|el| el.checked())
                                                .unwrap_or(false);
                                            auto_advance_enabled.set(checked);
                                        }
                                    />
                                    <span class="text-xs text-base-content/70">Auto</span>
                                </label>
                            </div>

                        </div>
                    </div>

                </div>
                // end right column
                </div>

            </div>
        </div>
    }
}
