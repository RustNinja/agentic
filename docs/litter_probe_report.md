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

Two attempted `codex-mobile-client` probes did not reach slicer validation
because the source workspace baseline failed before slicing in third-party
`temporal_rs`/`icu_calendar` dependencies. Those are source/baseline blockers,
not slicer failures.

## Learnings

- The production flow can accept multiple real Litter `codex-ipc` roots with
  compiler feedback, including compact protocol, pending-request, transport, and
  bridge/state roots.
- Large file counts are now the most visible gap for some accepted slices.
  `project_conversation_state` and `IpcBridge::new` both accepted production
  validation but generated 850+ files because retained protocol/support path
  dependencies are copied broadly.
- Support-package trimming is now measurably helping the feedback loop. The
  pinned five-root codex-ipc corpus case reran warning-clean after both import
  pruning and support-package module-closure copying. The latest run emitted 116
  generated files and removed generic test/tool-only support files such as
  `src/*_tests.rs`, `src/*/tests.rs`, and support binary `main.rs`.
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
- RA HIR helps, but the accepted heavy slices still show many unresolved
  method/path warning surfaces before feedback discharge. The next generic work
  should reduce support-package copying and improve semantic precision around
  broad protocol types, not add project-specific allowlists.
- Mobile/UniFFI probes need a source baseline that compiles first. Until the
  source dependency version issue is fixed upstream or pinned in the temp probe,
  these probes are not useful slicer signals.

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
- `manifest.support_monolith_item_pruning.001`: support crates with large
  protocol files should not retain unrelated request/notification siblings when
  one symbol is referenced.
- `manifest.support_transitive_reexport_pruning.001`: app-to-local-to-support
  reexport chains should keep only the selected upstream symbol and prune dead
  middle-layer reexports.
- `manifest.support_inline_test_cfg_asset_pruning.001`: retained support files
  should strip inline `#[cfg(test)]` modules and avoid copying target-gated
  assets for inactive targets.
- `protocol.enum_method.compact.001`: enum parser methods like `from_wire` should
  remain compact and keep only live variants/helpers.
- `bridge.state_constructor.heavy.001`: constructor roots with broad private
  state should expose exactly which fields/types force large support closure.
- `async_io.generic_root.001`: generic async I/O functions with `AsyncRead` /
  `AsyncWrite` bounds should stay compact and feedback-clean.
- `import.module_scoped.dead_item_only.001`: imports used only by dead items in
  a retained module must be pruned even when the same symbol is live in another
  module in the package.
