//! PCM f32 → Opus packets encoder via `web_sys::AudioEncoder` (WebCodecs).
//!
//! This module is **browser-only**: the code compiles to WASM and requires
//! the unstable `web-sys` APIs (flag `--cfg=web_sys_unstable_apis` in
//! `.cargo/config.toml` at the workspace root).
//!
//! Design principles:
//! - Single-shot: a call to [`encode_pcm_to_opus_packets`] creates a new
//!   `AudioEncoder`, configures it, encodes the given PCM buffer, flushes,
//!   collects packets via callback, and returns the list.
//! - No persistence: the encoder is closed at the end of each call,
//!   preventing leaks between consecutive synthesis runs.
//! - Input is **resampled to 48 kHz** (Opus native rate) and split into
//!   **20 ms frames** before feeding to `AudioEncoder`. Firefox's WebCodecs
//!   implementation does NOT auto-resample like its MediaRecorder API does;
//!   passing non-48 kHz input causes `flush()` to reject with an
//!   `EncodedAudioChunk` object. Resampling + chunking works across both
//!   Chrome and Firefox.

use std::cell::RefCell;
use std::rc::Rc;

use js_sys::{Function, Reflect, Uint8Array};
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    AudioData, AudioDataInit, AudioEncoder, AudioEncoderConfig, AudioEncoderInit,
    AudioSampleFormat, CodecState, EncodedAudioChunk,
};

use super::ogg_mux::OpusPacket;

/// Encodes a mono f32 PCM buffer into a sequence of Opus packets ready
/// to be muxed into an `.ogg` file by [`super::ogg_mux::write_ogg_opus`].
///
/// - `pcm` is interpreted as a single channel (mono).
/// - `sample_rate` is the input PCM sample rate (e.g. 24000 for Kokoro).
/// - `bitrate_bps` is the target Opus encoder bitrate.
///
/// Returns `Err` if `AudioEncoder` is unavailable in the browser (feature
/// detection), if the configuration is rejected, or if the encoder emits
/// an error during processing.
pub async fn encode_pcm_to_opus_packets(
    pcm: &[f32],
    sample_rate: u32,
    bitrate_bps: u32,
) -> Result<Vec<OpusPacket>, JsValue> {
    if pcm.is_empty() {
        return Ok(Vec::new());
    }
    if !is_supported() {
        return Err(JsValue::from_str(
            "WebCodecs AudioEncoder not available in the current browser",
        ));
    }

    // Collects (data_bytes, samples_48k) from each chunk emitted by the callback.
    let collected: Rc<RefCell<Vec<OpusPacket>>> = Rc::new(RefCell::new(Vec::new()));
    // First error reported by the encoder (latching).
    let first_error: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));

    // ─── Callback: output(chunk, metadata) ──────────────────────────────────
    let output_closure = {
        let collected = Rc::clone(&collected);
        Closure::<dyn FnMut(JsValue, JsValue)>::new(move |chunk_val: JsValue, _meta: JsValue| {
            let Ok(chunk) = chunk_val.dyn_into::<EncodedAudioChunk>() else { return };
            let len = chunk.byte_length() as usize;
            let mut buf = vec![0u8; len];
            if chunk.copy_to_with_u8_slice(&mut buf).is_err() {
                return;
            }

            // Duration is in microseconds. Convert to number of samples at
            // 48 kHz (the Opus reference rate): samples = us * 48 / 1000.
            // Fallback: 960 (20 ms) if the browser does not populate `duration`.
            let duration_us = Reflect::get(&chunk, &JsValue::from_str("duration"))
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or(20_000.0);
            let samples_48k = (duration_us * 48.0 / 1000.0).round() as u64;
            let samples_48k = if samples_48k == 0 { 960 } else { samples_48k };

            collected.borrow_mut().push(OpusPacket { data: buf, samples_48k });
        })
    };

    // ─── Callback: error(err) ───────────────────────────────────────────────
    let error_closure = {
        let first_error = Rc::clone(&first_error);
        Closure::<dyn FnMut(JsValue)>::new(move |err: JsValue| {
            let msg = err
                .as_string()
                .or_else(|| {
                    Reflect::get(&err, &JsValue::from_str("message"))
                        .ok()
                        .and_then(|v| v.as_string())
                })
                .unwrap_or_else(|| format!("{err:?}"));
            let mut slot = first_error.borrow_mut();
            if slot.is_none() {
                *slot = Some(msg);
            }
        })
    };

    let init = AudioEncoderInit::new(
        error_closure.as_ref().unchecked_ref::<Function>(),
        output_closure.as_ref().unchecked_ref::<Function>(),
    );
    let encoder = AudioEncoder::new(&init)?;

    // Always configure at 48 kHz — Firefox's WebCodecs AudioEncoder does NOT
    // auto-resample (unlike its MediaRecorder API). Passing 24 kHz directly
    // causes flush() to reject with an EncodedAudioChunk on Firefox.
    const OPUS_RATE: u32 = 48_000;

    let config = AudioEncoderConfig::new("opus", 1, OPUS_RATE);
    config.set_bitrate(bitrate_bps);
    encoder.configure(&config)?;

    // Resample input PCM to 48 kHz if needed.
    let pcm_48k = if sample_rate == OPUS_RATE {
        pcm.to_vec()
    } else {
        resample_linear(pcm, sample_rate, OPUS_RATE)
    };

    // Split into 20 ms frames (960 samples @ 48 kHz) and encode each one.
    let frame_samples = (OPUS_RATE / 50) as usize; // 20 ms = 960 samples
    let mut timestamp_us: i32 = 0;
    let frame_duration_us: i32 = 20_000; // 20 ms in μs

    for chunk in pcm_48k.chunks(frame_samples) {
        let data = build_audio_data_f32_mono(chunk, OPUS_RATE, timestamp_us)?;
        encoder.encode(&data)?;
        data.close();
        timestamp_us = timestamp_us.saturating_add(frame_duration_us);
    }

    // Flush: closes the queue and triggers emission of all remaining chunks.
    //
    // Firefox quirk: its WebCodecs AudioEncoder sometimes rejects the
    // `flush()` promise with an `EncodedAudioChunk` value instead of
    // resolving normally. When that happens we extract the chunk data
    // and treat it as a valid output packet rather than a fatal error.
    match JsFuture::from(encoder.flush()).await {
        Ok(_) => {}
        Err(rejected) => {
            // Try to interpret the rejection as a valid EncodedAudioChunk.
            if rejected.is_instance_of::<EncodedAudioChunk>() {
                let chunk: &EncodedAudioChunk = rejected.unchecked_ref();
                let len = chunk.byte_length() as usize;
                let mut buf = vec![0u8; len];
                if chunk.copy_to_with_u8_slice(&mut buf).is_ok() {
                    let duration_us = Reflect::get(&rejected, &JsValue::from_str("duration"))
                        .ok()
                        .and_then(|v| v.as_f64())
                        .unwrap_or(20_000.0);
                    let samples_48k = (duration_us * 48.0 / 1000.0).round() as u64;
                    let samples_48k = if samples_48k == 0 { 960 } else { samples_48k };
                    collected.borrow_mut().push(OpusPacket { data: buf, samples_48k });
                }
            } else {
                // Not an EncodedAudioChunk — propagate the real error.
                return Err(rejected);
            }
        }
    }
    let _ = encoder.close();

    // Closures must remain alive until the encoder has emitted the
    // final chunks. Since we are past the flush, we can drop them now.
    drop(output_closure);
    drop(error_closure);

    if let Some(msg) = first_error.borrow().clone() {
        return Err(JsValue::from_str(&format!("AudioEncoder error: {msg}")));
    }

    Ok(Rc::try_unwrap(collected)
        .map(RefCell::into_inner)
        .unwrap_or_else(|rc| rc.borrow().clone()))
}

/// Returns `true` if the current browser exposes `AudioEncoder`.
/// Performs feature detection without raising exceptions.
pub fn is_supported() -> bool {
    web_sys::window()
        .and_then(|w| Reflect::get(&w, &JsValue::from_str("AudioEncoder")).ok())
        .is_some_and(|v| !v.is_undefined() && !v.is_null())
}

/// Builds a mono `AudioData` from `pcm` (sample rate `sample_rate`).
fn build_audio_data_f32_mono(
    pcm: &[f32],
    sample_rate: u32,
    timestamp_us: i32,
) -> Result<AudioData, JsValue> {
    // Converts [f32] → Uint8Array (via ArrayBuffer view) without extra copies
    // on the Rust side; WebCodecs internally copies the data.
    let byte_len = std::mem::size_of_val(pcm);
    let u8_view = unsafe { std::slice::from_raw_parts(pcm.as_ptr().cast::<u8>(), byte_len) };
    let ja = Uint8Array::new_with_length(byte_len as u32);
    ja.copy_from(u8_view);

    let init = AudioDataInit::new(
        &ja,
        AudioSampleFormat::F32,
        1, // numberOfChannels
        u32::try_from(pcm.len()).unwrap_or(u32::MAX), // numberOfFrames
        sample_rate as f32,
        timestamp_us, // timestamp (μs)
    );
    AudioData::new(&init)
}

/// Resamples mono PCM from `from_rate` to `to_rate` using linear interpolation.
fn resample_linear(pcm: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate || pcm.is_empty() {
        return pcm.to_vec();
    }
    let ratio = from_rate as f64 / to_rate as f64;
    let out_len = ((pcm.len() as f64) / ratio).ceil() as usize;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let src_pos = i as f64 * ratio;
        let idx = src_pos as usize;
        let frac = src_pos - idx as f64;
        let sample = if idx + 1 < pcm.len() {
            pcm[idx] as f64 * (1.0 - frac) + pcm[idx + 1] as f64 * frac
        } else {
            pcm[idx.min(pcm.len() - 1)] as f64
        };
        out.push(sample as f32);
    }
    out
}

/// Current state of the encoder (useful for diagnostics / manual testing).
///
/// Thin wrapper function that the UI can call to label the pipeline state
/// in a progress bar.
pub fn encoder_state(encoder: &AudioEncoder) -> CodecState {
    encoder.state()
}
