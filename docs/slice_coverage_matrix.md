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
| Consts/statics | root fixture, `data_items.rs`, `component_matrix.rs`, `pattern_constants.rs`, `manifest_hardening.rs` | Constants/statics used in bodies, fields, associated const values, unqualified match patterns, and implicit `format!("{NAME}")` captures |
| Inherent impl methods | all integration fixtures | Associated constructors, receiver calls, async methods, generic impl blocks |
| Trait definitions | root fixture, `trait_ufcs.rs`, `uniffi_mobile.rs`, `component_matrix.rs` | Trait items retained when trait impl methods are reachable |
| Trait impl methods | root fixture, `trait_ufcs.rs`, `uniffi_mobile.rs`, `component_matrix.rs`, `manifest_hardening.rs` | Receiver calls, explicit `<Type as Trait>::method`, `Trait::method(&receiver, ...)`, format-only `Display`, and `to_string()`-required `Display` impls |
| Trait impl peers | `component_matrix.rs` | Required peer methods and associated type/const items are retained so trait impls compile |
| Associated types/consts | `component_matrix.rs` | Associated type and associated const dependencies are followed from retained impls |
| External trait imports | `manifest_hardening.rs` | Extension traits such as `tokio::io::AsyncReadExt`, private std traits such as `std::io::Write`, and trait-method imports without an `Ext` suffix such as `base64::Engine` are retained |
| External modules | `module_reexports.rs`, `component_matrix.rs` | `mod file;`, `mod/name/mod.rs`, nested modules, and empty dead module pruning |
| Inline modules | `module_reexports.rs`, `component_matrix.rs` | Inline modules with retained reexports are preserved |
| Reexports | `module_reexports.rs`, `component_matrix.rs`, `manifest_hardening.rs` | Reachable `pub use` targets are kept; dead grouped and external `pub use` reexports are pruned |
| Dependency aliases | root fixture, `module_reexports.rs` | `package = "..."` dependency aliases and renamed imports |
| Workspace member globs | `component_matrix.rs` | `[workspace].members = ["crates/*"]` expansion |
| Single-package binary roots | `manifest_hardening.rs`, `docs/real_rtk_slice_report.md` | Non-workspace crates, `src/main.rs` parsing, and stub binary roots |
| Macro-declared modules | `manifest_hardening.rs`, `docs/real_rtk_slice_report.md` | `automod::dir!(... "path")` directories are parsed and rendered as explicit reduced `mod` declarations |
| External dependencies | `uniffi_mobile.rs`, `component_matrix.rs` | Used workspace dependencies such as `serde = { features = ["derive"] }` are preserved; unused external deps with no retained source reference are pruned |
| Build dependencies | `manifest_hardening.rs`, `docs/real_rtk_slice_report.md` | `[build-dependencies]` are retained when `build.rs` is copied |
| Target dependencies | `manifest_hardening.rs` | `[target.'cfg(...)'.dependencies]` tables are preserved when retained source references them |
| Required target features | `manifest_hardening.rs`, CLI unit tests | `required-features` on retained example/test/bench/bin targets are reported and validation rejects Cargo argument sets that would skip the selected target or leave required bin features inactive |
| Feature pruning | `manifest_hardening.rs` | Optional dependency feature entries are removed when the dependency is pruned |
| Local path dependency pruning | `uniffi_mobile.rs`, `component_matrix.rs` | Unused local crates are omitted from manifests and source imports |
| Macro definitions | `component_matrix.rs` | Used `macro_rules!` definitions are kept, unused macro definitions are pruned, and direct helper calls inside retained macro bodies are followed |
| Item macro-generated items | `manifest_hardening.rs`, `docs/real_rtk_slice_report.md` | Item macro invocations such as `lazy_static!` are retained when generated identifiers are referenced by reachable code |
| Build-script assets | `manifest_hardening.rs`, `docs/real_rtk_slice_report.md` | Non-Rust assets referenced by `build.rs` string literal paths are copied without copying unrelated package docs/config |
| Source include assets | `manifest_hardening.rs` | Files referenced by retained `include!`, `include_str!`, and `include_bytes!` static literal/`concat!`/`CARGO_MANIFEST_DIR` paths are copied next to the sliced source; unknown, absolute, external, and `OUT_DIR` file includes are production-blocking |
| Symlink copy boundaries | `manifest_hardening.rs` | Internal package symlinked include assets are copied at the source-visible link path, while symlinked source or build assets resolving outside the package root are not copied |
| Workspace patches and locks | `manifest_hardening.rs`, CLI unit tests | Root `[patch.*]` tables and `Cargo.lock` are preserved; production validation adds `--locked` for source workspaces with lockfiles so sliced workspaces keep the source repository's dependency resolution |
| Workspace profiles | `manifest_hardening.rs` | Root `[profile.*]` tables are preserved so generated validation does not fall back to Cargo profile defaults |
| Toolchain context | `manifest_hardening.rs` | Root `rust-toolchain.toml` / `rust-toolchain` files are copied so generated validation uses the source workspace's pinned Rust toolchain |
| Async functions | `component_matrix.rs` | Async root and async impl method slices build |
| Unit tests | all generated fixtures | `#[test]` functions and `#[cfg(test)]` modules are dropped |
| Non-test cfg roots | core production-readiness tests | Selected roots behind feature/platform cfgs, including `runtime-benchmarks`, are retained in the graph and reported as production-blocking cfg-gated roots instead of being pruned as tests |
| UniFFI-shaped API | `uniffi_mobile.rs`, `uniffi_setup.rs` | FFI-facing records/enums, inactive `cfg_attr(..., uniffi::...)`, retained `uniffi::setup_scaffolding!()`, serde DTOs, and mobile bridge shape |
| Real UniFFI project | `docs/real_litter_uniffi_slice_report.md` | Litter `codex-mobile-client` cloud sync and preferences slices build after pruning |
| Real high-star Rust project | `docs/real_rtk_slice_report.md` | RTK `find_corrections` and `filter_json_string` slices build after binary, automod, macro, and build-script hardening |

## Current Boundaries

These are tracked limitations, not silently claimed support:

| Area | Boundary |
| --- | --- |
| Full rustc name resolution | The reducer is syntactic and does not replace rustc or rust-analyzer name resolution |
| Macro-expanded dependencies | The slicer does not run macro expansion; retained custom derives, custom attributes, and non-builtin macro invocations are production-blocking until an expansion-aware analyzer is available |
| Generic trait receiver inference | Calls through generic bounds such as `value.trait_method()` are not fully resolved without a concrete receiver type |
| Function pointers and dynamic dispatch | Function pointer calls, trait-object calls, and callback registries are not followed |
| Build scripts | `build.rs` and referenced non-Rust assets are copied; generated Rust files under `OUT_DIR` are not semantically modeled, and retained `include!(concat!(env!("OUT_DIR"), ...))` roots are production-blocking hazards |
| Feature/platform cfg matrices | Only `#[cfg(test)]` is pruned as test-only; broader feature/platform matrix evaluation is conservative and selected non-test cfg roots are production-blocking until validation proves the exact matrix |
| External crate pruning | External dependencies are pruned when their crate alias is absent from retained source tokens; full rustc-level unused import analysis is not implemented |

## Verification Commands

```sh
cargo test --workspace
cargo run -p opensource_cli --bin slicers -- --check . /tmp/slicers-proof
cargo check --manifest-path /tmp/slicers-proof/Cargo.toml
```
