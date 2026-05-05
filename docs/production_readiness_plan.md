# Production Readiness Plan

Last updated: 2026-05-06

See also `docs/production_hardening_learnings.md` for the durable lessons from
the Litter, RTK, RA feedback, compiler-repair, manifest, and proc-macro
hardening passes. That document is the project memory for why this plan favors
a copy/prove/cut architecture over either a proc-macro-only tool or a full
rust-analyzer rewrite.

## Direction

The fastest credible route is a hybrid slicer:

1. Keep the current `syn` reducer as the fast first-pass closure engine.
2. Use Cargo's own project model for workspace, target, dependency, feature, and platform shape.
3. Require compiler feedback on generated slices.
4. Add rust-analyzer or rustc semantic queries only for unresolved high-risk edges.
5. Fail closed when the tool cannot prove a precise slice.

This is intentionally not a rust-analyzer rewrite. HIR can answer semantic
questions, but it does not solve rendering, manifest rewriting, build-script
assets, output safety, or product diagnostics by itself.

## Current Branch Status

`main` is currently a semantic-assisted CLI with a `syn`
base reducer and rustc feedback/repair as the production gate. The default
`opensource_cli` feature set enables `ra-hir`, and the CLI defaults to
`--analyzer ra-hir`; users can still pass `--analyzer syn` or build with
`--no-default-features` for the fast syntactic fallback. The `--production`
preset now defaults to `--analyzer ra-hir-proc-macros` and enables the bounded
RA outgoing-call feedback closure, but the bounded default keeps dependency
artifacts excluded and therefore skips the proc-macro load request instead of
attempting an incompatible rust-analyzer configuration. The fast `ra-hir` path
keeps proc macros disabled, excludes dependency crates from the HIR load, and
does not query call hierarchy.
`--analyzer ra-feedback` remains an explicit bounded mode for testing the same
outgoing call hierarchy closure without requesting proc-macro/build-script
discovery. Both paths record project-local call hierarchy edges into the same
additive reduction hint map, then let the existing `syn` renderer prune items
outside the retained set.
Generation reports now expose an explicit usage classification produced through
a first-class `SlicePlan` and rendered through the same `UsageDecisionIndex`:
`usage.used`, `usage.unused_candidate`, `usage.blocked_by_unknown`,
`usage.prunable`, `usage.unused`, `usage.decision_map`, and `usage.unknown`.
The decision index keeps deterministic maps from callable/item ids to
`used`, `blocked_by_unknown`, or `prunable`; the JSON `decision_map` exposes the
same map for external automation. `used` means reachable from selected roots
through the current syntactic and semantic edge map; `unused_candidate` means
indexed but unreachable from selected roots; `blocked_by_unknown` means an
unreachable item or callable was retained because a scoped unknown surface
explicitly mentions it or one of its retained dependencies; `prunable` means
graph-unreachable and not blocked by unknown surfaces; `usage.unused` is the
public removable set and is intentionally the same as `usage.prunable`, not the
broader candidate set; `unknown` mirrors production hazards that prevent
treating the classification as a complete proof.
`usage.evidence` records one explanation per indexed callable/item, including
whether it was a selected root, whether it was reachable, and whether semantic
or syntactic fallback evidence participated in the retained graph. `SlicePlan`
centralizes the final render reduction and the exact usage decisions, so source
deletion and reporting consume one shared decision object instead of recomputing
liveness independently. Unknown semantic availability warnings are reported as
unknowns but no longer globally block every unrelated unused item. Scoped
macro/include/dyn/callback hazards promote only explicitly mentioned symbols
into a pre-render retained closure. In rust-analyzer modes, the analyzer also
records whether syn-indexed callables/items mapped back to RA definitions and
runs RA reference search for mapped source symbols. The generation report now
serializes the `analyzer.semantic_usage` proof surface: mapping counts, failed
reference-query ids, referenced ids, reference owners, and unowned reference
files. Reference search uses multiple focus candidates per symbol, preferring
declaration-name offsets over earlier doc/attribute mentions before falling
back to other identifier-boundary matches, so `semantic_usage_reference_incomplete`
is reserved for symbols RA still cannot answer after retrying plausible source
positions. RA outgoing-call feedback uses the same focus-candidate retry before
adding call-hierarchy edges. RA reference owners are promoted into the same
`SemanticReductionHints` graph as method/path/call hierarchy edges, so retained
owners pull referenced callables/items into the positive `used` closure before
rendering. Unknown
retention remains a fixed-point pass over the current retained slice:
graph-unreachable candidates are promoted into `blocked_by_unknown` when they
cannot be mapped, when reference search fails, or when a retained RA reference
cannot be discharged through a positive edge. File-level unowned references are
recorded for diagnostics but do not grant retention by themselves, because impl
headers for dead sibling types can otherwise over-retain whole same-file item
sets. Only mapped graph-unreachable candidates with no retained reference
evidence are treated as prunable/removable. The production invariant is
fail-closed: remove only prunable source, and keep/report anything blocked by
unknown until compiler feedback or deeper semantics discharges it.
Semantic method/path inventory is scoped to syntactically retained callable/item
owners before spending bounded RA query budgets, so dead same-file siblings
cannot exhaust the semantic budget ahead of selected code.
Production proc-macro mode stays bounded by default because full Cargo
dependency build-artifact discovery timed out on the pinned Litter corpus; set
`OPENSOURCE_RA_PROC_MACRO_LOAD_DEPS=1` only when a workspace can afford that
heavier analyzer pass. With that opt-in enabled, rust-analyzer runs build-script
output discovery and requests the sysroot proc-macro server before collecting
HIR semantics. If rust-analyzer proc-macro loading panics or the active
toolchain does not provide a proc-macro server, the analyzer reports that
expansion is not active and continues with bounded HIR. Both rust-analyzer paths
map exact project-local method/path resolutions into generic `CallableId` /
`ItemId` reduction hints and apply those hints as additive retained-graph
edges. RA definition-mapping, reference-search coverage, and promoted reference
edge counts are reported separately from reachability so missing semantic
coverage, failed reference queries, and retained RA references that cannot be
converted to graph edges become explicit unknown-retention signals instead of
silently becoming deletion permission. Files containing selected
`#[opensourced]` roots are analyzed first so
bounded semantic budgets prioritize the active slice. The analyzer now records
per-file semantic inventory; after reduction, production readiness scopes
semantic failure/budget/unresolved warnings to retained slice files when those
file reports are available, then falls back to selected-root file counts and
finally workspace-wide counts. Production validation therefore relies on
bounded RA semantics, guarded production
proc-macro/build-script discovery, fast static fallback reduction, explicit
production hazards, and
`cargo check --message-format=json` feedback.
The final `production_ready` validation gate is intentionally stricter than
plain compiler feedback: warning-only production hazards now produce
`review_required`, and only a slice with no remaining production hazards is
marked `accepted`.

The latest codex-ipc production probe on a fresh Litter checkout now validates
the pinned five-root `scripts/corpus_cases/litter_codex_ipc_random5.json` case
with package-scoped baseline/feedback args. The first feedback pass widened six
generic missing-method roots from E0599 diagnostics, the second pass produced a
warning-clean `cargo check -p codex-ipc --locked`, and the final validation
status was `review_required` because macro/dynamic-dispatch/semantic warning
hazards remain for human or deeper-semantic discharge. This exposed and fixed
three generic infrastructure gaps: package-scoped validation for marked roots,
corpus lockfile reconciliation after marker dependency injection, and
render-plan unioning of selected, feedback-widened, and unknown-retention roots.

The latest hardening milestone validated the pinned Litter UniFFI cases in
strict repair mode with `--deny-warnings`: all three pinned roots reached zero
final warnings through the RA analyzer path, and production reports show
`semantic_reduction_hints_applied` for each pinned root after RA root-file
prioritization. The apply-snapshot case removed three unused imports through
the compiler repair loop. The feedback runner now drains Cargo stdout/stderr
while the child process runs, preventing large JSON output from dependency-heavy
checks from blocking the feedback loop.

## Inspirations

- `cargo metadata` (`https://doc.rust-lang.org/cargo/commands/cargo-metadata.html`):
  Cargo already exposes machine-readable workspace members,
  targets, resolved dependencies, dependency kinds, target filters, and feature
  selection. The slicer should stop hand-rolling this model as it becomes more
  project-agnostic.
- `cargo-machete` (`https://github.com/bnjbvr/cargo-machete`): fast source
  scanning is useful, but imprecise. The product lesson is to report confidence
  and known blind spots instead of pretending token-level evidence is semantic
  proof.
- `cargo-udeps` (`https://github.com/est31/cargo-udeps`): compiler-backed
  dependency checking is slower and more toolchain-sensitive, but it provides a
  stronger correctness signal. The slicer should use this pattern for validation
  and widening.
- `cargo-minify` (`https://docs.rs/crate/cargo-minify/latest`): source deletion
  tools need explicit write safety. Its version-control safety stance maps
  directly to slicer output handling.
- `cargo-expand` (`https://github.com/dtolnay/cargo-expand`): macro-expanded
  text is useful for inspection, but lossy. Macro expansion should be an
  oracle/diagnostic input, not the rendered source of truth.
- rust-analyzer HIR: semantic APIs are the right long-term oracle for path,
  method, trait, and macro-expanded item questions that the syntactic reducer
  cannot answer confidently.

## Pitfalls To Design Around

- Compile success can still be semantically wrong. The Litter const-pattern
  failure showed that a slice can build while changing behavior.
- Dynamic dispatch and function-pointer callback surfaces can compile while
  hiding concrete call edges. They must block unsafe pruning and keep structured
  package/module/file/line details, but they are warning/review hazards rather
  than pre-feedback hard errors: Cargo feedback validates the generated
  signatures, and final production readiness remains `review_required` until
  semantic resolution proves the retained implementation or callback set.
- Unknown macros should not be pruned silently. Either retain bounded source,
  query a semantic oracle, or fail with an unsupported-construct report.
- Retained custom derives, custom attributes, and non-builtin macro invocations
  must be preserved verbatim and require compiler feedback until
  macro-expanded items are mapped into the retained reachability graph. Local
  proc-macro crates referenced by retained source are kept whole because their
  compile-time implementation is part of the derive/attribute contract, and
  helper paths inside retained helper attributes are promoted into the
  reachability graph. The production analyzer can request rust-analyzer
  proc-macro expansion, but macro-generated source is still not rendered as
  first-class slice source. These feedback-required macro hazards include
  structured package/module/file and line details for direct retained macro
  surfaces.
- Build scripts can execute arbitrary project logic and generate source under
  `OUT_DIR`; copied `build.rs` files are not enough for semantic modeling, so
  retained `OUT_DIR` Rust includes must fail closed until a semantic oracle
  models generated source. Retained build scripts are production-blocking
  because they can also read external state or emit link/env metadata that
  changes compiled behavior. Retained build-script hazards report package and
  build-script path details.
- Retained `include!` Rust source files must fail closed even when their paths
  are static and copied, because the included Rust is outside the current
  reachability graph. Include and compile-time environment hazards report
  package/module/file/line details so a production run points at the retained
  source surface directly.
- Retained `env!` or `option_env!` macros must fail closed unless they read
  Cargo manifest-derived package metadata, because they can embed machine-local
  compile-time state.
- File include assets must be copied only for statically resolved package-local
  paths; unknown env paths, absolute paths, external paths, and `OUT_DIR`
  generated assets are production-blocking.
- Feature and platform cfgs form a matrix, not a boolean. A slice should report
  the feature/platform configuration it was generated for, and only `cfg(test)`
  should be treated as test-only pruning. Retained non-test `cfg`/`cfg_attr`
  surfaces require compiler feedback for the selected matrix; selected roots
  behind non-test cfg gates still fail closed until validation explicitly covers
  that matrix. Cfg-gated root hazards should carry root, module, source span,
  cfg expression, and Cargo argument hints so a validation matrix planner can
  discharge them intentionally instead of relying on free-text diagnostics. The
  CLI now discharges cfg-gated roots when their `cfg(...)` expression is proven
  covered by matching `--cargo-check-arg --features ...` values,
  `--cargo-check-arg --all-features`, and host/explicit-target
  `rustc --print cfg` output, including recognized `all(...)`, `any(...)`, and
  `not(...)` combinations; custom cfgs remain production-blocking. Retained
  non-root cfg surfaces also report structured
  package/module/file/cfg details so corpus and matrix planners can validate the
  selected shape without scraping warning text. The first matrix planner is
  bounded to a concrete Cargo feature union derived from retained cfg details;
  it validates a matching source baseline and generated slice after primary
  feedback, then records `production_matrix` gates before final acceptance.
- Dependency aliases, workspace dependencies, optional dependency features, and
  target-specific dependencies need Cargo's resolver model.
- Retained direct or target-specific path dependencies outside the workspace are
  copied as bounded support packages under `support/`, including transitive
  local path dependency closures and workspace-inherited manifest fields.
- Workspace `[patch]` and `[replace]` path entries are copied and rewritten when
  they point at local package roots; uncopyable path entries remain
  production-blocking and report the manifest path plus patch/replace subject.
- Whole-crate fallback must be explicit and reported with size impact.
- Output deletion must be guarded. A production CLI must only replace empty or
  previously generated output directories and must reject paths inside or above
  the source workspace.

## Milestones

1. Safety gate:
   - Mark generated output directories.
   - Refuse unmarked non-empty outputs.
   - Refuse output paths inside or containing the input workspace.
   - Enforce fmt, clippy, tests, CLI feedback, and RA smoke tests in CI.

2. Cargo metadata substrate:
   - Load package/target/dependency shape from `cargo metadata --format-version=1`.
   - Preserve manual TOML rendering only for source manifest rewriting.
   - Record target kind, source path, required/default features, dependency
     kind, cfg target, and package ID.
   - Run Cargo from each checked manifest's parent directory and preserve
     workspace `.cargo/config.toml` or `.cargo/config` in generated slices.
   - Preserve root `rust-toolchain.toml` or `rust-toolchain` files so generated
     validation does not drift to the user's default Rust toolchain.
   - Preserve root `[profile.*]` policy so generated validation keeps the
     source workspace's Cargo profile semantics.
   - Copy source/include/build assets only after resolving symlinks inside the
     package root; never copy a symlink target that escapes the package.
   - Validate production slices with Cargo `--locked` whenever the source
     workspace provides a lockfile, including after every compiler-feedback
     widening re-render.

3. Feedback widening:
   - Parse `cargo check --message-format=json`.
   - Drain Cargo stdout/stderr while the child process is running so large JSON
     output cannot fill captured pipes.
   - Classify missing item/module/import/dependency/feature failures.
   - Widen the retained graph with a bounded iteration cap.
   - Apply only rustc `MachineApplicable` suggestions whose spans are inside
     the generated output root, including whole-use unused-import help.
   - Keep feedback repair running until repairable warnings are removed or
     exhausted before accepting a warning-bearing successful check.
   - Treat selected warnings, especially unreachable patterns, as semantic hazards.

4. Semantic oracle:
   - Keep rust-analyzer HIR in the default CLI/corpus flow and
     rust-analyzer proc-macro/build-script discovery in the production preset.
   - Continue applying exact project-local RA method/path edges as additive
     reduction hints.
   - Add narrow queries for path resolution, method resolution, trait impl lookup,
     macro-expanded item inventory, and active cfg file/module inventory.
   - Keep the syntactic reducer available for simple workspaces and offline runs.

5. Corpus validation:
   - Pin real repositories and root selections with repeatable corpus case files.
   - Run original baseline checks before slicing.
   - Generate slices, run feedback checks, and compare selected behavior probes.
   - Store retained/pruned inventories and failure reports.

## Current Implementation Summary

This branch has moved beyond the initial safety gate. It now has guarded output
replacement, Cargo metadata-backed workspace/target/dependency discovery,
preflight validation, compiler feedback widening, conservative repair,
production validation gates, pinned corpus cases, documented production hazard
reporting for known unsupported surfaces, and additive rust-analyzer semantic
edges in the retained graph. The main unfinished production step is expanding
the semantic oracle from exact local method/path edges into trait impl lookup,
macro-expanded item inventory, active cfg/module inventory, generated source,
and dynamic dispatch surfaces.
