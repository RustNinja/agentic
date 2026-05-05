# Generic Rule Catalog

The rule catalog is the scalable layer above the executable fixture tests. It
records broad Rust/Cargo combinations that the slicer must handle without
turning every combination into a slow, hand-written integration test.

`crates/opensource_core/tests/rule_catalog.rs` currently generates 1,200
generic rule records and validates them in milliseconds. The records are not
Litter-specific. They are composed from Rust/Cargo axes such as root kind, edge
shape, dependency surface, validation mode, expected behavior, and failure mode.

## Catalog Axes

| Axis | Examples |
| --- | --- |
| Group | `import`, `macro`, `trait`, `dyn`, `include`, `build`, `uniffi`, `manifest`, `repair`, `cfg` |
| Root kind | `free_fn`, `method`, `struct`, `enum`, `trait`, `module`, `binary`, `ffi_export` |
| Edge shape | direct path, receiver method, UFCS, associated projection, reexport, macro body, derive helper attr, source include, cfg gate |
| Dependency surface | local module, workspace package, external crate, path package, build dependency, proc macro dependency, target dependency, asset |
| Validation mode | `syn`, `ra_hir`, `ra_feedback`, `production`, `no_default_features`, `feature_target` |
| Expected behavior | retain live closure, prune dead siblings, copy assets/support packages, report hazards, repair feedback, move cfg intact |
| Failure mode | missing live edge, dead retention, malformed source, missing asset, overcopied support, unmodeled macro/dyn/generated source, cfg not proven |

## Coverage Families

Every generated catalog row is also mapped to an enforceable coverage family.
The validator keeps that mapping honest: each row has a family, an executable
seed rule, and an enforcement level. The levels are:

| Enforcement | Meaning |
| --- | --- |
| `executable_fixture` | At least one representative `rule_database.rs` fixture asserts output pruning and usually runs `cargo check` |
| `production_hazard_fixture` | A representative fixture asserts the structured warning/error that keeps production fail-closed |
| `catalog_guard` | A policy guard covers broad combinations that should not explode into a fixture matrix, such as cfg movement and compiler-feedback convergence |

This means the 1,200 records are coverage obligations. They are not counted as
1,200 generated-workspace checks. One high-quality executable fixture can cover
many axis combinations by family, but the docs and tests keep that distinction
explicit.

## Promotion Workflow

When a real repo probe fails or retains too much output:

1. Map the failure to a catalog entry by group, root kind, edge shape, dependency
   surface, validation mode, expected behavior, and failure mode.
2. Add or update a tiny executable fixture in `rule_database.rs` only if the
   combination exposes a distinct reducer/render/manifest behavior.
3. Fix the slicer generically. Do not add project-name or symbol-name allowlists.
4. Run `scripts/fast_rule_catalog_loop.sh`.
5. Record the learning in `docs/rule_database.md`,
   `docs/slice_coverage_matrix.md`, or the relevant real-repo probe report.

This keeps the fast loop aggressive: the catalog can hold thousands of planned
combinations, while executable fixtures stay focused on behavior that actually
protects the slicer.

## Cfg Policy

The catalog includes cfg-gate combinations, but it does not expand a giant
custom cfg matrix. The current production-safe policy is:

- Move cfg attributes on retained live items intact unless validation proves the
  surface is inactive.
- Treat selected roots behind unproven cfg predicates as production-blocking.
- Use bounded `--features`, `--all-features`, host/target `rustc --print cfg`,
  and recognized `all(...)`, `any(...)`, `not(...)` forms.
- Keep custom cfg predicates fail-closed until a semantic oracle proves them.
