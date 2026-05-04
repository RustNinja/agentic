#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$(mktemp -d "${TMPDIR:-/tmp}/slicers-fast-fixture-target.XXXXXX")}"
export CARGO_TARGET_DIR

cargo check --manifest-path fixtures/fast_macro_use/Cargo.toml --quiet
cargo test -p opensource_core --test fast_macro_use -- --nocapture
