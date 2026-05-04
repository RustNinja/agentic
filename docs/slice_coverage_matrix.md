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
| Trait impl methods | root fixture, `trait_ufcs.rs`, `uniffi_mobile.rs`, `component_matrix.rs`, `manifest_hardening.rs` | Receiver calls, explicit `<Type as Trait>::method`, `Trait::method(&receiver, ...)`, generic trait-bound receiver calls, trait-bound method return chaining, format-only `Display`, and `to_string()`-required `Display` impls |
| Ambiguous method fallback | core production-readiness tests, Litter pinned corpus | Unresolved receiver-less method calls retain only a single unambiguous same-name local method; ambiguous same-name sets are capped and reported as feedback hazards instead of pulling unrelated subsystems into the slice |
| Trait impl peers | `component_matrix.rs` | Required peer methods and associated type/const items are retained so trait impls compile |
| Dynamic dispatch boundaries | core production-readiness tests | Retained `dyn Trait` and `fn(...)` function pointer surfaces are production-blocking until semantic analysis can prove concrete dispatch/callback edges, and report package/module/file/line plus the retained surface text |
| Associated types/consts | `component_matrix.rs` | Associated type and associated const dependencies are followed from retained impls |
| External trait imports | `manifest_hardening.rs` | Extension traits such as `tokio::io::AsyncReadExt`, private std traits such as `std::io::Write`, and trait-method imports without an `Ext` suffix such as `base64::Engine` are retained |
| External modules | `module_reexports.rs`, `component_matrix.rs`, core output-safety tests | `mod file;`, `mod/name/mod.rs`, nested modules, empty dead module pruning, and rejection of path-attributed modules that would escape package output |
| Inline modules | `module_reexports.rs`, `component_matrix.rs` | Inline modules with retained reexports are preserved |
| Reexports | `module_reexports.rs`, `component_matrix.rs`, `manifest_hardening.rs` | Reachable `pub use` targets are kept; dead grouped and external `pub use` reexports are pruned |
| Dependency aliases | root fixture, `module_reexports.rs`, `manifest_hardening.rs` | `package = "..."` dependency aliases and renamed imports; renamed local imports are pruned when their resolved target is removed, even when a retained local binding has the same visible name |
| Workspace member globs | `component_matrix.rs` | `[workspace].members = ["crates/*"]` expansion |
| Single-package binary roots | `manifest_hardening.rs`, `docs/real_rtk_slice_report.md` | Non-workspace crates, `src/main.rs` parsing, and stub binary roots |
| Macro-declared modules | `manifest_hardening.rs`, `docs/real_rtk_slice_report.md` | `automod::dir!(... "path")` directories are parsed and rendered as explicit reduced `mod` declarations |
| External dependencies | `uniffi_mobile.rs`, `component_matrix.rs` | Used workspace dependencies such as `serde = { features = ["derive"] }` are preserved; unused external deps with no retained source reference are pruned |
| Build dependencies | `manifest_hardening.rs`, `docs/real_rtk_slice_report.md` | `[build-dependencies]` are retained when `build.rs` is copied |
| Target dependencies | `manifest_hardening.rs` | `[target.'cfg(...)'.dependencies]` tables are preserved when retained source references them |
| Required target features | `manifest_hardening.rs`, CLI unit tests | `required-features` on retained example/test/bench/bin targets are reported and validation rejects Cargo argument sets that would skip the selected target or leave required bin features inactive |
| Feature pruning | `manifest_hardening.rs` | Optional dependency feature entries are removed when the dependency is pruned |
| Local path dependency pruning | `uniffi_mobile.rs`, `component_matrix.rs` | Unused local crates are omitted from manifests and source imports |
| Non-workspace path dependencies | core production-readiness tests | Retained source references to direct or target-specific path dependencies outside the workspace are copied into generated-local `support/` packages when the path resolves to a local Cargo package root; uncopyable path entries remain production-blocking with manifest/package/table details |
| Macro definitions | `component_matrix.rs` | Used `macro_rules!` definitions are kept, unused macro definitions are pruned, and direct helper calls inside retained macro bodies are followed |
| Item macro-generated items | `manifest_hardening.rs`, `docs/real_rtk_slice_report.md` | Item macro invocations such as `lazy_static!` are retained when generated identifiers are referenced by reachable code; inline modules that retain macro-generated source remain valid import targets for child modules |
| Derive and helper-attribute dependencies | `manifest_hardening.rs` | Retained derive/custom-attribute surfaces keep source-mentioned local proc-macro crates whole, and helper functions/items referenced by retained helper attributes are promoted into the graph before rendering |
| Build-script assets | `manifest_hardening.rs`, `docs/real_rtk_slice_report.md`, core production-readiness tests | Non-Rust assets referenced by `build.rs` string literal paths are copied without copying unrelated package docs/config; retained build scripts are production-blocking with package/path details until hermetic build-script modeling exists |
| Source include assets | `manifest_hardening.rs`, core production-readiness tests | Files referenced by retained `include!`, `include_str!`, and `include_bytes!` static literal/`concat!`/`CARGO_MANIFEST_DIR` paths are copied next to the sliced source; retained `include!` Rust source, unknown file include paths, absolute paths, external paths, and `OUT_DIR` generated assets are production-blocking with structured package/module/file/line details |
| Compile-time environment macros | core production-readiness tests | Retained `env!`/`option_env!` macros that read outside Cargo manifest-derived package metadata are production-blocking because they can embed machine-local state, and report structured package/module/file/line details |
| Symlink copy boundaries | `manifest_hardening.rs` | Internal package symlinked include assets are copied at the source-visible link path, while symlinked source or build assets resolving outside the package root are not copied |
| Workspace patches and locks | `manifest_hardening.rs`, CLI unit tests, core production-readiness tests | Root `[patch.*]` tables and `Cargo.lock` are preserved; production validation adds `--locked` for source workspaces with lockfiles so sliced workspaces keep the source repository's dependency resolution, and uncopyable patch/replace path entries are production-blocking with manifest and subject details until copied or pruned |
| Workspace profiles | `manifest_hardening.rs` | Root `[profile.*]` tables are preserved so generated validation does not fall back to Cargo profile defaults |
| Toolchain context | `manifest_hardening.rs` | Root `rust-toolchain.toml` / `rust-toolchain` files are copied so generated validation uses the source workspace's pinned Rust toolchain |
| Analyzer modes | analyzer unit tests, CLI option parsing, Litter pinned corpus | The default CLI feature set enables `ra-hir` and defaults to bounded rust-analyzer HIR semantics for local workspace crates; `--production` defaults to `ra-hir-proc-macros` so rust-analyzer requests build-script output discovery and the sysroot proc-macro server while staying dependency-bounded by default; `--analyzer ra-feedback` is the copy/prove/cut proof-of-concept that queries rust-analyzer outgoing call hierarchy for selected-root and syntactic-retained owner files before applying the same additive retained-edge map; full Cargo dependency artifact discovery is opt-in with `OPENSOURCE_RA_PROC_MACRO_LOAD_DEPS=1` because it timed out on pinned Litter; semantic walking remains bounded to workspace Rust files and reports when proc-macro expansion is unavailable; files containing selected roots are analyzed first, exact project-local RA method/path targets are mapped into generic reduction hints and applied additively, per-file semantic inventory lets the production gate scope semantic warning hazards to retained slice files before selected-root/workspace fallbacks, while `syn` remains an explicit fallback |
| Final production gate | CLI production gate tests | `--production` records `production_ready=accepted` only when compiler feedback passes and no production hazards remain; warning-only hazards now record `production_ready=review_required` instead of being mislabeled as production-ready |
| Feedback runner scale | core feedback tests, Litter pinned corpus | Cargo stdout/stderr are drained while `cargo check --message-format=json` runs, so large dependency graphs cannot deadlock the feedback loop by filling captured output pipes; plain feedback, repair, and production-matrix loops stop on exact repeated diagnostics or repeated diagnostic shapes; repair mode removes repairable unused imports before accepting warning-bearing checks |
| Async functions | `component_matrix.rs` | Async root and async impl method slices build |
| Unit tests | all generated fixtures | `#[test]` functions and `#[cfg(test)]` modules are dropped |
| Non-test cfg roots and surfaces | core production-readiness tests, CLI production gate tests | Selected roots and retained source surfaces behind feature/platform cfgs, including `runtime-benchmarks`, are retained in the graph instead of being pruned as tests; selected cfg-gated roots remain production-blocking unless the cfg expression is proven covered by validation Cargo feature args and host/explicit-target `rustc --print cfg` output, including recognized `all(...)`, `any(...)`, and `not(...)` combinations, and non-root feature cfg surfaces carry structured package/module/file/cfg details that drive an extra production source/generated feature-matrix check |
| UniFFI-shaped API | `uniffi_mobile.rs`, `uniffi_setup.rs` | FFI-facing records/enums, inactive `cfg_attr(..., uniffi::...)`, retained `uniffi::setup_scaffolding!()`, serde DTOs, and mobile bridge shape |
| Real UniFFI project | `docs/real_litter_uniffi_slice_report.md` | Litter `codex-mobile-client` cloud sync and preferences slices build after pruning |
| Real high-star Rust project | `docs/real_rtk_slice_report.md` | RTK `find_corrections` and `filter_json_string` slices build after binary, automod, macro, and build-script hardening |

## Current Boundaries

These are tracked limitations, not silently claimed support:

| Area | Boundary |
| --- | --- |
| Full rustc name resolution | The default CLI flow applies exact local rust-analyzer method/path edges, but the reducer still uses syntactic fallback logic and does not yet replace rustc or rust-analyzer name resolution for trait impl lookup, macro-expanded items, generated source, dynamic dispatch, or all cfg-active reachability decisions |
| Macro-expanded dependencies | The production analyzer can request rust-analyzer proc-macro expansion, but macro-expanded items are not yet mapped into first-class retained source; retained custom derives, custom attributes, module-boundary custom attributes, and non-builtin macro invocations are preserved verbatim, retained helper-attribute paths are promoted into the graph, source-mentioned local proc-macro crates are kept whole, and compiler feedback remains required until macro-expanded inventory feeds the reachability graph |
| Generic trait receiver inference | Local trait-bound receiver calls such as `value.trait_method()` are followed for named type parameters, `where` bounds, `impl Trait` parameters, explicit local bindings, and simple transparent wrappers; full rustc-equivalent inference for associated types, substitutions through arbitrary containers, and complex projection bounds still requires the semantic oracle |
| Function pointers and dynamic dispatch | Function pointer calls, trait-object calls, and callback registries are not followed; retained `fn(...)` and `dyn Trait` surfaces remain production-blocking and report structured package/module/file/line details |
| Build scripts and `include!` source | `build.rs` and referenced non-Rust assets are copied; generated Rust files under `OUT_DIR`, build-script side effects, and any retained `include!` Rust source are not semantically modeled, so they are production-blocking hazards |
| Compile-time environment | `env!`/`option_env!` values outside Cargo package metadata are not modeled and are production-blocking |
| Feature/platform cfg matrices | Only `#[cfg(test)]` is pruned as test-only; broader feature/platform matrix evaluation is conservative, cfg-gated selected roots are production-blocking unless concrete Cargo feature cfgs are covered by `--features`/`--all-features` or recognized target predicates are proven by host/explicit-target `rustc --print cfg`; compound `all(...)`/`any(...)`/`not(...)` expressions are evaluated with tri-state fail-closed semantics, retained non-root Cargo feature cfg surfaces are validated by a bounded production matrix pass, and custom cfgs still require the semantic oracle |
| External crate pruning | External dependencies are pruned when their crate alias is absent from retained source tokens; full rustc-level static unused-import analysis is not implemented, so repair/production validation relies on compiler diagnostics for warning cleanup |
| Non-workspace path dependencies | Copying is bounded to local path package roots that expose `Cargo.toml`; uncopyable path entries remain production-blocking with structured manifest details |

## Verification Commands

```sh
cargo test --workspace
cargo run -p opensource_cli --bin slicers -- --check . /tmp/slicers-proof
cargo check --manifest-path /tmp/slicers-proof/Cargo.toml
```
