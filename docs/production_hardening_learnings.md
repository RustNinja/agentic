# Production Hardening Learnings

Last updated: 2026-05-05

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
only, callback root only, data-item root only, enum root only, trait-item root
only, macro-exported object root only, path-qualified derive record root,
derive-helper error enum root, Arc-returning object root, stored callback
registry object root, callback-registry function root, future-callback alias
root, private inline facade-object reexport root, macro-helper reexport root,
conversion roundtrip root, generic-header/where-clause root, trait-edge root
only, cfg/asset root only, associated-codec trait item root only, combined
macro+async roots, and combined macro+trait+cfg roots. That caught several
important issues: when a trait is selected as a root item, its method
declarations and their imports are part of the public slice and must be
preserved; retained inline modules that contain `include!(concat!(env!(
"OUT_DIR"), ...))` must still raise an OUT_DIR generated-source production
hazard even when the inline module is retained only because reachable code
mentions the module path; selected item roots with macro-bearing inherent impls
must retain exported constructors/methods and the dependencies from those impl
signatures and bodies even when the object lives behind an inline private
facade; rendered struct/trait/impl headers must keep local generic and
where-clause bounds; and path-qualified proc macro derives should not keep
unused simple imports only because the derive leaf appears in a qualified path.

The dynamic-dispatch rule is now split by ownership. Direct callback inputs on
the selected API boundary, such as `fn(...)` parameters and borrowed
`&dyn Trait` parameters, do not hide project-local implementation code and are
allowed to proceed to compiler feedback. Owned, returned, stored, or aliased
dynamic surfaces remain hard production hazards because they can hide concrete
project-local behavior that the static call graph has not proven.

Macro expansion warnings should identify the exact retained macro surface, not
only the module that contains it. Custom attribute, custom derive, nested helper
attribute, and non-builtin macro invocation hazards now put the macro path in
the hazard subject, so a macro-heavy Litter/UniFFI slice can be triaged without
manually opening every retained source line.

The rule database should grow as small named generic cases, not as one giant
fixture or as project-specific allowlists. The first seed rules cover grouped
public reexport pruning, removed alias pruning with local binding shadows,
owned/stored dynamic dispatch hazards, retained `include_bytes!` assets,
build-script `rustc-env`/`env!` blockers, nested callback/future aliases, and
`pub(crate)` macro helper reexports. Litter examples should be converted into
these generic groups before any slicer behavior is changed.

The generated rule catalog is the scalable backlog for that work. It validates
1,200 generic rule records across import, macro, trait, dyn, include, build,
UniFFI, manifest, repair, and cfg axes without naming Litter or any other real
project. A catalog entry becomes an executable fixture only when it protects a
distinct reducer, renderer, manifest, or production-gate behavior.

The second rule-database pass added eight more non-cfg shapes and exposed two
generic reducer issues. Retained `macro_rules!` bodies can call methods on
metavariables, so the reducer now links `$receiver.method()` patterns to the
types of retained invocation arguments. External constructor-style calls still
retain trait impl support for argument types, but that fallback must exclude
conversion-like traits; `.into()` and `.try_into()` have precise conversion
retention paths, and broad argument fallbacks otherwise pull unrelated
`TryFrom<Other> for Target` impls into the slice.

The third rule-database pass raised the fast rule set to 19 cases and confirmed
existing generic behavior for four more Litter-shaped surfaces: direct borrowed
`dyn Trait`/`fn(...)` API boundaries stay warning-only, retained source
`include!` macros stay production-blocking, dependency aliases mentioned only in
macro bodies are retained, and inline script-bundle modules copy only live
`include_str!` assets.

The fourth rule-database pass raised the executable fast rule set to 22 cases and
fixed two graph/render gaps generically. Retained surfaces must expand renamed
local aliases back to their resolved target item names before render planning;
otherwise a struct field typed as `PublicAlias` can keep the alias text but prune
the real `LongName` definition. Public local trait impls for reachable types are
part of the retained type surface, and receiver calls to trait default methods
must retain the trait item even when the concrete impl only supplies associated
consts or types. Inline callback/future trait-object fields now share the same
hard dynamic-dispatch hazard path as aliased callback/future signatures.

Five real Litter `codex-ipc` probes now pass production validation with compiler
feedback: `project_conversation_state`, `Method::from_wire`,
`PendingRequests::resolve`, `read_frame`, and `IpcBridge::new`. The compact
roots generated 12-15 files, while the bridge/state roots generated 850+ files.
That means correctness is improving, but support-package/path-dependency
precision is now a high-priority production-size problem. See
`docs/litter_probe_report.md` for the concrete run notes.

The first support-package precision fix is manifest/source-tree bounded.
Generated support packages now drop dev-dependencies, test/example/bench targets,
and build-dependencies when the support package has no build script. No-build
support packages copy their library source tree plus statically referenced
include assets instead of the whole package root, which removes dead examples,
tests, benches, fixtures, and dev/build-only path packages without slicing the
support package semantically yet.

The next support-package precision fix made no-build support copying structural
instead of directory-wide. The copier now walks the library external-module graph
from `src/lib.rs`, skips `#[cfg(test)]` external modules and orphan Rust files,
copies only static include assets referenced by copied support modules, and falls
back to the broader source-tree copy only when the support module graph cannot be
parsed safely. The pinned Litter codex-ipc five-root corpus run stayed
warning-clean under feedback with `--deny-warnings` and dropped from 121 to 116
generated files by removing generic support test/tool files.

Copied support packages are now production-reporting surfaces, not invisible
blobs. After rendering, the production gate scans generated `support/` packages
for retained build scripts, retained Rust `include!`, `OUT_DIR` source/file
includes, unresolved/absolute/external file includes, and non-modeled
`env!`/`option_env!` calls. This is intentionally narrower than scanning support
dependencies for dynamic-dispatch internals: support package internals are still
validated by Cargo, while generated-source and environment inputs can make the
slice depend on machine-local state.

The next Litter codex-ipc five-root corpus probe exposed a more precise import
bug: a module-local `use serde::Serialize` and a selected trait import survived
only because other reduced items in the same package mentioned those symbols.
The renderer now builds import liveness from the actual render plan, not every
reduced-but-unrendered item, and local trait imports are retained only when the
module still uses the trait name or calls a method that needs the trait in
scope. The rerun produced zero feedback warnings.

The fast macro/use fixture then exposed the same class in a proc-macro-shaped
form. A path-qualified derive such as `macro_helpers::FixtureRecord` should keep
the proc-macro crate but not retain `use macro_helpers::FixtureRecord`; import
pruning now distinguishes unqualified import uses from qualified paths. The
same pass added item-root macro impl closure: when a selected struct/object has
a retained macro-bearing inherent impl, the reducer and renderer include the
impl methods plus their referenced fields, return types, callback traits,
statics, and helper imports. This is generic UniFFI/object behavior, not a
fixture-symbol allowlist.

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
