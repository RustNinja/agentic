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

The reducer finds that marker, walks reachable calls and type/API references
through the workspace, and writes a new five-crate workspace without unreachable
implementation code or unit tests.

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
- enums, structs, type aliases, consts, and statics used in reachable
  signatures or implementations;
- pruning unused data/API items such as an unreachable enum;
- pruning public, private, and method functions that are not reachable;
- dropping `#[cfg(test)]` modules and `#[test]` functions from generated output;
- stripping the marker macro and its dependency from the generated workspace;
- compiling the generated workspace with `cargo check`.

The reducer still keeps imports and associated const/type items conservatively
inside retained impl blocks so the reduced source keeps compiling.

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
d::Score(Type)
d::Transform(Trait)
d::Worker(Struct)
e::DEFAULT_SEED(Const)
e::Score(Type)
```

## Current Boundaries

This proof intentionally uses a syntactic call graph instead of rustc name
resolution. It is useful for controlled workspaces and for proving the export
pipeline, but it is not a full compiler frontend. It now handles direct trait
method calls when the receiver type can be inferred locally, but macro-expanded
calls, complex function pointers, broad `cfg` feature matrices, build scripts,
and dependency pruning beyond local path crates need more work before this can
be treated as a production-grade Rust slicer.
