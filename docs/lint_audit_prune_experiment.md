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

Optional rust-analyzer audit:

```sh
slicers --lint-audit --ra-audit \
  --rust-analyzer /path/to/rust-analyzer \
  --ra-proc-macro-srv /path/to/rust-analyzer-proc-macro-srv \
  --ra-disable-build-scripts \
  <workspace-root> <output-root>
```

`--ra-audit` writes `slicers-rust-analyzer-report.json` next to the cargo
lint report. The rust-analyzer binary is resolved in this order:

1. `--rust-analyzer <path>`
2. `SLICERS_RUST_ANALYZER`
3. `rust-analyzer` on `PATH`

The optional proc-macro server path is resolved in this order:

1. `--ra-proc-macro-srv <path>`
2. `SLICERS_RA_PROC_MACRO_SRV`

Use `--ra-disable-proc-macros` for a faster/noisier audit that does not expand
proc macros. When proc macros are disabled, any configured proc-macro server is
ignored.

The rust-analyzer audit is best treated as an extra signal source. It can
report diagnostics and unresolved references from rust-analyzer's project model,
which is useful for macro-heavy code and incomplete slices, but it is not a
replacement for `cargo check`.

Use a rust-analyzer build that matches the active Rust toolchain when possible.
An older rust-analyzer can still run, but proc macro expansion and some type
diagnostics may contain version-mismatch noise.

This is intentionally an audit mode, not a replacement for graph slicing yet.
Rust lints are useful for private dead code and unreachable public items, but
they cannot prove that every public API, trait impl, macro-generated item, or
FFI surface is unused. The next experiment step is to consume the diagnostics
and remove reported spans in repeated cargo-check rounds, while keeping the
existing graph reducer as the correctness backstop.
