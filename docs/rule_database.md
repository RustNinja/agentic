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
| `struct.*` | struct fields, generic field usage, rendered data surfaces |
| `type.*` | type aliases, prelude-name shadowing, surface alias dependencies |
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

## Covered Rule IDs

This table includes the executable `rule_database.rs` cases plus promoted fast
fixture and integration fixture coverage. The current standalone
`rule_database.rs` executable subset is tracked in `docs/slice_coverage_matrix.md`;
extra covered IDs below identify nearby shapes already exercised outside that
file.

| Rule | Status | Generic behavior |
| --- | --- | --- |
| `import.reexport.grouped.001` | covered | Grouped public reexports prune removed names while keeping live names |
| `import.reexport.alias_removed.001` | covered | A removed public alias is pruned even when the alias text appears as a local binding |
| `import.renamed_surface_alias.001` | covered | Renamed local type and trait imports used only by retained struct/impl surfaces keep their resolved target items and prune dead sibling aliases |
| `import.module_scoped.dead_item_only.001` | covered | Module-local imports used only by pruned dead items are removed even when the same symbol is live in another module |
| `dyn.owned.registry.001` | covered | Stored `Box<dyn Trait>` and callback aliases are hard production hazards |
| `dyn.pruned_private_field.001` | covered | Dynamic hazards on private struct fields that are pruned from the rendered slice do not block production readiness |
| `struct.private_generic_field_usage.001` | covered | Private generic fields are pruned when other retained fields already keep the struct type parameter used |
| `include.bytes.static.001` | covered | Retained `include_bytes!` assets are copied and dead sibling assets are not |
| `build.rustc_env.001` | covered | Retained `env!` fed by build script state is production-blocking |
| `trait.default_method_assoc_const.001` | covered | Receiver calls to trait default methods retain the trait item and the concrete impl associated const/type surface needed to compile |
| `dyn.callback.inline_future_field.001` | covered | Inline `Arc<dyn Fn(...) -> Pin<Box<dyn Future...>>>` callback fields report hard dynamic-dispatch hazards without requiring a type alias |
| `static.lazy_lock_closure.001` | covered | `LazyLock::new(|| helper())` static initializers retain helper calls made only inside initializer closures |
| `static.once_lock_get_or_init.001` | covered | `OnceLock::get_or_init(|| Arc::new(T::new()))` singleton helpers retain init guards, constructor calls, and imports while pruning dead singleton builders |
| `pattern.let_else_slice_enum.001` | covered | Let-else slice patterns retain the enum variant path and prune unrelated helpers without deleting public enum surface variants |
| `const.chain_array_len.001` | covered | Const-to-const arithmetic and const array lengths retain every referenced const and prune dead sibling consts |
| `const.format_capture_identifier.001` | covered | Implicit `format!("{CONST}")` captures retain the referenced const and prune dead sibling consts |
| `macro.metavariable_variant_path.001` | covered | Macro metavariables used inside enum variant paths retain the macro definition, invocation variant token, and helper calls from the macro body |
| `macro.serde_hook_string_paths.001` | covered | Serde `serialize_with` and `deserialize_with` string helper paths retain the helper functions and prune dead hooks |
| `macro.serde_default_fn_path.001` | covered | Serde `default = "path"` field helper strings retain the default function even when live constructors do not call it |
| `macro.serde_with_module_helpers.001` | covered | Serde `with = "module"` helper modules retain both `serialize` and `deserialize` functions plus their trait imports while pruning dead helper modules |
| `dyn.async_trait_object.001` | covered | `async_trait` trait-object API surfaces report hard dynamic-dispatch hazards while method-only macro-expanded trait contents remain pruned until semantic dispatch is proven |
| `dyn.returned_trait_object_boundary.001` | covered | Returned `Box<dyn Trait>` API surfaces report hard dynamic-dispatch hazards while retaining the trait object type surface |
| `ffi.extern_called_symbol.001` | covered | Retained calls to foreign `extern "C"` functions keep only the called foreign declarations and prune dead sibling declarations |
| `ffi.extern_static_symbol.001` | covered | Retained reads of foreign `extern "C"` statics keep only the referenced static declaration and the imports used by that foreign item |
| `macro.zero_arg_generated_item.001` | covered | Zero-argument item macro invocations survive when their macro definition body generates a function called by retained code, while dead sibling invocations stay pruned |
| `macro.fixed_name_generated_item.001` | covered | Item macros that generate a fixed public function remain live even when invocation arguments do not mention the generated name |
| `macro.inline_module_item_invocation.001` | covered | Item macro invocations inside inline modules retain their lexical macro definition and body dependencies |
| `macro.shadowed_definition_scope.001` | covered | Same-named `macro_rules!` definitions in sibling modules resolve by lexical scope instead of package/name fallback |
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
| `closure.result_error_payload_primitive_ok.001` | covered | `Result::map_err` and error inspectors bind the real error payload even when the `Ok` type is primitive and absent from the local item graph |
| `type.local_prelude_shadow_alias.001` | covered | Local aliases named like prelude/std types, such as `Result<T>`, stay live when retained signatures mention them |
| `closure.local_generic_method_payload.001` | covered | Project-local generic methods that accept closures substitute impl generics from concrete receiver type arguments before walking closure bodies |
| `closure.local_option_is_some_and.001` | covered | Local `Option<T>` bindings feed `is_some_and` closure payload typing without pretending the binding itself is `T` |
| `ffi.callback_direct_input.001` | covered | Retained `extern "C" fn(...)` callback inputs are reported as direct API boundary warnings while preserving their signature imports |
| `trait.rendered_impl_surface_dependencies.001` | covered | Trait impls rendered because both self type and public trait surface are reachable retain impl header, associated item, and method-body dependencies |
| `trait.pruned_type_surface_method_hazard.001` | covered | Trait methods pruned from a type-only trait surface do not contribute false function-pointer production hazards |
| `trait.required_surface_member_pruning.001` | covered | Type-surface trait retention keeps required/mentioned trait members needed by rendered impls while pruning optional default-method dependencies and dead impl overrides |
| `trait.peer_receiver_precision.001` | covered | Resolved trait method peers are keyed by receiver type and trait input types so a live trait method does not retain same-named impl methods for unrelated receiver/input types |
| `serde.flatten_contract_field.001` | covered | Serde contract fields such as `#[serde(flatten)]` stay even when private and not read by live bodies, while unannotated dead private fields are pruned |
| `serde.flatten_nested_payload.001` | covered | Nested serde-flatten payload structs stay when the retained DTO contract depends on them, while dead nested DTOs prune away |
| `serde.deserialize_with_private_wire.001` | covered | Private wire DTOs using `#[serde(deserialize_with = "helper")]` retain helper functions, deserializer trait imports, and the private DTO surface used by `serde_json::from_str::<T>` |
| `serde.alias_private_wire.001` | covered | Private serde wire DTO fields with alias attrs remain as deserialization contract fields even when live code only reads normalized output |
| `serde.transparent_record_contract.001` | covered | `#[serde(transparent)]` wrapper records retain their field contract and wrapper item when used through serde parsing |
| `serde.skip_serializing_if_option_path.001` | covered | `#[serde(skip_serializing_if = "Option::is_none")]` fields stay as serialization contract fields without needing a local helper path |
| `serde.untagged_enum_contract.001` | covered | `#[serde(untagged)]` public enum variants remain intact as data-contract variants while unrelated sibling enums prune away |
| `serde.tagged_enum_payload_contract.001` | covered | Internally tagged serde enums retain field-bearing payload variants and variant attrs when used as a parsing contract |
| `serde.default_enum_variant_contract.001` | covered | `Default` serde enum variants and `#[serde(default)]` fields retain the enum contract while dead sibling enums prune away |
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
| `trait.conversion.bidirectional_from_pair.001` | covered | Boundary roundtrips retain both live `From<A> for B` and `From<B> for A` impls while pruning unrelated conversion impls |
| `dyn.callback.option_arc_trait.001` | covered | Stored `Option<Arc<dyn Trait + Send + Sync>>` callback slots are hard dynamic-dispatch hazards |
| `dyn.callback.nested_store_trait.001` | covered | Nested callback stores such as `Arc<RwLock<Option<Arc<dyn Trait + Send + Sync>>>>` are hard dynamic-dispatch hazards and keep only the live callback trait |
| `dyn.auto_trait.cast_keepalive.001` | covered | Explicit casts to `Arc<dyn Send + Sync>` retain the concrete source type while reporting the type-erased surface as a hazard |
| `dyn.bare_alias.once_lock_arc.001` | covered | Bare `dyn Fn` type aliases hidden behind `Arc<Alias>` and `OnceLock` still report dynamic-dispatch hazards |
| `dyn.boundary.direct_inputs.001` | covered | Selected `&dyn Trait` and `fn(...)` callback inputs stay as feedback-dischargeable API boundary warnings |
| `include.source.static.001` | covered | Retained plain `include!("...rs")` source inclusions are production-blocking |
| `include.source.inline_fallback_module.001` | covered | Fallback-retained inline modules run the full syntactic hazard scan, so plain source includes are reported even when the module was retained by path mention |
| `macro.external_crate_alias_body.001` | covered | Dependency aliases used only inside retained macro bodies keep the aliased dependency edge |
| `macro.serde_json.qualified_json.001` | covered | Fully qualified `serde_json::json!` macro invocations retain the dependency without keeping dead imported `json` uses |
| `include.str.inline_module_tree.001` | covered | Inline module script bundles copy only live `include_str!` assets and prune dead sibling assets |
| `build.option_env.001` | covered | Retained `option_env!` reads of non-Cargo metadata are production-blocking and dead sibling env readers are pruned |
| `manifest.support_library_module_closure.001` | covered | No-build support path packages copy only the library external-module graph plus live static assets, skipping `#[cfg(test)]` external modules and orphan Rust files |
| `manifest.support_build_script_hazard_parity.001` | covered | Copied support path packages report build-script, `OUT_DIR` source include, compile-time env, nonliteral include, absolute include, and package-external include hazards with generated support file details |
| `manifest.local_build_dependency.no_build_script.001` | covered | Local build-dependencies are rendered only when a retained package has a retained build script, so no-build packages do not keep build-only workspace crates |
| `manifest.local_dependency_edge.pruned_retained_package.001` | covered | A globally retained local package does not force every source package to keep an unused dependency edge or feature entry pointing at it |
| `manifest.local_dependency_alias.local_ident_false_positive.001` | covered | Local path dependencies are not retained just because a local variable/type identifier matches the dependency alias; direct dependency retention requires a path/import prefix |
| `macro.path_qualified_derive.001` | covered | Path-qualified derive macros such as `macro_helpers::FixtureRecord` keep the proc-macro package but do not retain unused simple `use` imports |
| `macro.proc_attr_import_retention.001` | covered | Unqualified retained custom attributes keep the proc-macro import that brings the attribute into scope while still pruning unused derive-only imports |
| `macro.proc_macro_export_pruning.001` | covered | Local proc-macro crates retain only exported derive/attribute/function-like proc macros referenced by reachable rendered surfaces, while unused proc-macro exports and their private helpers are pruned |
| `macro.proc_macro_helper_support.001` | covered | Local helper library crates used by retained proc-macro exports are copied as pruned `support/` packages instead of being promoted to root workspace members, and dead proc-macro helper crates are omitted |
| `macro.reachable_module_attr.001` | covered | Path-qualified proc-macro attributes on modules are retained when nested reachable code causes the module boundary to render |
| `macro.inline_impl_attr.001` | covered | Proc-macro attributes on impl blocks inside inline modules remain when reachable constructors or methods require the impl surface |
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
- Support facade exports where one retained DTO or constant should not keep a
  whole protocol/app-server support module fanout.
- Transitive reexport chains through local and copied support crates where dead
  middle-layer names must not pull sibling upstream modules.
- Support crates with target-gated assets, inline test modules, legacy facade
  reexports, websocket/remote-control siblings, or monolithic protocol files
  where the next production gap is item-level support pruning after the current
  module/file-level copy.

Cfg/custom-cfg matrix expansion is not expanded blindly. Catalog rules track
cfg gates as move-intact/fail-closed work, and executable rules should be added
only when a concrete reducer/render/validation bug appears.
