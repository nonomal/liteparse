pytest --benchmark-json=results_1page.json benchmarks/test_1page.py
pytest --benchmark-json=results_24pages.json benchmarks/test_24pages.py
pytest --benchmark-json=results_60pages.json benchmarks/test_60pages.py
pytest --benchmark-json=results_250pages.json benchmarks/test_250pages.py
uv run convert-csv.py results_1page.json
uv run convert-csv.py results_24pages.json
uv run convert-csv.py results_60pages.json
uv run convert-csv.py results_250pages.json
