# Production Hardening Learnings

Last updated: 2026-05-04

This document captures the implementation lessons from moving the slicer from a
small proof-of-concept toward a production Rust workspace slicer. It is written
as durable project memory: decisions here should guide future work before adding
new slicer-specific patches.

## North Star

The production architecture is copy, prove, cut:

1. Copy a Cargo-shaped workspace slice with the selected roots and conservative
   support closure.
2. Ask semantic oracles and the Rust compiler what the slice still needs.
3. Cut only the code that is proven unused, then validate with compiler feedback.

The fastest credible route is not a full rewrite around rust-analyzer. The
`syn` reducer is still valuable because it owns source rendering, manifest
rewriting, output safety, and fast deterministic structure. Rust-analyzer and
rustc should be used as semantic oracles that add edges, validate the result,
and block unsafe claims.

## What Worked

- A `syn`-first graph can produce useful small slices quickly when it is paired
  with compiler feedback.
- RA semantics are best used additively. Exact local method/path edges from RA
  should widen the retained graph, not replace the reducer's render model.
- Compiler diagnostics are the strongest convergence signal. Missing items,
  unresolved imports, unused imports, dead code, and malformed path remnants can
  be converted into generic repairs or graph widening.
- Real repositories are mandatory. The pinned Litter and RTK runs exposed issues
  that small fixtures did not cover: package patch propagation, workspace-level
  support assets, parent imports consumed through child `use super::*`, format
  string captures, target cfgs, and macro fallout.
- Production readiness must be a gate, not a slogan. A slice that compiles but
  still has unresolved semantic hazards should be `review_required`, not
  `accepted`.

## What Did Not Work

- Waiting for full builds as the only feedback loop is too slow. The tool needs
  preflight, static prediction, repair loops, and targeted Cargo check args.
- Hardcoded fixes for a named project do not scale. Each Litter failure had to
  become a generic rule, regression fixture, or documented fail-closed hazard.
- Treating proc macros as normal unused imports is unsafe. Derives and attribute
  macros create hidden compile-time and generated-code dependencies.
- RA alone does not solve slicing. It does not render source, rewrite manifests,
  copy assets, preserve Cargo policy, or guarantee that generated code has been
  mapped back into editable source.
- A failing real-repo slice does not always mean slicer failure. The
  second-largest Litter module hit API drift that also failed in the original
  marked source checkout; baseline comparison is required before blaming the
  generated slice.

## RA Feedback Lessons

The `ra-feedback` branch proved that a copy/prove/cut loop is viable:

- selected-root files should be analyzed first;
- syntactic retained owner files are the next best RA query scope;
- exact project-local call hierarchy/path targets map cleanly into
  `CallableId` and `ItemId` semantic hints;
- RA hints must remain additive so the reducer can keep conservative fallback
  behavior;
- Cargo/rustc feedback remains the final truth for acceptance.

The main missing RA work is deeper macro-expanded inventory, build-script
generated source mapping, cfg-active module inventory, and rustc-equivalent
trait/dynamic dispatch resolution.

Production proc-macro mode is intentionally two-tiered. The default
`ra-hir-proc-macros` analyzer keeps dependency artifacts excluded, so it now
skips the proc-macro/build-script output load instead of first attempting a
rust-analyzer configuration that can panic when dependency artifacts are absent.
Set `OPENSOURCE_RA_PROC_MACRO_LOAD_DEPS=1` only for workspaces that can afford
full Cargo artifact discovery; that path can reduce unresolved RA queries on
small fixtures, but it was too expensive for pinned Litter runs.

## Macro Lessons

Retained macro-bearing items must carry their macro contract:

- keep retained custom derives and custom attributes verbatim;
- retain source-mentioned local proc-macro crates whole because their
  compile-time implementation is part of the selected code path;
- promote helper paths inside retained helper attributes into the graph;
- keep macro definitions when retained macro invocations need them;
- retain item macro invocations when their generated identifiers feed reachable
  code;
- treat retained inline modules that contain macro-generated source as rendered
  module targets when reachable code imports the module by name;
- still require compiler feedback until expanded items can be mapped into the
  retained source graph.

This intentionally favors correctness over minimality. A missing macro helper
usually breaks the slice; an extra proc-macro crate is acceptable production
overhead until expanded-source mapping is reliable.

## Manifest And Support Package Lessons

Cargo shape is part of the program:

- preserve root lockfiles, toolchain files, `.cargo/config*`, profiles, lints,
  relevant patches, and replace tables;
- copy retained non-workspace path dependencies into generated `support/`
  packages when they resolve to package roots;
- copied support packages need their own path dependency closure and patch names
  propagated to the generated root manifest;
- copied support source can reference workspace-level assets through
  `include_str!`/`include_bytes!`; those assets must be copied relative to the
  generated support layout;
- target-specific dependency tables and required features must be validated
  against the actual Cargo check arguments.

Manifest pruning should be driven by rendered source usage, retained features,
build-script needs, and copied support source. If a path dependency cannot be
copied or proven unused, production must fail closed.

## Import And Source Repair Lessons

Generic source repair wins came from recognizing patterns, not projects:

- Rust 2021 format string captures mention identifiers even when they appear
  only inside string literals.
- Trait imports can be required only for associated functions such as
  `FromStr::from_str` or `Deserialize::deserialize`.
- Parent imports can be consumed by child modules through `use super::*`; child
  retained code has to be considered before pruning the parent import.
- Renamed imports need scoped local-binding checks. A local variable, closure
  argument, match binding, or loop binding with the same visible name must not
  make a removed import look reachable, and a short inner shadow must not hide a
  later real import use.
- Macro token scanning needs the same scope model. A single identifier inside a
  retained macro invocation should be treated as a local value when a visible
  local binding exists, while the local value's known receiver type can still
  feed method dependencies used inside the macro tokens.
- Once a local `use` target resolves to a removed callable or item, public-name
  and alias-name heuristics must not resurrect that import. Keep only imports
  whose resolved target is retained or whose target cannot be proven local and
  removed.
- Compiler-reported unused imports should be repaired before accepting
  warning-clean production output.
- Repeated malformed absolute path remnants such as `::Type` or `::::{...}` are
  structural repair signals, not reasons to special-case a source package.
- Exact repeated diagnostics and repeated diagnostic shapes now stop feedback,
  repair, and production-matrix loops early with structured reports. This keeps
  repeated builds from consuming time when the graph has stopped changing.

## Real-Repo Corpus Lessons

The useful corpus pattern is:

- one small selected function, run multiple times;
- two selected functions in the same module;
- three mixed selected items;
- wider real modules;
- pinned large real repositories with known-good source baselines;
- strict mode with `--deny-warnings`, `--feedback`, repair loops, and explicit
  package/target Cargo args.
- one tiny checked-in stress workspace for fast macro/use iteration, so common
  graph and render regressions are caught before running a large repository;
- one scriptable short loop that checks the stress workspace source and the
  generated slice, so production-hardening changes do not require a Litter run
  for first feedback.

For Litter-like UniFFI/mobile crates, the most important acceptance checks are:

- no missing roots or support packages;
- generated slice compiles with the same target package as the original check;
- no warning debt under `--deny-warnings`;
- UniFFI/serde/macro surfaces are retained only when selected roots require
  them;
- baseline failures are separated from generated-slice regressions.

The 2026-05-04 Litter scan found eight recurring edge classes worth keeping in
the short feedback corpus before running wider Litter checks:

- UniFFI-shaped derives and export attributes on records, enums, objects,
  constructors, and async methods;
- serde helper attributes on retained DTO fields and tagged/renamed enums;
- async/channel-heavy entrypoints whose selected root pulls support methods;
- platform `cfg` and `cfg_attr` module boundaries;
- `From`/`TryFrom` conversion chains between wire and domain types;
- reexport barrels, renamed imports, and local shadows;
- macro/include surfaces that must keep generated source and macro definitions;
- callback/function-pointer signatures that compile but remain production
  hazards unless concrete dispatch is proven.

Those patterns now have a lightweight replica in `fixtures/fast_macro_use` and
should be exercised with `scripts/fast_fixture_loop.sh` before waiting on a
large Litter build.

The fast fixture should keep growing as a matrix, not as one monolithic
all-roots check. Current slice angles include macro-heavy root only, async root
only, callback root only, data-item root only, trait-item root only, and
combined macro+async roots. That caught an important API-surface issue: when a
trait is selected as a root item, its method declarations and their imports are
part of the public slice and must be preserved; when a trait is only retained
as an object type surface, method-only dependencies can still be pruned.

The dynamic-dispatch rule is now split by ownership. Direct callback inputs on
the selected API boundary, such as `fn(...)` parameters and borrowed
`&dyn Trait` parameters, do not hide project-local implementation code and are
allowed to proceed to compiler feedback. Owned, returned, stored, or aliased
dynamic surfaces remain hard production hazards because they can hide concrete
project-local behavior that the static call graph has not proven.

## Current Production Boundaries

These are intentional fail-closed areas:

- proc-macro expansion can be requested, but expanded items are not yet rendered
  as first-class source;
- build scripts and `OUT_DIR` generated Rust are copied/reported but not fully
  semantically modeled;
- owned/returned/stored `dyn Trait` values, local function-pointer type
  surfaces, callback registries, and broad dynamic dispatch are not proven
  callgraph edges;
- complex/custom cfg inventories still need deeper rust-analyzer or rustc
  oracle coverage;
- external dependency internals are trusted through Cargo/rustc validation
  rather than sliced.

## Next Best Work

The next highest-value work is to make semantic feedback cheaper and more
precise:

1. Add persistent per-workspace RA query caching for selected-root and retained
   owner files.
2. Map macro-expanded item inventory into retained graph edges when RA provides
   stable spans.
3. Model `OUT_DIR` generated Rust includes through build-script output
   discovery and generated-file copying.
4. Add a pinned real-repo CI matrix with Litter small, medium, and wide module
   targets plus RTK smoke targets.
5. Promote recurring low-progress reports into preflight graph rules so common
   compiler failures are predicted before the first full build.

The rule for future work: if a real-repo failure is fixed, add a generic
fixture that proves the rule without naming that repo.
