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
| `fixture.*` | checked-in category fixture workspaces with explicit expected slice output |

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
| `manifest.support_private_direct_module_alias.001` | covered | Private support imports shaped like `use module::leaf as alias` resolve nested `alias::Type` paths to the leaf module closure while pruning dead sibling aliases/functions/modules |
| `manifest.support_inline_facade_module.001` | covered | Inline support facade modules are modeled as virtual module nodes, so `pub mod facade { pub use crate::leaf::*; }` retains only live reexports/items without broad-copying dead sibling support modules |
| `manifest.support_nested_inline_facade_module.001` | covered | Nested inline support facade modules resolve multi-hop paths such as `facade::nested::Type` and prune dead sibling inline facades plus dead support modules |
| `manifest.support_inline_facade_external_child.001` | covered | Inline support facades that declare file-backed child modules and `pub use child::*` resolve through both virtual and file-backed nodes while repeated dead glob misses do not block later live reexports |
| `manifest.support_pub_crate_glob_private_use.001` | covered | Restricted support reexports such as `pub(crate) use module::*` are pruned as private imports when retained code uses direct module paths, while the live target module/items stay rendered |
| `manifest.support_macro_reexport_alias_assoc.001` | covered | Copied support crates propagate methods called on macro metavariable receivers through reexported dependency type aliases, retaining only the required upstream associated methods |
| `fixture.trim_unused.workspace.001` | covered | A checked-in three-package Rust workspace fixture (`root`, `used`, `unused`) slices a selected root through the default analyzer path, keeps only the used package/items, prunes the unused dependency package, and cargo-checks the generated workspace |
| `fixture.sub_dependency_prune.used_closure.001` | covered | A checked-in path-dependency fixture slices a selected root through a direct dependency and its sub-dependency, proving generated dependency crates keep only the used callable/module/method closure, including a macro receiver call on a field expression, while pruning dead reexports, modules, files, functions, and sibling methods |
| `fixture.import_alias_prune.support_chain.001` | covered | A checked-in four-package support chain fixture slices through grouped imports, aliases, public prelude reexports, and macro receiver calls, proving generated dependency crates keep only used aliases/modules/functions and remove dead support modules, imports, and helper package items |
| `fixture.proc_macro_surface_prune.support_chain.001` | covered | A checked-in local proc-macro support fixture slices through retained derive and attribute macro surfaces, keeps helper attribute paths such as `model::wire_tag`, prunes unused derive/attribute macro exports, and removes dead API/model modules while cargo-checking the generated workspace |
| `fixture.asset_conversion_prune.support_chain.001` | covered | A checked-in support fixture slices through `include_str!` literal/`concat!` assets plus `TryFrom` conversion/error surfaces, copies only live assets, prunes dead support assets/modules/functions, and cargo-checks the generated workspace |
| `fixture.callback_boundary_prune.support_chain.001` | covered | A checked-in support fixture slices through a direct `&dyn Trait` and `fn(...)` callback boundary, keeps explicit callback-boundary hazard evidence, prunes dead callback modules/functions, and cargo-checks the generated workspace |
| `fixture.trait_ufcs_prune.support_chain.001` | covered | A checked-in support fixture slices through trait default methods, associated consts, and explicit UFCS calls while pruning dead trait modules, dead impl families, and dead live sibling methods |
| `fixture.iterator_result_prune.support_chain.001` | covered | A checked-in support fixture slices through public result aliases and `impl Iterator<Item = Dto>` surfaces while pruning dead stream modules, dead envelope/event families, and dead live iterator siblings |
| `fixture.facade_glob_prune.support_chain.001` | covered | A checked-in support fixture slices through inline and nested public facade glob reexports, keeps the imported live facade path across package boundaries, and prunes dead facade modules/items/functions |
| `fixture.macro_generated_prune.support_chain.001` | covered | A checked-in support fixture slices through a live `macro_rules!` item generator invocation, keeps the macro definition and live generated API, and prunes the dead generated invocation plus dead API surface |
| `fixture.static_registry_prune.support_chain.001` | covered | A checked-in support fixture slices through a `OnceLock<Mutex<Option<Arc<_>>>>` registry, keeps the live static/slot/handle closure, and prunes dead registry modules/APIs from both dependency crates |
| `fixture.patch_state_prune.support_chain.001` | covered | A checked-in support fixture slices through a request/patch state machine with `TryFrom`, enum operation, `Vec`, and `Option` surfaces, keeps the live request/segment/error closure, and prunes dead patch API/model surfaces |
| `fixture.uniffi_runtime_prune.support_chain.001` | covered | A checked-in support fixture slices through a UniFFI-shaped shared runtime object with `OnceLock<Arc<_>>`, keeps the live shared runtime/status closure, and prunes dead runtime object/API surfaces |
| `fixture.async_command_prune.support_chain.001` | covered | A checked-in support fixture slices through async command/state functions across path dependencies, keeps the live async enum/state/handler closure, and prunes dead async command APIs |
| `fixture.ffi_export_prune.support_chain.001` | covered | A checked-in support fixture slices through a retained `#[no_mangle] extern "C"` export and pointer helper chain, keeping only the selected FFI helper closure while pruning dead FFI exports/state |
| `fixture.conversion_roundtrip_prune.support_chain.001` | covered | A checked-in support fixture slices through bidirectional `From` conversion surfaces across dependency packages, keeping only the live wire/domain conversion closure and pruning dead wire APIs |
| `fixture.macro_receiver_prune.support_chain.001` | covered | A checked-in support fixture slices through a macro receiver flow where a macro-local binding calls a method on the return type of a `$metavariable.method()` call, retaining both method dependencies and pruning dead macro APIs |
| `fixture.error_source_prune.support_chain.001` | covered | A checked-in support fixture slices through error/source conversion with `?`, `From`, nested source records, and match rendering while pruning dead error APIs and source methods |
| `fixture.poll_adapter_prune.support_chain.001` | covered | A checked-in support fixture slices through `std::task::Poll` adapter state, retaining the live frame/poller/poll method closure while pruning dead poll adapters |
| `fixture.type_alias_surface_prune.support_chain.001` | covered | A checked-in support fixture slices through nested `Result<Option<Vec<Dto>>, Error>` type alias surfaces, retaining DTO/error/render closure and pruning dead envelope APIs |
| `fixture.callback_store_prune.support_chain.001` | covered | A checked-in support fixture slices through stored `Arc<RwLock<Option<Arc<dyn Trait>>>>` callback state, keeping scoped trait-object hazard evidence while pruning dead callback packages/modules |
| `fixture.const_chain_prune.support_chain.001` | covered | A checked-in support fixture slices through const/static/associated-const chains and keeps only the live const record closure while pruning dead const APIs |
| `fixture.struct_update_prune.support_chain.001` | covered | A checked-in support fixture slices through `Default` plus struct update syntax, retaining the live settings surface and pruning dead builder/model APIs |
| `fixture.pattern_destructure_prune.support_chain.001` | covered | A checked-in support fixture slices through `let-else` enum struct-pattern destructuring, retaining the live parser/event closure and pruning dead pattern APIs |
| `fixture.closure_combinator_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option`/`Result` closure combinators and `transpose`, retaining live payload/error methods while pruning dead closure APIs |
| `fixture.generic_bound_prune.support_chain.001` | covered | A checked-in support fixture slices through generic trait bounds, retained impl methods, and dependency package reexports while pruning dead generic APIs |
| `fixture.enum_variant_constructor_prune.support_chain.001` | covered | A checked-in support fixture slices through enum variant constructors used as iterator function values while pruning dead enum APIs and sibling support modules |
| `fixture.associated_projection_prune.support_chain.001` | covered | A checked-in support fixture slices through associated type projections and projected render bounds while pruning dead projection APIs |
| `fixture.iterator_method_ref_prune.support_chain.001` | covered | A checked-in support fixture slices through iterator method references such as `map(Type::method)`, retaining the used method closure while pruning dead iterator APIs |
| `fixture.display_format_prune.support_chain.001` | covered | A checked-in support fixture slices through captured `format!` arguments, retaining the concrete `Display` impl required by retained formatting while pruning dead display APIs |
| `fixture.deref_method_prune.support_chain.001` | covered | A checked-in support fixture slices through autoderef method calls, retaining the `Deref` target method closure while pruning dead wrapper and target methods |
| `fixture.question_mark_conversion_prune.support_chain.001` | covered | A checked-in support fixture slices through `?` error conversion plus `Err(err)` match rendering, retaining the conversion impl and error render method while pruning dead question APIs |
| `fixture.match_guard_prune.support_chain.001` | covered | A checked-in support fixture slices through match-guard receiver methods, retaining guard predicate/render methods while pruning dead guard APIs |
| `fixture.from_str_parse_prune.support_chain.001` | covered | A checked-in support fixture slices through `.parse::<T>()`, retaining the concrete `FromStr` impl, parsed value methods, and `Err(T::Err)` rendering while pruning dead parse APIs |
| `fixture.index_operator_prune.support_chain.001` | covered | A checked-in support fixture slices through indexing expressions such as `store[0].render()`, retaining the concrete `Index` impl and output method closure while pruning dead index APIs |
| `fixture.option_field_payload_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option` payloads reached from struct fields and `as_ref()`, retaining only the live payload render method while pruning dead option APIs |
| `fixture.closure_return_prune.support_chain.001` | covered | A checked-in support fixture slices through closure-return receiver calls such as `build().render()`, retaining the returned type method closure while pruning dead closure-return APIs |
| `fixture.try_from_transpose_prune.support_chain.001` | covered | A checked-in support fixture slices through fallible `TryFrom` conversion inside `Option<Vec<_>>::map(...).transpose()`, retaining converted payload methods and pruning dead conversion APIs |
| `fixture.returned_object_prune.support_chain.001` | covered | A checked-in support fixture slices through a returned object exported from a support package, retaining macro-exported impl surface methods and pruning dead object impl methods |
| `fixture.method_dispatch_prune.support_chain.001` | covered | A checked-in support fixture slices through string-typed method dispatch with a retained unknown fallback variant while pruning dead dispatch parameters and APIs |
| `fixture.map_payload_prune.support_chain.001` | covered | A checked-in support fixture slices through map payload traversal via `.values().map(Type::method)`, retaining only the live entry render closure while pruning dead map APIs |
| `fixture.free_function_closure_prune.support_chain.001` | covered | A checked-in support fixture slices through closures passed to resolved free functions, retaining the closure payload method chain while pruning dead free-function and payload siblings |
| `fixture.result_map_err_prune.support_chain.001` | covered | A checked-in support fixture slices through `Result::map(...).map_err(...)`, preserving the error payload type across OK-only combinators so only the live error render closure remains |
| `fixture.const_generic_array_prune.support_chain.001` | covered | A checked-in support fixture slices through const-generic array return values and keeps the returned payload impl method plus live array length const while pruning dead array/model APIs |
| `fixture.newtype_tuple_prune.support_chain.001` | covered | A checked-in support fixture slices through tuple newtype return values and receiver calls on free-function results, retaining nested newtype render methods while pruning dead tuple-newtype siblings |
| `fixture.lazy_parser_prune.support_chain.001` | covered | A checked-in support fixture slices through `LazyLock<T>` static initialization, exposing inner static type arguments so retained static receiver methods keep the concrete parser/token closure |
| `fixture.global_mutex_prune.support_chain.001` | covered | A checked-in support fixture slices through a global `Mutex<Vec<T>>` registry, retaining only the live entry constructor/render closure while pruning dead registry modules and methods |
| `fixture.protocol_projection_prune.support_chain.001` | covered | A checked-in support fixture slices through an internal protocol enum projection with struct-variant payload methods, retaining selected match payload dependencies while pruning dead protocol APIs |
| `fixture.iterator_fold_prune.support_chain.001` | covered | A checked-in support fixture slices through iterator `fold` closure item typing, retaining item methods used only inside the fold closure while pruning dead fold support code |
| `fixture.iterator_filter_map_prune.support_chain.001` | covered | A checked-in support fixture slices through iterator `filter_map` closure item typing, retaining payload methods used only inside filter-map closures while pruning dead filter APIs |
| `fixture.iterator_any_prune.support_chain.001` | covered | A checked-in support fixture slices through iterator predicate closures such as `any`, retaining predicate/render methods on the item type while pruning dead predicate support |
| `fixture.result_unwrap_or_else_prune.support_chain.001` | covered | A checked-in support fixture slices through `Result::unwrap_or_else`, retaining error recovery methods and the returned OK-type method chain while pruning dead error methods |
| `fixture.retain_sort_prune.support_chain.001` | covered | A checked-in support fixture slices through mutable collection closures such as `retain` and `sort_by_key`, retaining only live predicate/key/render methods |
| `fixture.iterator_for_each_prune.support_chain.001` | covered | A checked-in support fixture slices through iterator `for_each` closures, retaining item render methods used only for side effects while pruning dead event APIs |
| `fixture.iterator_flat_map_prune.support_chain.001` | covered | A checked-in support fixture slices through iterator `flat_map` closures, retaining expansion methods on the item type while pruning dead flat-map support |
| `fixture.iterator_map_while_prune.support_chain.001` | covered | A checked-in support fixture slices through iterator `map_while` closures, retaining item methods returning optional mapped values while pruning dead while-step support |
| `fixture.iterator_try_for_each_prune.support_chain.001` | covered | A checked-in support fixture slices through iterator `try_for_each` closures, retaining item fallible methods plus the closure-returned error render path |
| `fixture.iterator_take_while_prune.support_chain.001` | covered | A checked-in support fixture slices through iterator `take_while` closures, retaining live prefix predicates and downstream render methods while pruning dead take APIs |
| `fixture.iterator_skip_while_prune.support_chain.001` | covered | A checked-in support fixture slices through iterator `skip_while` closures, retaining live skip predicates and downstream render methods while pruning dead skip APIs |
| `fixture.iterator_skip_map_prune.support_chain.001` | covered | A checked-in support fixture slices through pass-through iterator `skip(...).map(...)`, retaining only the downstream payload render method |
| `fixture.iterator_take_map_prune.support_chain.001` | covered | A checked-in support fixture slices through pass-through iterator `take(...).map(...)`, retaining only the downstream payload render method |
| `fixture.iterator_step_by_map_prune.support_chain.001` | covered | A checked-in support fixture slices through pass-through iterator `step_by(...).map(...)`, retaining only the downstream payload render method |
| `fixture.iterator_by_ref_take_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `by_ref().take(...).map(...)`, retaining item methods through borrowed iterator chains without dead support |
| `fixture.iterator_filter_predicate_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `filter(...).map(...)`, retaining both live predicate and downstream render methods while pruning dead mutation methods |
| `fixture.iterator_inspect_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `inspect(...).map(...)`, retaining side-effect payload methods and downstream render methods only |
| `fixture.iterator_partition_prune.support_chain.001` | covered | A checked-in support fixture slices through iterator `partition` closure item typing, retaining only the live partition predicate and pruning dead partition support |
| `fixture.iterator_try_fold_prune.support_chain.001` | covered | A checked-in support fixture slices through iterator `try_fold` accumulator/item closures, retaining item fallible methods, accumulator render methods, and closure error recovery |
| `fixture.iterator_cloned_prune.support_chain.001` | covered | A checked-in support fixture slices through pass-through iterator `cloned` adapters, retaining cloned item render methods without keeping dead cloned support |
| `fixture.iterator_copied_map_prune.support_chain.001` | covered | A checked-in support fixture slices through pass-through iterator `copied().map(...)`, retaining copy payload render methods without dead support |
| `fixture.slice_iter_copied_map_prune.support_chain.001` | covered | A checked-in support fixture slices through slice `iter().copied().map(...)`, retaining only copied payload render methods |
| `fixture.result_iter_copied_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Result<T, E>::iter().copied().map(...)`, binding copied OK payloads without retaining unused error methods |
| `fixture.option_iter_cloned_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option<T>::iter().cloned().map(...)`, retaining cloned payload methods without mutable/dead siblings |
| `fixture.vec_into_iter_take_map_prune.support_chain.001` | covered | A checked-in support fixture slices through owned `Vec<T>::into_iter().take(...).map(...)`, retaining moved payload methods only |
| `fixture.array_into_iter_skip_map_prune.support_chain.001` | covered | A checked-in support fixture slices through array `into_iter().skip(...).map(...)`, retaining moved payload methods only |
| `fixture.iterator_chain_prune.support_chain.001` | covered | A checked-in support fixture slices through iterator `chain` adapters, retaining chained item render methods while pruning dead chain support |
| `fixture.iterator_enumerate_prune.support_chain.001` | covered | A checked-in support fixture slices through iterator `enumerate` tuple closure patterns, binding the item half precisely enough to retain only the live indexed render method |
| `fixture.iterator_zip_prune.support_chain.001` | covered | A checked-in support fixture slices through iterator `zip` tuple closure patterns, binding both zipped item halves and pruning dead zip support |
| `fixture.iterator_reduce_prune.support_chain.001` | covered | A checked-in support fixture slices through iterator `reduce` two-item closures and the returned optional item render path while pruning dead reduce support |
| `fixture.iterator_sort_by_prune.support_chain.001` | covered | A checked-in support fixture slices through mutable collection `sort_by` two-item closures, retaining only the live comparator and render methods |
| `fixture.iterator_dedup_by_prune.support_chain.001` | covered | A checked-in support fixture slices through mutable collection `dedup_by` two-item closures, retaining only the live equivalence and render methods |
| `fixture.iterator_scan_prune.support_chain.001` | covered | A checked-in support fixture slices through iterator `scan` state/item closures, retaining the live state transition and item render path |
| `fixture.iterator_tuple_map_prune.support_chain.001` | covered | A checked-in support fixture slices through tuple payload patterns in iterator `map` closures while pruning dead tuple support methods |
| `fixture.iterator_tuple_filter_prune.support_chain.001` | covered | A checked-in support fixture slices through tuple payload patterns across `filter` and downstream `map` closures while pruning dead tuple filter support |
| `fixture.iterator_tuple_for_each_prune.support_chain.001` | covered | A checked-in support fixture slices through tuple payload patterns in side-effect `for_each` closures while pruning dead side-effect support |
| `fixture.iterator_entry_map_prune.support_chain.001` | covered | A checked-in support fixture slices through map-entry-shaped tuple payloads from `BTreeMap::iter` while pruning dead entry support |
| `fixture.iterator_tuple_find_map_prune.support_chain.001` | covered | A checked-in support fixture slices through tuple payload patterns in `find_map` closures while pruning dead optional lookup support |
| `fixture.iterator_tuple_partition_prune.support_chain.001` | covered | A checked-in support fixture slices through tuple payload patterns in `partition` plus downstream map closures while pruning dead partition support |
| `fixture.iterator_tuple_inspect_prune.support_chain.001` | covered | A checked-in support fixture slices through tuple payload patterns in `inspect` side-effect closures while pruning dead inspect support |
| `fixture.iterator_tuple_sort_by_key_prune.support_chain.001` | covered | A checked-in support fixture slices through tuple payload patterns in mutable `sort_by_key` closures while pruning dead sort-key support |
| `fixture.iterator_nested_tuple_map_prune.support_chain.001` | covered | A checked-in support fixture slices through nested tuple payload patterns while pruning dead nested tuple support |
| `fixture.iterator_struct_map_prune.support_chain.001` | covered | A checked-in support fixture slices through named-struct payload patterns in iterator `map` closures while pruning dead struct support |
| `fixture.iterator_struct_filter_prune.support_chain.001` | covered | A checked-in support fixture slices through named-struct payload patterns across `filter` and downstream `map` closures while pruning dead struct filter support |
| `fixture.iterator_struct_inspect_prune.support_chain.001` | covered | A checked-in support fixture slices through named-struct payload patterns in `inspect` side-effect closures while pruning dead inspect support |
| `fixture.iterator_tuple_struct_map_prune.support_chain.001` | covered | A checked-in support fixture slices through tuple-struct payload patterns while pruning dead tuple-struct support |
| `fixture.iterator_nested_struct_tuple_prune.support_chain.001` | covered | A checked-in support fixture slices through nested tuple plus struct payload patterns while pruning dead nested struct support |
| `fixture.iterator_enum_struct_filter_map_prune.support_chain.001` | covered | A checked-in support fixture slices through enum struct-variant payloads inside iterator `filter_map` matches while pruning dead enum support |
| `fixture.iterator_enum_tuple_find_map_prune.support_chain.001` | covered | A checked-in support fixture slices through enum tuple-variant payloads inside iterator `find_map` matches while pruning dead tuple enum support |
| `fixture.iterator_enum_if_let_for_each_prune.support_chain.001` | covered | A checked-in support fixture slices through enum struct-variant payloads inside `if let` side-effect closures while pruning dead enum support |
| `fixture.iterator_enum_match_map_prune.support_chain.001` | covered | A checked-in support fixture slices through enum tuple-variant payloads inside iterator `map` match closures while pruning dead match support |
| `fixture.iterator_enum_flat_map_prune.support_chain.001` | covered | A checked-in support fixture slices through enum struct-variant payloads inside iterator `flat_map` matches while pruning dead flat-map enum support |
| `fixture.option_and_then_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option::and_then` payload closures while pruning dead optional support |
| `fixture.option_is_some_and_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option::is_some_and` predicate payload closures while pruning dead predicate support |
| `fixture.option_inspect_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option::inspect` side-effect closures plus downstream map payloads while pruning dead inspect support |
| `fixture.result_inspect_err_prune.support_chain.001` | covered | A checked-in support fixture slices through `Result::inspect_err` error-payload closures plus OK mapping while pruning dead result support |
| `fixture.result_or_else_prune.support_chain.001` | covered | A checked-in support fixture slices through `Result::or_else` recovery closures plus recovered payload mapping while pruning dead fallback support |
| `fixture.option_filter_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option::filter` predicate payload closures plus downstream map payloads while pruning dead filter support |
| `fixture.option_ok_or_else_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option::ok_or_else` error constructors plus result mapping while pruning dead option-to-result support |
| `fixture.option_as_ref_map_prune.support_chain.001` | covered | A checked-in support fixture slices through borrowed `Option::as_ref().map(...)` payload closures while pruning dead borrowed-option support |
| `fixture.result_and_then_prune.support_chain.001` | covered | A checked-in support fixture slices through `Result::and_then` payload closures and shared error fallbacks while pruning dead result chaining support |
| `fixture.result_inspect_prune.support_chain.001` | covered | A checked-in support fixture slices through OK-side `Result::inspect` side-effect closures plus downstream map payloads while pruning dead inspect support |
| `fixture.option_if_let_prune.support_chain.001` | covered | A checked-in support fixture slices through `if let Some(...)` option payload bindings while pruning dead option support |
| `fixture.result_match_prune.support_chain.001` | covered | A checked-in support fixture slices through `match Result` OK/error payload bindings while pruning dead result-match support |
| `fixture.while_let_payload_prune.support_chain.001` | covered | A checked-in support fixture slices through `while let Some(...)` iterator payload bindings while pruning dead loop support |
| `fixture.for_loop_payload_prune.support_chain.001` | covered | A checked-in support fixture slices through `for` loop iterable payload bindings while pruning dead loop support |
| `fixture.matches_guard_prune.support_chain.001` | covered | A checked-in support fixture slices through `matches!` guard payload bindings and downstream map payloads while pruning dead guard support |
| `fixture.option_let_else_prune.support_chain.001` | covered | A checked-in support fixture slices through `let Some(...) else` option payload bindings while pruning dead option support |
| `fixture.result_let_else_prune.support_chain.001` | covered | A checked-in support fixture slices through `let Ok(...) else` result payload bindings while pruning dead result support |
| `fixture.nested_option_result_match_prune.support_chain.001` | covered | A checked-in support fixture slices through nested `Option<Result<...>>` match payload bindings while pruning dead nested support |
| `fixture.nested_option_result_if_let_prune.support_chain.001` | covered | A checked-in support fixture slices through nested `if let Some(Ok(...))` and `Some(Err(...))` payload bindings while pruning dead nested support |
| `fixture.matches_result_guard_prune.support_chain.001` | covered | A checked-in support fixture slices through `matches!` result guard payload bindings while pruning dead result guard support |
| `fixture.result_option_match_prune.support_chain.001` | covered | A checked-in support fixture slices through nested `Result<Option<...>, ...>` match payload bindings while pruning dead nested support |
| `fixture.result_option_if_let_prune.support_chain.001` | covered | A checked-in support fixture slices through nested `if let Ok(Some(...))` payload bindings while pruning dead nested support |
| `fixture.option_struct_pattern_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option<Struct>` named-field destructuring while pruning dead struct support |
| `fixture.option_tuple_pattern_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option<(T, U)>` tuple destructuring while pruning dead tuple support |
| `fixture.option_tuple_struct_pattern_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option<TupleStruct>` tuple-struct destructuring while pruning dead tuple-struct support |
| `fixture.option_enum_named_pattern_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option<Enum::Variant { ... }>` named-variant destructuring while pruning dead enum support |
| `fixture.option_map_or_else_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option::map_or_else` payload and fallback closures while pruning dead fallback support |
| `fixture.result_map_or_else_prune.support_chain.001` | covered | A checked-in support fixture slices through `Result::map_or_else` OK/error closures while pruning dead result fallback support |
| `fixture.option_unwrap_or_else_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option::unwrap_or_else` fallback constructors plus downstream receiver methods while pruning dead unwrap support |
| `fixture.option_or_else_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option::or_else` fallback constructors plus downstream map payloads while pruning dead option fallback support |
| `fixture.result_is_ok_and_prune.support_chain.001` | covered | A checked-in support fixture slices through `Result::is_ok_and` OK payload predicates while pruning dead OK-check support |
| `fixture.result_is_err_and_prune.support_chain.001` | covered | A checked-in support fixture slices through `Result::is_err_and` error payload predicates while pruning dead error-check support |
| `fixture.option_map_prune.support_chain.001` | covered | A checked-in support fixture slices through owned `Option::map` payload closures while pruning dead optional support methods |
| `fixture.result_map_prune.support_chain.001` | covered | A checked-in support fixture slices through OK-side `Result::map` closures plus error fallback closures while pruning dead result support |
| `fixture.option_map_or_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option::map_or` default-value expressions plus payload closures while pruning dead fallback support |
| `fixture.result_map_or_prune.support_chain.001` | covered | A checked-in support fixture slices through `Result::map_or` default-value expressions plus OK payload closures while pruning dead result fallback support |
| `fixture.result_ok_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Result::ok().map(...)` OK-payload recovery while pruning dead result support |
| `fixture.result_err_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Result::err().map(...)` error-payload recovery while pruning dead result support |
| `fixture.option_zip_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option::zip(...).map(|(left, right)| ...)` tuple payload closures while pruning dead zip support |
| `fixture.iterator_all_prune.support_chain.001` | covered | A checked-in support fixture slices through iterator `all` predicate closures while pruning dead predicate support |
| `fixture.iterator_find_prune.support_chain.001` | covered | A checked-in support fixture slices through iterator `find` predicate closures plus returned option mapping while pruning dead find support |
| `fixture.iterator_max_by_prune.support_chain.001` | covered | A checked-in support fixture slices through iterator `max_by` two-item comparators plus returned option mapping while pruning dead comparator support |
| `fixture.iterator_min_by_prune.support_chain.001` | covered | A checked-in support fixture slices through iterator `min_by` two-item comparators plus returned option mapping while pruning dead comparator support |
| `fixture.iterator_max_by_key_prune.support_chain.001` | covered | A checked-in support fixture slices through iterator `max_by_key` key closures plus returned option mapping while pruning dead key support |
| `fixture.iterator_min_by_key_prune.support_chain.001` | covered | A checked-in support fixture slices through iterator `min_by_key` key closures plus returned option mapping while pruning dead key support |
| `fixture.iterator_rposition_prune.support_chain.001` | covered | A checked-in support fixture slices through iterator `rposition` predicate closures while pruning dead reverse-position support |
| `fixture.option_xor_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option::xor(...).map(...)` payload closures while pruning dead xor support |
| `fixture.option_flatten_prune.support_chain.001` | covered | A checked-in support fixture slices through nested `Option<Option<T>>::flatten().map(...)` payload closures while pruning dead nested-option support |
| `fixture.bool_then_prune.support_chain.001` | covered | A checked-in support fixture slices through `bool::then(|| T).map(...)` payload closures while pruning dead boolean-gated support |
| `fixture.bool_then_some_prune.support_chain.001` | covered | A checked-in support fixture slices through `bool::then_some(T).map(...)` payload closures while pruning dead boolean-gated support |
| `fixture.option_expect_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option::expect(...).method()` receiver typing while pruning dead option-expect support |
| `fixture.result_expect_prune.support_chain.001` | covered | A checked-in support fixture slices through `Result::expect(...).method()` OK receiver typing while pruning dead result-expect support |
| `fixture.option_unwrap_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option::unwrap().method()` receiver typing while pruning dead option-unwrap support |
| `fixture.option_unwrap_or_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option::unwrap_or(T).method()` fallback receiver typing while pruning dead unwrap-or support |
| `fixture.result_unwrap_prune.support_chain.001` | covered | A checked-in support fixture slices through `Result::unwrap().method()` OK receiver typing while pruning dead result-unwrap support |
| `fixture.result_unwrap_or_prune.support_chain.001` | covered | A checked-in support fixture slices through `Result::unwrap_or(T).method()` fallback receiver typing while pruning dead result unwrap-or support |
| `fixture.iterator_last_prune.support_chain.001` | covered | A checked-in support fixture slices through iterator `last().map(...)` terminal payload typing while pruning dead last support |
| `fixture.iterator_nth_prune.support_chain.001` | covered | A checked-in support fixture slices through iterator `nth(...).map(...)` terminal payload typing while pruning dead nth support |
| `fixture.iterator_rev_last_prune.support_chain.001` | covered | A checked-in support fixture slices through `rev().last().map(...)` payload pass-through while pruning dead reverse-last support |
| `fixture.iterator_peekable_nth_prune.support_chain.001` | covered | A checked-in support fixture slices through `peekable().nth(...).map(...)` payload pass-through while pruning dead peekable support |
| `fixture.iterator_fuse_last_prune.support_chain.001` | covered | A checked-in support fixture slices through `fuse().last().map(...)` payload pass-through while pruning dead fuse support |
| `fixture.iterator_cycle_nth_prune.support_chain.001` | covered | A checked-in support fixture slices through `cycle().nth(...).map(...)` payload pass-through while pruning dead cycle support |
| `fixture.vec_first_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::first().map(...)` payload typing while pruning dead vector support |
| `fixture.vec_last_prune.support_chain.001` | covered | A checked-in support fixture slices through collection `last().map(...)` payload typing while pruning dead vector-last support |
| `fixture.vec_get_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::get(...).map(...)` payload typing while pruning dead indexed support |
| `fixture.vec_pop_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::pop().map(...)` payload typing while pruning dead pop support |
| `fixture.vec_remove_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::remove(...).method()` receiver typing while pruning dead remove support |
| `fixture.vec_swap_remove_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::swap_remove(...).method()` receiver typing while pruning dead swap-remove support |
| `fixture.vecdeque_front_prune.support_chain.001` | covered | A checked-in support fixture slices through `VecDeque<T>::front().map(...)` payload typing while pruning dead deque front support |
| `fixture.vecdeque_back_prune.support_chain.001` | covered | A checked-in support fixture slices through `VecDeque<T>::back().map(...)` payload typing while pruning dead deque back support |
| `fixture.vecdeque_pop_front_prune.support_chain.001` | covered | A checked-in support fixture slices through `VecDeque<T>::pop_front().map(...)` payload typing while pruning dead deque pop-front support |
| `fixture.vecdeque_pop_back_prune.support_chain.001` | covered | A checked-in support fixture slices through `VecDeque<T>::pop_back().map(...)` payload typing while pruning dead deque pop-back support |
| `fixture.hashmap_get_prune.support_chain.001` | covered | A checked-in support fixture slices through `HashMap<K, V>::get(...).map(...)` value payload typing with a local key type while pruning dead hash-map support |
| `fixture.hashmap_get_mut_prune.support_chain.001` | covered | A checked-in support fixture slices through `HashMap<K, V>::get_mut(...).map(...)` mutable value payload typing while pruning dead hash-map support |
| `fixture.hashmap_remove_prune.support_chain.001` | covered | A checked-in support fixture slices through `HashMap<K, V>::remove(...).map(...)` owned value payload typing while pruning dead hash-map support |
| `fixture.hashmap_values_next_prune.support_chain.001` | covered | A checked-in support fixture slices through `HashMap<K, V>::values().next().map(...)` value iterator typing while pruning dead hash-map support |
| `fixture.hashmap_into_values_next_prune.support_chain.001` | covered | A checked-in support fixture slices through `HashMap<K, V>::into_values().next().map(...)` owned value iterator typing while pruning dead hash-map support |
| `fixture.hashmap_entry_or_insert_prune.support_chain.001` | covered | A checked-in support fixture slices through `HashMap<K, V>::entry(...).or_insert(...).method()` entry value receiver typing while pruning dead hash-map support |
| `fixture.hashmap_entry_or_insert_with_prune.support_chain.001` | covered | A checked-in support fixture slices through `HashMap<K, V>::entry(...).or_insert_with(...).method()` entry value receiver typing while pruning dead hash-map support |
| `fixture.hashmap_entry_and_modify_prune.support_chain.001` | covered | A checked-in support fixture slices through `HashMap<K, V>::entry(...).and_modify(...).or_insert(...).method()` entry value and closure payload typing while pruning dead hash-map support |
| `fixture.hashmap_entry_or_default_prune.support_chain.001` | covered | A checked-in support fixture slices through `HashMap<K, V>::entry(...).or_default().method()` entry value receiver typing plus `Default` impl retention while pruning dead hash-map support |
| `fixture.hashmap_entry_or_insert_with_key_prune.support_chain.001` | covered | A checked-in support fixture slices through `HashMap<K, V>::entry(...).or_insert_with_key(...).method()` entry value receiver typing while pruning dead hash-map support |
| `fixture.btreemap_get_prune.support_chain.001` | covered | A checked-in support fixture slices through `BTreeMap<K, V>::get(...).map(...)` value payload typing with a local key type while pruning dead tree-map support |
| `fixture.btreemap_get_mut_prune.support_chain.001` | covered | A checked-in support fixture slices through `BTreeMap<K, V>::get_mut(...).map(...)` mutable value payload typing while pruning dead tree-map support |
| `fixture.btreemap_remove_prune.support_chain.001` | covered | A checked-in support fixture slices through `BTreeMap<K, V>::remove(...).map(...)` owned value payload typing while pruning dead tree-map support |
| `fixture.btreemap_values_last_prune.support_chain.001` | covered | A checked-in support fixture slices through `BTreeMap<K, V>::values().last().map(...)` value iterator typing while pruning dead tree-map support |
| `fixture.btreemap_into_values_next_prune.support_chain.001` | covered | A checked-in support fixture slices through `BTreeMap<K, V>::into_values().next().map(...)` owned value iterator typing while pruning dead tree-map support |
| `fixture.btreemap_entry_or_insert_prune.support_chain.001` | covered | A checked-in support fixture slices through `BTreeMap<K, V>::entry(...).or_insert(...).method()` entry value receiver typing while pruning dead tree-map support |
| `fixture.btreemap_entry_or_insert_with_prune.support_chain.001` | covered | A checked-in support fixture slices through `BTreeMap<K, V>::entry(...).or_insert_with(...).method()` entry value receiver typing while pruning dead tree-map support |
| `fixture.btreemap_entry_and_modify_prune.support_chain.001` | covered | A checked-in support fixture slices through `BTreeMap<K, V>::entry(...).and_modify(...).or_insert(...).method()` entry value and closure payload typing while pruning dead tree-map support |
| `fixture.btreemap_entry_or_default_prune.support_chain.001` | covered | A checked-in support fixture slices through `BTreeMap<K, V>::entry(...).or_default().method()` entry value receiver typing plus `Default` impl retention while pruning dead tree-map support |
| `fixture.btreemap_entry_or_insert_with_key_prune.support_chain.001` | covered | A checked-in support fixture slices through `BTreeMap<K, V>::entry(...).or_insert_with_key(...).method()` entry value receiver typing while pruning dead tree-map support |
| `fixture.hashmap_iter_pairs_prune.support_chain.001` | covered | A checked-in support fixture slices through `HashMap<K, V>::iter().map(|(K, V)| ...)` pair payload typing while pruning dead hash-map support |
| `fixture.hashmap_iter_mut_pairs_prune.support_chain.001` | covered | A checked-in support fixture slices through `HashMap<K, V>::iter_mut().for_each(|(K, V)| ...)` mutable pair payload typing while pruning dead hash-map support |
| `fixture.hashmap_into_iter_pairs_prune.support_chain.001` | covered | A checked-in support fixture slices through `HashMap<K, V>::into_iter().map(|(K, V)| ...)` owned pair payload typing while pruning dead hash-map support |
| `fixture.hashmap_drain_pairs_prune.support_chain.001` | covered | A checked-in support fixture slices through `HashMap<K, V>::drain().map(|(K, V)| ...)` owned pair payload typing while pruning dead hash-map support |
| `fixture.hashmap_keys_find_prune.support_chain.001` | covered | A checked-in support fixture slices through `HashMap<K, V>::keys().find(...).map(...)` key payload typing while pruning dead hash-map value methods |
| `fixture.btreemap_iter_pairs_prune.support_chain.001` | covered | A checked-in support fixture slices through `BTreeMap<K, V>::iter().map(|(K, V)| ...)` pair payload typing while pruning dead tree-map support |
| `fixture.btreemap_iter_mut_pairs_prune.support_chain.001` | covered | A checked-in support fixture slices through `BTreeMap<K, V>::iter_mut().for_each(|(K, V)| ...)` mutable pair payload typing while pruning dead tree-map support |
| `fixture.btreemap_into_iter_pairs_prune.support_chain.001` | covered | A checked-in support fixture slices through `BTreeMap<K, V>::into_iter().map(|(K, V)| ...)` owned pair payload typing while pruning dead tree-map support |
| `fixture.btreemap_range_pairs_prune.support_chain.001` | covered | A checked-in support fixture slices through `BTreeMap<K, V>::range(...).map(|(K, V)| ...)` range pair payload typing while pruning dead tree-map support |
| `fixture.btreemap_range_mut_pairs_prune.support_chain.001` | covered | A checked-in support fixture slices through `BTreeMap<K, V>::range_mut(...).for_each(|(K, V)| ...)` mutable range pair payload typing while pruning dead tree-map support |
| `fixture.hashset_get_prune.support_chain.001` | covered | A checked-in support fixture slices through `HashSet<T>::get(...).map(...)` payload typing while pruning dead set support |
| `fixture.hashset_take_prune.support_chain.001` | covered | A checked-in support fixture slices through `HashSet<T>::take(...).map(...)` owned payload typing while pruning dead set support |
| `fixture.hashset_replace_prune.support_chain.001` | covered | A checked-in support fixture slices through `HashSet<T>::replace(...).map(...)` owned payload typing while pruning dead set support |
| `fixture.hashset_iter_find_prune.support_chain.001` | covered | A checked-in support fixture slices through `HashSet<T>::iter().find(...).map(...)` payload typing while pruning dead set support |
| `fixture.hashset_drain_prune.support_chain.001` | covered | A checked-in support fixture slices through `HashSet<T>::drain().map(...)` owned payload typing while pruning dead set support |
| `fixture.hashset_into_iter_prune.support_chain.001` | covered | A checked-in support fixture slices through `HashSet<T>::into_iter().map(...)` owned payload typing while pruning dead set support |
| `fixture.btreeset_get_prune.support_chain.001` | covered | A checked-in support fixture slices through `BTreeSet<T>::get(...).map(...)` payload typing while pruning dead set support |
| `fixture.btreeset_take_prune.support_chain.001` | covered | A checked-in support fixture slices through `BTreeSet<T>::take(...).map(...)` owned payload typing while pruning dead set support |
| `fixture.btreeset_replace_prune.support_chain.001` | covered | A checked-in support fixture slices through `BTreeSet<T>::replace(...).map(...)` owned payload typing while pruning dead set support |
| `fixture.btreeset_range_find_prune.support_chain.001` | covered | A checked-in support fixture slices through `BTreeSet<T>::range(...).find(...).map(...)` payload typing while pruning dead set support |
| `fixture.btreeset_pop_first_prune.support_chain.001` | covered | A checked-in support fixture slices through `BTreeSet<T>::pop_first().map(...)` owned payload typing while pruning dead set support |
| `fixture.btreeset_pop_last_prune.support_chain.001` | covered | A checked-in support fixture slices through `BTreeSet<T>::pop_last().map(...)` owned payload typing while pruning dead set support |
| `fixture.binaryheap_peek_prune.support_chain.001` | covered | A checked-in support fixture slices through `BinaryHeap<T>::peek().map(...)` borrowed payload typing while pruning dead heap support |
| `fixture.binaryheap_pop_prune.support_chain.001` | covered | A checked-in support fixture slices through `BinaryHeap<T>::pop().map(...)` owned payload typing while pruning dead heap support |
| `fixture.binaryheap_iter_find_prune.support_chain.001` | covered | A checked-in support fixture slices through `BinaryHeap<T>::iter().find(...).map(...)` payload typing while pruning dead heap support |
| `fixture.binaryheap_drain_prune.support_chain.001` | covered | A checked-in support fixture slices through `BinaryHeap<T>::drain().map(...)` owned payload typing while pruning dead heap support |
| `fixture.binaryheap_into_iter_prune.support_chain.001` | covered | A checked-in support fixture slices through `BinaryHeap<T>::into_iter().map(...)` owned payload typing while pruning dead heap support |
| `fixture.binaryheap_into_sorted_vec_prune.support_chain.001` | covered | A checked-in support fixture slices through `BinaryHeap<T>::into_sorted_vec().into_iter().map(...)` owned payload typing while pruning dead heap support |
| `fixture.linkedlist_front_prune.support_chain.001` | covered | A checked-in support fixture slices through `LinkedList<T>::front().map(...)` borrowed payload typing while pruning dead linked-list support |
| `fixture.linkedlist_back_prune.support_chain.001` | covered | A checked-in support fixture slices through `LinkedList<T>::back().map(...)` borrowed payload typing while pruning dead linked-list support |
| `fixture.linkedlist_pop_front_prune.support_chain.001` | covered | A checked-in support fixture slices through `LinkedList<T>::pop_front().map(...)` owned payload typing while pruning dead linked-list support |
| `fixture.linkedlist_pop_back_prune.support_chain.001` | covered | A checked-in support fixture slices through `LinkedList<T>::pop_back().map(...)` owned payload typing while pruning dead linked-list support |
| `fixture.linkedlist_iter_find_prune.support_chain.001` | covered | A checked-in support fixture slices through `LinkedList<T>::iter().find(...).map(...)` payload typing while pruning dead linked-list support |
| `fixture.linkedlist_into_iter_prune.support_chain.001` | covered | A checked-in support fixture slices through `LinkedList<T>::into_iter().map(...)` owned payload typing while pruning dead linked-list support |
| `fixture.vec_iter_find_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::iter().find(...).map(...)` payload typing while pruning dead vector support |
| `fixture.vec_drain_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::drain(..).map(...)` owned payload typing while pruning dead vector support |
| `fixture.vec_into_iter_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::into_iter().map(...)` owned payload typing while pruning dead vector support |
| `fixture.vecdeque_iter_find_prune.support_chain.001` | covered | A checked-in support fixture slices through `VecDeque<T>::iter().find(...).map(...)` payload typing while pruning dead deque support |
| `fixture.vecdeque_drain_prune.support_chain.001` | covered | A checked-in support fixture slices through `VecDeque<T>::drain(..).map(...)` owned payload typing while pruning dead deque support |
| `fixture.vecdeque_into_iter_prune.support_chain.001` | covered | A checked-in support fixture slices through `VecDeque<T>::into_iter().map(...)` owned payload typing while pruning dead deque support |
| `fixture.slice_chunks_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::chunks(...).map(...)` nested slice payload typing while pruning dead slice support |
| `fixture.slice_chunks_exact_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::chunks_exact(...).map(...)` nested slice payload typing while pruning dead slice support |
| `fixture.slice_rchunks_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::rchunks(...).map(...)` reverse nested slice payload typing while pruning dead slice support |
| `fixture.slice_rchunks_exact_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::rchunks_exact(...).map(...)` reverse exact chunk payload typing while pruning dead slice support |
| `fixture.slice_windows_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::windows(...).map(...)` overlapping slice payload typing while pruning dead slice support |
| `fixture.slice_split_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::split(predicate).map(...)`, retaining the split predicate and nested group payload methods while pruning dead slice support |
| `fixture.slice_split_inclusive_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::split_inclusive(predicate).map(...)`, retaining the inclusive split predicate and nested group payload methods while pruning dead slice support |
| `fixture.slice_rsplit_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::rsplit(predicate).map(...)`, retaining the reverse split predicate and nested group payload methods while pruning dead slice support |
| `fixture.slice_splitn_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::splitn(count, predicate).map(...)`, retaining the second-argument predicate closure and nested group payload methods while pruning dead slice support |
| `fixture.slice_rsplitn_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::rsplitn(count, predicate).map(...)`, retaining the second-argument predicate closure and nested group payload methods while pruning dead slice support |
| `fixture.slice_iter_mut_prune.support_chain.001` | covered | A checked-in support fixture slices through mutable slice iteration via `iter_mut().map(...)`, retaining only live mutating receiver methods |
| `fixture.array_iter_prune.support_chain.001` | covered | A checked-in support fixture slices through fixed-array `iter().find(...).map(...)` payload typing while pruning dead array support |
| `fixture.vec_retain_mut_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::retain_mut(...)`, retaining only live mutating predicate methods and downstream render methods |
| `fixture.vec_dedup_by_key_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::dedup_by_key(...)`, retaining only live key extraction and render methods |
| `fixture.vec_sort_by_cached_key_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::sort_by_cached_key(...)`, retaining only live cached-key and render methods |
| `fixture.vec_sort_unstable_by_key_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::sort_unstable_by_key(...)`, retaining only live key extraction and render methods |
| `fixture.slice_binary_search_by_prune.support_chain.001` | covered | A checked-in support fixture slices through `binary_search_by(...)`, retaining only live comparator-key, sort-key, and render methods |
| `fixture.slice_binary_search_by_key_prune.support_chain.001` | covered | A checked-in support fixture slices through `binary_search_by_key(key, closure)`, retaining only the second-argument key closure and render methods |
| `fixture.slice_partition_point_prune.support_chain.001` | covered | A checked-in support fixture slices through `partition_point(...)`, retaining only live predicate key and render methods |
| `fixture.slice_sort_unstable_by_prune.support_chain.001` | covered | A checked-in support fixture slices through two-argument `sort_unstable_by(...)` comparator closures while pruning unused key and mutation helpers |
| `fixture.slice_select_nth_unstable_by_prune.support_chain.001` | covered | A checked-in support fixture slices through `select_nth_unstable_by(index, comparator)`, retaining only the second-argument comparator and live render methods |
| `fixture.slice_select_nth_unstable_by_key_prune.support_chain.001` | covered | A checked-in support fixture slices through `select_nth_unstable_by_key(index, key)`, retaining only the second-argument key closure and live render methods |
| `fixture.hashmap_retain_prune.support_chain.001` | covered | A checked-in support fixture slices through `HashMap<K, V>::retain(key, value)`, retaining key and value methods separately while pruning dead support methods |
| `fixture.btreemap_retain_prune.support_chain.001` | covered | A checked-in support fixture slices through `BTreeMap<K, V>::retain(key, value)`, retaining key and value methods separately while pruning dead support methods |
| `fixture.hashset_retain_prune.support_chain.001` | covered | A checked-in support fixture slices through `HashSet<T>::retain(...)`, retaining only live predicate and render methods |
| `fixture.btreeset_retain_prune.support_chain.001` | covered | A checked-in support fixture slices through `BTreeSet<T>::retain(...)`, retaining only live predicate and render methods |
| `fixture.binaryheap_retain_prune.support_chain.001` | covered | A checked-in support fixture slices through `BinaryHeap<T>::retain(...)`, retaining only live predicate and render methods |
| `fixture.vecdeque_retain_prune.support_chain.001` | covered | A checked-in support fixture slices through `VecDeque<T>::retain(...)`, retaining only live predicate and render methods |
| `fixture.vecdeque_make_contiguous_sort_prune.support_chain.001` | covered | A checked-in support fixture slices through `VecDeque<T>::make_contiguous().sort_by_key(...)`, preserving slice-view payload typing without retaining dead sort helpers |
| `fixture.vec_as_slice_iter_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::as_slice().iter().find(...)`, preserving payload typing across slice views |
| `fixture.vec_as_mut_slice_iter_mut_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::as_mut_slice().iter_mut().map(...)`, preserving mutable payload typing across slice views |
| `fixture.vec_as_slice_chunks_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::as_slice().chunks(...).map(...)`, preserving nested slice payload typing across slice views |
| `fixture.collect_hashmap_prune.support_chain.001` | covered | A checked-in support fixture slices through `collect::<HashMap<K, V>>()`, retaining key/value constructors and value render methods while pruning dead map support methods |
| `fixture.collect_btreemap_prune.support_chain.001` | covered | A checked-in support fixture slices through `collect::<BTreeMap<K, V>>()`, retaining key/value constructors and value render methods while pruning dead tree-map support methods |
| `fixture.collect_hashset_prune.support_chain.001` | covered | A checked-in support fixture slices through `collect::<HashSet<T>>()`, retaining collection item payload methods from the turbofish target type while pruning dead set support methods |
| `fixture.collect_btreeset_prune.support_chain.001` | covered | A checked-in support fixture slices through `collect::<BTreeSet<T>>()`, retaining collection item payload methods from the turbofish target type while pruning dead set support methods |
| `fixture.collect_binaryheap_prune.support_chain.001` | covered | A checked-in support fixture slices through `collect::<BinaryHeap<T>>()`, retaining heap item payload methods from the turbofish target type while pruning dead heap support methods |
| `fixture.collect_vecdeque_prune.support_chain.001` | covered | A checked-in support fixture slices through `collect::<VecDeque<T>>()`, retaining deque item payload methods from the turbofish target type while pruning dead deque support methods |
| `fixture.collect_linkedlist_prune.support_chain.001` | covered | A checked-in support fixture slices through `collect::<LinkedList<T>>()`, retaining list item payload methods from the turbofish target type while pruning dead list support methods |
| `fixture.collect_vec_tuple_prune.support_chain.001` | covered | A checked-in support fixture slices through `collect::<Vec<(K, V)>>()`, retaining both tuple key and value payload methods while pruning dead tuple collection support |
| `fixture.collect_result_vec_prune.support_chain.001` | covered | A checked-in support fixture slices through `collect::<Result<Vec<T>, E>>()`, retaining OK item and error payload methods from the turbofish target type while pruning dead result support |
| `fixture.collect_option_vec_prune.support_chain.001` | covered | A checked-in support fixture slices through `collect::<Option<Vec<T>>>()`, retaining optional item payload methods from the turbofish target type while pruning dead option support |
| `fixture.collect_annotated_hashmap_prune.support_chain.001` | covered | A checked-in support fixture slices through `let collected: HashMap<K, V> = iter.collect()`, retaining key/value constructors and value render methods while pruning dead map support methods |
| `fixture.collect_annotated_btreemap_prune.support_chain.001` | covered | A checked-in support fixture slices through `let collected: BTreeMap<K, V> = iter.collect()`, retaining key/value constructors and value render methods while pruning dead tree-map support methods |
| `fixture.collect_annotated_hashset_prune.support_chain.001` | covered | A checked-in support fixture slices through `let collected: HashSet<T> = iter.collect()`, retaining item payload methods from the explicit local type while pruning dead set support methods |
| `fixture.collect_annotated_btreeset_prune.support_chain.001` | covered | A checked-in support fixture slices through `let collected: BTreeSet<T> = iter.collect()`, retaining item payload methods from the explicit local type while pruning dead set support methods |
| `fixture.collect_annotated_binaryheap_prune.support_chain.001` | covered | A checked-in support fixture slices through `let collected: BinaryHeap<T> = iter.collect()`, retaining item payload methods from the explicit local type while pruning dead heap support methods |
| `fixture.collect_annotated_vecdeque_prune.support_chain.001` | covered | A checked-in support fixture slices through `let collected: VecDeque<T> = iter.collect()`, retaining item payload methods from the explicit local type while pruning dead deque support methods |
| `fixture.collect_annotated_linkedlist_prune.support_chain.001` | covered | A checked-in support fixture slices through `let collected: LinkedList<T> = iter.collect()`, retaining item payload methods from the explicit local type while pruning dead list support methods |
| `fixture.collect_annotated_vec_tuple_prune.support_chain.001` | covered | A checked-in support fixture slices through `let collected: Vec<(K, V)> = iter.collect()`, retaining tuple key and value payload methods from the explicit local type while pruning dead tuple support |
| `fixture.collect_annotated_result_vec_prune.support_chain.001` | covered | A checked-in support fixture slices through `let collected: Result<Vec<T>, E> = iter.collect()`, retaining OK item and error payload methods from the explicit local type while pruning dead result support |
| `fixture.collect_annotated_option_vec_prune.support_chain.001` | covered | A checked-in support fixture slices through `let collected: Option<Vec<T>> = iter.collect()`, retaining optional item payload methods from the explicit local type while pruning dead option support |
| `fixture.option_as_deref_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option<Box<T>>::as_deref().map(...)`, retaining borrowed payload methods while pruning unused mutable/dead payload methods |
| `fixture.option_as_deref_mut_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option<Box<T>>::as_deref_mut().map(...)`, retaining mutable payload methods while pruning unused immutable/dead payload methods |
| `fixture.result_as_ref_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Result<T, E>::as_ref().map(...).unwrap_or_else(...)`, retaining borrowed OK/error methods while pruning dead siblings |
| `fixture.result_as_deref_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Result<Box<T>, E>::as_deref().map(...)`, retaining dereferenced OK payload methods and error fallback methods |
| `fixture.result_as_deref_mut_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Result<Box<T>, E>::as_deref_mut().map(...)`, retaining mutable OK payload methods and error fallback methods |
| `fixture.box_as_ref_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Box<T>::as_ref()` receiver calls, retaining only the referenced payload method |
| `fixture.arc_as_ref_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Arc<T>::as_ref()` receiver calls, retaining the required `Arc` import and only the referenced payload method |
| `fixture.rc_as_ref_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Rc<T>::as_ref()` receiver calls, retaining the required `Rc` import and only the referenced payload method |
| `fixture.vec_box_iter_as_ref_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<Box<T>>::iter().map(|x| x.as_ref().method())`, retaining boxed collection payload methods without dead sibling retention |
| `fixture.option_copied_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option<&T>::copied().map(...)`, retaining copied value payload methods while pruning unused mutable/dead methods |
| `fixture.refcell_borrow_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `RefCell<T>::borrow().method()`, retaining only the borrowed payload method and pruning dead/mutable siblings |
| `fixture.refcell_borrow_mut_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `RefCell<T>::borrow_mut().method()`, retaining only the mutable payload method and pruning dead/immutable siblings |
| `fixture.cell_get_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Cell<T>::get().method()` for copy payloads while pruning unused mutable/dead siblings |
| `fixture.once_lock_get_or_init_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `OnceLock<T>::get_or_init(...).method()`, retaining initializer payload construction and only the returned reference method |
| `fixture.mutex_lock_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Mutex<T>::lock()` guard autoderef, retaining only the immutable guard payload method |
| `fixture.mutex_lock_mut_map_prune.support_chain.001` | covered | A checked-in support fixture slices through mutable `Mutex<T>::lock()` guard autoderef, retaining only the mutable guard payload method |
| `fixture.rwlock_read_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `RwLock<T>::read()` guard autoderef, retaining only the immutable guard payload method |
| `fixture.rwlock_write_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `RwLock<T>::write()` guard autoderef, retaining only the mutable guard payload method |
| `fixture.option_refcell_borrow_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option<RefCell<T>>::as_ref().map(|x| x.borrow().method())`, retaining only the nested borrowed payload method |
| `fixture.arc_mutex_lock_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Arc<Mutex<T>>::lock()` guard autoderef, retaining only the live synchronized payload method and imports |
| `fixture.rc_refcell_borrow_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Rc<RefCell<T>>::borrow()` guard autoderef, retaining only the live interior-mutable payload method and imports |
| `fixture.cow_borrowed_as_ref_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Cow::Borrowed(...).as_ref().method()`, retaining only the borrowed payload method and required `Clone` surface |
| `fixture.cow_owned_into_owned_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Cow::Owned(...).into_owned().method()`, retaining only the owned payload method and pruning mutable/dead siblings |
| `fixture.cow_to_mut_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Cow<T>::to_mut().method()`, retaining only the mutable payload method while pruning immutable/dead siblings |
| `fixture.borrow_trait_map_prune.support_chain.001` | covered | A checked-in support fixture slices through a project-local `Borrow<T>` impl and `.borrow()` call, retaining the trait impl and only the borrowed payload method |
| `fixture.as_ref_trait_map_prune.support_chain.001` | covered | A checked-in support fixture slices through a project-local `AsRef<T>` impl and `.as_ref()` call, retaining the trait impl and only the borrowed payload method |
| `fixture.as_mut_trait_map_prune.support_chain.001` | covered | A checked-in support fixture slices through a project-local `AsMut<T>` impl and `.as_mut()` call, retaining the trait impl and only the mutable payload method |
| `fixture.deref_mut_trait_map_prune.support_chain.001` | covered | A checked-in support fixture slices through project-local `Deref`/`DerefMut` wrapper method dispatch, retaining mutable target methods without dead wrapper siblings |
| `fixture.pin_box_as_ref_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Pin<Box<T>>::as_ref().get_ref().method()`, retaining only the pinned borrowed payload method |
| `fixture.pin_box_as_mut_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Pin<Box<T>>::as_mut().get_mut().method()`, retaining only the pinned mutable payload method |
| `fixture.phantomdata_surface_map_prune.support_chain.001` | covered | A checked-in support fixture slices through a generic `PhantomData<T>` envelope surface, retaining the live marker/payload closure while pruning dead marker siblings |
| `fixture.weak_upgrade_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Arc::downgrade(...).upgrade().map(...)`, retaining only the upgraded payload method and pruning dead weak-pointer siblings |
| `fixture.rc_weak_upgrade_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Rc::downgrade(...).upgrade().map(...)`, retaining only the upgraded payload method and pruning dead single-thread weak-pointer siblings |
| `fixture.option_take_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option<T>::take().map(...)` from a retained slot type, keeping the live slot method and payload render method only |
| `fixture.option_get_or_insert_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option<T>::get_or_insert(...).method()`, retaining only the inserted payload receiver method and pruning dead sibling methods |
| `fixture.option_get_or_insert_with_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option<T>::get_or_insert_with(...).method()`, retaining closure-created payload construction and the live inserted receiver method |
| `fixture.option_get_or_insert_default_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option<T>::get_or_insert_default().method()`, retaining the required `Default` impl and only the live inserted receiver method |
| `fixture.option_insert_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option<T>::insert(...).method()`, retaining only the inserted payload method and pruning unused slot siblings |
| `fixture.mem_take_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `std::mem::take(&mut T).method()`, retaining the `Default`-backed moved payload method without dead mutation siblings |
| `fixture.mem_replace_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `std::mem::replace(&mut T, ...).method()`, retaining the replaced payload method without dead mutation siblings |
| `fixture.option_as_mut_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option<T>::as_mut().map(...)`, retaining only mutable payload methods and pruning unused immutable/dead siblings |
| `fixture.result_as_mut_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Result<T, E>::as_mut().map(...)`, retaining mutable OK payload methods plus live error fallback methods while pruning dead siblings |
| `fixture.option_take_if_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option<T>::take_if(...).map(...)`, retaining predicate and moved payload methods without unrelated slot helpers |
| `fixture.option_unwrap_or_default_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option<T>::unwrap_or_default().method()`, retaining the payload `Default` impl and pruning unused mutable/dead siblings |
| `fixture.result_unwrap_or_default_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Result<T, E>::unwrap_or_default().method()`, retaining the OK payload `Default` impl without retaining unused error methods |
| `fixture.option_ok_or_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option<T>::ok_or(E).map(...).unwrap_or_else(...)`, retaining both OK payload and live error fallback methods |
| `fixture.option_cloned_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option<&T>::cloned().map(...)`, retaining the cloned payload method without mutable/dead siblings |
| `fixture.result_cloned_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Result<&T, E>::cloned().map(...).unwrap_or_else(...)`, retaining cloned OK payload and live error fallback methods only |
| `fixture.result_copied_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Result<&T, E>::copied().map(...).unwrap_or_else(...)`, retaining copy OK payload and live error fallback methods only |
| `fixture.option_transpose_unwrap_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option<Result<T, E>>::transpose().unwrap_or_else(...).map(...)`, retaining payload/error methods reached through the transposed flow |
| `fixture.result_transpose_unwrap_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Result<Option<T>, E>::transpose().unwrap_or_else(...).map(...)`, retaining fallback construction plus live payload/error methods |
| `fixture.option_unzip_pair_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option<(A, B)>::unzip()` tuple binding, retaining only left/right methods used by the selected flow |
| `fixture.iterator_unzip_pair_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Iterator<Item = (A, B)>::unzip()` tuple binding, retaining only the live left/right payload methods |
| `fixture.mem_swap_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `std::mem::swap(&mut T, &mut T)` followed by receiver use, retaining only the swapped live payload method |
| `fixture.maybeuninit_assume_init_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `MaybeUninit<T>::assume_init().method()`, retaining the initialized payload method without dead wrapper helpers |
| `fixture.manuallydrop_into_inner_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `ManuallyDrop::into_inner(T).method()`, proving transparent associated return typing for wrapped payload methods |
| `fixture.nonnull_as_ref_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `NonNull<T>::as_ref().method()`, retaining only the pointed payload method and required pointer import |
| `fixture.box_pin_as_ref_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Box::pin(T).as_ref().get_ref().method()`, retaining pinned payload receiver methods without dead siblings |
| `fixture.arc_make_mut_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Arc::make_mut(&mut Arc<T>).method()`, retaining only clone-safe mutable payload methods and required `Arc` surface |
| `fixture.rc_make_mut_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Rc::make_mut(&mut Rc<T>).method()`, retaining only single-thread mutable payload methods and required `Rc` surface |
| `fixture.control_flow_continue_match_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `ControlFlow::Continue(T)` match payloads, retaining only the live continue payload method |
| `fixture.control_flow_break_match_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `ControlFlow::Break(E)` match payloads, retaining only the live break payload method |
| `fixture.option_iter_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option<T>::iter().map(...)`, retaining only immutable payload methods and pruning mutable/dead siblings |
| `fixture.option_iter_mut_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option<T>::iter_mut().map(...)`, retaining only mutable payload methods and pruning immutable/dead siblings |
| `fixture.result_iter_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Result<T, E>::iter().map(...)`, proving Result iterator payloads bind to the OK type, not the error type |
| `fixture.result_iter_mut_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Result<T, E>::iter_mut().map(...)`, retaining mutable OK payload methods plus live error fallback methods |
| `fixture.option_as_slice_iter_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option<T>::as_slice().iter().map(...)`, retaining only slice-view payload methods |
| `fixture.option_as_mut_slice_iter_mut_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option<T>::as_mut_slice().iter_mut().map(...)`, retaining only mutable slice-view payload methods |
| `fixture.option_replace_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Option<T>::replace(...).map(...)`, retaining only the returned old payload method |
| `fixture.btreemap_first_key_value_prune.support_chain.001` | covered | A checked-in support fixture slices through `BTreeMap::first_key_value()`, retaining only live key/value pair render methods |
| `fixture.btreemap_last_key_value_prune.support_chain.001` | covered | A checked-in support fixture slices through `BTreeMap::last_key_value()`, retaining only live key/value pair render methods |
| `fixture.btreemap_pop_first_pair_prune.support_chain.001` | covered | A checked-in support fixture slices through `BTreeMap::pop_first()`, retaining only moved key/value pair render methods |
| `fixture.btreemap_pop_last_pair_prune.support_chain.001` | covered | A checked-in support fixture slices through `BTreeMap::pop_last()`, retaining only moved key/value pair render methods |
| `fixture.array_from_fn_iter_prune.support_chain.001` | covered | A checked-in support fixture slices through `std::array::from_fn(...).iter().map(...)`, retaining array payload construction and only live payload methods |
| `fixture.vec_split_off_into_iter_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::split_off(...).into_iter().map(...)`, retaining only live payload methods |
| `fixture.vec_splice_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::splice(...).map(...)`, retaining only returned payload methods from the removed range |
| `fixture.vec_into_boxed_slice_iter_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::into_boxed_slice().iter().map(...)`, retaining boxed-slice payload methods |
| `fixture.vec_leak_iter_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::leak().iter().map(...)`, retaining leaked slice payload methods without dead mutable siblings |
| `fixture.vec_resize_with_last_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::resize_with(...).last().map(...)`, retaining constructor and live payload render methods |
| `fixture.vec_extend_from_slice_iter_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::extend_from_slice(...).iter().map(...)`, retaining cloned payload construction and read-only methods |
| `fixture.vecdeque_split_off_into_iter_prune.support_chain.001` | covered | A checked-in support fixture slices through `VecDeque<T>::split_off(...).into_iter().map(...)`, retaining deque payload methods |
| `fixture.linkedlist_split_off_into_iter_prune.support_chain.001` | covered | A checked-in support fixture slices through `LinkedList<T>::split_off(...).into_iter().map(...)`, retaining list payload methods |
| `fixture.btreeset_split_off_into_iter_prune.support_chain.001` | covered | A checked-in support fixture slices through `BTreeSet<T>::split_off(...).into_iter().map(...)`, retaining set payload methods |
| `fixture.btreemap_split_off_into_values_prune.support_chain.001` | covered | A checked-in support fixture slices through `BTreeMap<K, V>::split_off(...).into_values().map(...)`, retaining only live value methods and pruning dead key/value helpers |
| `fixture.btreemap_append_values_prune.support_chain.001` | covered | A checked-in support fixture slices through `BTreeMap<K, V>::append(...).values().map(...)`, retaining only live value methods after map mutation |
| `fixture.binaryheap_append_into_sorted_vec_prune.support_chain.001` | covered | A checked-in support fixture slices through `BinaryHeap<T>::append(...).into_sorted_vec().into_iter().map(...)`, retaining heap payload methods |
| `fixture.hashset_intersection_prune.support_chain.001` | covered | A checked-in support fixture slices through `HashSet<T>::intersection(...).map(...)`, retaining only live set payload methods |
| `fixture.hashset_union_prune.support_chain.001` | covered | A checked-in support fixture slices through `HashSet<T>::union(...).map(...)`, retaining only live set payload methods |
| `fixture.hashset_difference_prune.support_chain.001` | covered | A checked-in support fixture slices through `HashSet<T>::difference(...).map(...)`, retaining only live set payload methods |
| `fixture.hashset_symmetric_difference_prune.support_chain.001` | covered | A checked-in support fixture slices through `HashSet<T>::symmetric_difference(...).map(...)`, retaining only live set payload methods |
| `fixture.btreeset_intersection_prune.support_chain.001` | covered | A checked-in support fixture slices through `BTreeSet<T>::intersection(...).map(...)`, retaining only live set payload methods |
| `fixture.btreeset_union_prune.support_chain.001` | covered | A checked-in support fixture slices through `BTreeSet<T>::union(...).map(...)`, retaining only live set payload methods |
| `fixture.btreeset_difference_prune.support_chain.001` | covered | A checked-in support fixture slices through `BTreeSet<T>::difference(...).map(...)`, retaining only live set payload methods |
| `fixture.btreeset_symmetric_difference_prune.support_chain.001` | covered | A checked-in support fixture slices through `BTreeSet<T>::symmetric_difference(...).map(...)`, retaining only live set payload methods |
| `fixture.hashmap_values_mut_prune.support_chain.001` | covered | A checked-in support fixture slices through `HashMap<K, V>::values_mut().map(...)`, retaining only mutable value methods and pruning dead key/value helpers |
| `fixture.btreemap_values_mut_prune.support_chain.001` | covered | A checked-in support fixture slices through `BTreeMap<K, V>::values_mut().map(...)`, retaining only mutable value methods and pruning dead key/value helpers |
| `fixture.vec_append_iter_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::append(...).iter().map(...)`, retaining only live appended-vector payload methods |
| `fixture.vecdeque_append_iter_prune.support_chain.001` | covered | A checked-in support fixture slices through `VecDeque<T>::append(...).iter().map(...)`, retaining only live appended-deque payload methods |
| `fixture.slice_strip_prefix_iter_prune.support_chain.001` | covered | A checked-in support fixture slices through `[T]::strip_prefix(...).unwrap_or(...).iter().map(...)`, retaining only live slice payload methods |
| `fixture.slice_strip_suffix_iter_prune.support_chain.001` | covered | A checked-in support fixture slices through `[T]::strip_suffix(...).unwrap_or(...).iter().map(...)`, retaining only live slice payload methods |
| `fixture.vec_truncate_iter_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::truncate(...).iter().map(...)`, retaining only live vector payload methods after mutation |
| `fixture.vec_reverse_iter_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::reverse().iter().map(...)`, retaining only live vector payload methods after reordering |
| `fixture.vec_rotate_left_iter_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::rotate_left(...).iter().map(...)`, retaining only live vector payload methods after reordering |
| `fixture.vec_rotate_right_iter_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::rotate_right(...).iter().map(...)`, retaining only live vector payload methods after reordering |
| `fixture.vec_swap_iter_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::swap(...).iter().map(...)`, retaining only live vector payload methods after in-place swapping |
| `fixture.vec_fill_with_iter_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::fill_with(...).iter().map(...)`, retaining only live replacement payload construction and render methods |
| `fixture.slice_reverse_iter_prune.support_chain.001` | covered | A checked-in support fixture slices through `[T]::reverse().iter().map(...)`, retaining only live fixed-slice payload methods after reordering |
| `fixture.slice_rotate_left_iter_prune.support_chain.001` | covered | A checked-in support fixture slices through `[T]::rotate_left(...).iter().map(...)`, retaining only live fixed-slice payload methods after reordering |
| `fixture.slice_rotate_right_iter_prune.support_chain.001` | covered | A checked-in support fixture slices through `[T]::rotate_right(...).iter().map(...)`, retaining only live fixed-slice payload methods after reordering |
| `fixture.slice_swap_iter_prune.support_chain.001` | covered | A checked-in support fixture slices through `[T]::swap(...).iter().map(...)`, retaining only live fixed-slice payload methods after in-place swapping |
| `fixture.slice_split_first_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `[T]::split_first().map(|(item, tail)| ...)`, retaining only the live first-item payload method |
| `fixture.slice_split_last_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `[T]::split_last().map(|(item, tail)| ...)`, retaining only the live last-item payload method |
| `fixture.slice_split_first_mut_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `[T]::split_first_mut().map(|(item, tail)| ...)`, retaining only the live mutable first-item payload method |
| `fixture.slice_split_last_mut_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `[T]::split_last_mut().map(|(item, tail)| ...)`, retaining only the live mutable last-item payload method |
| `fixture.vec_split_first_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::split_first().map(|(item, tail)| ...)`, retaining only the live first-item payload method through autoderef |
| `fixture.vec_split_last_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::split_last().map(|(item, tail)| ...)`, retaining only the live last-item payload method through autoderef |
| `fixture.vec_split_first_mut_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::split_first_mut().map(|(item, tail)| ...)`, retaining only the live mutable first-item payload method through autoderef |
| `fixture.vec_split_last_mut_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Vec<T>::split_last_mut().map(|(item, tail)| ...)`, retaining only the live mutable last-item payload method through autoderef |
| `fixture.slice_split_at_tail_iter_prune.support_chain.001` | covered | A checked-in support fixture slices through `let (_, tail) = [T]::split_at(...); tail.iter().map(...)`, retaining only live tail payload methods |
| `fixture.slice_split_at_mut_tail_iter_prune.support_chain.001` | covered | A checked-in support fixture slices through `let (_, tail) = [T]::split_at_mut(...); tail.iter_mut().map(...)`, retaining only live mutable tail payload methods |
| `fixture.vecdeque_as_slices_iter_prune.support_chain.001` | covered | A checked-in support fixture slices through `VecDeque<T>::as_slices()` tuple destructuring and front-slice iteration without dead payload methods |
| `fixture.vecdeque_as_mut_slices_iter_prune.support_chain.001` | covered | A checked-in support fixture slices through `VecDeque<T>::as_mut_slices()` tuple destructuring and mutable front-slice iteration without dead payload methods |
| `fixture.cell_replace_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `Cell<T>::replace(...).method()`, retaining only the returned payload method and required interior-mutability import |
| `fixture.refcell_replace_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `RefCell<T>::replace(...).method()`, retaining only the returned payload method and pruning dead borrowed/mutable siblings |
| `fixture.once_lock_get_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `OnceLock<T>::get().map(...)`, retaining only the read-only locked payload method and import surface |
| `fixture.iter_from_fn_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `std::iter::from_fn(...).next().map(...)`, retaining payload methods created inside iterator factory closures |
| `fixture.iter_successors_map_prune.support_chain.001` | covered | A checked-in support fixture slices through `std::iter::successors(...).next().map(...)`, retaining successor seed payload methods without dead iterator siblings |
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
- Transitive support reexport aliases: copied support facades now forward
  associated method requirements through renamed dependency type aliases,
  including methods called on macro metavariable receivers.
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
