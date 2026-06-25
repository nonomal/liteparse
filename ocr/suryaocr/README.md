# Surya OCR Service

A FastAPI server wrapping [Surya OCR 2](https://github.com/datalab-to/surya) to
conform to the LiteParse OCR API specification (see `../../OCR_API_SPEC.md`).

Surya 2 is a multilingual OCR foundation model with strong accuracy across many
languages — a single model handles all languages with no per-language setup.

## Build and Run

```bash
# install and run (in one command)
uv run server.py
```

The first run downloads Surya model weights from Hugging Face and may take a few
minutes; weights are cached afterward.

## Inference backend (required)

Surya 2 is a VLM-backed model: text recognition runs through a separate
inference backend, which you must provide. Surya does **not** bundle it.

- **CPU / local (llama.cpp):** install the `llama-server` binary so Surya's
  `llamacpp` backend can spawn it:
  - macOS: `brew install llama.cpp`
  - Linux: `brew install llama.cpp`, or download a release from
    https://github.com/ggml-org/llama.cpp/releases and put `llama-server` on
    your `PATH` (or set `LLAMA_CPP_BINARY=/path/to/llama-server`).
- **GPU (vllm):** set `SURYA_INFERENCE_BACKEND=vllm` (requires a CUDA GPU).
- **External server:** point `SURYA_INFERENCE_URL` at an already-running Surya
  inference server to attach without spawning one locally.

Without a backend, startup succeeds and `GET /health` works, but `POST /ocr`
returns a 500 with "llama-server binary not found".

## Usage

The service exposes:

- `POST /ocr` — Perform OCR on an uploaded image
- `GET /health` — Health check

### Parameters

- `file` — Image file (multipart/form-data)
- `language` — Language code (accepted for API compatibility; **ignored**, since
  Surya 2 is multilingual)

### Example

```bash
curl -X POST -F "file=@image.png" http://localhost:8830/ocr
```

### Response Format

```json
{
  "results": [
    {
      "text": "recognized text",
      "bbox": [x1, y1, x2, y2],
      "confidence": 0.95,
      "polygon": [[x1, y1], [x2, y2], [x3, y3], [x4, y4]]
    }
  ]
}
```

Results are **block-level** (one entry per detected layout block), with each
block's HTML stripped to plain text. This conforms to the LiteParse OCR API spec.

## Supported Languages

Surya 2 is a single multilingual model — no `language` parameter is required
(the `language` field is accepted but ignored).

Per Surya's own benchmark, it scores an **87.2% overall pass rate across 91
languages**, with 38 of the 91 languages scoring ≥ 90% and 76 scoring ≥ 80%,
covering text accuracy, layout, tables, math, and reading order. It has strong
performance across both Latin and non-Latin scripts.

- Full 91-language results: https://github.com/datalab-to/surya/blob/master/static/docs/multilingual.md
- Project overview: https://github.com/datalab-to/surya

## Device / GPU

Surya auto-detects the best available device. Force a device with the
`TORCH_DEVICE` environment variable:

```bash
TORCH_DEVICE=cuda uv run server.py   # GPU
TORCH_DEVICE=cpu  uv run server.py   # CPU
```

## Use with LiteParse

```bash
lit parse document.pdf --ocr-server-url http://localhost:8830/ocr
```

Or in code:

```typescript
import { LiteParse } from 'liteparse';

const parser = new LiteParse({
  ocrServerUrl: 'http://localhost:8830/ocr',
});

const result = await parser.parse('document.pdf');
```

## Testing

```bash
uv run pytest test_server.py
```

Tests mock the Surya predictor, so they run without downloading any models.

## Notes

- First request/startup may be slow while models download.
- Default port is 8830 (easyocr 8828, paddleocr 8829).
- Output is block-granular; Surya 2 has no per-line text API.
