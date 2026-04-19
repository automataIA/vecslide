use std::io::Write;

use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

use crate::{compile_html::UnpackedPresentation, error::VecslideError};

/// Packages an `UnpackedPresentation` into a `.vecslide` archive written to `writer`.
///
/// WASM-safe: operates entirely in memory — no `std::fs` access.
/// Pass a `std::io::Cursor<Vec<u8>>` to collect the bytes.
///
/// Compression strategy:
/// - `manifest.yaml`, `.svg`, and `extra_files`: `Deflated`
/// - Audio: `Stored` (Opus is already compressed)
pub fn pack_to_writer<W: Write + std::io::Seek>(
    unpacked: &UnpackedPresentation,
    writer: W,
) -> Result<(), VecslideError> {
    let mut zip = ZipWriter::new(writer);

    let deflated = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let stored   = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

    // ── manifest.yaml ────────────────────────────────────────────────────────
    let yaml = serde_norway::to_string(&unpacked.manifest)?;
    zip.start_file("manifest.yaml", deflated)?;
    zip.write_all(yaml.as_bytes())?;

    // ── SVG files ────────────────────────────────────────────────────────────
    for (path, bytes) in &unpacked.svgs {
        zip.start_file(path.as_str(), deflated)?;
        zip.write_all(bytes)?;
    }

    // ── extra_files (source.typ and any other editing artefacts) ─────────────
    for (path, bytes) in &unpacked.extra_files {
        zip.start_file(path.as_str(), deflated)?;
        zip.write_all(bytes)?;
    }

    // ── audio (stored — Opus is already compressed) ───────────────────────────
    if !unpacked.audio.is_empty()
        && let Some(ref track) = unpacked.manifest.audio_track
    {
        zip.start_file(track.as_str(), stored)?;
        zip.write_all(&unpacked.audio)?;
    }

    zip.finish()?;
    Ok(())
}

/// Packages a source directory into a `.vecslide` archive (ZIP with renamed extension).
///
/// Only available with the `native` feature (requires `std::fs`).
/// For in-memory packing (e.g. WASM), use `pack_to_writer` instead.
///
/// Compression strategy:
/// - `manifest.yaml` and all `.svg` files: `Deflated`
/// - Audio file: `Stored` (Opus is already compressed; re-compressing wastes CPU)
#[cfg(feature = "native")]
pub fn pack(
    source_dir: &std::path::Path,
    manifest: &crate::manifest::Presentation,
    output: &std::path::Path,
) -> Result<(), VecslideError> {
    use std::{
        fs::File,
        io::{BufWriter, Read},
    };

    let file = File::create(output)?;
    let mut zip = ZipWriter::new(BufWriter::new(file));

    let deflated = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let stored   = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

    // Write manifest.yaml
    let yaml = serde_norway::to_string(manifest)?;
    zip.start_file("manifest.yaml", deflated)?;
    zip.write_all(yaml.as_bytes())?;

    // Track all paths already written to avoid duplicates.
    let mut written: std::collections::HashSet<String> = std::collections::HashSet::new();
    written.insert("manifest.yaml".to_string());

    // Write SVG files (only slides with a direct svg_file reference).
    for slide in &manifest.slides {
        if let Some(ref svg_file) = slide.svg_file {
            if written.contains(svg_file.as_str()) {
                continue;
            }
            let svg_path = source_dir.join(svg_file);
            let mut buf = Vec::new();
            File::open(&svg_path)?.read_to_end(&mut buf)?;

            zip.start_file(svg_file.as_str(), deflated)?;
            zip.write_all(&buf)?;
            written.insert(svg_file.clone());
        }
    }

    // Write typst_source (single-file mode, e.g. "lezione.typ").
    if let Some(ref typst_source) = manifest.typst_source
        && !written.contains(typst_source.as_str())
    {
        let path = source_dir.join(typst_source);
        let mut buf = Vec::new();
        File::open(&path)?.read_to_end(&mut buf)?;

        zip.start_file(typst_source.as_str(), deflated)?;
        zip.write_all(&buf)?;
        written.insert(typst_source.clone());
    }

    // Write per-slide typst_file references.
    for slide in &manifest.slides {
        if let Some(ref typst_file) = slide.typst_file {
            if written.contains(typst_file.as_str()) {
                continue;
            }
            let path = source_dir.join(typst_file);
            let mut buf = Vec::new();
            File::open(&path)?.read_to_end(&mut buf)?;

            zip.start_file(typst_file.as_str(), deflated)?;
            zip.write_all(&buf)?;
            written.insert(typst_file.clone());
        }
    }

    // Write audio without compression (Opus is already compressed).
    // Skipped in Light mode (audio_track = None).
    if let Some(ref track) = manifest.audio_track {
        let audio_path = source_dir.join(track);
        let mut audio_buf = Vec::new();
        File::open(&audio_path)?.read_to_end(&mut audio_buf)?;
        zip.start_file(track.as_str(), stored)?;
        zip.write_all(&audio_buf)?;
        written.insert(track.clone());
    }

    zip.finish()?;
    Ok(())
}
