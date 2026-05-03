# Scripts

## Generic Corpus Feedback Loop

`corpus_feedback_loop.py` is the project-agnostic production verification
harness. It discovers local Rust package targets with `cargo metadata`, selects
random candidate roots, injects `#[opensourced::opensourced]`, runs `slicers`
with `--slice-report` plus either fast preflight or compiler feedback, then
appends one JSONL metrics row per batch.

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
dependency builds across generated outputs. Use `--deny-warnings` for
production gates that require warning-free generated feedback. Each corpus row
includes production-readiness status, production hazard codes, and compiler
feedback widening candidate/hazard kinds.

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
