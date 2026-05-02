# Real Hard-Rust Slice Report: rtk-ai/rtk

Date: 2026-05-02

Source project: `rtk-ai/rtk`

- GitHub: `https://github.com/rtk-ai/rtk`
- Stars/forks at check time: 39,681 stars, 2,411 forks
- Default branch tested: `master`
- Note: this is not a UniFFI project. It was selected as a large, high-signal
  Rust CLI project after GitHub search did not show a Rust UniFFI project with
  both 1,000+ stars and 1,000+ forks. The prior real UniFFI validation remains
  `dnakov/litter`.

The source tarball was downloaded from GitHub and extracted under
`/tmp/slicers-rtk.SqnZUB`. The project contains 101 Rust files and 61,237 Rust
LOC under `src`.

## Why This Project

`rtk` has several patterns that are useful for production hardening:

- binary-only Cargo package
- `build.rs` with required runtime input files
- macro-declared modules via `automod::dir!`
- item macros that generate referenced statics (`lazy_static!`)
- CLI-sized manifest metadata and dependency tables
- nested module trees and large unrelated command implementations

## Setup

The experiment added:

```toml
opensourced = { path = "/Users/mykyta/dev/rustclaw/agentic-sg/crates/opensourced" }
```

Then exactly one real function was marked with `#[opensourced]`, and each
generated slice was validated with `cargo check` only:

```sh
cargo build -p opensource_cli --bin slicers
./target/debug/slicers /tmp/slicers-rtk.SqnZUB /tmp/slicers-rtk-out-...
cargo check --manifest-path /tmp/slicers-rtk-out-.../Cargo.toml
```

## Experiments

| Root function | Output Rust LOC | Output files, excluding `target` | Result |
| --- | ---: | ---: | --- |
| `learn::detector::find_corrections` | 186 | 67 | `cargo check` passed |
| `cmds::system::json_cmd::filter_json_string` | 99 | 68 | `cargo check` passed |

### `find_corrections`

Retained source:

- `src/main.rs`
- `src/learn/mod.rs`
- `src/learn/detector.rs`
- `src/filters/*.toml` required by `build.rs`

Important retained callables:

- `find_corrections`
- `is_command_error`
- `classify_error`
- `extract_base_command`
- `command_similarity`
- `is_tdd_cycle_error`
- `differs_only_by_path`

Important retained items:

- `CommandExecution`
- `CorrectionPair`
- `ErrorType`
- `CORRECTION_WINDOW`
- `MIN_CONFIDENCE`
- `lazy_static!` regex definitions feeding retained functions

### `filter_json_string`

Retained source:

- `src/main.rs`
- `src/cmds/mod.rs`
- `src/cmds/system/mod.rs`
- `src/cmds/system/json_cmd.rs`
- `src/filters/*.toml` required by `build.rs`

Important retained callables:

- `filter_json_string`
- `extract_schema`

Manifest pruning result:

- Kept: `anyhow`, `serde_json`, `clap`, `toml` build dependency
- Removed: `automod`, `lazy_static`, `regex`, and unrelated CLI dependencies
  when absent from retained source

## Failures Found And Fixed

1. Binary-only packages were not modeled.

   Fix: single-package Cargo roots now parse `src/main.rs` when no `src/lib.rs`
   exists. Generated binary roots get a stub `fn main() {}` when the original
   CLI entrypoint is outside the slice.

2. Stub binary roots kept stale imports.

   Fix: when a binary root is reduced to only module declarations plus a stub
   main, original root imports are dropped so pruned modules do not leak into
   the slice.

3. `lazy_static!` generated statics were pruned.

   Fix: item macro invocations with no item ident are retained when identifiers
   inside the macro tokens are referenced by reachable code. Matching macro
   definitions are retained when needed.

4. `automod::dir!` modules were invisible.

   Fix: the parser expands `automod::dir!(... "path")` directories into real
   module files, and the renderer writes explicit `mod` declarations for the
   reduced child modules. This also allows the `automod` dependency to be
   pruned when the generated slice no longer uses the macro.

5. Build-script assets were copied too broadly.

   Fix: `build.rs` is copied when present, but non-Rust assets are now copied
   only when their paths appear as build-script string literals and exist under
   the package root. This kept RTK's required `src/filters/*.toml` files and
   stopped copying unrelated docs, hooks, CI, and agent config.

6. Manifest shape was too narrow.

   Fix: dependency collection and rendering now handle single-package roots,
   `[build-dependencies]`, target-specific dependency tables, feature references
   to pruned optional deps, and explicit `[[bin]]`/`[bin]` metadata.

Regression coverage added:

- `crates/opensource_core/tests/manifest_hardening.rs`

## Current Result

The slicer produced compiling RTK slices from two unrelated components. The
output retained only the source modules, functions, data items, generated macro
items, manifest dependencies, and build-script assets needed for each selected
root. Unit tests from the source project were not copied.

Remaining expected warning: generated slices can still contain unused imports.
That does not change runtime semantics and remains warning-only under
`cargo check`.
