use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use opensource_core::{generate_with_analyzer, AnalyzerMode, GenerateOptions};

#[test]
fn trims_unused_checked_fixture_workspace_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/trim_unused");
    let output = temp_path("slice-case-trim-unused-output");
    let target_dir = temp_path("slice-case-trim-unused-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("trim_unused fixture should slice");

    assert_eq!(report.packages, ["root", "used"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_total"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let used_root = read(output.join("used/src/lib.rs"));
    let used_live = read(output.join("used/src/live.rs"));
    let workspace = read(output.join("Cargo.toml"));

    assert!(workspace.contains("members = ["), "{workspace}");
    assert!(workspace.contains("\"root\""), "{workspace}");
    assert!(workspace.contains("\"used\""), "{workspace}");
    assert!(!workspace.contains("\"unused\""), "{workspace}");
    assert!(root_manifest.contains("used"), "{root_manifest}");
    assert!(!root_manifest.contains("unused"), "{root_manifest}");
    assert!(
        root_source.contains("pub fn selected_total"),
        "{root_source}"
    );
    assert!(root_source.contains("used::format_live"), "{root_source}");
    assert!(!root_source.contains("dead_entry"), "{root_source}");
    assert!(
        !root_source.contains("unused::format_dead"),
        "{root_source}"
    );

    assert!(used_root.contains("mod live"), "{used_root}");
    assert!(
        used_root.contains("pub use live::{format_live, LiveRecord}"),
        "{used_root}"
    );
    assert!(!used_root.contains("mod dead"), "{used_root}");
    assert!(!used_root.contains("DeadRecord"), "{used_root}");
    assert!(used_live.contains("pub struct LiveRecord"), "{used_live}");
    assert!(used_live.contains("pub fn new"), "{used_live}");
    assert!(used_live.contains("pub fn render"), "{used_live}");
    assert!(used_live.contains("pub fn format_live"), "{used_live}");
    assert!(used_live.contains("fn normalize"), "{used_live}");
    assert!(!used_live.contains("dead_method"), "{used_live}");
    assert!(!used_live.contains("dead_live_helper"), "{used_live}");
    assert!(!output.join("used/src/dead.rs").exists());
    assert!(!output.join("unused").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_support_sub_dependencies_to_used_closure_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/sub_dependency_prune");
    let output = temp_path("slice-case-sub-dependency-prune-output");
    let target_dir = temp_path("slice-case-sub-dependency-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("sub_dependency_prune fixture should slice");

    assert_eq!(report.packages, ["adapter", "leaf", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_summary"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let adapter_manifest = read(output.join("adapter/Cargo.toml"));
    let adapter_root = read(output.join("adapter/src/lib.rs"));
    let adapter_live = read(output.join("adapter/src/live.rs"));
    let leaf_root = read(output.join("leaf/src/lib.rs"));
    let leaf_live = read(output.join("leaf/src/live.rs"));

    assert!(root_manifest.contains("../adapter"), "{root_manifest}");
    assert!(
        root_source.contains("pub fn selected_summary"),
        "{root_source}"
    );
    assert!(
        root_source.contains("adapter::selected_bridge"),
        "{root_source}"
    );
    assert!(!root_source.contains("dead_summary"), "{root_source}");
    assert!(
        !root_source.contains("adapter::dead_bridge"),
        "{root_source}"
    );

    assert!(adapter_manifest.contains("../leaf"), "{adapter_manifest}");
    assert!(adapter_root.contains("mod live"), "{adapter_root}");
    assert!(
        adapter_root.contains("pub use live::{selected_bridge, AdapterRecord}"),
        "{adapter_root}"
    );
    assert!(!adapter_root.contains("mod dead"), "{adapter_root}");
    assert!(!adapter_root.contains("dead_bridge"), "{adapter_root}");
    assert!(!adapter_root.contains("DeadAdapter"), "{adapter_root}");
    assert!(
        adapter_live.contains("macro_rules! render_leaf"),
        "{adapter_live}"
    );
    assert!(
        adapter_live.contains("pub struct AdapterRecord"),
        "{adapter_live}"
    );
    assert!(adapter_live.contains("pub fn new"), "{adapter_live}");
    assert!(adapter_live.contains("pub fn render"), "{adapter_live}");
    assert!(
        adapter_live.contains("pub fn selected_bridge"),
        "{adapter_live}"
    );
    assert!(!adapter_live.contains("dead_method"), "{adapter_live}");
    assert!(!adapter_live.contains("dead_live_bridge"), "{adapter_live}");
    assert!(!output.join("adapter/src/dead.rs").exists());

    assert!(leaf_root.contains("mod live"), "{leaf_root}");
    assert!(
        leaf_root.contains("pub use live::{make_leaf, LeafRecord}")
            || leaf_root.contains("pub use live::LeafRecord"),
        "{leaf_root}"
    );
    assert!(!leaf_root.contains("mod dead"), "{leaf_root}");
    assert!(!leaf_root.contains("DeadLeaf"), "{leaf_root}");
    assert!(leaf_live.contains("pub struct LeafRecord"), "{leaf_live}");
    assert!(leaf_live.contains("pub fn new"), "{leaf_live}");
    assert!(
        leaf_live.contains("pub fn render"),
        "macro receiver call in support dependency should retain used leaf method\n{leaf_live}"
    );
    assert!(leaf_live.contains("fn normalize"), "{leaf_live}");
    assert!(!leaf_live.contains("dead_method"), "{leaf_live}");
    assert!(!leaf_live.contains("dead_live_leaf"), "{leaf_live}");
    assert!(!output.join("leaf/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_alias_import_support_chain_to_used_closure_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/import_alias_prune");
    let output = temp_path("slice-case-import-alias-prune-output");
    let target_dir = temp_path("slice-case-import-alias-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("import_alias_prune fixture should slice");

    assert_eq!(report.packages, ["domain", "gateway", "helper", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let gateway_manifest = read(output.join("gateway/Cargo.toml"));
    let gateway_root = read(output.join("gateway/src/lib.rs"));
    let gateway_live = read(output.join("gateway/src/live.rs"));
    let domain_manifest = read(output.join("domain/Cargo.toml"));
    let domain_root = read(output.join("domain/src/lib.rs"));
    let domain_live = read(output.join("domain/src/live.rs"));
    let helper_root = read(output.join("helper/src/lib.rs"));
    let helper_live = read(output.join("helper/src/live.rs"));

    assert!(root_manifest.contains("../gateway"), "{root_manifest}");
    assert!(
        root_source.contains("pub fn selected_report"),
        "{root_source}"
    );
    assert!(
        root_source.contains("gateway::selected_report"),
        "{root_source}"
    );
    assert!(!root_source.contains("dead_report"), "{root_source}");

    assert!(gateway_manifest.contains("../domain"), "{gateway_manifest}");
    assert!(gateway_root.contains("mod live"), "{gateway_root}");
    assert!(
        gateway_root.contains("pub use live::{selected_report, GatewayRecord}"),
        "{gateway_root}"
    );
    assert!(!gateway_root.contains("mod dead"), "{gateway_root}");
    assert!(!gateway_root.contains("DeadGateway"), "{gateway_root}");
    assert!(
        gateway_live.contains("build_model as make_model"),
        "{gateway_live}"
    );
    assert!(
        gateway_live.contains("LiveModel as PublicModel"),
        "{gateway_live}"
    );
    assert!(gateway_live.contains("model_prefix"), "{gateway_live}");
    assert!(
        gateway_live.contains("macro_rules! render_model"),
        "{gateway_live}"
    );
    assert!(
        gateway_live.contains("pub struct GatewayRecord"),
        "{gateway_live}"
    );
    assert!(gateway_live.contains("pub fn new"), "{gateway_live}");
    assert!(gateway_live.contains("pub fn render"), "{gateway_live}");
    assert!(
        gateway_live.contains("pub fn selected_report"),
        "{gateway_live}"
    );
    assert_no_dead_tokens("gateway/src/live.rs", &gateway_live);
    assert!(!output.join("gateway/src/dead.rs").exists());

    assert!(domain_manifest.contains("../helper"), "{domain_manifest}");
    assert!(domain_root.contains("pub mod live"), "{domain_root}");
    assert!(domain_root.contains("pub mod prelude"), "{domain_root}");
    assert!(
        domain_root.contains("pub use live::{build_model, LiveModel}"),
        "{domain_root}"
    );
    assert!(!domain_root.contains("pub mod dead"), "{domain_root}");
    assert!(!domain_root.contains("DeadModel"), "{domain_root}");
    assert!(domain_live.contains("format_label"), "{domain_live}");
    assert!(domain_live.contains("normalize_value"), "{domain_live}");
    assert!(
        domain_live.contains("pub struct LiveModel"),
        "{domain_live}"
    );
    assert!(domain_live.contains("pub fn new"), "{domain_live}");
    assert!(domain_live.contains("pub fn render"), "{domain_live}");
    assert!(domain_live.contains("pub fn build_model"), "{domain_live}");
    assert!(domain_live.contains("pub fn model_prefix"), "{domain_live}");
    assert_no_dead_tokens("domain/src/live.rs", &domain_live);
    assert!(!output.join("domain/src/dead.rs").exists());

    assert!(helper_root.contains("mod live"), "{helper_root}");
    assert!(helper_root.contains("format_label"), "{helper_root}");
    assert!(helper_root.contains("normalize_value"), "{helper_root}");
    assert!(!helper_root.contains("mod dead"), "{helper_root}");
    assert!(!helper_root.contains("DeadHelper"), "{helper_root}");
    assert!(
        helper_live.contains("pub fn normalize_value"),
        "{helper_live}"
    );
    assert!(helper_live.contains("pub fn format_label"), "{helper_live}");
    assert_no_dead_tokens("helper/src/live.rs", &helper_live);
    assert!(!output.join("helper/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_proc_macro_surface_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/proc_macro_surface_prune");
    let output = temp_path("slice-case-proc-macro-surface-prune-output");
    let target_dir = temp_path("slice-case-proc-macro-surface-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("proc_macro_surface_prune fixture should slice");

    assert_eq!(report.packages, ["api", "macro_support", "model", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_wire"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("api/Cargo.toml"));
    let api_root = read(output.join("api/src/lib.rs"));
    let api_live = read(output.join("api/src/live.rs"));
    let model_root = read(output.join("model/src/lib.rs"));
    let model_live = read(output.join("model/src/live.rs"));
    let macro_support = read(output.join("macro_support/src/lib.rs"));

    assert!(root_manifest.contains("../api"), "{root_manifest}");
    assert!(
        root_source.contains("pub fn selected_wire"),
        "{root_source}"
    );
    assert!(root_source.contains("api::selected_wire"), "{root_source}");
    assert!(!root_source.contains("dead_wire"), "{root_source}");

    assert!(api_manifest.contains("../macro_support"), "{api_manifest}");
    assert!(api_manifest.contains("../model"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(
        api_root.contains("pub use live::{selected_wire, ApiRecord}"),
        "{api_root}"
    );
    assert!(!api_root.contains("mod dead"), "{api_root}");
    assert!(!api_root.contains("DeadApiRecord"), "{api_root}");
    assert!(api_live.contains("SurfaceRecord"), "{api_live}");
    assert!(api_live.contains("surface_attr"), "{api_live}");
    assert!(api_live.contains("model::wire_tag"), "{api_live}");
    assert!(api_live.contains("pub struct ApiRecord"), "{api_live}");
    assert!(api_live.contains("pub fn selected_wire"), "{api_live}");
    assert_no_dead_tokens("api/src/live.rs", &api_live);
    assert!(!output.join("api/src/dead.rs").exists());

    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(
        model_root.contains("pub use live::{wire_tag, LiveWire}"),
        "{model_root}"
    );
    assert!(!model_root.contains("mod dead"), "{model_root}");
    assert!(!model_root.contains("DeadWire"), "{model_root}");
    assert!(model_live.contains("pub struct LiveWire"), "{model_live}");
    assert!(model_live.contains("pub fn new"), "{model_live}");
    assert!(model_live.contains("pub fn render"), "{model_live}");
    assert!(model_live.contains("fn normalize_label"), "{model_live}");
    assert!(model_live.contains("pub fn wire_tag"), "{model_live}");
    assert_no_dead_tokens("model/src/live.rs", &model_live);
    assert!(!output.join("model/src/dead.rs").exists());

    assert!(macro_support.contains("surface_record"), "{macro_support}");
    assert!(macro_support.contains("surface_attr"), "{macro_support}");
    assert!(!macro_support.contains("dead_record"), "{macro_support}");
    assert!(!macro_support.contains("dead_attr"), "{macro_support}");
    assert!(!macro_support.contains("DeadRecord"), "{macro_support}");

    assert_cargo_check(&output, &target_dir, &root_source);
}

fn assert_cargo_check(workspace: &Path, target_dir: &Path, root_source: &str) {
    let output = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(workspace)
        .env("CARGO_TARGET_DIR", target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        output.status.success(),
        "generated slice-case fixture did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nroot/src/lib.rs:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
        root_source,
    );
}

fn assert_no_dead_tokens(label: &str, source: &str) {
    for token in [
        "Dead",
        "dead_report",
        "dead_prefix",
        "dead_live",
        "dead_wire",
        "dead-",
        "unused_helper",
        "dead_imported_helper",
        "make_dead_model",
    ] {
        assert!(
            !source.contains(token),
            "{label} should not contain dead support token {token:?}\n{source}"
        );
    }
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root should exist")
        .to_path_buf()
}

fn temp_path(label: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    path.push(format!("slicer-{label}-{nanos}"));
    if path.exists() {
        fs::remove_dir_all(&path).expect("old temp directory should remove");
    }
    path
}

fn read(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path.as_ref())
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.as_ref().display()))
}
