<p align="center">
  <img src="https://raw.githubusercontent.com/wolvie1986/vecslide/main/svg/vecslide-icon-color.svg" alt="VecSlide" width="120" />
</p>

# vecslide

[![Crates.io](https://img.shields.io/crates/v/vecslide.svg)](https://crates.io/crates/vecslide)
[![docs.rs](https://img.shields.io/docsrs/vecslide)](https://docs.rs/vecslide)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

Facade crate for the **`.vecslide`** vector-presentation format.

This crate re-exports the entire public API of
[`vecslide-core`](https://crates.io/crates/vecslide-core) under a shorter
name, so you can write:

```rust,ignore
use vecslide::manifest::Presentation;
use vecslide::{unpack_from_reader, compile_html};
```

Both crates are released in lockstep at the same version. Pick the one whose
import path you prefer — the functionality is identical.

## Installation

```toml
[dependencies]
# Default: WASM-safe, no ZIP I/O, no Typst compilation.
vecslide = "0.1"

# In-memory ZIP read/write (still WASM-safe).
vecslide = { version = "0.1", features = ["zip-io"] }

# Native tooling: std::fs + Typst → SVG + bundled fonts.
vecslide = { version = "0.1", features = ["native"] }
```

`native` is a superset of `zip-io`. Do **not** enable `native` on WASM targets.

## Why a facade?

- Shorter, more memorable name on crates.io and in imports.
- Keeps `vecslide-core` internally factored with a precise name.
- Forward-compatible: if the project later grows a CLI/binary crate, it can
  still live under the `vecslide` *family* without breaking this import path.

For the full feature table, modules list, examples, and format spec see the
[`vecslide-core` README](https://crates.io/crates/vecslide-core) or the
[project repository](https://github.com/automataIA/vecslide).

## License

Licensed under either of [MIT](../LICENSE-MIT) or
[Apache-2.0](../LICENSE-APACHE) at your option.
