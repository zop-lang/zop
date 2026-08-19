#!/usr/bin/env bash
set -euo pipefail

readonly EXPECTED_TOPCOAT=a2bd596af2a149f38fcf49570481f356a6cb1069
readonly TOPCOAT_DIR=${TOPCOAT_DIR:?set TOPCOAT_DIR to the pinned Topcoat checkout}
ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
readonly ROOT
readonly OUTPUT="$ROOT/.zop/benchmarks/javascript/affine.mjs"
readonly TOPCOAT_OUTPUT="$ROOT/.zop/benchmarks/javascript/topcoat.mjs"

actual_topcoat=$(git -C "$TOPCOAT_DIR" rev-parse HEAD)
if [[ "$actual_topcoat" != "$EXPECTED_TOPCOAT" ]]; then
    echo "FAIL  topcoat commit ($actual_topcoat)" >&2
    exit 1
fi

mkdir -p "$(dirname "$OUTPUT")"
cargo run --quiet --release --manifest-path "$ROOT/Cargo.toml" -- \
    javascript "$ROOT/benchmarks/javascript/affine.zop" "$OUTPUT"
bun build "$TOPCOAT_DIR/crates/topcoat-runtime/browser/src/surrogate/f64.ts" \
    --target=browser --format=esm --minify --outfile="$TOPCOAT_OUTPUT" >/dev/null
bun "$ROOT/benchmarks/javascript/core.mjs" "$OUTPUT" "$TOPCOAT_OUTPUT"
