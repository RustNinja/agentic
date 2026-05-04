#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

cargo check --manifest-path fixtures/fast_macro_use/Cargo.toml --quiet
cargo test -p opensource_core --test fast_macro_use -- --nocapture
