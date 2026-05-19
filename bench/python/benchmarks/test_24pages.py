from pytest_benchmark.fixture import BenchmarkFixture

from .conftest import parse_liteparse, parse_pdftotext, parse_pymupdf

PATH = "./dataset/24_pages.pdf"


def test_liteparse_24pages(benchmark: BenchmarkFixture) -> None:
    benchmark(parse_liteparse, PATH)


def test_pdftotext_24pages(benchmark: BenchmarkFixture) -> None:
    benchmark(parse_pdftotext, PATH)


def test_pymupdf_24pages(benchmark: BenchmarkFixture) -> None:
    benchmark(parse_pymupdf, PATH)
