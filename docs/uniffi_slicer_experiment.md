# UniFFI Slicer Experiment

## Reference Shape

The production reference shape is Mozilla-style UniFFI: a Rust core exposes a
small FFI boundary to Swift/Kotlin/Python/etc. The concrete references used for
the fixture design were:

- `mozilla/uniffi-rs`: the UniFFI binding generator.
- `mozilla/application-services`: Mozilla's production Rust services workspace
  where Rust components are exposed to mobile clients through UniFFI-style FFI.
- `dnakov/litter`: a compact mobile app workspace with a shared Rust bridge and
  UniFFI-generated bindings.

The repository does not vendor those projects. The proof is a deterministic
local fixture that mirrors the shape without requiring a large third-party clone
or network-dependent test.

## Local Fixture

The fixture lives in `crates/opensource_core/tests/uniffi_mobile.rs` and builds a
temporary four-crate workspace:

```text
mobile_api -> sync_core -> crypto_box
mobile_api -> diagnostics
```

Only `mobile_api::sync_preview` is marked with `#[opensourced]`.

The fixture includes:

- UniFFI-shaped `cfg_attr(feature = "ffi", uniffi::...)` API annotations;
- FFI DTO structs/enums with serde derives;
- object-like inherent impl methods;
- trait impl calls, including `Trait::method(&receiver, ...)`;
- transitive cross-crate helper calls;
- a direct but unused local `diagnostics` dependency;
- unit tests and `#[cfg(test)]` modules that must not render.

## Expected Slice

Generated packages:

```text
crypto_box
mobile_api
sync_core
```

Pruned package:

```text
diagnostics
```

Preserved external dependency:

```text
serde = { version = "1.0", features = ["derive"] }
```

The generated `mobile_api` manifest keeps `sync_core` and `serde`, removes
`opensourced` and `diagnostics`, and the generated source strips the
`#[opensourced]` marker while preserving the inactive UniFFI `cfg_attr`
annotations.

## Verified Commands

```sh
cargo test --workspace
cargo run -p opensource_cli --bin slicers -- . /tmp/slicers-proof
cargo check --manifest-path /tmp/slicers-proof/Cargo.toml
```

All commands passed after the slicer changes.
