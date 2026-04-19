/// Returns the text and math fonts bundled with the binary.
///
/// Noto Sans for text (Regular / Bold / Italic),
/// New Computer Modern Sans Math for formulas — GUST OTF equivalent of
/// the LaTeX `sansmathfonts` package (CMSS-style math glyphs).
/// Embedded via `include_bytes!` so the binary is self-contained.
///
/// 🔒 These fonts are used only at compile-time; they are never included in the
/// generated `.html` viewer output.
#[cfg(feature = "native")]
pub fn bundled_fonts() -> Vec<Vec<u8>> {
    vec![
        include_bytes!("fonts/NotoSans-Regular.otf").to_vec(),
        include_bytes!("fonts/NotoSans-Bold.otf").to_vec(),
        include_bytes!("fonts/NotoSans-Italic.otf").to_vec(),
        include_bytes!("fonts/NewCMSansMath-Regular.otf").to_vec(),
    ]
}
