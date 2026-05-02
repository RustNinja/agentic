use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use opensource_core::{generate, GenerateOptions};

#[test]
fn slices_single_package_binary_crate_with_stub_main() {
    let workspace = temp_path("single-workspace");
    let output = temp_path("single-output");
    let target_dir = temp_path("single-target");
    write_single_binary_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    assert_eq!(report.packages, ["rtk_like"]);

    let source = read(output.join("rtk_like/src/main.rs"));
    assert!(source.contains("fn selected"));
    assert!(source.contains("fn helper"));
    assert!(source.contains("fn main()"));
    assert!(!source.contains("noisy_entrypoint"));
    assert!(!source.contains("#[opensourced]"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated binary slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/main.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        source,
    );
}

#[test]
fn slices_binary_child_module_with_generated_items_and_pruned_stub_imports() {
    let workspace = temp_path("child-binary-workspace");
    let output = temp_path("child-binary-output");
    let target_dir = temp_path("child-binary-target");
    write_child_binary_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    assert_eq!(report.packages, ["rtk_child_like"]);

    let main_source = read(output.join("rtk_child_like/src/main.rs"));
    assert!(main_source.contains("mod exposed"));
    assert!(main_source.contains("fn main()"));
    assert!(!main_source.contains("use noisy"));
    assert!(!output.join("rtk_child_like/src/noisy.rs").exists());

    let exposed_source = read(output.join("rtk_child_like/src/exposed.rs"));
    assert!(exposed_source.contains("macro_rules! define_generated"));
    assert!(exposed_source.contains("define_generated!(GENERATED_LABEL)"));
    assert!(exposed_source.contains("pub fn selected"));
    assert!(!exposed_source.contains("#[opensourced]"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated child binary slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/main.rs:\n{}\nsrc/exposed.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        main_source,
        exposed_source,
    );
}

#[test]
fn slices_automod_directory_modules_into_explicit_reduced_modules() {
    let workspace = temp_path("automod-workspace");
    let output = temp_path("automod-output");
    let target_dir = temp_path("automod-target");
    write_automod_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let plugins_mod = read(output.join("automod_like/src/plugins/mod.rs"));
    assert!(plugins_mod.contains("pub mod public_slice"));
    assert!(!plugins_mod.contains("dead_slice"));
    assert!(!plugins_mod.contains("automod::dir"));
    assert!(output
        .join("automod_like/src/plugins/public_slice.rs")
        .exists());
    assert!(!output
        .join("automod_like/src/plugins/dead_slice.rs")
        .exists());

    let package_manifest = read(output.join("automod_like/Cargo.toml"));
    assert!(!package_manifest.contains("automod ="));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated automod slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nplugins/mod.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        plugins_mod,
    );
}

#[test]
fn rewrites_features_and_keeps_target_dependencies_and_build_script() {
    let workspace = temp_path("manifest-workspace");
    let output = temp_path("manifest-output");
    let target_dir = temp_path("manifest-target");
    write_manifest_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let app_manifest = read(output.join("app/Cargo.toml"));
    assert!(app_manifest.contains("build = \"build.rs\""));
    assert!(app_manifest.contains("cfg(unix)"));
    assert!(app_manifest.contains("platform_dep"));
    assert!(app_manifest.contains("live_dep"));
    assert!(!app_manifest.contains("dead_dep"));
    assert!(!app_manifest.contains("dep:dead_dep"));
    assert!(!app_manifest.contains("dead_dep?/extra"));
    assert!(output.join("app/build.rs").exists());
    assert!(output.join("app/assets/required.txt").exists());
    assert!(!output.join("app/docs/noise.md").exists());

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated manifest slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\napp/Cargo.toml:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        app_manifest,
    );
}

fn write_single_binary_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "rtk_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("src/main.rs"),
        r#"use opensourced::opensourced;

fn main() {
    noisy_entrypoint();
}

#[opensourced]
fn selected(value: &str) -> String {
    helper(value)
}

fn helper(value: &str) -> String {
    format!("selected:{value}")
}

fn noisy_entrypoint() {
    let _ = dead();
}

fn dead() -> String {
    "dead".to_string()
}
"#,
    );
}

fn write_child_binary_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "rtk_child_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("src/main.rs"),
        r#"mod exposed;
mod noisy;

use noisy::dead_entry;

fn main() {
    dead_entry();
}
"#,
    );
    write(
        root.join("src/exposed.rs"),
        r#"use opensourced::opensourced;

macro_rules! define_generated {
    ($name:ident) => {
        static $name: &str = "generated";
    };
}

define_generated!(GENERATED_LABEL);

#[opensourced]
pub fn selected() -> &'static str {
    GENERATED_LABEL
}
"#,
    );
    write(
        root.join("src/noisy.rs"),
        r#"pub fn dead_entry() {
    let _ = dead_value();
}

fn dead_value() -> &'static str {
    "dead"
}
"#,
    );
}

fn write_automod_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "automod_like"
version = "0.1.0"
edition = "2021"

[dependencies]
automod = "1"
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("src/main.rs"),
        r#"mod plugins;

fn main() {}
"#,
    );
    write(
        root.join("src/plugins/mod.rs"),
        r#"automod::dir!(pub "src/plugins");
"#,
    );
    write(
        root.join("src/plugins/public_slice.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected() -> String {
    helper()
}

fn helper() -> String {
    "live".to_string()
}
"#,
    );
    write(
        root.join("src/plugins/dead_slice.rs"),
        r#"pub fn dead() -> String {
    "dead".to_string()
}
"#,
    );
}

fn write_manifest_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[workspace]
members = ["app", "dead_dep", "live_dep", "platform_dep"]
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
build = "build.rs"

[features]
default = ["dep:dead_dep", "dep:live_dep"]
ffi = ["dep:live_dep", "dead_dep?/extra", "custom-flag"]
custom-flag = []

[dependencies]
dead_dep = { path = "../dead_dep", optional = true }
live_dep = { path = "../live_dep", optional = true }
opensourced.workspace = true

[target.'cfg(unix)'.dependencies]
platform_dep = { path = "../platform_dep" }
"#,
    );
    write(
        root.join("app/build.rs"),
        r#"fn main() {
    let _required = std::fs::read_to_string("assets/required.txt").unwrap();
    println!("cargo:rerun-if-changed=assets/required.txt");
    println!("cargo:rerun-if-changed=build.rs");
}
"#,
    );
    write(root.join("app/assets/required.txt"), "required");
    write(root.join("app/docs/noise.md"), "noise");
    write(
        root.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected() -> String {
    let mut value = live_dep::live();
    #[cfg(unix)]
    {
        value.push_str(&platform_dep::platform());
    }
    value
}

pub fn dead() -> String {
    dead_dep::dead()
}
"#,
    );
    write_package(
        root,
        "live_dep",
        r#"
pub fn live() -> String {
    "live".to_string()
}
"#,
    );
    write_package(
        root,
        "platform_dep",
        r#"
pub fn platform() -> String {
    ":platform".to_string()
}
"#,
    );
    write_package(
        root,
        "dead_dep",
        r#"
pub fn dead() -> String {
    "dead".to_string()
}
"#,
    );
}

fn write_package(root: &Path, name: &str, source: &str) {
    write(
        root.join(name).join("Cargo.toml"),
        &format!(
            r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"
"#
        ),
    );
    write(root.join(name).join("src/lib.rs"), source);
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
    std::env::temp_dir().join(format!("opensourced-manifest-hardening-{label}-{unique}"))
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
