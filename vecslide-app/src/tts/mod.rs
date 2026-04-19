//! TTS (Text-to-Speech) module based on Ferrocarril / Kokoro-82M.
//!
//! End-to-end pipeline:
//! 1. [`loader`] — dynamic import of Ferrocarril + fetch weights into IndexedDB.
//! 2. [`synth`] — calls Ferrocarril → f32 PCM 24 kHz mono.
//! 3. [`encoder`] — encodes PCM → Opus packets via `web_sys::AudioEncoder`.
//! 4. [`ogg_mux`] — muxes Opus packets into a valid `.ogg` file.
//!
//! Ferrocarril lives in a **separate WASM bundle** served from `public/ferrocarril/`
//! and loaded on-demand on the first "Synthesize" click. This keeps the main
//! `vecslide-app` bundle small (see plan D1).

pub mod bindings;
pub mod config;
pub mod encoder;
pub mod loader;
pub mod ogg_mux;
pub mod state;
pub mod synth;

pub use loader::ensure_model_loaded;
pub use bindings::list_voices;
pub use ogg_mux::{OpusPacket, concat_ogg_streams, write_ogg_opus};
pub use state::{SlideAudio, TtsState, hash_text};
pub use synth::synthesize_slide;
