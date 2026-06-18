---
title: Markdown Output
description: Render documents to clean, structured Markdown for LLMs and RAG pipelines.
sidebar:
  order: 2
---

LiteParse can render documents directly to Markdown, reconstructing headings,
tables, lists, images, and links from the spatial layout. This is ideal for
feeding documents to LLMs and RAG pipelines, where clean, structured text
matters more than exact visual fidelity.

Markdown is a first-class output format alongside `text` and `json`.

## CLI

```bash
# Render a document to Markdown
lit parse document.pdf --format markdown -o output.md

# Print Markdown to stdout
lit parse document.pdf --format markdown
```

### Images

By default, raster images are emitted as Markdown placeholders
(`![](image_pN_K.png)`) in reading order. Control this with `--image-mode`:

| Mode | Behavior |
|------|----------|
| `placeholder` (default) | Emit `![](image_pN_K.png)` references in reading order |
| `off` | Strip images entirely |
| `embed` | Write each image's PNG bytes to `--image-output-dir` and reference them |

```bash
# Strip images
lit parse document.pdf --format markdown --image-mode off

# Extract embedded images to disk and reference them from the markdown
lit parse document.pdf --format markdown --image-mode embed --image-output-dir ./images
```

### Links

Hyperlink annotations are rendered as `[text](url)` by default. Pass
`--no-links` to emit the anchor text as plain text instead:

```bash
lit parse document.pdf --format markdown --no-links
```

## Library

The rendered Markdown is returned on `result.text`.

<Tabs>
<TabItem value="typescript" label="TypeScript">

```typescript
import { LiteParse } from "@llamaindex/liteparse";

const parser = new LiteParse({
  outputFormat: "markdown",   // "json" | "text" | "markdown"
  imageMode: "placeholder",   // "placeholder" | "off" | "embed" (default: "placeholder")
  extractLinks: true,         // render [text](url) link syntax (default: true)
});
const result = await parser.parse("document.pdf");
console.log(result.text); // rendered Markdown
```

</TabItem>
<TabItem value="python" label="Python">

```python
from liteparse import LiteParse

parser = LiteParse(
    output_format="markdown",   # "json" | "text" | "markdown"
    image_mode="placeholder",   # "placeholder" | "off" | "embed"
    extract_links=True,         # render [text](url) link syntax (default: True)
)
result = parser.parse("document.pdf")
print(result.text)  # rendered Markdown
```
</TabItem>
<TabItem value="rust" label="Rust">

```rust
use liteparse::config::{ImageMode, LiteParseConfig, OutputFormat};
use liteparse::LiteParse;

let config = LiteParseConfig {
    output_format: OutputFormat::Markdown,
    image_mode: ImageMode::Placeholder,
    extract_links: true,
    ..Default::default()
};
let result = LiteParse::new(config).parse("document.pdf").await?;
println!("{}", result.text); // rendered Markdown
```

</TabItem>
</Tabs>

## Quality notes

Markdown reconstruction quality varies with document complexity. LiteParse does
a strong job on typical documents, handling prose, headings, simple-to-moderate tables,
and lists. This runs entirely locally with no models using rule-based heuristics. For the hardest documents (dense or
multi-level tables, complex multi-column layouts, charts, and scans),
[LlamaParse](https://developers.llamaindex.ai/python/cloud/llamaparse/?utm_source=github&utm_medium=liteparse)
remains the most accurate option.
