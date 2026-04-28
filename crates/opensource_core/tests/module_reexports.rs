use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use opensource_core::{generate, GenerateOptions};

#[test]
fn reduces_external_and_nested_modules_with_aliases() {
    let workspace = temp_path("workspace");
    let output = temp_path("output");
    write_fixture_workspace(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace.clone(),
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let reachable = report
        .reachable
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    for expected in [
        "app::exported_entry",
        "app::service::serve",
        "core::domain::ReachableModel::new",
        "core::domain::nested::normalize_kind",
        "core::worker::process_model",
        "leaf::format_label",
        "leaf::math::adjust",
    ] {
        assert!(
            reachable.iter().any(|actual| actual == expected),
            "missing reachable callable {expected}; got {reachable:?}",
        );
    }

    for not_expected in [
        "app::unused_app_function",
        "app::service::unused_service_function",
        "core::unused_core_root",
        "core::domain::unused_domain_function",
        "core::domain::nested::unused_nested_function",
        "core::domain::ReachableModel::unused_method",
        "core::worker::unused_worker_function",
        "leaf::unused_leaf_root",
        "leaf::math::unused_math",
    ] {
        assert!(
            !reachable.iter().any(|actual| actual == not_expected),
            "unreachable callable {not_expected} was retained in graph",
        );
    }

    let reachable_items = report
        .reachable_items
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    for expected in [
        "core::domain::ReachableModel(Struct)",
        "core::domain::nested::Kind(Enum)",
    ] {
        assert!(
            reachable_items.iter().any(|actual| actual == expected),
            "missing reachable item {expected}; got {reachable_items:?}",
        );
    }
    assert!(
        !reachable_items
            .iter()
            .any(|actual| actual == "core::domain::UnusedModel(Struct)"),
        "unreachable struct was retained in item graph",
    );

    let app_lib = read_output(&output, "app/src/lib.rs");
    assert!(app_lib.contains("mod service;"));
    assert!(app_lib.contains("pub fn exported_entry"));
    assert_not_present(
        &app_lib,
        [
            "opensourced",
            "unused_app_function",
            "cfg(test)",
            "app_test_only",
        ],
    );

    let app_service = read_output(&output, "app/src/service.rs");
    assert!(app_service.contains("ReachableModel as Model"));
    assert!(app_service.contains("process_model as run_model"));
    assert!(app_service.contains("pub fn serve"));
    assert_not_present(
        &app_service,
        ["unused_service_function", "cfg(test)", "service_test_only"],
    );

    let core_lib = read_output(&output, "core/src/lib.rs");
    assert!(core_lib.contains("pub mod domain;"));
    assert!(core_lib.contains("pub mod worker;"));
    assert!(core_lib.contains("pub use domain::ReachableModel;"));
    assert_not_present(
        &core_lib,
        ["unused_core_root", "cfg(test)", "core_test_only"],
    );

    let core_domain = read_output(&output, "core/src/domain/mod.rs");
    assert!(core_domain.contains("mod nested;"));
    assert!(core_domain.contains("pub use nested::Kind;"));
    assert!(core_domain.contains("pub struct ReachableModel"));
    assert!(core_domain.contains("pub fn new"));
    assert_not_present(
        &core_domain,
        [
            "UnusedModel",
            "unused_domain_function",
            "unused_method",
            "cfg(test)",
            "domain_test_only",
        ],
    );

    let core_nested = read_output(&output, "core/src/domain/nested.rs");
    assert!(core_nested.contains("pub enum Kind"));
    assert!(core_nested.contains("pub fn normalize_kind"));
    assert_not_present(
        &core_nested,
        ["unused_nested_function", "cfg(test)", "nested_test_only"],
    );

    let core_worker = read_output(&output, "core/src/worker.rs");
    assert!(core_worker.contains("pub fn process_model"));
    assert_not_present(
        &core_worker,
        ["unused_worker_function", "cfg(test)", "worker_test_only"],
    );

    let leaf_lib = read_output(&output, "leaf/src/lib.rs");
    assert!(leaf_lib.contains("pub mod math;"));
    assert!(leaf_lib.contains("pub fn format_label"));
    assert_not_present(
        &leaf_lib,
        ["unused_leaf_root", "cfg(test)", "leaf_test_only"],
    );

    let leaf_math = read_output(&output, "leaf/src/math.rs");
    assert!(leaf_math.contains("pub fn adjust"));
    assert_not_present(&leaf_math, ["unused_math", "cfg(test)", "math_test_only"]);

    let target_dir = temp_path("target");
    let status = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", target_dir)
        .status()
        .expect("cargo check should start");

    assert!(status.success(), "generated workspace did not compile");
}

fn write_fixture_workspace(root: &Path) {
    let opensourced_path = repo_root().join("crates/opensourced");
    let opensourced_path = toml_path(&opensourced_path);

    write_file(
        &root.join("Cargo.toml"),
        r#"[workspace]
members = ["app", "core", "leaf"]
resolver = "2"

[workspace.package]
edition = "2021"
version = "0.1.0"
license = "MIT"
"#,
    );

    write_file(
        &root.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
core_api = {{ package = "core", path = "../core" }}
opensourced = {{ path = "{opensourced_path}" }}
"#
        ),
    );
    write_file(
        &root.join("app/src/lib.rs"),
        r#"mod service;

#[opensourced::opensourced]
pub fn exported_entry(input: u32) -> String {
    service::serve(input)
}

pub fn unused_app_function() -> String {
    "unused app".to_string()
}

#[cfg(test)]
pub fn app_test_only() -> String {
    "test only".to_string()
}
"#,
    );
    write_file(
        &root.join("app/src/service.rs"),
        r#"use core_api::domain::ReachableModel as Model;
use core_api::worker::process_model as run_model;

pub fn serve(input: u32) -> String {
    let model = Model::new(input);
    run_model(model)
}

pub fn unused_service_function() -> String {
    "unused service".to_string()
}

#[cfg(test)]
pub fn service_test_only() -> String {
    "test only".to_string()
}
"#,
    );

    write_file(
        &root.join("core/Cargo.toml"),
        r#"[package]
name = "core"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
leaf = { path = "../leaf" }
"#,
    );
    write_file(
        &root.join("core/src/lib.rs"),
        r#"pub mod domain;
pub mod worker;

pub use domain::ReachableModel;

pub fn unused_core_root() -> u32 {
    0
}

#[cfg(test)]
pub fn core_test_only() -> u32 {
    1
}
"#,
    );
    write_file(
        &root.join("core/src/domain/mod.rs"),
        r#"mod nested;

pub use nested::Kind;

pub struct ReachableModel {
    pub value: u32,
    pub kind: Kind,
}

impl ReachableModel {
    pub fn new(value: u32) -> Self {
        Self {
            value,
            kind: nested::normalize_kind(value),
        }
    }

    pub fn unused_method(&self) -> u32 {
        self.value + 100
    }
}

pub struct UnusedModel;

pub fn unused_domain_function() -> u32 {
    2
}

#[cfg(test)]
pub fn domain_test_only() -> u32 {
    3
}
"#,
    );
    write_file(
        &root.join("core/src/domain/nested.rs"),
        r#"use leaf::math::adjust as tweak;

pub enum Kind {
    Even,
    Odd,
}

pub fn normalize_kind(value: u32) -> Kind {
    if tweak(value) % 2 == 0 {
        Kind::Even
    } else {
        Kind::Odd
    }
}

pub fn unused_nested_function() -> Kind {
    Kind::Odd
}

#[cfg(test)]
pub fn nested_test_only() -> Kind {
    Kind::Even
}
"#,
    );
    write_file(
        &root.join("core/src/worker.rs"),
        r#"use crate::domain::ReachableModel as Model;
use leaf::format_label as label;

pub fn process_model(model: Model) -> String {
    label(model.value)
}

pub fn unused_worker_function() -> String {
    "unused worker".to_string()
}

#[cfg(test)]
pub fn worker_test_only() -> String {
    "test only".to_string()
}
"#,
    );

    write_file(
        &root.join("leaf/Cargo.toml"),
        r#"[package]
name = "leaf"
version.workspace = true
edition.workspace = true
license.workspace = true
"#,
    );
    write_file(
        &root.join("leaf/src/lib.rs"),
        r#"pub mod math;

pub fn format_label(value: u32) -> String {
    format!("leaf-{}", math::adjust(value))
}

pub fn unused_leaf_root() -> String {
    "unused leaf".to_string()
}

#[cfg(test)]
pub fn leaf_test_only() -> String {
    "test only".to_string()
}
"#,
    );
    write_file(
        &root.join("leaf/src/math.rs"),
        r#"pub fn adjust(value: u32) -> u32 {
    value + 1
}

pub fn unused_math() -> u32 {
    4
}

#[cfg(test)]
pub fn math_test_only() -> u32 {
    5
}
"#,
    );
}

fn assert_not_present<const N: usize>(source: &str, needles: [&str; N]) {
    for needle in needles {
        assert!(
            !source.contains(needle),
            "unexpected `{needle}` in reduced source:\n{source}",
        );
    }
}

fn read_output(root: &Path, relative: &str) -> String {
    fs::read_to_string(root.join(relative))
        .unwrap_or_else(|error| panic!("failed to read {relative}: {error}"))
}

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("core crate should live under crates/opensource_core")
        .to_path_buf()
}

fn temp_path(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("opensourced-module-reexports-{label}-{unique}"));
    if path.exists() {
        fs::remove_dir_all(&path).unwrap();
    }
    path
}

fn toml_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "\\\\")
}
