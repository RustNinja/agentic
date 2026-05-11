# Production Hardening Learnings

Last updated: 2026-05-11

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
- Support-package minimization needs member-level proof, not only item-level
  proof. Public fields and enum variants can be dead even when their parent type
  is retained for an impl or helper surface, so the renderer must classify and
  prune members with concrete owner evidence.
- External path support packages need the same member-level pruning when they
  are copied as restricted source. A helper record retained only behind a live
  support function can drop unused pure fields and matching pure struct-literal
  initializers, while a support type that appears directly in the selected API
  signature must keep its public field surface.
- Concrete member evidence must include cross-package typed bindings. A retained
  helper can receive `dependency::Type` as a parameter and access one field; the
  slicer must retain that field without promoting every same-named field in the
  dependency graph.

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
- when scanning support proc-macro exports, follow actual helper function calls
  from the AST rather than every matching identifier token, because local
  variables can share names with dead helper functions;
- resolve module-qualified proc-macro helper calls through the local helper
  module graph, so expansion evidence can follow real builder modules without
  broadening to same-named helpers in unrelated modules;
- resolve local proc-macro helper `use` aliases and globs before following
  helper calls, because proc-macro crates often re-name builder functions at the
  export boundary;
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
The follow-up gap was RA feedback edges discovered after that initial syntactic
retention pass. A support file can be retained only because outgoing call
hierarchy finds it, which means the old semantic file walk could still spend
its file budget on unrelated workspace files before analyzing that support
owner. RA feedback is now a transitive retained-owner queue: newly discovered
callable targets are queried for their own outgoing calls, and the resulting
hint graph refreshes the retained-owner/file priority before bounded semantic
inventory starts. The regression fixture uses ambiguous method names across
modules (`dispatch` and `finish`) to prove the closure keeps the real
two-hop support chain while pruning the unrelated ambiguous methods.
On the same Litter `codex-ipc-public-mixed-surfaces` random5 probe, this removed
the retained-slice budget-exhaustion hazards: method calls went from 1000
queried plus 82 unqueried to 9 queried and 0 unqueried, paths went from 2000
queried plus 806 unqueried to 233 queried and 0 unqueried, and the generated
slice still passed production feedback with zero compiler errors or warnings.
Remaining review hazards are now unresolved retained-owner RA queries,
macro/derive/invocation surfaces, trait-object surfaces, and syntactic fallback
evidence rather than budget starvation.

The next Litter probe exposed a narrower import-use false positive in
`codex-ipc/src/client/reconnect.rs`: the source imports
`tracing::{error, info, warn}`, but a compact slice only needs `warn!` while
retained code still has local pattern bindings named `error`. A raw token check
kept the removed `tracing::error` import leaf because the local binding text
matched the import name. Private external import pruning now walks retained
functions, methods, and item surfaces with a scoped syn visitor, records local
bindings from `let`, match arms, closures, and loops, and counts an import leaf
only when it is actually used as a path/macro name rather than as a shadowing
local variable. Macro bodies remain fail-closed by token scan because macro
expansion is not yet semantically modeled.

That change also had to share the renderer's existing surface boundary. Traits
retained only as object/type surfaces are scanned only through generics,
supertraits, inert surface attrs, and trait items that will actually be rendered;
pruned trait method signatures no longer retain method-only imports or
dependencies. The regression suite caught the inverse macro case as well:
`tokio::select!` macro bodies must retain imports such as `Duration` that appear
inside macro tokens. The generic fixture
`prunes_external_group_imports_shadowed_by_local_bindings` now covers the local
shadow case, `async.select_reconnect_loop.001` covers macro-body retention, and
the Litter random-five probe generated `client/reconnect.rs` with
`use tracing::warn;`, zero feedback errors, and zero warnings.

The same rule now applies to renamed import aliases. A private
`use crate::shared::OtherValue as local_shadow` can point at an item that is
retained elsewhere while still being unused in the current module. Raw alias
text was not enough because a local `let local_shadow = ...` made the alias look
live. Private renamed imports now require an actual retained path/macro use of
the visible alias, unless the import has an implicit scope effect such as a
trait import needed for method resolution. The fixture
`prunes_renamed_imports_shadowed_by_local_binding_when_target_is_retained_elsewhere`
keeps the target item in the module that actually uses it while pruning the
shadowed alias from the unrelated module.

Direct private import leaves now follow the same structured rule. A retained
helper function used in one module no longer keeps
`use crate::shared::{FeatureValue, helper}` in another module just because that
module has a local `let helper = ...`. The first broad run exposed the surfaces
that must remain fail-closed: retained impl headers such as `impl Service`,
format-string captures such as `format!("{PROFILE_INIT}")`, and retained custom
attributes such as `#[handle_error(LocalError)]`. Those surfaces are now added
as explicit import-use evidence instead of falling back to raw retained
function/body tokens.

Parent imports consumed through child modules now use the same structured
import-use check. A child module with `use super::*` no longer keeps the
parent's `helper` import only because the child has a local binding named
`helper`; the parent leaf is retained only when the child actually uses the
imported name through a retained path, macro token, signature, retained
attribute, or other explicit surface. Existing `use super::*` cases that need
parent imports for child signatures and trait-method resolution remain covered.

RA unresolved query reporting is now node-level instead of count-only. Each
unresolved retained-owner method call or path records kind, category, reason,
file span, snippet, AST kind, symbol, and owning callable/item when known. The
production gate suppresses generic unresolved-method/path hazards when all
diagnostics for that kind are benign, while macro-context and project-local
dependency-risk unresolved nodes remain explicit review hazards and keep the
usage classifier fail-closed. On the Litter `codex-ipc` random-five probe, the
same slice still passed feedback cargo check with zero warnings, all 9
unresolved method calls were classified benign, unresolved path diagnostics
split into 97 benign, 75 macro-blocked, and 2 dependency-risk local anchors
(`TurnStartParams` and `ReasoningEffort`). The important classifier fix was to
evaluate only the unresolved path's actual `A::B` segments, not every
identifier inside rendered generic arguments; otherwise benign external paths
such as `Result<String, LocalError>` and `String::new` inherited local tokens
from nested syntax and looked risky.

Macro surfaces are now first-class report data instead of only production
hazard prose. Retained derive macros, attribute/helper attributes, and
function-like macro invocations are serialized under `macro_surfaces` with
kind, macro path, owning callable/item, source span, category, and explicit
`blocked_idents`. The usage-retention path now trusts those explicit blockers
for custom macro hazards instead of scraping free-form subjects, so macro names
and helper meta keys such as `Serialize`, `serde`, `error`, `from`, or `with`
do not retain unrelated graph-unreachable code. Macro hazards still remain
review surfaces, but only a real helper/path identifier such as
`#[serde(with = "wire_helper")]` can block an otherwise-unused matching local
candidate. Macro-blocked RA unresolved nodes are still recorded in
`analyzer.semantic.unresolved_diagnostics`, but they no longer create duplicate
generic `semantic_unresolved_*` hazards; project-local dependency-risk
unresolved nodes still do.

On the refreshed Litter `codex-ipc` random-five probe, feedback cargo check
passed with zero errors and warnings and the harness reported
`production_ready=accepted`. The report indexed 74 macro surfaces
(29 derives, 1 attribute macro, 40 helper attributes, and 4 macro invocations),
reduced `semantic_unresolved_paths` details to the 2 real local anchors, and
classified all 1,750 unused callables plus 861 unused items as prunable with
zero `blocked_by_unknown` entries.

Transparent selected API wrappers around borrowed callbacks are dynamic
boundaries, not owned dynamic-dispatch storage. The scanner now treats
`Option<&dyn Trait>`, `Result<fn(...), E>`, and the same shapes under
`std`/`core`/`alloc` wrapper paths as `dynamic_callback_boundaries` while still
leaving `Box<dyn Trait>`, `Arc<dyn Trait>`, returned trait objects, stored
registries, and local function-pointer surfaces as review-required dynamic
hazards. This reduces false unknown pressure on UniFFI-style input signatures
without making owned/stored dispatch look proven.

Pinned real-repo corpora need a source identity guard before expensive builds.
The UniFFI smoke file pinned `5ccb9a7`, but the local source checkout was
`b5d9469`; without a guard, the harness spent minutes proving the stale source
baseline failed. The corpus runner now checks a roots file's top-level
`commit` against the source checkout and fails fast with a clear mismatch unless
`--allow-source-commit-mismatch` is set for deliberate stale-corpus probing.

Dynamic-dispatch review pressure should shrink only when the code carries its
own proof. Auto-trait-only objects such as `dyn Send + Sync` have no callable
dispatch surface, so they no longer produce `trait_object_surfaces` hazards.
Returned trait-object surfaces are also cleared when the retained function or
method returns an expression that visibly constructs a local concrete type and
that type has a local impl for every non-auto trait in the returned `dyn`
object. The proof is deliberately narrow: non-return-position constructions,
dead constructions inside returned blocks, opaque factory-call arguments,
forwarded `Box<dyn Trait>` values, stored `Arc<dyn Trait>` registries, `dyn Fn`
callbacks, future aliases, and local function-pointer fields still remain review
hazards. This keeps unknown retention scoped without pretending callback
registries are statically proven.

Root mining should not require source edits. The CLI now accepts explicit
`--root` and `--roots-file` selectors that are resolved to the same internal
`RootId` graph roots as `#[opensourced]`, but without writing markers or adding
temporary macro dependencies to the source checkout. Batch mode reuses one
parsed project and one analyzer report for many selected roots, then writes one
output workspace per root plus a JSONL status row. Real Litter mining showed
two more CLI requirements: random roots need package scoping so a probe does not
silently pick a huge unrelated workspace surface, and batch feedback must share
one target directory so external dependency builds are reused instead of
recompiled per root. This is the faster failure mining path: keep the original
repo warm and immutable, run many independent slices, classify failures from
generated preflight/check reports, and promote real failures into generic
fixtures or fail-closed hazards.

The first package-scoped Litter `codex-ipc` random-10 probe exposed generic
renderer cleanup issues rather than Litter-specific cases. With a single
feedback check the batch accepted only 4/10 roots; with two checks, compiler
diagnostics widened missing method impls and moved the client roots to compile.
The remaining failures were warning-only under `--deny-warnings`: a public glob
reexport survived after its source module was removed, and macro dependency
imports such as `serde::de::DeserializeOwned` and `thiserror::Error` survived
after their owning dead items were removed. The renderer now drops public glob
reexports unless the source module is rendered and exposes a referenced public
name, and macro dependency import retention is leaf-scoped rather than
package-wide. The rerun accepted all 10/10 selected `codex-ipc` roots with zero
errors and zero warnings in 252 seconds, using one RA load and one shared batch
target directory.

The follow-up rootless random-20 `codex-ipc` probe reused the same seedable
batch path and found the next layer of over-retention. The first run accepted
15/20 roots; the five failures were all warning-only under `--deny-warnings`.
The recurring shapes were derive-macro imports retained because another live
path mentioned `Error`, grouped external type imports retained because a pruned
private field mentioned the type, and public glob reexports retained because the
original source module had public names even though the rendered module exposed
none for that root. The generic fix was to make thiserror derive imports
rendered-attribute-scoped, stop treating every probable external uppercase
import as a trait when only raw type mentions exist, and compute public glob
exposure from rendered public items. The rerun accepted all 20/20 roots with
zero preflight errors, zero feedback/check errors, and zero warnings; a separate
manual `RUSTFLAGS='-D warnings' cargo check` loop also passed each generated
workspace.

The first `codex-mobile-client` random-five probe showed a different failure
mode: all five roots compiled cleanly, but tiny roots still wrote about 1,000
files because RA-unmapped unused candidates were treated as global
`blocked_by_unknown` roots. That made every unmapped UniFFI/serde surface pull
its dependency closure into unrelated slices. The policy is now narrower:
retained RA reference edges and failed reference queries still block pruning,
but an unmapped candidate with no retained reference is removed and recorded in
the semantic proof as unproven instead of rendered. Compiler feedback remains
the acceptance gate. On the same random-five seed, the shell-preflight helper
dropped from 989 files written to 5, and the full batch accepted 5/5 roots with
zero errors and zero warnings; generated workspaces ranged from 5 to 18 written
files instead of roughly 989 to 997.

The `codex-bridge::voice_handoff(Mod)` probe exposed why capped syntactic
method fallback must not become a broad method-name blocker. After narrowing
RA-unmapped retention, `manager.reset()` failed because the local `manager`
binding came from `unsafe { arc_from_raw(handle) }` and the dependency visitor
did not infer the tail expression type through `unsafe` blocks. Treating
`syntactic_method_fallback_cap` as a blocker for every recorded method name
would retain common names such as `new`, `insert`, `join`, and `push`, causing
large over-retention and slow planning. The generic fix is to improve receiver
type evidence instead: blocks and unsafe blocks with tail expressions now
propagate receiver type, receiver candidates, return type arguments, result
types, and destructuring evidence. The same Litter module-root slice now keeps
`HandoffManager::reset`, removes unrelated code, and passes feedback cargo
check with zero errors and zero warnings.

The `codex-mobile-client` SSH probes found a warning-only acceptance blocker:
two independent roots compiled after slicing but failed `--deny-warnings`
because a retained public field exposed a child-module-private handler type,
triggering rustc's `private_interfaces` lint. This is not a missing dependency
edge and widening would only add redundant code. The repair loop now handles
`private_interfaces` generically by inserting a scoped lint allow at rustc's
primary span, tracked separately from dead-code allows in the repair report.
Both mined SSH roots now pass feedback cargo check with zero warnings while
preserving the narrow slice.

The next bridge random-ten seed exposed an over-retention bug in private field
pruning. Several `voice_handoff` function roots retained `HandoffManagerInner`
and its live `action_queue` field, but also kept the unrelated private
`transcript: TranscriptBuffer` field because another retained public enum
surface had a field named `transcript`. The graph already marked
`TranscriptBuffer` reachable, but the render plan did not render it because no
retained callable or impl actually used that field. The better generic fix is
not to widen: private field retention now ignores broad module-level item
surface name collisions and relies on callable/impl evidence plus field attrs,
public surfaces, and generic field requirements. The three mined bridge roots
now remove the unrelated transcript field and pass feedback cargo check with
zero warnings.

The follow-up bridge mining pass found a support-package pruning gap rather
than a direct compile failure. `codex_bridge_init` still accepts cleanly, but it
writes a large support tree because target-gated `codex_core` usage cannot be
proven for the current host cfg matrix. Debug tracing also showed a generic
support reducer failure on local enum glob imports such as
`use parser::ParseError::*`; a live function that matches bare enum variants
caused the support package reducer to give up and broad-copy the whole support
crate. The reducer now recognizes glob imports from local enums, retains the
owning enum when a variant name is live, preserves the glob import, and still
prunes unrelated modules and functions. The focused fixture moves the original
support crate away, proves the generated support package builds alone, and
keeps dead support modules out of the output.

The next mobile mining pass exposed a performance and reexport-retention pair.
`codex-mobile-client::types::server_requests::ask_for_approval_into_upstream`
initially burned minutes in dependency reduction because method resolution and
glob reexport lookup repeatedly scanned the full parsed project. The project
model now carries module-source and receiver-method indexes, dropping that root
to a 66 ms reducer pass after the RA inventory completes. The same root then
showed a correctness gap: a retained child module imported
`super::AppAskForApproval`, while the enum lived in a sibling `models` module
and was visible through the parent `pub use models::*`. Public glob reexport
retention now consults the existing child import-scope analysis, so that parent
reexport remains only when retained child code actually refers to the exposed
name. The focused fixture proves the sibling/parent shape builds and still
prunes unrelated child modules; the mined Litter root now reaches feedback cargo
check with zero final warnings.

The next `codex-mobile-client::ffi::client::AppClient::list_plugins` mining
root became a scale probe for the reducer and renderer. It selects a small
method, but its surrounding UniFFI client type can expose many package-local
imports, foreign item surfaces, support structs, and method candidates while RA
is still bounded to retained owner files. Recomputing reachable callable token
idents and scanning every method receiver for each dependency/import decision
made the run look stalled even before compiler feedback. The model/reducer now
carry receiver-method and reachable-callable-ident indexes, and the renderer
builds a reachable-token-ident index once per render plan for reexports and
foreign item surfaces. That keeps the mining loop responsive enough to expose
the real next gap: the generated slice can still over-retain broad mobile
surfaces and feedback can still require generic method/impl/support-schema
widening, but those are now visible correctness questions instead of hidden
inside repeated whole-project scans.

The next support-package pass tightened the "used / unknown / unused" boundary
for copied path dependencies. A support type that was retained only as a data
surface used to keep every public inherent method, which made monolithic
protocol/helper crates carry dead sibling methods into the slice. Support impl
rendering now keeps inherent methods only when an associated-method requirement
is observed. To avoid unsafe pruning, token usage records method calls through
dependency-typed locals and parameters, so shapes like
`let value = provider::Payload::new(...); value.render()` retain both `new` and
`render` while pruning unrelated public methods. Macro token bodies are also
scanned for dotted method calls so live support methods mentioned inside
`format!` or similar macros are not lost. The same pass fixed import pruning
for local bindings that shadow removed imports: scoped callable usage now wins
over coarse module token mentions, so a local `helper` or `fmt` binding does
not keep `use ...::helper` or `use std::fmt` alive. The full manifest hardening
suite now covers this with support typed-local, type-only public-method, enum
payload macro-method, and shadowed-import fixtures.

The rule database then caught the next transitive support edge: a helper support
crate can reexport a leaf dependency type under a facade alias, and the root can
call methods through that alias inside normal code or macro arguments. The
support source plan now forwards required alias methods through public external
reexports, so `external_helper::LeafAlias::new()` and `leaf.label()` become
`external_leaf::LeafLive::new` and `LeafLive::label` requirements. This keeps
the leaf support crate buildable while still pruning the dead leaf package,
dead facade exports, and unrelated public methods.

The `ThreadSnapshot::from_info` Litter probe showed a related import-cleanup
case in local packages: a `pub(crate) use snapshot::QueuedFollowUpDraft`
remained even though the retained item was used only inside `snapshot.rs`.
Restricted reexports are now pruned like internal imports unless retained code
actually uses the alias. Full public `pub use` items still preserve exported
API paths, but crate-restricted aliases no longer survive merely because their
target item is live somewhere else in the package. The regenerated Litter slice
removed the stale reexport and `cargo check --quiet` completed without warnings.

The next `SshClient::resolve_codex_binary` Litter mining root exposed a batch
repair scheduling bug rather than a renderer dependency bug. The generated
slice compiled, but the first repair pass had both structural warning fixes and
a `private_interfaces` lint candidate. Repair intentionally deferred lint
allows until after structural edits, so batch mode with
`--feedback-repair-loop 1 --deny-warnings` reran Cargo, saw only the deferred
warning, and reported `check_failed`. Batch repair now performs one final
warning-only repair when a prior structural pass explicitly deferred lint or
dead-code allows and the current Cargo report has no errors or semantic warning
hazards. The same Litter root now adds a scoped `#[allow(private_interfaces)]`
to the generated `SshClient` item and reaches zero-warning feedback acceptance.

The next `codex-mobile-client::alleycat::list_agents` root found a real
conversion-edge gap. Retained code built `AgentInfo { wire:
agent.wire.into(), ... }`; the field target type was `AgentWire`, while the
source field came from a private wire DTO enum. The reducer now uses struct
literal field types as expected conversion targets, so it retains
`impl From<AgentWireWire> for AgentWire` without retaining dead sibling
conversions to the same target. The same probe also produced an `unused_mut`
warning after cfg pruning removed the Android-only reassignment branch. Repair
now applies rustc `MachineApplicable` `unused_mut` suggestions, so strict
batch feedback can remove the stale `mut` and accept the generated workspace
with zero warnings.

The fixture harness now enforces the used/unknown/unused contract after every
generated slice build. Each checked slice-case fixture runs `cargo check`, then
verifies that rendered callables/items are partitioned as `used` or
`blocked_by_unknown`, that public `usage.unused` equals the prunable/removable
set, and that generated Rust source does not still declare prunable functions,
methods, structs, enums, traits, aliases, constants, statics, or macros. The
first pass exposed a report/render mismatch for structural module declarations:
file-backed `mod live;` wrappers were rendered to reach live child code but
were still reported as prunable. The reducer now promotes module wrappers for
reachable file-backed modules into the retained item set. Inline facade and
prelude modules remain structural scaffolding for import paths; they are not
treated as semantic declarations in the generated-source prunable scan, because
feeding those public reexport wrappers back into the reducer causes dead public
reexport leaves to survive.

A new lightweight Litter-shaped fixture mirrors the demonstrated
`codex-tui::theme::{health_color, health_symbol}` slice without depending on
the full Litter build. The fixture keeps two selected theme roots, a local
`mobile-client` support package, an enum matched by both roots, live constants,
and dead sibling helpers/API surfaces. The expected output keeps only the theme
roots, required constants/type surface, and `ServerHealthSnapshot` support path,
while pruning dead mobile modules, support methods, dead enum/data surfaces,
and unrelated theme helpers. The full slice-case suite now covers this contract
across 763 fast fixtures.

The next fixture pass moved the build-script/`OUT_DIR` blocker into the same
hard slice-case harness instead of leaving it only in generated unit fixtures.
`out_dir_generated_prune` is a checked-in two-package workspace where the root
selects a support function that reaches Rust generated by
`include!(concat!(env!("OUT_DIR"), ...))`. The test seeds the analyzer-side
generated file, verifies retained build-script and OUT_DIR production hazards,
then cargo-checks the generated workspace where `build.rs` recreates the file.
The same used/unknown/unused contract proves the support package keeps only the
live generated helper and prunes dead generated-support functions. The full
slice-case suite now covers this contract across 764 fast fixtures.

The cfg/feature blocker also now has a checked-in hard fixture.
`cfg_attr_uniffi_prune` models a Litter-shaped UniFFI surface with
`cfg_attr(feature = "ffi", derive(...))`, a nested `cfg_attr(...,
uniffi::export)` impl, and a helper attribute on a retained record field. The
production report must keep scoped `conditional_compilation_attrs`,
`custom_attribute_macros`, and `custom_derive_macros` hazards, while the
generated default-feature output still cargo-checks and prunes dead API/model
modules, functions, and sibling methods. This keeps cfg/macro uncertainty as
review evidence without letting it broad-copy unrelated support code. The full
slice-case suite now covers this contract across 765 fast fixtures.

The returned trait-object pass exposed a real over-retention bug in retained
support traits. When a trait was kept only because a rendered impl method was
reachable, the renderer kept the whole trait body, including unused default
methods. `returned_dyn_trait_prune` now slices through a returned
`Box<dyn Reader>` plus a dynamic `reader.read()` call and requires the generated
support crate to keep only the required trait item, concrete impl, constructor,
and render helper. The fix is generic: rendered impl methods no longer force
the entire trait surface to render; the existing trait-item predicate decides
which required, referenced, or macro-protected items remain. The full
slice-case suite now covers this contract across 766 fast fixtures.

The fixture harness now also audits trait default methods as part of the generic
used/unknown/unused contract. After every generated slice cargo-checks, the
harness scans rendered Rust and fails if a retained trait default method is not
reachable from retained call sites or from another reachable default method, and
is not protected by a `blocked_by_unknown` trait item. This turns the
`dead_default` class of redundancy into a global fixture failure instead of a
one-off string assertion, while still allowing valid default-method flows such
as generic bounds and UFCS dispatch.

Generated slice-case workspaces are now checked with `unused_imports` denied.
This directly targets the stale `use path::{x, y}` class of failures without
turning deliberate enum-shape fallout, cfg review surfaces, or public unknown
API remnants into unrelated hard errors. Unused `use` remnants can no longer
hide behind a successful `cargo check --quiet`.

The first failure from that stricter import check was a retained private
`pub(crate) use render_record;` next to a local `macro_rules! render_record`
definition. The renderer now prunes same-module, non-public macro self-reexports
before normal use-tree pruning. The macro definition and invocation still remain,
but the generated support package no longer carries an unused import.

The hard fixture loop now has explicit RA-backed proof tests for both a true
support sub-dependency chain and a Litter-shaped theme/support-package slice.
These tests force `AnalyzerMode::RustAnalyzerHir`, require semantic usage
mapping and promoted reference edges, require
`complete_for_retained_packages`, require zero unproven prunable callables or
items, then cargo-check the generated output with `unused_imports` denied and
run the same rendered-source used/unknown/prunable audit. This proves the
important production contract for those fixtures: retained root and dependency
package source may contain only used symbols or symbols explicitly protected by
unknown analysis surfaces; prunable retained-package symbols must be absent
from generated Rust.

That RA proof now runs as a high-risk fixture matrix instead of only isolated
examples. The matrix covers proc-macro surfaces, macro-generated references,
macro receiver calls, cfg_attr/UniFFI-style macro attrs, returned dyn trait
surfaces, stored callbacks, FFI exports, static registries, poll adapters,
UniFFI runtime-shaped helpers, and build.rs/OUT_DIR includes. Each row asserts
the exact retained package set, exact scoped production hazards, complete RA
proof for every prunable retained-package symbol, a generated cargo check, and
the rendered-source used/unknown/prunable audit. This keeps conservative
blockers visible while proving they do not force unrelated dependency code to
survive in those fixtures.

The latest Litter-shaped fixture adds a `codex-tui` conversation render root
that reaches nested screen modules plus `codex-core` state helpers and
`codex-protocol` event/message/role types. It prunes the dead settings/theme UI
modules, dead support packages modules, and dead sibling methods while keeping
the live iterator/filter/render closure. Under RA it proves 20 retained-package
callables and 7 retained-package items are prunable with zero unproven symbols,
then still reports only scoped unresolved/fallback warnings. The full checked
slice-case suite now covers 767 generated workspaces.

The next Litter-shaped fixture covers a `codex-mobile-client` FFI session root
with UniFFI-style `cfg_attr(..., derive(...))` records/objects, an exported
object impl, core session state, and protocol wire DTO/status types. It prunes
dead voice-handoff/settings UI modules, dead support modules, dead exported
methods, and dead DTO helpers. Under RA it proves 23 retained-package callables
and 9 retained-package items are prunable with zero unproven symbols, while
leaving only scoped cfg/derive and semantic unresolved warnings. The full
checked slice-case suite now covers 768 generated workspaces.

The bridge IPC fixture adds a Litter-shaped `codex-bridge::ipc::handle_frame`
root that decodes a wire frame, resolves a string method enum, dispatches
through a core session helper, and returns a protocol response. It prunes dead
ssh entrypoints, dead support modules, dead protocol response helpers, and
unused support methods across `bridge-core` and `bridge-protocol`. Under RA it
maps all 31 callables and 14 items, proves 16 retained-package callables and 5
retained-package items are prunable with zero unproven symbols, and reports
only scoped unresolved method/path warnings. The full checked slice-case suite
now covers 769 generated workspaces.

The event callback fixture adds a Litter-shaped mobile FFI root that registers
a callback handle, stores an `Arc<dyn EventCallback + Send + Sync>` in a core
event bus, emits an event envelope, and converts it into protocol DTO records.
It keeps the dynamic-dispatch and cfg/derive surfaces as scoped production
warnings, but still prunes dead diagnostics/voice modules, dead support
modules, dead callback helpers, and dead protocol DTO helpers. Under RA it maps
all 48 callables and 24 items, proves 23 retained-package callables and 9
retained-package items are prunable with zero unproven symbols, and leaves only
scoped cfg, derive, trait-object, fallback, and unresolved semantic warnings.
The full checked slice-case suite now covers 770 generated workspaces.

The Litter-shaped OUT_DIR codegen fixture exposed a real fail-closed gap:
`generated::generated_event(label).render()` had a generated function return
type, so RA/syn could not prove the receiver and the name-only fallback was
capped. The renderer previously pruned `GeneratedEvent::render`, producing a
non-buildable slice. The fix is generic: `syntactic_method_fallback_cap`
hazards now participate in the unknown-deletion guard with package/module
scope, so the affected same-module method stays `blocked_by_unknown` without
globally retaining unrelated `render` methods. The fixture keeps retained
build-script and OUT_DIR errors, proves 14 retained-package callables and 6
items prunable with zero unproven symbols, and keeps only six scoped
generated-code callables blocked by unknown. The full checked slice-case suite
now covers 771 generated workspaces.

The Litter-shaped reconnect grouped-import fixture tightened the sub-dependency
contract from "builds" to "contains only rendered used/unknown surfaces." It
found two generic issues. First, the fixture validator did not compare rendered
trait impl methods using the same `<Type as Trait>::method` spelling that
`CallableId` reports, so retained prunable trait impl methods could evade the
hard validation loop. The validator now canonicalizes rendered aliases and
trait impl method IDs before checking that no `usage.prunable` callable remains
in generated source. Second, private fields initialized by side-effect-free
std/core/alloc constructors such as `Duration::from_secs(...)` and
`Cow::Borrowed(...)` stayed in a support package solely because the initializer
was not recognized as prunable; their grouped `use std::{...}` leaves then
remained too. The renderer now threads module aliases into field-use scanning,
prunes those pure std constructor initializers when the field is otherwise
unused, and runs import liveness over the pruned block view. The full checked
slice-case suite now covers 772 generated workspaces.

The same validation pass turned the focused rule database fully green again
after exposing stale reexport pruning gaps. Public reexport target retention now
ignores the body of the module that defines the target, so a dead sibling such
as `DeadType` inside `mod api` no longer proves `pub use api::DeadType` is
live. Root glob reexports similarly ignore inline/file module bodies when
checking whether an exposed leaf is used, preventing dangling
`pub use removed_mod::*` leaves when retained code references the same type
through a different module path. Restricted macro helper reexports remain
generic: `pub(crate) use helper_macro` is kept only when a rendered module or
reachable callable imports/calls that macro through the defining module path.

The hard slice-case audit now also fails if generated source declares any
callable or item that is not classified as `used` or `blocked_by_unknown`.
Previously it only proved that `usage.prunable` symbols were absent, which left
room for symbols missing from the report inventory to survive unnoticed. The
stricter check passed across the full fixture suite and now protects support
packages from both explicit prunable leakage and unclassified rendered leakage.

The next Litter-shaped fixture covers the `codex-ipc` reconnect callback shape:
`Arc<RwLock<Option<Arc<dyn RequestHandler>>>>` plus a connector typed as
`dyn Fn() -> Pin<Box<dyn Future<Output = Result<...>> + Send>>`. The generated
support package keeps the live request handler, connector aliases, future/error
surface, and selected reconnect controller while pruning dead controller,
handler, response, and sibling module code. Under RA it reports only scoped
method fallback and trait-object review warnings, and the full checked
slice-case suite now covers 773 generated workspaces.

The hard validation loop now also includes a Litter-shaped conversation-state
fixture with real `serde`, `serde_json`, and `thiserror` dependencies. The
selected `codex-ipc::conversation_preview` root parses a private serde DTO tree,
uses a thiserror conversion enum in the public result surface, and prunes the
dead sibling parsing module plus dead live helpers from the support package.
This fixture is intentionally macro-heavy but should not create production
hazards because the derive/helper surfaces are dependency-proven known
contracts. It passes both the default slice path and the RA high-risk matrix
with zero expected RA hazards, while the full checked slice-case suite now
covers 774 generated workspaces.

The slice-case hard audit now validates package inventory before source-symbol
classification. Every generated `Cargo.toml` with a `[package]` section must
match `GenerateReport.packages` exactly, and duplicate generated package names
fail the test. This closes a package-level escape hatch where an extra copied
support crate could exist outside the report and avoid the used/unknown source
audit. Focused checks on trim-unused, Litter serde, and Litter reconnect
callback fixtures passed with the stricter package inventory contract.

The same hard audit now treats public reexport leaves as part of the rendered
surface. For every generated `pub use` leaf that resolves to a known local
callable, item, or module path, the fixture harness checks that the target is
classified as retained `used` or `blocked_by_unknown`, not `prunable`. External
or unresolved reexports stay outside this proof instead of becoming false
positives. This closes another escape hatch where source declarations could be
removed correctly but a stale public facade alias still advertised a pruned
local target. The full slice-case suite passed across all 774 generated
workspaces with the stricter reexport contract.

Tightening that audit again found a real facade-chain proof gap. A generated
support package can reexport a local facade leaf, where that facade leaf is
itself a public reexport into another retained support package. Treating only
the first local path as a concrete item mislabeled the chain as unclassified.
The harness now builds an exposed-public-reexport index and follows reexport
chains until they reach a retained/prunable symbol, while still ignoring
arbitrary external crate reexports that are outside the local proof. It now
fails both stale-prunable and unclassified-local reexport targets, and the full
slice-case suite passes across 776 tests with that stronger contract.

That proof has moved into the production report path. Each generated slice now
serializes `usage.public_reexports` with per-leaf facade-chain resolution and
summary counts, and the production gate emits hard hazards if a generated
public facade points at a prunable or unclassified local target. The
`returned_object_prune` fixture asserts this through a real support-package
facade from `returned_api` into `returned_model`, so the contract is no longer
test-only infrastructure.

The next hard-audit tightening closed a support-package blind spot. The fixture
validator and production public-reexport proof now resolve package source roots
from generated `Cargo.toml` manifests instead of assuming every retained
package lives at `<output>/<package>/src`. Nested copied packages such as
`support/external_helper` are now scanned for rendered symbols and public
facades, so copied sub-dependencies must satisfy the same used/unknown contract
as normal workspace packages.

The rendered source-symbol audit is now a production report gate, not only a
fixture assertion. `usage.rendered_symbols` records every generated callable and
item in retained packages, compares it against the used/blocked/prunable
decision index, and raises hard production hazards for rendered prunable or
unclassified local symbols. Exported proc-macro functions annotated with
`#[proc_macro]`, `#[proc_macro_attribute]`, or `#[proc_macro_derive]` are
classified as scoped `blocked_by_unknown`, because their true call sites are
macro expansion surfaces rather than normal Rust references. This keeps retained
proc-macro packages conservative while still proving that ordinary support
package functions and items do not leak as redundant code.

The first rule-database run against that production gate exposed two proof
normalization bugs. Inline child modules must not inherit parent `use` aliases
when the rendered scanner reconstructs impl receiver paths, otherwise local
types such as `jni::JNIEnv` can be reported as `jni::jni::JNIEnv`. Required
trait impl methods are also rendered surface obligations: if the retained source
still contains `impl Trait for Type`, the methods required by that trait are
classified as scoped `blocked_by_unknown` in the rendered-symbol proof even when
no selected root calls them directly. Optional/default trait methods still remain
eligible for normal pruning proof.

The trait-default-method hard audit has also moved into production reports.
`usage.rendered_symbols` now counts rendered default methods, tracks direct call
references plus references made from other retained default methods, and emits a
hard `rendered_unproven_trait_default_methods` hazard if a generated trait keeps
an unreferenced default body without the trait itself being
`blocked_by_unknown`. This closes another support-package redundancy class where
a retained trait item could hide dead executable code inside its body while the
top-level item proof still looked clean.

Rendered module declarations are now part of the same production proof. The
scanner records `mod name;` and inline `mod name { ... }` declarations as
`ItemKind::Mod` entries. Structural module shells are allowed only when they are
parents of rendered code, public facade reexports, or item-level `include!`
source injection that is already reported as an unknown/generated-source hazard;
otherwise a generated support package cannot keep a prunable module shell after
its contents were removed. This is intentionally separate from public-reexport
module-path tracking: module paths help resolve facades, while module items are
now checked against the used/unknown/prunable decision index.

Trait associated const overrides need the same path-aware treatment as inherent
associated items. A selected root can read a trait const override through
`Type::CONST` when the trait is in scope, not only through
`<Type as Trait>::CONST`. The renderer now treats that as a concrete reference
to the live trait impl item and keeps the matching trait associated const
declaration as well; otherwise the slice can either fail to compile with an impl
member that no longer exists in the trait, or worse, compile after falling back
to a default value and silently change behavior. This is a generic type-path
resolution rule, not a project-specific fixture string.

The same rule must cross package boundaries. A root crate can implement a trait
from a support package for a local type, then read the associated const through
`LocalType::CONST`. That path is resolved through the local type, while the trait
declaration and import live behind a dependency edge. The renderer now resolves
impl trait headers to their real package/path, keeps the rendered impl header
import, and uses that resolved trait target when deciding whether the support
trait's associated const declaration is live.

Generated Rust source is an unknown boundary, but the method closure for that
boundary must be scoped. A retained `include!` in one module does not justify
retaining same-named methods on every type in the package. The slicer now treats
method calls as generated-source roots only when the receiver expression flows
through the include/generated module, then keeps the local receiver method
closure from that point. This preserves OUT_DIR-style `generated::factory().run()`
chains without reintroducing unrelated prunable methods such as enum helper
methods on ordinary DTO surfaces.

Public visibility in a support crate is not enough to prove a variant is part of
the selected slice contract. Full enum variant surfaces are still preserved for
selected root items, resolver-proven selected callable signature types, and
macro/serde/FFI-style contract attributes, but a plain `pub enum` used only as an
internal support implementation detail can now be narrowed to the variants
mentioned by retained code. This directly supports the used/unknown invariant
for sub-dependencies: dead support variants and their payload types should not
survive merely because they are `pub` in the original crate. The signature check
uses the reducer's path resolver rather than token/name matching, so an app-local
type with the same name as a support enum does not accidentally promote the
support enum to full public-surface retention.

The same resolver rule applies to macro-bearing inherent impl surfaces. A
selected function signature that mentions `api::Client` must not make every
same-named `model::Client` impl block with `#[cfg_attr(..., uniffi::export)]`
retain all exported methods. Root signature surface checks now ask the reducer's
signature dependency resolver whether the concrete impl receiver item is part of
the selected API surface before preserving a whole macro-contract impl; otherwise
normal method reachability keeps only the used methods.

Public support struct fields now follow the same API-surface rule. A retained
public helper record inside a support package no longer keeps every public field
merely because the original crate exposed the field. Full public field surfaces
are preserved for selected item roots and for structs that the selected callable
signature resolver proves are input/return API types; other support fields must
be referenced by retained code, protected by field/struct attributes, or remain
blocked by an unknown surface. This closes another over-retention path in the
used/unused/unknown split without hard-coding Litter symbols.

Package-wide field-name fallback is only safe when it is unambiguous. A retained
`.value` access on `LiveRecord` must not keep `AuditRecord.value` in the same
support module. Struct fields now get concrete-owner evidence first; if another
rendered public struct in the same module has the same public field name, the
broad fallback is disabled and the field must be kept by concrete use,
same-struct impl use, API-surface proof, attributes, or unknown retention.
Cross-module conversion impls still rely on the broader fallback until their
type-flow proof is strong enough to narrow safely.

Concrete support-field proof must scan typed function and method parameter
patterns, not just their bodies. A retained helper can use a dependency field
entirely in the signature pattern, for example
`fn read(Record { value, .. }: Record)`, and same-module collision pruning will
otherwise remove that field while leaving the pattern intact.

Concrete support-field proof must bind typed closure inputs as well as typed
function and method parameters. Iterator and callback bodies often carry the
only non-API evidence that a dependency field is live, and a pure struct
initializer is not enough reason to keep that field. Binding closure inputs keeps
those fields precise without reopening package-wide field-name retention. The
closure override must also continue visiting input patterns, because
destructuring such as `|Record { value, .. }: Record| value` is itself concrete
field evidence.

Copied external path support packages need member-level minimization, not just
file/module pruning. Restricted support source now removes unused pure fields
and matching pure struct-literal initializers from internal helper structs, while
support structs directly exposed by the selected API signature keep their public
field surface. Public support facades, glob reexports, and module aliases also
mark the resolved support struct as API surface before field pruning runs.

Retained trait impl methods are live dependency owners even when they were not
selected roots. Trait-surface methods must promote their inherent helper calls
and associated references back into the rendered inherent impl, otherwise valid
slices can drop helper methods such as `self.as_str()` used only by a retained
`Deref` impl.

Import pruning must look at rendered associated item decisions, not just tokens
inside any impl block that has a retained method. Pruned associated const/type
items should not keep their imports alive, and non-public reexports should not
be treated as public API unless retained code actually uses that alias.

Returned trait-object proof can safely follow local helper return chains when
the helper's returned value constructs a concrete local implementor. This drops
review noise for `selected() -> Box<dyn Trait> { make_live() }` without
pretending that input-forwarded values such as `fn selected(x: Box<dyn Trait>) ->
Box<dyn Trait> { x }` are statically proven.

Static package-local `include!` needs a smaller production gate than generated
item source. If the included file parses only as an expression or type, it
cannot declare new Rust items, so the slicer can copy it, retain helper
identifiers mentioned by the included tokens as scoped unknown blockers, and
downgrade the production hazard to feedback review. Item/module `include!` and
`OUT_DIR` generated source still stay fail-closed because they can introduce
unindexed symbols.

Build-script generated source is not always item-shaped. `OUT_DIR` includes can
expand to a single expression such as `helper()`, so generated-source literal
scanning must recognize parseable call/method/macro expressions and generic
type strings in addition to obvious item syntax. Otherwise a helper referenced
only by generated expression source can be misclassified as unused and pruned
before compiler feedback has a chance to validate the slice.

Build scripts also assemble source with compile-time string macros. A fragmented
`concat!("helper", "()", "\n")` payload is one generated Rust expression even
though none of the individual literal fragments are meaningful Rust. The
build-script scanner now evaluates literal-only `concat!` payloads before
blocked-identifier scanning, keeping generated-source blockers scoped to the
real helper names while still pruning unrelated dead siblings.

The same issue appears in format-style build-script writes. A generated file can
be produced by `writeln!(file, "{}{}", "helper", "()")`, where no individual
string literal parses as Rust source but the formatted output does. The scanner
now evaluates literal-only `format!`, `format_args!`, `write!`, and `writeln!`
payloads, including escaped braces and simple positional/named placeholders,
before source-blocker extraction.

Generated Rust can also be assembled as token trees rather than strings. A
build script that writes `quote! { helper() }` has no source-like string literal
for the old scanner to inspect. Non-interpolated `quote!` and `quote_spanned!`
token bodies are now converted to source text and scanned conservatively. Rust
attributes inside quoted source, such as `#[allow(...)]` and `#![...]`, are
treated as source syntax rather than quote interpolation; real `#name`/`#(...)`
interpolation stays unknown because final generated identifiers depend on
runtime build-script values.

Root-level `include!(concat!(env!("OUT_DIR"), ...))` needs a pre-render pass,
not just a final hazard report. When selected code calls a function defined by
generated source, the include macro itself is unnamed and would otherwise be
pruned before its generated helper dependencies can block deletion. The scanner
now records generated-source declaration/candidate identifiers separately from
helper blockers, retains the top-level OUT_DIR include only when retained code
mentions one of those generated identifiers, and still blocks only the helper
paths found inside the generated source.

Support proc-macro expansion evidence must be scoped to the exported macro that
is actually live. A proc-macro crate can define `LiveDerive` and `DeadDerive`
next to each other, and both may contain `quote!` bodies that reference helpers
in the consuming support package. Treating every quote body in that proc-macro
crate as live when only one derive is used over-retains dead helper modules in
the copied support package. The support scanner now records quote-body
dependencies per exported macro and follows helper functions called by that
export before merging generated-source dependency evidence.

Workspace patches should be copied for retained dependency names, not because
they exist in `[patch]`. A root or support workspace can carry extra patch path
entries for packages unrelated to the selected API. Broadly discovering every
patch path creates redundant `support/` packages even when the generated root
manifest later filters the `[patch]` table. Patch discovery is now driven by the
dependency package name encountered in retained root/support manifests; retained
patched transitive dependencies still get copied, while unused patch entries are
left out of both the root manifest and support tree.

The FFI manifest bundle needs to be tested as one shape, not as isolated
manifest features. A selected `cfg_attr(feature = "ffi", uniffi::export)` root
can require `cdylib`/`staticlib` crate types, optional macro/runtime feature
edges, target-specific version dependencies resolved through `[patch]`,
lockfiles, and `rust-toolchain.toml` at the same time. The combined fixture now
checks that all live context is preserved while dead optional dependencies and
unused patch packages stay out of the generated workspace.

External protocol enum projection needs variant requirements, not whole-enum
support retention. Root code and copied support packages can mention
`DependencyEnum::Variant` through direct imports, public glob facades, renamed
type aliases, or another support crate. Those variant names now become
first-class support live-set requirements, and match patterns over dependency
enums bind payload variables back to their concrete dependency payload types.
That lets the slicer retain exact payload methods such as `session_id()` or
`text()` while pruning dead variants, dead payload structs, and dead payload
methods from transitive support packages.

Top-down fallback must stay typed. A JSON patch state-machine fixture showed
that `serde_json::Map::new()` was being treated as an unresolved name-only
method fallback, which retained an unrelated local `DeadState::new` and its
dead module. The reducer now carries the associated receiver type into
unresolved fallback matching. External/std constructors such as
`std::collections::HashMap::new()` and dependency constructors such as
`serde_json::Map::new()` therefore produce zero local candidates instead of
pulling same-name local methods into the slice. This keeps the support closure
aligned with the top-down invariant: selected roots expand only through typed
or explicitly unknown dependencies.

Top-down static support needs dependency-crate proof, not only local static
proof. The `static.lazy_regex_constructor.001` fixture models a Litter-shaped
`LazyLock<Regex>` parser with a separate regex-like path dependency. The
selected root walks through the static initializer, keeps `Regex::new`,
`Regex::captures`, `Captures::name`, `Match::as_str`, and
`RegexError::message`, and prunes unrelated `replace_all`/dead-regex APIs.
This is the important production shape: static singletons can pull a helper
crate into the slice, but that crate still must be reduced item-by-item under
the same used/unknown/prunable contract.

Global registries need operation-level proof. The
`static.global_mutex_registry.001` fixture models an
`OnceLock<Mutex<BTreeMap<String, Arc<_>>>>` registry with register, lookup,
remove, and list APIs. A selected register-plus-lookup root keeps the static,
lock helper, entry constructor, lookup return surface, and `render` method, but
still prunes remove/list/dead render APIs. This guards against a tempting but
wrong whole-registry retention rule: the static registry itself is live, but
each operation hanging off it still needs top-down evidence or a scoped unknown
blocker.
