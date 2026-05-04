# Scripts

## Generic Corpus Feedback Loop

`corpus_feedback_loop.py` is the project-agnostic production verification
harness. It discovers local Rust package targets with `cargo metadata`, selects
random candidate roots, injects `#[opensourced::opensourced]`, runs `slicers`
with `--slice-report` plus either fast preflight or compiler feedback, then
appends one JSONL metrics row per batch.

The runner defaults to `--analyzer ra-hir`, which builds the CLI with the
default rust-analyzer HIR feature and records semantic inventory. Pass
`--analyzer syn` for the fast syntactic fallback, or
`--analyzer ra-hir-proc-macros` when a corpus run should exercise
rust-analyzer build-script output discovery and the sysroot proc-macro server.
RA exact project-local method/path resolutions are part of the default corpus
flow and are applied as additive reduction hints. The fast RA load is bounded
to local workspace crates by default. Proc-macro mode may load dependency
metadata/build artifacts for discovery when
`OPENSOURCE_RA_PROC_MACRO_LOAD_DEPS=1` is set, but that heavier pass timed out
on the pinned Litter corpus. Without that opt-in, the semantic walk still visits
only workspace Rust files, analyzes files containing selected roots first, and
reports when proc-macro expansion is unavailable.

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

Use `--roots-file path/to/cases.json` for pinned real-repo batches instead of
random roots. The JSON can contain `batches`, each with a `name`, optional
`cargo_check_args`, and root selectors using `path` plus `name`, `kind`, `line`,
`package`, or `target`. When `--roots-file` is provided without
`--max-batches`, the runner executes all pinned batches. The checked-in Litter
smoke set can be run against the pinned checkout like this:

```sh
scripts/corpus_feedback_loop.py \
  --source /tmp/litter/shared/rust-bridge \
  --roots-file scripts/corpus_cases/litter_uniffi.json \
  --output-prefix /tmp/slicers-litter-pinned \
  --validation production \
  --feedback-target-dir /tmp/slicers-litter-target \
  --report reports/litter_uniffi.jsonl
```

By default it restores and cleans git-backed sources before and after each
batch, keeps failed outputs for debugging, removes generated `target-feedback`,
and prunes older successful outputs. Use `--baseline-check` to separate source
environment failures from slicer failures; add `--allow-baseline-failures` when
known-broken sources should still run and generated baseline-matching errors
should be classified separately from slicer regressions. Use `--continuous` for
soak runs. New semantic hazard warnings, including unreachable or irrefutable
patterns and non-snake-case pattern bindings introduced by pruning, are
classified as slicer failures even when general warnings are allowed.
Use `--validation preflight` when build time is the bottleneck; it validates the
predicted generated shape without compiling dependencies. Use
`--validation feedback` for slower compiler-confirmed runs,
`--validation repair` to run the bounded compiler repair loop, or
`--validation production` to run the strict slicers production preset. Use
`--feedback-target-dir` on feedback, repair, or production corpus runs to reuse
dependency builds across generated outputs. Use repeated `--cargo-check-arg`
values to validate feature or target matrix cases such as `--all-features` or
`--target wasm32-unknown-unknown`; production validation also rejects
non-default targets whose `required-features` are not activated by those
arguments, preserves pinned Rust toolchain files, and adds Cargo `--locked` when
the source checkout has a lockfile. Generated slices also preserve root
`[profile.*]` policy and refuse to copy symlinked assets that resolve outside
their package root. They copy retained non-workspace path packages into
`support/` and rewrite copied package manifests so local path dependency
closures stay self-contained. They also fail production on file includes with
unresolved env paths, absolute paths, external paths, or `OUT_DIR` generated
assets, while retained macros and non-root cfg surfaces are preserved and
require compiler feedback. Selected roots behind concrete Cargo feature cfg
gates are reported with structured details and can proceed to feedback when the
same corpus run passes matching `--features` values or `--all-features`; cfg
gates without feature hints still fail production before feedback. Retained
non-root feature cfg hints now trigger an additional production matrix check,
with source and generated matrix reports recorded next to the primary feedback
report, so corpus rows can distinguish primary feedback from feature-surface
validation. Use
`--deny-warnings` for production gates that
require warning-free generated feedback. Selected root cfg gates can now be
discharged when recognized `all(...)`, `any(...)`, and `not(...)` feature/target
expressions are proven by the validation args and host or explicit target
`rustc --print cfg`; custom cfgs still fail closed. Each corpus row includes
the slicer's authoritative validation verdict, production-readiness status,
production hazard codes and structured hazard details, compiler feedback
widening candidate/hazard kinds, feedback-widened root counts, compiler
suggestion counts, and conservative repair totals including applied
machine-applicable suggestions.
Feedback cargo checks drain stdout/stderr while Cargo runs, so large
`--message-format=json` output from real dependency graphs does not block the
child process. Repair validation removes repairable warnings, including unused
imports, before accepting a successful generated check.
Dynamic dispatch blockers such as retained `fn(...)` pointer surfaces and
`dyn Trait` objects include package/module/file/line details and the retained
surface text in the structured hazard payload.
Retained Rust `include!`, unresolved/external/absolute include assets, OUT_DIR
include assets, and unmodeled `env!`/`option_env!` calls also include the same
location details.
Feedback-required macro hazards for retained custom attributes, custom derives,
and non-builtin macro invocations include source locations as well.
Retained build-script blockers include the package and build script path.
Uncopyable retained path-dependency and workspace patch/replace blockers include
manifest paths, owning packages when applicable, and dependency subjects.

## Litter Feedback Loop

`litter_feedback_loop.py` continuously restores a Litter checkout, marks five
random Rust roots with `#[opensourced::opensourced]`, runs the slicer with cargo
feedback, removes the generated `target-feedback`, and repeats until a slice
emits an error or warning.

```sh
scripts/litter_feedback_loop.py \
  --source /tmp/rustninja-litter-sparse/shared/rust-bridge \
  --output-prefix /tmp/rustninja-litter-loop \
  --start 1
```

The script discovers candidates from all module-reachable workspace packages,
prefers one `fn`, `mod`, `trait`, `struct`, and `enum` root per batch, injects
the local `opensourced` dependency only into selected packages, keeps failing
outputs for debugging, and prunes older successful outputs by default.
