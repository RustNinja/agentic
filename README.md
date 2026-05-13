# agentic-sg slicers

## Slicer Proof

This workspace is a proof-of-concept for marking one Rust function with
`#[opensourced]` and generating a reduced Rust workspace that keeps only the
function-level dependency closure needed by that entry point. The intended use
case is giving a human or AI agent a small, compilable slice of a large Rust
workspace instead of the full source tree.

## Shape

- `crates/opensourced`: proc-macro crate that validates and preserves a marked
  free function.
- `crates/opensource_core`: `syn`-first parser, call-graph reducer, source
  renderer, compiler-feedback repair, and rust-analyzer HIR inventory when the
  `ra-hir` feature is enabled.
- `crates/opensource_cli`: command-line wrapper around `opensource_core`.
  The primary binary is `slicers`.
- `fixtures/a` through `fixtures/e`: five crates used as the proof workspace.
- `fixtures/fast_macro_use`: a standalone fast stress workspace for Litter-like
  macro-declared items, imports, derive/helper attributes, UniFFI-shaped export,
  async, callback, cfg, static include assets, conversion impl, associated
  type/const trait impl, `FromStr`/`Display`/`Deref` trait edges, item-root,
  OUT_DIR blocker, and dead-pruning iteration.

The fixture dependency graph is:

```text
a -> b -> d -> e
a -> c -> d -> e
```

`fixtures/a/src/lib.rs` marks only `open_source_entry`:

```rust
#[opensourced]
pub fn open_source_entry(value: i32) -> i32 {
    renamed_b::compute(value) + c::adjust(value)
}
```

The reducer finds that marker, walks reachable calls and type/API references
through the workspace, and writes a new workspace without unreachable
implementation code, unused local crates, or unit tests.

## Why This Is Not Proc-Macro-Only

A Rust proc macro receives only the token stream it is attached to. It cannot
reliably inspect the whole crate or dependency workspace. The macro is therefore
only the marker/validator. Whole-workspace analysis is done by a separate tool
that parses source files with `syn`.

## Covered In The Proof

The fixture and tests cover:

- cross-crate free function calls;
- transitive dependency calls;
- module alias calls such as `use b as renamed_b`;
- function alias calls such as `use d::hash as renamed_hash`;
- private helpers reached from public code;
- associated function calls such as `d::Worker::new`;
- receiver method calls inferred from a local binding such as `worker.run(...)`;
- trait method calls inferred from a local receiver, such as
  `worker.transform(...)`;
- trait definitions and trait impl blocks needed by reachable methods;
- borrowed trait UFCS calls such as `Trait::method(&receiver, ...)`;
- enums, structs, type aliases, consts, and statics used in reachable
  signatures or implementations;
- pruning unused data/API items such as an unreachable enum;
- pruning public, private, and method functions that are not reachable;
- pruning unused local path dependency crates from generated manifests;
- pruning unused local dependency imports from retained source files;
- skipping external module files with no reachable descendants;
- preserving external and workspace dependencies such as `serde`;
- pruning unused UniFFI scaffolding, UniFFI-only derive entries, and mobile
  `cdylib` crate types when the selected root is plain Rust;
- dropping `#[cfg(test)]` modules and `#[test]` functions from generated output;
- stripping the marker macro and its dependency from the generated workspace;
- compiling the generated workspace with `cargo check`.

The reducer still keeps imports and associated const/type items conservatively
inside retained impl blocks so the reduced source keeps compiling.

## Production Notes

The production architecture, current boundaries, and real-repo lessons are kept
in `docs/production_readiness_plan.md`, `docs/slice_coverage_matrix.md`, and
`docs/production_hardening_learnings.md`. Read the learnings document before
adding project-specific fixes; real-repo failures should become generic rules,
regression fixtures, or explicit fail-closed hazards.

## Run

```sh
cargo test --workspace
cargo run -p opensource_cli --bin slicers -- --check . /tmp/slicers-proof
cargo run -p opensource_cli --bin slicers -- --preflight . /tmp/slicers-preflight
cargo run -p opensource_cli --bin slicers -- --feedback . /tmp/slicers-feedback
cargo run -p opensource_cli --bin slicers -- --feedback-repair-loop 3 . /tmp/slicers-repair
cargo run -p opensource_cli --bin slicers -- --baseline-check --feedback . /tmp/slicers-feedback
cargo run -p opensource_cli --bin slicers -- --production . /tmp/slicers-production
cargo run -p opensource_cli --bin slicers -- --slice-report /tmp/slicers-report.json . /tmp/slicers-proof
cargo check --manifest-path /tmp/slicers-proof/Cargo.toml
```

The CLI can also select roots without editing the source tree. `--root` and
`--roots-file` add in-memory roots equivalent to placing `#[opensourced]` on
those functions/items; existing `#[opensourced]` markers still work and are
combined with explicit roots.

```sh
cargo run -p opensource_cli --bin slicers -- \
  --root my_crate::module::function \
  --preflight \
  /path/to/source/Cargo.toml /tmp/slicers-rootless
```

For failure mining, use `--batch-roots` or `--random-roots` so the CLI loads the
workspace and rust-analyzer database once, then emits one generated workspace
per selected root under the output directory. Each row is written to
`batch-report.jsonl`; when validation runs and no package scope was supplied,
the generated `cargo check` is scoped to the root package. Batch feedback uses a
shared target directory under the batch output by default, or the explicit
`--feedback-target-dir`, so dependency builds are reused across roots.
Production batch validation also reconciles each generated `Cargo.lock` before
running locked checks, using Cargo's offline resolver first and falling back to
online resolution only when the user did not request `--offline`/`--frozen`.
Batch mode writes a process-level `batch-events.jsonl` under the batch output by
default, so resolver/analyzer startup, heartbeats, and shared-session decisions
are visible even if no generated root finishes. The shared session records
manifest loading, selected-root package-closure parsing, and semantic analyzer
loading as separate events, which makes slow I/O, syntax parsing, and RA startup
distinguishable in mined failures. Each batch root also gets its own
`slice-events.jsonl` in the generated root folder. During generation the live
root log is written to a sibling path so it does not trip the output overwrite
guard, then it is preserved into the final root folder with the same generation,
preflight, check, repair, and final classification events. This lets multiple
mined roots be inspected independently without filtering the global batch log
first.
Rows also include rendered usage contract counts and RA semantic budget coverage
(`semantic_*` and `selected_root_*` counters), so mining can separate clean,
well-proven slices from accepted slices that still spent or skipped too much
semantic analysis. `semantic_proof_status` gives the short version:
`complete`, `selected_root_complete_workspace_limited`,
`selected_root_limited`, or another explicit failure/limited state. Under
each generated root, `slice-decision-log.json` records the top-down phases and
now includes `file_retention`, `member_pruning`, and `feedback_diagnostics`
steps. `file_retention` lists retained packages, generated targets, files
written, and sampled source files with reachable symbol counts.
`member_pruning` records retained/blocked/prunable member and assoc-item counts
plus sampled decisions, which is the quickest way to inspect whether support
packages contain only used or unknown surfaces. `feedback_diagnostics`
summarizes the generated `slice-feedback.json` report with compiler
success/error/warning counts, widening candidates, and prioritized diagnostic
spans, so failed mined roots can be triaged from the decision log without first
opening the raw cargo JSON.
`--production`, selected-root proof failures or selected-root budget limits
reject the slice before compiler feedback. Wider workspace limits remain visible
but non-fatal when the selected-root proof itself is complete.

```sh
cargo run -p opensource_cli --bin slicers -- \
  --production \
  --random-roots 10 \
  --random-root-package my_crate \
  --random-seed 42 \
  --feedback-loop 2 \
  --deny-warnings \
  /path/to/source/Cargo.toml /tmp/slicers-mine
```

The default `slicers` binary now enables the `ra-hir` feature and uses
rust-analyzer HIR semantics by default. RA-resolved project-local method and
path targets are mapped into the slicer's generic `CallableId` / `ItemId`
model and applied as additive reduction edges. The `syn` reducer still provides
the base closure and fallback behavior, so RA is part of the production path but
is not yet a full rustc-equivalent reachability oracle. RA loads local
workspace crates with dependency crates excluded by default; external
correctness is still checked by Cargo/rustc feedback.
Files containing selected `#[opensourced]` roots are analyzed first so bounded
semantic budgets are spent on the active slice before wider workspace
inventory. Batch reports expose the configured file/method/path budgets, the
unresolved/unqueried selected-root counters, and a `semantic_proof_status` label
to make budget exhaustion visible in the fast mining loop. Production validation
requires selected-root semantic proof to be complete; this keeps top-down
slicing fail-closed where RA could not inspect the actual selected root.

```sh
cargo run -p opensource_cli --bin slicers -- \
  . /tmp/slicers-ra-report
```

Use `--analyzer syn` for the fast syntactic fallback, or build with
`--no-default-features` when the RA dependency stack is not desired.

`--preflight` is the fast prediction tier. It writes `slice-preflight.json` and
validates the generated workspace without compiling dependencies: manifests,
no-build `cargo metadata --no-deps`, local path dependencies, Cargo target
sources including explicit target tables and auto-discovered
bins/examples/tests/benches, Rust syntax, external module files, and
build-script paths. `--feedback` runs this preflight first and fails before
`cargo check` when the generated shape is already structurally invalid.

`--feedback` runs `cargo check --message-format=json` against the generated
workspace, prints prioritized compiler diagnostics, and writes
`slice-feedback.json` under the slice output, including exit code, timeout
state, duration, Cargo working directory, target directory, timeout, and extra
cargo check arguments for reproducibility. Feedback Cargo stdout/stderr are
drained while the child process is running, so dependency-heavy
`--message-format=json` checks cannot deadlock the feedback loop by filling a
captured output pipe. Feedback and baseline Cargo commands run from the checked
manifest's parent directory, and generated slices preserve workspace-level
`.cargo/config.toml` or legacy `.cargo/config` files so validation sees the same
Cargo cfg, target, registry, source-replacement, and rustflag context as the
source workspace. Generated slices also preserve root `rust-toolchain.toml` or
`rust-toolchain` files so Cargo validation uses the same pinned toolchain, plus
root `[profile.*]` policy so profile-sensitive checks do not fall back to Cargo
defaults. Rendered source and asset copies resolve symlinks through the package
root: internal package links are copied at the path the generated source
expects, while links that resolve outside the package are not copied into the
slice. `--feedback-loop <n>` repeats that
compiler feedback pass up to `n` times, widens the generated slice from bounded
compiler diagnostics when they map back to known project symbols, stops early
when diagnostics repeat without progress, and fails with the JSON report path
when the slice still does not compile. Feedback checks have a 600-second timeout
by default; use
`--feedback-timeout 0` to disable it or pass another second count.
Use `--feedback-target-dir <path>` to reuse a Cargo target directory across
repeated generated slices when dependency build time dominates validation.
Use repeated `--cargo-check-arg <arg>` flags to pass feature or target matrix
arguments through to baseline and generated `cargo check` runs, for example
`--cargo-check-arg --all-features` or `--cargo-check-arg --target
--cargo-check-arg wasm32-unknown-unknown`.
In `--production` mode, selected roots behind non-test cfg gates remain
reported in `slice-report.json`; when their cfg expression is proven by the
supplied `--cargo-check-arg` feature/target values, `--all-features`, and host
or explicit target `rustc --print cfg`, the production gate downgrades them to
compiler feedback instead of failing before validation. Recognized
`all(...)`, `any(...)`, and `not(...)` expressions use fail-closed tri-state
evaluation, while custom cfgs and other error hazards remain fail-closed.
Retained non-root feature cfg surfaces also feed a bounded production matrix
pass: after the primary feedback loop, the CLI runs matching source and
generated checks for uncovered concrete feature hints and writes
`slice-baseline-matrix-N.json` plus `slice-feedback-matrix-N.json`.
Feedback reports classify unresolved compiler diagnostics into widening
candidates and production hazards, so missing paths, items, methods, crates,
module files, timeouts, and manifest-shape failures can be triaged without
reading raw `cargo check` output. Generation reports record
`feedback_widened_roots` when compiler diagnostics caused a re-render. Semantic
drift warnings that can mean pruning changed behavior, such as unreachable
patterns or missing-constant pattern bindings, are classified as feedback
hazards too.
Use `--deny-warnings` when a production validation run should reject generated
workspaces that compile with warnings.
`--feedback-repair-loop <n>` runs bounded compiler feedback with conservative
source repairs between attempts. The first repair tier only handles diagnostics
that are safe to edit mechanically, such as `unused_imports`, item-level
`dead_code`, and deferred dead-code allows for retained fields or enum variants.
It also applies rustc `MachineApplicable` suggestions when every edited span is
inside the generated output root, including whole-use unused-import removals
reported as warning help. Repair mode does not accept a successful Cargo check
until repairable warnings have been removed or exhausted. Plain feedback,
repair, and production-matrix loops stop on repeated diagnostics, low-progress
diagnostic shapes, or no-progress rounds instead of burning build time after
the retained graph stops changing.
Validation runs write `slice-validation.json` by default. That report is the
authoritative gate verdict: final status, rejection reason when present,
baseline/preflight/feedback gate states, cargo check arguments, and per-attempt
feedback or repair outcomes.
If the selected root lives in a non-default target such as an example, validation
rejects runs whose cargo check arguments do not cover that target or its
`required-features`; retained binary targets with `required-features` also
require matching feature activation because Cargo can skip them otherwise. Pass
`--cargo-check-arg --all-targets`, `--cargo-check-arg --examples`, or the
matching `--cargo-check-arg --example --cargo-check-arg <name>`, plus
`--cargo-check-arg --features --cargo-check-arg <feature-list>` or
`--cargo-check-arg --all-features` when Cargo would otherwise skip the target.

`--baseline-check` runs `cargo check --message-format=json` against the source
workspace before slicing and writes `slice-baseline.json`. By default a failing
source baseline stops the run, which prevents already-broken upstream projects
from being misreported as slicer regressions. Use `--allow-baseline-failures`
when validating against a known-broken source; generated compiler errors that
match the source baseline are then treated as a baseline-limited pass, while new
generated errors still fail feedback. The baseline Cargo target directory
defaults to a sibling of the output root so baseline compilation never makes the
slice output look user-owned before rendering starts; use `--baseline-target-dir`
to override it.

`--production` is the strict validation preset for release-style runs. It
enables baseline checking, preflight, `--feedback-repair-loop 3`, and
`--deny-warnings`, reconciles the generated `Cargo.lock`, adds `--locked` when
the source workspace has a `Cargo.lock` and the caller did not already pass
`--locked` or `--frozen`, and writes
`slice-report.json` plus
`slice-validation.json` by default. If compiler feedback widens and re-renders
the slice, production mode reconciles the generated lockfile again before the
next locked check. Production mode still allows explicit flags such as
`--feedback-repair-loop 5`, `--feedback-timeout`, `--cargo-check-arg`, and
target/report paths to override the preset where needed. Production mode fails
closed on readiness error hazards before compiler feedback and records a final
`production_ready` gate only after baseline, generation, preflight, target
coverage, and compiler feedback have accepted the slice.

This is the current production feedback layer: the normal CLI run loads
bounded rust-analyzer HIR semantics for local workspace crates, applies exact
project-local semantic edges to the retained graph, keeps the slicer fast with
the existing syntactic reducer for fallback coverage, then lets rustc give
precise diagnostics for the generated slice. The production report marks the
remaining boundary explicitly until semantic edges cover trait impl lookup,
macro-expanded inventory, cfg-active modules, generated source, and dynamic
dispatch. Expanding the RA/rustc oracle for those cases before rendering
remains the next precision step.
Generation reports fail closed on known syntactic trust hazards as well,
including retained `include!` source macros that read generated Rust from
`OUT_DIR`; other retained `include!` source macros are reported as validation
hazards because they cannot be fully validated by static path copying alone.
Static `include_str!` and `include_bytes!` paths are copied for string literals,
`concat!` literals, and `concat!(env!("CARGO_MANIFEST_DIR"), "...")` package
paths. Unknown env-driven paths, `OUT_DIR` file assets, absolute include paths,
and paths resolving outside the package fail closed. The report fails closed on
function pointer and trait-object surfaces plus retained build scripts because
they can hide callback/dispatch edges, generate source, emit link metadata, or
add asset requirements outside the static parse tree. Retained custom derives,
custom attributes, non-builtin macro invocations, and non-root `cfg`/`cfg_attr`
surfaces are feedback-gated warnings: the source is preserved, but production
acceptance requires compiler feedback for the selected matrix.
Retained local path dependencies that are not workspace
members are copied into `support/`, their own path dependency closures are
rewritten to generated-local paths, and workspace-inherited package/dependency
fields are materialized so generated manifests do not point back to the
original checkout.
Selected roots behind non-test `cfg` or `cfg_attr` gates remain
production-blocking until the exact feature/target matrix is proven by
validation arguments because the selected root itself may not exist under the
default configuration.

`--slice-report <path>` writes machine-readable generation metrics: analyzer
mode/notes, production-readiness hazards and hazard details, roots, packages,
internal graph-reachable callables/items, rendered callables/items that actually
survived in generated source, generated target metadata, feedback-widened roots,
files written, and a source map for parsed callables/items with file spans and
reachability flags. The `usage.rendered_decision_map` object is the final
generated-source used/blocked/prunable map; the older `usage.decision_map`
records the pre-render graph decision used to explain why symbols were initially
kept or rejected. During CLI validation, the `rendered_usage_contract` gate
fails the slice if any generated callable, item, member, associated item, or
trait default method is classified as anything other than `used` or
`blocked_by_unknown`. Batch/mining rows include the same rendered usage counts
and stop a root as `rendered_usage_failed` before preflight or compiler feedback
when that contract is violated. Cfg-gated root hazards include the affected root, package,
module path, source span, cfg expression, and feature-oriented Cargo argument
hints where those can be derived. It also records phase timings for analyzer
loading, manifest loading, parsing, reduction, rendering, and total generation
time. That source map is the join point for semantic analyzer edges. The generic
corpus runner carries the same production hazard fields into each JSONL row and
uses them with compiler feedback widening categories to track real-project slice
size, diagnostics, runtime, and timeout outcomes over repeated random root
selection:

```sh
scripts/corpus_feedback_loop.py \
  --source /path/to/rust/workspace \
  --output-prefix /tmp/slicers-corpus \
  --max-batches 20 \
  --validation preflight \
  --roots-per-batch 5 \
  --feedback-loop 1 \
  --deny-warnings \
  --feedback-timeout 600 \
  --report reports/corpus_feedback.jsonl
```

The corpus runner discovers package targets through `cargo metadata --no-deps`,
injects the local marker dependency only into packages selected for that batch,
restores git-backed sources before and after mutation by default, preserves
failed outputs for debugging, and appends one JSONL metrics row per batch.
Use `--roots-file scripts/corpus_cases/litter_uniffi.json` to run pinned
real-repo root selections instead of random batches; per-case
`cargo_check_args` in that JSON are merged with repeated `--cargo-check-arg`
values.
Use `--validation preflight` for fast no-build corpus exploration, rerun
interesting or suspicious cases with `--validation feedback`, and use
`--validation repair` when a corpus batch should exercise the conservative
compiler repair loop and record `slice-repair.json` metrics. Use
`--validation production` for the strict preset. Pass `--feedback-target-dir`
for feedback, repair, or production batches when running a corpus against
dependency-heavy projects, and `--deny-warnings` when corpus success must mean
warning-free compiler feedback. Use `--baseline-check` to separate source
failures from slicer failures; add `--allow-baseline-failures` for known-broken
sources where generated baseline-matching errors should be tracked as
baseline-limited passes instead of slicer regressions. Feedback validation also
treats new semantic hazard warnings, such as unreachable patterns or
non-snake-case pattern bindings caused by missing constants or variants, as
slicer failures even when general warnings are allowed.

Workspace/package discovery is backed by `cargo metadata --no-deps`, so Cargo is
the source of truth for workspace members, excludes, target entry paths,
dependency aliases, dependency kinds, and package roots.

Output roots are fail-closed: `slicers` refuses to write inside the input
workspace, refuses to overwrite non-empty directories it did not create, and
marks generated outputs with `.slicers-output` so repeat runs can replace only
known generated directories.

## Additional Generated Fixtures

`opensource_core` also has integration tests that generate temporary workspaces
with different source shapes and then run the reducer against them:

- `data_items.rs`: structs, enums, type aliases, consts, statics, associated
  constructors, body-only data items, and unit-test pruning.
- `module_reexports.rs`: external module files, nested modules, renamed
  dependencies, local aliases, reachable `pub use` items, and unreachable module
  functions.
- `trait_ufcs.rs`: explicit UFCS trait calls such as
  `<Thing as Describe>::describe(...)`, trait impl reachability, enum variants,
  tuple structs, and unreachable impl methods.
- `uniffi_mobile.rs`: a Mozilla UniFFI/application-services-shaped mobile
  bridge workspace with FFI-facing DTO structs/enums, `cfg_attr(...,
  uniffi::...)` annotations, serde workspace dependencies, object-like impl
  methods, trait calls, and an unused local diagnostics crate that must be
  removed from the slice.
- `fast_macro_use.rs`: a checked-in tiny multi-package corpus that exercises
  renamed imports with local shadows, custom derive and attribute proc macros,
  macro_rules definitions, retained include-generated source, prelude and
  module reexports, inline child imports, type aliases, const/static items,
  enums, associated trait items, trait imports, and dead item/package pruning
  without waiting on a large repository. Run `scripts/fast_fixture_loop.sh` for
  the shortest source-check plus generated-slice check loop.

See `docs/uniffi_slicer_experiment.md` for the UniFFI fixture rationale and
verified result.
See `docs/slice_coverage_matrix.md` for the full supported slice matrix and
current boundaries.

Expected reachable callables for the fixture:

```text
a::open_source_entry
b::compute
b::helper
c::adjust
d::hash
d::normalize
d::shared
d::<Worker as Transform>::transform
d::Worker::new
d::Worker::run
e::finish
e::mix
e::seed
```

Expected pruned callables include:

```text
a::internal_entry
b::unused_public
c::unused
d::Worker::unused_method
d::unused_public
d::unused_private
e::tempting_but_unused
e::unused_leaf
```

Expected reachable items include:

```text
d::Mode(Enum)
d::SALT(Const)
d::Score(Type)
d::Transform(Trait)
d::Worker(Struct)
e::DEFAULT_SEED(Const)
e::Score(Type)
```

## Current Boundaries

This branch uses a syntactic call graph as the default reducer and rustc
feedback as the correctness gate; it is not yet a full compiler frontend. It
now handles direct trait method calls when the receiver type can be inferred
locally, borrowed UFCS trait calls, contextual `into`/`try_into` conversions,
workspace member globs, external dependencies, renamed-import pruning with
scoped local shadowing, local path crate pruning, copied
non-workspace path support packages, and compiler-driven unused-import cleanup
in repair/production validation. It also scans retained macro bodies for direct
local paths, keeps inline macro-generated modules importable when retained code
uses them, and prunes external dependencies whose crate alias is absent from
the retained source. Complex function pointers, trait objects, custom proc macro
expansion, `OUT_DIR` generated Rust, broad custom cfg inventories, and
rustc-equivalent name resolution remain outside the current reducer and are
reported as production hazards or feedback-required boundaries.
