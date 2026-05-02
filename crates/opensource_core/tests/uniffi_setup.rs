use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use opensource_core::{generate, GenerateOptions};

#[test]
fn retains_uniffi_setup_scaffolding_for_exported_surface() {
    let workspace = temp_path("workspace");
    let output = temp_path("output");
    write_fixture_workspace(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let source = read(output.join("app/src/lib.rs"));
    assert!(
        source.contains("uniffi::setup_scaffolding!"),
        "UniFFI setup macro must be retained when exported UniFFI items remain:\n{source}",
    );
    assert!(
        source.contains("#[uniffi::export]"),
        "exported UniFFI root should remain:\n{source}",
    );
    assert!(
        !source.contains("#[opensourced]"),
        "marker attribute must be stripped:\n{source}",
    );

    let manifest = read(output.join("app/Cargo.toml"));
    assert!(
        manifest.contains("uniffi"),
        "uniffi dep should remain:\n{manifest}"
    );
    assert!(
        !manifest.contains("opensourced"),
        "marker dependency should be pruned:\n{manifest}",
    );
}

#[test]
fn drops_uniffi_scaffolding_for_plain_rust_custom_type_slice() {
    let workspace = temp_path("custom-type-workspace");
    let output = temp_path("custom-type-output");
    write_custom_type_fixture_workspace(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let source = read(output.join("app/src/lib.rs"));
    assert!(
        source.contains("pub struct ApiTimestamp"),
        "reachable Rust type should remain:\n{source}",
    );
    assert!(
        !source.contains("uniffi::custom_type!"),
        "plain Rust slices should not retain UniFFI-only custom type macros:\n{source}",
    );
    assert!(
        !source.contains("uniffi::include_scaffolding!"),
        "plain Rust slices should not retain full UDL scaffolding:\n{source}",
    );
    assert!(
        !source.contains("DeadTimestamp"),
        "dead custom types should still be pruned:\n{source}",
    );
}

fn write_fixture_workspace(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }

    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[workspace]
members = ["app"]
resolver = "2"

[workspace.dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );

    write(
        root.join("app/Cargo.toml"),
        r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["lib", "cdylib"]

[dependencies]
opensourced.workspace = true
uniffi = "0.31.0"
"#,
    );

    write(
        root.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[derive(Debug, uniffi::Record)]
pub struct ApiRecord {
    pub value: String,
}

#[opensourced]
#[uniffi::export]
pub fn exported_value(value: String) -> ApiRecord {
    ApiRecord { value }
}

pub fn dead_value() -> String {
    "dead".to_string()
}

uniffi::setup_scaffolding!();
"#,
    );
}

fn write_custom_type_fixture_workspace(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }

    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[workspace]
members = ["app"]
resolver = "2"

[workspace.dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );

    write(
        root.join("app/Cargo.toml"),
        r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced.workspace = true
uniffi = "0.31.0"
"#,
    );

    write(
        root.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

pub struct ApiTimestamp(pub u64);

pub struct DeadTimestamp(pub u64);

#[opensourced]
pub fn selected_timestamp() -> ApiTimestamp {
    ApiTimestamp(1)
}

uniffi::custom_type!(ApiTimestamp, i64, {
    lower: |obj| obj.0 as i64,
    try_lift: |value| Ok(Self(value as u64)),
});

uniffi::include_scaffolding!("app");

uniffi::custom_type!(DeadTimestamp, i64, {
    lower: |obj| obj.0 as i64,
    try_lift: |value| Ok(Self(value as u64)),
});
"#,
    );
}

fn write(path: PathBuf, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

fn read(path: PathBuf) -> String {
    fs::read_to_string(path).unwrap()
}

fn temp_path(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("opensourced-uniffi-setup-{label}-{unique}"))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn manifest_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "\\\\")
}
