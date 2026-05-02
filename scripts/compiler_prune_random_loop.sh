#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
limit="${SLICERS_LOOP_LIMIT:-0}"
cycle=0

cases=(
  "fn-a-internal|a|fixtures/a/src/lib.rs|pub fn internal_entry"
  "fn-b-compute|b|fixtures/b/src/lib.rs|pub fn compute"
  "fn-e-mix|e|fixtures/e/src/lib.rs|pub fn mix"
  "enum-d-mode|d|fixtures/d/src/lib.rs|pub enum Mode"
  "struct-d-worker|d|fixtures/d/src/lib.rs|pub struct Worker"
  "trait-d-transform|d|fixtures/d/src/lib.rs|pub trait Transform"
  "mod-d-nested|d|fixtures/d/src/lib.rs|pub mod nested"
)

cargo build -p opensource_cli

while true; do
  cycle=$((cycle + 1))
  if [[ "$limit" != "0" && "$cycle" -gt "$limit" ]]; then
    break
  fi

  mapfile -t selected < <(
    printf '%s\n' "${cases[@]}" |
      awk 'BEGIN { srand() } { print rand() "\t" $0 }' |
      sort -n |
      head -n 5 |
      cut -f2-
  )

  echo "cycle $cycle: ${#selected[@]} roots"
  index=0
  for case in "${selected[@]}"; do
    index=$((index + 1))
    IFS='|' read -r label package file pattern <<<"$case"

    workspace="/tmp/slicers-loop-${cycle}-${index}-${label}-src"
    slice="/tmp/slicers-loop-${cycle}-${index}-${label}-slice"
    original_target="/tmp/slicers-loop-${cycle}-${index}-${label}-original-target"
    slice_target="/tmp/slicers-loop-${cycle}-${index}-${label}-slice-target"

    mkdir -p "$workspace"
    git -C "$repo_root" archive --format=tar HEAD | tar -x -C "$workspace"

    perl -0pi -e 's/\n#\[opensourced\]\n/\n/g' "$workspace"/fixtures/*/src/lib.rs
    manifest="$workspace/fixtures/$package/Cargo.toml"
    if ! rg -q 'opensourced' "$manifest"; then
      if rg -q '^\[dependencies\]' "$manifest"; then
        printf '\nopensourced = { path = "../../crates/opensourced" }\n' >>"$manifest"
      else
        printf '\n[dependencies]\nopensourced = { path = "../../crates/opensourced" }\n' >>"$manifest"
      fi
    fi

    source="$workspace/$file"
    if ! rg -q 'use opensourced::opensourced;' "$source"; then
      perl -0pi -e 's/\A/use opensourced::opensourced;\n/' "$source"
    fi
    PATTERN="$pattern" perl -0pi -e 'BEGIN { $pattern = $ENV{"PATTERN"}; $done = 0 } if (!$done) { $done = s/\Q$pattern\E/#[opensourced]\n$pattern/ }' "$source"
    rg -q '#\[opensourced\]' "$source"

    echo "  [$cycle.$index] root=$label package=$package"
    CARGO_TARGET_DIR="$original_target" cargo check --manifest-path "$workspace/Cargo.toml" -p "$package" --lib
    SLICERS_CARGO_TARGET_DIR="$slice_target" "$repo_root/target/debug/slicers" --compiler-prune "$workspace" "$slice"
    CARGO_TARGET_DIR="$slice_target" cargo check --manifest-path "$slice/Cargo.toml" -p "$package" --lib
    rg -q '"cargo_status": 0' "$slice/slicers-compiler-prune-report.json"

    CARGO_TARGET_DIR="$original_target" cargo clean --manifest-path "$workspace/Cargo.toml"
    CARGO_TARGET_DIR="$slice_target" cargo clean --manifest-path "$slice/Cargo.toml"
  done
done
