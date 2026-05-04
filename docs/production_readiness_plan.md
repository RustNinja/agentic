# Production Readiness Plan

Last updated: 2026-05-04

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

`codex/production-hardening` is currently a semantic-inventory-first CLI with a
`syn` reducer and rustc feedback/repair as the production gate. The default
`opensource_cli` feature set enables `ra-hir`, and the CLI defaults to
`--analyzer ra-hir`; users can still pass `--analyzer syn` or build with
`--no-default-features` for the fast syntactic fallback. The rust-analyzer path
loads local workspace crates with dependency crates excluded by default, and is
still report-only semantic inventory today; it does not yet drive the retained
graph. Production validation therefore relies on bounded RA inventory, fast
static reduction, explicit production hazards, and
`cargo check --message-format=json` feedback.

The latest hardening milestone validated the pinned Litter UniFFI cases in
strict repair mode with `--deny-warnings`: all three pinned roots reached zero
final warnings through the default `ra-hir` analyzer path, and the
apply-snapshot case removed three unused imports through the compiler repair
loop. The feedback runner now drains Cargo stdout/stderr while the child process
runs, preventing large JSON output from dependency-heavy checks from blocking
the feedback loop.

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
  hiding concrete call edges; they must fail closed until semantic resolution
  proves the retained implementation or callback set. These hazards now include
  structured package/module/file/line details so users and corpus triage can
  find the exact retained dynamic surface.
- Unknown macros should not be pruned silently. Either retain bounded source,
  query a semantic oracle, or fail with an unsupported-construct report.
- Retained custom derives, custom attributes, and non-builtin macro invocations
  must be preserved verbatim and require compiler feedback until macro
  expansion feeds the reachability graph. These feedback-required macro hazards
  now include structured package/module/file/line details for direct retained
  macro surfaces.
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
   - Keep rust-analyzer HIR in the default CLI/corpus flow.
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
production validation gates, pinned corpus cases, and documented production
hazard reporting for known unsupported surfaces. The main unfinished production
step is consuming semantic oracle edges from rust-analyzer HIR or rustc for
unresolved high-risk constructs instead of using the oracle as report-only
inventory.
