use crate::types::ParsedPage;

/// Format pypdf-style pages as plain text.
///
/// Pages are joined with a single newline, mirroring the common
/// `"\n".join(page.extract_text() for page in reader.pages)` idiom used
/// with pypdf. No `--- Page N ---` headers are added, since pypdf itself
/// emits none.
pub fn format_pypdf(pages: &[ParsedPage]) -> String {
    pages
        .iter()
        .map(|page| page.text.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ParsedPage;

    fn page(n: usize, text: &str) -> ParsedPage {
        ParsedPage {
            page_number: n,
            page_width: 0.0,
            page_height: 0.0,
            text: text.into(),
            text_items: vec![],
        }
    }

    #[test]
    fn empty() {
        assert_eq!(format_pypdf(&[]), "");
    }

    #[test]
    fn joins_pages_with_newline() {
        let out = format_pypdf(&[page(1, "alpha"), page(2, "beta")]);
        assert_eq!(out, "alpha\nbeta");
    }
}
