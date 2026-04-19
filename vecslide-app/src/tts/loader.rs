//! Kokoro model loader — guarantees the ~92 MB ONNX download happens at
//! most once per page session and maps Transformers.js progress events
//! onto our [`TtsState`] state machine.
//!
//! Call [`ensure_model_loaded`] from UI handlers before invoking
//! [`super::synth::synthesize_slide`]. The first call drives the full
//! `Uninit → LoadingWasm → LoadingWeights → Ready` transition; later
//! calls short-circuit to `Ready`.

use std::cell::Cell;

use wasm_bindgen::JsValue;

use super::bindings::{self, LoadProgress};
use super::state::TtsState;

thread_local! {
    /// `true` once the JS-side model has been built at least once in
    /// this page session. We mirror the JS shim's memoization so Rust
    /// can skip rebuilding the progress closure for subsequent calls.
    static LOADED: Cell<bool> = const { Cell::new(false) };
}

/// Ensures the Kokoro model is fully loaded, reporting state transitions
/// through `on_state`. Safe to call concurrently and repeatedly: the
/// expensive work runs at most once (guarded both by this module's
/// `LOADED` flag and the JS shim's `_model` singleton).
///
/// State-machine mapping:
///
/// - On entry (first call): emit `LoadingWasm { progress: -1.0 }`, then
///   every file download event becomes `LoadingWeights { progress }`.
///   Transformers.js emits percentage-carrying events during `progress`
///   status and bare lifecycle events (`initiate`, `done`, `ready`) with
///   `progress = -1.0`.
/// - On success: emit `Ready` and mark `LOADED = true`.
/// - On failure: propagate the `JsValue` unchanged; the caller is
///   responsible for transitioning to `TtsState::Error`.
///
/// The `on_state` closure must be `Fn + Clone + 'static` because the
/// loader both calls it directly (for the entry `LoadingWasm` tick and
/// the final `Ready` tick) and hands a clone to the JS progress
/// callback, which lives inside a `wasm_bindgen::Closure` with `'static`
/// bound.
pub async fn ensure_model_loaded<F>(on_state: F) -> Result<(), JsValue>
where
    F: Fn(TtsState) + Clone + 'static,
{
    // Fast path: already loaded this session.
    if LOADED.with(Cell::get) {
        on_state(TtsState::Ready);
        return Ok(());
    }

    // Belt-and-braces: if the JS shim reports ready (e.g. another code
    // path loaded it, or a hot reload kept the singleton alive), trust
    // it and just sync our own flag.
    if bindings::is_ready() {
        LOADED.with(|cell| cell.set(true));
        on_state(TtsState::Ready);
        return Ok(());
    }

    on_state(TtsState::LoadingWasm { progress: -1.0 });

    let on_state_cb = on_state.clone();
    bindings::ensure_loaded(move |p: LoadProgress| {
        // Every fetch event transitions the UI to LoadingWeights. The
        // `status` field is currently ignored — a future revision could
        // keep LoadingWasm during `initiate` and only flip to
        // LoadingWeights on the first `download` tick, but that's a
        // cosmetic refinement, not a correctness issue.
        on_state_cb(TtsState::LoadingWeights { progress: p.progress });
    })
    .await?;

    LOADED.with(|cell| cell.set(true));
    on_state(TtsState::Ready);
    Ok(())
}
