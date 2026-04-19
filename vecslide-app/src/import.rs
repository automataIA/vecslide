//! Import and export of the `.vecslide` format from/to in-memory bytes.
//!
//! These functions are WASM-safe — they never touch the filesystem.
//! ZIP I/O goes through `std::io::Cursor<Vec<u8>>`.

use std::{collections::HashMap, io::Cursor};

use vecslide_core::{
    UnpackedPresentation,
    manifest::{Presentation, Slide, Transcript, TranscriptMode, TranscriptSegment},
    pack_to_writer,
    theme::ThemeColors,
    unpack_from_reader,
};

use crate::tts::{SlideAudio, concat_ogg_streams, hash_text};
use crate::typst_world::{FontAssets, compile_slide_to_svg};

// ─── Import ──────────────────────────────────────────────────────────────────

/// State extracted from a `.vecslide` archive — enough to restore the editor.
pub struct ImportedState {
    /// Multi-slide Typst source separated by `"\n----\n"`.
    pub source: String,
    /// Narration text, one entry per slide (may be empty strings).
    pub narrations: Vec<String>,
    /// Raw Opus/OGG audio bytes, if the archive contained a **master** track
    /// (legacy import flow: imported `.ogg`). This is separate from
    /// `slide_audios` which holds per-slide synthesized audio.
    pub audio: Option<Vec<u8>>,
    /// Per-slide synthesized audio, recovered from `audio/slide_*.ogg` files
    /// in the archive. Same length as `narrations`; `None` = no audio for
    /// that slide. If no per-slide files are present, every entry is `None`.
    pub slide_audios: Vec<Option<SlideAudio>>,
}

/// Deserialises a `.vecslide` byte buffer into editor state.
///
/// - Recovers the Typst source from `extra_files["source.typ"]`.
/// - Falls back to `typst_inline` per slide if `typst_source` is absent
///   (compatibility with files authored without the round-trip field).
/// - Recovers narrations from `manifest.transcript.segments`.
pub fn import_vecslide_from_bytes(bytes: Vec<u8>) -> Result<ImportedState, String> {
    let unpacked = unpack_from_reader(Cursor::new(bytes))
        .map_err(|e| format!("Error opening .vecslide: {e}"))?;

    // ── Recover Typst source ─────────────────────────────────────────────────
    let source = if let Some(ref path) = unpacked.manifest.typst_source {
        unpacked
            .extra_files
            .get(path.as_str())
            .and_then(|b| std::str::from_utf8(b).ok())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                format!(
                    "The manifest declares typst_source = \"{path}\" but the file is not in the archive"
                )
            })?
    } else {
        // Fallback: reconstruct from per-slide typst_inline
        unpacked
            .manifest
            .slides
            .iter()
            .map(|s| s.typst_inline.clone().unwrap_or_default())
            .collect::<Vec<_>>()
            .join("\n----\n")
    };

    // ── Recover narrations from transcript ───────────────────────────────────
    let slide_count = unpacked.manifest.slides.len();
    let mut narrations = vec![String::new(); slide_count];

    if let Some(ref transcript) = unpacked.manifest.transcript {
        for seg in &transcript.segments {
            if let Some(ref slide_id) = seg.slide_ref
                && let Some(idx) = unpacked.manifest.slides.iter().position(|s| &s.id == slide_id)
            {
                if narrations[idx].is_empty() {
                    narrations[idx] = seg.text.clone();
                } else {
                    narrations[idx].push(' ');
                    narrations[idx].push_str(&seg.text);
                }
            }
        }
    }

    // ── Recover per-slide synthesized audio ─────────────────────────────────
    // Path convention written by `export_vecslide`: `audio/slide_{id}.ogg`.
    // Duration is taken from the Slide's `time_start` delta to the next slide
    // (last slide falls back to the remaining transcript-estimated duration).
    let mut slide_audios: Vec<Option<SlideAudio>> = vec![None; slide_count];
    for (i, slide) in unpacked.manifest.slides.iter().enumerate() {
        let path = format!("audio/{}.ogg", slide.id);
        if let Some(bytes) = unpacked.extra_files.get(path.as_str()) {
            // Duration: prefer delta to next slide's `time_start`; otherwise 0
            // (caller may still play via the <audio> element which knows the
            // real duration from the OGG itself).
            let duration_ms = unpacked
                .manifest
                .slides
                .get(i + 1)
                .map(|next| next.time_start.saturating_sub(slide.time_start))
                .unwrap_or(0);
            slide_audios[i] = Some(SlideAudio {
                ogg_bytes: bytes.clone(),
                duration_ms,
                // We don't know the original text that produced this audio,
                // so we snapshot the current narration. That means a
                // re-imported slide is not marked stale until the user edits.
                generated_from: hash_text(narrations.get(i).map(String::as_str).unwrap_or("")),
            });
        }
    }

    let audio = if unpacked.audio.is_empty() { None } else { Some(unpacked.audio) };
    Ok(ImportedState { source, narrations, audio, slide_audios })
}

// ─── Export ──────────────────────────────────────────────────────────────────

/// Compiles a `.vecslide` archive in memory from the current editor state.
///
/// - Compiles every slide to SVG (same as HTML export, light theme).
/// - Stores the raw Typst source as `source.typ` in `extra_files` for
///   round-trip editing.
/// - Narrations are saved in `manifest.transcript.segments`.
/// - **Per-slide audio** (from `slide_audios`): each synthesized OGG is
///   stored at `audio/{slide_id}.ogg`, and a master `audio/voce.ogg` is
///   built by concatenating the per-slide packets via [`concat_ogg_streams`],
///   with slide boundaries flushed to page boundaries for precise seeking.
/// - If `slide_audios` is all `None`, falls back to the legacy master-only
///   `audio` parameter (imported `.ogg` from the Audio Timeline tab).
pub fn export_vecslide(
    source: &str,
    narrations: &[String],
    slide_audios: &[Option<SlideAudio>],
    legacy_audio: Option<Vec<u8>>,
    font_assets: &FontAssets,
    theme: &ThemeColors,
) -> Result<Vec<u8>, String> {
    let slide_sources: Vec<&str> = source.split("\n----\n").collect();
    let total = slide_sources.len();

    let mut svgs: HashMap<String, Vec<u8>> = HashMap::new();
    let mut slides: Vec<Slide> = Vec::with_capacity(total);

    for (i, slide_src) in slide_sources.iter().enumerate() {
        let svg_path = format!("vector_assets/slide_{:02}.svg", i + 1);
        let svg = compile_slide_to_svg(slide_src.trim(), theme, font_assets)
            .map_err(|e| format!("Slide {}/{total}: {e}", i + 1))?;

        svgs.insert(svg_path.clone(), svg.into_bytes());
        slides.push(Slide {
            id: format!("slide_{:02}", i + 1),
            svg_file: Some(svg_path),
            typst_file: None,
            typst_inline: None,
            time_start: 0,
            animations: vec![],
            pointer_trail: None,
            transition: None,
        });
    }

    // ── Per-slide audio: build master, compute cumulative time_start ─────────
    let has_synth = slide_audios.iter().any(|o| o.is_some());
    let mut extra_files: HashMap<String, Vec<u8>> = HashMap::new();
    extra_files.insert("source.typ".to_string(), source.as_bytes().to_vec());

    // Master audio + audio_track path populated below based on which source
    // (synth per-slide vs legacy imported .ogg) is in use.
    let mut master_audio: Vec<u8> = Vec::new();
    let mut audio_track: Option<String> = None;

    if has_synth {
        // Write per-slide OGG files into extra_files for round-trip recovery.
        for (i, entry) in slide_audios.iter().enumerate() {
            if let (Some(audio), Some(slide)) = (entry.as_ref(), slides.get(i)) {
                let path = format!("audio/{}.ogg", slide.id);
                extra_files.insert(path, audio.ogg_bytes.clone());
            }
        }

        // Cumulative `time_start`: each slide starts at the sum of previous
        // slide durations. Slides without synth audio contribute 0 duration.
        let mut cursor_ms: u64 = 0;
        for (i, slide) in slides.iter_mut().enumerate() {
            slide.time_start = cursor_ms;
            if let Some(Some(audio)) = slide_audios.get(i) {
                cursor_ms = cursor_ms.saturating_add(audio.duration_ms);
            }
        }

        // Concat the non-None per-slide OGG streams into the master.
        let owned: Vec<&[u8]> = slide_audios
            .iter()
            .filter_map(|o| o.as_ref().map(|a| a.ogg_bytes.as_slice()))
            .collect();
        master_audio = concat_ogg_streams(
            &owned,
            1,
            crate::tts::config::SAMPLE_RATE_HZ,
        )
        .map_err(|e| format!("Master OGG concat: {e}"))?;
        audio_track = Some("audio/voce.ogg".to_string());
    } else if let Some(bytes) = legacy_audio.as_ref()
        && !bytes.is_empty()
    {
        master_audio = bytes.clone();
        audio_track = Some("audio.ogg".to_string());
    }

    // ── Build transcript from narrations ─────────────────────────────────────
    // Duration is estimated from word count: 150 WPM (presentation pace), min 2 s per segment.
    const WPM: u64 = 150;
    const MIN_DURATION_MS: u64 = 2_000;
    let transcript = if narrations.iter().any(|n| !n.is_empty()) {
        let mut segments: Vec<TranscriptSegment> = Vec::new();
        let mut cursor_ms: u64 = 0;
        for (i, slide) in slides.iter().enumerate() {
            let text = narrations.get(i).cloned().unwrap_or_default();
            if text.is_empty() { continue; }
            let word_count = text.split_whitespace().count() as u64;
            let duration_ms = (word_count * 60_000 / WPM).max(MIN_DURATION_MS);
            segments.push(TranscriptSegment {
                start_ms: cursor_ms,
                end_ms: cursor_ms + duration_ms,
                text,
                slide_ref: Some(slide.id.clone()),
                words: vec![],
            });
            cursor_ms += duration_ms;
        }
        Some(Transcript {
            mode: TranscriptMode::Standalone,
            language: "en".to_string(),
            segments,
        })
    } else {
        None
    };

    let manifest = Presentation {
        format_version: "1.0".to_string(),
        title: "Untitled".to_string(),
        author: None,
        description: None,
        date: None,
        language: None,
        audio_track,
        typst_source: Some("source.typ".to_string()),
        slides,
        annotations: vec![],
        transcript,
    };

    let unpacked = UnpackedPresentation {
        manifest,
        svgs,
        extra_files,
        audio: master_audio,
        theme_css: None,
    };

    // ── Pack to in-memory ZIP ─────────────────────────────────────────────────
    let mut buf = Cursor::new(Vec::new());
    pack_to_writer(&unpacked, &mut buf).map_err(|e| format!("Error creating .vecslide: {e}"))?;
    Ok(buf.into_inner())
}
