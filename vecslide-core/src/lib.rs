//! # vecslide-core
//!
//! Core library for the **`.vecslide`** format — vector presentations with
//! synchronized Opus audio, playable from a single self-contained HTML file.
//!
//! A `.vecslide` file is a ZIP archive that bundles:
//! - a `manifest.yaml` describing slides, timestamps, animations, pointer trail,
//! - one or more SVG slides (or a single Typst source split by `----`),
//! - an optional Opus audio track (OGG container),
//! - optional author-only annotations (stripped on HTML export).
//!
//! This crate is UI-free and has no hard dependency on the filesystem:
//! the default feature set is **WASM-safe** and is what powers the web
//! authoring tool. Native tooling enables extra features behind feature flags.
//!
//! ## Feature flags
//!
//! | Feature             | Adds                                        | Typical consumer     |
//! |---------------------|---------------------------------------------|----------------------|
//! | _(default)_         | `manifest`, `compile_html`, `validation`, `pointer`, `typst_split` | WASM app             |
//! | `zip-io`            | In-memory ZIP read/write (`pack`, `unpack`) via `miniz_oxide`      | WASM app, browser    |
//! | `native`            | `zip-io` + Typst compilation + bundled fonts + `std::fs` helpers   | CLI, server-side     |
//!
//! `native` is a superset of `zip-io`. Enabling `native` on WASM is **not** supported.
//!
//! ## Quick start: parse a manifest
//!
//! ```ignore
//! use vecslide_core::manifest::Presentation;
//!
//! let yaml = r#"
//! title: "Cell Anatomy"
//! slides:
//!   - id: slide_01
//!     time_start: 0
//!     svg_file: vector_assets/01.svg
//! "#;
//! let p: Presentation = serde_norway::from_str(yaml)?;
//! assert_eq!(p.title, "Cell Anatomy");
//! # Ok::<_, serde_norway::Error>(())
//! ```
//!
//! ## Quick start: unpack a `.vecslide` in memory (requires `zip-io`)
//!
//! ```ignore
//! use vecslide_core::unpack_from_reader;
//!
//! let bytes = std::fs::read("lesson.vecslide")?;
//! let unpacked = unpack_from_reader(std::io::Cursor::new(bytes))?;
//! println!("{} slides", unpacked.presentation.slides.len());
//! # Ok::<_, vecslide_core::VecslideError>(())
//! ```
//!
//! ## Quick start: compile to a single self-contained HTML
//!
//! ```ignore
//! use vecslide_core::{unpack_from_reader, compile_html::compile};
//!
//! let bytes = std::fs::read("lesson.vecslide")?;
//! let unpacked = unpack_from_reader(std::io::Cursor::new(bytes))?;
//! let html: String = compile(&unpacked)?;
//! std::fs::write("lesson.html", html)?;
//! # Ok::<_, vecslide_core::VecslideError>(())
//! ```
//!
//! ## Modules at a glance
//!
//! | Module          | Purpose                                                                  |
//! |-----------------|--------------------------------------------------------------------------|
//! | [`manifest`]    | Data structures + YAML/JSON serde for `manifest.yaml`                    |
//! | [`validation`]  | Consistency checks (ordered timestamps, referenced files, durations)     |
//! | [`compile_html`]| Compile an [`UnpackedPresentation`] into a single self-contained HTML    |
//! | [`pointer`]     | Pointer-trail logic: movement threshold, decimation, fading opacity      |
//! | [`player_template`] | Embedded HTML/CSS/JS viewer template (`include_str!`)                |
//! | [`theme`]       | Theme tokens shared between editor and viewer                            |
//! | [`typst_split`] | Split a `.typ` source into per-slide sections separated by `----`        |
//! | [`pack`]        | Fold a source folder into a `.vecslide` ZIP (requires `zip-io`)          |
//! | [`unpack`]      | Read a `.vecslide` ZIP into memory (requires `zip-io`)                   |
//! | [`typst_render`]| Compile Typst sources to SVG (requires `native`)                         |
//! | [`typst_fonts`] | Fonts bundled for Typst compilation (requires `native`)                  |
//!
//! ## Companion crates
//!
//! - [`vecslide`](https://crates.io/crates/vecslide) — thin facade re-exporting this crate under a shorter name.
//!
//! ## License
//!
//! Licensed under either of [MIT](https://opensource.org/license/mit) or
//! [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0) at your option.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod compile_html;
pub mod error;
pub mod manifest;
pub mod player_template;
pub mod pointer;
pub mod theme;
pub mod typst_fonts;
pub mod typst_split;
pub mod validation;

#[cfg(feature = "zip-io")]
#[cfg_attr(docsrs, doc(cfg(feature = "zip-io")))]
pub mod pack;
#[cfg(feature = "zip-io")]
#[cfg_attr(docsrs, doc(cfg(feature = "zip-io")))]
pub mod unpack;

#[cfg(feature = "native")]
#[cfg_attr(docsrs, doc(cfg(feature = "native")))]
pub mod typst_render;

pub use compile_html::UnpackedPresentation;
pub use error::VecslideError;

#[cfg(feature = "zip-io")]
#[cfg_attr(docsrs, doc(cfg(feature = "zip-io")))]
pub use pack::pack_to_writer;
#[cfg(feature = "zip-io")]
#[cfg_attr(docsrs, doc(cfg(feature = "zip-io")))]
pub use unpack::unpack_from_reader;
