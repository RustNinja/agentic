use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use opensource_core::{generate, GenerateOptions};

#[test]
fn prunes_dead_names_from_grouped_public_reexports() {
    let workspace = temp_path("rule-public-reexport-workspace");
    let output = temp_path("rule-public-reexport-output");
    let target_dir = temp_path("rule-public-reexport-target");
    write_public_reexport_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("public reexport rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("public_reexport_rule/src/lib.rs"));
    assert!(lib.contains("pub use api::{live, LiveType}"), "{lib}");
    assert!(!lib.contains("dead,"), "{lib}");
    assert!(!lib.contains("DeadType"), "{lib}");
    assert!(!lib.contains("pub fn dead"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn prunes_removed_public_reexport_alias_even_when_alias_name_is_used_locally() {
    let workspace = temp_path("rule-public-alias-workspace");
    let output = temp_path("rule-public-alias-output");
    let target_dir = temp_path("rule-public-alias-target");
    write_removed_public_alias_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("removed public alias rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("public_alias_rule/src/lib.rs"));
    assert!(lib.contains("let selected_value = 7"), "{lib}");
    assert!(!lib.contains("dead_fn as selected_value"), "{lib}");
    assert!(!lib.contains("pub fn dead_fn"), "{lib}");
    assert!(!lib.contains("mod dead"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn reports_owned_dynamic_dispatch_surfaces_inside_retained_structs() {
    let workspace = temp_path("rule-owned-dyn-workspace");
    let output = temp_path("rule-owned-dyn-output");
    write_owned_dynamic_dispatch_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("owned dyn rule should reduce");

    assert_eq!(report.production.status, "hazards_detected");
    let trait_object = report
        .production
        .hazards
        .iter()
        .find(|hazard| hazard.code == "trait_object_surfaces" && hazard.severity == "error")
        .expect("owned dyn field should be a hard production hazard");
    assert!(trait_object.details.iter().any(|detail| {
        detail.subject.contains("dyn Worker")
            && detail
                .file
                .as_ref()
                .is_some_and(|file| file.ends_with("owned_dyn_rule/src/lib.rs"))
    }));

    let function_pointer = report
        .production
        .hazards
        .iter()
        .find(|hazard| hazard.code == "function_pointer_surfaces" && hazard.severity == "error")
        .expect("stored callback field should be a hard production hazard");
    assert!(function_pointer
        .details
        .iter()
        .any(|detail| detail.subject.contains("fn (u32) -> u32")));

    let lib = read(output.join("owned_dyn_rule/src/lib.rs"));
    assert!(lib.contains("Box<dyn Worker>"), "{lib}");
    assert!(lib.contains("pub type Callback = fn(u32) -> u32"), "{lib}");
}

#[test]
fn copies_retained_include_bytes_assets_and_prunes_dead_siblings() {
    let workspace = temp_path("rule-include-bytes-workspace");
    let output = temp_path("rule-include-bytes-output");
    let target_dir = temp_path("rule-include-bytes-target");
    write_include_bytes_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("include bytes rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("include_bytes_rule/src/lib.rs"));
    assert!(lib.contains("include_bytes!(\"certs/live.bin\")"), "{lib}");
    assert!(!lib.contains("DEAD_BYTES"), "{lib}");
    assert!(output
        .join("include_bytes_rule/src/certs/live.bin")
        .exists());
    assert!(!output
        .join("include_bytes_rule/src/certs/dead.bin")
        .exists());
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn flags_build_script_rustc_env_inputs_as_production_blockers() {
    let workspace = temp_path("rule-rustc-env-workspace");
    let output = temp_path("rule-rustc-env-output");
    let target_dir = temp_path("rule-rustc-env-target");
    write_rustc_env_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("rustc-env rule should reduce");

    assert_eq!(report.production.status, "hazards_detected");
    assert!(report
        .production
        .hazards
        .iter()
        .any(|hazard| { hazard.code == "retained_build_scripts" && hazard.severity == "error" }));
    assert!(report
        .production
        .hazards
        .iter()
        .any(|hazard| { hazard.code == "compile_env_macros" && hazard.severity == "error" }));

    let lib = read(output.join("rustc_env_rule/src/lib.rs"));
    assert!(lib.contains("env!(\"GENERATED_TOKEN\")"), "{lib}");
    assert!(output.join("rustc_env_rule/build.rs").exists());
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn reports_nested_callback_future_aliases_as_dynamic_hazards() {
    let workspace = temp_path("rule-callback-future-workspace");
    let output = temp_path("rule-callback-future-output");
    write_callback_future_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("callback future rule should reduce");

    assert_eq!(report.production.status, "hazards_detected");
    let trait_object = report
        .production
        .hazards
        .iter()
        .find(|hazard| hazard.code == "trait_object_surfaces" && hazard.severity == "error")
        .expect("nested callback future should report trait object hazards");
    assert!(trait_object
        .details
        .iter()
        .any(|detail| detail.subject.contains("dyn Fn")));
    assert!(trait_object
        .details
        .iter()
        .any(|detail| detail.subject.contains("dyn Future")));

    let lib = read(output.join("callback_future_rule/src/lib.rs"));
    assert!(lib.contains("Pin<Box<dyn Future"), "{lib}");
    assert!(lib.contains("Arc<"), "{lib}");
    assert!(lib.contains("dyn Fn"), "{lib}");
}

#[test]
fn retains_pub_crate_macro_helper_reexports_used_by_live_modules() {
    let workspace = temp_path("rule-macro-helper-workspace");
    let output = temp_path("rule-macro-helper-output");
    let target_dir = temp_path("rule-macro-helper-target");
    write_macro_helper_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("macro helper rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("macro_helper_rule/src/lib.rs"));
    assert!(lib.contains("macro_rules! blocking_async"), "{lib}");
    assert!(lib.contains("pub(crate) use blocking_async"), "{lib}");
    assert!(lib.contains("blocking_async!(value)"), "{lib}");
    assert!(!lib.contains("unused_helper"), "{lib}");
    assert!(!lib.contains("dead_api"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

fn write_public_reexport_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "public_reexport_rule",
        r#"use opensourced::opensourced;

pub mod api {
    pub struct LiveType;

    pub struct DeadType;

    pub fn live() -> u32 {
        1
    }

    pub fn dead() -> u32 {
        99
    }
}

pub use api::{dead, live, DeadType, LiveType};

#[opensourced]
pub fn selected() -> LiveType {
    let _ = live();
    LiveType
}
"#,
    );
}

fn write_removed_public_alias_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "public_alias_rule",
        r#"use opensourced::opensourced;

mod dead {
    pub fn dead_fn() -> u32 {
        99
    }
}

pub use dead::dead_fn as selected_value;

#[opensourced]
pub fn selected() -> u32 {
    let selected_value = 7;
    selected_value
}
"#,
    );
}

fn write_owned_dynamic_dispatch_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "owned_dyn_rule",
        r#"use opensourced::opensourced;

pub type Callback = fn(u32) -> u32;

pub trait Worker {
    fn work(&self, value: u32) -> u32;
}

pub struct RealWorker;

impl Worker for RealWorker {
    fn work(&self, value: u32) -> u32 {
        value + 1
    }
}

pub struct Registry {
    worker: Box<dyn Worker>,
    callback: Callback,
}

impl Registry {
    pub fn new(callback: Callback) -> Self {
        Self {
            worker: Box::new(RealWorker),
            callback,
        }
    }

    pub fn run(&self, value: u32) -> u32 {
        self.worker.work((self.callback)(value))
    }
}

#[opensourced]
pub fn selected(callback: Callback) -> Registry {
    Registry::new(callback)
}
"#,
    );
}

fn write_include_bytes_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "include_bytes_rule",
        r#"use opensourced::opensourced;

const LIVE_BYTES: &[u8] = include_bytes!("certs/live.bin");
const DEAD_BYTES: &[u8] = include_bytes!("certs/dead.bin");

#[opensourced]
pub fn selected() -> usize {
    LIVE_BYTES.len()
}

pub fn dead_api() -> usize {
    DEAD_BYTES.len()
}
"#,
    );
    write(root.join("include_bytes_rule/src/certs/live.bin"), "live");
    write(root.join("include_bytes_rule/src/certs/dead.bin"), "dead");
}

fn write_rustc_env_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "rustc_env_rule",
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected() -> &'static str {
    env!("GENERATED_TOKEN")
}
"#,
    );
    let manifest = root.join("rustc_env_rule/Cargo.toml");
    let manifest_text = read(&manifest);
    fs::write(
        &manifest,
        manifest_text.replace(
            "edition = \"2021\"\n\n[dependencies]",
            "edition = \"2021\"\nbuild = \"build.rs\"\n\n[dependencies]",
        ),
    )
    .expect("manifest should be writable");
    write(
        root.join("rustc_env_rule/build.rs"),
        r#"fn main() {
    println!("cargo:rustc-env=GENERATED_TOKEN=from-build-script");
}
"#,
    );
}

fn write_callback_future_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "callback_future_rule",
        r#"use opensourced::opensourced;
use std::{future::Future, pin::Pin, sync::Arc};

pub type Reconnector =
    Arc<dyn Fn() -> Pin<Box<dyn Future<Output = Result<u32, String>> + Send>> + Send + Sync>;

pub struct ReconnectHandle {
    reconnect: Reconnector,
}

impl ReconnectHandle {
    pub fn new(reconnect: Reconnector) -> Self {
        Self { reconnect }
    }
}

#[opensourced]
pub fn selected(reconnect: Reconnector) -> ReconnectHandle {
    ReconnectHandle::new(reconnect)
}
"#,
    );
}

fn write_macro_helper_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "macro_helper_rule",
        r#"use opensourced::opensourced;

mod shared {
    macro_rules! blocking_async {
        ($value:expr) => {
            $value + 1
        };
    }

    macro_rules! unused_helper {
        () => {
            99
        };
    }

    pub(crate) use blocking_async;
}

mod client {
    use super::shared::blocking_async;

    pub fn call(value: u32) -> u32 {
        blocking_async!(value)
    }
}

#[opensourced]
pub fn selected(value: u32) -> u32 {
    client::call(value)
}

pub fn dead_api() -> u32 {
    99
}
"#,
    );
}

fn write_workspace(root: &Path, package: &str, lib: &str) {
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[workspace]
members = [{package:?}]
resolver = "2"
"#,
        ),
    );
    write(
        root.join(format!("{package}/Cargo.toml")),
        &format!(
            r#"[package]
name = {package:?}
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&repo_root().join("crates/opensourced"))
        ),
    );
    write(root.join(format!("{package}/src/lib.rs")), lib);
}

fn assert_cargo_check(output: &Path, target_dir: &Path, lib: &str) {
    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(output)
        .env("CARGO_TARGET_DIR", target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated rule slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        lib,
    );
}

fn assert_no_error_hazards(hazards: &[opensource_core::ProductionHazardReport]) {
    assert!(
        hazards.iter().all(|hazard| hazard.severity != "error"),
        "rule case should not produce hard production hazards: {hazards:?}",
    );
}

fn write(path: impl AsRef<Path>, contents: &str) {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent directory should be created");
    }
    fs::write(path, contents).expect("fixture file should be written");
}

fn read(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path).expect("file should be readable")
}

fn manifest_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn temp_path(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("opensourced-{label}-{unique}"))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}
