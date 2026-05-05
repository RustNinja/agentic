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

That feedback closure is now part of the production default analyzer path:
`--production` still requests `ra-hir-proc-macros`, but it also queries
rust-analyzer outgoing call hierarchy and feeds those edges into the same
generic retained-edge map. `--analyzer ra-feedback` remains useful as the
bounded call-hierarchy path without proc-macro/build-script discovery.

The main missing RA work is deeper macro-expanded inventory, build-script
generated source mapping, cfg-active module inventory, and rustc-equivalent
trait/dynamic dispatch resolution.

Production proc-macro mode is intentionally two-tiered. The default
`ra-hir-proc-macros` analyzer keeps dependency artifacts excluded, so it skips
the proc-macro/build-script output load instead of first attempting a
rust-analyzer configuration that can panic when dependency artifacts are absent,
while still keeping RA feedback closure enabled for project-local source.
Set `OPENSOURCE_RA_PROC_MACRO_LOAD_DEPS=1` only for workspaces that can afford
full Cargo artifact discovery; that path can reduce unresolved RA queries on
small fixtures, but it was too expensive for pinned Litter runs.

## Macro Lessons

Retained macro-bearing items must carry their macro contract:

- keep retained custom derives and custom attributes verbatim;
- retain only the exported proc-macro derives, attributes, and function-like
  macros referenced by reachable rendered surfaces;
- keep local helper library crates used by retained proc-macro exports as
  pruned `support/` packages instead of promoting them into the root slice;
- promote helper paths inside retained helper attributes into the graph;
- keep macro definitions when retained macro invocations need them;
- retain item macro invocations when their generated identifiers feed reachable
  code;
- treat proc-macro attributes on rendered module boundaries and inline impl
  blocks as part of the macro contract when nested reachable code or reachable
  methods force that surface to render;
- treat retained inline modules that contain macro-generated source as rendered
  module targets when reachable code imports the module by name;
- still require compiler feedback until expanded items can be mapped into the
  retained source graph.

This intentionally favors correctness while tightening minimality. A missing
macro helper usually breaks the slice, but dead exported proc macros and dead
helper crates are now treated as removable when no reachable rendered surface
mentions them.

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
- Real compiler feedback widening must be part of the retained render plan, not
  just the first reduction. A 2026-05-06 Litter codex-ipc run exposed that
  unknown-retention re-rendering could drop feedback-widened roots when both
  paths were active. The fix is generic: render from the union of selected
  roots, compiler-widened roots, and unknown-retention roots, then classify
  usage from the same `UsageDecisionIndex`.
- Dynamic dispatch and callback surfaces should block unsafe pruning, but they
  should not stop before compiler feedback. They are now warning/review hazards:
  retained `dyn Trait` and function-pointer surfaces keep nearby unknowns
  fail-closed, the generated signatures are checked by Cargo, and final
  production status stays `review_required` until deeper semantics discharges
  the warning.

## Real-Repo Corpus Lessons

The useful corpus pattern is:

- one small selected function, run multiple times;
- two selected functions in the same module;
- three mixed selected items;
- wider real modules;
- pinned large real repositories with known-good source baselines;
- strict mode with `--deny-warnings`, `--feedback`, repair loops, and explicit
  package/target Cargo args.
- production corpus runs need at least two feedback iterations so a first-pass
  compiler diagnostic can widen roots and a second pass can verify the
  regenerated slice;
- when the corpus runner mutates a temp source manifest to add the local
  `opensourced` marker dependency, it must reconcile the temporary lockfile
  before invoking the CLI production preset, because the preset validates with
  `--locked`;
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

The generated rule catalog is the scalable backlog for that work, and it now
executes the full 1,200-row catalog as real generated slices. The harness batches
about 50 rows per temporary workspace, runs `opensource_core::generate()` on
each batch, and checks unique live/dead sentinels, copied/omitted assets,
production hazard codes, and `syn` parseability of rendered Rust. It covers
import, macro, trait, dyn, include, build, UniFFI, manifest, repair, and cfg axes
without naming Litter or any other real project. A catalog entry still becomes a
focused cargo-checked fixture only when it protects a distinct reducer,
renderer, manifest, or production-gate behavior.

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

The fifth rule-database pass raised the executable fast rule set to 26 cases
without adding project-specific behavior. Four Litter-shaped non-cfg surfaces are
now locked as fast rules: `LazyLock` static initializer closures, let-else slice
patterns over enum variants, const-to-const chains with const array lengths, and
macro metavariables used inside enum variant paths. The let-else and macro rules
also confirmed an important minimality boundary: public enum variants remain part
of the public API surface, while unrelated helper functions and dead sibling
consts are still pruned.

The sixth rule-database pass raised the executable fast rule set to 30 cases and
added one renderer fix. Serde helper strings are now locked for both
`serialize_with`/`deserialize_with` and `default = "path"` forms; these were
already handled by generic attribute string-path scanning, and the rules keep
that behavior from regressing. `async_trait` trait-object APIs remain a hard
dynamic-dispatch hazard and intentionally keep only the object type surface until
semantic dispatch proves concrete callees. Foreign `extern "C"` blocks now prune
dead sibling declarations by retained symbol name instead of keeping the whole
foreign block whenever one native call is used.

The seventh rule-database pass raised the executable fast rule set to 36 cases.
The important fixes were generic graph improvements rather than symbol
allowlists: zero-argument item macro invocations can be retained from the macro
definition body when they generate names used by retained code; path dependency
trait imports can be retained by reading source trait method names when the trait
name does not predict the method; derive impl closure now descends into array,
slice, and pointer field types; associated-type equality bounds propagate
`Self::Assoc` method returns into later calls; tuple destructuring locals inherit
return element types; and generated support package include paths compare
canonical roots before reporting external-file hazards. The support bundle rule
also verifies relative support manifest rewrites and dead support bins,
examples, tests, benches, fixtures, and orphan modules stay pruned after the
original external packages are moved away.

The eighth rule-database pass raised the executable fast rule set to 44 cases.
It hardened type propagation through places where Rust code naturally introduces
new local names: typed destructured function parameters, tuple and struct local
destructuring, struct patterns in match/if-let, for-loop item bindings,
free-function closure parameters, and `Option`/`Result` closure combinators.
These are generic receiver-inference fixes and do not rely on project-specific
names. The same pass added serde flatten contract-field pruning coverage and a
nonstandard support package `[lib] path` fixture, so support copying honors the
actual library root instead of assuming `src/lib.rs`.

The ninth rule-database pass raised the executable fast rule set to 47 cases.
Serde `with = "module"` attrs now retain the module's `serialize` and
`deserialize` helpers generically, not only direct string function paths.
External trait imports that are commonly derive-only, such as
`serde::Serialize`, are still pruned for pure derives but stay when retained
helper bodies call their trait methods. Foreign `extern` retained-item liveness
now contributes unqualified type imports, so `extern` statics keep required ABI
types while dead sibling foreign items stay pruned. Returned trait objects are
locked as hard dynamic-dispatch hazards.

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

The next support-package precision fix adds bounded item-level pruning for
no-build support library roots when retained source names the exact dependency
public item, such as `external_helper::decorate` or `use external_helper::Live`.
The support copier transforms the library root, keeps only the named item and
simple local item/module dependencies, computes dependency usage from the
transformed support source, and removes support manifest path dependencies used
only by pruned dead items. Build-script packages, opaque module graphs, missing
named roots, and unsupported reexport shapes still fall back to the broader
module-level support copy.

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

The next rule-database pass grew the executable fast set to 54 cases. It fixed
three reducer graph gaps: `Result` error combinators now bind the real error
payload even when the ok type is primitive; project-local generic methods that
accept closures substitute impl generics from the concrete receiver before
walking closure bodies; and trait impls rendered by public self-type/trait
surface reachability now retain impl header, associated item, and body
dependencies. The same pass tightened production reporting for fallback-retained
inline modules by running the full syntactic hazard visitor there, so plain
`include!`, file includes, and compile-time env macros cannot bypass the
production gate.

The follow-up Litter-shaped fixture pass grew the executable set to 58 cases
without needing repo-specific code. Existing generic rules already handled
private serde `deserialize_with` wire DTOs, `skip_serializing_if =
"Option::is_none"` contract fields, untagged serde enum variants, and
`OnceLock::get_or_init` singleton initialization chains. Keeping these as
executable fixtures matters because they are common UniFFI/Litter-adjacent
shapes and now stay in the fast loop.

The next pass grew the executable set to 64 cases and fixed a production
reporting false positive: syntactic hazard scanning now mirrors rendered
private-field pruning for struct surfaces, so a private `Box<dyn Fn()>` field
that is not present in the output cannot block production readiness. The same
pass added module-scoped import liveness, private serde alias wire contracts,
nested callback-store trait-object hazards, auto-trait object casts, and
qualified `serde_json::json!` macro dependency coverage.

The next executable-rule pass grew the set to 72 cases. It fixed the same
render-vs-production mismatch for type-only trait surfaces: trait methods that
the renderer prunes no longer create false function-pointer hazards. It also
added Litter-shaped fast coverage for `option_env!` blockers, implicit
`format!("{CONST}")` captures, bare `dyn Fn` aliases stored through
`OnceLock<Arc<_>>`, serde transparent records, tagged/default enum contracts,
and bidirectional `From` conversion roundtrips.

The following macro/struct pass grew the executable set to 76 cases and fixed
three reducer/render over-retention paths. Item macros are now tested by their
lexical definition and fixed generated identifiers, while metavariables and
macro fragment specifiers such as `ident`/`expr` do not count as live names.
Inline-module item macro invocations are scanned recursively, same-named
`macro_rules!` definitions in sibling modules resolve by scope, and private
generic fields are pruned when another retained field already keeps the type
parameter used.

The next alias-shadowing pass grew the executable set to 77 cases and fixed a
render-plan under-retention bug. Generic mention collection must not discard
ordinary identifiers such as `Result`, `Option`, `Vec`, or `Box`, because real
projects often define local aliases with those names. Macro fragment/noise
filtering now stays scoped to macro-definition item-name extraction, while
normal Rust signatures and bodies keep every identifier available to the local
item resolver.

Validation also exposed the opposite pressure on struct surfaces: full-item
token scans can revive private dead-field dependencies through dead sibling impl
methods. The reducer now walks only rendered struct fields for item dependency
closure, then adds a separate reachable-field dependency pass for fields
actually mentioned by retained callables. The renderer mirrors this by checking
field mentions only inside impl items that will render. Non-test `#[cfg(...)]`
fields remain retained and reported as feedback hazards instead of being
silently pruned, while `#[cfg(test)]` fields stay test-only.

The next minimality pass applied the same discipline to trait and manifest
surfaces. Non-root traits that are retained only as type/object surfaces now
contribute only their type surface dependencies, required members, and members
explicitly referenced by reachable callables or retained macro impl surfaces.
Rendered trait impls keep required and reachable members but prune unrelated
optional default-method overrides and their private dependencies. On the Cargo
side, local build-dependencies are kept only when the generated package still
has a retained build script, and local dependency edges/features are rendered by
edge usage instead of by package-wide retention. A package can remain in the
slice because another root needs it without forcing unrelated packages to keep a
dead dependency edge to it.

Proc-macro import pruning is now split by use form. Path-qualified derives still
keep the proc-macro package without keeping dead simple imports, while retained
unqualified custom attributes keep the import that brings the attribute macro
into scope. This avoids both common failures: deleting `use helper::attr` while
`#[attr]` remains, or retaining `use helper::Derive` only because
`#[derive(helper::Derive)]` appears in a retained item.

The next anti-over-retention pass tightened two high-noise generic paths.
Resolved trait peers now include the receiver type and trait input type paths in
their key, so `Trait::work` on a live type no longer retains same-trait
implementations for unrelated receiver or input types. Source-mentioned local
dependency package retention now distinguishes path/import prefixes from
ordinary identifiers: `payload::Thing` or `use payload::Thing` can retain the
local `payload` package, while `let payload = ...` cannot. Unqualified import
aliases still participate when they resolve to a dependency, which keeps
derive/attribute macro imports such as `use helper::Derive; #[derive(Derive)]`
working without reintroducing local-variable false positives.

The usage classifier now has a concrete center of gravity. `SlicePlan` builds
the final render reduction and the `UsageDecisionIndex` together, so the report
and renderer share the same used/blocked/prunable decision object. Unknown
retention is explicit input to that plan: scoped macro/include/dyn/callback
hazards add named unreachable symbols as blocked roots, and rust-analyzer
definition-mapping gaps, failed reference searches, and retained RA references
now block otherwise prunable candidates instead of silently deleting code the
semantic oracle failed to prove removable.
The public `usage.unused` field now matches `usage.prunable`, so automation
that consumes the report sees only the removable set. The broader
graph-unreachable set remains available as `usage.unused_candidate`, while
`usage.blocked_by_unknown` records retained candidates whose removal could not
be proven safe. `UsageDecisionIndex` also keeps a deterministic per-id decision
map, and the report exposes it as `usage.decision_map` so downstream automation
does not need to reconstruct decisions from parallel arrays. The report also
serializes `analyzer.semantic_usage`, including mapped ids, failed
reference-query ids, reference owners, and unowned reference files, so the
used/unused/unknown split can be audited without scraping analyzer notes.

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
3. Extend copied support-package item pruning deeper into monolithic protocol
   files and transitive support reexport chains. The current no-build support
   path prunes root and child-module items plus simple, transitive,
   local-glob, and explicit dependency-crate public facade reexports/imports
   when retained source names concrete dependency public symbols, and drops impl
   blocks whose local self type was pruned; remaining broad fallbacks are
   unresolved relative paths, external dependency glob reexports/imports,
   unsupported glob prefixes, build-script packages, generated-source packages,
   and macro-expanded support internals.
4. Model `OUT_DIR` generated Rust includes through build-script output
   discovery and generated-file copying.
5. Add a pinned real-repo CI matrix with Litter small, medium, and wide module
   targets plus RTK smoke targets.
6. Promote recurring low-progress reports into preflight graph rules so common
   compiler failures are predicted before the first full build.

Fast fixture expansion added a Litter-shaped dependency type barrel with live
and dead dependency reexports. While adding it, a separate reducer gap surfaced
and was fixed: private inline modules whose only retained content is a public
local glob could be kept in slices that did not reference the module, then lose
the child module that the glob pointed at. Normal glob-prefix pruning now treats
inline child modules as module targets instead of only recognizing file-backed
modules.
The same fixture now covers renamed dependency type reexports used through
same-package barrel paths. Render planning records public reexport visible names
that retained code actually references, resolves those reexports to their
concrete dependency items, and keeps those target items before use-tree pruning
runs. That lets strict pruning remove dead sibling aliases while preserving live
aliases such as `SharedRecord as BarrelRecord` and the private imports that
consume them.

Multi-hop dependency barrels need the same treatment for functions, not only
types. A live call such as `local_facade::score(...)` may resolve through an app
barrel into a support package prelude and then into a private support helper
module. The reducer now follows alias chains recursively for free-function calls,
so the exact live helper remains reachable while dead helper reexports and dead
leaf dependency items still prune away.

The rule for future work: if a real-repo failure is fixed, add a generic
fixture that proves the rule without naming that repo.

The fast fixture now also mirrors Litter-style derive-helper attributes that
behave like serde but are not literally named `serde`. The reducer treats
attribute paths ending in `_serde` as serde-helper carriers for string path
closure, so retained fields with `with = "module"` keep `module::serialize`
and `module::deserialize`, and direct helper strings such as
`deserialize_with = "module::parse"` or `default = "crate::module::empty"` keep
only the named helper functions. Dead sibling helpers remain pruned.

The shared-runtime macro case also found that receiver method calls inside
macro invocation arguments must be walked before expansion. Parseable expression
macro arguments are now visited as normal Rust expressions, which keeps
dependencies such as `self.dto.score()` inside retained helper macros without
requiring project-specific macro knowledge.

For UniFFI-style objects, the selected root matters. Selecting the object type
means every retained exported method may be part of the public API surface; a
narrow function root that constructs the object and calls only selected methods
should prune unused exported sibling methods. The fast fixture now includes a
callback-provider setter flow that converts `Box<dyn Trait>` into stored
`Arc<dyn Trait>` and verifies the unused provider method path stays out of that
narrow slice.

Litter's SSH module also exposes boxed I/O trait-object aliases through a
module facade. The rule database now has a generic boxed I/O alias case: a live
`Box<dyn Read + Send>` alias reexport is retained and reported as a dynamic
surface, while sibling `Write` aliases and imports are pruned. This keeps broad
facades from turning one live stream alias into every adjacent stream type.

The next batched rule pass raised the executable set to 92 cases in one fast
iteration. It added generic coverage for facade const/function aliases, result
aliases, returned `Arc` object handle records, nested DTO collections, map DTO
surfaces, mirror `TryFrom` request conversions, selected section
`include_str!` assets, runtime singleton facades, and dead inherent impl method
pruning. That batch also exposed a renderer bug: parent imports used only by
retained child modules through `super::{...}` were pruned. Import-scope
liveness now checks retained inline and file-backed child modules before
dropping parent imports.

The following batch raised the executable set to 102 cases. It added coverage
for alias chains, tuple newtype surfaces, `impl Iterator<Item = Dto>` returns,
enum variant constructors used as functions, struct update defaults, inherent
method references in iterator adapters, boxed future return aliases, const array
surface lengths, and nested `Result<Option<T>, E>` aliases. The failure found by
that batch was a minimality bug in `?`: the reducer kept every
`From<_> for TargetError` impl when a retained function returned
`Result<_, TargetError>`. It now resolves the source error type of the tried
expression and retains only the exact `From<SourceError> for TargetError` impl,
falling back to broader target-error retention only when the source error type
cannot be inferred.

The next Litter-exploration batch raised the executable set to 112 cases and
recorded source evidence in `docs/rule_database.md`. The added rules cover
UniFFI-style callback interface facade reexports, returned subscription objects,
split exported wrapper impls, nested tagged/untagged serde patch contracts,
method-string typed dispatch with unknown fallback, fallible
`Option<Vec<T>>`/`transpose()?` request conversions, function-local
`include_bytes!` assets, compile-time env reads inside retained `macro_rules!`
bodies, dependency-crate alias manifest edges, and private-child wildcard imports
with selected public reexports. Two generic hardening changes came out of the
batch: reachable macro/FFI impl surfaces now expand only for selected item roots
or types that appear in retained callable signatures, and retained macro bodies
are scanned for `env!`/`option_env!` hazards before feedback-only macro warnings
decide production status.

The current rule pass raises the executable database to 132 cases and adds an
enforceable coverage-family map for all 1,200 generated catalog records. The new
fixtures cover nested private facade chains, UniFFI enum struct variants, nested
serde response envelopes, multiple serde helper paths, generic
`T: TryInto<Target, Error = E>` bridges, wire-error fallback conversions,
`concat!` include arrays, `OnceLock` include initializers, borrowed dyn facade
boundaries, and local facade glob pruning. The concrete bug found by the batch
was generic conversion over-retention: a generic helper body with
`input.try_into()?` kept every local `TryFrom<_>` candidate because `T` was not
known while analyzing the helper body. The reducer now treats generic
`Into`/`TryInto` receiver calls as bounded generic operations inside the helper,
then resolves the exact concrete conversion impl at the call site where the
actual argument type is known. The same validation pass caught an FFI impl
surface over-retention bug: macro/export impl methods are now expanded only for
item roots or root callable API signatures, so internal helper return types keep
only methods that are actually called.

The fixture also now includes a private FFI barrel shaped like Litter's mobile
client modules: one selected app-store subscription module is reexported beside
dead reconnect/alleycat siblings. The expected behavior is strict barrel
pruning: keep the selected object, subscription, update record, `VecDeque`
state, and exact live `pub use`; remove unrelated sibling exports and dead
object methods.

The usage-classification pass now uses RA reference evidence as a first-class
guard instead of treating RA definition mapping as deletion permission by
itself. The analyzer maps syn-indexed functions/items to RA definitions, runs
`find_all_refs` for mapped non-module symbols, and stores reference owners as
callables, items, or unowned file-level references. Callable/item owners are
also promoted into `SemanticReductionHints`, so retained owners pull referenced
targets into the positive `used` graph before rendering instead of leaving
every RA reference as unknown retention. Unknown retention is still a
fixed-point over the retained slice for unmapped graph-unreachable symbols,
symbols whose reference query failed, and retained RA references that cannot be
turned into owner-target graph edges. This keeps dead-to-dead references
removable while preventing retained macro/import/semantic surfaces from losing
a target that syn failed to connect. Unowned file-level references are
intentionally diagnostic-only for retention: RA reports dead impl headers such
as `impl DeadType` as references with no callable/item owner, and treating those
as retained-file references kept unrelated same-file siblings.
Renderer import pruning now uses that same fail-closed retained set. Items kept
only because they are `blocked_by_unknown` are included in the rendered mention
index, so imports required by their public fields/surfaces are preserved while
proven-prunable sibling imports are still removed. This closes the case where a
retained unknown struct field referenced `IpcConnection` but the local `use`
was stripped because the item was not in the positive used graph.
The reference-search focus is now resilient to doc/attribute collisions. Rather
than querying only the first textual occurrence of a symbol inside its full syn
span, the analyzer builds candidate offsets from identifier-boundary matches,
tries declaration-name matches first (`fn`, `struct`, `enum`, `trait`, etc.),
and only records `semantic_usage_reference_incomplete` after every candidate
fails. The RA fixture covers callables and items whose names appear in `#[doc =
"..."]` before the actual declaration, and asserts zero failed reference
queries.
RA outgoing-call feedback now uses the same candidate-offset strategy. That
keeps the default production analyzer path from losing call-hierarchy edges when
a retained function's span mentions its name in an attribute before the real
`fn` declaration.
Locked production feedback also reconciles the generated `Cargo.lock` after a
repair changes files, because conservative warning repairs can let Cargo advance
far enough on the next attempt to notice a stale lockfile.
The Litter `codex-ipc-public-mixed-surfaces` random5 corpus slice now exercises
this path end to end: RA mapped 1712/1751 callables and 838/885 items, ran 2421
reference queries with zero failures, pruned 1747 callables and 840 items as
proven removable, retained 3 callables and 21 items as `blocked_by_unknown`, and
passed production feedback with zero compiler errors or warnings after one
unused-import repair. The remaining gate was `review_required` because retained
macro, trait-object, syntactic fallback, and bounded semantic-budget hazards are
still warning-level semantic review items, not because pruning produced a broken
slice.

The next semantic-budget gap was whole-file query spend. Retained files can
contain large dead same-file siblings before the selected API, and the old
semantic inventory counted every method/path in those files against the bounded
RA budgets. The analyzer now builds the syntactic retained owner set first and
collects method/path semantic inventory only inside those retained callable/item
owners. The regression fixture puts more than the default method-call budget in
a dead same-file function before the selected entry and proves the retained
owner still gets queried with zero method-call budget exhaustion.
On the same Litter `codex-ipc-public-mixed-surfaces` random5 probe, this removed
the retained-slice budget-exhaustion hazards: method calls went from 1000
queried plus 82 unqueried to 9 queried and 0 unqueried, paths went from 2000
queried plus 806 unqueried to 233 queried and 0 unqueried, and the generated
slice still passed production feedback with zero compiler errors or warnings.
Remaining review hazards are now unresolved retained-owner RA queries,
macro/derive/invocation surfaces, trait-object surfaces, and syntactic fallback
evidence rather than budget starvation.
