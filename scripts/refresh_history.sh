#!/usr/bin/env bash
# Refresh the historical international results CSV used for DC/pi-ratings fitting.
# Default source is a maintained mirror of the original martj42 dataset.
set -euo pipefail

OUT="$(dirname "$0")/../data/international_results.csv"
URL="${1:-https://raw.githubusercontent.com/JamshedAli18/International-football-results-from-1872-to-2024/master/results.csv}"

curl -sSL --fail -o "$OUT" "$URL"
echo "Wrote $(wc -l < "$OUT") rows to $OUT"