# /// script
# requires-python = ">=3.9"
# dependencies = ["pypdf==6.1.3", "cryptography>=3.1"]
# ///
"""
pypdf extraction worker — one PDF per invocation.

Run as a short-lived subprocess so the parent harness can enforce a timeout
(pypdf's `extract_text()` is CPU-bound and pathologically slow on some PDFs).

Usage:  python pypdf_worker.py <pdf-path>
Prints the extracted text (pages joined by newline) to stdout.
"""

import sys

from pypdf import PdfReader


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: pypdf_worker.py <pdf-path>", file=sys.stderr)
        return 2
    reader = PdfReader(sys.argv[1])
    text = "\n".join(page.extract_text() for page in reader.pages)
    sys.stdout.write(text)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
