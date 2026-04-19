/// Minimal in-memory Typst World for WASM compilation.
///
/// Does not touch the file system — all fonts are bundled via include_bytes!,
/// the source is passed in memory. Package imports are unsupported (return error).
use typst::{
    Library, LibraryExt,
    diag::{FileError, FileResult},
    foundations::{Bytes, Datetime},
    layout::PagedDocument,
    syntax::{FileId, Source, VirtualPath},
    text::{Font, FontBook},
    utils::LazyHash,
};

// ── Bundled fonts (Noto Sans for text, New CM Sans Math for formulas) ────────
static FONT_NOTO_REGULAR: &[u8] = include_bytes!("../../vecslide-core/src/fonts/NotoSans-Regular.otf");
static FONT_NOTO_BOLD: &[u8] = include_bytes!("../../vecslide-core/src/fonts/NotoSans-Bold.otf");
static FONT_NOTO_ITALIC: &[u8] = include_bytes!("../../vecslide-core/src/fonts/NotoSans-Italic.otf");
static FONT_NCMSANS_MATH: &[u8] = include_bytes!("../../vecslide-core/src/fonts/NewCMSansMath-Regular.otf");

const BUNDLED_FONT_BYTES: &[&[u8]] = &[
    FONT_NOTO_REGULAR,
    FONT_NOTO_BOLD,
    FONT_NOTO_ITALIC,
    FONT_NCMSANS_MATH,
];

/// Pre-built font book + font list + Typst library, constructed once.
/// Shared across all compilations to avoid re-initializing Library::default()
/// (the entire Typst stdlib) on every compile.
pub struct FontAssets {
    fonts: Vec<Font>,
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
}

impl FontAssets {
    pub fn build() -> Self {
        let mut font_book = FontBook::new();
        let mut fonts = Vec::new();
        for &bytes in BUNDLED_FONT_BYTES {
            let data = Bytes::new(bytes.to_vec());
            for index in 0u32.. {
                match Font::new(data.clone(), index) {
                    Some(font) => {
                        font_book.push(font.info().clone());
                        fonts.push(font);
                    }
                    None => break,
                }
            }
        }
        Self {
            fonts,
            library: LazyHash::new(Library::default()),
            book: LazyHash::new(font_book),
        }
    }
}

/// Single-file Typst World: one source string, bundled fonts, no packages.
/// Borrows the cached Library and FontBook from FontAssets — zero clones per compile.
pub struct InMemoryWorld<'a> {
    library: &'a LazyHash<Library>,
    book: &'a LazyHash<FontBook>,
    fonts: &'a Vec<Font>,
    source: Source,
}

impl<'a> InMemoryWorld<'a> {
    pub fn new(source_text: &str, assets: &'a FontAssets) -> Self {
        let main_id = FileId::new(None, VirtualPath::new("/main.typ"));
        let source = Source::new(main_id, source_text.to_string());
        Self {
            library: &assets.library,
            book: &assets.book,
            fonts: &assets.fonts,
            source,
        }
    }
}

impl typst::World for InMemoryWorld<'_> {
    fn library(&self) -> &LazyHash<Library> {
        self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        self.book
    }

    fn main(&self) -> FileId {
        self.source.id()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.source.id() {
            Ok(self.source.clone())
        } else {
            Err(FileError::NotFound(id.vpath().as_rootless_path().into()))
        }
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        Err(FileError::NotFound(id.vpath().as_rootless_path().into()))
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }

    fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
        None
    }
}

// ── Dynamic preamble: typography + theme colors from DaisyUI ─────────────────

use vecslide_core::theme::ThemeColors;

/// Build a Typst preamble with typography settings and theme-derived colors.
///
/// Covers DA-FARE items 4 (typography), 5 (color palette), 6 (code blocks),
/// 7 (tables), and 8 (figures).
fn build_preamble(theme: &ThemeColors) -> String {
    format!(
        r##"
// === PALETTE FROM DAISYUI THEME "{name}" ===
// Generated automatically — do NOT edit by hand
#let c-bg          = rgb("{base_100}")
#let c-bg-elevated = rgb("{base_200}")
#let c-bg-deep     = rgb("{base_300}")
#let c-text        = rgb("{base_content}")
#let c-primary     = rgb("{primary}")
#let c-primary-tx  = rgb("{primary_content}")
#let c-secondary   = rgb("{secondary}")
#let c-secondary-tx = rgb("{secondary_content}")
#let c-accent      = rgb("{accent}")
#let c-accent-tx   = rgb("{accent_content}")
#let c-neutral     = rgb("{neutral}")
#let c-muted       = rgb("{neutral}")
#let c-muted-tx    = rgb("{neutral_content}")

// --- Page layout ---
#set page(width: 1920pt, height: 1080pt, margin: (x: 80pt, y: 60pt), fill: c-bg)

// --- Typography (Item 4) ---
#set text(font: "Noto Sans", size: 28pt, fill: c-text, lang: "it")
#set par(leading: 0.65em, spacing: 1.2em)
#set heading(numbering: none)
#show heading.where(level: 1): set block(above: 1.5em, below: 0.75em)
#show heading.where(level: 2): set block(above: 1.2em, below: 0.6em)
#show heading.where(level: 3): set block(above: 1.0em, below: 0.5em)
#show math.equation: set block(above: 8pt, below: 16pt)

// --- Colors applied to elements (Item 5) ---
#show heading.where(level: 1): set text(fill: c-primary, weight: "bold")
#show heading.where(level: 1): it => text(size: 48pt, weight: "bold", it.body)
#show heading.where(level: 2): set text(fill: c-secondary)
#show heading.where(level: 3): set text(fill: c-accent)
#show math.equation: set text(font: "New Computer Modern Sans Math", weight: "regular", fill: c-text)
#show link: set text(fill: c-secondary)
#show figure.caption: set text(fill: c-muted, size: 0.9em)

// --- Code blocks (Item 6) ---
// Note: Cascadia Code is not bundled; using Noto Sans as fallback for raw text.
#show raw: set text(size: 0.85em, fill: c-text)
#show raw.where(block: true): block.with(fill: c-bg-elevated, inset: 10pt, radius: 4pt)
#show raw.where(block: false): box.with(fill: c-bg-elevated, inset: (x: 3pt, y: 0pt), outset: (y: 3pt), radius: 2pt)

// --- Tables (Item 7) ---
#set table(stroke: none, gutter: 0.2em, fill: (x, y) => if y == 0 {{ c-primary }} else if calc.odd(y) {{ c-bg-elevated }}, inset: 8pt)
#show table.cell.where(y: 0): set text(fill: c-primary-tx, weight: "bold")
#show table.cell.where(x: 0): strong

// --- Figures (Item 8) ---
#show figure: set block(breakable: true)
#show figure.where(kind: table): set figure.caption(position: top)
// CeTZ: package imports not supported in WASM InMemoryWorld.
// Use c-primary/c-accent variables when writing CeTZ manually.
"##,
        name = theme.theme_name,
        base_100 = theme.base_100,
        base_200 = theme.base_200,
        base_300 = theme.base_300,
        base_content = theme.base_content,
        primary = theme.primary,
        primary_content = theme.primary_content,
        secondary = theme.secondary,
        secondary_content = theme.secondary_content,
        accent = theme.accent,
        accent_content = theme.accent_content,
        neutral = theme.neutral,
        neutral_content = theme.neutral_content,
    )
}

/// Compile a single slide source string → SVG string, or an error message.
/// Theme colors control the text fill, backgrounds, and accent colours in the preamble.
pub fn compile_slide_to_svg(source: &str, theme: &ThemeColors, assets: &FontAssets) -> Result<String, String> {
    let preamble = build_preamble(theme);
    let full_source = format!("{preamble}\n{source}");
    let world = InMemoryWorld::new(&full_source, assets);
    let result = typst::compile::<PagedDocument>(&world);
    match result.output {
        Ok(doc) => {
            let page = doc.pages.into_iter().next().ok_or("no pages")?;
            Ok(typst_svg::svg(&page))
        }
        Err(errors) => {
            let msg = errors
                .iter()
                .map(|e| e.message.to_string())
                .collect::<Vec<_>>()
                .join("\n");
            Err(msg)
        }
    }
}
