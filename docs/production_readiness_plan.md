# Production Readiness Plan

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
- Unknown macros should not be pruned silently. Either retain bounded source,
  query a semantic oracle, or fail with an unsupported-construct report.
- Build scripts can execute arbitrary project logic and generate source under
  `OUT_DIR`; copied `build.rs` files are not enough for semantic modeling, so
  retained `OUT_DIR` Rust includes must fail closed until a semantic oracle
  models generated source.
- Feature and platform cfgs form a matrix, not a boolean. A slice should report
  the feature/platform configuration it was generated for.
- Dependency aliases, workspace dependencies, optional dependency features, and
  target-specific dependencies need Cargo's resolver model.
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
     workspace provides a lockfile.

3. Feedback widening:
   - Parse `cargo check --message-format=json`.
   - Classify missing item/module/import/dependency/feature failures.
   - Widen the retained graph with a bounded iteration cap.
   - Apply only rustc `MachineApplicable` suggestions whose spans are inside
     the generated output root.
   - Treat selected warnings, especially unreachable patterns, as semantic hazards.

4. Semantic oracle:
   - Add narrow queries for path resolution, method resolution, trait impl lookup,
     macro-expanded item inventory, and active cfg file/module inventory.
   - Keep the syntactic reducer available for simple workspaces and offline runs.

5. Corpus validation:
   - Pin real repositories and root selections.
   - Run original baseline checks before slicing.
   - Generate slices, run feedback checks, and compare selected behavior probes.
   - Store retained/pruned inventories and failure reports.

## Current First Step

This branch starts with the safety gate because it removes destructive failure
modes before larger resolver work begins. It also begins the Cargo metadata
substrate by using `cargo metadata --no-deps` for workspace member, target entry,
and dependency discovery, while preserving the existing TOML renderer for slice
manifests.
