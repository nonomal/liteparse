from pytest_benchmark.fixture import BenchmarkFixture

from .conftest import parse_liteparse, parse_pdftotext, parse_pymupdf

PATH = "./dataset/1_page.pdf"


def test_liteparse_1page(benchmark: BenchmarkFixture) -> None:
    benchmark(parse_liteparse, PATH)


def test_pdftotext_1page(benchmark: BenchmarkFixture) -> None:
    benchmark(parse_pdftotext, PATH)


def test_pymupdf_1page(benchmark: BenchmarkFixture) -> None:
    benchmark(parse_pymupdf, PATH)
