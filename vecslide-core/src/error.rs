use thiserror::Error;

#[derive(Debug, Error)]
pub enum VecslideError {
    #[error("timestamps not ascending at slide index {index}: {detail}")]
    TimestampOrder { index: usize, detail: String },

    #[error("audio_track field is empty")]
    EmptyAudioTrack,

    #[error("slides list is empty")]
    EmptySlides,

    #[error("referenced SVG file not found: {path}")]
    MissingSvgFile { path: String },

    #[error("slide '{id}' has no source (svg_file, typst_file, or typst_inline must be set)")]
    NoSlideSource { id: String },

    #[error("slide id '{id}' is not unique")]
    DuplicateSlideId { id: String },

    #[error("slide '{slide_id}' animation[{anim_index}] has an empty element_id")]
    EmptyElementId { slide_id: String, anim_index: usize },

    #[error("slide '{slide_id}' has an animation with time_start outside the slide's time range")]
    AnimationOutOfRange { slide_id: String },

    #[error("transcript segment[{index}] has start_ms >= end_ms")]
    InvalidSegment { index: usize },

    #[error("transcript segment[{index}] overlaps with the previous segment")]
    OverlappingSegments { index: usize },

    #[error("typst compilation failed: {0}")]
    TypstCompile(String),

    #[error("slide count mismatch: typst source has {typst} slides but manifest has {manifest}")]
    SlideCountMismatch { typst: usize, manifest: usize },

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_norway::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    Other(String),

    #[cfg(feature = "zip-io")]
    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[cfg(feature = "zip-io")]
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
