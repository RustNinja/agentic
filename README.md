# agentic

## opensourced proof

This workspace is a proof-of-concept for marking one Rust function with
`#[opensourced]` and generating a reduced Rust workspace that keeps only the
function-level dependency closure needed by that entry point.

## Shape

- `crates/opensourced`: proc-macro crate that validates and preserves a marked
  free function.
- `crates/opensource_core`: `syn`-based parser, call-graph reducer, and source
  renderer.
- `crates/opensource_cli`: command-line wrapper around `opensource_core`.
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

The reducer finds that marker, walks reachable calls through the workspace, and
writes a new five-crate workspace without the unreachable functions.

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
- pruning public, private, and method functions that are not reachable;
- stripping the marker macro and its dependency from the generated workspace;
- compiling the generated workspace with `cargo check`.

The reducer conservatively keeps non-function items such as structs, type
aliases, constants, imports, and associated constants so the reduced source keeps
compiling while function-level pruning is proven.

## Run

```sh
cargo test --workspace
cargo run -p opensource_cli -- . /tmp/opensourced-proof
cargo check --manifest-path /tmp/opensourced-proof/Cargo.toml
```

Expected reachable callables for the fixture:

```text
a::open_source_entry
b::compute
b::helper
c::adjust
d::hash
d::normalize
d::shared
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

## Current Boundaries

This proof intentionally uses a syntactic call graph instead of rustc name
resolution. It is useful for controlled workspaces and for proving the export
pipeline, but it is not a full compiler frontend. Trait method resolution,
macro-expanded calls, complex function pointers, `cfg` combinations, build
scripts, and dependency pruning beyond local path crates need more work before
this can be treated as a production-grade Rust slicer.
