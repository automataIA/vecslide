<p align="center">
  <img src="https://raw.githubusercontent.com/wolvie1986/vecslide/main/svg/vecslide-icon-color.svg" alt="VecSlide" width="120" />
</p>

# vecslide-core

[![Crates.io](https://img.shields.io/crates/v/vecslide-core.svg)](https://crates.io/crates/vecslide-core)
[![docs.rs](https://img.shields.io/docsrs/vecslide-core)](https://docs.rs/vecslide-core)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

Core library for the **`.vecslide`** format — vector presentations (SVG + Opus
audio) playable from a single self-contained HTML file. UI-free, no mandatory
filesystem access, and WASM-safe by default.

> 💡 Prefer the shorter import path `vecslide::…`? Use the facade crate
> [`vecslide`](https://crates.io/crates/vecslide) — same API, same version.

## Installation

```toml
[dependencies]
# Default: WASM-safe, no ZIP I/O, no Typst compilation.
vecslide-core = "0.1"

# In-memory ZIP read/write (still WASM-safe, no std::fs).
vecslide-core = { version = "0.1", features = ["zip-io"] }

# Native tooling: std::fs + Typst → SVG + bundled fonts.
vecslide-core = { version = "0.1", features = ["native"] }
```

`native` is a superset of `zip-io`. Do **not** enable `native` on WASM targets.

## What is `.vecslide`?

A `.vecslide` is a renamed ZIP archive with this layout:

```
lesson.vecslide
├── manifest.yaml           # Slides, timestamps, animations, pointer trail
├── audio/voice.opus        # Opus audio (OGG container), optional
└── vector_assets/*.svg     # SVG slides (or a single split Typst source)
```

Why this format (1-hour 4K lecture):

|                    | 4K MP4        | VecSlide              |
|--------------------|---------------|-----------------------|
| Size               | ~930 MB       | ~13 MB                |
| Visual quality     | Compression   | Perfect at any DPI    |
| Text               | Baked pixels  | Selectable, searchable|
| Offline playback   | Needs player  | Any modern browser    |

## Quick start

### Parse a manifest

```rust,ignore
use vecslide_core::manifest::Presentation;

let yaml = r#"
title: "Cell Anatomy"
slides:
  - id: slide_01
    time_start: 0
    svg_file: vector_assets/01.svg
"#;

let p: Presentation = serde_norway::from_str(yaml)?;
assert_eq!(p.slides.len(), 1);
```

### Unpack a `.vecslide` (feature `zip-io`)

```rust,ignore
use vecslide_core::unpack_from_reader;

let bytes = std::fs::read("lesson.vecslide")?;
let unpacked = unpack_from_reader(std::io::Cursor::new(bytes))?;
println!("{} slides", unpacked.presentation.slides.len());
```

### Compile to a single self-contained HTML file

```rust,ignore
use vecslide_core::{unpack_from_reader, compile_html};

let bytes = std::fs::read("lesson.vecslide")?;
let unpacked = unpack_from_reader(std::io::Cursor::new(bytes))?;
let html: String = compile_html::compile(&unpacked)?;
std::fs::write("lesson.html", html)?;
```

The resulting `.html` inlines every SVG slide, embeds the Opus track as a
Base64 data URI, and ships the player JS/CSS — no external assets, no CDN.

### Pack a folder into a `.vecslide` (feature `zip-io`)

```rust,ignore
use std::fs::File;
use vecslide_core::pack_to_writer;

let out = File::create("lesson.vecslide")?;
pack_to_writer(std::path::Path::new("./source_folder"), out)?;
```

## Modules

| Module            | Default | `zip-io` | `native` | Purpose                                                |
|-------------------|:------:|:-------:|:-------:|--------------------------------------------------------|
| `manifest`        |   ✅   |         |         | `Presentation`, `Slide`, `Animation`, YAML/JSON serde  |
| `validation`      |   ✅   |         |         | Ordered timestamps, referenced files, durations        |
| `compile_html`    |   ✅   |         |         | `UnpackedPresentation` → single self-contained HTML    |
| `player_template` |   ✅   |         |         | HTML/CSS/JS viewer (`include_str!`)                    |
| `pointer`         |   ✅   |         |         | Trail: movement threshold, decimation, fade            |
| `theme`           |   ✅   |         |         | Theme tokens                                           |
| `typst_split`     |   ✅   |         |         | Split `.typ` sources on `----`                         |
| `pack`            |        |   ✅    |         | Folder → `.vecslide` ZIP (in-memory)                   |
| `unpack`          |        |   ✅    |         | `.vecslide` ZIP → `UnpackedPresentation` (in-memory)   |
| `typst_render`    |        |         |   ✅    | Typst source → SVG                                     |
| `typst_fonts`     |        |         |   ✅    | Fonts bundled for Typst                                |

## Compression

Compression is applied **only** when packing:

```
pack.rs           →  .vecslide (ZIP)
  SVG + YAML     :  Deflated  (~75–80% size reduction on XML text)
  Opus audio     :  Stored    (already compressed)

compile_html.rs   →  .html (no ZIP)
  SVG            :  raw text in <script type="text/xml">
  audio          :  inline Base64 data URI
```

## Viewer modes

The compiled HTML supports two playback modes selected by the manifest:

- **Audio mode** — `manifest.audio_track` is set. The `<audio>` element is the
  master clock; `requestAnimationFrame` reads `audio.currentTime` at 60 fps and
  drives slides, animations, and the pointer trail.
- **Static mode** — no `audio_track`. Navigation is by index: arrows, Space,
  and swipe advance / go back without depending on audio.

## Minimum supported Rust version

Currently **1.90**. Bumping the MSRV is not a breaking change and will be
documented in the changelog.

## License

Licensed under either of

- MIT license ([LICENSE-MIT](../LICENSE-MIT) or <https://opensource.org/license/mit>)
- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
