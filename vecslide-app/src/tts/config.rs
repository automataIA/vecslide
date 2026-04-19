//! TTS pipeline configuration constants.
//!
//! Kokoro-82M is loaded at runtime through the `kokoro-js` NPM package
//! (maintained by Xenova, built on Transformers.js + ONNX Runtime Web).
//! A tiny JS shim in `index.html` exposes a Promise-returning API on
//! `window.__vecslideKokoro`, bridged to Rust by [`super::bindings`].
//!
//! All model fetch + IndexedDB/OPFS caching is handled by Transformers.js
//! — we do **not** ship our own weight fetcher or hash verifier. The
//! HuggingFace hub URL and LFS SHA are managed upstream.

/// HuggingFace model id for Kokoro-82M v1.0 (ONNX community port).
///
/// Maintained by `onnx-community` on the HuggingFace Hub and kept in
/// sync with `hexgrad/Kokoro-82M`. Compatible with Transformers.js
/// and hence with `kokoro-js`.
pub const KOKORO_MODEL_ID: &str = "onnx-community/Kokoro-82M-v1.0-ONNX";

/// Quantization variant we request from kokoro-js.
///
/// - `q8` — 8-bit, ~92 MB, best tradeoff size/quality. **Default.**
/// - `q8f16` — int8 weights + fp16 activations, ~86 MB.
/// - `fp16` — 16-bit, ~163 MB, transparent quality.
/// - `fp32` — 32-bit, ~326 MB, reference.
/// - `q4` — 4-bit matmul, ~305 MB (includes fp16 fallback tables).
/// - `q4f16` — 4-bit + fp16 weights, ~154 MB.
pub const KOKORO_DTYPE: &str = "q8";

/// Default voice used for synthesis. American female ("Heart").
///
/// The full voice list lives in the model repo under `voices/voices.json`
/// and can be switched via the future voice-picker UI (plan D8).
pub const DEFAULT_VOICE: &str = "af_heart";

/// Sample rate of the PCM output produced by Kokoro-82M (f32 mono, 24 kHz).
///
/// This is the model's native output rate. Transformers.js returns it
/// alongside the `Float32Array`, so this constant is a fallback / sanity
/// check, not the source of truth.
pub const SAMPLE_RATE_HZ: u32 = 24_000;

/// Target bitrate for the Opus encoder (bps). 64 kbps is the sweet spot
/// for mono voice at 24 kHz: transparent quality, reasonable file size.
pub const OPUS_BITRATE_BPS: u32 = 64_000;
