use crate::types::{Anchor, ProjectedLine};

use super::inline::{SpanStyle, line_uniform_style, render_line_inline};

/// Multiplier on line height used as the paragraph-break threshold.
pub(super) const PARAGRAPH_GAP_MULTIPLIER: f32 = 1.5;

/// Tolerance for treating two font sizes as "the same" when grouping
/// paragraph lines. Generous because we sometimes derive the "size" from
/// `bbox.height`, which varies a few points line-to-line based on whether
/// the glyphs include descenders (`y`, `g`, `p`).
pub(super) const FONT_SIZE_PARAGRAPH_TOLERANCE: f32 = 2.5;

/// Tolerance in points for treating two indent positions as "the same column".
pub(super) const INDENT_TOLERANCE: f32 = 6.0;

/// Collapse runs of whitespace into single spaces. The projected text from
/// `projection.rs` pads with column-alignment spaces (e.g. `for    instance`)
/// which look fine as a layout grid but are wrong for prose.
pub(super) fn collapse_whitespace(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_space = false;
    for c in s.chars() {
        if c.is_whitespace() {
            if !prev_space && !out.is_empty() {
                out.push(' ');
            }
            prev_space = true;
        } else {
            out.push(c);
            prev_space = false;
        }
    }
    if out.ends_with(' ') {
        out.pop();
    }
    out
}

/// Append `to_append` onto `prev`, de-hyphenating across the boundary. When
/// `prev` ends with `-` and `check` (the plain text of the continuation) starts
/// with an ASCII lowercase letter, the trailing hyphen is dropped and the text
/// concatenated directly (`co-` + `operate` → `cooperate`); otherwise the join
/// is a single space.
///
/// `check` and `to_append` are separate so a caller tracking a styled `inline`
/// representation can test the condition against the raw text while appending
/// the inline-rendered chunk. Plain-text callers pass the same string twice.
pub(super) fn dehyphenate_join(prev: &mut String, check: &str, to_append: &str) {
    if check.is_empty() {
        return;
    }
    if prev.is_empty() {
        prev.push_str(to_append);
        return;
    }
    if prev.ends_with('-') && check.chars().next().is_some_and(|c| c.is_ascii_lowercase()) {
        prev.pop();
        prev.push_str(to_append);
    } else {
        prev.push(' ');
        prev.push_str(to_append);
    }
}

/// Decide whether `cur` continues the paragraph started by `prev`.
pub(super) fn continues_paragraph(prev: &ProjectedLine, cur: &ProjectedLine) -> bool {
    // Anchor only signals a paragraph break when one of the lines is clearly
    // centered while the other isn't — justified prose routinely alternates
    // between Left / Right / Floating dominant anchors as line widths flex,
    // and treating those as paragraph breaks shreds normal text.
    let centered_mismatch = (prev.anchor == Anchor::Center) ^ (cur.anchor == Anchor::Center);
    if centered_mismatch {
        return false;
    }
    if (prev.dominant_font_size - cur.dominant_font_size).abs() > FONT_SIZE_PARAGRAPH_TOLERANCE {
        return false;
    }
    // Uniform-bold ↔ non-bold transition is a paragraph break. Catches
    // body-size headings that share font size with the surrounding prose but
    // are emitted in a bold variant (e.g. Brill-Bold over Brill-Roman). Both
    // sides must have a uniform style for this to fire; mid-line emphasis
    // (None style) falls through to the gap/indent checks so prose with
    // mid-paragraph bold spans doesn't get fragmented.
    if let (Some(p), Some(c)) = (line_uniform_style(prev), line_uniform_style(cur))
        && p.bold != c.bold
    {
        return false;
    }
    if prev.region_path != cur.region_path {
        // Cross-region continuation: the same paragraph can wrap from the
        // bottom of one column into the top of the next. Only bridge regions
        // when the previous line clearly breaks mid-sentence (no terminal
        // punctuation) AND the next line starts with a lowercase letter — a
        // strict signal that catches the column-wrap case while rejecting
        // unrelated paragraphs that happen to sit in adjacent leaves.
        let prev_trim = prev.text.trim_end();
        let ends_open = !prev_trim.ends_with(|c: char| {
            matches!(
                c,
                '.' | '!' | '?' | ':' | ';' | '”' | '"' | ')' | ']' | '。' | '』' | '」'
            )
        });
        let starts_lower = cur
            .text
            .trim_start()
            .chars()
            .next()
            .is_some_and(|c| c.is_lowercase());
        return ends_open && starts_lower;
    }
    if (prev.indent_x - cur.indent_x).abs() > INDENT_TOLERANCE && cur.anchor == Anchor::Left {
        // Indent change on a left-aligned block usually means a new paragraph
        // (block-quote, list, indented passage, etc.). Allow first-line indent
        // by checking only when the *next* line shifts right relative to prev.
        if cur.indent_x > prev.indent_x + INDENT_TOLERANCE {
            return false;
        }
    }
    // Vertical gap check.
    let prev_bottom = prev.bbox.y + prev.bbox.height;
    let gap = cur.bbox.y - prev_bottom;
    let line_height = prev.bbox.height.max(cur.bbox.height).max(1.0);
    gap <= line_height * PARAGRAPH_GAP_MULTIPLIER
}

/// Paragraph accumulator state. We track two parallel representations of the
/// running paragraph text:
///
/// - `raw` — plain text (no emphasis markers). Used for the paragraph-uniform
///   shortcut: if every contributing line had the same uniform style, we wrap
///   the whole paragraph once with `wrap_emphasis(raw, …)` to avoid the
///   `**foo** **bar** **baz**` per-line noise pymupdf4llm warns about.
/// - `inline` — per-line markdown with emphasis baked in via
///   `render_line_inline`. Used when the paragraph contains mid-line emphasis
///   shifts or lines with differing uniform styles.
///
/// `uniform` is `Some((bold, italic))` while every line so far has been a
/// uniformly-styled line sharing the same (bold, italic) flags, and `None` as
/// soon as that invariant breaks.
pub(super) struct ParaAccum {
    pub(super) raw: String,
    pub(super) inline: String,
    pub(super) last: ProjectedLine,
    pub(super) uniform: Option<(bool, bool)>,
}

/// Append `next_line` to a paragraph accumulator. Maintains both the `raw` and
/// `inline` text representations and updates the running `uniform` flag.
/// De-hyphenation runs on the `raw` boundary; the `inline` boundary mirrors it
/// when the trailing char is still a literal `-` (i.e. the hyphen sits outside
/// any emphasis wrap — the common case).
pub(super) fn append_to_paragraph(accum: &mut ParaAccum, next_line: &ProjectedLine) {
    let next_raw = collapse_whitespace(next_line.text.trim());
    if next_raw.is_empty() {
        return;
    }
    let next_inline = render_line_inline(next_line);
    let next_uniform: Option<SpanStyle> = line_uniform_style(next_line);

    if accum.raw.is_empty() {
        accum.raw.push_str(&next_raw);
        accum.inline.push_str(&next_inline);
        accum.uniform = next_uniform.map(|s| (s.bold, s.italic));
        accum.last = next_line.clone();
        return;
    }

    // Raw side de-hyphenates against its own boundary. The inline side keys off
    // the same raw lowercase test but checks *its own* trailing char: a hyphen
    // tucked inside an emphasis wrap ends in `*`/`` ` `` rather than `-`, so it
    // won't strip and falls through to a space join — exactly the prior
    // behavior, now via one helper.
    dehyphenate_join(&mut accum.raw, &next_raw, &next_raw);
    dehyphenate_join(&mut accum.inline, &next_raw, &next_inline);

    // Maintain the running uniform-style flag.
    accum.uniform = match (accum.uniform, next_uniform) {
        (Some(cur), Some(s)) if cur == (s.bold, s.italic) => Some(cur),
        _ => None,
    };
    accum.last = next_line.clone();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dehyphenate_join_only_strips_before_lowercase() {
        let mut s = String::from("co-");
        dehyphenate_join(&mut s, "operate", "operate");
        assert_eq!(s, "cooperate");

        let mut s = String::from("Vitamin-");
        dehyphenate_join(&mut s, "A", "A");
        assert_eq!(s, "Vitamin- A");
    }
}
