from pytest_benchmark.fixture import BenchmarkFixture

from .conftest import parse_liteparse, parse_pdftotext, parse_pymupdf

PATH = "./dataset/250_pages.pdf"


def test_liteparse_250pages(benchmark: BenchmarkFixture) -> None:
    benchmark(parse_liteparse, PATH)


def test_pdftotext_250pages(benchmark: BenchmarkFixture) -> None:
    benchmark(parse_pdftotext, PATH)


def test_pymupdf_250pages(benchmark: BenchmarkFixture) -> None:
    benchmark(parse_pymupdf, PATH)
