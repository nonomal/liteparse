use pdfium::{Library, TextPage};
use crate::types::{TextItem, Page};

/// Extract pages from a PDF file and return them as structured data.
pub fn extract_pages(pdf_path: &str, page_num: Option<u32>) -> Result<Vec<Page>, Box<dyn std::error::Error>> {
    let lib = Library::init();
    let document = lib.load_document(pdf_path, None)?;
    let page_count = document.page_count();
    let mut pages = Vec::new();

    for page_index in 0..page_count {
        if let Some(target_page) = page_num {
            if page_index as u32 + 1 != target_page {
                continue;
            }
        }

        let page = document.page(page_index)?;
        let text_page = page.text()?;
        let mut text_items = extract_page_text_items(&text_page)?;
        let page_height = page.height();

        // Convert from pdfium bottom-left origin to top-left origin
        for item in &mut text_items {
            item.y = page_height - item.y - item.height;
        }

        pages.push(Page {
            page_number: (page_index + 1) as usize,
            page_width: page.width(),
            page_height,
            text_items,
        });
    }

    Ok(pages)
}

/// Extract raw text items and print each page as a JSON-line object to stdout.
pub fn extract(pdf_path: &str, page_num: Option<u32>) -> Result<(), Box<dyn std::error::Error>> {
    let pages = extract_pages(pdf_path, page_num)?;
    for page in &pages {
        println!("{}", serde_json::to_string(page)?);
    }
    Ok(())
}

/// Character-level text extraction.
///
/// Instead of using PDFium's rect API (which splits text at every font attribute
/// change), we iterate through individual characters and group them by spatial
/// proximity. This keeps words like "A-MEM" together even when internal characters
/// have different font sizes (e.g. small-caps), and keeps punctuation attached to
/// adjacent text (e.g. citation commas/semicolons).
///
/// Segments break at:
/// - Line changes (large vertical shift)
/// - Column breaks (large horizontal gap)
/// - Explicit newline characters
fn extract_page_text_items(text_page: &TextPage) -> Result<Vec<TextItem>, Box<dyn std::error::Error>> {
    let char_count = text_page.char_count();
    if char_count <= 0 {
        return Ok(Vec::new());
    }

    // Hard limit: gaps larger than this always cause a split (column breaks).
    const MAX_INLINE_GAP: f64 = 15.0;

    let mut items: Vec<TextItem> = Vec::new();
    let mut seg = SegmentBuilder::new();

    for i in 0..char_count {
        let Some(ch) = text_page.char_at(i) else { continue };
        let unicode = ch.unicode();
        let is_generated = ch.is_generated();

        // Skip null / invalid sentinels
        if unicode == 0 || unicode == 0xFFFE || unicode == 0xFFFF {
            continue;
        }

        // Map to a Rust char, with special-case replacements
        let c = match unicode {
            0x02 => '-',  // STX → hyphen (common in some PDF encodings)
            _ => match char::from_u32(unicode) {
                Some(c) => c,
                None => continue,
            },
        };

        // Newlines: flush the current segment
        if c == '\n' || c == '\r' {
            seg.flush(&mut items);
            continue;
        }

        // Spaces: mark that we're in a pending-space state.
        // The space will be committed (or cause a split) when the next
        // visible character arrives and we can measure the actual gap.
        if c == ' ' {
            seg.mark_pending_space();
            continue;
        }

        // Skip non-space generated characters (synthetic glyphs)
        if is_generated {
            continue;
        }

        // Get the character's bounding box (PDF coordinates: bottom-left origin)
        let Some(cb) = ch.char_box() else { continue };

        if seg.has_content {
            // Check whether this character belongs to the current segment.
            // Use actual bounding box overlap with a small tolerance (2pt)
            // to handle floating point imprecision while still keeping
            // separate lines apart (~3pt gap between 10pt lines).
            // Superscripts/subscripts naturally overlap the baseline bbox.
            let y_tolerance = 2.0;
            let y_overlap = cb.bottom < seg.top + y_tolerance
                && cb.top > seg.bottom - y_tolerance;

            let gap = cb.left - seg.last_char_right;

            if !y_overlap || gap >= MAX_INLINE_GAP {
                // Different line or huge gap — always split.
                seg.flush(&mut items);
                seg.start(c, &cb, &ch);
            } else if seg.pending_space {
                // There was a space before this character. Decide whether
                // to keep it in the same segment or split.
                // Split when the gap is large relative to the avg char width —
                // this separates table columns while keeping body text words
                // together.
                let avg_cw = seg.avg_char_width();
                if gap > avg_cw * 1.6 {
                    seg.flush(&mut items);
                    seg.start(c, &cb, &ch);
                } else {
                    seg.commit_pending_space();
                    seg.push_char(c, &cb);
                }
            } else {
                seg.push_char(c, &cb);
            }
        } else {
            seg.start(c, &cb, &ch);
        }
    }

    seg.flush(&mut items);
    Ok(items)
}

/// Accumulates characters into a single TextItem segment.
struct SegmentBuilder {
    text: String,
    // Bounding box in PDF coordinates (bottom-left origin)
    left: f64,
    right: f64,
    bottom: f64,
    top: f64,
    // Right edge of the last non-space character (for gap calculation)
    last_char_right: f64,
    // Count of non-space characters (for avg width calculation)
    char_count: usize,
    // Font metadata (captured from the first character)
    font_name: Option<String>,
    font_size: f32,
    rotation_deg: f32,
    has_content: bool,
    // True when we've seen a space but haven't yet decided whether to
    // commit it (keep in segment) or split on it.
    pending_space: bool,
}

impl SegmentBuilder {
    fn new() -> Self {
        Self {
            text: String::new(),
            left: f64::MAX,
            right: f64::MIN,
            bottom: f64::MAX,
            top: f64::MIN,
            last_char_right: f64::MIN,
            char_count: 0,
            font_name: None,
            font_size: 0.0,
            rotation_deg: 0.0,
            has_content: false,
            pending_space: false,
        }
    }

    /// Average width of non-space characters in the current segment.
    fn avg_char_width(&self) -> f64 {
        if self.char_count == 0 {
            return 5.0; // sensible default
        }
        (self.right - self.left) / self.char_count as f64
    }

    /// Start a new segment with the given character.
    fn start(&mut self, c: char, cb: &pdfium::CharBox, ch: &pdfium::TextChar) {
        self.text.clear();
        self.text.push(c);
        self.left = cb.left;
        self.right = cb.right;
        self.bottom = cb.bottom;
        self.top = cb.top;
        self.last_char_right = cb.right;
        self.char_count = 1;
        self.has_content = true;
        self.pending_space = false;

        self.font_name = ch.font_name();
        let fs = ch.font_size() as f32;
        self.font_size = if fs > 0.0 { fs } else { (cb.top - cb.bottom) as f32 };

        let angle_rad = ch.angle();
        self.rotation_deg = if angle_rad >= 0.0 {
            let mut deg = angle_rad.to_degrees();
            if deg < 0.0 { deg += 360.0; }
            deg
        } else {
            0.0
        };
    }

    /// Add a visible character to the current segment.
    fn push_char(&mut self, c: char, cb: &pdfium::CharBox) {
        self.text.push(c);
        self.left = self.left.min(cb.left);
        self.right = self.right.max(cb.right);
        self.bottom = self.bottom.min(cb.bottom);
        self.top = self.top.max(cb.top);
        self.last_char_right = cb.right;
        self.char_count += 1;
    }

    /// Record that a space was seen; the actual decision to include it
    /// or to split is deferred until the next visible character.
    fn mark_pending_space(&mut self) {
        if self.has_content {
            self.pending_space = true;
        }
    }

    /// Commit a pending space into the segment text.
    fn commit_pending_space(&mut self) {
        if self.pending_space {
            self.text.push(' ');
            self.pending_space = false;
        }
    }

    /// Flush the current segment into the items list and reset.
    fn flush(&mut self, items: &mut Vec<TextItem>) {
        if !self.has_content {
            return;
        }

        let trimmed = self.text.trim();
        if !trimmed.is_empty() {
            let height = (self.top - self.bottom) as f32;
            items.push(TextItem {
                text: trimmed.to_string(),
                x: self.left as f32,
                y: self.bottom as f32,
                width: (self.right - self.left) as f32,
                height,
                rotation: self.rotation_deg,
                font_name: self.font_name.clone(),
                font_size: Some(if self.font_size > 0.0 { self.font_size } else { height }),
            });
        }

        self.text.clear();
        self.left = f64::MAX;
        self.right = f64::MIN;
        self.bottom = f64::MAX;
        self.top = f64::MIN;
        self.last_char_right = f64::MIN;
        self.char_count = 0;
        self.font_name = None;
        self.font_size = 0.0;
        self.rotation_deg = 0.0;
        self.has_content = false;
        self.pending_space = false;
    }
}
