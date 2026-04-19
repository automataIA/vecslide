use std::collections::HashMap;

use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use serde::Serialize;
use tracing::instrument;

use crate::{
    error::VecslideError,
    manifest::{Animation, PointerTrail, Presentation, Slide, Transcript, TransitionKind},
    player_template::TEMPLATE,
};

/// Default theme CSS for the viewer when no explicit theme is provided (dark theme).
const DEFAULT_THEME_CSS: &str = "\
--vs-primary: #1c4f82;\n  \
--vs-primary-content: #d6e4f0;\n  \
--vs-secondary: #7c3aed;\n  \
--vs-accent: #e68a00;\n  \
--vs-neutral: #23282f;\n  \
--vs-neutral-content: #a6adba;\n  \
--vs-base-100: #1f2937;\n  \
--vs-base-200: #111827;\n  \
--vs-base-300: #0f1623;\n  \
--vs-base-content: #d5d9de;\n  \
--vs-error: #f87272;";

/// Input to the HTML compiler. Contains the already-decoded binary content of a .vecslide.
pub struct UnpackedPresentation {
    pub manifest: Presentation,
    /// Maps svg_file path (as in manifest) to raw SVG UTF-8 bytes.
    pub svgs: HashMap<String, Vec<u8>>,
    /// All other files in the archive, keyed by path (used for typst_file lookups).
    pub extra_files: HashMap<String, Vec<u8>>,
    /// Raw audio bytes (Opus/OGG).
    pub audio: Vec<u8>,
    /// CSS custom properties for viewer theming. If `None`, uses dark-theme defaults.
    pub theme_css: Option<String>,
}

impl UnpackedPresentation {
    /// Reads a file from the archive by path (checks SVGs first, then extra_files).
    pub fn read_file_str(&self, path: &str) -> Result<String, VecslideError> {
        let bytes = self
            .svgs
            .get(path)
            .or_else(|| self.extra_files.get(path))
            .ok_or_else(|| VecslideError::MissingSvgFile { path: path.to_string() })?;
        std::str::from_utf8(bytes)
            .map(|s| s.to_string())
            .map_err(|e| VecslideError::Other(format!("file '{path}' is not valid UTF-8: {e}")))
    }
}

/// Viewer-only manifest: mirrors Presentation but excludes annotation data.
/// This struct is what gets serialized into the compiled HTML — provably annotation-free.
#[derive(Serialize)]
struct ViewerManifest<'a> {
    title: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    author: Option<&'a str>,
    /// `None` signals Light mode: no audio, viewer uses TTS instead.
    #[serde(skip_serializing_if = "Option::is_none")]
    audio_track: Option<&'a str>,
    total_duration_ms: u64,
    slides: Vec<ViewerSlide<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    transcript: Option<&'a Transcript>,
}

#[derive(Serialize)]
struct ViewerSlide<'a> {
    id: &'a str,
    time_start: u64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    animations: &'a Vec<Animation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pointer_trail: Option<&'a PointerTrail>,
    #[serde(skip_serializing_if = "Option::is_none")]
    transition: Option<&'a TransitionKind>,
}

/// Compiles an unpacked presentation into a single self-contained HTML file.
/// The result can be saved as `.html` and opened in any modern browser without a server.
///
/// Annotations are intentionally excluded from the output.
#[instrument(skip(unpacked), fields(title = %unpacked.manifest.title, slides = unpacked.manifest.slides.len()))]
pub fn compile_html(unpacked: &UnpackedPresentation) -> Result<String, VecslideError> {
    let pres = &unpacked.manifest;

    // Build the viewer manifest (no annotations).
    let total_duration_ms = pres
        .transcript
        .as_ref()
        .and_then(|t| t.segments.last())
        .map(|seg| seg.end_ms)
        .or_else(|| pres.slides.last().map(|s| s.time_start + 5000))
        .unwrap_or(0);

    let viewer_manifest = ViewerManifest {
        title: &pres.title,
        author: pres.author.as_deref(),
        audio_track: pres.audio_track.as_deref(),
        total_duration_ms,
        transcript: pres.transcript.as_ref(),
        slides: pres
            .slides
            .iter()
            .map(|s: &Slide| ViewerSlide {
                id: &s.id,
                time_start: s.time_start,
                animations: &s.animations,
                pointer_trail: s.pointer_trail.as_ref(),
                transition: s.transition.as_ref(),
            })
            .collect(),
    };

    let manifest_json = serde_json::to_string(&viewer_manifest)?;

    // Base64-encode the audio.
    let audio_b64 = BASE64.encode(&unpacked.audio);

    // Pre-compile all slides from typst_source if present (F3 single-file mode).
    #[cfg(feature = "native")]
    let typst_source_svgs: Option<Vec<String>> = if let Some(ref path) = pres.typst_source {
        use crate::{typst_fonts, typst_render, typst_split};
        let source = unpacked.read_file_str(path)?;
        let (_, slide_sources) = typst_split::split_typst_slides(&source);
        if slide_sources.len() != pres.slides.len() {
            return Err(VecslideError::SlideCountMismatch {
                typst: slide_sources.len(),
                manifest: pres.slides.len(),
            });
        }
        let fonts = typst_fonts::bundled_fonts();
        let svgs: Result<Vec<_>, _> = slide_sources
            .iter()
            .map(|src| typst_render::compile_typst_to_svg(src, &fonts))
            .collect();
        Some(svgs?)
    } else {
        None
    };

    #[cfg(not(feature = "native"))]
    let typst_source_svgs: Option<Vec<String>> = None;

    // Build <script type="text/xml"> blocks for each slide (in order).
    let slide_scripts = pres
        .slides
        .iter()
        .enumerate()
        .map(|(i, slide)| {
            // Resolve the SVG string from whichever source is configured.
            let svg_string: String = if let Some(ref svgs) = typst_source_svgs {
                // Single typst_source mode: SVGs pre-compiled above.
                svgs[i].clone()
            } else if let Some(ref svg_path) = slide.svg_file {
                // Pre-existing SVG from the archive.
                let svg_bytes = unpacked.svgs.get(svg_path.as_str()).ok_or_else(|| {
                    VecslideError::MissingSvgFile { path: svg_path.clone() }
                })?;
                std::str::from_utf8(svg_bytes)
                    .map(|s| s.to_string())
                    .map_err(|e| VecslideError::Other(format!("SVG is not valid UTF-8: {e}")))?
            } else if let Some(ref src) = slide.typst_inline {
                // Inline Typst source.
                #[cfg(feature = "native")]
                {
                    use crate::{typst_fonts, typst_render};
                    let fonts = typst_fonts::bundled_fonts();
                    typst_render::compile_typst_to_svg(src, &fonts)?
                }
                #[cfg(not(feature = "native"))]
                {
                    let _ = src;
                    return Err(VecslideError::Other(
                        "typst_inline requires the 'native' feature".into(),
                    ));
                }
            } else if let Some(ref path) = slide.typst_file {
                // External .typ file inside the archive.
                #[cfg(feature = "native")]
                {
                    use crate::{typst_fonts, typst_render};
                    let src = unpacked.read_file_str(path)?;
                    let fonts = typst_fonts::bundled_fonts();
                    typst_render::compile_typst_to_svg(&src, &fonts)?
                }
                #[cfg(not(feature = "native"))]
                {
                    let _ = path;
                    return Err(VecslideError::Other(
                        "typst_file requires the 'native' feature".into(),
                    ));
                }
            } else {
                return Err(VecslideError::NoSlideSource { id: slide.id.clone() });
            };

            // Escape </script> occurrences inside SVG content to avoid breaking the HTML parser.
            let escaped = svg_string.replace("</script>", "<\\/script>");
            Ok(format!(
                r#"<script type="text/xml" id="slide-{i}">{escaped}</script>"#
            ))
        })
        .collect::<Result<Vec<_>, VecslideError>>()?
        .join("\n");

    let theme_css = unpacked
        .theme_css
        .as_deref()
        .unwrap_or(DEFAULT_THEME_CSS);

    let html = TEMPLATE
        .replace("{{MANIFEST_JSON}}", &manifest_json)
        .replace("{{AUDIO_BASE64}}", &audio_b64)
        .replace("{{SLIDE_SCRIPTS}}", &slide_scripts)
        .replace("{{THEME_CSS}}", theme_css);

    Ok(html)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{Presentation, Slide};

    fn minimal_unpacked(title: &str, svg: &str) -> UnpackedPresentation {
        let slide = Slide {
            id: "slide_01".to_string(),
            svg_file: Some("assets/01.svg".to_string()),
            typst_file: None,
            typst_inline: None,
            time_start: 0,
            animations: vec![],
            pointer_trail: None,
            transition: None,
        };
        let manifest = Presentation {
            format_version: "1.0".to_string(),
            title: title.to_string(),
            author: None,
            description: None,
            date: None,
            language: None,
            audio_track: Some("audio/test.opus".to_string()),
            typst_source: None,
            slides: vec![slide],
            annotations: vec![],
            transcript: None,
        };
        let mut svgs = HashMap::new();
        svgs.insert("assets/01.svg".to_string(), svg.as_bytes().to_vec());
        UnpackedPresentation {
            manifest,
            svgs,
            extra_files: HashMap::new(),
            audio: b"fake-opus-bytes".to_vec(),
            theme_css: None,
        }
    }

    #[test]
    fn output_contains_manifest_json() {
        let unpacked = minimal_unpacked("Test Title", "<svg></svg>");
        let html = compile_html(&unpacked).unwrap();
        assert!(html.contains("\"title\":\"Test Title\""), "manifest JSON missing from output");
    }

    #[test]
    fn output_contains_slide_script_block() {
        let unpacked = minimal_unpacked("T", "<svg><circle/></svg>");
        let html = compile_html(&unpacked).unwrap();
        assert!(html.contains(r#"id="slide-0""#), "slide script block missing");
        assert!(html.contains("<svg><circle/></svg>"), "SVG content missing");
    }

    #[test]
    fn output_contains_audio_base64() {
        let unpacked = minimal_unpacked("T", "<svg/>");
        let html = compile_html(&unpacked).unwrap();
        // base64 of b"fake-opus-bytes"
        let expected_b64 = BASE64.encode(b"fake-opus-bytes");
        assert!(html.contains(&expected_b64), "audio base64 missing from output");
    }

    #[test]
    fn annotations_are_excluded_from_output() {
        use crate::manifest::{Annotation, AnnotationType, Point};
        let mut unpacked = minimal_unpacked("T", "<svg/>");
        unpacked.manifest.annotations.push(Annotation {
            slide_id: "slide_01".to_string(),
            kind: AnnotationType::Comment {
                position: Point { x: 10.0, y: 20.0 },
                text: "SECRET_NOTE".to_string(),
            },
        });
        let html = compile_html(&unpacked).unwrap();
        assert!(!html.contains("SECRET_NOTE"), "annotation data leaked into viewer HTML");
        assert!(!html.contains("annotations"), "annotations key leaked into viewer HTML");
    }

    #[test]
    fn missing_svg_file_returns_error() {
        let unpacked = UnpackedPresentation {
            manifest: Presentation {
                format_version: "1.0".to_string(),
                title: "T".to_string(),
                author: None,
                description: None,
                date: None,
                language: None,
                audio_track: Some("audio/test.opus".to_string()),
                typst_source: None,
                slides: vec![Slide {
                    id: "s1".to_string(),
                    svg_file: Some("missing.svg".to_string()),
                    typst_file: None,
                    typst_inline: None,
                    time_start: 0,
                    animations: vec![],
                    pointer_trail: None,
                    transition: None,
                }],
                annotations: vec![],
                transcript: None,
            },
            svgs: HashMap::new(), // intentionally empty
            extra_files: HashMap::new(),
            audio: vec![],
            theme_css: None,
        };
        assert!(matches!(
            compile_html(&unpacked),
            Err(VecslideError::MissingSvgFile { .. })
        ));
    }

    #[test]
    fn script_tag_in_svg_is_escaped() {
        let svg_with_script = "<svg><script>alert(1)</script></svg>";
        let unpacked = minimal_unpacked("T", svg_with_script);
        let html = compile_html(&unpacked).unwrap();
        // The literal </script> inside SVG content must be escaped to not break the HTML parser
        assert!(!html.contains("</script>alert"), "unescaped </script> in SVG content");
    }
}
