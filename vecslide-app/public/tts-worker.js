// Web Worker for Kokoro-82M TTS inference.
//
// Runs KokoroTTS model loading and audio generation off the main thread
// so the UI stays responsive during synthesis.
//
// Protocol (postMessage):
//   Main -> Worker:
//     { type: "load" }
//     { type: "generate", id: number, text: string, voice: string }
//   Worker -> Main:
//     { type: "progress", status, file, progress }
//     { type: "loaded" }
//     { type: "generated", id: number, pcm: Float32Array, sampleRate: number }
//     { type: "error", id: number|null, message: string }

import { KokoroTTS } from "https://esm.sh/kokoro-js@1.2.0";

const MODEL_ID = "onnx-community/Kokoro-82M-v1.0-ONNX";
const DTYPE    = "q8";
const DEVICE   = "wasm";

let _model = null;
let _loading = null;

async function handleLoad() {
  if (_model) {
    self.postMessage({ type: "loaded" });
    return;
  }
  if (_loading) {
    await _loading;
    self.postMessage({ type: "loaded" });
    return;
  }
  _loading = KokoroTTS.from_pretrained(MODEL_ID, {
    dtype: DTYPE,
    device: DEVICE,
    progress_callback: (p) => {
      try {
        self.postMessage({
          type:     "progress",
          status:   p.status || "",
          file:     p.file || "",
          progress: typeof p.progress === "number" ? p.progress / 100 : -1,
        });
      } catch (_) { /* never let a callback break loading */ }
    },
  });
  try {
    _model = await _loading;
  } finally {
    _loading = null;
  }
  self.postMessage({ type: "loaded" });
}

async function handleGenerate(id, text, voice) {
  if (!_model) {
    self.postMessage({ type: "error", id, message: "Kokoro model not loaded" });
    return;
  }
  try {
    const result = await _model.generate(text, { voice });
    // Transfer the underlying ArrayBuffer for zero-copy.
    const pcm = result.audio;
    const sampleRate = result.sampling_rate;
    self.postMessage(
      { type: "generated", id, pcm, sampleRate },
      [pcm.buffer]
    );
  } catch (e) {
    self.postMessage({ type: "error", id, message: e.message || String(e) });
  }
}

self.addEventListener("message", (event) => {
  const msg = event.data;
  if (msg.type === "load") {
    handleLoad().catch((e) => {
      self.postMessage({ type: "error", id: null, message: e.message || String(e) });
    });
  } else if (msg.type === "generate") {
    handleGenerate(msg.id, msg.text, msg.voice).catch((e) => {
      self.postMessage({ type: "error", id: msg.id, message: e.message || String(e) });
    });
  }
});
