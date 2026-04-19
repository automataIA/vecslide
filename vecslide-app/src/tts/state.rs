//! Shared state types for the TTS module.
//!
//! - [`SlideAudio`]: per-slide Opus/OGG blob with staleness-check metadata.
//! - [`TtsState`]: global state machine for Ferrocarril loading.
//! - [`hash_text`]: stable hash of the narration text used as a snapshot.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Synthesized audio for a single slide.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlideAudio {
    /// Complete contents of a `.ogg` file (OGG container, Opus codec),
    /// ready to be written to disk or played via an `<audio>` tag.
    pub ogg_bytes: Vec<u8>,

    /// Track duration in milliseconds, calculated from the original PCM
    /// sample count divided by (sample_rate / 1000).
    pub duration_ms: u64,

    /// Hash of the narration text at the time of synthesis. If the current
    /// text differs, the editor shows the "stale" badge.
    pub generated_from: u64,
}

impl SlideAudio {
    /// Returns `true` if the current text differs from the one used to
    /// synthesize the audio — the audio needs to be regenerated.
    pub fn is_stale(&self, current_text: &str) -> bool {
        self.generated_from != hash_text(current_text)
    }
}

/// State machine for Ferrocarril loading.
///
/// Expected transitions on the first Synthesize click:
/// `Uninit → LoadingWasm → LoadingWeights → Ready → Synthesizing → Ready`.
///
/// Subsequent clicks skip to `Ready → Synthesizing → Ready`.
#[derive(Debug, Clone, PartialEq)]
pub enum TtsState {
    /// Never initialized — first click will start loading.
    Uninit,

    /// Fetching the Ferrocarril wasm bundle (~1.1 MB gzip).
    LoadingWasm {
        /// Value in `[0.0, 1.0]`. `-1.0` if the length is unknown.
        progress: f32,
    },

    /// Fetching the 340 MB Kokoro weights from HuggingFace → IndexedDB.
    LoadingWeights {
        /// Value in `[0.0, 1.0]`. `-1.0` if the length is unknown.
        progress: f32,
    },

    /// Everything loaded, ready to synthesize.
    Ready,

    /// Currently synthesizing a specific slide.
    Synthesizing { slide_idx: usize },

    /// Synthesizing all slides sequentially.
    SynthesizingAll { current: usize, total: usize },

    /// Fatal error: fetch failed, SHA mismatch, unsupported API, etc.
    Error(String),
}

impl TtsState {
    /// `true` if the state represents an in-progress operation (loading or synth).
    pub fn is_busy(&self) -> bool {
        matches!(
            self,
            Self::LoadingWasm { .. } | Self::LoadingWeights { .. } | Self::Synthesizing { .. } | Self::SynthesizingAll { .. }
        )
    }
}

/// 64-bit hash of the narration text. Used as a snapshot for the
/// `SlideAudio::generated_from` flag. Not cryptographically secure, but stable
/// within a single build (which is all we need for the stale check).
pub fn hash_text(text: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_text_is_deterministic() {
        assert_eq!(hash_text("hello"), hash_text("hello"));
    }

    #[test]
    fn hash_text_differs_for_different_strings() {
        assert_ne!(hash_text("hello"), hash_text("world"));
    }

    #[test]
    fn slide_audio_stale_detection() {
        let audio = SlideAudio {
            ogg_bytes: vec![1, 2, 3],
            duration_ms: 1_000,
            generated_from: hash_text("original text"),
        };
        assert!(!audio.is_stale("original text"));
        assert!(audio.is_stale("modified text"));
    }

    #[test]
    fn tts_state_is_busy() {
        assert!(!TtsState::Uninit.is_busy());
        assert!(!TtsState::Ready.is_busy());
        assert!(!TtsState::Error("x".into()).is_busy());
        assert!(TtsState::LoadingWasm { progress: 0.0 }.is_busy());
        assert!(TtsState::LoadingWeights { progress: 0.5 }.is_busy());
        assert!(TtsState::Synthesizing { slide_idx: 0 }.is_busy());
    }
}
