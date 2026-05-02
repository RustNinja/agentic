# Slice Coverage Matrix

Every checked fixture generates a temporary reduced workspace and runs
`cargo check` against the generated slice. The tests also assert that expected
dead functions, items, modules, tests, and local crates are absent.

## Verified Slices

| Area | Verified by | Coverage |
| --- | --- | --- |
| Cross-crate free functions | `opensource_core` unit fixture | Root function, direct calls, transitive calls across five crates |
| Private helpers | `data_items.rs`, root fixture | Private functions reached from exported code are retained |
| Public dead APIs | all integration fixtures | Public functions not reached from the root are pruned |
| Structs | `data_items.rs`, `uniffi_mobile.rs`, `component_matrix.rs` | Unit structs, tuple structs, named-field structs, generic structs |
| Enums | `data_items.rs`, `trait_ufcs.rs`, `uniffi_mobile.rs`, `component_matrix.rs` | Unit, tuple, and struct-like variants used in signatures, bodies, and patterns |
| Unions | `component_matrix.rs` | Union item reached through a retained struct field and constructor literal |
| Type aliases | `data_items.rs`, root fixture, `component_matrix.rs` | Aliases used in signatures, fields, and local bindings |
| Consts/statics | root fixture, `data_items.rs`, `component_matrix.rs` | Constants/statics used in bodies, fields, and associated const values |
| Inherent impl methods | all integration fixtures | Associated constructors, receiver calls, async methods, generic impl blocks |
| Trait definitions | root fixture, `trait_ufcs.rs`, `uniffi_mobile.rs`, `component_matrix.rs` | Trait items retained when trait impl methods are reachable |
| Trait impl methods | root fixture, `trait_ufcs.rs`, `uniffi_mobile.rs`, `component_matrix.rs` | Receiver calls, explicit `<Type as Trait>::method`, and `Trait::method(&receiver, ...)` |
| Trait impl peers | `component_matrix.rs` | Required peer methods and associated type/const items are retained so trait impls compile |
| Associated types/consts | `component_matrix.rs` | Associated type and associated const dependencies are followed from retained impls |
| External modules | `module_reexports.rs`, `component_matrix.rs` | `mod file;`, `mod/name/mod.rs`, nested modules, and empty dead module pruning |
| Inline modules | `module_reexports.rs`, `component_matrix.rs` | Inline modules with retained reexports are preserved |
| Reexports | `module_reexports.rs`, `component_matrix.rs` | Reachable `pub use` targets are kept; dead grouped reexports are pruned |
| Dependency aliases | root fixture, `module_reexports.rs` | `package = "..."` dependency aliases and renamed imports |
| Workspace member globs | `component_matrix.rs` | `[workspace].members = ["crates/*"]` expansion |
| External dependencies | `uniffi_mobile.rs`, `component_matrix.rs` | Used workspace dependencies such as `serde = { features = ["derive"] }` are preserved; unused external deps with no retained source reference are pruned |
| Local path dependency pruning | `uniffi_mobile.rs`, `component_matrix.rs` | Unused local crates are omitted from manifests and source imports |
| Macro definitions | `component_matrix.rs` | Used `macro_rules!` definitions are kept, unused macro definitions are pruned, and direct helper calls inside retained macro bodies are followed |
| Async functions | `component_matrix.rs` | Async root and async impl method slices build |
| Unit tests | all generated fixtures | `#[test]` functions and `#[cfg(test)]` modules are dropped |
| UniFFI-shaped API | `uniffi_mobile.rs` | FFI-facing records/enums, inactive `cfg_attr(..., uniffi::...)`, serde DTOs, and mobile bridge shape |

## Current Boundaries

These are tracked limitations, not silently claimed support:

| Area | Boundary |
| --- | --- |
| Full rustc name resolution | The reducer is syntactic and does not replace rustc or rust-analyzer name resolution |
| Macro-expanded dependencies | The slicer scans retained macro bodies for direct local paths, but does not run macro expansion or model generated code |
| Generic trait receiver inference | Calls through generic bounds such as `value.trait_method()` are not fully resolved without a concrete receiver type |
| Function pointers and dynamic dispatch | Function pointer calls, trait-object calls, and callback registries are not followed |
| Build scripts | `build.rs` outputs and generated Rust files are not modeled |
| Feature/platform cfg matrices | `#[cfg(test)]` is pruned; broader feature/platform matrix evaluation is still conservative |
| External crate pruning | External dependencies are pruned when their crate alias is absent from retained source tokens; full rustc-level unused import analysis is not implemented |

## Verification Commands

```sh
cargo test --workspace
cargo run -p opensource_cli --bin slicers -- --check . /tmp/slicers-proof
cargo check --manifest-path /tmp/slicers-proof/Cargo.toml
```
