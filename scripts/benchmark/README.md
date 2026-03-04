# Benchmark: gdsr vs KLayout

Compares GDS read/write performance between gdsr and KLayout using [hyperfine](https://github.com/sharkdp/hyperfine).

## Prerequisites

- Rust toolchain
- [hyperfine](https://github.com/sharkdp/hyperfine): `brew install hyperfine`
- Python 3.13+ with klayout:

```bash
python3 -m venv .venv
.venv/bin/pip install klayout
```

## Run benchmarks

```bash
./benchmark_comparison.sh
```

This builds the gdsr benchmark binary, runs hyperfine for both read and write operations, and saves results to `results/write.json` and `results/read.json`.

## Generate plot

```bash
uv run plot.py
```

Outputs `results/benchmark.svg`.
