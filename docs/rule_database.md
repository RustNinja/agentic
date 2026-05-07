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
| `semantic.*` | RA/syntactic semantic fallback classification and production-review noise |
| `dyn.*` | function pointers, `dyn Trait`, callback registries, async callback aliases |
| `include.*` | `include!`, `include_str!`, `include_bytes!`, copied/dead assets |
| `build.*` | build scripts, `OUT_DIR`, `cargo:rustc-env`, generated files |
| `uniffi.*` | UniFFI records/enums/objects/callbacks/scaffolding/helper attrs |
| `manifest.*` | dependency tables, patches, path dependencies, locks, toolchains |
| `repair.*` | compiler-feedback cleanup, import repair, repeated diagnostics |

Current executable seed cases live in
`crates/opensource_core/tests/rule_database.rs`. The broader generated catalog
of 1,200 generic slice combinations lives in
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
| `import.dependency_barrel.renamed_type.001` | covered | Renamed dependency type reexports used through a local barrel keep the concrete dependency target item while pruning dead sibling dependency aliases and helper reexports |
| `import.dependency_barrel.multi_hop_function.001` | covered | Multi-hop local and dependency barrel reexports resolve live function aliases through support packages while pruning dead support helper functions, dead dependency aliases, and dead leaf items |
| `import.external_reexport.ra_unresolved_benign.001` | covered | Bounded rust-analyzer unresolved path misses are benign when the unresolved symbol is covered by a retained external `use` or `pub use` leaf, while local `crate`/`self`/`super` imports never receive that downgrade |
| `import.module_scoped.dead_item_only.001` | covered | Module-local imports used only by pruned dead items are removed even when the same symbol is live in another module |
| `import.external_group_shadowed_local.001` | covered | Private external grouped import leaves are retained only for actual path/macro uses, so local bindings with the same name do not keep dead sibling imports |
| `import.renamed_alias_shadowed_local.001` | covered | Private renamed import aliases are retained only for actual alias path/macro uses or implicit trait scope effects, so local bindings do not keep aliases to items used only elsewhere |
| `import.direct_leaf_shadowed_local.001` | covered | Private direct import leaves are retained only for actual retained path/macro/surface uses, so a local binding does not keep an imported function or item that is used only from another module |
| `import.super_glob.shadowed_parent_leaf.001` | covered | Child `use super::*` imports retain parent leaves only for actual child path/macro/surface uses, not child local bindings with the same name |
| `dyn.owned.registry.001` | covered | Stored `Box<dyn Trait>` and callback aliases are hard production hazards |
| `dyn.pruned_private_field.001` | covered | Dynamic hazards on private struct fields that are pruned from the rendered slice do not block production readiness |
| `struct.private_generic_field_usage.001` | covered | Private generic fields are pruned when other retained fields already keep the struct type parameter used |
| `include.bytes.static.001` | covered | Retained `include_bytes!` assets are copied and dead sibling assets are not |
| `build.rustc_env.001` | covered | Retained `env!` fed by build script state is production-blocking |
| `trait.default_method_assoc_const.001` | covered | Receiver calls to trait default methods retain the trait item and the concrete impl associated const/type surface needed to compile |
| `dyn.callback.inline_future_field.001` | covered | Inline `Arc<dyn Fn(...) -> Pin<Box<dyn Future...>>>` callback fields report hard dynamic-dispatch hazards without requiring a type alias |
| `static.lazy_lock_closure.001` | covered | `LazyLock::new(|| helper())` static initializers retain helper calls made only inside initializer closures |
| `static.once_lock_get_or_init.001` | covered | `OnceLock::get_or_init(|| Arc::new(T::new()))` singleton helpers retain init guards, constructor calls, and imports while pruning dead singleton builders |
| `static.once_lock_global_runtime_surface.001` | covered | Runtime singleton facades retain `OnceLock<Arc<_>>`, global registries, builder chains, and initializer helpers while pruning dead singleton globals |
| `async.actor_loop.channel_command.001` | covered | Tokio actor-loop roots retain channel sender/receiver/task wiring and prune private dead command variants plus their payload types |
| `async.select_reconnect_loop.001` | covered | Tokio `select!` reconnect loops retain watch/broadcast/mpsc edges and prune dead event siblings |
| `trait.async_io_poll_impl.001` | covered | Generic async I/O helper roots keep the concrete `AsyncRead`/`AsyncWrite` poll impls needed by the selected stream type |
| `ffi.no_mangle_export_surface.001` | covered | Rust 2024-style `#[unsafe(no_mangle)] extern "C"` roots are treated as built-in FFI surfaces, not custom attribute macro hazards |
| `ffi.jni_extern_system_entrypoint.001` | covered | JNI-shaped `extern "system"` exports retain ABI argument imports and live helper calls while pruning dead sibling native exports |
| `macro.clap_nested_command_contract.001` | covered | Clap parser/subcommand/args derive trees retain command/arg helper attrs and nested command structs while pruning unrelated CLI helpers |
| `macro.thiserror_from_source_contract.001` | covered | Thiserror derives retain `#[from]` fields and format-captured error fields while pruning unrelated error enums/helpers |
| `cfg.platform_asset_extern_bundle.001` | covered | Target-cfg platform init roots copy cfg-gated assets and keep cfg-gated extern import blocks intact while pruning dead platform helpers/assets |
| `serde.custom_numeric_string_helpers.001` | covered | Adjacent-tag serde DTOs retain custom numeric/string serializer and deserializer helpers referenced only through serde field attrs |
| `pattern.let_else_slice_enum.001` | covered | Let-else slice patterns retain the enum variant path and prune unrelated helpers without deleting public enum surface variants |
| `const.chain_array_len.001` | covered | Const-to-const arithmetic and const array lengths retain every referenced const and prune dead sibling consts |
| `const.format_capture_identifier.001` | covered | Implicit `format!("{CONST}")` captures retain the referenced const and prune dead sibling consts |
| `macro.metavariable_variant_path.001` | covered | Macro metavariables used inside enum variant paths retain the macro definition, invocation variant token, and helper calls from the macro body |
| `macro.expression_argument_receiver.001` | covered | Parseable expression macro arguments are walked like normal expressions, so receiver method calls inside retained macro invocations keep their concrete impl methods |
| `macro.logging_imported_format_like.001` | covered | Imported `tracing`/`log` event macros such as `debug!`, `trace!`, and `warn!` are treated as format-like inert expression macros only when dependency/import evidence proves the macro source, while their token dependencies are still retained and unknown local macros remain fail-closed |
| `macro.async_trait_dependency_proven.001` | covered | Path-qualified or imported `async_trait` attributes are treated as a known async trait transform only when package dependency evidence proves the macro source, while arbitrary attribute macros remain production feedback blockers |
| `macro.known_derive_contracts.001` | covered | Dependency/import-proven serde `Serialize`/`Deserialize` and thiserror `Error` derives use the slicer's existing contract modeling instead of becoming unknown macro blockers, while unrecognized custom derives remain fail-closed |
| `macro.serde_hook_string_paths.001` | covered | Serde `serialize_with` and `deserialize_with` string helper paths retain the helper functions and prune dead hooks |
| `macro.serde_default_fn_path.001` | covered | Serde `default = "path"` field helper strings retain the default function even when live constructors do not call it |
| `macro.serde_with_module_helpers.001` | covered | Serde `with = "module"` helper modules retain both `serialize` and `deserialize` functions plus their trait imports while pruning dead helper modules |
| `macro.helper_attr.data_string_not_blocker.001` | covered | Serde/thiserror helper-attribute data strings such as rename/tag/error messages do not become macro blockers, while path-valued helper keys still retain the named helper |
| `dyn.async_trait_object.001` | covered | `async_trait` trait-object API surfaces report hard dynamic-dispatch hazards while method-only macro-expanded trait contents remain pruned until semantic dispatch is proven |
| `semantic.method_fallback.no_local_candidates.001` | covered | Unresolved syntactic method fallback calls that retain zero local candidate methods do not create production review hazards |
| `semantic.associated_external_call.no_method_cap.001` | covered | Unresolved associated calls such as `String::new()` do not retain or cap local same-name methods unless the receiver prefix resolves to a local type |
| `semantic.field_option_arc_payload.001` | covered | Closure payloads from struct fields shaped like `Option<Arc<T>>::as_ref().is_some_and(...)` retain methods on the inner `T` without falling back to broad method-name retention |
| `semantic.prunable_retained_package_proof.001` | covered | The usage report separates whole-package pruning from item/function pruning inside retained packages, and marks retained-package prunable code as proven only when RA mapping and reference-search checks succeed without retained-owner references |
| `semantic.complete_proof_clears_generic_warnings.001` | covered | Generic semantic-inventory and semantic-edge production warnings are suppressed only when the retained-package semantic pruning proof is complete; specific unresolved/query hazards still remain fail-closed |
| `dyn.returned_trait_object_boundary.001` | covered | Returned `Box<dyn Trait>` API surfaces are cleared only when a return-position expression visibly constructs a retained local concrete type that implements the trait; forwarded/unconstructed returned trait objects remain dynamic-dispatch hazards |
| `dyn.auto_trait_object_no_dispatch.001` | covered | Auto-trait-only objects such as `dyn Send + Sync` do not create dynamic-dispatch hazards because they expose no callable dispatch surface while still retaining the concrete source type |
| `dyn.wrapped_borrowed_callback_boundary.001` | covered | Transparent selected API wrappers such as `Option<&dyn Trait>` and `Result<fn(...), E>` are reported as direct callback boundary warnings without adding broad owned/stored trait-object noise |
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
| `manifest.support_dependency_item_pruning.001` | covered | No-build copied support packages with concrete retained public names prune dead sibling items and dead local-self impls in the library root and child modules, narrow public facade, transitive facade, local glob facade reexports/imports, and explicit dependency-crate reexport aliases, copy only live static include assets, and omit local path dependencies used only by dead support items |
| `manifest.support_reexported_module_alias.001` | covered | No-build copied support packages resolve required public paths through public module alias reexports, retaining the alias and target module closure while pruning dead sibling aliases/modules |
| `manifest.support_grouped_self_module_alias.001` | covered | Grouped support reexports shaped like `pub use module::{self as alias}` resolve the alias as the module prefix instead of broad-copying dead sibling modules |
| `manifest.support_private_self_module_alias.001` | covered | Private support imports shaped like `use module::{self as alias}` resolve nested `alias::Type` paths to the real module item closure while pruning dead private aliases/functions/modules |
| `manifest.support_nonstandard_lib_root.001` | covered | External support path packages with `[lib] path = "..."` copy the nonstandard library module graph and skip default orphan roots |
| `dyn.callback.future_alias.001` | covered | Nested `Arc<dyn Fn() -> Pin<Box<dyn Future...>>>` aliases are hard hazards |
| `macro.pub_crate_reexport.001` | covered | `pub(crate) use` macro helper reexports survive when live modules invoke them |
| `import.reexport.chain_hub.001` | covered | Reexport chains prune dead grouped names at each public hub |
| `include.str.static_concat.001` | covered | Literal and `concat!` `include_str!` assets are copied while dead siblings are pruned |
| `build.out_dir_source_include.001` | covered | Retained `include!(concat!(env!("OUT_DIR"), ...))` is production-blocking, and simple build-script generated Rust literals retain only referenced helper paths while pruning unrelated dead helpers |
| `include.source.static_helper_refs.001` | covered | Static package-local `include!` Rust is scanned for referenced helper paths so helpers used only by included source are retained without keeping unrelated dead siblings |
| `macro.helper_attr.std_path_noise.001` | covered | Helper attributes such as `#[serde(skip_serializing_if = "Option::is_none")]` do not retain local items named like std/prelude wrapper paths or methods |
| `macro.item_invocation.generated_api.001` | covered | Live macro-generated item invocations survive while dead sibling invocations are pruned |
| `macro.metavariable_method.001` | covered | Methods referenced through `$receiver.method()` in retained `macro_rules!` bodies are resolved from invocation argument types |
| `trait.associated_projection.001` | covered | Associated type/const projections retain the live impl and prune dead projection impls |
| `trait.conversion.try_from_chain.001` | covered | `.try_into()` retains only the matching `TryFrom<Input> for Target` impl, not unrelated conversions for the same target |
| `trait.conversion.bidirectional_from_pair.001` | covered | Boundary roundtrips retain both live `From<A> for B` and `From<B> for A` impls while pruning unrelated conversion impls |
| `dyn.callback.option_arc_trait.001` | covered | Stored `Option<Arc<dyn Trait + Send + Sync>>` callback slots are hard dynamic-dispatch hazards |
| `dyn.callback.nested_store_trait.001` | covered | Nested callback stores such as `Arc<RwLock<Option<Arc<dyn Trait + Send + Sync>>>>` are hard dynamic-dispatch hazards and keep only the live callback trait |
| `dyn.auto_trait.cast_keepalive.001` | covered | Explicit casts to `Arc<dyn Send + Sync>` retain the concrete source type without reporting a dynamic-dispatch hazard because auto-trait-only objects expose no callable dispatch surface |
| `dyn.bare_alias.once_lock_arc.001` | covered | Bare `dyn Fn` type aliases hidden behind `Arc<Alias>` and `OnceLock` still report dynamic-dispatch hazards |
| `dyn.boxed_io_alias.facade.001` | covered | Public module facades reexporting boxed `dyn Read`/`dyn Write` aliases keep the live alias and import, report the trait-object surface, and prune dead sibling aliases/imports |
| `import.facade_const_alias.001` | covered | Local facade const aliases keep only referenced constants and prune dead alias siblings |
| `import.facade_function_alias.001` | covered | Local facade function aliases keep only the called function and prune dead function aliases |
| `import.child_super_parent_import.001` | covered | Parent imports stay live when retained child modules explicitly import them through `super::{...}` and use those names |
| `static.runtime_facade_singleton.001` | covered | Runtime singleton helpers reexported through FFI-style facades keep parent imports needed by child `super` imports and prune dead singleton helpers |
| `include.str.section_selector.001` | covered | Section-selector helpers copy only retained `include_str!` assets and omit dead sibling files |
| `type.result_alias_surface.001` | covered | Public result aliases retain their error and payload surfaces while pruning dead sibling aliases and response records |
| `type.arc_handle_record_surface.001` | covered | Returned records containing `Arc<Handle>` retain the object handle and sidecar info DTO without keeping dead handle records |
| `type.option_vec_dto_surface.001` | covered | Nested `Option<Vec<Dto>>` fields retain the DTO surface and prune dead sibling DTO/request types |
| `type.map_payload_dto_surface.001` | covered | Map payload fields like `BTreeMap<String, Dto>` retain value DTOs and prune dead snapshot siblings |
| `trait.conversion.mirror_try_from_request.001` | covered | Mirror DTO `TryFrom` conversions retain only the selected request/params/helper conversion path and prune dead mirror request families |
| `impl.dead_inherent_method.function_root.001` | covered | Function roots keep only called inherent impl methods and private helpers while pruning unused public/private methods on the same type |
| `type.alias_chain_surface.001` | covered | Public type aliases chained through intermediate aliases retain concrete payload items and prune dead sibling alias families |
| `type.newtype_tuple_surface.001` | covered | Tuple newtype records retain nested tuple field surfaces while pruning dead newtype siblings |
| `type.impl_trait_iterator_item_surface.001` | covered | `impl Iterator<Item = Dto>` return surfaces retain the item DTO and helper constructor without keeping dead iterator families |
| `trait.conversion.question_mark_from_error.001` | covered | `?` retains the exact `From<SourceError> for TargetError` impl and prunes unrelated conversion impls for the same target error |
| `enum.variant_constructor_function.001` | covered | Enum variant constructors used as function values in iterator adapters retain payload structs and prune dead enum families |
| `struct.update_default_surface.001` | covered | Struct update expressions with `..Default::default()` retain the live default-derived record and prune dead default-derived siblings |
| `call.method_reference_iterator.001` | covered | Inherent method references used as iterator adapter functions retain only the referenced associated functions and methods |
| `dyn.boxed_future_return_alias.001` | covered | Boxed `Pin<Box<dyn Future<Output = T>>>` return aliases retain output DTOs, report hard dynamic hazards, and prune dead future aliases |
| `const.array_surface_len.001` | covered | Const-generic array fields in public surfaces retain the live const length and prune dead const/array siblings |
| `type.nested_result_option_alias.001` | covered | Nested `Result<Option<Payload>, Error>` aliases retain payload/error surfaces and prune dead alias payloads |
| `uniffi.callback_interface.facade_reexport.001` | covered | UniFFI-style callback traits reexported through an FFI facade keep trait method record surfaces and prune dead callback siblings |
| `uniffi.object.returned_subscription.001` | covered | Returned UniFFI-style object surfaces retain their exported impl methods and signature DTO/error dependencies without expanding unrelated internal objects |
| `uniffi.object.split_impl_wrapper.001` | covered | Exported wrapper impl methods retain called private same-type helpers and conversion helpers while pruning unused exported/private sibling methods on internally constructed objects |
| `serde.tagged_patch_path_contract.001` | covered | Nested tagged outer contracts with untagged patch path payloads retain patch/path/value DTO surfaces and prune dead patch families |
| `request.method_typed_dispatch_unknown.001` | covered | Method-string dispatch with an unknown fallback retains only selected typed params plus fallback payloads, not dead typed request DTOs |
| `trait.conversion.try_from_option_vec_transpose.001` | covered | Fallible `TryFrom` mirrors using `Option<Vec<T>>`, nested `try_into`, `collect::<Result<Vec<_>, E>>()`, and `transpose()?` retain only the live nested conversion path |
| `include.bytes.function_local_static.001` | covered | Function-local static `include_bytes!` assets are copied when retained and dead local static assets are omitted |
| `macro.compile_env_in_retained_body.001` | covered | Retained `macro_rules!` bodies are scanned for `env!`/`option_env!` hazards even when the env read is hidden behind a live macro invocation |
| `manifest.dependency_crate_alias.001` | covered | `use dependency_crate as alias` qualified paths retain the aliased dependency edge and concrete live dependency items while pruning dead sibling dependency items |
| `import.private_child_wildcard_selected_reexport.001` | covered | Private child wildcard imports combined with selected public reexports keep only the live child helper path and prune dead child reexports/items |
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
| `import.nested_private_facade_chain.001` | covered | Nested private facade modules reexport only selected live constructors and client objects while pruning dead sibling facade names |
| `uniffi.enum.struct_variants.001` | covered | UniFFI-style enum roots with struct variants retain every variant payload DTO/error surface and prune dead enum families |
| `serde.nested_discriminated_response.001` | covered | Nested discriminated serde envelopes retain both outer and inner tagged contracts plus payload DTOs while pruning dead envelope families |
| `serde.multi_deserialize_with_helpers.001` | covered | Multiple `deserialize_with = "module::helper"` fields retain every referenced helper and shared helper imports while pruning unused helper functions |
| `trait.conversion.generic_try_into_bridge.001` | covered | Generic `T: TryInto<Target, Error = E>` helper functions retain only the concrete call-site conversion impl and matching error bridge |
| `error.wire_fallback_conversion.001` | covered | Wire error fallback enums and `map_err(ApiError::from)` retain unknown/fallback variants and exact error conversion impls without dead parsers |
| `include.str.concat_array_assets.001` | covered | Arrays of `include_str!(concat!(...))` assets copy only the live asset set and omit dead sibling files |
| `include.str.once_lock_initializer_asset.001` | covered | `OnceLock::get_or_init` struct initializers retain `include_str!` assets referenced only inside retained initializer closures |
| `dyn.boundary.facade_method_input.001` | covered | Facade-exported borrowed `&dyn Trait` method boundaries stay feedback warnings and prune dead callback traits |
| `import.facade_glob_selected_symbol.001` | covered | Local facade glob reexports can remain syntactically broad while the child module is item-pruned to selected live symbols |

## Workflow

When a real repo slice fails or retains too much code:

1. Identify the smallest generic Rust shape that caused the issue.
2. Add a tiny rule case under `rule_database.rs` or a future grouped rule file.
3. Record the source inspiration in this doc if it came from Litter or another
   real repo.
4. Fix the slicer generically, avoiding project-name/symbol-name allowlists.
5. Run `scripts/fast_rule_loop.sh`, then the broader workspace checks before
   committing.

The target is hundreds of small cargo-checked executable rules plus thousands of
generated catalog slice combinations, not a thousand hand-written one-off tests.
More focused executable cases are valuable only when they introduce a distinct
Rust/Cargo shape or a distinct failure mode.

## Source Evidence

Recent Litter-driven rules came from these source patterns:

| Rule | Source evidence |
| --- | --- |
| `uniffi.callback_interface.facade_reexport.001` | `codex-mobile-client/src/reconnect.rs:73`, `codex-mobile-client/src/ffi/mod.rs:33` |
| `uniffi.object.returned_subscription.001` | `codex-mobile-client/src/ffi/discovery.rs:28`, `codex-mobile-client/src/ffi/discovery.rs:60`, `codex-mobile-client/src/ffi/discovery.rs:435` |
| `uniffi.object.split_impl_wrapper.001` | `codex-mobile-client/src/session/voice_handoff.rs:163`, `codex-mobile-client/src/session/voice_handoff.rs:678` |
| `serde.tagged_patch_path_contract.001` | `codex-ipc/src/protocol/params.rs:49`, `codex-ipc/src/protocol/params.rs:61`, `codex-ipc/src/protocol/params.rs:78` |
| `request.method_typed_dispatch_unknown.001` | `codex-ipc/src/protocol/method.rs:10`, `codex-ipc/src/protocol/params.rs:228`, `codex-ipc/src/protocol/params.rs:318` |
| `trait.conversion.try_from_option_vec_transpose.001` | `codex-mobile-client/src/types/server_requests.rs:312`, `codex-mobile-client/src/types/server_requests.rs:331`, `codex-mobile-client/src/types/server_requests.rs:371` |
| `include.bytes.function_local_static.001` | `codex-bridge/src/lib.rs:76`, `codex-bridge/src/lib.rs:93` |
| `macro.compile_env_in_retained_body.001` | `third_party/codex/codex-rs/utils/cargo-bin/src/lib.rs:118`, `third_party/codex/codex-rs/utils/cargo-bin/src/lib.rs:127` |
| `manifest.dependency_crate_alias.001` | `codex-ipc/src/conversation_state.rs:3`, `codex-mobile-client/src/store/reducer.rs:5`, `codex-mobile-client/src/ffi/client.rs:8` |
| `import.private_child_wildcard_selected_reexport.001` | `codex-mobile-client/src/mobile_client/mod.rs:47`, `codex-mobile-client/src/mobile_client/mod.rs:56` |
| `import.nested_private_facade_chain.001` | `third_party/codex/codex-rs/codex-mcp/src/lib.rs:1`, `third_party/codex/codex-rs/codex-mcp/src/mcp/mod.rs:1` |
| `uniffi.enum.struct_variants.001` | `codex-mobile-client/src/conversation_uniffi.rs:182`, `codex-mobile-client/src/conversation_uniffi.rs:201`, `codex-mobile-client/src/conversation_uniffi.rs:222` |
| `serde.nested_discriminated_response.001` | `codex-ipc/src/protocol/envelope.rs:4`, `codex-ipc/src/protocol/envelope.rs:26`, `codex-ipc/src/protocol/envelope.rs:40` |
| `serde.multi_deserialize_with_helpers.001` | `codex-mobile-client/src/parser.rs:208`, `codex-mobile-client/src/parser.rs:445`, `codex-mobile-client/src/parser.rs:504` |
| `trait.conversion.generic_try_into_bridge.001` | `codex-mobile-client/src/ffi/client.rs:35`, `codex-mobile-client/src/ffi/errors.rs:1`, `codex-mobile-client/src/lib.rs:155` |
| `error.wire_fallback_conversion.001` | `codex-ipc/src/error.rs:4`, `codex-ipc/src/error.rs:42`, `codex-ipc/src/conversation_state.rs:84` |
| `include.str.concat_array_assets.001` | `third_party/codex/codex-rs/tui/src/frames.rs:4`, `third_party/codex/codex-rs/tui/src/frames.rs:47` |
| `include.str.once_lock_initializer_asset.001` | `third_party/codex/codex-rs/hooks/src/engine/schema_loader.rs:21`, `third_party/codex/codex-rs/hooks/src/engine/schema_loader.rs:64` |
| `dyn.boundary.facade_method_input.001` | `third_party/codex/codex-rs/codex-client/src/transport.rs:18`, `third_party/codex/codex-rs/codex-client/src/sse.rs:12` |
| `import.facade_glob_selected_symbol.001` | `codex-ipc/src/lib.rs:20`, `codex-ipc/src/lib.rs:37` |
| `import.external_reexport.ra_unresolved_benign.001` | `codex-ipc/src/protocol/params.rs:1`, `codex-ipc/src/protocol/params.rs:6` |
| `macro.helper_attr.data_string_not_blocker.001` | `codex-ipc/src/protocol/envelope.rs:27`, `codex-ipc/src/error.rs:6` |
| `import.thiserror_derive_leaf_scope.001` | `codex-ipc/src/conversation_state.rs:4`, `codex-ipc/src/conversation_state.rs:6`, `codex-ipc/src/conversation_state.rs:84` |
| `import.grouped_external_private_field_pruned.001` | `codex-ipc/src/client/reconnect.rs:9`, `codex-ipc/src/client/reconnect.rs:35`, `codex-ipc/src/client/reconnect.rs:168`, `codex-ipc/src/client/reconnect.rs:179` |
| `reexport.public_glob.rendered_surface_only.001` | `codex-ipc/src/lib.rs:22`, `codex-ipc/src/lib.rs:32` |

## Litter Patterns To Convert Next

Read-only Litter exploration found these high-value non-cfg patterns:

The deeper gap scan is tracked in `docs/rule_gap_analysis.md`. The next focused
fixture batch should prioritize async actor loops, FFI/JNI export surfaces, clap
derive command contracts, thiserror field contracts, async I/O poll impls,
runtime singletons, platform cfg plus asset bundles, and serde protocol helper
contracts.

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
- Direct callback API boundaries, source `include!` blockers, generated-source
  helper references, dependency aliases inside macro bodies, and inline
  script-bundle modules.
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
- Support dependency typed locals: a retained root can construct a dependency
  type, store it in a local, and call a method later. The support package must
  keep that associated method but still prune unrelated public methods on the
  same type.
- Transitive support reexport aliases: if a copied support facade reexports a
  leaf dependency type under a new name, associated method requirements must be
  forwarded to the original leaf type, including method calls inside macro
  arguments.
- Support type-only surfaces: a retained dependency struct used only as data
  must not keep public inherent methods unless a live call, macro token, or
  unknown surface requires them.
- Shadowed import leaves: local bindings named like removed imports must not
  keep dead `use` leaves alive, including grouped, renamed, direct, and
  parent/child `super::*` import shapes.
- Restricted reexports: `pub(crate)` aliases should be treated as internal
  imports for pruning, so a live target item in its source module does not keep
  an otherwise unused crate-restricted reexport alive.

Cfg/custom-cfg matrix expansion is not expanded blindly. Catalog rules track
cfg gates as move-intact/fail-closed work, and executable rules should be added
only when a concrete reducer/render/validation bug appears.
