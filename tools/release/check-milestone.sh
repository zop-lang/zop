#!/usr/bin/env bash
# Copyright 2026 Zop contributors
# SPDX-License-Identifier: Apache-2.0

set -euo pipefail

if (( $# != 4 )); then
    echo "usage: check-milestone.sh TAG MILESTONES_JSON ISSUES_JSON ROADMAP" >&2
    exit 2
fi

readonly TAG=$1
readonly MILESTONES_JSON=$2
readonly ISSUES_JSON=$3
readonly ROADMAP=$4

if [[ ! $TAG =~ ^v([0-9]+\.[0-9]+\.[0-9]+)(-[0-9A-Za-z.-]+)?$ ]]; then
    echo "release tag must be vMAJOR.MINOR.PATCH with an optional prerelease suffix: $TAG" >&2
    exit 1
fi

readonly VERSION=${BASH_REMATCH[1]}
readonly GATE_TITLE="Release gate: $VERSION"

milestone_count=$(jq --arg version "$VERSION" \
    '[.[] | select(.title == $version)] | length' "$MILESTONES_JSON")
if (( milestone_count != 1 )); then
    echo "release requires exactly one GitHub milestone named $VERSION" >&2
    exit 1
fi

milestone_state=$(jq -r --arg version "$VERSION" \
    '.[] | select(.title == $version) | .state' "$MILESTONES_JSON")
open_items=$(jq -r --arg version "$VERSION" \
    '.[] | select(.title == $version) | .open_issues' "$MILESTONES_JSON")
if [[ $milestone_state != closed || $open_items != 0 ]]; then
    echo "milestone $VERSION must be closed with zero open issues or pull requests" >&2
    exit 1
fi

gate_count=$(jq --arg title "$GATE_TITLE" \
    '[.[] | select(.title == $title and (has("pull_request") | not))] | length' \
    "$ISSUES_JSON")
if (( gate_count != 1 )); then
    echo "milestone $VERSION requires exactly one issue titled '$GATE_TITLE'" >&2
    exit 1
fi

gate_state=$(jq -r --arg title "$GATE_TITLE" \
    '.[] | select(.title == $title and (has("pull_request") | not)) | .state' \
    "$ISSUES_JSON")
if [[ $gate_state != closed ]]; then
    echo "release gate issue for $VERSION must be closed" >&2
    exit 1
fi

gate_body=$(jq -r --arg title "$GATE_TITLE" \
    '.[] | select(.title == $title and (has("pull_request") | not)) | .body // ""' \
    "$ISSUES_JSON")

SCRATCH=$(mktemp -d)
readonly SCRATCH
trap 'rm -rf "$SCRATCH"' EXIT

printf '%s\n' "$gate_body" > "$SCRATCH/gate-body.md"
start_markers=$(grep -cFx '<!-- roadmap-section:start -->' \
    "$SCRATCH/gate-body.md" || true)
end_markers=$(grep -cFx '<!-- roadmap-section:end -->' \
    "$SCRATCH/gate-body.md" || true)
if (( start_markers != 1 || end_markers != 1 )); then
    echo "release gate issue for $VERSION must contain one canonical roadmap section" >&2
    exit 1
fi

awk -v version="$VERSION" '
    $0 ~ "^## " version "(:|$)" { capture = 1; next }
    capture && /^## / { exit }
    capture { lines[++count] = $0 }
    END {
        first = 1
        while (first <= count && lines[first] == "") first++
        last = count
        while (last >= first && lines[last] == "") last--
        for (line = first; line <= last; line++) print lines[line]
    }
' "$ROADMAP" > "$SCRATCH/roadmap-section.md"

awk '
    /<!-- roadmap-section:start -->/ { capture = 1; next }
    /<!-- roadmap-section:end -->/ { exit }
    capture { lines[++count] = $0 }
    END {
        first = 1
        while (first <= count && lines[first] == "") first++
        last = count
        while (last >= first && lines[last] == "") last--
        for (line = first; line <= last; line++) print lines[line]
    }
' "$SCRATCH/gate-body.md" \
    | sed -E 's/^([[:space:]]*)- \[[ xX]\] /\1- /' \
    > "$SCRATCH/gate-section.md"

if ! cmp -s "$SCRATCH/roadmap-section.md" "$SCRATCH/gate-section.md"; then
    echo "release gate issue for $VERSION has drifted from docs/architecture/roadmap.md" >&2
    exit 1
fi

if grep -Eq '^[[:space:]]*-[[:space:]]+\[[[:space:]]\]' <<< "$gate_body"; then
    echo "release gate issue for $VERSION contains unchecked criteria" >&2
    exit 1
fi

echo "release gate passed: $TAG"
