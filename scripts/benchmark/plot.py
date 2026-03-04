# /// script
# requires-python = ">=3.13"
# dependencies = [
#     "matplotlib",
#     "numpy",
# ]
# ///
"""Generate an SVG benchmark chart from hyperfine JSON results.

Reads results/write.json and results/read.json (hyperfine --export-json format).

Usage:
    uv run scripts/benchmark/plot.py
"""

import json
import sys
from pathlib import Path

import matplotlib
import matplotlib.pyplot as plt
import numpy as np

matplotlib.use("Agg")

SCRIPT_DIR = Path(__file__).parent
RESULTS_DIR = SCRIPT_DIR / "results"

WRITE_COLOR = "#45744a"
READ_COLOR = "#3a7ca5"


def load_hyperfine(path: Path) -> dict:
    with open(path) as f:
        data = json.load(f)

    results = {}
    for entry in data["results"]:
        results[entry["command"]] = entry["median"] * 1000
    return results


def main() -> None:
    for name in ["write.json", "read.json"]:
        if not (RESULTS_DIR / name).exists():
            print(
                f"Error: {RESULTS_DIR / name} not found. Run benchmark_comparison.sh first.",
                file=sys.stderr,
            )
            sys.exit(1)

    write_data = load_hyperfine(RESULTS_DIR / "write.json")
    read_data = load_hyperfine(RESULTS_DIR / "read.json")

    labels = ["gdsr", "KLayout"]
    write_times = [write_data["gdsr"], write_data["klayout"]]
    read_times = [read_data["gdsr"], read_data["klayout"]]

    plt.style.use("dark_background")

    y_pos = np.arange(len(labels))

    fig, ax = plt.subplots(figsize=(8, 2.5))
    fig.patch.set_facecolor("black")
    ax.set_facecolor("black")

    ax.spines["top"].set_visible(False)
    ax.spines["right"].set_visible(False)
    ax.spines["left"].set_visible(False)
    ax.spines["bottom"].set_visible(True)
    ax.spines["bottom"].set_color("grey")
    ax.tick_params(
        axis="both",
        which="both",
        bottom=True,
        top=False,
        labelbottom=True,
        colors="grey",
    )

    bar_height = 0.5
    write_bars = ax.barh(
        y_pos, write_times, color=WRITE_COLOR, height=bar_height, label="write"
    )
    read_bars = ax.barh(
        y_pos,
        read_times,
        left=write_times,
        color=READ_COLOR,
        height=bar_height,
        label="read",
    )

    max_time = max(w + r for w, r in zip(write_times, read_times))
    ax.set_xlim(0, max_time * 1.15)
    linspace = np.linspace(0, np.ceil(max_time), 5)
    ax.set_xticks(linspace)
    ax.set_xticklabels([f"{x:.0f}ms" for x in linspace], color="grey")

    ax.set_yticks(y_pos)
    ax.set_yticklabels(labels, fontsize=14, color="grey")

    for i, (bw, br) in enumerate(zip(write_bars, read_bars)):
        w_val = bw.get_width()
        r_val = br.get_width()
        total = w_val + r_val
        y = bw.get_y() + bw.get_height() / 2.0

        ax.text(
            total + max_time * 0.01,
            y,
            f"{w_val:.1f}ms write / {r_val:.1f}ms read",
            ha="left",
            va="center",
            color="grey",
            fontsize=9,
        )

    ax.legend(
        [write_bars, read_bars],
        ["write", "read"],
        loc="lower right",
        fontsize=10,
        frameon=False,
        labelcolor="grey",
    )

    write_speedup = write_data["klayout"] / write_data["gdsr"]
    read_speedup = read_data["klayout"] / read_data["gdsr"]

    plt.title(
        f"GDS I/O — gdsr is {write_speedup:.1f}x faster (write), {read_speedup:.1f}x faster (read)",
        fontsize=13,
        pad=20,
        color="grey",
        y=-0.55,
    )

    output_path = RESULTS_DIR / "benchmark.svg"
    plt.savefig(output_path, dpi=600, bbox_inches="tight", transparent=True)
    plt.close()
    print(f"Chart saved to {output_path}")


if __name__ == "__main__":
    main()
