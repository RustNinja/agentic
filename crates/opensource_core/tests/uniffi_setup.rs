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

#[test]
fn prunes_unrelated_uniffi_sibling_modules() {
    let workspace = temp_path("sibling-workspace");
    let output = temp_path("sibling-output");
    write_sibling_module_fixture_workspace(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let lib = read(output.join("app/src/lib.rs"));
    assert!(lib.contains("pub mod selected"));
    assert!(
        !lib.contains("pub mod heavy"),
        "dead sibling module kept:\n{lib}"
    );
    assert!(output.join("app/src/selected.rs").exists());
    assert!(
        !output.join("app/src/heavy.rs").exists(),
        "dead UniFFI sibling source should not be written"
    );
}

#[test]
fn prunes_unmarked_root_uniffi_exports_when_submodule_is_selected() {
    let workspace = temp_path("root-export-workspace");
    let output = temp_path("root-export-output");
    write_root_export_fixture_workspace(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let lib = read(output.join("app/src/lib.rs"));
    assert!(lib.contains("pub mod selected"));
    assert!(
        !lib.contains("pub mod heavy"),
        "dead dependency kept:\n{lib}"
    );
    assert!(
        !lib.contains("unmarked_root_export"),
        "unmarked root UniFFI export should not become a slice root:\n{lib}"
    );
    assert!(output.join("app/src/selected.rs").exists());
    assert!(!output.join("app/src/heavy.rs").exists());
}

#[test]
fn prunes_unrelated_public_glob_reexport_modules() {
    let workspace = temp_path("glob-reexport-workspace");
    let output = temp_path("glob-reexport-output");
    write_glob_reexport_fixture_workspace(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let lib = read(output.join("app/src/lib.rs"));
    assert!(lib.contains("pub mod selected"));
    assert!(
        !lib.contains("pub use facade::*"),
        "dead public glob reexport should be pruned:\n{lib}"
    );
    assert!(
        !lib.contains("pub mod facade"),
        "dead facade module should be pruned:\n{lib}"
    );
    assert!(output.join("app/src/selected.rs").exists());
    assert!(!output.join("app/src/facade.rs").exists());
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

fn write_sibling_module_fixture_workspace(root: &Path) {
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
        r#"pub mod selected;
pub mod heavy;

uniffi::setup_scaffolding!();
"#,
    );
    write(
        root.join("app/src/selected.rs"),
        r#"use opensourced::opensourced;

#[derive(Debug, uniffi::Record)]
pub struct SelectedRecord {
    pub value: String,
}

#[opensourced]
#[uniffi::export]
pub fn selected_value(value: String) -> SelectedRecord {
    SelectedRecord { value }
}
"#,
    );
    write(
        root.join("app/src/heavy.rs"),
        r#"#[derive(Debug, uniffi::Record)]
pub struct HeavyRecord {
    pub value: String,
}

#[uniffi::export]
pub fn heavy_value(value: String) -> HeavyRecord {
    HeavyRecord { value }
}
"#,
    );
}

fn write_root_export_fixture_workspace(root: &Path) {
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
        r#"pub mod selected;
pub mod heavy;

pub use heavy::HeavyRecord;

#[uniffi::export]
pub fn unmarked_root_export(value: String) -> HeavyRecord {
    HeavyRecord { value }
}

uniffi::setup_scaffolding!();
"#,
    );
    write(
        root.join("app/src/selected.rs"),
        r#"use opensourced::opensourced;

#[derive(Debug, uniffi::Record)]
pub struct SelectedRecord {
    pub value: String,
}

#[opensourced]
#[uniffi::export]
pub fn selected_value(value: String) -> SelectedRecord {
    SelectedRecord { value }
}
"#,
    );
    write(
        root.join("app/src/heavy.rs"),
        r#"#[derive(Debug, uniffi::Record)]
pub struct HeavyRecord {
    pub value: String,
}
"#,
    );
}

fn write_glob_reexport_fixture_workspace(root: &Path) {
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
        r#"pub mod selected;
pub mod facade;

pub use facade::*;

uniffi::setup_scaffolding!();
"#,
    );
    write(
        root.join("app/src/selected.rs"),
        r#"use opensourced::opensourced;

#[derive(Debug, uniffi::Record)]
pub struct SelectedRecord {
    pub value: String,
}

#[opensourced]
#[uniffi::export]
pub fn selected_value(value: String) -> SelectedRecord {
    SelectedRecord { value }
}
"#,
    );
    write(
        root.join("app/src/facade.rs"),
        r#"#[derive(Debug, uniffi::Record)]
pub struct UnrelatedRecord {
    pub value: String,
}

#[uniffi::export]
pub fn unrelated_value(value: String) -> UnrelatedRecord {
    UnrelatedRecord { value }
}
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
