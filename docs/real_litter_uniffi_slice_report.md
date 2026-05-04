# Real UniFFI Slice Report: dnakov/litter

Date: 2026-05-02
Last updated: 2026-05-04

Source project: `dnakov/litter`

- GitHub: `https://github.com/dnakov/litter`
- Stars at check time: 1436
- Default branch commit tested: `5ccb9a7641946805abdd057e7327dbf005719a77`
- Rust bridge root: `shared/rust-bridge`
- Main UniFFI crate tested: `codex-mobile-client`

The initial `git clone --depth 1` failed with an RPC/early EOF network error, so
the experiment used the GitHub `main` tarball extracted under `/tmp/slicers-real`.

## Why This Project

`litter` is a real mobile Rust/UniFFI codebase with Swift/Kotlin-facing exports
and nontrivial business logic. The tested area was the mobile preferences and
iCloud KVS sync path:

- `codex-mobile-client/src/cloud_sync/mod.rs`
- `codex-mobile-client/src/preferences.rs`

The original Rust bridge contains 6 workspace members, 146 Rust source files,
and 72,917 Rust LOC outside `target`.

## Setup

Each experiment copied the real Rust bridge into `/tmp`, added:

```toml
opensourced = { path = "/Users/mykyta/dev/rustclaw/agentic-sg/crates/opensourced" }
```

and placed `#[opensourced]` on exactly one real UniFFI-exported function. The
slicer was run as a built binary:

```sh
cargo build -p opensource_cli --bin slicers
./target/debug/slicers <experiment-workspace> <slice-output>
cargo check --manifest-path <slice-output>/Cargo.toml
```

For the apply slice, I also ran:

```sh
cargo build --manifest-path /tmp/slicers-litter-out-apply-fixed/Cargo.toml
```

## Experiments

| Root function | Output | Result |
| --- | --- | --- |
| `cloud_sync_export_snapshot` | `/tmp/slicers-litter-out-cloud-export-fixed.KcwsU3` | `cargo check` passed |
| `cloud_sync_apply_snapshot` | `/tmp/slicers-litter-out-apply-fixed` | `cargo check` passed, `cargo build` passed |
| `preferences_add_hidden_thread` | `/tmp/slicers-litter-out-preferences` | `cargo check` passed |

## Pinned Corpus Rerun

On 2026-05-04, the same three roots were added to
`scripts/corpus_cases/litter_uniffi.json` and rerun through the generic pinned
corpus harness in preflight mode against the pinned Litter commit. All three
cases passed preflight:

| Corpus case | Files written | Packages | Reachable callables | Reachable items |
| --- | ---: | --- | ---: | ---: |
| `cloud-sync-export-snapshot` | 6 | `codex-mobile-client` | 15 | 15 |
| `cloud-sync-apply-snapshot` | 13 | `codex-mobile-client` | 24 | 38 |
| `preferences-add-hidden-thread` | 9 | `codex-mobile-client` | 10 | 16 |

The same pinned cases also passed strict feedback repair with `--deny-warnings`
through the RA analyzer path once RA was bounded to local workspace crates, RA
semantic analysis prioritized files containing selected roots, and the feedback
runner drained Cargo JSON while the child process was still running:

| Corpus case | Feedback result | Repair changes | End warnings |
| --- | --- | ---: | ---: |
| `cloud-sync-export-snapshot` | accepted | 0 | 0 |
| `cloud-sync-apply-snapshot` | repaired, then accepted | 3 unused imports removed | 0 |
| `preferences-add-hidden-thread` | accepted | 0 | 0 |

The apply and preferences cases previously retained roughly 2,360 generated
files by following ambiguous name-only method fallbacks into unrelated mobile
client and Codex IPC surfaces. The reducer now keeps only unambiguous
receiver-less fallback matches and reports ambiguous matches as feedback
hazards, which keeps these pinned UniFFI slices small without hardcoding Litter
paths or symbols.

## RA Feedback 10-Root Codex IPC Smoke

On 2026-05-04, a fresh `dnakov/litter` clone at
`50be6e1 pets perf; fix server filter on search` was tested with the
`codex/slice-ra-feedback` branch. The Codex submodule was initialized, then a
temporary copy under `/tmp/litter-ra-feedback-marked` added the local
`opensourced` marker dependency and marked 10 real functions in one module:
`shared/rust-bridge/codex-ipc/src/conversation_state.rs`.

Marked roots:

- `strip_request_wrapper`
- `infer_cwd`
- `parse_turn_status`
- `serialize_turn_status`
- `parse_unix_seconds`
- `parse_timestamp`
- `request_id_string`
- `path_to_string`
- `non_empty`
- `non_empty_option_owned`

Command shape:

```sh
slicers \
  --analyzer ra-feedback \
  --preflight \
  --check \
  --feedback \
  --feedback-repair-loop 4 \
  --deny-warnings \
  --cargo-check-arg -p \
  --cargo-check-arg codex-ipc \
  --cargo-check-arg --all-targets \
  /tmp/litter-ra-feedback-marked/shared/rust-bridge \
  /tmp/litter-ra-feedback-slice-clean
```

Result:

- Generation passed in about 12s.
- Preflight passed for 1 retained package and 2 retained Rust files.
- Reachable callables were exactly the 10 marked functions.
- Reachable support items were `DesktopConversationState`,
  `DesktopPendingRequest`, `DesktopTurn`, `DesktopTurnParams`, and
  `MY_REQUEST_HEADER`.
- First feedback check passed compilation but reported 6 warnings.
- Conservative repair removed 3 unused imports across `conversation_state.rs`
  and `lib.rs`.
- Second feedback check passed with 0 warnings.
- Final validation status: `accepted` with `--deny-warnings`.

The generated `codex-ipc/src/conversation_state.rs` retained these declarations
only:

- `MY_REQUEST_HEADER`
- `DesktopConversationState`
- `DesktopTurn`
- `DesktopTurnParams`
- `DesktopPendingRequest`
- the 10 selected helper functions above

## RA Feedback Large Module Smoke

On 2026-05-04, the same fresh Litter clone was tested by marking one of the
largest Rust modules in the project:

```rust
#[opensourced]
pub mod reducer;
```

Target module:
`codex-mobile-client/src/store/reducer.rs` (`5,368` original LOC).

Command shape:

```sh
slicers \
  --analyzer ra-feedback \
  --preflight \
  --check \
  --feedback \
  --feedback-repair-loop 6 \
  --deny-warnings \
  --cargo-check-arg -p \
  --cargo-check-arg codex-mobile-client \
  --cargo-check-arg --lib \
  /tmp/litter-ra-feedback-big-module/shared/rust-bridge \
  /tmp/litter-ra-feedback-big-module-slice-fixed
```

Initial RA pass:

- RA feedback queried 597 callable owners.
- RA feedback observed 766 outgoing call targets.
- RA feedback mapped 662 project-local edges.
- Preflight passed for 2 retained packages, 29 target Rust files, and 1 local
  path dependency.

The first compiler feedback check exposed three missing semantic edges:

- `VoiceRealtimeThreadState::handle_item`
- `From<codex_app_server_protocol::RateLimitSnapshot> for RateLimitSnapshot`
- `From<codex_app_server_protocol::TurnPlanStepStatus> for AppPlanStepStatus`

The slicer was hardened to treat path-form Cargo package IDs as package hints
and to classify `E0277` conversion trait errors as feedback widening
candidates. The rerun then widened those roots, exposed nested conversion impls,
widened those too, and accepted the generated slice:

| Attempt | Result | Errors | Warnings | Action |
| --- | --- | ---: | ---: | --- |
| 1 | widened | 3 | 5 | added 4 roots from compiler diagnostics |
| 2 | widened | 8 | 5 | added 7 conversion/method roots total |
| 3 | repaired | 0 | 5 | removed 5 unused imports |
| 4 | accepted | 0 | 0 | final warning-clean compile |

Final retained target surface:

- `codex-mobile-client`
- `codex-ipc`
- 29 target Rust files
- 241 reachable callables
- 167 reachable items
- 8,539 retained target Rust LOC
- `store/reducer.rs` reduced from 5,368 LOC to 3,533 LOC

Final validation status: `accepted` with `--deny-warnings`.

## RA Feedback Second-Largest Module Smoke

On 2026-05-04, the second-largest module in the same Litter checkout was
marked:

```rust
#[opensourced]
mod mobile_client;
```

Target module:
`codex-mobile-client/src/mobile_client/mod.rs` (`3,147` original LOC).

Command shape:

```sh
slicers \
  --analyzer ra-feedback \
  --preflight \
  --check \
  --feedback \
  --feedback-repair-loop 6 \
  --deny-warnings \
  --cargo-check-arg -p \
  --cargo-check-arg codex-mobile-client \
  --cargo-check-arg --lib \
  /tmp/litter-ra-feedback-second-module/shared/rust-bridge \
  /tmp/litter-ra-feedback-second-module-slice
```

This smoke found and fixed three generic slicer issues before hitting
source-checkout API drift:

- Copied support-package manifests can depend on crates patched only by the
  original workspace root. The generated root now retains patch entries needed
  by copied support packages, not only selected workspace packages.
- Copied support packages can contain `include_str!`/`include_bytes!` paths
  that intentionally reach to their original workspace root. The support copy
  now scans Rust files and copies those workspace-level include assets into the
  equivalent generated relative location.
- Imports used only through Rust 2021 format-string captures, `FromStr` trait
  associated calls, or child modules with `use super::*` are now retained.

After those fixes, the strict feedback loop progressed from 64 errors to 23
errors and eliminated the slicer-owned missing import failures
(`PROFILE_INIT`, `FromStr`, `Hash`, and `Hasher`). The remaining top failures
match the original Litter checkout rather than the generated slice:

- `RemoteAppServerClient::connect_websocket_stream` is referenced by
  `codex-mobile-client/src/alleycat.rs`, but the pinned
  `codex-app-server-client` source does not define it.
- `RemoteAppServerClient::connect_json_line_stream` is referenced by
  `alleycat.rs` and `ssh_bridge.rs`, but the pinned dependency does not define
  it.
- `codex_app_server_protocol` is missing protocol fields/variants used by the
  mobile client, including `DynamicToolCallArgumentsDelta`,
  `approval_policy`, and `sandbox`.

Direct baseline check of the marked original package,
`cargo check -p codex-mobile-client --lib`, also failed with those same
dependency/API drift errors. This means the second-largest module is currently
not a valid acceptance corpus until the Litter checkout and pinned Codex
dependency versions are aligned.

### `cloud_sync_export_snapshot`

Retained package: `codex-mobile-client`

Retained Rust source:

- `codex-mobile-client/src/lib.rs`
- `codex-mobile-client/src/cloud_sync/mod.rs`
- `codex-mobile-client/src/preferences.rs`

Size: 3 Rust files, 251 LOC.

Important retained callables:

- `cloud_sync_export_snapshot`
- `export_snapshot`
- `build_snapshot`
- `encode_envelope`
- platform table helpers
- preference read/path/default helpers
- `From<PersistedPreferences> for MobilePreferences`

Manifest pruning result:

- Kept: `serde`, `serde_json`, `thiserror`, `uniffi`
- Removed: local `codex-ipc`, Codex third-party workspace deps, SSH/session/store/discovery deps, `opensourced`

### `cloud_sync_apply_snapshot`

Retained package: `codex-mobile-client`

Size: 3 Rust files, 399 LOC.

Important retained callables:

- `cloud_sync_apply_snapshot`
- `apply_snapshot`
- `decode_envelope`
- `merge_rust_key`
- `with_platform_table`
- preference read/write/path helpers
- both local `From` conversion impls used by `.into()`

Important retained items:

- `PlatformWriteback`
- `CloudEntry`
- `CloudSnapshot`
- `CloudSyncError`
- `PLATFORM_KEYS`
- `RUST_KEY_PINNED_THREADS`
- `RUST_KEY_HIDDEN_THREADS`
- `RUST_KEY_HOME_SELECTION`
- preference DTOs and persistence structs

Manifest pruning result:

- Kept: `serde`, `serde_json`, `thiserror`, `tracing`, `uniffi`
- Removed: unrelated local crates and large workspace deps

### `preferences_add_hidden_thread`

Retained package: `codex-mobile-client`

Size: 2 Rust files, 161 LOC.

Retained source:

- `codex-mobile-client/src/lib.rs`
- `codex-mobile-client/src/preferences.rs`

Important retained callables:

- `preferences_add_hidden_thread`
- `preferences_path`
- `read_preferences`
- `write_preferences`
- default impls and conversion impls required by persistence

Manifest pruning result:

- Kept: `serde`, `serde_json`, `tracing`, `uniffi`
- Removed: `cloud_sync`, SSH/session/store/discovery/mobile-client modules and deps

## Failures Found And Fixed

1. UniFFI scaffolding was pruned.

   The first generated real slice retained `#[uniffi::export]`,
   `#[derive(uniffi::Record)]`, and `#[derive(uniffi::Error)]`, but dropped
   `uniffi::setup_scaffolding!();` from crate root. `cargo check` failed with
   missing `UniFfiTag`.

   Fix: retain `setup_scaffolding!` item macro invocations for packages with
   reachable UniFFI exports/records/errors.

2. `.into()` dropped required `From` impls.

   Real preferences code used `persisted.into()` and `value.into()`. The slicer
   retained the structs but pruned `impl From<PersistedPreferences> for
   MobilePreferences` and `impl From<&MobilePreferences> for
   PersistedPreferences`.

   Fix: record trait input type paths, seed function parameter types into the
   dependency visitor, infer `Result::Ok` bindings from turbofish calls, and
   resolve `.into()` / `.try_into()` to local `From` / `TryFrom` impls.

3. Unqualified const patterns in `match` arms were missed.

   The first apply slice compiled, but `cargo check` warned that
   `RUST_KEY_PINNED_THREADS` was treated as a binding, making later match arms
   unreachable. That meant the slice built but had wrong merge semantics.

   Fix: resolve unqualified `Pat::Ident` patterns to reachable local
   `const`/`static` items when such items exist in scope.

Regression coverage added:

- `crates/opensource_core/tests/pattern_constants.rs`

4. Feedback cargo checks could appear hung on real dependency graphs.

   `cargo check --message-format=json` can emit enough compiler artifact JSON
   to fill a captured stdout pipe before the process exits. The first strict
   pinned Litter repair run timed out after 600s while Cargo was blocked.

   Fix: drain Cargo stdout/stderr on reader threads while polling the child
   process for completion and timeout. The same pinned strict repair rerun then
   completed all three cases in seconds with warmed dependencies.

5. Warning repair was accepted too early.

   The repair loop accepted successful Cargo checks before removing repairable
   warnings when warning denial was disabled. This hid generated unused imports
   until strict mode.

   Fix: feedback repair now requires zero repairable warnings before accepting
   a successful check, and consumes machine-applicable rustc help for whole-use
   unused import removals.

## What Was Correctly Cut

The real slices did not include:

- `codex-ipc`
- `codex-bridge`
- `codex-tui`
- `codex-debug-cli`
- `uniffi-bindgen`
- SSH modules
- session modules
- store reducers
- mobile client event loop
- discovery/alleycat code
- cloud sync code in the preference-only slice
- apply/update paths in the export-only slice, except comments left by original module docs

## Remaining Known Gaps

These experiments now pass strict compiler feedback repair through the
RA-enabled CLI path. The default CLI feature set loads bounded rust-analyzer HIR
semantics for local workspace crates and applies exact project-local RA
method/path resolutions as additive reduction hints; the production preset now
uses `ra-hir-proc-macros` to ask rust-analyzer for build-script output discovery
and proc-macro expansion during semantic inventory. The latest pinned Litter
repair run recorded `semantic_reduction_hints_applied` for all three roots. RA
semantic inventory now includes per-file reports, so production readiness
semantic warnings are scoped to retained slice files when reduction can map the
slice back to source files, before falling back to selected-root and workspace
counts. The
reducer still keeps the syntactic closure as fallback and does not yet use RA as
the authoritative oracle for trait impl lookup, macro-expanded item retention,
generated source, dynamic dispatch, or every cfg-active reachability decision.
The `codex/slice-ra-feedback` branch adds an isolated `ra-feedback` analyzer
mode to test the next architecture: copy enough source first, ask
rust-analyzer outgoing call hierarchy for selected-root and syntactic-retained
owner files, then prune with the existing `syn` renderer from the RA-backed
keep set.

Known remaining risks:

- Fast preflight can still predict slices that later need compiler repair.
  Production/repair validation now removes repairable unused imports before
  accepting warning-free feedback.
- `build.rs` and referenced non-Rust assets are copied, but generated `OUT_DIR`
  Rust code is not semantically modeled.
- Target-specific dependency tables are preserved when retained source
  references them. Selected cfg-gated roots can now be discharged for
  recognized feature/target `all(...)`, `any(...)`, and `not(...)` expressions
  proven by validation args and `rustc --print cfg`, but custom cfgs and
  retained non-root platform matrices remain conservative.
- Feature tables are rewritten for pruned optional dependencies, but full
  feature-resolution semantics are still conservative.
- Proc macro expansion can be requested by the production analyzer, but
  macro-expanded items are not yet mapped into first-class retained source.
  Full Cargo dependency artifact discovery for proc macros is opt-in because it
  timed out on this pinned Litter corpus.
- Glob imports/reexports are still conservative.
- Full rustc-equivalent `cfg` matrix inventory is not implemented.

The real Litter slices exercised traits, impls, enums, structs, constants,
statics, modules, UniFFI exports, UniFFI records/errors, match patterns, and
manifest dependency pruning across a real mobile UniFFI crate.
