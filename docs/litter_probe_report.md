# Litter Probe Report

This report records real Litter probe runs from the local checkout at
`/Users/mykyta/Documents/New project 6/litter-analysis/shared/rust-bridge`.

The source checkout required `shared/third_party/codex` to be initialized before
Cargo could load the Rust workspace. Probe runs used temp copies with one
`#[opensourced::opensourced]` marker and a temp-only dependency on this repo's
`opensourced` proc macro. The original Litter source files were not edited.

## 2026-05-05 Non-Cfg Probe Batch

| Probe | Source item | Result | Generated files | Notes |
| --- | --- | --- | --- | --- |
| `ipc-project-state` | `codex-ipc::conversation_state::project_conversation_state` | production accepted | 854 | Heavy slice. Baseline passed, RA HIR analyzed 19 files, feedback `cargo check` passed. Warnings were discharged by feedback/matrix validation. |
| `ipc-method-from-wire` | `codex-ipc::protocol::method::Method::from_wire` | production accepted | 12 | Compact protocol enum/method slice. Good fast real-repo sanity probe. |
| `ipc-pending-resolve` | `codex-ipc::client::pending::PendingRequests::resolve` | production accepted | 15 | Compact async/channel-shaped support slice. Custom derive/attribute warnings were discharged. |
| `ipc-read-frame` | `codex-ipc::transport::frame::read_frame` | production accepted | 13 | Generic async I/O function root. Custom macro and syntactic fallback warnings were discharged. |
| `ipc-bridge-new` | `codex-ipc::bridge::IpcBridge::new` | production accepted | 862 | Heavy slice. Broad bridge/protocol state roots pull large support surfaces even when the selected item is small. |
| `ipc-random5-mixed-surfaces` | `Method`, `RequestHandler`, `IpcClientConfig`, `PendingRequests`, `read_frame` | production accepted | 116 | Pinned five-root corpus case in `scripts/corpus_cases/litter_codex_ipc_random5.json`. First run produced two unused imports in `client/handle.rs`; generic render-plan import pruning fixed them. Support library module-closure copying then removed test/tool-only support Rust files. The 2026-05-05 rerun after renamed-surface/default-trait hardening produced zero preflight errors, zero feedback errors, zero warnings, and `production_ready=accepted` under `--deny-warnings`. |
| `ipc-random5-dyn-boundary-regression` | `Method`, `RequestHandler`, `IpcClientConfig`, `PendingRequests`, `read_frame` | production accepted | 119 | 2026-05-06 rerun after transparent callback-boundary hardening and pinned-corpus source guard. Baseline passed, preflight passed, feedback `cargo check -p codex-ipc` produced zero errors/warnings, no production hazards remained, and `production_ready=accepted`. |
| `ipc-random5-dyn-proof-regression` | `Method`, `RequestHandler`, `IpcClientConfig`, `PendingRequests`, `read_frame` | production accepted | 119 | 2026-05-06 rerun after auto-trait and tightened concrete returned trait-object proof. Baseline passed, preflight passed, feedback `cargo check -p codex-ipc` produced zero errors/warnings, no production hazards remained, and `production_ready=accepted`. |
| `ipc-rootless-random20-import-pruning` | 20 package-scoped `codex-ipc` roots, seed `2026050620` | production accepted | 5-119 per slice | Rootless batch mining against `/tmp/litter-agentic-probe/shared/rust-bridge/Cargo.toml` used one RA HIR load, `--feedback-loop 2`, `--deny-warnings`, and a shared batch target directory. The first 20-root run exposed five warning-only failures: dead thiserror derive imports in conversation-state roots, a dead private-field `tokio::sync::RwLock` grouped import in reconnect roots, and an empty public glob reexport in the bridge root. Generic renderer fixes made the final rerun accept all 20/20 with zero warnings and zero errors; a separate manual `RUSTFLAGS='-D warnings' cargo check` pass also succeeded for every generated workspace. |

Two attempted `codex-mobile-client` probes did not reach slicer validation
because the source workspace baseline failed before slicing in third-party
`temporal_rs`/`icu_calendar` dependencies. Those are source/baseline blockers,
not slicer failures.

A later `codex-mobile-client` UniFFI smoke attempt also stopped before slicing:
the checked-out source was at `b5d9469`, while
`scripts/corpus_cases/litter_uniffi.json` pins `5ccb9a7`. The corpus runner now
validates a roots file's top-level `commit` before baseline builds, so stale
source/corpus mismatches fail fast unless `--allow-source-commit-mismatch` is
passed deliberately.

## Learnings

- The production flow can accept multiple real Litter `codex-ipc` roots with
  compiler feedback, including compact protocol, pending-request, transport, and
  bridge/state roots.
- A 2026-05-06 fresh-checkout rerun of the pinned five-root codex-ipc corpus
  case exposed three generic hardening gaps and then passed with the fixes:
  validation must scope Cargo checks to selected root packages (`-p codex-ipc`)
  so unrelated workspace members do not block the source baseline; the corpus
  runner must reconcile its temporary lockfile after injecting the local
  `opensourced` marker dependency because production validation uses `--locked`;
  and the render plan must keep feedback-widened roots when unknown-retention
  roots are also present. The accepted run generated 124 files, widened six
  missing-method roots from E0599 diagnostics, passed warning-clean compiler
  feedback on the second iteration, and finished as `review_required` because
  dynamic/macro/semantic warnings remain.
- Large file counts are now the most visible gap for some accepted slices.
  `project_conversation_state` and `IpcBridge::new` both accepted production
  validation but generated 850+ files because retained protocol/support path
  dependencies are copied broadly.
- Support-package trimming is now measurably helping the feedback loop. The
  pinned five-root codex-ipc corpus case reran warning-clean after both import
  pruning and support-package module-closure copying. The latest run emitted 116
  generated files and removed generic test/tool-only support files such as
  `src/*_tests.rs`, `src/*/tests.rs`, and support binary `main.rs`.
- Package-scoped rootless mining is now a useful fast feedback path. A 20-root
  `codex-ipc` batch reused one RA report and found renderer over-retention that
  the five-root corpus did not cover. The fixes stayed generic: derive-macro
  imports are retained only when rendered attributes need them, probable
  external trait imports no longer win on raw uppercase type mentions from
  pruned surfaces, and public glob reexports are checked against the rendered
  public surface instead of the original module.
- Support-package production hazard parity did not add blockers to the pinned
  five-root codex-ipc run: copied support packages had no retained support
  build-script/OUT_DIR/compile-env/file-include hazard debt, and the run stayed
  zero-warning under `--deny-warnings`.
- A follow-up read-only scan of `codex-mobile-client` added lightweight fixture
  coverage for the UniFFI/object shapes that make full Litter runs expensive:
  path-qualified derives, split helper attrs, error enums, Arc-returning object
  constructors, stored callback registries, callback/future aliases, and static
  `include_bytes!` assets. The generic fixes from that fixture are now in the
  reducer/import renderer rather than Litter-specific branches.
- The same fast fixture now covers private inline facade modules that reexport
  macro-decorated object surfaces, `pub(crate)` macro-helper reexports, and
  generic struct/trait/impl headers with `where` bounds. The fix was generic:
  retained impl surface scanning and render mention indexing now walk inline
  module items, so constructors, helper methods, local bounds, and imports are
  retained because the graph needs them, not because of Litter symbol names.
- After those generic fixes, the pinned five-root `codex-ipc` batch was rerun
  from `/Users/mykyta/Documents/New project 6/litter-analysis/shared/rust-bridge`
  with known source baseline failures allowed. The generated slice remained at
  116 files, passed production validation, and produced zero feedback errors or
  warnings under `--deny-warnings`.
- The renamed-surface and default-trait hardening pass did not regress the pinned
  corpus: the same five-root batch still generated 116 files and reached
  `production_ready=accepted` with zero warnings under `--deny-warnings`.
- A 2026-05-06 fresh `/tmp/litter-fresh` rerun found a grouped-import
  minimality bug in `codex-ipc/src/client/reconnect.rs`: `use tracing::{error,
  info, warn}` survived because retained code had local `Err(error)` bindings
  even though only `warn!` remained live. The renderer now uses scoped import-use
  analysis for private external imports, so the generated file contains
  `use tracing::warn;`. The same five-root batch generated 122 files, had zero
  preflight errors, zero feedback errors, zero warnings, and remained
  `review_required` only because semantic review hazards such as retained
  macros, trait objects, and unresolved RA queries still require review.
- The closure/trait-impl/hazard-scan pass also did not regress the pinned
  corpus: the same five-root batch still generated 116 files, had zero preflight
  errors, zero feedback errors, zero warnings, and reached
  `production_ready=accepted` under `--deny-warnings`.
- The trait/manifest minimality pass tightened generic pruning rather than
  adding Litter-specific names: type-only trait surfaces now prune dead optional
  members, local build-dependencies require a retained build script, and local
  dependency edges are kept by actual edge usage instead of retained package
  membership. The pinned five-root batch was rerun after that pass and still
  generated 116 files with zero preflight errors, zero feedback errors, zero
  warnings, and `production_ready=accepted` under `--deny-warnings`.
- The peer/alias precision pass also did not regress the pinned five-root batch:
  resolved trait peer methods are now receiver/input-type keyed, dependency
  package retention requires path/import evidence instead of ordinary local
  identifier matches, and the run still generated 116 files with zero preflight
  errors, zero feedback errors, zero warnings, and `production_ready=accepted`.
- The proc-macro/support pruning pass stayed generic and also did not regress
  the pinned batch: local proc-macro crates now retain only referenced exported
  proc macros, live proc-macro helper crates are copied as pruned `support/`
  packages instead of root members, dead helper crates are omitted, and retained
  module/inline-impl proc-macro attributes are detected from reachable surfaces.
  The pinned batch still generated 116 files with zero preflight errors, zero
  feedback errors, zero warnings, and `production_ready=accepted`.
- The bounded support item-pruning pass also kept the pinned batch accepted:
  no-build support package roots can now be transformed from concrete retained
  dependency public names, dead sibling root items are removed, and local path
  dependencies used only by those dead support items are no longer copied. The
  pinned batch still generated 116 files with zero preflight errors, zero
  feedback errors, zero warnings, and `production_ready=accepted`.
- The follow-up support pruning pass extended the same generic rule into child
  modules and simple public facade reexports: qualified `mod_name::item_name`
  edges seed live child items, retained `pub use module::{...}` leaves are
  narrowed to required public names, include assets are scanned from transformed
  support sources, and dead child/facade dependency uses disappear before
  support manifest dependency closure is computed. The pinned random-five batch
  still generated 116 files with zero preflight errors, zero feedback errors,
  zero warnings, and `production_ready=accepted`.
- Support facade pruning now follows one or more concrete public reexport
  hops, such as `root -> facade -> inner`, before rendering. This avoids
  copying dead middle-layer facade leaves and dead inner-module dependency
  uses. The pinned random-five batch remained accepted after the transitive
  pass with 116 files, zero preflight errors, zero feedback errors, and zero
  warnings.
- Local support glob facades are now resolved by concrete live names instead
  of forcing broad support copies. The renderer keeps original `pub use
  facade::*` / `use atoms::*` syntax, but prunes the target modules behind
  those globs so dead barrel siblings and their support dependencies disappear
  before manifest dependency closure. The pinned random-five batch remained
  accepted after the glob pass with 116 files, zero preflight errors, zero
  feedback errors, and zero warnings.
- Copied support packages now also resolve explicit dependency-crate reexport
  barrels such as `pub use dependency::{Live, Dead}` by manifest dependency
  alias/code-name roots. Live aliases stay in the support facade, dead aliases
  are pruned, and the downstream copied dependency is restricted to the concrete
  original dependency symbols that remain.
- Restricted copied support rendering now drops impl blocks for removed local
  self types. This keeps live type methods intact while preventing dead support
  impls from reintroducing removed types or redundant helper surfaces.
- RA HIR helps, but the accepted heavy slices still show many unresolved
  method/path warning surfaces before feedback discharge. The next generic work
  should reduce support-package copying and improve semantic precision around
  broad protocol types, not add project-specific allowlists.
- Mobile/UniFFI probes need a source baseline that compiles first. Until the
  source dependency version issue is fixed upstream or pinned in the temp probe,
  these probes are not useful slicer signals.
- On 2026-05-06, the `dnakov-litter-codex-ipc-random5` production corpus was
  rerun against a fresh Litter clone with the Codex submodule initialized. The
  batch selected `Method`, `RequestHandler`, `IpcClientConfig`,
  `PendingRequests`, and `read_frame`, generated 119 files, passed source
  baseline, preflight, feedback repair, lockfile, target coverage, and
  production-ready gates, and finished with zero feedback errors and zero
  warnings. The post-filter macro report retained real macro blockers but no
  longer reported benign `Option::is_none` blockers from serde helper attrs;
  `usage.blocked_by_unknown` stayed at zero while 1,750 callables and 861 items
  remained prunable.
- A follow-up rerun of the same fresh-checkout corpus after RA unresolved-path
  import classification also generated 119 files and remained
  `production_ready=accepted` with zero preflight errors, zero feedback errors,
  and zero warnings. Bounded RA still cannot load every external dependency
  definition in the default path, but unresolved local path names that are
  covered by retained external `use` / `pub use` leaves are now classified as
  benign instead of dependency risk. This removed the remaining
  `semantic_unresolved_paths` hazards for external protocol reexports such as
  `TurnStartParams` and `ReasoningEffort`; `semantic_unresolved_method_calls`
  also stayed absent from the Litter hazard list. Usage proof stayed narrow:
  `usage.blocked_by_unknown` remained zero, 1,750 callables and 861 items were
  prunable, and the production report only retained macro and bounded-syntactic
  review hazards.
- The next fresh-checkout rerun tightened helper-attribute macro modeling
  without changing the generated slice size or compile result. Serde/thiserror
  helper attributes are now treated as scoped data-contract/path helpers instead
  of possible source-rewriting attribute macros unless they name a real helper
  path. `custom_attribute_macros` dropped from 41 retained details to one real
  attribute macro surface, `#[async_trait::async_trait]`, while the same
  `dnakov-litter-codex-ipc-random5` batch stayed at 119 generated files, zero
  preflight errors, zero feedback errors, zero warnings, and
  `production_ready=accepted`.
- A subsequent fallback-noise pass stopped reporting generic
  `syntactic_method_fallbacks` when unresolved method calls retained zero local
  candidate methods by name. On the same Litter corpus, the generated slice
  stayed at 119 files and `production_ready=accepted`; `syntactic_method_fallbacks`
  disappeared, only the capped-fallback marker remained, unknown surfaces dropped
  from 7 to 6, and dependency-risk unknown surfaces dropped from 2 to 1 without
  increasing `usage.blocked_by_unknown`.
- The capped-fallback follow-up made the fallback marker precise enough to find
  the remaining real source shape: `String::new()` inside
  `codex-ipc::transport::frame::read_frame` was being treated as an unresolved
  local `new` method and hit the same-name cap because two local `new` methods
  existed elsewhere. The reducer now only applies associated-call method-name
  fallback when the receiver prefix resolves to a local type. The same Litter
  corpus stayed at 119 generated files and `production_ready=accepted`;
  `syntactic_method_fallback_cap` disappeared, dependency-risk unknown surfaces
  dropped to zero, and `usage.blocked_by_unknown` remained zero.
- The imported logging macro pass removed the remaining function-like macro
  invocation warnings from the pinned five-root batch without name-specific
  Litter logic: `tracing`/`log` macros such as `trace!`, `debug!`, and `warn!`
  are now classified as format-like only when package dependency and `use`
  evidence prove the macro source. The corpus still generated 119 files with
  zero preflight errors, zero feedback errors, zero warnings, and
  `production_ready=accepted`; `custom_macro_invocations` dropped to zero, macro
  invocation surfaces dropped to zero, and `usage.blocked_by_unknown` remained
  zero.
- The dependency-proven `async_trait` pass removed the one remaining retained
  custom attribute macro warning from the same corpus. Path-qualified or imported
  `async_trait` attributes are now treated as a known async trait transform only
  when package dependency evidence proves the macro source; arbitrary attribute
  macros remain fail-closed. The batch still generated 119 files, passed with
  zero preflight errors, zero feedback errors, zero warnings, and
  `production_ready=accepted`; `custom_attribute_macros` dropped to zero,
  macro-blocked unknown surfaces dropped to one, and `usage.blocked_by_unknown`
  remained zero.
- The known derive-contract pass removed the remaining serde/thiserror macro
  blockers without relaxing arbitrary derive handling. Dependency/import-proven
  serde `Serialize`/`Deserialize` and thiserror `Error` derives now reuse the
  slicer's existing field/helper/trait contract modeling; unrecognized custom
  derives still produce production feedback warnings. The pinned five-root batch
  again generated 119 files with zero preflight errors, zero feedback errors,
  zero warnings, and `production_ready=accepted`; all `custom_*_macros` and
  function-like macro invocation hazards disappeared, macro surfaces dropped to
  zero, macro-blocked unknown surfaces dropped to zero, and
  `usage.blocked_by_unknown` remained zero.
- The semantic proof pass was report-only and did not change the generated
  Litter slice. The same five-root batch stayed at 119 files with zero preflight
  errors, zero feedback errors, zero warnings, and `production_ready=accepted`.
  The new `usage.semantic_proof` certificate reported
  `complete_for_retained_packages`: 1 retained package, 1,621 package-pruned
  callables and 789 package-pruned items, plus 129 retained-package prunable
  callables and 72 retained-package prunable items, all proven by RA mapping and
  reference search with zero unproven, unmapped, failed-query, or retained-owner
  reference cases.
- The semantic proof gate then used that certificate to clear the last generic
  semantic production warnings. The five-root batch still generated 119 files
  with zero preflight errors, zero feedback errors, zero warnings, and
  `production_ready=accepted`, while `production.hazards` became empty. This is
  intentionally narrow: only broad `semantic_reduction_hints_applied` and
  `semantic_inventory_partially_applied` warnings are suppressed when
  retained-package pruning is fully proven; concrete unresolved/query,
  macro/generated/dyn/cfg hazards remain fail-closed.

## Next Rule Targets

- `manifest.support_path_dependency.minimal.001`: retained source refers to one
  external path dependency type, but the copied support package should not bring
  unrelated files when only a narrow API is required.
- `manifest.support_library_module_closure.001`: no-build copied support
  packages should copy the library module graph, skip `#[cfg(test)]` external
  modules and orphan Rust files, copy only static assets mentioned by copied
  modules, and fall back to broader copying only when the module graph cannot be
  parsed safely.
- `manifest.support_build_script_hazard_parity.001`: covered for copied support
  packages with build scripts, `OUT_DIR` source includes, compile-time env,
  nonliteral includes, absolute includes, and package-external support include
  paths.
- `manifest.support_facade_export_pruning.001`: covered for simple and
  transitive copied support facades, including local glob facades, that
  reexport concrete child-module leaves or explicit dependency-crate symbols.
- `manifest.support_monolith_item_pruning.001`: covered for protocol-shaped
  support crates where dead enum variants mention sibling payload structs/enums.
  The render plan now treats those payload names as droppable unless another
  reachable item, root surface, or unknown blocker really uses them, so same-name
  dead facade reexports and now-empty child modules are pruned.
- `manifest.support_transitive_reexport_pruning.001`: app-to-local-to-support
  reexport chains should keep only the selected upstream symbol and prune dead
  middle-layer reexports.
- `manifest.support_inline_test_cfg_asset_pruning.001`: covered for
  `#[cfg(test)]` items/statements/expressions in retained root and support
  files. Generated slices strip those test-only statements before rendering and
  skip their `include_str!`/`include_bytes!` assets. Broader non-test target cfg
  asset pruning remains under the cfg-oracle track.
- `protocol.enum_method.compact.001`: covered for enum parser/render methods
  like `from_wire` that retain live variants/helpers while dropping payload-only
  items introduced by pruned variants.
- `bridge.state_constructor.heavy.001`: constructor roots with broad private
  state should expose exactly which fields/types force large support closure.
- `async_io.generic_root.001`: generic async I/O functions with `AsyncRead` /
  `AsyncWrite` bounds should stay compact and feedback-clean.
- `import.module_scoped.dead_item_only.001`: imports used only by dead items in
  a retained module must be pruned even when the same symbol is live in another
  module in the package.
