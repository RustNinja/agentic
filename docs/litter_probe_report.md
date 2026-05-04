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
| `ipc-random5-mixed-surfaces` | `Method`, `RequestHandler`, `IpcClientConfig`, `PendingRequests`, `read_frame` | feedback accepted | 121 | Pinned five-root corpus case in `scripts/corpus_cases/litter_codex_ipc_random5.json`. First run produced two unused imports in `client/handle.rs`; generic render-plan import pruning fixed them, and the rerun had zero feedback warnings. |

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
  pinned five-root codex-ipc corpus case reran in about 18 seconds with a warm
  feedback target dir, emitted 121 generated files, and had zero warnings after
  module-scoped import pruning and support `src/bin` exclusion.
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
- `protocol.enum_method.compact.001`: enum parser methods like `from_wire` should
  remain compact and keep only live variants/helpers.
- `bridge.state_constructor.heavy.001`: constructor roots with broad private
  state should expose exactly which fields/types force large support closure.
- `async_io.generic_root.001`: generic async I/O functions with `AsyncRead` /
  `AsyncWrite` bounds should stay compact and feedback-clean.
- `import.module_scoped.dead_item_only.001`: imports used only by dead items in
  a retained module must be pruned even when the same symbol is live in another
  module in the package.
