# Lint Audit Prune Experiment

This branch experiments with a compiler-assisted pruning path:

1. Copy the source workspace broadly instead of reducing it immediately.
2. Preserve `Cargo.lock` and rewrite path dependencies that point outside the
   copied workspace to absolute source paths.
3. Inject crate-level lint probes into package roots:
   `dead_code`, `unused_imports`, `unused_macros`, and `unreachable_pub`.
4. Run `cargo check --message-format=json`.
5. Write `slicers-lint-audit-report.json` with compiler diagnostics that can
   drive later removal passes.

Run it with:

```sh
slicers --lint-audit <workspace-root> <output-root>
```

This is intentionally an audit mode, not a replacement for graph slicing yet.
Rust lints are useful for private dead code and unreachable public items, but
they cannot prove that every public API, trait impl, macro-generated item, or
FFI surface is unused. The next experiment step is to consume the diagnostics
and remove reported spans in repeated cargo-check rounds, while keeping the
existing graph reducer as the correctness backstop.
