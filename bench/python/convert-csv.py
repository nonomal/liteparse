import csv
import json
from argparse import ArgumentParser


def convert(path: str) -> None:
    with open(path) as f:
        data = json.load(f)

    with open(path.replace(".json", ".csv"), "w", newline="") as f:
        writer = csv.DictWriter(
            f, fieldnames=["name", "min", "max", "mean", "stddev", "rounds"]
        )
        writer.writeheader()
        for b in data["benchmarks"]:
            writer.writerow(
                {
                    "name": b["name"],
                    "min": b["stats"]["min"],
                    "max": b["stats"]["max"],
                    "mean": b["stats"]["mean"],
                    "stddev": b["stats"]["stddev"],
                    "rounds": b["stats"]["rounds"],
                }
            )


if __name__ == "__main__":
    parser = ArgumentParser()
    parser.add_argument("path", help="Path to convert")
    args = parser.parse_args()
    convert(args.path)
