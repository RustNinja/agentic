# agentic-sg slicers

## Slicer Proof

This workspace is a proof-of-concept for marking one Rust function with
`#[opensourced]` and generating a reduced Rust workspace that keeps only the
function-level dependency closure needed by that entry point. The intended use
case is giving a human or AI agent a small, compilable slice of a large Rust
workspace instead of the full source tree.

## Shape

- `crates/opensourced`: proc-macro crate that validates and preserves a marked
  free function.
- `crates/opensource_core`: `syn`-based parser, call-graph reducer, and source
  renderer.
- `crates/opensource_cli`: command-line wrapper around `opensource_core`.
  The primary binary is `slicers`.
- `fixtures/a` through `fixtures/e`: five crates used as the proof workspace.

The fixture dependency graph is:

```text
a -> b -> d -> e
a -> c -> d -> e
```

`fixtures/a/src/lib.rs` marks only `open_source_entry`:

```rust
#[opensourced]
pub fn open_source_entry(value: i32) -> i32 {
    renamed_b::compute(value) + c::adjust(value)
}
```

The reducer finds that marker, walks reachable calls and type/API references
through the workspace, and writes a new workspace without unreachable
implementation code, unused local crates, or unit tests.

## Why This Is Not Proc-Macro-Only

A Rust proc macro receives only the token stream it is attached to. It cannot
reliably inspect the whole crate or dependency workspace. The macro is therefore
only the marker/validator. Whole-workspace analysis is done by a separate tool
that parses source files with `syn`.

## Covered In The Proof

The fixture and tests cover:

- cross-crate free function calls;
- transitive dependency calls;
- module alias calls such as `use b as renamed_b`;
- function alias calls such as `use d::hash as renamed_hash`;
- private helpers reached from public code;
- associated function calls such as `d::Worker::new`;
- receiver method calls inferred from a local binding such as `worker.run(...)`;
- trait method calls inferred from a local receiver, such as
  `worker.transform(...)`;
- trait definitions and trait impl blocks needed by reachable methods;
- borrowed trait UFCS calls such as `Trait::method(&receiver, ...)`;
- enums, structs, type aliases, consts, and statics used in reachable
  signatures or implementations;
- pruning unused data/API items such as an unreachable enum;
- pruning public, private, and method functions that are not reachable;
- pruning unused local path dependency crates from generated manifests;
- pruning unused local dependency imports from retained source files;
- skipping external module files with no reachable descendants;
- preserving external and workspace dependencies such as `serde`;
- dropping `#[cfg(test)]` modules and `#[test]` functions from generated output;
- stripping the marker macro and its dependency from the generated workspace;
- compiling the generated workspace with `cargo check`.

The reducer still keeps imports and associated const/type items conservatively
inside retained impl blocks so the reduced source keeps compiling.

## Run

```sh
cargo test --workspace
cargo run -p opensource_cli --bin slicers -- --check . /tmp/slicers-proof
cargo run -p opensource_cli --bin slicers -- --feedback . /tmp/slicers-feedback
cargo check --manifest-path /tmp/slicers-proof/Cargo.toml
```

`--feedback` runs `cargo check --message-format=json` against the generated
workspace, prints prioritized compiler diagnostics, and writes
`slice-feedback.json` under the slice output. `--feedback-loop <n>` repeats that
compiler feedback pass up to `n` times and fails with the JSON report path when
the slice still does not compile. This is the current production feedback layer:
the slicer stays fast and syntactic, then rustc gives precise diagnostics for
the generated slice. A rust-analyzer HIR or rustc-driver backend remains the
next precision step for resolving hard name-resolution cases before rendering.

## Additional Generated Fixtures

`opensource_core` also has integration tests that generate temporary workspaces
with different source shapes and then run the reducer against them:

- `data_items.rs`: structs, enums, type aliases, consts, statics, associated
  constructors, body-only data items, and unit-test pruning.
- `module_reexports.rs`: external module files, nested modules, renamed
  dependencies, local aliases, reachable `pub use` items, and unreachable module
  functions.
- `trait_ufcs.rs`: explicit UFCS trait calls such as
  `<Thing as Describe>::describe(...)`, trait impl reachability, enum variants,
  tuple structs, and unreachable impl methods.
- `uniffi_mobile.rs`: a Mozilla UniFFI/application-services-shaped mobile
  bridge workspace with FFI-facing DTO structs/enums, `cfg_attr(...,
  uniffi::...)` annotations, serde workspace dependencies, object-like impl
  methods, trait calls, and an unused local diagnostics crate that must be
  removed from the slice.

See `docs/uniffi_slicer_experiment.md` for the UniFFI fixture rationale and
verified result.
See `docs/slice_coverage_matrix.md` for the full supported slice matrix and
current boundaries.

Expected reachable callables for the fixture:

```text
a::open_source_entry
b::compute
b::helper
c::adjust
d::hash
d::normalize
d::shared
d::<Worker as Transform>::transform
d::Worker::new
d::Worker::run
e::finish
e::mix
e::seed
```

Expected pruned callables include:

```text
a::internal_entry
b::unused_public
c::unused
d::Worker::unused_method
d::unused_public
d::unused_private
e::tempting_but_unused
e::unused_leaf
```

Expected reachable items include:

```text
d::Mode(Enum)
d::SALT(Const)
d::Score(Type)
d::Transform(Trait)
d::Worker(Struct)
e::DEFAULT_SEED(Const)
e::Score(Type)
```

## Current Boundaries

This proof intentionally uses a syntactic call graph instead of rustc name
resolution. It is useful for controlled workspaces and for proving the slice
pipeline, but it is not a full compiler frontend. It now handles direct trait
method calls when the receiver type can be inferred locally, borrowed UFCS trait
calls, workspace member globs, external dependencies, and local path crate
pruning. It also scans retained macro bodies for direct local paths and prunes
external dependencies whose crate alias is absent from the retained source.
Complex function pointers, trait objects, broad `cfg` feature matrices, full
macro expansion, build scripts, and rustc-level unused import analysis remain
outside the current syntactic model.
