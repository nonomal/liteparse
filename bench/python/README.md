# Python Benchmarks

Benchmarks and profiling for the `liteparse-python` bindings, comparing
LiteParse against [`pdftotext`](https://pypi.org/project/pdftotext/) and
[`PyMuPDF`](https://pypi.org/project/PyMuPDF/) across a range of document
sizes.

## Layout

```
bench/python/
├── benchmarks/      # pytest-benchmark test suites (1 / 24 / 60 / 250 pages)
├── dataset/         # Sample PDFs used as inputs
├── profiling/       # Scalene profiling scripts, one per parser × size
├── pyproject.toml   # uv project definition
├── run-bench.sh     # Run all benchmarks and emit JSON/CSV results
└── run-prof.sh      # Run Scalene over every profiling script
```

The `benchmarks/conftest.py` module defines the three `parse_*` helpers that
each test suite invokes via `pytest-benchmark`.

## Prerequisites

- Python `>=3.13, <3.14`
- [`uv`](https://github.com/astral-sh/uv) for environment management
- A locally built liteparse wheel at
  `../../target/wheels/liteparse_python-2.0.0-cp313-cp313-macosx_11_0_arm64.whl`
  (see the top-level repo for build instructions). Update the path in
  `pyproject.toml` if your platform/version differs.
- System libraries required by `pdftotext` (poppler) and `PyMuPDF`.

Install dependencies:

```sh
uv sync
```

## Running the benchmarks

```sh
./run-bench.sh
```

This produces `results_<size>.json` files via `pytest-benchmark` and converts
them to CSV with `convert-csv.py`.

To run a single suite:

```sh
uv run pytest benchmarks/test_24pages.py
```

## Profiling

```sh
./run-prof.sh
```

The script iterates over the profiling entry points and runs each one under
[Scalene](https://github.com/plasma-umass/scalene), writing a `<name>.json`
report next to each script. Individual scripts can also be run directly, e.g.:

```sh
uv run scalene profiling/liteparse_60.py --outfile profiling/liteparse_60.json
```

## Adding a new parser or document size

1. Add a `parse_<name>` helper in `benchmarks/conftest.py`.
2. Reference it from the relevant `benchmarks/test_<size>pages.py` file (or
   add a new test module and a corresponding line in `run-bench.sh`).
3. Add a matching `profiling/<parser>_<size>.py` script if you also want
   Scalene coverage.
4. Drop any new sample PDFs into `dataset/`.
