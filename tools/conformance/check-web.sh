#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
readonly ROOT

cargo test --locked --manifest-path "$ROOT/Cargo.toml" \
    --test javascript_tests \
    --test web_conformance_tests

for source in "$ROOT"/benchmarks/javascript/*.mjs; do
    node --check "$source"
done
