# Scripts

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
