#!/usr/bin/env bash
# Benchmark comparison: KLayout vs gdsr using hyperfine.
#
# Prerequisites:
#   brew install hyperfine
#   cargo (Rust toolchain)
#   scripts/benchmark/.venv with klayout installed
#
# Usage:
#   ./scripts/benchmark/benchmark_comparison.sh
#
# Outputs:
#   scripts/benchmark/results/write.json
#   scripts/benchmark/results/read.json
#
# Then run plot.py to visualize:
#   ./scripts/benchmark/.venv/bin/python3 scripts/benchmark/plot.py

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

if [[ -x "$SCRIPT_DIR/.venv/bin/python3" ]]; then
    PYTHON="${PYTHON:-$SCRIPT_DIR/.venv/bin/python3}"
else
    PYTHON="${PYTHON:-python3}"
fi

WARMUP=3
RUNS=20
RESULTS_DIR="$SCRIPT_DIR/results"
mkdir -p "$RESULTS_DIR"

# Build gdsr benchmark
echo "Building gdsr benchmark (release)..."
cargo build --release --manifest-path "$SCRIPT_DIR/Cargo.toml" 2>&1
GDSR_BIN="$SCRIPT_DIR/target/release/benchmark_gdsr"

# Verify dependencies
if ! "$PYTHON" -c "import klayout.db" 2>/dev/null; then
    echo "Error: klayout not found. Run: $SCRIPT_DIR/.venv/bin/pip install klayout"
    exit 1
fi

if ! command -v hyperfine &>/dev/null; then
    echo "Error: hyperfine not found. Install with: brew install hyperfine"
    exit 1
fi

# Seed read benchmark files
echo "Preparing read benchmark files..."
"$GDSR_BIN" write
"$PYTHON" "$SCRIPT_DIR/benchmark_klayout.py" write

# Run benchmarks
echo ""
echo "=== Write ==="
hyperfine \
    --warmup "$WARMUP" \
    --runs "$RUNS" \
    --shell=none \
    --export-json "$RESULTS_DIR/write.json" \
    --command-name "gdsr"    "$GDSR_BIN write" \
    --command-name "klayout" "$PYTHON $SCRIPT_DIR/benchmark_klayout.py write"

echo ""
echo "=== Read ==="
hyperfine \
    --warmup "$WARMUP" \
    --runs "$RUNS" \
    --shell=none \
    --export-json "$RESULTS_DIR/read.json" \
    --command-name "gdsr"    "$GDSR_BIN read" \
    --command-name "klayout" "$PYTHON $SCRIPT_DIR/benchmark_klayout.py read"

echo ""
echo "Results saved to $RESULTS_DIR/"
echo "Run plot.py to visualize: $PYTHON $SCRIPT_DIR/plot.py"
