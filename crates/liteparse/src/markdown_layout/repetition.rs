use crate::types::{ParsedPage, ProjectedLine};

use super::paragraphs::collapse_whitespace;

/// Fraction of page height treated as the "top band" for header detection.
/// Most running headers sit within the top 8–12% of a page; 12% gives some
/// slack for two-line headers without sweeping in body text.
pub(super) const HEADER_BAND_FRACTION: f32 = 0.12;

/// Fraction of page height treated as the "bottom band" for footer detection.
pub(super) const FOOTER_BAND_FRACTION: f32 = 0.12;

/// Fraction of pages on which a normalized line must appear (in the same
/// band) to be classified as a running header/footer.
const HEADER_FOOTER_MIN_FRACTION: f32 = 0.5;

/// Absolute floor on header/footer matches — single-page docs can't have a
/// "running" header by definition, and a single match on a 2-page doc is too
/// weak to act on.
const HEADER_FOOTER_MIN_PAGES: usize = 2;

/// Normalize a line for cross-page header/footer matching. Lowercases,
/// collapses whitespace, and replaces every run of ASCII digits with `#` so
/// `Page 1 of 6` and `Page 2 of 6` collapse to the same key.
pub(super) fn normalize_for_repetition(s: &str) -> String {
    let collapsed = collapse_whitespace(s).to_lowercase();
    let mut out = String::with_capacity(collapsed.len());
    let mut in_digits = false;
    for c in collapsed.chars() {
        if c.is_ascii_digit() {
            if !in_digits {
                out.push('#');
                in_digits = true;
            }
        } else {
            out.push(c);
            in_digits = false;
        }
    }
    out
}

/// Cross-page repetition detector. Returns the set of normalized strings that
/// appear in the top or bottom band of ≥ `HEADER_FOOTER_MIN_FRACTION` of
/// pages (capped below by `HEADER_FOOTER_MIN_PAGES`). The caller uses this to
/// filter `ProjectedLine`s before classification.
///
/// "Same band" means a line whose top is within `HEADER_BAND_FRACTION` of the
/// page top (header) or whose bottom is within `FOOTER_BAND_FRACTION` of the
/// page bottom (footer). Header and footer bands are tracked separately so a
/// company name that appears as both a header and a body-section title on
/// different pages isn't stripped from the body.
pub fn compute_header_footer_set(pages: &[ParsedPage]) -> std::collections::HashSet<String> {
    use std::collections::{HashMap, HashSet};
    let mut set: HashSet<String> = HashSet::new();
    if pages.len() < HEADER_FOOTER_MIN_PAGES {
        return set;
    }
    // Two counters keyed by `(band, normalized_text)` — band is `'h'` or `'f'`.
    let mut counts: HashMap<(char, String), HashSet<usize>> = HashMap::new();
    for page in pages {
        let header_cutoff = page.page_height * HEADER_BAND_FRACTION;
        let footer_cutoff = page.page_height * (1.0 - FOOTER_BAND_FRACTION);
        for line in &page.projected_lines {
            let text = line.text.trim();
            if text.is_empty() {
                continue;
            }
            let norm = normalize_for_repetition(text);
            if norm.is_empty() {
                continue;
            }
            // Header band: top of line within the top band.
            if line.bbox.y <= header_cutoff {
                counts
                    .entry(('h', norm.clone()))
                    .or_default()
                    .insert(page.page_number);
            }
            // Footer band: bottom of line at or below the footer cutoff.
            let line_bottom = line.bbox.y + line.bbox.height;
            if line_bottom >= footer_cutoff {
                counts
                    .entry(('f', norm))
                    .or_default()
                    .insert(page.page_number);
            }
        }
    }
    let threshold = (pages.len() as f32 * HEADER_FOOTER_MIN_FRACTION)
        .ceil()
        .max(HEADER_FOOTER_MIN_PAGES as f32) as usize;
    for ((_, norm), pages_seen) in counts {
        if pages_seen.len() >= threshold {
            set.insert(norm);
        }
    }
    set
}

/// Returns true if `line` (located on `page`) matches the running
/// header/footer set: the line sits in the top or bottom band AND its
/// normalized text is in `header_footer`.
pub(super) fn is_header_or_footer(
    line: &ProjectedLine,
    page: &ParsedPage,
    header_footer: &std::collections::HashSet<String>,
) -> bool {
    if header_footer.is_empty() {
        return false;
    }
    let header_cutoff = page.page_height * HEADER_BAND_FRACTION;
    let footer_cutoff = page.page_height * (1.0 - FOOTER_BAND_FRACTION);
    let in_band = line.bbox.y <= header_cutoff || line.bbox.y + line.bbox.height >= footer_cutoff;
    if !in_band {
        return false;
    }
    let norm = normalize_for_repetition(line.text.trim());
    header_footer.contains(&norm)
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers::header_footer_page;
    use super::*;

    #[test]
    fn normalize_collapses_digits_and_case() {
        assert_eq!(normalize_for_repetition("Page 1 of 6"), "page # of #");
        assert_eq!(normalize_for_repetition("PAGE 12 OF 6"), "page # of #");
        assert_eq!(normalize_for_repetition("Confidential"), "confidential");
    }

    #[test]
    fn detects_repeating_header_and_footer() {
        let pages = vec![
            header_footer_page(1, "Acme Confidential", "Page 1 of 3", "Body one."),
            header_footer_page(2, "Acme Confidential", "Page 2 of 3", "Body two."),
            header_footer_page(3, "Acme Confidential", "Page 3 of 3", "Body three."),
        ];
        let set = compute_header_footer_set(&pages);
        assert!(set.contains("acme confidential"));
        assert!(set.contains("page # of #"));
    }

    #[test]
    fn skips_repetition_check_on_single_page() {
        let pages = vec![header_footer_page(1, "Solo", "footer", "body")];
        let set = compute_header_footer_set(&pages);
        assert!(set.is_empty());
    }

    #[test]
    fn body_text_not_classified_as_header() {
        // Same text in the body of every page should NOT be stripped — only
        // text within the top/bottom band qualifies.
        let mut pages = Vec::new();
        for n in 1..=3 {
            let mut p = header_footer_page(n, "unique header", "unique footer", "shared body text");
            // Move "shared body text" out of header/footer bands (already at y=50 in mid-page).
            // No-op — just illustrating intent.
            p.projected_lines[0].text = format!("unique header {n}");
            p.projected_lines[2].text = format!("unique footer {n}");
            pages.push(p);
        }
        let set = compute_header_footer_set(&pages);
        // "shared body text" sits mid-page — never matched.
        assert!(!set.contains("shared body text"));
    }
}
