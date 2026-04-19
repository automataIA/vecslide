//! Thin wasm-bindgen bridge to the kokoro-js loader shim.
//!
//! The JS shim in `index.html` exposes three functions on
//! `window.__vecslideKokoro`:
//!
//! ```text
//! load(onProgress): Promise<void>
//! generate(text, voice): Promise<{ pcm: Float32Array, sampleRate: number }>
//! isReady(): bool
//! ```
//!
//! This module wraps them into idiomatic Rust async functions so the
//! rest of the TTS module never has to touch raw JS values.
//!
//! **Browser-only.** These calls panic on native targets because
//! `window.__vecslideKokoro` does not exist outside the wasm build.

use js_sys::{Array, Float32Array, Function, Promise, Reflect};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

// ─── extern declarations ─────────────────────────────────────────────────────

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__vecslideKokoro"], js_name = "load")]
    fn kokoro_load_js(on_progress: &Function) -> Promise;

    #[wasm_bindgen(js_namespace = ["window", "__vecslideKokoro"], js_name = "generate")]
    fn kokoro_generate_js(text: &str, voice: &str) -> Promise;

    #[wasm_bindgen(js_namespace = ["window", "__vecslideKokoro"], js_name = "isReady")]
    fn kokoro_is_ready_js() -> bool;

    #[wasm_bindgen(js_namespace = ["window", "__vecslideKokoro"], js_name = "listVoices")]
    fn kokoro_list_voices_js() -> JsValue;
}

// ─── Public types ────────────────────────────────────────────────────────────

/// One progress event emitted by Transformers.js while kokoro-js loads the
/// model from the HuggingFace Hub.
#[derive(Debug, Clone)]
pub struct LoadProgress {
    /// Lifecycle stage, one of `initiate`, `download`, `progress`, `done`,
    /// `ready`. Empty if the JS shim could not read it.
    pub status: String,
    /// File being fetched, e.g. `onnx/model_quantized.onnx`. Empty when
    /// the event is not file-specific.
    pub file: String,
    /// Fraction in `[0.0, 1.0]`. `-1.0` when the event carries no
    /// progress value (lifecycle transitions, final ready event).
    pub progress: f32,
}

// ─── API ─────────────────────────────────────────────────────────────────────

/// Returns `true` if the kokoro-js shim has already finished loading
/// the model in this page session.
///
/// Cheap to call; consults the JS-side `_model !== null` cache.
pub fn is_ready() -> bool {
    kokoro_is_ready_js()
}

/// Returns the list of voice IDs available in the loaded model.
///
/// Returns an empty `Vec` if the model is not loaded yet.
pub fn list_voices() -> Vec<String> {
    let val = kokoro_list_voices_js();
    if val.is_undefined() || val.is_null() {
        return Vec::new();
    }
    let arr: Array = match val.dyn_into() {
        Ok(a) => a,
        Err(_) => return Vec::new(),
    };
    arr.iter()
        .filter_map(|v| v.as_string())
        .collect()
}

/// Triggers `KokoroTTS.from_pretrained()` and resolves when the model
/// is ready. Subsequent calls resolve immediately (the shim memoizes).
///
/// The supplied `on_progress` is invoked synchronously on the JS
/// callback thread for every Transformers.js progress event. It must
/// stay alive until the returned future resolves; this function keeps
/// the backing `Closure` alive until `await` completes, then drops it.
pub async fn ensure_loaded<F>(mut on_progress: F) -> Result<(), JsValue>
where
    F: FnMut(LoadProgress) + 'static,
{
    let closure = Closure::<dyn FnMut(JsValue)>::new(move |p: JsValue| {
        on_progress(parse_progress_event(&p));
    });
    let promise = kokoro_load_js(closure.as_ref().unchecked_ref::<Function>());
    let result = JsFuture::from(promise).await;
    // Drop the closure only after the load promise has settled — the JS
    // side may still have fired a final "ready" event while the promise
    // was resolving.
    drop(closure);
    result.map(|_| ())
}

/// Runs Kokoro synthesis for the given English `text` with `voice`,
/// returning `(pcm, sample_rate)` where `pcm` is mono f32 audio.
///
/// The caller must ensure [`ensure_loaded`] has completed successfully
/// before calling this, otherwise the JS shim throws.
pub async fn generate(text: &str, voice: &str) -> Result<(Vec<f32>, u32), JsValue> {
    let promise = kokoro_generate_js(text, voice);
    let result = JsFuture::from(promise).await?;

    let pcm_js = Reflect::get(&result, &JsValue::from_str("pcm"))
        .map_err(|_| JsValue::from_str("kokoro_generate result missing .pcm"))?;
    let sample_rate_js = Reflect::get(&result, &JsValue::from_str("sampleRate"))
        .map_err(|_| JsValue::from_str("kokoro_generate result missing .sampleRate"))?;

    // `pcm_js` is a Float32Array living in the JS heap; copy it into a
    // Rust Vec so the wasm linear-memory-side pipeline owns it outright.
    let pcm_array = Float32Array::from(pcm_js);
    let len = pcm_array.length() as usize;
    let mut pcm = vec![0.0f32; len];
    pcm_array.copy_to(&mut pcm);

    let sample_rate = sample_rate_js
        .as_f64()
        .ok_or_else(|| JsValue::from_str("kokoro_generate sampleRate is not a number"))?
        as u32;

    Ok((pcm, sample_rate))
}

// ─── helpers ─────────────────────────────────────────────────────────────────

fn parse_progress_event(p: &JsValue) -> LoadProgress {
    let status = Reflect::get(p, &JsValue::from_str("status"))
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_default();
    let file = Reflect::get(p, &JsValue::from_str("file"))
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_default();
    let progress = Reflect::get(p, &JsValue::from_str("progress"))
        .ok()
        .and_then(|v| v.as_f64())
        .map(|v| v as f32)
        .unwrap_or(-1.0);
    LoadProgress { status, file, progress }
}
