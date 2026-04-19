/// Compiles a Typst source string into a pure SVG string.
///
/// 🔒 The output is SVG — no trace of Typst remains. The compiled HTML viewer
/// receives only the SVG bytes and has no knowledge of Typst or this module.
#[cfg(feature = "native")]
pub fn compile_typst_to_svg(source: &str, fonts: &[Vec<u8>]) -> Result<String, crate::error::VecslideError> {
    use typst::layout::PagedDocument;
    use typst_as_lib::TypstEngine;

    let engine = TypstEngine::builder()
        .main_file(source)
        .fonts(fonts.iter().map(|f| f.as_slice()))
        .build();

    let doc: PagedDocument = engine
        .compile()
        .output
        .map_err(|e| crate::error::VecslideError::TypstCompile(format!("{e:?}")))?;

    let page = doc.pages.into_iter().next().ok_or_else(|| {
        crate::error::VecslideError::TypstCompile("no pages in compiled Typst document".into())
    })?;

    Ok(typst_svg::svg(&page))
}

#[cfg(all(test, feature = "native"))]
mod tests {
    use super::*;
    use crate::typst_fonts::bundled_fonts;

    #[test]
    fn simple_text_compiles_to_svg() {
        let source = r#"
#set page(width: 400pt, height: 300pt, margin: 1em, fill: none)
Hello, *world*!
"#;
        let fonts = bundled_fonts();
        let svg = compile_typst_to_svg(source, &fonts).expect("compilation failed");
        assert!(svg.contains("<svg"), "output must be an SVG element");
    }

    #[test]
    fn math_formula_compiles_to_svg() {
        let source = r#"
#set page(width: 400pt, height: 300pt, margin: 1em, fill: none)
#show math.equation: set text(font: "New Computer Modern Sans Math", weight: "regular")
$ E = m c^2 $
"#;
        let fonts = bundled_fonts();
        let svg = compile_typst_to_svg(source, &fonts).expect("math compilation failed");
        assert!(svg.contains("<svg"), "output must be an SVG element");
    }

    #[test]
    fn invalid_typst_returns_error() {
        let source = "#this_is_not_a_valid_function_call(";
        let fonts = bundled_fonts();
        let result = compile_typst_to_svg(source, &fonts);
        assert!(result.is_err(), "invalid Typst should return an error");
    }

}
