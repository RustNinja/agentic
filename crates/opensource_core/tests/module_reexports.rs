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
    for not_expected in [
        "core::domain::UnusedModel(Struct)",
        "core::domain::nested::UnusedKind(Enum)",
    ] {
        assert!(
            !reachable_items.iter().any(|actual| actual == not_expected),
            "unreachable item {not_expected} was retained in item graph",
        );
    }

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
            "UnusedKind",
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
        [
            "UnusedKind",
            "unused_nested_function",
            "cfg(test)",
            "nested_test_only",
        ],
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

#[test]
fn keeps_private_parent_reexports_used_by_rendered_child_modules() {
    let workspace = temp_path("parent-reexport-workspace");
    let output = temp_path("parent-reexport-output");
    write_parent_reexport_fixture_workspace(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let ssh_mod = read_output(&output, "app/src/ssh/mod.rs");
    assert!(
        ssh_mod.contains("pub(crate) use crate::ssh_scripts::posix::{"),
        "parent reexport group was not retained:\n{ssh_mod}",
    );
    assert!(
        ssh_mod.contains("PACKAGE_MANAGER_PROBE"),
        "missing child-imported reexport:\n{ssh_mod}",
    );
    assert!(
        ssh_mod.contains("PROFILE_INIT"),
        "missing child-imported reexport:\n{ssh_mod}",
    );
    assert_not_present(&ssh_mod, ["UNUSED_SCRIPT_CONST", "unused_ssh"]);

    let child = read_output(&output, "app/src/ssh/codex_binary.rs");
    assert!(
        child.contains("use super::{PACKAGE_MANAGER_PROBE, PROFILE_INIT};"),
        "child import should remain wired through the parent reexport:\n{child}",
    );

    let target_dir = temp_path("parent-reexport-target");
    let output = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", target_dir)
        .output()
        .expect("cargo check should start");

    assert!(
        output.status.success(),
        "generated workspace did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

#[test]
fn moves_cfg_gated_dependencies_to_matching_target_table() {
    let workspace = temp_path("cfg-target-workspace");
    let output = temp_path("cfg-target-output");
    write_cfg_target_dependency_fixture_workspace(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let manifest = read_output(&output, "app/Cargo.toml");
    assert!(
        !manifest.contains("[dependencies.heavy]"),
        "cfg-only dependency should not be retained as always-on:\n{manifest}",
    );
    assert!(
        manifest.contains("[target.'cfg(target_os=\"ios\")'.dependencies.heavy]"),
        "cfg-only dependency should be promoted into the matching target table:\n{manifest}",
    );

    let target_dir = temp_path("cfg-target-build");
    let output = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", target_dir)
        .output()
        .expect("cargo check should start");

    assert!(
        output.status.success(),
        "host cargo check should not build the target-only dependency\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

#[test]
fn prunes_trait_methods_when_trait_is_only_used_as_object_type() {
    let workspace = temp_path("trait-surface-workspace");
    let output = temp_path("trait-surface-output");
    write_trait_surface_fixture_workspace(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let manifest = read_output(&output, "app/Cargo.toml");
    assert!(
        !manifest.contains("async-trait"),
        "method-only macro dependency should be pruned:\n{manifest}",
    );
    assert!(
        !manifest.contains("heavy"),
        "method-only parameter dependency should be pruned:\n{manifest}",
    );

    let api = read_output(&output, "app/src/api.rs");
    assert!(
        api.contains("pub trait Handler: Send + Sync {}"),
        "trait retained only as a type surface should have no method-only dependencies:\n{api}",
    );
    assert_not_present(&api, ["async_trait", "heavy::Request", "handle"]);

    let target_dir = temp_path("trait-surface-build");
    let output = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", target_dir)
        .output()
        .expect("cargo check should start");

    assert!(
        output.status.success(),
        "generated workspace did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

#[test]
fn prunes_unused_private_fields_from_public_struct_surface() {
    let workspace = temp_path("public-struct-surface-workspace");
    let output = temp_path("public-struct-surface-output");
    write_public_struct_surface_fixture_workspace(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let manifest = read_output(&output, "app/Cargo.toml");
    assert!(
        !manifest.contains("heavy"),
        "unused private field dependency should be pruned:\n{manifest}",
    );

    let lib = read_output(&output, "app/src/lib.rs");
    assert!(
        lib.contains("handler: Handler"),
        "reachable private field should remain:\n{lib}",
    );
    assert_not_present(&lib, ["heavy::Envelope", "unused_tx", "unused_envelope"]);

    let target_dir = temp_path("public-struct-surface-build");
    let output = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", target_dir)
        .output()
        .expect("cargo check should start");

    assert!(
        output.status.success(),
        "generated workspace did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
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

pub use nested::{Kind, UnusedKind};

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

pub enum UnusedKind {
    Never,
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

fn write_trait_surface_fixture_workspace(root: &Path) {
    let opensourced_path = repo_root().join("crates/opensourced");
    let opensourced_path = toml_path(&opensourced_path);
    let heavy_root = root.with_file_name(format!(
        "{}-heavy",
        root.file_name().unwrap().to_string_lossy()
    ));
    let heavy_path = toml_path(&heavy_root);

    write_file(
        &root.join("Cargo.toml"),
        r#"[workspace]
members = ["app"]
resolver = "2"

[workspace.dependencies]
opensourced = { path = "OPEN_SOURCED_PATH" }
"#
        .replace("OPEN_SOURCED_PATH", &opensourced_path)
        .as_str(),
    );

    write_file(
        &root.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced.workspace = true
async-trait = "0.1"
heavy = {{ path = "{heavy_path}" }}
"#,
        ),
    );

    write_file(
        &root.join("app/src/lib.rs"),
        r#"pub mod api;

use std::sync::Arc;

#[opensourced::opensourced]
pub fn selected(handler: &Arc<dyn api::Handler>) -> &Arc<dyn api::Handler> {
    handler
}
"#,
    );

    write_file(
        &root.join("app/src/api.rs"),
        r#"use heavy::Request;

#[async_trait::async_trait]
pub trait Handler: Send + Sync {
    async fn handle(&self, request: Request) -> String;
}
"#,
    );

    write_file(
        &heavy_root.join("Cargo.toml"),
        r#"[package]
name = "heavy"
version = "0.1.0"
edition = "2021"
"#,
    );
    write_file(
        &heavy_root.join("src/lib.rs"),
        r#"compile_error!("method-only trait dependency was built");

pub struct Request;
"#,
    );
}

fn write_public_struct_surface_fixture_workspace(root: &Path) {
    let opensourced_path = repo_root().join("crates/opensourced");
    let opensourced_path = toml_path(&opensourced_path);
    let heavy_root = root.with_file_name(format!(
        "{}-heavy",
        root.file_name().unwrap().to_string_lossy()
    ));
    let heavy_path = toml_path(&heavy_root);

    write_file(
        &root.join("Cargo.toml"),
        r#"[workspace]
members = ["app"]
resolver = "2"

[workspace.dependencies]
opensourced = { path = "OPEN_SOURCED_PATH" }
"#
        .replace("OPEN_SOURCED_PATH", &opensourced_path)
        .as_str(),
    );

    write_file(
        &root.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced.workspace = true
heavy = {{ path = "{heavy_path}" }}
"#,
        ),
    );

    write_file(
        &root.join("app/src/lib.rs"),
        r#"use heavy::Envelope;

pub struct Handler;

pub struct Connection {
    unused_tx: Envelope,
    handler: Handler,
}

impl Connection {
    #[opensourced::opensourced]
    pub fn handler(&self) -> &Handler {
        &self.handler
    }

    pub fn unused_envelope(&self) -> &Envelope {
        &self.unused_tx
    }
}
"#,
    );

    write_file(
        &heavy_root.join("Cargo.toml"),
        r#"[package]
name = "heavy"
version = "0.1.0"
edition = "2021"
"#,
    );

    write_file(
        &heavy_root.join("src/lib.rs"),
        r#"compile_error!("unused private field dependency was built");

pub struct Envelope;
"#,
    );
}

fn write_cfg_target_dependency_fixture_workspace(root: &Path) {
    let opensourced_path = repo_root().join("crates/opensourced");
    let opensourced_path = toml_path(&opensourced_path);
    let heavy_root = root.with_file_name(format!(
        "{}-heavy",
        root.file_name().unwrap().to_string_lossy()
    ));
    let heavy_path = toml_path(&heavy_root);

    write_file(
        &root.join("Cargo.toml"),
        r#"[workspace]
members = ["app"]
resolver = "2"

[workspace.dependencies]
opensourced = { path = "OPEN_SOURCED_PATH" }
"#
        .replace("OPEN_SOURCED_PATH", &opensourced_path)
        .as_str(),
    );

    write_file(
        &root.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced.workspace = true
heavy = {{ path = "{heavy_path}" }}
"#,
        ),
    );

    write_file(
        &root.join("app/src/lib.rs"),
        r#"#[cfg(target_os = "ios")]
pub mod gated;

pub fn host_safe() -> i32 {
    1
}
"#,
    );

    write_file(
        &root.join("app/src/gated.rs"),
        r#"#[opensourced::opensourced]
pub fn selected() -> &'static str {
    heavy::value()
}
"#,
    );

    write_file(
        &heavy_root.join("Cargo.toml"),
        r#"[package]
name = "heavy"
version = "0.1.0"
edition = "2021"
"#,
    );

    write_file(
        &heavy_root.join("src/lib.rs"),
        r#"compile_error!("target-only dependency was built on the host");

pub fn value() -> &'static str {
    "heavy"
}
"#,
    );
}

fn write_parent_reexport_fixture_workspace(root: &Path) {
    let opensourced_path = repo_root().join("crates/opensourced");
    let opensourced_path = toml_path(&opensourced_path);

    write_file(
        &root.join("Cargo.toml"),
        r#"[workspace]
members = ["app"]
resolver = "2"

[workspace.dependencies]
opensourced = { path = "OPEN_SOURCED_PATH" }
"#
        .replace("OPEN_SOURCED_PATH", &opensourced_path)
        .as_str(),
    );

    write_file(
        &root.join("app/Cargo.toml"),
        r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced.workspace = true
"#,
    );

    write_file(
        &root.join("app/src/lib.rs"),
        r#"pub mod local_server;
pub mod ssh;
pub mod ssh_scripts;

use opensourced::opensourced;

#[opensourced]
pub fn selected_script() -> String {
    ssh::codex_binary::resolve_codex_binary_script_posix()
}

pub fn unused_root() -> String {
    "unused".to_string()
}
"#,
    );

    write_file(
        &root.join("app/src/local_server.rs"),
        r#"pub fn shell_candidate_lines() -> Vec<&'static str> {
    vec!["/usr/bin/codex", "/opt/codex/bin/codex"]
}

pub fn unused_local_server() -> &'static str {
    "unused"
}
"#,
    );

    write_file(
        &root.join("app/src/ssh/mod.rs"),
        r#"pub mod codex_binary;

pub(crate) use crate::ssh_scripts::posix::{
    PACKAGE_MANAGER_PROBE, PROFILE_INIT, UNUSED_SCRIPT_CONST,
};

pub fn unused_ssh() -> &'static str {
    "unused"
}
"#,
    );

    write_file(
        &root.join("app/src/ssh/codex_binary.rs"),
        r#"use super::{PACKAGE_MANAGER_PROBE, PROFILE_INIT};

pub fn resolve_codex_binary_script_posix() -> String {
    let shared_lines = crate::local_server::shell_candidate_lines().join("\n");
    crate::ssh_scripts::render(
        crate::ssh_scripts::posix::RESOLVE_CODEX_BINARY,
        &[
            ("PROFILE_INIT", PROFILE_INIT),
            ("PACKAGE_MANAGER_PROBE", PACKAGE_MANAGER_PROBE),
            ("SHARED_LINES", &shared_lines),
        ],
    )
}

pub fn unused_codex_binary() -> &'static str {
    "unused"
}
"#,
    );

    write_file(
        &root.join("app/src/ssh_scripts/mod.rs"),
        r#"pub(crate) mod posix;

pub(crate) fn render(template: &str, replacements: &[(&str, &str)]) -> String {
    let mut rendered = template.to_string();
    for (key, value) in replacements {
        rendered = rendered.replace(key, value);
    }
    rendered
}

pub(crate) fn unused_render() -> &'static str {
    "unused"
}
"#,
    );

    write_file(
        &root.join("app/src/ssh_scripts/posix.rs"),
        r#"pub(crate) const PROFILE_INIT: &str = "source ~/.profile";
pub(crate) const PACKAGE_MANAGER_PROBE: &str = "command -v codex";
pub(crate) const RESOLVE_CODEX_BINARY: &str =
    "PROFILE_INIT\nPACKAGE_MANAGER_PROBE\nSHARED_LINES";
pub(crate) const UNUSED_SCRIPT_CONST: &str = "unused";
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
