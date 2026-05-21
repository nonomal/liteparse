# pypdfTest

Test harness for LiteParse's `--format pypdf` output mode.

`lit parse <file> --format pypdf` reconstructs plain text in a way that
closely emulates [`pypdf`](https://github.com/py-pdf/pypdf)'s
`extract_text()` "plain" mode. It skips OCR and grid projection and instead
walks PDFium characters in content-stream order, applying pypdf's newline /
space heuristics (see `crates/liteparse/src/extract.rs` → `pypdf_page_text`).

Because LiteParse extracts via PDFium while pypdf parses the raw content
stream, output is **not** byte-identical — but line breaks and word grouping
track closely.

## Running

Requires [`uv`](https://docs.astral.sh/uv/) (pypdf is declared inline in the
script via PEP 723 — `uv` installs it into an ephemeral environment).

First build the CLI:

```bash
cargo build --release -p liteparse --bin lit
```

Then compare against pypdf:

```bash
# whole finance dataset
uv run pypdfTest/compare.py

# a random sample of 30, dumping per-file text + diffs for inspection
uv run pypdfTest/compare.py --sample 30 --seed 1 --dump /tmp/pypdfcmp

# a different dataset directory
uv run pypdfTest/compare.py /path/to/pdfs
```

## Metrics

For each PDF the harness reports a `difflib` similarity ratio (0..1):

- **line** — ratio over the sequence of non-blank lines (whitespace within each
  line collapsed) — measures line-breaking agreement.
- **word** — ratio over the whitespace-split word sequence; content agreement,
  independent of spacing / line-break noise.

## Results

On the 125-PDF finance dataset (`pdfDataSetOrdered/finance`):

| metric | mean  | median | p10   |
|--------|-------|--------|-------|
| word   | 0.921 | 0.978  | 0.780 |
| line   | 0.792 | 0.867  | 0.502 |

88% of files land at word ≥ 0.85 ("good" or "excellent"). The low-scoring
outliers fall into two groups, neither a LiteParse defect:

1. **Degenerate PDFs** — fonts with no usable `ToUnicode` map; *both* pypdf and
   LiteParse emit gibberish, and the two gibberish strings differ.
2. **pypdf failure cases** — pypdf letter-spaces every glyph
   (`D i s c l a i m e r`) or jams words/lines together (`2005Oslo`);
   LiteParse produces the correct, readable text and so "disagrees".

The one unavoidable systematic difference is dot leaders: pypdf renders them
`. . . .` (one token per dot) while LiteParse keeps `....` — the leader dots
have zero positional gap, so the spacing is not recoverable from geometry.

## Tuning

Two heuristics drive the reconstruction; both can be overridden at runtime
without rebuilding, which is useful when sweeping for the best values:

| Env var                   | Default | Meaning                                                       |
|---------------------------|---------|---------------------------------------------------------------|
| `LITEPARSE_PYPDF_NL_K`    | `0.65`  | newline when the line moves vertically by > `k × line height` |
| `LITEPARSE_PYPDF_SPACE_K` | `0.18`  | space when the gap to the previous glyph is ≥ `k × line height`|

`LITEPARSE_PYPDF_DEBUG=1` prints per-character geometry to stderr.
