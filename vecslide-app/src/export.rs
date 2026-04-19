use std::collections::HashMap;

use js_sys::{Array, Uint8Array};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, Url};

use vecslide_core::{
    UnpackedPresentation,
    compile_html::compile_html,
    manifest::{Presentation, Slide, Transcript, TranscriptMode, TranscriptSegment},
    theme::ThemeColors,
};

use crate::typst_world::{FontAssets, compile_slide_to_svg};

/// Compiles every slide in `source` to SVG, wraps them in an `UnpackedPresentation`,
/// then calls `vecslide_core::compile_html` to produce a self-contained HTML viewer.
///
/// Returns the HTML string or a human-readable error describing which slide failed.
pub fn export_html(source: &str, narrations: &[String], audio: Option<Vec<u8>>, font_assets: &FontAssets, theme: &ThemeColors) -> Result<String, String> {
    let slide_sources: Vec<&str> = source.split("\n----\n").collect();
    let total = slide_sources.len();

    let mut svgs: HashMap<String, Vec<u8>> = HashMap::new();
    let mut slides: Vec<Slide> = Vec::with_capacity(total);

    for (i, slide_src) in slide_sources.iter().enumerate() {
        let svg_path = format!("vector_assets/slide_{:02}.svg", i + 1);
        let svg = compile_slide_to_svg(slide_src.trim(), theme, font_assets)
            .map_err(|e| format!("Slide {}/{}: {}", i + 1, total, e))?;

        svgs.insert(svg_path.clone(), svg.into_bytes());
        slides.push(Slide {
            id: format!("slide_{:02}", i + 1),
            svg_file: Some(svg_path),
            typst_file: None,
            typst_inline: None,
            // No audio: time_start is 0 for every slide; the viewer uses static navigation.
            time_start: 0,
            animations: vec![],
            pointer_trail: None,
            transition: None,
        });
    }

    // Build a transcript if any narration text was provided.
    // Duration is estimated from word count: 150 WPM (presentation pace), min 2 s per segment.
    const WPM: u64 = 150;
    const MIN_DURATION_MS: u64 = 2_000;
    let transcript = if narrations.iter().any(|n| !n.is_empty()) {
        let mut segments: Vec<TranscriptSegment> = Vec::new();
        let mut cursor_ms: u64 = 0;
        for (i, slide) in slides.iter_mut().enumerate() {
            let text = narrations.get(i).cloned().unwrap_or_default();
            // Set slide time_start so the viewer can sync slides to audio.
            slide.time_start = cursor_ms;
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
        Some(Transcript { mode: TranscriptMode::Standalone, language: "en".to_string(), segments })
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
        audio_track: if audio.is_some() { Some("audio.ogg".to_string()) } else { None },
        typst_source: None,
        slides,
        annotations: vec![],
        transcript,
    };

    let unpacked = UnpackedPresentation {
        manifest,
        svgs,
        extra_files: HashMap::new(),
        audio: audio.unwrap_or_default(),
        theme_css: Some(theme.to_viewer_css()),
    };

    compile_html(&unpacked).map_err(|e| e.to_string())
}

/// Triggers a browser file-download of binary `data` using a temporary Blob URL.
///
/// Use this for non-UTF-8 content such as ZIP archives (`.vecslide`).
pub fn trigger_download_binary(data: &[u8], filename: &str, mime: &str) {
    let window = web_sys::window().expect("no global window");
    let document = window.document().expect("no document on window");

    // Copy bytes into a JS Uint8Array, then use its underlying ArrayBuffer as Blob source.
    let uint8 = Uint8Array::new_with_length(data.len() as u32);
    uint8.copy_from(data);

    let parts = Array::new();
    parts.push(&uint8.buffer());
    let opts = BlobPropertyBag::new();
    opts.set_type(mime);
    let blob = Blob::new_with_buffer_source_sequence_and_options(&parts, &opts)
        .expect("Blob::new failed");

    let url = Url::create_object_url_with_blob(&blob).expect("createObjectURL failed");

    let a: HtmlAnchorElement = document
        .create_element("a")
        .expect("createElement('a') failed")
        .dyn_into()
        .expect("dyn_into HtmlAnchorElement failed");
    a.set_href(&url);
    a.set_download(filename);

    let body = document.body().expect("document has no <body>");
    body.append_child(&a).expect("append_child failed");
    a.click();
    body.remove_child(&a).expect("remove_child failed");

    Url::revoke_object_url(&url).expect("revokeObjectURL failed");
}

/// Triggers a browser file-download of `content` using a temporary Blob URL.
///
/// Creates a hidden `<a download>` element, clicks it, then immediately cleans up.
pub fn trigger_download(content: &str, filename: &str, mime: &str) {
    let window = web_sys::window().expect("no global window");
    let document = window.document().expect("no document on window");

    // Build the Blob from the string content.
    let parts = Array::new();
    parts.push(&JsValue::from_str(content));
    let opts = BlobPropertyBag::new();
    opts.set_type(mime);
    let blob = Blob::new_with_str_sequence_and_options(&parts, &opts)
        .expect("Blob::new failed");

    // Create an object URL pointing to the Blob.
    let url = Url::create_object_url_with_blob(&blob).expect("createObjectURL failed");

    // Inject a hidden <a download="..."> and programmatically click it.
    let a: HtmlAnchorElement = document
        .create_element("a")
        .expect("createElement('a') failed")
        .dyn_into()
        .expect("dyn_into HtmlAnchorElement failed");
    a.set_href(&url);
    a.set_download(filename);

    let body = document.body().expect("document has no <body>");
    body.append_child(&a).expect("append_child failed");
    a.click();
    body.remove_child(&a).expect("remove_child failed");

    // Release the object URL immediately — the download has already started.
    Url::revoke_object_url(&url).expect("revokeObjectURL failed");
}
