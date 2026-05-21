# /// script
# requires-python = ">=3.9"
# dependencies = ["pypdf==6.1.3", "cryptography>=3.1"]
# ///
"""
Compare LiteParse's `--format pypdf` output against real `pypdf` output.

LiteParse's pypdf output format is a best-effort emulation of pypdf's
`extract_text()` "plain" mode. This harness runs both extractors over a folder
of PDFs and reports how closely they agree.

Usage:
    uv run pypdfTest/compare.py [DATASET_DIR] [options]

    DATASET_DIR        folder to scan for *.pdf  (default: the finance dataset)
    --lit PATH         path to the `lit` binary (default: target/release/lit)
    --sample N         randomly sample N PDFs (default: all)
    --seed N           RNG seed for sampling (default: 0)
    --dump DIR         write per-file pypdf/lite/diff text into DIR
    --worst N          show the N least-similar files (default: 10)
    --timeout N        per-file timeout in seconds for each extractor (default: 60)
    --quiet            only print the final summary

Both extractors run as short-lived subprocesses so a slow/hanging PDF can be
timed out instead of stalling the whole run (pypdf's `extract_text()` is
CPU-bound and pathologically slow on some PDFs).

Metrics (difflib.SequenceMatcher ratio, 0..1):
    line     ratio over the sequence of non-blank lines — captures both content
             and line-breaking agreement
    word     ratio over the whitespace-split word sequence — content agreement,
             independent of spacing / line-break noise

Both ratios are computed over token sequences (lines / words) rather than raw
characters, so they stay fast even on large documents.
"""

from __future__ import annotations

import argparse
import difflib
import hashlib
import random
import re
import subprocess
import sys
import time
from pathlib import Path

DEFAULT_DATASET = "/Users/pierre/Code/pdfDataSetOrdered/finance"
DEFAULT_LIT = "target/release/lit"
WORKER = Path(__file__).with_name("pypdf_worker.py")

_WS = re.compile(r"\s+")

# difflib's SequenceMatcher.ratio() is O(n*m); cap token sequences so a single
# huge document (dense numeric tables can hit 60k+ tokens) can't stall the run.
# Comparing the first ~12k tokens is plenty representative for a similarity score.
MAX_TOKENS = 12000


def lines_of(text: str) -> list:
    """Non-blank lines, with each line's whitespace collapsed to single spaces.

    Intra-line spacing and indentation are the noisiest part of the output
    (they depend on whether a PDF stores gaps as literal spaces or positioning
    operators), so the line metric collapses them and focuses on what matters:
    where line breaks land and what content sits on each line.
    """
    out = []
    for ln in text.splitlines():
        norm = _WS.sub(" ", ln).strip()
        if norm:
            out.append(norm)
    return out


def words_of(text: str) -> list:
    """Whitespace-split word sequence."""
    return _WS.sub(" ", text).strip().split(" ") if text.strip() else []


def pypdf_text(path: Path, timeout: int, cache: Path | None) -> str:
    """Ground truth: pypdf plain-mode extraction (run in a worker subprocess).

    pypdf output never changes, so it is cached on disk — this makes threshold
    sweeps (which only change LiteParse's output) fast.
    """
    cache_file = None
    if cache is not None:
        key = hashlib.sha1(str(path.resolve()).encode()).hexdigest()[:16]
        cache_file = cache / f"{key}.txt"
        if cache_file.exists():
            return cache_file.read_text()

    result = subprocess.run(
        [sys.executable, str(WORKER), str(path)],
        capture_output=True,
        text=True,
        timeout=timeout,
    )
    if result.returncode != 0:
        raise RuntimeError(result.stderr.strip() or "pypdf worker exited non-zero")
    if cache_file is not None:
        cache_file.write_text(result.stdout)
    return result.stdout


def lit_text(lit: str, path: Path, timeout: int) -> str:
    """LiteParse `--format pypdf` output."""
    result = subprocess.run(
        [lit, "parse", str(path), "--format", "pypdf", "--quiet"],
        capture_output=True,
        text=True,
        timeout=timeout,
    )
    if result.returncode != 0:
        raise RuntimeError(result.stderr.strip() or "lit exited non-zero")
    # `lit` prints the formatted output followed by one trailing newline.
    return result.stdout


def seq_ratio(a: list, b: list) -> float:
    """SequenceMatcher ratio over token lists, capped at MAX_TOKENS."""
    if not a and not b:
        return 1.0
    a, b = a[:MAX_TOKENS], b[:MAX_TOKENS]
    return difflib.SequenceMatcher(None, a, b, autojunk=False).ratio()


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("dataset", nargs="?", default=DEFAULT_DATASET)
    ap.add_argument("--lit", default=DEFAULT_LIT)
    ap.add_argument("--sample", type=int, default=0)
    ap.add_argument("--seed", type=int, default=0)
    ap.add_argument("--dump", default=None)
    ap.add_argument("--worst", type=int, default=10)
    ap.add_argument("--timeout", type=int, default=60)
    ap.add_argument("--cache", default="/tmp/pypdf_cache",
                    help="dir to cache pypdf output (speeds up repeated runs)")
    ap.add_argument("--quiet", action="store_true")
    args = ap.parse_args()

    dataset = Path(args.dataset)
    pdfs = sorted(dataset.rglob("*.pdf"))
    if not pdfs:
        print(f"no PDFs found under {dataset}", file=sys.stderr)
        return 1

    if args.sample and args.sample < len(pdfs):
        random.Random(args.seed).shuffle(pdfs)
        pdfs = sorted(pdfs[: args.sample])

    dump_dir = Path(args.dump) if args.dump else None
    if dump_dir:
        dump_dir.mkdir(parents=True, exist_ok=True)

    cache = Path(args.cache) if args.cache else None
    if cache:
        cache.mkdir(parents=True, exist_ok=True)

    rows = []  # (name, line_ratio, word_ratio)
    errors = []  # (name, message)
    t0 = time.time()

    for i, pdf in enumerate(pdfs, 1):
        try:
            gt = pypdf_text(pdf, args.timeout, cache)
            lt = lit_text(args.lit, pdf, args.timeout)
        except subprocess.TimeoutExpired:
            errors.append((pdf.name, "timeout"))
            if not args.quiet:
                print(f"[{i}/{len(pdfs)}] {pdf.name}: TIMEOUT")
            continue
        except Exception as exc:  # noqa: BLE001
            errors.append((pdf.name, str(exc)))
            if not args.quiet:
                print(f"[{i}/{len(pdfs)}] {pdf.name}: ERROR {exc}")
            continue

        ln = seq_ratio(lines_of(gt), lines_of(lt))
        wd = seq_ratio(words_of(gt), words_of(lt))
        rows.append((pdf.name, ln, wd))

        if dump_dir:
            stem = dump_dir / pdf.stem
            stem.with_suffix(".pypdf.txt").write_text(gt)
            stem.with_suffix(".lite.txt").write_text(lt)
            diff = difflib.unified_diff(
                gt.splitlines(keepends=True),
                lt.splitlines(keepends=True),
                fromfile="pypdf",
                tofile="liteparse",
                n=2,
            )
            stem.with_suffix(".diff").write_text("".join(diff))

        if not args.quiet:
            print(f"[{i}/{len(pdfs)}] {pdf.name}: line={ln:.3f} word={wd:.3f}",
                  flush=True)

    elapsed = time.time() - t0
    print()
    print("=" * 64)
    print(f"dataset      : {dataset}")
    print(f"pdfs         : {len(pdfs)}  ({len(rows)} ok, {len(errors)} errors)")
    print(f"elapsed      : {elapsed:.1f}s")
    if rows:
        ln_vals = sorted(r[1] for r in rows)
        wd_vals = sorted(r[2] for r in rows)

        def mean(v):
            return sum(v) / len(v)

        def pct(v, p):
            return v[min(len(v) - 1, int(p * len(v)))]

        print(f"line ratio   : mean={mean(ln_vals):.3f}  "
              f"median={pct(ln_vals, 0.5):.3f}  p10={pct(ln_vals, 0.1):.3f}")
        print(f"word ratio   : mean={mean(wd_vals):.3f}  "
              f"median={pct(wd_vals, 0.5):.3f}  p10={pct(wd_vals, 0.1):.3f}")
        for label, lo, hi in [("excellent", 0.95, 1.01),
                              ("good", 0.85, 0.95),
                              ("fair", 0.70, 0.85),
                              ("poor", 0.0, 0.70)]:
            n = sum(1 for v in wd_vals if lo <= v < hi)
            print(f"  word {label:<10}: {n:3d}  ({100 * n / len(wd_vals):.0f}%)")

        worst = sorted(rows, key=lambda r: r[2])[: args.worst]
        print(f"\nworst {len(worst)} by word ratio:")
        for name, ln, wd in worst:
            print(f"  word={wd:.3f} line={ln:.3f}  {name}")

    if errors:
        print(f"\n{len(errors)} errors:")
        for name, msg in errors[:20]:
            print(f"  {name}: {msg}")
    print("=" * 64)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
