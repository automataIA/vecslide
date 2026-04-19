//! # vecslide
//!
//! Facade crate for the [`.vecslide`](https://github.com/automataIA/vecslide)
//! vector-presentation format. This crate re-exports the entire public API of
//! [`vecslide-core`](https://crates.io/crates/vecslide-core) under a shorter
//! name so downstream users can write `vecslide::manifest::Presentation`
//! instead of `vecslide_core::manifest::Presentation`.
//!
//! Both crates are released in lockstep at the same version — pick whichever
//! import path you prefer.
//!
//! ## Feature flags
//!
//! | Feature    | Forwards to                 | Notes                                                |
//! |------------|-----------------------------|------------------------------------------------------|
//! | _(default)_| `vecslide-core` (default)   | WASM-safe; manifest, validation, HTML compilation.   |
//! | `zip-io`   | `vecslide-core/zip-io`      | In-memory ZIP read/write — WASM-safe.                |
//! | `native`   | `vecslide-core/native`      | `std::fs` + Typst compilation. Not for WASM builds.  |
//!
//! ## Example
//!
//! ```ignore
//! use vecslide::manifest::Presentation;
//!
//! let yaml = r#"
//! title: "Cell Anatomy"
//! slides:
//!   - id: slide_01
//!     time_start: 0
//!     svg_file: vector_assets/01.svg
//! "#;
//! let p: Presentation = serde_norway::from_str(yaml)?;
//! assert_eq!(p.slides.len(), 1);
//! # Ok::<_, serde_norway::Error>(())
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]

pub use vecslide_core::*;
