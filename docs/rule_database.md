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
`crates/opensource_core/tests/rule_database.rs`.

## First Seed Rules

| Rule | Status | Generic behavior |
| --- | --- | --- |
| `import.reexport.grouped.001` | covered | Grouped public reexports prune removed names while keeping live names |
| `import.reexport.alias_removed.001` | covered | A removed public alias is pruned even when the alias text appears as a local binding |
| `dyn.owned.registry.001` | covered | Stored `Box<dyn Trait>` and callback aliases are hard production hazards |
| `include.bytes.static.001` | covered | Retained `include_bytes!` assets are copied and dead sibling assets are not |
| `build.rustc_env.001` | covered | Retained `env!` fed by build script state is production-blocking |
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

## Workflow

When a real repo slice fails or retains too much code:

1. Identify the smallest generic Rust shape that caused the issue.
2. Add a tiny rule case under `rule_database.rs` or a future grouped rule file.
3. Record the source inspiration in this doc if it came from Litter or another
   real repo.
4. Fix the slicer generically, avoiding project-name/symbol-name allowlists.
5. Run `scripts/fast_rule_loop.sh`, then the broader workspace checks before
   committing.

The target is hundreds of small rules plus generated combinations where useful,
not a thousand hand-written one-off tests. More cases are valuable only when
they introduce a distinct Rust/Cargo shape or a distinct failure mode.

## Litter Patterns To Convert Next

Read-only Litter exploration found these high-value non-cfg patterns:

- UniFFI object roots with `#[derive(uniffi::Object)]`, exported impls,
  constructors, async methods, and private inner state.
- Callback interface roots stored as `Option<Arc<dyn CallbackTrait>>`.
- `async_trait` traits used through `Arc<dyn Trait>`.
- Nested callback/future aliases like
  `Arc<dyn Fn() -> Pin<Box<dyn Future<...>>>>`.
- `macro_rules!` helper modules that `pub(crate) use` a macro and invoke it from
  exported methods.
- Error enums combining `thiserror::Error` and `uniffi::Error`.
- Static `include_str!` and `include_bytes!` asset groups.
- Broad `pub use` hubs that need live-name pruning through reexport chains.
- Serde/UniFFI helper attrs such as field defaults and skip helpers.
- Conversion-heavy boundary impls such as `TryFrom<Request> for Params`.
- Macro bodies that call methods on metavariables, where the invocation argument
  type is the only generic way to retain the required method.
- Direct callback API boundaries, source `include!` blockers, dependency aliases
  inside macro bodies, and inline script-bundle modules.

Cfg/custom-cfg matrix expansion is intentionally not part of this batch.
