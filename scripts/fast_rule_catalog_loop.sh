#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

cargo test -p opensource_core --test rule_catalog -- --nocapture
cargo test -p opensource_core --test rule_database -- --nocapture
