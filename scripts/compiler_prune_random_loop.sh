#!/usr/bin/env bash
set -euo pipefail
export LC_ALL=C
export LANG=C

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

copy_current_tree() {
  local destination="$1"
  mkdir -p "$destination"
  (
    cd "$repo_root"
    git ls-files -z | tar --null -T - -cf -
  ) | tar -xf - -C "$destination"
}

item_inventory() {
  local root="$1"
  find "$root" -name '*.rs' -print |
    sort |
    while IFS= read -r file; do
      local relative="${file#"$root"/}"
      sed -E \
        -e '/^#!\[/d' \
        -e '/^#\[allow\(dead_code\)\]$/d' \
        -e '/slicers compiler-prune/d' \
        "$file" |
        rg --no-line-number '^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?(async[[:space:]]+)?(fn|struct|enum|trait|union|type|const|static|mod)[[:space:]]+[A-Za-z_][A-Za-z0-9_]*|^[[:space:]]*impl([[:space:]]|<)' |
        sed "s#^#$relative:#"
    done
}

package_inventory() {
  local root="$1"
  find "$root" -mindepth 2 -maxdepth 2 -name Cargo.toml -print |
    sed "s#$root/##; s#/Cargo.toml##" |
    sort
}

workspace_file_inventory() {
  local root="$1"
  find "$root" \
    \( -path "$root/target" -o -path "$root/.git" \) -prune -o \
    -type f \
    ! -name slicers-compiler-prune-report.json \
    -print |
    sed "s#$root/##" |
    sort
}

raw_workspace_file_count() {
  local root="$1"
  find "$root" \
    \( -path "$root/target" -o -path "$root/.git" \) -prune -o \
    -type f \
    -print |
    wc -l |
    tr -d ' '
}

while true; do
  cycle=$((cycle + 1))
  if [[ "$limit" != "0" && "$cycle" -gt "$limit" ]]; then
    break
  fi

  selected="$(
    printf '%s\n' "${cases[@]}" |
      awk 'BEGIN { srand() } { print rand() "\t" $0 }' |
      sort -n |
      head -n 5 |
      cut -f2-
  )"

  echo "cycle $cycle: 5 roots"
  index=0
  while IFS= read -r case; do
    if [[ -z "$case" ]]; then
      continue
    fi
    index=$((index + 1))
    IFS='|' read -r label package file pattern <<<"$case"

    workspace="/tmp/slicers-loop-${cycle}-${index}-${label}-src"
    expected="/tmp/slicers-loop-${cycle}-${index}-${label}-expected"
    slice="/tmp/slicers-loop-${cycle}-${index}-${label}-slice"
    original_target="/tmp/slicers-loop-${cycle}-${index}-${label}-original-target"
    expected_target="/tmp/slicers-loop-${cycle}-${index}-${label}-expected-target"
    slice_target="/tmp/slicers-loop-${cycle}-${index}-${label}-slice-target"

    copy_current_tree "$workspace"

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
    "$repo_root/target/debug/slicers" "$workspace" "$expected"
    SLICERS_CARGO_TARGET_DIR="$slice_target" "$repo_root/target/debug/slicers" --compiler-prune "$workspace" "$slice"
    CARGO_TARGET_DIR="$slice_target" cargo check --manifest-path "$slice/Cargo.toml" -p "$package" --lib
    rg -q '"cargo_status": 0' "$slice/slicers-compiler-prune-report.json"

    expected_packages="/tmp/slicers-loop-${cycle}-${index}-${label}-expected-packages.txt"
    slice_packages="/tmp/slicers-loop-${cycle}-${index}-${label}-slice-packages.txt"
    expected_items="/tmp/slicers-loop-${cycle}-${index}-${label}-expected-items.txt"
    slice_items="/tmp/slicers-loop-${cycle}-${index}-${label}-slice-items.txt"
    expected_files="/tmp/slicers-loop-${cycle}-${index}-${label}-expected-files.txt"
    slice_files="/tmp/slicers-loop-${cycle}-${index}-${label}-slice-files.txt"
    package_inventory "$expected" >"$expected_packages"
    package_inventory "$slice" >"$slice_packages"
    item_inventory "$expected" >"$expected_items"
    item_inventory "$slice" >"$slice_items"
    workspace_file_inventory "$expected" >"$expected_files"
    workspace_file_inventory "$slice" >"$slice_files"
    expected_file_count="$(wc -l <"$expected_files" | tr -d ' ')"
    slice_file_count="$(wc -l <"$slice_files" | tr -d ' ')"
    slice_raw_file_count="$(raw_workspace_file_count "$slice")"
    echo "workspace files: expected=$expected_file_count compiler-prune=$slice_file_count raw=$slice_raw_file_count"
    diff -u "$expected_files" "$slice_files"
    diff -u "$expected_packages" "$slice_packages"
    diff -u "$expected_items" "$slice_items"

    CARGO_TARGET_DIR="$original_target" cargo clean --manifest-path "$workspace/Cargo.toml"
    CARGO_TARGET_DIR="$expected_target" cargo clean --manifest-path "$expected/Cargo.toml"
    CARGO_TARGET_DIR="$slice_target" cargo clean --manifest-path "$slice/Cargo.toml"
  done <<<"$selected"
done
