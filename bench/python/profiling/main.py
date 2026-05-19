import gc
import timeit
from pathlib import Path

from _liteparse import LiteParse  # type: ignore

DATASET_DIR = Path(__file__).resolve().parent.parent / "python" / "dataset"
DATASET_FILES = ["1_page.pdf", "24_pages.pdf", "60_pages.pdf"]
REPEAT = 3
NUMBER = 5

PARSER = LiteParse(ocr_enabled=False, quiet=True, output_format="json")


def parse_liteparse(data: bytes) -> None:
    PARSER.parse_bytes(data)


def parse_liteparse_path(path: str) -> None:
    PARSER.parse(path)


def profile(path: Path) -> dict:
    data = path.read_bytes()

    gc.collect()
    timer = timeit.Timer(lambda: parse_liteparse(data))
    times = timer.repeat(repeat=REPEAT, number=NUMBER)
    per_call = [t / NUMBER for t in times]

    return {
        "file": path.name,
        "size_kb": path.stat().st_size / 1024,
        "best_s": min(per_call),
        "mean_s": sum(per_call) / len(per_call),
    }


def profile_on_path(path: Path) -> dict:
    gc.collect()
    timer = timeit.Timer(lambda: parse_liteparse_path(str(path)))
    times = timer.repeat(repeat=REPEAT, number=NUMBER)
    per_call = [t / NUMBER for t in times]

    return {
        "file": path.name,
        "size_kb": path.stat().st_size / 1024,
        "best_s": min(per_call),
        "mean_s": sum(per_call) / len(per_call),
    }


def run_bytes() -> None:
    print(f"Dataset: {DATASET_DIR}")
    print(f"Repeats: {REPEAT} x {NUMBER} calls each\n")
    header = f"{'file':<16}{'size(KB)':>12}{'best(s)':>12}{'mean(s)':>12}"
    print(header)
    print("-" * len(header))
    for name in DATASET_FILES:
        path = DATASET_DIR / name
        if not path.exists():
            print(f"{name:<16}  <missing>")
            continue
        r = profile(path)
        print(
            f"{r['file']:<16}{r['size_kb']:>12.1f}"
            f"{r['best_s']:>12.4f}{r['mean_s']:>12.4f}"
        )


def run_paths() -> None:
    print(f"Dataset: {DATASET_DIR}")
    print(f"Repeats: {REPEAT} x {NUMBER} calls each\n")
    header = f"{'file':<16}{'size(KB)':>12}{'best(s)':>12}{'mean(s)':>12}"
    print(header)
    print("-" * len(header))
    for name in DATASET_FILES:
        path = DATASET_DIR / name
        if not path.exists():
            print(f"{name:<16}  <missing>")
            continue
        r = profile_on_path(path)
        print(
            f"{r['file']:<16}{r['size_kb']:>12.1f}"
            f"{r['best_s']:>12.4f}{r['mean_s']:>12.4f}"
        )


if __name__ == "__main__":
    print("BYTES")
    print("---")
    print()
    run_bytes()
    print()
    print("PATHS")
    print("---")
    print()
    run_paths()
