use std::{
    collections::{HashMap, HashSet},
    io::{Read, Seek},
};

use zip::ZipArchive;

use crate::{
    compile_html::UnpackedPresentation,
    error::VecslideError,
    manifest::Presentation,
};

/// Reads a `.vecslide` archive from any `Read + Seek` source into memory.
///
/// WASM-safe: pass `std::io::Cursor<Vec<u8>>` as the reader.
/// Used by `compile_html`, the CLI `compile` subcommand, and the WASM import path.
pub fn unpack_from_reader<R: Read + Seek>(reader: R) -> Result<UnpackedPresentation, VecslideError> {
    let mut archive = ZipArchive::new(reader)?;

    // ── 1. Read manifest ────────────────────────────────────────────────────
    let manifest: Presentation = {
        let mut entry = archive.by_name("manifest.yaml")?;
        let mut yaml = String::new();
        entry.read_to_string(&mut yaml)?;
        serde_norway::from_str(&yaml)?
    };

    // ── 2. Build set of paths already handled (SVG + audio) ─────────────────
    let known_svgs: HashSet<&str> = manifest
        .slides
        .iter()
        .filter_map(|s| s.svg_file.as_deref())
        .collect();
    let audio_path: Option<&str> = manifest.audio_track.as_deref();

    // ── 3. Read audio (empty in Light mode) ─────────────────────────────────
    let audio = if let Some(track) = audio_path {
        let mut entry = archive.by_name(track)?;
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf)?;
        buf
    } else {
        Vec::new()
    };

    // ── 4. Read SVG files referenced by the manifest ────────────────────────
    let mut svgs = HashMap::new();
    for slide in &manifest.slides {
        if let Some(ref svg_file) = slide.svg_file {
            if svgs.contains_key(svg_file) {
                continue;
            }
            let mut entry = archive.by_name(svg_file)?;
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf)?;
            svgs.insert(svg_file.clone(), buf);
        }
    }

    // ── 5. Read extra_files: everything not manifest, SVG, or audio ─────────
    // This includes source.typ and any other files stored for round-trip editing.
    let mut extra_files = HashMap::new();
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();

        if name == "manifest.yaml"
            || known_svgs.contains(name.as_str())
            || audio_path == Some(name.as_str())
        {
            continue;
        }

        let mut buf = Vec::new();
        entry.read_to_end(&mut buf)?;
        extra_files.insert(name, buf);
    }

    Ok(UnpackedPresentation { manifest, svgs, extra_files, audio, theme_css: None })
}

/// Extracts a `.vecslide` archive to a directory on disk.
/// Used by the CLI `unpack` subcommand.
#[cfg(feature = "native")]
pub fn unpack_to_dir(
    input: &std::path::Path,
    output_dir: &std::path::Path,
) -> Result<Presentation, VecslideError> {
    use std::fs;

    let file = std::fs::File::open(input)?;
    let mut archive = ZipArchive::new(file)?;

    fs::create_dir_all(output_dir)?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let entry_path = output_dir.join(entry.name());

        if entry.is_dir() {
            fs::create_dir_all(&entry_path)?;
        } else {
            if let Some(parent) = entry_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut out = fs::File::create(&entry_path)?;
            std::io::copy(&mut entry, &mut out)?;
        }
    }

    let manifest_path = output_dir.join("manifest.yaml");
    let yaml = fs::read_to_string(manifest_path)?;
    let manifest: Presentation = serde_norway::from_str(&yaml)?;
    Ok(manifest)
}

/// Reads a `.vecslide` archive from a file path.
/// Used by the CLI `compile` subcommand.
#[cfg(feature = "native")]
pub fn unpack(input: &std::path::Path) -> Result<UnpackedPresentation, VecslideError> {
    let file = std::fs::File::open(input)?;
    unpack_from_reader(file)
}
