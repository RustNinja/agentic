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

#[test]
fn prunes_reexport_chains_at_each_grouped_hub() {
    let workspace = temp_path("rule-reexport-chain-workspace");
    let output = temp_path("rule-reexport-chain-output");
    let target_dir = temp_path("rule-reexport-chain-target");
    write_reexport_chain_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reexport chain rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("reexport_chain_rule/src/lib.rs"));
    assert!(lib.contains("pub use facade::{live, LiveType}"), "{lib}");
    assert!(!lib.contains("dead,"), "{lib}");
    assert!(!lib.contains("DeadType"), "{lib}");
    assert!(!lib.contains("pub fn dead"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn copies_include_str_concat_assets_and_prunes_dead_siblings() {
    let workspace = temp_path("rule-include-str-workspace");
    let output = temp_path("rule-include-str-output");
    let target_dir = temp_path("rule-include-str-target");
    write_include_str_concat_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("include_str concat rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("include_str_rule/src/lib.rs"));
    assert!(lib.contains("include_str!(\"guides/live.md\")"), "{lib}");
    assert!(
        lib.contains("include_str!(concat!(\"guides/\", \"concat.md\"))"),
        "{lib}"
    );
    assert!(!lib.contains("DEAD_GUIDE"), "{lib}");
    assert!(output.join("include_str_rule/src/guides/live.md").exists());
    assert!(output
        .join("include_str_rule/src/guides/concat.md")
        .exists());
    assert!(!output.join("include_str_rule/src/guides/dead.md").exists());
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn flags_out_dir_source_includes_as_production_blockers() {
    let workspace = temp_path("rule-out-dir-source-workspace");
    let output = temp_path("rule-out-dir-source-output");
    let target_dir = temp_path("rule-out-dir-source-target");
    write_out_dir_source_include_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("OUT_DIR include rule should reduce");

    assert_eq!(report.production.status, "hazards_detected");
    assert!(report.production.hazards.iter().any(|hazard| {
        hazard.code == "out_dir_source_include_macros" && hazard.severity == "error"
    }));
    assert!(report
        .production
        .hazards
        .iter()
        .any(|hazard| hazard.code == "retained_build_scripts" && hazard.severity == "error"));

    let lib = read(output.join("out_dir_source_rule/src/lib.rs"));
    assert!(lib.contains("include!(concat!(env!(\"OUT_DIR\"), \"/generated.rs\"))"));
    assert!(output.join("out_dir_source_rule/build.rs").exists());
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_macro_generated_items_used_by_live_code() {
    let workspace = temp_path("rule-macro-generated-item-workspace");
    let output = temp_path("rule-macro-generated-item-output");
    let target_dir = temp_path("rule-macro-generated-item-target");
    write_macro_generated_item_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("macro generated item rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("macro_generated_item_rule/src/lib.rs"));
    assert!(lib.contains("macro_rules! declare_value"), "{lib}");
    assert!(lib.contains("declare_value!(live_generated, 41)"), "{lib}");
    assert!(!lib.contains("dead_generated"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_methods_referenced_only_from_live_macro_bodies() {
    let workspace = temp_path("rule-macro-method-body-workspace");
    let output = temp_path("rule-macro-method-body-output");
    let target_dir = temp_path("rule-macro-method-body-target");
    write_macro_method_body_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("macro method body rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("macro_method_body_rule/src/lib.rs"));
    assert!(lib.contains("macro_rules! call_step"), "{lib}");
    assert!(lib.contains("fn step(&self) -> u32"), "{lib}");
    assert!(!lib.contains("fn unused_step"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_associated_type_const_and_projection_impls() {
    let workspace = temp_path("rule-associated-projection-workspace");
    let output = temp_path("rule-associated-projection-output");
    let target_dir = temp_path("rule-associated-projection-target");
    write_associated_projection_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("associated projection rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("associated_projection_rule/src/lib.rs"));
    assert!(lib.contains("type Output = LiveValue"), "{lib}");
    assert!(lib.contains("const KIND: &'static str = \"live\""), "{lib}");
    assert!(!lib.contains("DeadDecoder"), "{lib}");
    assert!(!lib.contains("DeadValue"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_try_from_conversion_impls_for_boundary_types() {
    let workspace = temp_path("rule-try-from-workspace");
    let output = temp_path("rule-try-from-output");
    let target_dir = temp_path("rule-try-from-target");
    write_try_from_boundary_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("try_from boundary rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("try_from_rule/src/lib.rs"));
    assert!(lib.contains("impl TryFrom<RawRequest> for Params"), "{lib}");
    assert!(lib.contains("try_into()"), "{lib}");
    assert!(!lib.contains("DeadRequest"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn reports_option_arc_callback_trait_objects_as_dynamic_hazards() {
    let workspace = temp_path("rule-option-arc-callback-workspace");
    let output = temp_path("rule-option-arc-callback-output");
    write_option_arc_callback_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("Option<Arc<dyn callback>> rule should reduce");

    assert_eq!(report.production.status, "hazards_detected");
    let trait_object = report
        .production
        .hazards
        .iter()
        .find(|hazard| hazard.code == "trait_object_surfaces" && hazard.severity == "error")
        .expect("callback trait object should be a hard production hazard");
    assert!(trait_object.details.iter().any(|detail| {
        detail.subject.contains("dyn Callback")
            && detail.subject.contains("Send")
            && detail.subject.contains("Sync")
    }));

    let lib = read(output.join("option_arc_callback_rule/src/lib.rs"));
    assert!(
        lib.contains("Option<Arc<dyn Callback + Send + Sync>>"),
        "{lib}"
    );
    assert!(!lib.contains("DeadCallback"), "{lib}");
}

#[test]
fn reports_direct_callback_boundaries_as_feedback_warnings() {
    let workspace = temp_path("rule-direct-callback-boundary-workspace");
    let output = temp_path("rule-direct-callback-boundary-output");
    let target_dir = temp_path("rule-direct-callback-boundary-target");
    write_direct_callback_boundary_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("direct callback boundary rule should reduce");

    assert_eq!(report.production.status, "requires_feedback");
    assert_no_error_hazards(&report.production.hazards);
    let boundary = report
        .production
        .hazards
        .iter()
        .find(|hazard| hazard.code == "dynamic_callback_boundaries")
        .expect("direct callback boundaries should be reported");
    assert!(boundary
        .details
        .iter()
        .any(|detail| detail.subject.contains("& dyn Callback")));
    assert!(boundary
        .details
        .iter()
        .any(|detail| detail.subject.contains("fn (u32) -> u32")));

    let lib = read(output.join("direct_callback_boundary_rule/src/lib.rs"));
    assert!(lib.contains("handler: &dyn Callback"), "{lib}");
    assert!(lib.contains("callback: fn(u32) -> u32"), "{lib}");
    assert!(!lib.contains("DeadCallback"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn flags_retained_source_include_macros_as_production_blockers() {
    let workspace = temp_path("rule-source-include-workspace");
    let output = temp_path("rule-source-include-output");
    write_source_include_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("source include rule should reduce");

    assert_eq!(report.production.status, "hazards_detected");
    assert!(report
        .production
        .hazards
        .iter()
        .any(|hazard| { hazard.code == "source_include_macros" && hazard.severity == "error" }));
    let lib = read(output.join("source_include_rule/src/lib.rs"));
    assert!(lib.contains("include!(\"generated_expr.rs\")"), "{lib}");
    assert!(!lib.contains("dead_generated"), "{lib}");
}

#[test]
fn retains_dependency_aliases_used_only_inside_macro_bodies() {
    let workspace = temp_path("rule-macro-alias-body-workspace");
    let output = temp_path("rule-macro-alias-body-output");
    let target_dir = temp_path("rule-macro-alias-body-target");
    write_macro_alias_body_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("macro alias body rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let app = read(output.join("macro_alias_app/src/lib.rs"));
    let support = read(output.join("macro_alias_support/src/lib.rs"));
    assert!(app.contains("use macro_alias_support as upstream"), "{app}");
    assert!(app.contains("macro_rules! build_value"), "{app}");
    assert!(app.contains("upstream::make_value()"), "{app}");
    assert!(support.contains("pub fn make_value"), "{support}");
    assert!(!support.contains("dead_value"), "{support}");
    assert_cargo_check(&output, &target_dir, &app);
}

#[test]
fn prunes_inline_include_str_module_bundles_by_live_branch() {
    let workspace = temp_path("rule-inline-include-tree-workspace");
    let output = temp_path("rule-inline-include-tree-output");
    let target_dir = temp_path("rule-inline-include-tree-target");
    write_inline_include_tree_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("inline include tree rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("inline_include_tree_rule/src/lib.rs"));
    assert!(lib.contains("include_str!(\"scripts/live.sh\")"), "{lib}");
    assert!(!lib.contains("include_str!(\"scripts/dead.ps1\")"), "{lib}");
    assert!(output
        .join("inline_include_tree_rule/src/scripts/live.sh")
        .exists());
    assert!(!output
        .join("inline_include_tree_rule/src/scripts/dead.ps1")
        .exists());
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

fn write_reexport_chain_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "reexport_chain_rule",
        r#"use opensourced::opensourced;

pub mod leaf {
    pub struct LiveType;

    pub struct DeadType;

    pub fn live() -> LiveType {
        LiveType
    }

    pub fn dead() -> DeadType {
        DeadType
    }
}

pub mod facade {
    pub use crate::leaf::{dead, live, DeadType, LiveType};
}

pub use facade::{dead, live, DeadType, LiveType};

#[opensourced]
pub fn selected() -> LiveType {
    live()
}
"#,
    );
}

fn write_include_str_concat_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "include_str_rule",
        r#"use opensourced::opensourced;

const LIVE_GUIDE: &str = include_str!("guides/live.md");
const CONCAT_GUIDE: &str = include_str!(concat!("guides/", "concat.md"));
const DEAD_GUIDE: &str = include_str!("guides/dead.md");

#[opensourced]
pub fn selected() -> usize {
    LIVE_GUIDE.len() + CONCAT_GUIDE.len()
}

pub fn dead_api() -> usize {
    DEAD_GUIDE.len()
}
"#,
    );
    write(root.join("include_str_rule/src/guides/live.md"), "live");
    write(root.join("include_str_rule/src/guides/concat.md"), "concat");
    write(root.join("include_str_rule/src/guides/dead.md"), "dead");
}

fn write_out_dir_source_include_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "out_dir_source_rule",
        r#"use opensourced::opensourced;

mod generated {
    include!(concat!(env!("OUT_DIR"), "/generated.rs"));
}

#[opensourced]
pub fn selected() -> u32 {
    generated::generated_value()
}
"#,
    );
    let manifest = root.join("out_dir_source_rule/Cargo.toml");
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
        root.join("out_dir_source_rule/build.rs"),
        r#"use std::{env, fs, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR should be set"));
    fs::write(out_dir.join("generated.rs"), "pub fn generated_value() -> u32 { 42 }\n")
        .expect("generated source should be writable");
}
"#,
    );
}

fn write_macro_generated_item_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "macro_generated_item_rule",
        r#"use opensourced::opensourced;

macro_rules! declare_value {
    ($name:ident, $value:expr) => {
        pub fn $name() -> u32 {
            $value
        }
    };
}

macro_rules! declare_dead_value {
    ($name:ident) => {
        pub fn $name() -> u32 {
            99
        }
    };
}

declare_value!(live_generated, 41);
declare_dead_value!(dead_generated);

#[opensourced]
pub fn selected() -> u32 {
    live_generated() + 1
}
"#,
    );
}

fn write_macro_method_body_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "macro_method_body_rule",
        r#"use opensourced::opensourced;

macro_rules! call_step {
    ($runner:expr) => {
        $runner.step()
    };
}

pub struct Runner {
    value: u32,
}

impl Runner {
    pub fn new(value: u32) -> Self {
        Self { value }
    }

    fn step(&self) -> u32 {
        self.value + 1
    }

    fn unused_step(&self) -> u32 {
        99
    }
}

#[opensourced]
pub fn selected(runner: &Runner) -> u32 {
    call_step!(runner)
}
"#,
    );
}

fn write_associated_projection_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "associated_projection_rule",
        r#"use opensourced::opensourced;

pub trait Decoder {
    type Output;

    const KIND: &'static str;

    fn decode(value: u32) -> Self::Output;
}

pub struct LiveValue(pub u32);

pub struct DeadValue(pub u32);

pub struct LiveDecoder;

pub struct DeadDecoder;

impl Decoder for LiveDecoder {
    type Output = LiveValue;

    const KIND: &'static str = "live";

    fn decode(value: u32) -> Self::Output {
        LiveValue(value)
    }
}

impl Decoder for DeadDecoder {
    type Output = DeadValue;

    const KIND: &'static str = "dead";

    fn decode(value: u32) -> Self::Output {
        DeadValue(value)
    }
}

#[opensourced]
pub fn selected(value: u32) -> (<LiveDecoder as Decoder>::Output, &'static str) {
    (
        <LiveDecoder as Decoder>::decode(value),
        <LiveDecoder as Decoder>::KIND,
    )
}
"#,
    );
}

fn write_try_from_boundary_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "try_from_rule",
        r#"use opensourced::opensourced;
use std::convert::{TryFrom, TryInto};

pub struct RawRequest {
    pub value: u32,
}

pub struct Params {
    value: u32,
}

impl TryFrom<RawRequest> for Params {
    type Error = &'static str;

    fn try_from(request: RawRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            value: request.value,
        })
    }
}

pub struct DeadRequest;

impl TryFrom<DeadRequest> for Params {
    type Error = &'static str;

    fn try_from(_: DeadRequest) -> Result<Self, Self::Error> {
        Ok(Self { value: 99 })
    }
}

#[opensourced]
pub fn selected(request: RawRequest) -> Result<Params, &'static str> {
    request.try_into()
}
"#,
    );
}

fn write_option_arc_callback_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "option_arc_callback_rule",
        r#"use opensourced::opensourced;
use std::sync::Arc;

pub trait Callback {
    fn notify(&self, value: u32);
}

pub trait DeadCallback {
    fn notify(&self);
}

pub struct Manager {
    callback: Option<Arc<dyn Callback + Send + Sync>>,
}

impl Manager {
    pub fn new(callback: Option<Arc<dyn Callback + Send + Sync>>) -> Self {
        Self { callback }
    }

    pub fn notify(&self, value: u32) {
        if let Some(callback) = &self.callback {
            callback.notify(value);
        }
    }
}

#[opensourced]
pub fn selected(callback: Option<Arc<dyn Callback + Send + Sync>>) -> Manager {
    Manager::new(callback)
}
"#,
    );
}

fn write_direct_callback_boundary_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "direct_callback_boundary_rule",
        r#"use opensourced::opensourced;

pub trait Callback {
    fn handle(&self, value: u32) -> u32;
}

pub trait DeadCallback {
    fn handle(&self) -> u32;
}

#[opensourced]
pub fn selected(handler: &dyn Callback, callback: fn(u32) -> u32) -> u32 {
    handler.handle(callback(7))
}
"#,
    );
}

fn write_source_include_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "source_include_rule",
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected() -> u32 {
    include!("generated_expr.rs")
}

pub fn dead_api() -> u32 {
    include!("dead_generated.rs")
}
"#,
    );
    write(root.join("source_include_rule/src/generated_expr.rs"), "41");
    write(root.join("source_include_rule/src/dead_generated.rs"), "99");
}

fn write_macro_alias_body_rule_fixture(root: &Path) {
    write(
        root.join("Cargo.toml"),
        r#"[workspace]
members = ["macro_alias_app", "macro_alias_support"]
resolver = "2"
"#,
    );
    write(
        root.join("macro_alias_app/Cargo.toml"),
        &format!(
            r#"[package]
name = "macro_alias_app"
version = "0.1.0"
edition = "2021"

[dependencies]
macro_alias_support = {{ path = "../macro_alias_support" }}
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&repo_root().join("crates/opensourced"))
        ),
    );
    write(
        root.join("macro_alias_app/src/lib.rs"),
        r#"use macro_alias_support as upstream;
use opensourced::opensourced;

macro_rules! build_value {
    () => {
        upstream::make_value()
    };
}

#[opensourced]
pub fn selected() -> u32 {
    build_value!()
}
"#,
    );
    write(
        root.join("macro_alias_support/Cargo.toml"),
        r#"[package]
name = "macro_alias_support"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        root.join("macro_alias_support/src/lib.rs"),
        r#"pub fn make_value() -> u32 {
    44
}

pub fn dead_value() -> u32 {
    99
}
"#,
    );
}

fn write_inline_include_tree_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "inline_include_tree_rule",
        r#"use opensourced::opensourced;

mod posix {
    pub const SCRIPT: &str = include_str!("scripts/live.sh");
}

mod powershell {
    pub const SCRIPT: &str = include_str!("scripts/dead.ps1");
}

#[opensourced]
pub fn selected() -> &'static str {
    posix::SCRIPT
}

pub fn dead_api() -> &'static str {
    powershell::SCRIPT
}
"#,
    );
    write(
        root.join("inline_include_tree_rule/src/scripts/live.sh"),
        "echo live",
    );
    write(
        root.join("inline_include_tree_rule/src/scripts/dead.ps1"),
        "Write-Output dead",
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
