#!/usr/bin/env bash
# Copyright 2026 Zop contributors
# SPDX-License-Identifier: Apache-2.0

set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
readonly ROOT
FIXTURES=$(mktemp -d)
readonly FIXTURES
trap 'rm -rf "$FIXTURES"' EXIT

write_milestone() {
    local state=$1
    local open_items=$2

    printf '[{"number":2,"title":"0.1.0","state":"%s","open_issues":%s}]\n' \
        "$state" "$open_items" > "$FIXTURES/milestones.json"
}

write_gate() {
    local state=$1
    local mark=$2

    local body
    body=$(printf '%s\n' \
        '<!-- roadmap-section:start -->' \
        "- [$mark] compiler" \
        '<!-- roadmap-section:end -->')
    jq -n --arg state "$state" --arg body "$body" \
        '[{"title":"Release gate: 0.1.0","state":$state,"body":$body}]' \
        > "$FIXTURES/issues.json"
}

expect_failure() {
    if "$ROOT/tools/release/check-milestone.sh" \
        v0.1.0 "$FIXTURES/milestones.json" "$FIXTURES/issues.json" \
        "$FIXTURES/roadmap.md" \
        >/dev/null 2>&1; then
        echo "expected release gate rejection" >&2
        exit 1
    fi
}

printf '# Roadmap\n\n## 0.1.0: bootstrap\n\n- compiler\n' \
    > "$FIXTURES/roadmap.md"

write_milestone closed 0
write_gate closed x
"$ROOT/tools/release/check-milestone.sh" \
    v0.1.0 "$FIXTURES/milestones.json" "$FIXTURES/issues.json" \
    "$FIXTURES/roadmap.md"

write_milestone open 0
expect_failure

write_milestone closed 1
expect_failure

write_milestone closed 0
write_gate open x
expect_failure

write_gate closed " "
expect_failure

write_gate closed x
printf '\n- drift\n' >> "$FIXTURES/roadmap.md"
expect_failure

echo "release gate tests passed"
