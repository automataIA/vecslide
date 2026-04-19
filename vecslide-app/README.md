<p align="center">
  <img src="../svg/vecslide-logo-horizontal-color.svg" alt="VecSlide" width="420" />
</p>

# vecslide-app

Authoring tool for VecSlide. Rust/WASM web app with Leptos 0.8 + Trunk (CSR).
It allows writing slides in Typst, viewing a live SVG preview, and exporting
the result as a standalone HTML viewer.

## Development server

```sh
trunk serve --port 3000 --open
```

## Main modules

| Module | Responsibility |
|--------|----------------|
| `editor` | Typst editor + live SVG preview, slide navigation (prev/next, keyboard) |
| `export` | Compiles all slides → SVG → self-contained HTML; download via Blob URL |
| `typst_world` | Typst `InMemoryWorld` for WASM compilation (bundled fonts, no filesystem) |

## HTML Export

The **Export HTML** button in the editor toolbar:

1. Splits the Typst source into sections separated by `\n----\n`
2. Compiles each section → SVG via `typst_world::compile_slide_to_svg`
3. Builds an `UnpackedPresentation` (without audio)
4. Calls `vecslide_core::compile_html` to produce the HTML
5. Downloads `presentation.html` via Blob URL

The output file is self-contained (inline SVG, no compression). For the compressed
`.vecslide` format use the CLI: `vecslide compile`.

The generated HTML viewer enters **static mode** (no audio): arrows,
Space, and touch swipe navigate slides by index without depending on audio.

## Production build

```sh
trunk build --release
# output in dist/
```
