/// Preamble injected before every slide snippet.
/// Background is transparent so the viewer can apply its own theme.
const SLIDE_PREAMBLE: &str = "#set page(width: 1920pt, height: 1080pt, margin: 2em, fill: none)\n\
                               #set text(font: \"Noto Sans\", weight: \"bold\", size: 24pt)\n\
                               #show math.equation: set text(font: \"New Computer Modern Sans Math\", weight: \"regular\")\n\
                               #set heading(numbering: none)\n";

/// Metadata parsed from the optional YAML front-matter block at the top of a `.typ` file.
#[derive(Debug, Default)]
pub struct TypstFrontmatter {
    pub title: Option<String>,
    pub author: Option<String>,
}

/// Splits a `.typ` source file into individual slide snippets.
///
/// The file may start with a YAML front-matter block delimited by `---` lines.
/// Slides are separated by `\n----\n` (four dashes on their own line).
/// Each snippet has `SLIDE_PREAMBLE` prepended so it can be compiled independently.
pub fn split_typst_slides(source: &str) -> (TypstFrontmatter, Vec<String>) {
    let (frontmatter, body) = parse_frontmatter(source);

    let slides: Vec<String> = body
        .split("\n----\n")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| format!("{SLIDE_PREAMBLE}\n{s}"))
        .collect();

    (frontmatter, slides)
}

/// Parses the optional YAML front-matter and returns (frontmatter, remaining body).
fn parse_frontmatter(source: &str) -> (TypstFrontmatter, &str) {
    // Front-matter must start at the very beginning with "---\n"
    if !source.starts_with("---\n") {
        return (TypstFrontmatter::default(), source);
    }

    // Find the closing "---" line
    let rest = &source[4..]; // skip opening "---\n"
    if let Some(end_pos) = rest.find("\n---\n") {
        let yaml_str = &rest[..end_pos];
        let body = &rest[end_pos + 5..]; // skip "\n---\n"

        let frontmatter = parse_yaml_frontmatter(yaml_str);
        (frontmatter, body)
    } else {
        // Malformed front-matter: treat whole source as body
        (TypstFrontmatter::default(), source)
    }
}

fn parse_yaml_frontmatter(yaml: &str) -> TypstFrontmatter {
    let mut fm = TypstFrontmatter::default();
    for line in yaml.lines() {
        if let Some(rest) = line.strip_prefix("title:") {
            fm.title = Some(rest.trim().trim_matches('"').to_string());
        } else if let Some(rest) = line.strip_prefix("author:") {
            fm.author = Some(rest.trim().trim_matches('"').to_string());
        }
    }
    fm
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_on_dashes() {
        let src = "slide one\n----\nslide two\n----\nslide three";
        let (_, slides) = split_typst_slides(src);
        assert_eq!(slides.len(), 3);
        assert!(slides[0].contains("slide one"));
        assert!(slides[1].contains("slide two"));
        assert!(slides[2].contains("slide three"));
    }

    #[test]
    fn each_slide_has_preamble() {
        let src = "content";
        let (_, slides) = split_typst_slides(src);
        assert_eq!(slides.len(), 1);
        assert!(slides[0].contains("#set page"));
    }

    #[test]
    fn empty_sections_are_ignored() {
        let src = "a\n----\n\n----\nb";
        let (_, slides) = split_typst_slides(src);
        assert_eq!(slides.len(), 2);
    }

    #[test]
    fn frontmatter_is_parsed() {
        let src = "---\ntitle: My Talk\nauthor: Prof. Rossi\n---\nslide one";
        let (fm, slides) = split_typst_slides(src);
        assert_eq!(fm.title.as_deref(), Some("My Talk"));
        assert_eq!(fm.author.as_deref(), Some("Prof. Rossi"));
        assert_eq!(slides.len(), 1);
        assert!(slides[0].contains("slide one"));
    }

    #[test]
    fn no_frontmatter_works() {
        let src = "slide one\n----\nslide two";
        let (fm, slides) = split_typst_slides(src);
        assert!(fm.title.is_none());
        assert_eq!(slides.len(), 2);
    }
}
