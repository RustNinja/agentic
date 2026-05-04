# Rule Database

The slicer rule database is the growing set of tiny, generic Rust cases that
turn real failures into reusable coverage. A rule is not tied to a project name
or symbol name. It captures one Rust/Cargo shape, the expected retained/pruned
surface, and any production hazards that must be reported.

## Rule Groups

Use stable group prefixes when adding cases:

| Prefix | Purpose |
| --- | --- |
| `import.*` | `use`, `pub use`, glob, rename, shadowing, and reexport cleanup |
| `macro.*` | `macro_rules!`, item macros, proc macro derives/attrs, helper attrs |
| `trait.*` | trait impls, associated types/consts, UFCS, blanket impls, projections |
| `dyn.*` | function pointers, `dyn Trait`, callback registries, async callback aliases |
| `include.*` | `include!`, `include_str!`, `include_bytes!`, copied/dead assets |
| `build.*` | build scripts, `OUT_DIR`, `cargo:rustc-env`, generated files |
| `uniffi.*` | UniFFI records/enums/objects/callbacks/scaffolding/helper attrs |
| `manifest.*` | dependency tables, patches, path dependencies, locks, toolchains |
| `repair.*` | compiler-feedback cleanup, import repair, repeated diagnostics |

Current executable seed cases live in
`crates/opensource_core/tests/rule_database.rs`. The broader generated catalog
of planned generic combinations lives in
`crates/opensource_core/tests/rule_catalog.rs` and is documented in
`docs/rule_catalog.md`.

## First Seed Rules

| Rule | Status | Generic behavior |
| --- | --- | --- |
| `import.reexport.grouped.001` | covered | Grouped public reexports prune removed names while keeping live names |
| `import.reexport.alias_removed.001` | covered | A removed public alias is pruned even when the alias text appears as a local binding |
| `import.renamed_surface_alias.001` | covered | Renamed local type and trait imports used only by retained struct/impl surfaces keep their resolved target items and prune dead sibling aliases |
| `dyn.owned.registry.001` | covered | Stored `Box<dyn Trait>` and callback aliases are hard production hazards |
| `include.bytes.static.001` | covered | Retained `include_bytes!` assets are copied and dead sibling assets are not |
| `build.rustc_env.001` | covered | Retained `env!` fed by build script state is production-blocking |
| `trait.default_method_assoc_const.001` | covered | Receiver calls to trait default methods retain the trait item and the concrete impl associated const/type surface needed to compile |
| `dyn.callback.inline_future_field.001` | covered | Inline `Arc<dyn Fn(...) -> Pin<Box<dyn Future...>>>` callback fields report hard dynamic-dispatch hazards without requiring a type alias |
| `static.lazy_lock_closure.001` | covered | `LazyLock::new(|| helper())` static initializers retain helper calls made only inside initializer closures |
| `pattern.let_else_slice_enum.001` | covered | Let-else slice patterns retain the enum variant path and prune unrelated helpers without deleting public enum surface variants |
| `const.chain_array_len.001` | covered | Const-to-const arithmetic and const array lengths retain every referenced const and prune dead sibling consts |
| `macro.metavariable_variant_path.001` | covered | Macro metavariables used inside enum variant paths retain the macro definition, invocation variant token, and helper calls from the macro body |
| `macro.serde_hook_string_paths.001` | covered | Serde `serialize_with` and `deserialize_with` string helper paths retain the helper functions and prune dead hooks |
| `macro.serde_default_fn_path.001` | covered | Serde `default = "path"` field helper strings retain the default function even when live constructors do not call it |
| `dyn.async_trait_object.001` | covered | `async_trait` trait-object API surfaces report hard dynamic-dispatch hazards while method-only macro-expanded trait contents remain pruned until semantic dispatch is proven |
| `ffi.extern_called_symbol.001` | covered | Retained calls to foreign `extern "C"` functions keep only the called foreign declarations and prune dead sibling declarations |
| `macro.zero_arg_generated_item.001` | covered | Zero-argument item macro invocations survive when their macro definition body generates a function called by retained code, while dead sibling invocations stay pruned |
| `import.external_source_trait_method.001` | covered | Path dependency trait imports are retained by reading the source trait method names when the trait name does not imply the called method |
| `trait.derive_array_field_impl.001` | covered | Derive-driven trait impl retention descends into array, slice, pointer, and nested field types so generated derives keep required field impls |
| `trait.associated_type_equality_method_chain.001` | covered | Generic bounds such as `T: Trait<Item = Payload>` propagate `Self::Item` return types into following method calls |
| `pattern.tuple_destructure_receiver.001` | covered | Tuple destructuring locals inherit return-position element types so method calls on destructured bindings keep the correct impls |
| `pattern.struct_destructure_receiver.001` | covered | Struct destructuring locals inherit field types from returned structs so method calls on destructured bindings keep the correct impls |
| `pattern.typed_param_destructure_receiver.001` | covered | Typed destructured function parameters bind inner receiver types before body analysis |
| `pattern.for_loop_item_receiver.001` | covered | For-loop item bindings inherit iterable output element types such as `Vec<T>` before loop body analysis |
| `closure.free_function_input_payload.001` | covered | Closures passed to resolved free functions inherit callable input payload types so methods inside closure bodies keep the correct impls |
| `pattern.struct_match_receiver.001` | covered | Struct patterns in `match`/`if let` bind real struct field receiver types instead of only enum variant payloads |
| `closure.option_result_payload_map.001` | covered | Common `Option`/`Result` closure combinators such as `map`, `and_then`, `map_err`, and inspectors bind payload/error types from receiver type arguments |
| `serde.flatten_contract_field.001` | covered | Serde contract fields such as `#[serde(flatten)]` stay even when private and not read by live bodies, while unannotated dead private fields are pruned |
| `manifest.support_path_bundle.001` | covered | External support path dependency bundles rewrite absolute paths to generated relative support paths, copy dependency closure assets, and drop dead bins/examples/tests/benches/fixtures/orphan modules |
| `manifest.support_nonstandard_lib_root.001` | covered | External support path packages with `[lib] path = "..."` copy the nonstandard library module graph and skip default orphan roots |
| `dyn.callback.future_alias.001` | covered | Nested `Arc<dyn Fn() -> Pin<Box<dyn Future...>>>` aliases are hard hazards |
| `macro.pub_crate_reexport.001` | covered | `pub(crate) use` macro helper reexports survive when live modules invoke them |
| `import.reexport.chain_hub.001` | covered | Reexport chains prune dead grouped names at each public hub |
| `include.str.static_concat.001` | covered | Literal and `concat!` `include_str!` assets are copied while dead siblings are pruned |
| `build.out_dir_source_include.001` | covered | Retained `include!(concat!(env!("OUT_DIR"), ...))` is production-blocking |
| `macro.item_invocation.generated_api.001` | covered | Live macro-generated item invocations survive while dead sibling invocations are pruned |
| `macro.metavariable_method.001` | covered | Methods referenced through `$receiver.method()` in retained `macro_rules!` bodies are resolved from invocation argument types |
| `trait.associated_projection.001` | covered | Associated type/const projections retain the live impl and prune dead projection impls |
| `trait.conversion.try_from_chain.001` | covered | `.try_into()` retains only the matching `TryFrom<Input> for Target` impl, not unrelated conversions for the same target |
| `dyn.callback.option_arc_trait.001` | covered | Stored `Option<Arc<dyn Trait + Send + Sync>>` callback slots are hard dynamic-dispatch hazards |
| `dyn.boundary.direct_inputs.001` | covered | Selected `&dyn Trait` and `fn(...)` callback inputs stay as feedback-dischargeable API boundary warnings |
| `include.source.static.001` | covered | Retained plain `include!("...rs")` source inclusions are production-blocking |
| `macro.external_crate_alias_body.001` | covered | Dependency aliases used only inside retained macro bodies keep the aliased dependency edge |
| `include.str.inline_module_tree.001` | covered | Inline module script bundles copy only live `include_str!` assets and prune dead sibling assets |
| `manifest.support_library_module_closure.001` | covered | No-build support path packages copy only the library external-module graph plus live static assets, skipping `#[cfg(test)]` external modules and orphan Rust files |
| `manifest.support_build_script_hazard_parity.001` | covered | Copied support path packages report build-script, `OUT_DIR` source include, compile-time env, nonliteral include, absolute include, and package-external include hazards with generated support file details |
| `macro.path_qualified_derive.001` | covered | Path-qualified derive macros such as `macro_helpers::FixtureRecord` keep the proc-macro package but do not retain unused simple `use` imports |
| `macro.root_item_impl_surface.001` | covered | Selected item roots with macro-bearing inherent impls keep exported constructors/methods plus their signature/body dependencies |
| `macro.inline_root_item_impl_surface.001` | covered | Selected item roots inside inline modules keep macro-bearing impl surfaces, helper constructor/method dependencies, and the imports referenced by retained impl bodies |
| `dyn.callback.registry_object.001` | covered | Selected object roots with stored `Arc<dyn Trait + Send + Sync>` callback fields keep the callback trait method surface and report hard dynamic-dispatch hazards |
| `dyn.callback.future_static.001` | covered | Selected callback APIs using `Arc<dyn Fn(...) -> Pin<Box<dyn Future...>>>` aliases and static registries keep aliases/statics while reporting hard dynamic hazards |
| `include.bytes.fast_fixture.001` | covered | The fast fixture now exercises `include_bytes!` beside `include_str!` literal and `concat!` assets |
| `trait.generic_header_bounds.001` | covered | Rendered struct/trait/impl headers keep local generic and where-clause bounds even when the retained method body only mentions `Self` or fields |
| `import.inline_facade_reexport.001` | covered | Private inline modules reexporting public facade objects prune dead aliases while retaining imports required by the live facade object surface |

## Workflow

When a real repo slice fails or retains too much code:

1. Identify the smallest generic Rust shape that caused the issue.
2. Add a tiny rule case under `rule_database.rs` or a future grouped rule file.
3. Record the source inspiration in this doc if it came from Litter or another
   real repo.
4. Fix the slicer generically, avoiding project-name/symbol-name allowlists.
5. Run `scripts/fast_rule_loop.sh`, then the broader workspace checks before
   committing.

The target is hundreds of small executable rules plus thousands of generated
catalog combinations, not a thousand hand-written one-off tests. More executable
cases are valuable only when they introduce a distinct Rust/Cargo shape or a
distinct failure mode.

## Litter Patterns To Convert Next

Read-only Litter exploration found these high-value non-cfg patterns:

- UniFFI object roots with `#[derive(uniffi::Object)]`, exported impls,
  constructors, async methods, private inner state, and public facade reexports.
- Private UniFFI object modules reexported through a public FFI facade.
- Exported async UniFFI impl blocks on reexported objects.
- Callback interface roots stored as `Option<Arc<dyn CallbackTrait>>`.
- `async_trait` traits used through `Arc<dyn Trait>`.
- Nested callback/future aliases like
  `Arc<dyn Fn() -> Pin<Box<dyn Future<...>>>>`, including static callback
  registries.
- Nested callback stores such as `Arc<RwLock<Option<Arc<dyn Trait>>>>`.
- `macro_rules!` helper modules that `pub(crate) use` a macro and invoke it from
  exported methods.
- Error enums combining `thiserror::Error` and `uniffi::Error`.
- Static `include_str!` and `include_bytes!` asset groups.
- Broad `pub use` hubs that need live-name pruning through reexport chains.
- Serde/UniFFI helper attrs such as field defaults and skip helpers.
- Split derive/helper attrs, including serde attrs between `derive(...)` and
  UniFFI derives, path-qualified derive macros, `#[serde(transparent)]`
  records, default enum variants, and tagged enums with payload variants.
- Conversion-heavy boundary impls such as `TryFrom<Request> for Params`.
- Adjacent bidirectional `From<A> for B` / `From<B> for A` impls across FFI and
  internal types.
- Macro bodies that call methods on metavariables, where the invocation argument
  type is the only generic way to retain the required method.
- Direct callback API boundaries, source `include!` blockers, dependency aliases
  inside macro bodies, and inline script-bundle modules.
- Support path dependencies that compile but over-copy examples, tests, benches,
  dev/build-only path packages, and fixture assets.
- Support path dependencies with library test modules, orphan Rust files,
  support binaries, support build scripts, and static assets referenced only by
  copied support modules.
- Exported free functions or object impls with cfg-split bodies, preserving cfg
  attributes intact without expanding a broad custom cfg matrix.
- Public async functions with boxed dyn callback args, generic connector
  functions erased into boxed futures, and foreign trait impls with associated
  errors plus return-position `impl Future`.
- Public type aliases for `Box<dyn AsyncRead/AsyncWrite + Send + Unpin>`
  reexported from module facades.
- Generic async helper functions with `where` bounds over sink/stream traits and
  associated error projections.
- Root-level setup macros that appear after module declarations, exported
  functions, and facade `pub use` statements.

Cfg/custom-cfg matrix expansion is not expanded blindly. Catalog rules track
cfg gates as move-intact/fail-closed work, and executable rules should be added
only when a concrete reducer/render/validation bug appears.
