import pdftotext
import pymupdf
from _liteparse import LiteParse  # type: ignore

LITEPARSE_PARSER = LiteParse(ocr_enabled=False)


def parse_liteparse(path: str) -> None:
    with open(path, "rb") as f:
        LITEPARSE_PARSER.parse_bytes(f.read())


def parse_pdftotext(path: str) -> None:
    with open(path, "rb") as f:
        pdf = pdftotext.PDF(f)
    "\n\n".join(pdf)


def parse_pymupdf(path: str) -> None:
    with pymupdf.open(path) as doc:
        "\n\n".join(page.get_text() for page in doc)  # type: ignore
