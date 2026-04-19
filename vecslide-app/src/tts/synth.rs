//! Audio synthesis for a single slide, end-to-end.
//!
//! The pipeline is three stages:
//!
//! 1. **Kokoro** — [`super::bindings::generate`] calls the kokoro-js shim
//!    which runs Transformers.js + ONNX Runtime Web and returns a
//!    `Float32Array` of mono PCM at 24 kHz (the model's native rate).
//! 2. **Opus encode** — [`super::encoder::encode_pcm_to_opus_packets`]
//!    feeds the PCM into a WebCodecs `AudioEncoder` configured for Opus.
//! 3. **OGG mux** — [`super::ogg_mux::write_ogg_opus`] wraps the Opus
//!    packets into a valid `.ogg` container (pure-Rust, testable on
//!    native).
//!
//! The caller is responsible for ensuring [`super::loader::ensure_model_loaded`]
//! has completed successfully before calling this function; [`synthesize_slide`]
//! assumes the JS-side `_model` singleton is populated and will propagate
//! the shim's "Kokoro model not loaded" error otherwise.

use wasm_bindgen::JsValue;

use super::bindings;
use super::config::OPUS_BITRATE_BPS;
use super::encoder::encode_pcm_to_opus_packets;
use super::ogg_mux::write_ogg_opus;
use super::state::{SlideAudio, hash_text};

/// Synthesises an OGG/Opus blob for a single slide's narration text.
///
/// Returns a [`SlideAudio`] whose `generated_from` is the hash of the
/// supplied text, so subsequent edits mark the entry stale automatically.
///
/// # Errors
///
/// - Kokoro model not loaded, or the shim's `generate()` throws.
/// - WebCodecs `AudioEncoder` unavailable or rejects the Opus config.
/// - OGG muxing fails (pure-Rust error path, extremely unlikely).
pub async fn synthesize_slide(text: &str, voice: &str) -> Result<SlideAudio, JsValue> {
    // ── 1. Kokoro inference ──────────────────────────────────────────────────
    let (pcm, sample_rate) = bindings::generate(text, voice).await?;
    if pcm.is_empty() {
        return Err(JsValue::from_str("Kokoro returned empty PCM"));
    }

    // ── 2. PCM f32 → Opus packets ────────────────────────────────────────────
    let packets =
        encode_pcm_to_opus_packets(&pcm, sample_rate, OPUS_BITRATE_BPS).await?;

    // ── 3. Opus packets → .ogg container ─────────────────────────────────────
    let ogg_bytes = write_ogg_opus(&packets, 1, sample_rate)
        .map_err(|e| JsValue::from_str(&format!("OGG mux failed: {e}")))?;

    // Duration from sample count — exact because Kokoro returns integer
    // frame counts. `sample_rate` is nonzero by contract (24 kHz from
    // Kokoro, verified in bindings::generate).
    let duration_ms = (pcm.len() as u64 * 1_000) / u64::from(sample_rate);

    Ok(SlideAudio {
        ogg_bytes,
        duration_ms,
        generated_from: hash_text(text),
    })
}
