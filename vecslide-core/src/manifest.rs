use serde::{Deserialize, Serialize};

/// Top-level presentation descriptor. Maps to manifest.yaml inside a .vecslide archive.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Presentation {
    /// Format version for forward compatibility (e.g. "1.0").
    #[serde(default = "default_format_version")]
    pub format_version: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// Short description of the presentation content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Creation or recording date in ISO 8601 format (e.g. "2026-04-07").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    /// BCP-47 language tag for the presentation language (e.g. "it-IT", "en-US").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// Path of the audio file inside the ZIP archive (e.g. "audio/voce.opus").
    /// `None` for "Light" mode (transcript-only, no recorded audio).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_track: Option<String>,
    /// Single .typ source file with `----` separators for all slides.
    /// Alternative to specifying svg_file/typst_file on each individual slide.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub typst_source: Option<String>,
    pub slides: Vec<Slide>,
    /// Editor-only data: highlights and comments. Excluded from the compiled HTML viewer.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub annotations: Vec<Annotation>,
    /// Synchronized or standalone transcript. Placed last in the manifest to keep
    /// metadata and slides readable at a glance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcript: Option<Transcript>,
}

fn default_format_version() -> String {
    "1.0".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Slide {
    /// Unique identifier used as the DOM element id (e.g. "slide_01").
    pub id: String,
    /// Absolute timestamp from audio start, in milliseconds.
    pub time_start: u64,
    /// Path of a pre-existing SVG file inside the ZIP archive (e.g. "vector_assets/01.svg").
    /// Exactly one of svg_file, typst_file, or typst_inline must be set
    /// (unless typst_source is set on the Presentation).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub svg_file: Option<String>,
    /// Path of an external .typ file inside the ZIP archive to compile to SVG.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub typst_file: Option<String>,
    /// Inline Typst source to compile to SVG.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub typst_inline: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub animations: Vec<Animation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pointer_trail: Option<PointerTrail>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transition: Option<TransitionKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Animation {
    /// Id of the SVG element to animate (must match an `id` attribute in the SVG file).
    pub element_id: String,
    /// Absolute timestamp when the animation starts, in milliseconds.
    pub time_start: u64,
    /// Duration of the animation in milliseconds.
    pub duration: u64,
    /// Animation kind identifier (e.g. "fade-in", "draw-path", "slide-left").
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PointerTrail {
    pub points: Vec<TrailPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TrailPoint {
    /// Absolute timestamp in milliseconds.
    pub time_ms: u64,
    pub x: f32,
    pub y: f32,
}

/// Editor-only annotation attached to a slide. Never included in the compiled HTML viewer.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Annotation {
    pub slide_id: String,
    #[serde(flatten)]
    pub kind: AnnotationType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnnotationType {
    Highlight {
        rect: Rect,
        color: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        author_note: Option<String>,
    },
    Comment {
        position: Point,
        text: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransitionKind {
    Cut,
    Fade,
    SlideLeft,
    SlideRight,
}

// ── Transcript ────────────────────────────────────────────────────────────────

/// Full transcript of the presentation, placed at the end of the manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Transcript {
    pub mode: TranscriptMode,
    /// BCP-47 language tag (e.g. "it-IT", "en-US").
    pub language: String,
    pub segments: Vec<TranscriptSegment>,
}

/// How the transcript is used by the viewer.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscriptMode {
    /// Highlights are driven by `audio.currentTime`. Requires `audio_track`.
    Synchronized,
    /// No audio: viewer uses Web Speech API (TTS) to read segments aloud.
    Standalone,
}

/// One spoken phrase, aligned to a time range.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TranscriptSegment {
    /// Start of the segment in milliseconds (absolute from audio start).
    pub start_ms: u64,
    /// End of the segment in milliseconds.
    pub end_ms: u64,
    pub text: String,
    /// ID of the slide visible during this segment (for click-to-slide navigation).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slide_ref: Option<String>,
    /// Word-level timestamps for karaoke-style highlighting (optional).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub words: Vec<TranscriptWord>,
}

/// Word-level timestamp inside a segment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TranscriptWord {
    pub text: String,
    pub start_ms: u64,
    pub end_ms: u64,
}
