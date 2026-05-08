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

#[test]
fn prunes_asset_conversion_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/asset_conversion_prune");
    let output = temp_path("slice-case-asset-conversion-prune-output");
    let target_dir = temp_path("slice-case-asset-conversion-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("asset_conversion_prune fixture should slice");

    assert_eq!(report.packages, ["asset_api", "asset_codec", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_asset_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("asset_api/Cargo.toml"));
    let api_root = read(output.join("asset_api/src/lib.rs"));
    let api_live = read(output.join("asset_api/src/live.rs"));
    let codec_root = read(output.join("asset_codec/src/lib.rs"));
    let codec_live = read(output.join("asset_codec/src/live.rs"));

    assert!(root_manifest.contains("../asset_api"), "{root_manifest}");
    assert!(
        root_source.contains("pub fn selected_asset_report"),
        "{root_source}"
    );
    assert!(
        root_source.contains("asset_api::selected_asset_report"),
        "{root_source}"
    );
    assert_absent("root/src/lib.rs", &root_source, &["dead_asset_report"]);

    assert!(api_manifest.contains("../asset_codec"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_asset_report"), "{api_root}");
    assert_absent(
        "asset_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_asset_report"],
    );
    assert!(api_live.contains("convert_selected_asset"), "{api_live}");
    assert!(api_live.contains("AssetError"), "{api_live}");
    assert_absent("asset_api/src/live.rs", &api_live, &["dead_asset"]);

    assert!(codec_root.contains("mod live"), "{codec_root}");
    assert!(
        codec_root.contains("convert_selected_asset"),
        "{codec_root}"
    );
    assert!(codec_root.contains("AssetReport"), "{codec_root}");
    assert!(codec_root.contains("AssetError"), "{codec_root}");
    assert_absent(
        "asset_codec/src/lib.rs",
        &codec_root,
        &["mod dead", "DeadAssetReport", "build_dead_asset"],
    );
    assert!(
        codec_live.contains("include_str!(\"assets/header.txt\")"),
        "{codec_live}"
    );
    assert!(
        codec_live.contains("include_str!(concat!(\"assets/\", \"body.txt\"))"),
        "{codec_live}"
    );
    assert!(
        codec_live.contains("impl TryFrom<RawAsset> for AssetReport"),
        "{codec_live}"
    );
    assert!(
        codec_live.contains("pub fn convert_selected_asset"),
        "{codec_live}"
    );
    assert_absent(
        "asset_codec/src/live.rs",
        &codec_live,
        &["DEAD_ASSET", "dead_live_asset", "dead-asset"],
    );
    assert!(output.join("asset_codec/src/assets/header.txt").exists());
    assert!(output.join("asset_codec/src/assets/body.txt").exists());
    assert!(!output.join("asset_codec/src/assets/dead.txt").exists());
    assert!(!output.join("asset_codec/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_callback_boundary_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/callback_boundary_prune");
    let output = temp_path("slice-case-callback-boundary-prune-output");
    let target_dir = temp_path("slice-case-callback-boundary-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("callback_boundary_prune fixture should slice");

    assert_eq!(
        report.packages,
        ["callback_api", "callback_runtime", "root"]
    );
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_callback_score"),
        "selected root should be recorded: {:?}",
        report.roots
    );
    assert!(
        report
            .production
            .hazards
            .iter()
            .any(|hazard| hazard.code == "dynamic_callback_boundaries"),
        "callback boundary fixture should keep explicit callback boundary evidence: {:?}",
        report.production.hazards
    );
    assert!(
        report
            .production
            .hazards
            .iter()
            .all(|hazard| hazard.severity != "error"),
        "direct callback boundaries should not be hard errors: {:?}",
        report.production.hazards
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("callback_api/Cargo.toml"));
    let api_root = read(output.join("callback_api/src/lib.rs"));
    let api_live = read(output.join("callback_api/src/live.rs"));
    let runtime_root = read(output.join("callback_runtime/src/lib.rs"));
    let runtime_live = read(output.join("callback_runtime/src/live.rs"));

    assert!(root_manifest.contains("../callback_api"), "{root_manifest}");
    assert!(
        root_source.contains("pub fn selected_callback_score"),
        "{root_source}"
    );
    assert!(
        root_source.contains("callback_api::selected_callback_score"),
        "{root_source}"
    );
    assert_absent("root/src/lib.rs", &root_source, &["dead_callback_score"]);

    assert!(
        api_manifest.contains("../callback_runtime"),
        "{api_manifest}"
    );
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_callback_score"), "{api_root}");
    assert_absent(
        "callback_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_callback_score"],
    );
    assert!(api_live.contains("struct ApiCallback"), "{api_live}");
    assert!(
        api_live.contains("impl Callback for ApiCallback"),
        "{api_live}"
    );
    assert!(api_live.contains("apply_callback"), "{api_live}");
    assert!(api_live.contains("bump_callback"), "{api_live}");
    assert_absent(
        "callback_api/src/live.rs",
        &api_live,
        &["dead_live_callback_score", "DeadCallback", "dead_callback"],
    );
    assert!(!output.join("callback_api/src/dead.rs").exists());

    assert!(runtime_root.contains("mod live"), "{runtime_root}");
    assert!(runtime_root.contains("CallbackInput"), "{runtime_root}");
    assert!(runtime_root.contains("apply_callback"), "{runtime_root}");
    assert_absent(
        "callback_runtime/src/lib.rs",
        &runtime_root,
        &["mod dead", "DeadCallback", "dead_callback_fn", "DeadInput"],
    );
    assert!(
        runtime_live.contains("pub trait Callback"),
        "{runtime_live}"
    );
    assert!(
        runtime_live.contains("pub type CallbackFn"),
        "{runtime_live}"
    );
    assert!(
        runtime_live.contains("handler: &dyn Callback"),
        "{runtime_live}"
    );
    assert!(
        runtime_live.contains("pub fn bump_callback"),
        "{runtime_live}"
    );
    assert_absent(
        "callback_runtime/src/live.rs",
        &runtime_live,
        &[
            "unused_callback_fn",
            "DeadCallback",
            "dead_callback",
            "DeadInput",
        ],
    );
    assert!(!output.join("callback_runtime/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_trait_ufcs_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/trait_ufcs_prune");
    let output = temp_path("slice-case-trait-ufcs-prune-output");
    let target_dir = temp_path("slice-case-trait-ufcs-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("trait_ufcs_prune fixture should slice");

    assert_eq!(report.packages, ["root", "trait_api", "trait_model"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_trait_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("trait_api/Cargo.toml"));
    let api_root = read(output.join("trait_api/src/lib.rs"));
    let api_live = read(output.join("trait_api/src/live.rs"));
    let model_root = read(output.join("trait_model/src/lib.rs"));
    let model_live = read(output.join("trait_model/src/live.rs"));

    assert!(root_manifest.contains("../trait_api"), "{root_manifest}");
    assert!(
        root_source.contains("pub fn selected_trait_report"),
        "{root_source}"
    );
    assert!(
        root_source.contains("trait_api::selected_trait_report"),
        "{root_source}"
    );
    assert_absent("root/src/lib.rs", &root_source, &["dead_trait_report"]);

    assert!(api_manifest.contains("../trait_model"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_trait_report"), "{api_root}");
    assert_absent(
        "trait_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_trait_report"],
    );
    assert!(api_live.contains("LiveSurface"), "{api_live}");
    assert!(api_live.contains("render_live_surface"), "{api_live}");
    assert_absent(
        "trait_api/src/live.rs",
        &api_live,
        &["dead_live_trait_report", "DeadSurface", "dead_trait"],
    );
    assert!(!output.join("trait_api/src/dead.rs").exists());

    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(model_root.contains("SurfaceRender"), "{model_root}");
    assert!(model_root.contains("LiveSurface"), "{model_root}");
    assert!(model_root.contains("render_live_surface"), "{model_root}");
    assert_absent(
        "trait_model/src/lib.rs",
        &model_root,
        &[
            "mod dead",
            "DeadSurface",
            "DeadSurfaceTrait",
            "render_dead_surface",
        ],
    );
    assert!(
        model_live.contains("pub trait SurfaceRender"),
        "{model_live}"
    );
    assert!(model_live.contains("const PREFIX"), "{model_live}");
    assert!(model_live.contains("fn render(&self)"), "{model_live}");
    assert!(
        model_live.contains("pub struct LiveSurface"),
        "{model_live}"
    );
    assert!(
        model_live.contains("impl SurfaceRender for LiveSurface"),
        "{model_live}"
    );
    assert!(
        model_live.contains("<LiveSurface as SurfaceRender>::render(surface)"),
        "{model_live}"
    );
    assert_absent(
        "trait_model/src/live.rs",
        &model_live,
        &["dead_live_surface", "dead_method", "DeadSurface"],
    );
    assert!(!output.join("trait_model/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_iterator_result_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/iterator_result_prune");
    let output = temp_path("slice-case-iterator-result-prune-output");
    let target_dir = temp_path("slice-case-iterator-result-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("iterator_result_prune fixture should slice");

    assert_eq!(report.packages, ["root", "stream_api", "stream_model"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_stream_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("stream_api/Cargo.toml"));
    let api_root = read(output.join("stream_api/src/lib.rs"));
    let api_live = read(output.join("stream_api/src/live.rs"));
    let model_root = read(output.join("stream_model/src/lib.rs"));
    let model_live = read(output.join("stream_model/src/live.rs"));

    assert!(root_manifest.contains("../stream_api"), "{root_manifest}");
    assert!(
        root_source.contains("pub fn selected_stream_report"),
        "{root_source}"
    );
    assert!(
        root_source.contains("stream_api::selected_stream_report"),
        "{root_source}"
    );
    assert_absent("root/src/lib.rs", &root_source, &["dead_stream_report"]);

    assert!(api_manifest.contains("../stream_model"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_stream_report"), "{api_root}");
    assert_absent(
        "stream_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_stream_report"],
    );
    assert!(api_live.contains("selected_stream"), "{api_live}");
    assert!(api_live.contains("StreamError"), "{api_live}");
    assert_absent("stream_api/src/live.rs", &api_live, &["dead_stream"]);
    assert!(!output.join("stream_api/src/dead.rs").exists());

    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(model_root.contains("StreamResult"), "{model_root}");
    assert!(model_root.contains("EventDto"), "{model_root}");
    assert!(model_root.contains("EventEnvelope"), "{model_root}");
    assert!(model_root.contains("StreamError"), "{model_root}");
    assert_absent(
        "stream_model/src/lib.rs",
        &model_root,
        &["mod dead", "DeadEvent", "DeadEnvelope", "dead_stream"],
    );
    assert!(
        model_live.contains("pub type StreamResult = Result<EventEnvelope, StreamError>"),
        "{model_live}"
    );
    assert!(model_live.contains("pub struct EventDto"), "{model_live}");
    assert!(
        model_live.contains("pub struct EventEnvelope"),
        "{model_live}"
    );
    assert!(
        model_live.contains("pub struct StreamError"),
        "{model_live}"
    );
    assert!(
        model_live.contains("pub fn event_iter(raw: &str) -> impl Iterator<Item = EventDto>"),
        "{model_live}"
    );
    assert!(
        model_live.contains("pub fn selected_stream"),
        "{model_live}"
    );
    assert_absent(
        "stream_model/src/live.rs",
        &model_live,
        &[
            "dead_live_stream",
            "DeadEvent",
            "DeadEnvelope",
            "dead_stream",
        ],
    );
    assert!(!output.join("stream_model/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_facade_glob_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/facade_glob_prune");
    let output = temp_path("slice-case-facade-glob-prune-output");
    let target_dir = temp_path("slice-case-facade-glob-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("facade_glob_prune fixture should slice");

    assert_eq!(report.packages, ["facade_api", "facade_support", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_facade_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("facade_api/Cargo.toml"));
    let api_root = read(output.join("facade_api/src/lib.rs"));
    let api_live = read(output.join("facade_api/src/live.rs"));
    let support_root = read(output.join("facade_support/src/lib.rs"));
    let support_live = read(output.join("facade_support/src/live.rs"));

    assert!(root_manifest.contains("../facade_api"), "{root_manifest}");
    assert!(
        root_source.contains("pub fn selected_facade_report"),
        "{root_source}"
    );
    assert!(
        root_source.contains("facade_api::selected_facade_report"),
        "{root_source}"
    );
    assert_absent("root/src/lib.rs", &root_source, &["dead_facade_report"]);

    assert!(api_manifest.contains("../facade_support"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_facade_report"), "{api_root}");
    assert_absent(
        "facade_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_facade_report"],
    );
    assert!(api_live.contains("build_live"), "{api_live}");
    assert!(api_live.contains("LiveRecord"), "{api_live}");
    assert_absent(
        "facade_api/src/live.rs",
        &api_live,
        &["dead_live_facade_report", "DeadRecord", "dead_factory"],
    );
    assert!(!output.join("facade_api/src/dead.rs").exists());

    assert!(support_root.contains("mod live"), "{support_root}");
    assert!(support_root.contains("pub mod facade"), "{support_root}");
    assert!(support_root.contains("pub mod nested"), "{support_root}");
    assert!(support_root.contains("build_live"), "{support_root}");
    assert!(support_root.contains("LiveRecord"), "{support_root}");
    assert_absent(
        "facade_support/src/lib.rs",
        &support_root,
        &["mod dead", "DeadRecord", "dead_factory"],
    );
    assert!(
        support_live.contains("pub struct LiveRecord"),
        "{support_live}"
    );
    assert!(support_live.contains("pub fn build_live"), "{support_live}");
    assert_absent(
        "facade_support/src/live.rs",
        &support_live,
        &[
            "dead_live_factory",
            "dead_method",
            "dead-live-facade",
            "DeadRecord",
        ],
    );
    assert!(!output.join("facade_support/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_macro_generated_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/macro_generated_prune");
    let output = temp_path("slice-case-macro-generated-prune-output");
    let target_dir = temp_path("slice-case-macro-generated-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("macro_generated_prune fixture should slice");

    assert_eq!(report.packages, ["macro_api", "macro_support", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_macro_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("macro_api/Cargo.toml"));
    let api_root = read(output.join("macro_api/src/lib.rs"));
    let api_live = read(output.join("macro_api/src/live.rs"));
    let support_root = read(output.join("macro_support/src/lib.rs"));

    assert!(root_manifest.contains("../macro_api"), "{root_manifest}");
    assert!(
        root_source.contains("pub fn selected_macro_report"),
        "{root_source}"
    );
    assert!(
        root_source.contains("macro_api::selected_macro_report"),
        "{root_source}"
    );
    assert_absent("root/src/lib.rs", &root_source, &["dead_macro_report"]);

    assert!(api_manifest.contains("../macro_support"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_macro_report"), "{api_root}");
    assert_absent(
        "macro_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_macro_report"],
    );
    assert!(
        api_live.contains("macro_support::selected_generated"),
        "{api_live}"
    );
    assert_absent(
        "macro_api/src/live.rs",
        &api_live,
        &["dead_live_macro_report", "dead_generated"],
    );
    assert!(!output.join("macro_api/src/dead.rs").exists());

    assert!(
        support_root.contains("macro_rules! define_generated"),
        "{support_root}"
    );
    assert!(support_root.contains("LiveGenerated"), "{support_root}");
    assert!(
        support_root.contains("build_live_generated"),
        "{support_root}"
    );
    assert!(
        support_root.contains("pub fn selected_generated"),
        "{support_root}"
    );
    assert_absent(
        "macro_support/src/lib.rs",
        &support_root,
        &["DeadGenerated", "build_dead_generated", "dead_generated"],
    );

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_static_registry_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/static_registry_prune");
    let output = temp_path("slice-case-static-registry-prune-output");
    let target_dir = temp_path("slice-case-static-registry-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("static_registry_prune fixture should slice");

    assert_eq!(
        report.packages,
        ["registry_api", "registry_support", "root"]
    );
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_registry_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("registry_api/Cargo.toml"));
    let api_root = read(output.join("registry_api/src/lib.rs"));
    let api_live = read(output.join("registry_api/src/live.rs"));
    let support_root = read(output.join("registry_support/src/lib.rs"));
    let support_live = read(output.join("registry_support/src/live.rs"));

    assert!(root_manifest.contains("../registry_api"), "{root_manifest}");
    assert!(
        root_source.contains("pub fn selected_registry_report"),
        "{root_source}"
    );
    assert!(
        root_source.contains("registry_api::selected_registry_report"),
        "{root_source}"
    );
    assert_absent("root/src/lib.rs", &root_source, &["dead_registry_report"]);

    assert!(
        api_manifest.contains("../registry_support"),
        "{api_manifest}"
    );
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_registry_report"), "{api_root}");
    assert_absent(
        "registry_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_registry_report"],
    );
    assert!(
        api_live.contains("registry_support::selected_registry"),
        "{api_live}"
    );
    assert_absent(
        "registry_api/src/live.rs",
        &api_live,
        &["dead_live_registry_report"],
    );
    assert!(!output.join("registry_api/src/dead.rs").exists());

    assert!(support_root.contains("mod live"), "{support_root}");
    assert!(support_root.contains("selected_registry"), "{support_root}");
    assert!(support_root.contains("RegistryHandle"), "{support_root}");
    assert_absent(
        "registry_support/src/lib.rs",
        &support_root,
        &["mod dead", "DeadRegistry", "dead_registry"],
    );
    assert!(support_live.contains("OnceLock"), "{support_live}");
    assert!(support_live.contains("Mutex"), "{support_live}");
    assert!(support_live.contains("Arc"), "{support_live}");
    assert!(support_live.contains("LIVE_REGISTRY"), "{support_live}");
    assert!(
        support_live.contains("pub struct RegistryHandle"),
        "{support_live}"
    );
    assert!(support_live.contains("fn live_slot"), "{support_live}");
    assert!(
        support_live.contains("pub fn selected_registry"),
        "{support_live}"
    );
    assert!(support_live.contains("pub fn render"), "{support_live}");
    assert_absent(
        "registry_support/src/live.rs",
        &support_live,
        &["dead_method", "dead_live_registry", "dead-registry"],
    );
    assert!(!output.join("registry_support/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_patch_state_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/patch_state_prune");
    let output = temp_path("slice-case-patch-state-prune-output");
    let target_dir = temp_path("slice-case-patch-state-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("patch_state_prune fixture should slice");

    assert_eq!(report.packages, ["patch_api", "patch_model", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_patch_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("patch_api/Cargo.toml"));
    let api_root = read(output.join("patch_api/src/lib.rs"));
    let api_live = read(output.join("patch_api/src/live.rs"));
    let model_root = read(output.join("patch_model/src/lib.rs"));
    let model_live = read(output.join("patch_model/src/live.rs"));

    assert!(root_manifest.contains("../patch_api"), "{root_manifest}");
    assert!(
        root_source.contains("pub fn selected_patch_report"),
        "{root_source}"
    );
    assert!(
        root_source.contains("patch_api::selected_patch_report"),
        "{root_source}"
    );
    assert_absent("root/src/lib.rs", &root_source, &["dead_patch_report"]);

    assert!(api_manifest.contains("../patch_model"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_patch_report"), "{api_root}");
    assert_absent(
        "patch_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_patch_report"],
    );
    assert!(
        api_live.contains("patch_model::selected_patch"),
        "{api_live}"
    );
    assert_absent(
        "patch_api/src/live.rs",
        &api_live,
        &["dead_live_patch_report"],
    );
    assert!(!output.join("patch_api/src/dead.rs").exists());

    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(model_root.contains("selected_patch"), "{model_root}");
    assert!(model_root.contains("PatchRequest"), "{model_root}");
    assert_absent(
        "patch_model/src/lib.rs",
        &model_root,
        &["mod dead", "DeadPatch", "dead_patch"],
    );
    assert!(model_live.contains("pub enum PatchOp"), "{model_live}");
    assert!(model_live.contains("pub enum PatchError"), "{model_live}");
    assert!(
        model_live.contains("pub struct PatchSegment"),
        "{model_live}"
    );
    assert!(model_live.contains("impl TryFrom"), "{model_live}");
    assert!(
        model_live.contains("pub struct PatchRequest"),
        "{model_live}"
    );
    assert!(model_live.contains("pub fn build_patch"), "{model_live}");
    assert!(model_live.contains("pub fn selected_patch"), "{model_live}");
    assert_absent(
        "patch_model/src/live.rs",
        &model_live,
        &["dead_method", "dead_live_patch", "dead-segment"],
    );
    assert!(!output.join("patch_model/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_uniffi_runtime_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/uniffi_runtime_prune");
    let output = temp_path("slice-case-uniffi-runtime-prune-output");
    let target_dir = temp_path("slice-case-uniffi-runtime-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("uniffi_runtime_prune fixture should slice");

    assert_eq!(report.packages, ["root", "runtime_api", "runtime_support"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_runtime_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("runtime_api/Cargo.toml"));
    let api_root = read(output.join("runtime_api/src/lib.rs"));
    let api_live = read(output.join("runtime_api/src/live.rs"));
    let support_root = read(output.join("runtime_support/src/lib.rs"));
    let support_live = read(output.join("runtime_support/src/live.rs"));

    assert!(root_manifest.contains("../runtime_api"), "{root_manifest}");
    assert!(
        root_source.contains("pub fn selected_runtime_report"),
        "{root_source}"
    );
    assert!(
        root_source.contains("runtime_api::selected_runtime_report"),
        "{root_source}"
    );
    assert_absent("root/src/lib.rs", &root_source, &["dead_runtime_report"]);

    assert!(
        api_manifest.contains("../runtime_support"),
        "{api_manifest}"
    );
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_runtime_report"), "{api_root}");
    assert_absent(
        "runtime_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_runtime_report"],
    );
    assert!(
        api_live.contains("runtime_support::selected_runtime"),
        "{api_live}"
    );
    assert_absent(
        "runtime_api/src/live.rs",
        &api_live,
        &["dead_live_runtime_report"],
    );
    assert!(!output.join("runtime_api/src/dead.rs").exists());

    assert!(support_root.contains("mod live"), "{support_root}");
    assert!(support_root.contains("selected_runtime"), "{support_root}");
    assert!(support_root.contains("RuntimeObject"), "{support_root}");
    assert_absent(
        "runtime_support/src/lib.rs",
        &support_root,
        &["mod dead", "DeadRuntimeObject", "dead_runtime"],
    );
    assert!(support_live.contains("OnceLock"), "{support_live}");
    assert!(support_live.contains("Arc"), "{support_live}");
    assert!(support_live.contains("static RUNTIME"), "{support_live}");
    assert!(
        support_live.contains("pub struct RuntimeObject"),
        "{support_live}"
    );
    assert!(
        support_live.contains("pub fn shared_runtime"),
        "{support_live}"
    );
    assert!(
        support_live.contains("pub fn selected_runtime"),
        "{support_live}"
    );
    assert!(support_live.contains("pub fn status"), "{support_live}");
    assert_absent(
        "runtime_support/src/live.rs",
        &support_live,
        &["dead_method", "dead_live_runtime", "dead-runtime"],
    );
    assert!(!output.join("runtime_support/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_async_command_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/async_command_prune");
    let output = temp_path("slice-case-async-command-prune-output");
    let target_dir = temp_path("slice-case-async-command-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("async_command_prune fixture should slice");

    assert_eq!(report.packages, ["command_api", "command_runtime", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_command_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("command_api/Cargo.toml"));
    let api_root = read(output.join("command_api/src/lib.rs"));
    let api_live = read(output.join("command_api/src/live.rs"));
    let runtime_root = read(output.join("command_runtime/src/lib.rs"));
    let runtime_live = read(output.join("command_runtime/src/live.rs"));

    assert!(root_manifest.contains("../command_api"), "{root_manifest}");
    assert!(root_source.contains("async fn selected_command_report"));
    assert!(root_source.contains("command_api::selected_command_report"));
    assert_absent("root/src/lib.rs", &root_source, &["dead_command_report"]);

    assert!(
        api_manifest.contains("../command_runtime"),
        "{api_manifest}"
    );
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_command_report"), "{api_root}");
    assert_absent(
        "command_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_command_report"],
    );
    assert!(
        api_live.contains("command_runtime::selected_command"),
        "{api_live}"
    );
    assert_absent(
        "command_api/src/live.rs",
        &api_live,
        &["dead_live_command_report"],
    );
    assert!(!output.join("command_api/src/dead.rs").exists());

    assert!(runtime_root.contains("mod live"), "{runtime_root}");
    assert!(runtime_root.contains("selected_command"), "{runtime_root}");
    assert!(runtime_root.contains("WorkerCommand"), "{runtime_root}");
    assert_absent(
        "command_runtime/src/lib.rs",
        &runtime_root,
        &["mod dead", "DeadCommand", "dead_command"],
    );
    assert!(
        runtime_live.contains("pub struct CommandJob"),
        "{runtime_live}"
    );
    assert!(
        runtime_live.contains("pub enum WorkerCommand"),
        "{runtime_live}"
    );
    assert!(
        runtime_live.contains("pub struct WorkerState"),
        "{runtime_live}"
    );
    assert!(
        runtime_live.contains("async fn handle_command"),
        "{runtime_live}"
    );
    assert!(
        runtime_live.contains("pub async fn selected_command"),
        "{runtime_live}"
    );
    assert_absent(
        "command_runtime/src/live.rs",
        &runtime_live,
        &["dead_method", "dead_live_command", "dead-command"],
    );
    assert!(!output.join("command_runtime/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_ffi_export_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/ffi_export_prune");
    let output = temp_path("slice-case-ffi-export-prune-output");
    let target_dir = temp_path("slice-case-ffi-export-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("ffi_export_prune fixture should slice");

    assert_eq!(report.packages, ["ffi_api", "ffi_support", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_ffi_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("ffi_api/Cargo.toml"));
    let api_root = read(output.join("ffi_api/src/lib.rs"));
    let api_live = read(output.join("ffi_api/src/live.rs"));
    let support_root = read(output.join("ffi_support/src/lib.rs"));
    let support_live = read(output.join("ffi_support/src/live.rs"));

    assert!(root_manifest.contains("../ffi_api"), "{root_manifest}");
    assert!(root_source.contains("pub fn selected_ffi_report"));
    assert!(root_source.contains("ffi_api::selected_ffi_report"));
    assert_absent("root/src/lib.rs", &root_source, &["dead_ffi_report"]);

    assert!(api_manifest.contains("../ffi_support"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_ffi_report"), "{api_root}");
    assert_absent(
        "ffi_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_ffi_report"],
    );
    assert!(api_live.contains("#[no_mangle]"), "{api_live}");
    assert!(api_live.contains("extern \"C\" fn selected_ffi_export"));
    assert!(api_live.contains("ffi_support::selected_ffi_len"));
    assert_absent("ffi_api/src/live.rs", &api_live, &["dead_live_ffi_report"]);
    assert!(!output.join("ffi_api/src/dead.rs").exists());

    assert!(support_root.contains("mod live"), "{support_root}");
    assert!(support_root.contains("selected_ffi_len"), "{support_root}");
    assert!(support_root.contains("FfiState"), "{support_root}");
    assert_absent(
        "ffi_support/src/lib.rs",
        &support_root,
        &["mod dead", "DeadFfiState", "dead_ffi_len"],
    );
    assert!(
        support_live.contains("pub struct FfiState"),
        "{support_live}"
    );
    assert!(support_live.contains("pub fn from_raw"), "{support_live}");
    assert!(support_live.contains("pub fn score"), "{support_live}");
    assert!(support_live.contains("pub fn selected_ffi_len"));
    assert_absent(
        "ffi_support/src/live.rs",
        &support_live,
        &["dead_method", "dead_live_ffi_len"],
    );
    assert!(!output.join("ffi_support/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_conversion_roundtrip_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/conversion_roundtrip_prune");
    let output = temp_path("slice-case-conversion-roundtrip-prune-output");
    let target_dir = temp_path("slice-case-conversion-roundtrip-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("conversion_roundtrip_prune fixture should slice");

    assert_eq!(report.packages, ["root", "wire_api", "wire_model"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_wire_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("wire_api/Cargo.toml"));
    let api_root = read(output.join("wire_api/src/lib.rs"));
    let api_live = read(output.join("wire_api/src/live.rs"));
    let model_root = read(output.join("wire_model/src/lib.rs"));
    let model_live = read(output.join("wire_model/src/live.rs"));

    assert!(root_manifest.contains("../wire_api"), "{root_manifest}");
    assert!(root_source.contains("pub fn selected_wire_report"));
    assert!(root_source.contains("wire_api::selected_wire_report"));
    assert_absent("root/src/lib.rs", &root_source, &["dead_wire_report"]);

    assert!(api_manifest.contains("../wire_model"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_wire_report"), "{api_root}");
    assert_absent(
        "wire_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_wire_report"],
    );
    assert!(api_live.contains("wire_model::selected_wire"), "{api_live}");
    assert_absent(
        "wire_api/src/live.rs",
        &api_live,
        &["dead_live_wire_report"],
    );
    assert!(!output.join("wire_api/src/dead.rs").exists());

    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(model_root.contains("selected_wire"), "{model_root}");
    assert!(model_root.contains("WireValue"), "{model_root}");
    assert!(model_root.contains("DomainValue"), "{model_root}");
    assert_absent(
        "wire_model/src/lib.rs",
        &model_root,
        &["mod dead", "DeadWire", "dead_wire"],
    );
    assert!(model_live.contains("pub struct WireValue"), "{model_live}");
    assert!(
        model_live.contains("pub struct DomainValue"),
        "{model_live}"
    );
    assert!(model_live.contains("impl From<&str> for WireValue"));
    assert!(model_live.contains("impl From<WireValue> for DomainValue"));
    assert!(model_live.contains("impl From<DomainValue> for WireValue"));
    assert!(model_live.contains("pub fn render"), "{model_live}");
    assert!(model_live.contains("pub fn selected_wire"), "{model_live}");
    assert_absent(
        "wire_model/src/live.rs",
        &model_live,
        &["dead_method", "dead_live_wire", "dead-wire"],
    );
    assert!(!output.join("wire_model/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_macro_receiver_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/macro_receiver_prune");
    let output = temp_path("slice-case-macro-receiver-prune-output");
    let target_dir = temp_path("slice-case-macro-receiver-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("macro_receiver_prune fixture should slice");

    assert_eq!(report.packages, ["macro_api", "macro_support", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_macro_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("macro_api/Cargo.toml"));
    let api_root = read(output.join("macro_api/src/lib.rs"));
    let api_live = read(output.join("macro_api/src/live.rs"));
    let support_root = read(output.join("macro_support/src/lib.rs"));
    let support_live = read(output.join("macro_support/src/live.rs"));

    assert!(root_manifest.contains("../macro_api"), "{root_manifest}");
    assert!(root_source.contains("pub fn selected_macro_report"));
    assert!(root_source.contains("macro_api::selected_macro_report"));
    assert_absent("root/src/lib.rs", &root_source, &["dead_macro_report"]);

    assert!(api_manifest.contains("../macro_support"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_macro_report"), "{api_root}");
    assert_absent(
        "macro_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_macro_report"],
    );
    assert!(api_live.contains("macro_support::selected_macro"));
    assert_absent(
        "macro_api/src/live.rs",
        &api_live,
        &["dead_live_macro_report"],
    );
    assert!(!output.join("macro_api/src/dead.rs").exists());

    assert!(support_root.contains("mod live"), "{support_root}");
    assert!(support_root.contains("selected_macro"), "{support_root}");
    assert!(support_root.contains("MacroRecord"), "{support_root}");
    assert!(support_root.contains("NormalizedRecord"), "{support_root}");
    assert_absent(
        "macro_support/src/lib.rs",
        &support_root,
        &["mod dead", "DeadMacroRecord", "dead_macro"],
    );
    assert!(support_live.contains("macro_rules! render_record"));
    assert!(support_live.contains("pub(crate) use render_record"));
    assert!(support_live.contains("pub struct MacroRecord"));
    assert!(support_live.contains("pub fn normalize"));
    assert!(support_live.contains("pub struct NormalizedRecord"));
    assert!(support_live.contains("pub fn render"));
    assert!(support_live.contains("pub fn selected_macro"));
    assert_absent(
        "macro_support/src/live.rs",
        &support_live,
        &["dead_method", "dead_live_macro", "dead-macro"],
    );
    assert!(!output.join("macro_support/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_error_source_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/error_source_prune");
    let output = temp_path("slice-case-error-source-prune-output");
    let target_dir = temp_path("slice-case-error-source-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("error_source_prune fixture should slice");

    assert_eq!(report.packages, ["error_api", "error_model", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_error_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("error_api/Cargo.toml"));
    let api_root = read(output.join("error_api/src/lib.rs"));
    let api_live = read(output.join("error_api/src/live.rs"));
    let model_root = read(output.join("error_model/src/lib.rs"));
    let model_live = read(output.join("error_model/src/live.rs"));

    assert!(root_manifest.contains("../error_api"), "{root_manifest}");
    assert!(root_source.contains("pub fn selected_error_report"));
    assert!(root_source.contains("error_api::selected_error_report"));
    assert_absent("root/src/lib.rs", &root_source, &["dead_error_report"]);

    assert!(api_manifest.contains("../error_model"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_error_report"), "{api_root}");
    assert_absent(
        "error_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_error_report"],
    );
    assert!(api_live.contains("error_model::selected_error"));
    assert_absent(
        "error_api/src/live.rs",
        &api_live,
        &["dead_live_error_report"],
    );
    assert!(!output.join("error_api/src/dead.rs").exists());

    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(model_root.contains("selected_error"), "{model_root}");
    assert!(model_root.contains("WireError"), "{model_root}");
    assert!(model_root.contains("ParseFailure"), "{model_root}");
    assert_absent(
        "error_model/src/lib.rs",
        &model_root,
        &["mod dead", "DeadWireError", "dead_error"],
    );
    assert!(model_live.contains("pub enum WireError"), "{model_live}");
    assert!(
        model_live.contains("pub struct ParseFailure"),
        "{model_live}"
    );
    assert!(model_live.contains("impl From<ParseFailure> for WireError"));
    assert!(model_live.contains("fn normalize"), "{model_live}");
    assert!(model_live.contains("pub fn parse_wire"), "{model_live}");
    assert!(model_live.contains("pub fn selected_error"), "{model_live}");
    assert_absent(
        "error_model/src/live.rs",
        &model_live,
        &["dead_method", "dead_live_error", "dead-error", "dead-parse"],
    );
    assert!(!output.join("error_model/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_poll_adapter_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/poll_adapter_prune");
    let output = temp_path("slice-case-poll-adapter-prune-output");
    let target_dir = temp_path("slice-case-poll-adapter-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("poll_adapter_prune fixture should slice");

    assert_eq!(report.packages, ["poll_api", "poll_support", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_poll_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("poll_api/Cargo.toml"));
    let api_root = read(output.join("poll_api/src/lib.rs"));
    let api_live = read(output.join("poll_api/src/live.rs"));
    let support_root = read(output.join("poll_support/src/lib.rs"));
    let support_live = read(output.join("poll_support/src/live.rs"));

    assert!(root_manifest.contains("../poll_api"), "{root_manifest}");
    assert!(root_source.contains("pub fn selected_poll_report"));
    assert!(root_source.contains("poll_api::selected_poll_report"));
    assert_absent("root/src/lib.rs", &root_source, &["dead_poll_report"]);

    assert!(api_manifest.contains("../poll_support"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_poll_report"), "{api_root}");
    assert_absent(
        "poll_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_poll_report"],
    );
    assert!(api_live.contains("poll_support::selected_poll"));
    assert_absent(
        "poll_api/src/live.rs",
        &api_live,
        &["dead_live_poll_report"],
    );
    assert!(!output.join("poll_api/src/dead.rs").exists());

    assert!(support_root.contains("mod live"), "{support_root}");
    assert!(support_root.contains("selected_poll"), "{support_root}");
    assert!(support_root.contains("PollFrame"), "{support_root}");
    assert!(support_root.contains("Poller"), "{support_root}");
    assert_absent(
        "poll_support/src/lib.rs",
        &support_root,
        &["mod dead", "DeadPoller", "dead_poll"],
    );
    assert!(
        support_live.contains("use std::task::Poll"),
        "{support_live}"
    );
    assert!(
        support_live.contains("pub struct PollFrame"),
        "{support_live}"
    );
    assert!(support_live.contains("pub struct Poller"), "{support_live}");
    assert!(support_live.contains("pub fn poll_next"), "{support_live}");
    assert!(
        support_live.contains("pub fn selected_poll"),
        "{support_live}"
    );
    assert_absent(
        "poll_support/src/live.rs",
        &support_live,
        &["dead_method", "dead_live_poll", "dead-frame"],
    );
    assert!(!output.join("poll_support/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_type_alias_surface_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/type_alias_surface_prune");
    let output = temp_path("slice-case-type-alias-surface-prune-output");
    let target_dir = temp_path("slice-case-type-alias-surface-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("type_alias_surface_prune fixture should slice");

    assert_eq!(report.packages, ["envelope_api", "envelope_model", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_envelope_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("envelope_api/Cargo.toml"));
    let api_root = read(output.join("envelope_api/src/lib.rs"));
    let api_live = read(output.join("envelope_api/src/live.rs"));
    let model_root = read(output.join("envelope_model/src/lib.rs"));
    let model_live = read(output.join("envelope_model/src/live.rs"));

    assert!(root_manifest.contains("../envelope_api"), "{root_manifest}");
    assert!(root_source.contains("pub fn selected_envelope_report"));
    assert!(root_source.contains("envelope_api::selected_envelope_report"));
    assert_absent("root/src/lib.rs", &root_source, &["dead_envelope_report"]);

    assert!(api_manifest.contains("../envelope_model"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_envelope_report"), "{api_root}");
    assert_absent(
        "envelope_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_envelope_report"],
    );
    assert!(api_live.contains("envelope_model::render_envelope"));
    assert_absent(
        "envelope_api/src/live.rs",
        &api_live,
        &["dead_live_envelope_report"],
    );
    assert!(!output.join("envelope_api/src/dead.rs").exists());

    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(model_root.contains("render_envelope"), "{model_root}");
    assert!(model_root.contains("EnvelopeDto"), "{model_root}");
    assert!(model_root.contains("EnvelopeError"), "{model_root}");
    assert_absent(
        "envelope_model/src/lib.rs",
        &model_root,
        &["mod dead", "DeadEnvelope", "dead_envelope"],
    );
    assert!(
        model_live.contains("pub type EnvelopeResult"),
        "{model_live}"
    );
    assert!(
        model_live.contains("pub struct EnvelopeDto"),
        "{model_live}"
    );
    assert!(
        model_live.contains("pub enum EnvelopeError"),
        "{model_live}"
    );
    assert!(
        model_live.contains("pub fn selected_envelope"),
        "{model_live}"
    );
    assert!(
        model_live.contains("pub fn render_envelope"),
        "{model_live}"
    );
    assert_absent(
        "envelope_model/src/live.rs",
        &model_live,
        &["dead_method", "dead_live_envelope", "dead-dto"],
    );
    assert!(!output.join("envelope_model/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_callback_store_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/callback_store_prune");
    let output = temp_path("slice-case-callback-store-prune-output");
    let target_dir = temp_path("slice-case-callback-store-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("callback_store_prune fixture should slice");

    assert_eq!(
        report.packages,
        ["callback_store_api", "callback_store_support", "root"]
    );
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_callback_store_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );
    assert!(
        report
            .production
            .hazards
            .iter()
            .any(|hazard| hazard.code == "trait_object_surfaces"),
        "stored callback fixture should keep explicit dynamic-dispatch hazard evidence: {:?}",
        report.production.hazards
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("callback_store_api/Cargo.toml"));
    let api_root = read(output.join("callback_store_api/src/lib.rs"));
    let api_live = read(output.join("callback_store_api/src/live.rs"));
    let support_root = read(output.join("callback_store_support/src/lib.rs"));
    let support_live = read(output.join("callback_store_support/src/live.rs"));

    assert!(
        root_manifest.contains("../callback_store_api"),
        "{root_manifest}"
    );
    assert!(root_source.contains("pub fn selected_callback_store_report"));
    assert!(root_source.contains("callback_store_api::selected_callback_store_report"));
    assert_absent(
        "root/src/lib.rs",
        &root_source,
        &["dead_callback_store_report"],
    );

    assert!(
        api_manifest.contains("../callback_store_support"),
        "{api_manifest}"
    );
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(
        api_root.contains("selected_callback_store_report"),
        "{api_root}"
    );
    assert_absent(
        "callback_store_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_callback_store_report"],
    );
    assert!(api_live.contains("callback_store_support::selected_callback_store"));
    assert_absent(
        "callback_store_api/src/live.rs",
        &api_live,
        &["dead_live_callback_store_report"],
    );
    assert!(!output.join("callback_store_api/src/dead.rs").exists());

    assert!(support_root.contains("mod live"), "{support_root}");
    assert!(
        support_root.contains("selected_callback_store"),
        "{support_root}"
    );
    assert!(support_root.contains("CallbackStore"), "{support_root}");
    assert!(support_root.contains("EventRecord"), "{support_root}");
    assert!(support_root.contains("EventSink"), "{support_root}");
    assert_absent(
        "callback_store_support/src/lib.rs",
        &support_root,
        &["mod dead", "DeadCallbackStore", "dead_callback_store"],
    );
    assert!(support_live.contains("Arc"), "{support_live}");
    assert!(support_live.contains("RwLock"), "{support_live}");
    assert!(support_live.contains("dyn EventSink"), "{support_live}");
    assert!(
        support_live.contains("pub trait EventSink"),
        "{support_live}"
    );
    assert!(
        support_live.contains("pub struct EventRecord"),
        "{support_live}"
    );
    assert!(
        support_live.contains("pub struct CallbackStore"),
        "{support_live}"
    );
    assert!(
        support_live.contains("pub fn selected_callback_store"),
        "{support_live}"
    );
    assert_absent(
        "callback_store_support/src/live.rs",
        &support_live,
        &["dead_live_callback_store", "dead-store", "dead-event"],
    );
    assert!(!output.join("callback_store_support/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_const_chain_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/const_chain_prune");
    let output = temp_path("slice-case-const-chain-prune-output");
    let target_dir = temp_path("slice-case-const-chain-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("const_chain_prune fixture should slice");

    assert_eq!(report.packages, ["const_api", "const_support", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_const_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("const_api/Cargo.toml"));
    let api_root = read(output.join("const_api/src/lib.rs"));
    let api_live = read(output.join("const_api/src/live.rs"));
    let support_root = read(output.join("const_support/src/lib.rs"));
    let support_live = read(output.join("const_support/src/live.rs"));

    assert!(root_manifest.contains("../const_api"), "{root_manifest}");
    assert!(root_source.contains("pub fn selected_const_report"));
    assert!(root_source.contains("const_api::selected_const_report"));
    assert_absent("root/src/lib.rs", &root_source, &["dead_const_report"]);

    assert!(api_manifest.contains("../const_support"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_const_report"), "{api_root}");
    assert_absent(
        "const_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_const_report"],
    );
    assert!(api_live.contains("const_support::selected_const"));
    assert_absent(
        "const_api/src/live.rs",
        &api_live,
        &["dead_live_const_report"],
    );
    assert!(!output.join("const_api/src/dead.rs").exists());

    assert!(support_root.contains("mod live"), "{support_root}");
    assert!(support_root.contains("selected_const"), "{support_root}");
    assert_absent(
        "const_support/src/lib.rs",
        &support_root,
        &["mod dead", "DeadConstRecord", "dead_const"],
    );
    assert!(support_live.contains("pub const BASE"), "{support_live}");
    assert!(support_live.contains("pub const SCALE"), "{support_live}");
    assert!(support_live.contains("pub static LABEL"), "{support_live}");
    assert!(
        support_live.contains("pub struct ConstRecord"),
        "{support_live}"
    );
    assert!(support_live.contains("pub const OFFSET"), "{support_live}");
    assert!(support_live.contains("pub fn render"), "{support_live}");
    assert!(
        support_live.contains("pub fn selected_const"),
        "{support_live}"
    );
    assert_absent(
        "const_support/src/live.rs",
        &support_live,
        &["dead_method", "dead_live_const", "dead-const"],
    );
    assert!(!output.join("const_support/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_struct_update_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/struct_update_prune");
    let output = temp_path("slice-case-struct-update-prune-output");
    let target_dir = temp_path("slice-case-struct-update-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("struct_update_prune fixture should slice");

    assert_eq!(report.packages, ["root", "settings_api", "settings_model"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_settings_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("settings_api/Cargo.toml"));
    let api_root = read(output.join("settings_api/src/lib.rs"));
    let api_live = read(output.join("settings_api/src/live.rs"));
    let model_root = read(output.join("settings_model/src/lib.rs"));
    let model_live = read(output.join("settings_model/src/live.rs"));

    assert!(root_manifest.contains("../settings_api"), "{root_manifest}");
    assert!(root_source.contains("pub fn selected_settings_report"));
    assert!(root_source.contains("settings_api::selected_settings_report"));
    assert_absent("root/src/lib.rs", &root_source, &["dead_settings_report"]);

    assert!(api_manifest.contains("../settings_model"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_settings_report"), "{api_root}");
    assert_absent(
        "settings_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_settings_report"],
    );
    assert!(api_live.contains("settings_model::selected_settings"));
    assert_absent(
        "settings_api/src/live.rs",
        &api_live,
        &["dead_live_settings_report"],
    );
    assert!(!output.join("settings_api/src/dead.rs").exists());

    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(model_root.contains("selected_settings"), "{model_root}");
    assert!(model_root.contains("Settings"), "{model_root}");
    assert_absent(
        "settings_model/src/lib.rs",
        &model_root,
        &["mod dead", "DeadSettings", "dead_settings"],
    );
    assert!(model_live.contains("pub struct Settings"), "{model_live}");
    assert!(model_live.contains("impl Default for Settings"));
    assert!(model_live.contains("..Settings::default()"));
    assert!(model_live.contains("pub fn render"), "{model_live}");
    assert!(
        model_live.contains("pub fn selected_settings"),
        "{model_live}"
    );
    assert_absent(
        "settings_model/src/live.rs",
        &model_live,
        &["dead_method", "dead_live_settings", "dead-settings"],
    );
    assert!(!output.join("settings_model/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_pattern_destructure_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/pattern_destructure_prune");
    let output = temp_path("slice-case-pattern-destructure-prune-output");
    let target_dir = temp_path("slice-case-pattern-destructure-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("pattern_destructure_prune fixture should slice");

    assert_eq!(report.packages, ["pattern_api", "pattern_model", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_pattern_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("pattern_api/Cargo.toml"));
    let api_root = read(output.join("pattern_api/src/lib.rs"));
    let api_live = read(output.join("pattern_api/src/live.rs"));
    let model_root = read(output.join("pattern_model/src/lib.rs"));
    let model_live = read(output.join("pattern_model/src/live.rs"));

    assert!(root_manifest.contains("../pattern_api"), "{root_manifest}");
    assert!(root_source.contains("pub fn selected_pattern_report"));
    assert!(root_source.contains("pattern_api::selected_pattern_report"));
    assert_absent("root/src/lib.rs", &root_source, &["dead_pattern_report"]);

    assert!(api_manifest.contains("../pattern_model"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_pattern_report"), "{api_root}");
    assert_absent(
        "pattern_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_pattern_report"],
    );
    assert!(api_live.contains("pattern_model::selected_pattern"));
    assert_absent(
        "pattern_api/src/live.rs",
        &api_live,
        &["dead_live_pattern_report"],
    );
    assert!(!output.join("pattern_api/src/dead.rs").exists());

    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(model_root.contains("selected_pattern"), "{model_root}");
    assert_absent(
        "pattern_model/src/lib.rs",
        &model_root,
        &["mod dead", "DeadPattern", "dead_pattern"],
    );
    assert!(model_live.contains("enum PatternEvent"), "{model_live}");
    assert!(model_live.contains("Started"), "{model_live}");
    assert!(model_live.contains("pub fn parse_event"), "{model_live}");
    assert!(model_live.contains("let Some(PatternEvent::Started"));
    assert!(
        model_live.contains("pub fn selected_pattern"),
        "{model_live}"
    );
    assert_absent(
        "pattern_model/src/live.rs",
        &model_live,
        &["dead_method", "dead_live_pattern", "dead-pattern"],
    );
    assert!(!output.join("pattern_model/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_closure_combinator_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/closure_combinator_prune");
    let output = temp_path("slice-case-closure-combinator-prune-output");
    let target_dir = temp_path("slice-case-closure-combinator-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("closure_combinator_prune fixture should slice");

    assert_eq!(report.packages, ["closure_api", "closure_model", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_closure_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("closure_api/Cargo.toml"));
    let api_root = read(output.join("closure_api/src/lib.rs"));
    let api_live = read(output.join("closure_api/src/live.rs"));
    let model_root = read(output.join("closure_model/src/lib.rs"));
    let model_live = read(output.join("closure_model/src/live.rs"));

    assert!(root_manifest.contains("../closure_api"), "{root_manifest}");
    assert!(root_source.contains("pub fn selected_closure_report"));
    assert!(root_source.contains("closure_api::selected_closure_report"));
    assert_absent("root/src/lib.rs", &root_source, &["dead_closure_report"]);

    assert!(api_manifest.contains("../closure_model"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_closure_report"), "{api_root}");
    assert_absent(
        "closure_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_closure_report"],
    );
    assert!(api_live.contains("closure_model::selected_closure"));
    assert_absent(
        "closure_api/src/live.rs",
        &api_live,
        &["dead_live_closure_report"],
    );
    assert!(!output.join("closure_api/src/dead.rs").exists());

    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(model_root.contains("selected_closure"), "{model_root}");
    assert!(model_root.contains("RawItem"), "{model_root}");
    assert!(model_root.contains("CleanItem"), "{model_root}");
    assert!(model_root.contains("CleanError"), "{model_root}");
    assert_absent(
        "closure_model/src/lib.rs",
        &model_root,
        &["mod dead", "DeadClosure", "dead_closure"],
    );
    assert!(model_live.contains("pub struct RawItem"), "{model_live}");
    assert!(model_live.contains("pub fn clean"), "{model_live}");
    assert!(model_live.contains("pub struct CleanItem"), "{model_live}");
    assert!(model_live.contains("pub enum CleanError"), "{model_live}");
    assert!(model_live.contains(".transpose()"), "{model_live}");
    assert!(
        model_live.contains("pub fn selected_closure"),
        "{model_live}"
    );
    assert_absent(
        "closure_model/src/live.rs",
        &model_live,
        &["dead_method", "dead_live_closure", "dead-raw"],
    );
    assert!(!output.join("closure_model/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_generic_bound_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/generic_bound_prune");
    let output = temp_path("slice-case-generic-bound-prune-output");
    let target_dir = temp_path("slice-case-generic-bound-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("generic_bound_prune fixture should slice");

    assert_eq!(report.packages, ["generic_api", "generic_model", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_generic_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("generic_api/Cargo.toml"));
    let api_root = read(output.join("generic_api/src/lib.rs"));
    let api_live = read(output.join("generic_api/src/live.rs"));
    let model_root = read(output.join("generic_model/src/lib.rs"));
    let model_live = read(output.join("generic_model/src/live.rs"));

    assert!(root_manifest.contains("../generic_api"), "{root_manifest}");
    assert!(root_source.contains("pub fn selected_generic_report"));
    assert!(root_source.contains("generic_api::selected_generic_report"));
    assert_absent("root/src/lib.rs", &root_source, &["dead_generic_report"]);

    assert!(api_manifest.contains("../generic_model"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_generic_report"), "{api_root}");
    assert_absent(
        "generic_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_generic_report"],
    );
    assert!(api_live.contains("generic_model::selected_generic"));
    assert_absent(
        "generic_api/src/live.rs",
        &api_live,
        &["dead_live_generic_report"],
    );
    assert!(!output.join("generic_api/src/dead.rs").exists());

    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(model_root.contains("selected_generic"), "{model_root}");
    assert!(model_root.contains("GenericRecord"), "{model_root}");
    assert!(model_root.contains("LabelRender"), "{model_root}");
    assert_absent(
        "generic_model/src/lib.rs",
        &model_root,
        &["mod dead", "DeadGeneric", "dead_generic"],
    );
    assert!(model_live.contains("pub trait LabelRender"), "{model_live}");
    assert!(
        model_live.contains("pub struct GenericRecord"),
        "{model_live}"
    );
    assert!(model_live.contains("impl LabelRender for GenericRecord"));
    assert!(model_live.contains("pub fn render_with_bound"));
    assert!(
        model_live.contains("pub fn selected_generic"),
        "{model_live}"
    );
    assert_absent(
        "generic_model/src/live.rs",
        &model_live,
        &["dead_method", "dead_live_generic", "dead-generic"],
    );
    assert!(!output.join("generic_model/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_enum_variant_constructor_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/enum_variant_constructor_prune");
    let output = temp_path("slice-case-enum-variant-constructor-prune-output");
    let target_dir = temp_path("slice-case-enum-variant-constructor-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("enum_variant_constructor_prune fixture should slice");

    assert_eq!(report.packages, ["enum_api", "enum_model", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_enum_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("enum_api/Cargo.toml"));
    let api_root = read(output.join("enum_api/src/lib.rs"));
    let api_live = read(output.join("enum_api/src/live.rs"));
    let model_root = read(output.join("enum_model/src/lib.rs"));
    let model_live = read(output.join("enum_model/src/live.rs"));

    assert!(root_manifest.contains("../enum_api"), "{root_manifest}");
    assert!(root_source.contains("pub fn selected_enum_report"));
    assert!(root_source.contains("enum_api::selected_enum_report"));
    assert_absent("root/src/lib.rs", &root_source, &["dead_enum_report"]);

    assert!(api_manifest.contains("../enum_model"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_enum_report"), "{api_root}");
    assert_absent(
        "enum_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_enum_report"],
    );
    assert!(api_live.contains("enum_model::selected_enum"));
    assert_absent(
        "enum_api/src/live.rs",
        &api_live,
        &["dead_live_enum_report"],
    );
    assert!(!output.join("enum_api/src/dead.rs").exists());

    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(model_root.contains("selected_enum"), "{model_root}");
    assert!(model_root.contains("WireEvent"), "{model_root}");
    assert_absent(
        "enum_model/src/lib.rs",
        &model_root,
        &["mod dead", "DeadEnum", "dead_enum"],
    );
    assert!(model_live.contains("pub enum WireEvent"), "{model_live}");
    assert!(model_live.contains("WireEvent::Named"), "{model_live}");
    assert!(model_live.contains("WireEvent::render"), "{model_live}");
    assert!(model_live.contains("pub fn selected_enum"), "{model_live}");
    assert_absent(
        "enum_model/src/live.rs",
        &model_live,
        &["dead_method", "dead_live_enum", "dead-enum"],
    );
    assert!(!output.join("enum_model/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_associated_projection_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/associated_projection_prune");
    let output = temp_path("slice-case-associated-projection-prune-output");
    let target_dir = temp_path("slice-case-associated-projection-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("associated_projection_prune fixture should slice");

    assert_eq!(
        report.packages,
        ["projection_api", "projection_model", "root"]
    );
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_projection_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("projection_api/Cargo.toml"));
    let api_root = read(output.join("projection_api/src/lib.rs"));
    let api_live = read(output.join("projection_api/src/live.rs"));
    let model_root = read(output.join("projection_model/src/lib.rs"));
    let model_live = read(output.join("projection_model/src/live.rs"));

    assert!(
        root_manifest.contains("../projection_api"),
        "{root_manifest}"
    );
    assert!(root_source.contains("pub fn selected_projection_report"));
    assert!(root_source.contains("projection_api::selected_projection_report"));
    assert_absent("root/src/lib.rs", &root_source, &["dead_projection_report"]);

    assert!(
        api_manifest.contains("../projection_model"),
        "{api_manifest}"
    );
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(
        api_root.contains("selected_projection_report"),
        "{api_root}"
    );
    assert_absent(
        "projection_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_projection_report"],
    );
    assert!(api_live.contains("projection_model::selected_projection"));
    assert_absent(
        "projection_api/src/live.rs",
        &api_live,
        &["dead_live_projection_report"],
    );
    assert!(!output.join("projection_api/src/dead.rs").exists());

    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(model_root.contains("selected_projection"), "{model_root}");
    assert!(model_root.contains("ProjectionResolver"), "{model_root}");
    assert!(model_root.contains("ProjectionDto"), "{model_root}");
    assert_absent(
        "projection_model/src/lib.rs",
        &model_root,
        &["mod dead", "DeadProjection", "dead_projection"],
    );
    assert!(model_live.contains("pub trait Resolver"), "{model_live}");
    assert!(
        model_live.contains("type Output = ProjectionDto"),
        "{model_live}"
    );
    assert!(model_live.contains("R::Output: ProjectionRender"));
    assert!(model_live.contains("pub fn render_projection"));
    assert!(
        model_live.contains("pub fn selected_projection"),
        "{model_live}"
    );
    assert_absent(
        "projection_model/src/live.rs",
        &model_live,
        &["dead_method", "dead_live_projection", "dead-projection"],
    );
    assert!(!output.join("projection_model/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_iterator_method_reference_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/iterator_method_ref_prune");
    let output = temp_path("slice-case-iterator-method-ref-prune-output");
    let target_dir = temp_path("slice-case-iterator-method-ref-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("iterator_method_ref_prune fixture should slice");

    assert_eq!(report.packages, ["iterator_api", "iterator_model", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_iterator_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("iterator_api/Cargo.toml"));
    let api_root = read(output.join("iterator_api/src/lib.rs"));
    let api_live = read(output.join("iterator_api/src/live.rs"));
    let model_root = read(output.join("iterator_model/src/lib.rs"));
    let model_live = read(output.join("iterator_model/src/live.rs"));

    assert!(root_manifest.contains("../iterator_api"), "{root_manifest}");
    assert!(root_source.contains("pub fn selected_iterator_report"));
    assert!(root_source.contains("iterator_api::selected_iterator_report"));
    assert_absent("root/src/lib.rs", &root_source, &["dead_iterator_report"]);

    assert!(api_manifest.contains("../iterator_model"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_iterator_report"), "{api_root}");
    assert_absent(
        "iterator_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_iterator_report"],
    );
    assert!(api_live.contains("iterator_model::selected_iterator"));
    assert_absent(
        "iterator_api/src/live.rs",
        &api_live,
        &["dead_live_iterator_report"],
    );
    assert!(!output.join("iterator_api/src/dead.rs").exists());

    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(model_root.contains("selected_iterator"), "{model_root}");
    assert!(model_root.contains("IteratorItem"), "{model_root}");
    assert_absent(
        "iterator_model/src/lib.rs",
        &model_root,
        &["mod dead", "DeadIterator", "dead_iterator"],
    );
    assert!(
        model_live.contains("pub struct IteratorItem"),
        "{model_live}"
    );
    assert!(model_live.contains("pub fn render"), "{model_live}");
    assert!(model_live.contains("IteratorItem::render"), "{model_live}");
    assert!(
        model_live.contains("pub fn selected_iterator"),
        "{model_live}"
    );
    assert_absent(
        "iterator_model/src/live.rs",
        &model_live,
        &["dead_method", "dead_live_iterator", "dead-iterator"],
    );
    assert!(!output.join("iterator_model/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_display_format_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/display_format_prune");
    let output = temp_path("slice-case-display-format-prune-output");
    let target_dir = temp_path("slice-case-display-format-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("display_format_prune fixture should slice");

    assert_eq!(report.packages, ["display_api", "display_model", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_display_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("display_api/Cargo.toml"));
    let api_root = read(output.join("display_api/src/lib.rs"));
    let api_live = read(output.join("display_api/src/live.rs"));
    let model_root = read(output.join("display_model/src/lib.rs"));
    let model_live = read(output.join("display_model/src/live.rs"));

    assert!(root_manifest.contains("../display_api"), "{root_manifest}");
    assert!(root_source.contains("pub fn selected_display_report"));
    assert!(root_source.contains("display_api::selected_display_report"));
    assert_absent("root/src/lib.rs", &root_source, &["dead_display_report"]);

    assert!(api_manifest.contains("../display_model"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_display_report"), "{api_root}");
    assert_absent(
        "display_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_display_report"],
    );
    assert!(api_live.contains("display_model::selected_display"));
    assert_absent(
        "display_api/src/live.rs",
        &api_live,
        &["dead_live_display_report"],
    );
    assert!(!output.join("display_api/src/dead.rs").exists());

    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(model_root.contains("selected_display"), "{model_root}");
    assert!(model_root.contains("DisplayRecord"), "{model_root}");
    assert_absent(
        "display_model/src/lib.rs",
        &model_root,
        &["mod dead", "DeadDisplay", "dead_display"],
    );
    assert!(
        model_live.contains("pub struct DisplayRecord"),
        "{model_live}"
    );
    assert!(
        model_live.contains("impl fmt::Display for DisplayRecord"),
        "{model_live}"
    );
    assert!(model_live.contains("format!(\"record:{record}\")"));
    assert!(
        model_live.contains("pub fn selected_display"),
        "{model_live}"
    );
    assert_absent(
        "display_model/src/live.rs",
        &model_live,
        &["dead_method", "dead_live_display", "dead-display"],
    );
    assert!(!output.join("display_model/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_deref_method_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/deref_method_prune");
    let output = temp_path("slice-case-deref-method-prune-output");
    let target_dir = temp_path("slice-case-deref-method-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("deref_method_prune fixture should slice");

    assert_eq!(report.packages, ["deref_api", "deref_model", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_deref_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("deref_api/Cargo.toml"));
    let api_root = read(output.join("deref_api/src/lib.rs"));
    let api_live = read(output.join("deref_api/src/live.rs"));
    let model_root = read(output.join("deref_model/src/lib.rs"));
    let model_live = read(output.join("deref_model/src/live.rs"));

    assert!(root_manifest.contains("../deref_api"), "{root_manifest}");
    assert!(root_source.contains("pub fn selected_deref_report"));
    assert!(root_source.contains("deref_api::selected_deref_report"));
    assert_absent("root/src/lib.rs", &root_source, &["dead_deref_report"]);

    assert!(api_manifest.contains("../deref_model"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_deref_report"), "{api_root}");
    assert_absent(
        "deref_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_deref_report"],
    );
    assert!(api_live.contains("deref_model::selected_deref"));
    assert_absent(
        "deref_api/src/live.rs",
        &api_live,
        &["dead_live_deref_report"],
    );
    assert!(!output.join("deref_api/src/dead.rs").exists());

    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(model_root.contains("selected_deref"), "{model_root}");
    assert!(model_root.contains("DerefInner"), "{model_root}");
    assert!(model_root.contains("DerefWrapper"), "{model_root}");
    assert_absent(
        "deref_model/src/lib.rs",
        &model_root,
        &["mod dead", "DeadDeref", "dead_deref"],
    );
    assert!(model_live.contains("impl Deref for DerefWrapper"));
    assert!(model_live.contains("pub fn render"), "{model_live}");
    assert!(model_live.contains("DerefWrapper::new(raw).render()"));
    assert!(model_live.contains("pub fn selected_deref"), "{model_live}");
    assert_absent(
        "deref_model/src/live.rs",
        &model_live,
        &[
            "dead_inner_method",
            "dead_wrapper_method",
            "dead_live_deref",
            "dead-deref",
        ],
    );
    assert!(!output.join("deref_model/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_question_mark_conversion_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/question_mark_conversion_prune");
    let output = temp_path("slice-case-question-mark-conversion-prune-output");
    let target_dir = temp_path("slice-case-question-mark-conversion-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("question_mark_conversion_prune fixture should slice");

    assert_eq!(report.packages, ["question_api", "question_model", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_question_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("question_api/Cargo.toml"));
    let api_root = read(output.join("question_api/src/lib.rs"));
    let api_live = read(output.join("question_api/src/live.rs"));
    let model_root = read(output.join("question_model/src/lib.rs"));
    let model_live = read(output.join("question_model/src/live.rs"));

    assert!(root_manifest.contains("../question_api"), "{root_manifest}");
    assert!(root_source.contains("pub fn selected_question_report"));
    assert!(root_source.contains("question_api::selected_question_report"));
    assert_absent("root/src/lib.rs", &root_source, &["dead_question_report"]);

    assert!(api_manifest.contains("../question_model"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_question_report"), "{api_root}");
    assert_absent(
        "question_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_question_report"],
    );
    assert!(api_live.contains("question_model::selected_question"));
    assert!(api_live.contains("Err(err) => err.render()"));
    assert_absent(
        "question_api/src/live.rs",
        &api_live,
        &["dead_live_question_report"],
    );
    assert!(!output.join("question_api/src/dead.rs").exists());

    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(model_root.contains("selected_question"), "{model_root}");
    assert!(model_root.contains("ParsedQuestion"), "{model_root}");
    assert!(model_root.contains("WireError"), "{model_root}");
    assert_absent(
        "question_model/src/lib.rs",
        &model_root,
        &["mod dead", "DeadQuestion", "dead_question"],
    );
    assert!(
        model_live.contains("pub type QuestionResult"),
        "{model_live}"
    );
    assert!(model_live.contains("impl From<ParseError> for WireError"));
    assert!(model_live.contains("ParsedQuestion::parse(raw)?"));
    assert!(model_live.contains("pub fn selected_question"));
    assert_absent(
        "question_model/src/live.rs",
        &model_live,
        &["dead_method", "dead_live_question", "dead-question"],
    );
    assert!(!output.join("question_model/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_match_guard_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/match_guard_prune");
    let output = temp_path("slice-case-match-guard-prune-output");
    let target_dir = temp_path("slice-case-match-guard-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("match_guard_prune fixture should slice");

    assert_eq!(report.packages, ["guard_api", "guard_model", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_guard_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("guard_api/Cargo.toml"));
    let api_root = read(output.join("guard_api/src/lib.rs"));
    let api_live = read(output.join("guard_api/src/live.rs"));
    let model_root = read(output.join("guard_model/src/lib.rs"));
    let model_live = read(output.join("guard_model/src/live.rs"));

    assert!(root_manifest.contains("../guard_api"), "{root_manifest}");
    assert!(root_source.contains("pub fn selected_guard_report"));
    assert!(root_source.contains("guard_api::selected_guard_report"));
    assert_absent("root/src/lib.rs", &root_source, &["dead_guard_report"]);

    assert!(api_manifest.contains("../guard_model"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_guard_report"), "{api_root}");
    assert_absent(
        "guard_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_guard_report"],
    );
    assert!(api_live.contains("guard_model::selected_guard"));
    assert_absent(
        "guard_api/src/live.rs",
        &api_live,
        &["dead_live_guard_report"],
    );
    assert!(!output.join("guard_api/src/dead.rs").exists());

    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(model_root.contains("selected_guard"), "{model_root}");
    assert!(model_root.contains("GuardState"), "{model_root}");
    assert!(model_root.contains("GuardRecord"), "{model_root}");
    assert_absent(
        "guard_model/src/lib.rs",
        &model_root,
        &["mod dead", "DeadGuard", "dead_guard"],
    );
    assert!(model_live.contains("pub enum GuardState"), "{model_live}");
    assert!(model_live.contains("if record.is_ready()"));
    assert!(model_live.contains("record.render()"));
    assert!(model_live.contains("pub fn selected_guard"));
    assert_absent(
        "guard_model/src/live.rs",
        &model_live,
        &["dead_method", "dead_live_guard", "dead-guard"],
    );
    assert!(!output.join("guard_model/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_from_str_parse_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/from_str_parse_prune");
    let output = temp_path("slice-case-from-str-parse-prune-output");
    let target_dir = temp_path("slice-case-from-str-parse-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("from_str_parse_prune fixture should slice");

    assert_eq!(report.packages, ["parse_api", "parse_model", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_parse_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("parse_api/Cargo.toml"));
    let api_root = read(output.join("parse_api/src/lib.rs"));
    let api_live = read(output.join("parse_api/src/live.rs"));
    let model_root = read(output.join("parse_model/src/lib.rs"));
    let model_live = read(output.join("parse_model/src/live.rs"));

    assert!(root_manifest.contains("../parse_api"), "{root_manifest}");
    assert!(root_source.contains("pub fn selected_parse_report"));
    assert!(root_source.contains("parse_api::selected_parse_report"));
    assert_absent("root/src/lib.rs", &root_source, &["dead_parse_report"]);

    assert!(api_manifest.contains("../parse_model"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_parse_report"), "{api_root}");
    assert_absent(
        "parse_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_parse_report"],
    );
    assert!(api_live.contains("parse_model::selected_parse"));
    assert_absent(
        "parse_api/src/live.rs",
        &api_live,
        &["dead_live_parse_report"],
    );
    assert!(!output.join("parse_api/src/dead.rs").exists());

    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(model_root.contains("selected_parse"), "{model_root}");
    assert!(model_root.contains("ParseRecord"), "{model_root}");
    assert!(model_root.contains("ParseError"), "{model_root}");
    assert_absent(
        "parse_model/src/lib.rs",
        &model_root,
        &["mod dead", "DeadParse", "dead_parse"],
    );
    assert!(model_live.contains("impl FromStr for ParseRecord"));
    assert!(model_live.contains("type Err = ParseError"));
    assert!(model_live.contains("raw.parse::<ParseRecord>()"));
    assert!(model_live.contains("Err(err) => err.render()"));
    assert_absent(
        "parse_model/src/live.rs",
        &model_live,
        &["dead_method", "dead_live_parse", "dead-parse"],
    );
    assert!(!output.join("parse_model/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_index_operator_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/index_operator_prune");
    let output = temp_path("slice-case-index-operator-prune-output");
    let target_dir = temp_path("slice-case-index-operator-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("index_operator_prune fixture should slice");

    assert_eq!(report.packages, ["index_api", "index_model", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_index_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("index_api/Cargo.toml"));
    let api_root = read(output.join("index_api/src/lib.rs"));
    let api_live = read(output.join("index_api/src/live.rs"));
    let model_root = read(output.join("index_model/src/lib.rs"));
    let model_live = read(output.join("index_model/src/live.rs"));

    assert!(root_manifest.contains("../index_api"), "{root_manifest}");
    assert!(root_source.contains("pub fn selected_index_report"));
    assert!(root_source.contains("index_api::selected_index_report"));
    assert_absent("root/src/lib.rs", &root_source, &["dead_index_report"]);

    assert!(api_manifest.contains("../index_model"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_index_report"), "{api_root}");
    assert_absent(
        "index_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_index_report"],
    );
    assert!(api_live.contains("index_model::selected_index"));
    assert_absent(
        "index_api/src/live.rs",
        &api_live,
        &["dead_live_index_report"],
    );
    assert!(!output.join("index_api/src/dead.rs").exists());

    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(model_root.contains("selected_index"), "{model_root}");
    assert!(model_root.contains("IndexStore"), "{model_root}");
    assert!(model_root.contains("IndexItem"), "{model_root}");
    assert_absent(
        "index_model/src/lib.rs",
        &model_root,
        &["mod dead", "DeadIndex", "dead_index"],
    );
    assert!(model_live.contains("impl Index<usize> for IndexStore"));
    assert!(model_live.contains("type Output = IndexItem"));
    assert!(model_live.contains("store[0].render()"));
    assert!(model_live.contains("pub fn render"), "{model_live}");
    assert_absent(
        "index_model/src/live.rs",
        &model_live,
        &[
            "dead_method",
            "dead_store_method",
            "dead_live_index",
            "dead-index",
        ],
    );
    assert!(!output.join("index_model/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_option_field_payload_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/option_field_payload_prune");
    let output = temp_path("slice-case-option-field-payload-prune-output");
    let target_dir = temp_path("slice-case-option-field-payload-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("option_field_payload_prune fixture should slice");

    assert_eq!(report.packages, ["option_api", "option_model", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_option_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("option_api/Cargo.toml"));
    let api_root = read(output.join("option_api/src/lib.rs"));
    let api_live = read(output.join("option_api/src/live.rs"));
    let model_root = read(output.join("option_model/src/lib.rs"));
    let model_live = read(output.join("option_model/src/live.rs"));

    assert!(root_manifest.contains("../option_api"), "{root_manifest}");
    assert!(root_source.contains("pub fn selected_option_report"));
    assert!(root_source.contains("option_api::selected_option_report"));
    assert_absent("root/src/lib.rs", &root_source, &["dead_option_report"]);

    assert!(api_manifest.contains("../option_model"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_option_report"), "{api_root}");
    assert_absent(
        "option_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_option_report"],
    );
    assert!(api_live.contains("option_model::selected_option"));
    assert_absent(
        "option_api/src/live.rs",
        &api_live,
        &["dead_live_option_report"],
    );
    assert!(!output.join("option_api/src/dead.rs").exists());

    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(model_root.contains("selected_option"), "{model_root}");
    assert!(model_root.contains("OptionHolder"), "{model_root}");
    assert!(model_root.contains("OptionPayload"), "{model_root}");
    assert_absent(
        "option_model/src/lib.rs",
        &model_root,
        &["mod dead", "DeadOption", "dead_option"],
    );
    assert!(model_live.contains("holder.payload.as_ref()"));
    assert!(model_live.contains("payload.render()"));
    assert!(model_live.contains("pub fn render"), "{model_live}");
    assert_absent(
        "option_model/src/live.rs",
        &model_live,
        &[
            "dead_method",
            "dead_holder_method",
            "dead_live_option",
            "dead-option",
        ],
    );
    assert!(!output.join("option_model/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_closure_return_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/closure_return_prune");
    let output = temp_path("slice-case-closure-return-prune-output");
    let target_dir = temp_path("slice-case-closure-return-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("closure_return_prune fixture should slice");

    assert_eq!(
        report.packages,
        ["closure_return_api", "closure_return_model", "root"]
    );
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_closure_return_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("closure_return_api/Cargo.toml"));
    let api_root = read(output.join("closure_return_api/src/lib.rs"));
    let api_live = read(output.join("closure_return_api/src/live.rs"));
    let model_root = read(output.join("closure_return_model/src/lib.rs"));
    let model_live = read(output.join("closure_return_model/src/live.rs"));

    assert!(
        root_manifest.contains("../closure_return_api"),
        "{root_manifest}"
    );
    assert!(root_source.contains("pub fn selected_closure_return_report"));
    assert!(root_source.contains("closure_return_api::selected_closure_return_report"));
    assert_absent(
        "root/src/lib.rs",
        &root_source,
        &["dead_closure_return_report"],
    );

    assert!(
        api_manifest.contains("../closure_return_model"),
        "{api_manifest}"
    );
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(
        api_root.contains("selected_closure_return_report"),
        "{api_root}"
    );
    assert_absent(
        "closure_return_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_closure_return_report"],
    );
    assert!(api_live.contains("closure_return_model::selected_closure_return"));
    assert_absent(
        "closure_return_api/src/live.rs",
        &api_live,
        &["dead_live_closure_return_report"],
    );
    assert!(!output.join("closure_return_api/src/dead.rs").exists());

    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(
        model_root.contains("selected_closure_return"),
        "{model_root}"
    );
    assert!(model_root.contains("ClosureReturnRecord"), "{model_root}");
    assert_absent(
        "closure_return_model/src/lib.rs",
        &model_root,
        &["mod dead", "DeadClosureReturn", "dead_closure_return"],
    );
    assert!(model_live.contains("let build = || ClosureReturnRecord::new(raw)"));
    assert!(model_live.contains("build().render()"));
    assert!(model_live.contains("pub fn render"), "{model_live}");
    assert_absent(
        "closure_return_model/src/live.rs",
        &model_live,
        &[
            "dead_method",
            "dead_live_closure_return",
            "dead-closure-return",
        ],
    );
    assert!(!output.join("closure_return_model/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_try_from_transpose_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/try_from_transpose_prune");
    let output = temp_path("slice-case-try-from-transpose-prune-output");
    let target_dir = temp_path("slice-case-try-from-transpose-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("try_from_transpose_prune fixture should slice");

    assert_eq!(
        report.packages,
        ["root", "transpose_api", "transpose_model"]
    );
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_transpose_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("transpose_api/Cargo.toml"));
    let api_root = read(output.join("transpose_api/src/lib.rs"));
    let api_live = read(output.join("transpose_api/src/live.rs"));
    let model_root = read(output.join("transpose_model/src/lib.rs"));
    let model_live = read(output.join("transpose_model/src/live.rs"));

    assert!(
        root_manifest.contains("../transpose_api"),
        "{root_manifest}"
    );
    assert!(root_source.contains("pub fn selected_transpose_report"));
    assert!(root_source.contains("transpose_api::selected_transpose_report"));
    assert_absent("root/src/lib.rs", &root_source, &["dead_transpose_report"]);

    assert!(
        api_manifest.contains("../transpose_model"),
        "{api_manifest}"
    );
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_transpose_report"), "{api_root}");
    assert_absent(
        "transpose_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_transpose_report"],
    );
    assert!(api_live.contains("transpose_model::parse_optional_specs"));
    assert_absent(
        "transpose_api/src/live.rs",
        &api_live,
        &["dead_live_transpose_report"],
    );
    assert!(!output.join("transpose_api/src/dead.rs").exists());

    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(model_root.contains("parse_optional_specs"), "{model_root}");
    assert!(model_root.contains("DynamicSpec"), "{model_root}");
    assert!(model_root.contains("TransposeError"), "{model_root}");
    assert_absent(
        "transpose_model/src/lib.rs",
        &model_root,
        &["mod dead", "DeadTranspose", "dead_transpose"],
    );
    assert!(model_live.contains(".transpose()"), "{model_live}");
    assert!(model_live.contains("impl TryFrom<WireSpec> for DynamicSpec"));
    assert!(model_live.contains("pub fn render"), "{model_live}");
    assert_absent(
        "transpose_model/src/live.rs",
        &model_live,
        &["dead_method", "dead_live_transpose", "dead-transpose"],
    );
    assert!(!output.join("transpose_model/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_returned_object_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/returned_object_prune");
    let output = temp_path("slice-case-returned-object-prune-output");
    let target_dir = temp_path("slice-case-returned-object-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("returned_object_prune fixture should slice");

    assert_eq!(report.packages, ["returned_api", "returned_model", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_returned_subscription"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("returned_api/Cargo.toml"));
    let api_root = read(output.join("returned_api/src/lib.rs"));
    let api_live = read(output.join("returned_api/src/live.rs"));
    let model_root = read(output.join("returned_model/src/lib.rs"));
    let model_live = read(output.join("returned_model/src/live.rs"));

    assert!(root_manifest.contains("../returned_api"), "{root_manifest}");
    assert!(root_source.contains("pub fn selected_returned_subscription"));
    assert!(root_source.contains("returned_api::ReturnedSubscription"));
    assert_absent("root/src/lib.rs", &root_source, &["dead_returned_report"]);

    assert!(api_manifest.contains("../returned_model"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("ReturnedSubscription"), "{api_root}");
    assert_absent(
        "returned_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_returned_report"],
    );
    assert!(api_live.contains("pub use returned_model::ReturnedSubscription"));
    assert_absent(
        "returned_api/src/live.rs",
        &api_live,
        &["dead_live_returned_report"],
    );
    assert!(!output.join("returned_api/src/dead.rs").exists());

    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(model_root.contains("open_subscription"), "{model_root}");
    assert!(model_root.contains("ReturnedSubscription"), "{model_root}");
    assert!(model_root.contains("ReturnedEvent"), "{model_root}");
    assert_absent(
        "returned_model/src/lib.rs",
        &model_root,
        &["mod dead", "DeadReturned", "dead_returned"],
    );
    assert!(model_live.contains("pub fn next_event"), "{model_live}");
    assert!(model_live.contains("pub fn close"), "{model_live}");
    assert_absent(
        "returned_model/src/live.rs",
        &model_live,
        &["dead_method", "dead_event_method", "dead_live_returned"],
    );
    assert!(!output.join("returned_model/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_method_dispatch_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/method_dispatch_prune");
    let output = temp_path("slice-case-method-dispatch-prune-output");
    let target_dir = temp_path("slice-case-method-dispatch-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("method_dispatch_prune fixture should slice");

    assert_eq!(report.packages, ["dispatch_api", "dispatch_model", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_dispatch_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("dispatch_api/Cargo.toml"));
    let api_root = read(output.join("dispatch_api/src/lib.rs"));
    let api_live = read(output.join("dispatch_api/src/live.rs"));
    let model_root = read(output.join("dispatch_model/src/lib.rs"));
    let model_live = read(output.join("dispatch_model/src/live.rs"));

    assert!(root_manifest.contains("../dispatch_api"), "{root_manifest}");
    assert!(root_source.contains("pub fn selected_dispatch_report"));
    assert!(root_source.contains("dispatch_api::selected_dispatch_report"));
    assert_absent("root/src/lib.rs", &root_source, &["dead_dispatch_report"]);

    assert!(api_manifest.contains("../dispatch_model"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_dispatch_report"), "{api_root}");
    assert_absent(
        "dispatch_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_dispatch_report"],
    );
    assert!(api_live.contains("dispatch_model::selected_dispatch"));
    assert_absent(
        "dispatch_api/src/live.rs",
        &api_live,
        &["dead_live_dispatch_report"],
    );
    assert!(!output.join("dispatch_api/src/dead.rs").exists());

    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(model_root.contains("selected_dispatch"), "{model_root}");
    assert!(model_root.contains("DispatchMethod"), "{model_root}");
    assert!(model_root.contains("StartParams"), "{model_root}");
    assert_absent(
        "dispatch_model/src/lib.rs",
        &model_root,
        &["mod dead", "DeadDispatch", "dead_dispatch", "StopParams"],
    );
    assert!(model_live.contains("Unknown { method }"), "{model_live}");
    assert!(model_live.contains("Start(StartParams)"), "{model_live}");
    assert_absent(
        "dispatch_model/src/live.rs",
        &model_live,
        &["dead_method", "dead_live_dispatch", "dead-dispatch"],
    );
    assert!(!output.join("dispatch_model/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_map_payload_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/map_payload_prune");
    let output = temp_path("slice-case-map-payload-prune-output");
    let target_dir = temp_path("slice-case-map-payload-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("map_payload_prune fixture should slice");

    assert_eq!(report.packages, ["map_api", "map_model", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_map_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("map_api/Cargo.toml"));
    let api_root = read(output.join("map_api/src/lib.rs"));
    let api_live = read(output.join("map_api/src/live.rs"));
    let model_root = read(output.join("map_model/src/lib.rs"));
    let model_live = read(output.join("map_model/src/live.rs"));

    assert!(root_manifest.contains("../map_api"), "{root_manifest}");
    assert!(root_source.contains("pub fn selected_map_report"));
    assert!(root_source.contains("map_api::selected_map_report"));
    assert_absent("root/src/lib.rs", &root_source, &["dead_map_report"]);

    assert!(api_manifest.contains("../map_model"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_map_report"), "{api_root}");
    assert_absent(
        "map_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_map_report"],
    );
    assert!(api_live.contains("map_model::selected_map"));
    assert_absent("map_api/src/live.rs", &api_live, &["dead_live_map_report"]);
    assert!(!output.join("map_api/src/dead.rs").exists());

    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(model_root.contains("selected_map"), "{model_root}");
    assert!(model_root.contains("MapPayload"), "{model_root}");
    assert!(model_root.contains("MapEntry"), "{model_root}");
    assert_absent(
        "map_model/src/lib.rs",
        &model_root,
        &["mod dead", "DeadMap", "dead_map"],
    );
    assert!(model_live.contains(".values().map(MapEntry::render)"));
    assert_absent(
        "map_model/src/live.rs",
        &model_live,
        &["dead_method", "dead_live_map", "dead-map"],
    );
    assert!(!output.join("map_model/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_free_function_closure_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/free_function_closure_prune");
    let output = temp_path("slice-case-free-function-closure-prune-output");
    let target_dir = temp_path("slice-case-free-function-closure-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("free_function_closure_prune fixture should slice");

    assert_eq!(report.packages, ["closure_api", "closure_model", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_free_closure_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("closure_api/Cargo.toml"));
    let api_root = read(output.join("closure_api/src/lib.rs"));
    let api_live = read(output.join("closure_api/src/live.rs"));
    let model_root = read(output.join("closure_model/src/lib.rs"));
    let model_live = read(output.join("closure_model/src/live.rs"));

    assert!(root_manifest.contains("../closure_api"), "{root_manifest}");
    assert!(root_source.contains("pub fn selected_free_closure_report"));
    assert!(root_source.contains("closure_api::selected_free_closure_report"));
    assert_absent(
        "root/src/lib.rs",
        &root_source,
        &["dead_free_closure_report"],
    );

    assert!(api_manifest.contains("../closure_model"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(
        api_root.contains("selected_free_closure_report"),
        "{api_root}"
    );
    assert_absent(
        "closure_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_free_closure_report"],
    );
    assert!(api_live.contains("closure_model::with_free_payload"));
    assert!(api_live.contains("|payload| payload.render()"));
    assert_absent(
        "closure_api/src/live.rs",
        &api_live,
        &["dead_live_free_closure_report"],
    );
    assert!(!output.join("closure_api/src/dead.rs").exists());

    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(model_root.contains("with_free_payload"), "{model_root}");
    assert!(model_root.contains("FreePayload"), "{model_root}");
    assert_absent(
        "closure_model/src/lib.rs",
        &model_root,
        &["mod dead", "DeadFreeClosure", "dead_free_closure"],
    );
    assert!(model_live.contains("pub fn render"), "{model_live}");
    assert!(model_live.contains("render(FreePayload::new(raw))"));
    assert_absent(
        "closure_model/src/live.rs",
        &model_live,
        &["dead_method", "dead_live_free_closure", "dead-free-closure"],
    );
    assert!(!output.join("closure_model/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_result_map_err_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/result_map_err_prune");
    let output = temp_path("slice-case-result-map-err-prune-output");
    let target_dir = temp_path("slice-case-result-map-err-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("result_map_err_prune fixture should slice");

    assert_eq!(report.packages, ["result_api", "result_model", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_result_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("result_api/Cargo.toml"));
    let api_root = read(output.join("result_api/src/lib.rs"));
    let api_live = read(output.join("result_api/src/live.rs"));
    let model_root = read(output.join("result_model/src/lib.rs"));
    let model_live = read(output.join("result_model/src/live.rs"));

    assert!(root_manifest.contains("../result_api"), "{root_manifest}");
    assert!(root_source.contains("pub fn selected_result_report"));
    assert!(root_source.contains("result_api::selected_result_report"));
    assert_absent("root/src/lib.rs", &root_source, &["dead_result_report"]);

    assert!(api_manifest.contains("../result_model"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_result_report"), "{api_root}");
    assert_absent(
        "result_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_result_report"],
    );
    assert!(api_live.contains("result_model::parse_result"));
    assert!(api_live.contains(".map_err(|err| err.render())"));
    assert_absent(
        "result_api/src/live.rs",
        &api_live,
        &["dead_live_result_report"],
    );
    assert!(!output.join("result_api/src/dead.rs").exists());

    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(model_root.contains("parse_result"), "{model_root}");
    assert!(model_root.contains("ResultValue"), "{model_root}");
    assert!(model_root.contains("ResultError"), "{model_root}");
    assert_absent(
        "result_model/src/lib.rs",
        &model_root,
        &["mod dead", "DeadResult", "dead_result"],
    );
    assert!(model_live.contains("pub fn render"), "{model_live}");
    assert_absent(
        "result_model/src/live.rs",
        &model_live,
        &["dead_method", "dead_live_result", "dead-result"],
    );
    assert!(!output.join("result_model/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_const_generic_array_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/const_generic_array_prune");
    let output = temp_path("slice-case-const-generic-array-prune-output");
    let target_dir = temp_path("slice-case-const-generic-array-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("const_generic_array_prune fixture should slice");

    assert_eq!(report.packages, ["array_api", "array_model", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_array_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("array_api/Cargo.toml"));
    let api_root = read(output.join("array_api/src/lib.rs"));
    let api_live = read(output.join("array_api/src/live.rs"));
    let model_root = read(output.join("array_model/src/lib.rs"));
    let model_live = read(output.join("array_model/src/live.rs"));

    assert!(root_manifest.contains("../array_api"), "{root_manifest}");
    assert!(root_source.contains("pub fn selected_array_report"));
    assert!(root_source.contains("array_api::selected_array_report"));
    assert_absent("root/src/lib.rs", &root_source, &["dead_array_report"]);

    assert!(api_manifest.contains("../array_model"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_array_report"), "{api_root}");
    assert_absent(
        "array_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_array_report"],
    );
    assert!(api_live.contains("array_model::selected_array"));
    assert_absent(
        "array_api/src/live.rs",
        &api_live,
        &["dead_live_array_report"],
    );
    assert!(!output.join("array_api/src/dead.rs").exists());

    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(model_root.contains("selected_array"), "{model_root}");
    assert!(model_root.contains("ArrayPayload"), "{model_root}");
    assert!(model_root.contains("LIVE_LEN"), "{model_root}");
    assert_absent(
        "array_model/src/lib.rs",
        &model_root,
        &["mod dead", "DeadArray", "dead_array", "DEAD_LEN"],
    );
    assert!(model_live.contains("ArrayPayload<LIVE_LEN>"));
    assert!(model_live.contains("pub fn render"), "{model_live}");
    assert_absent(
        "array_model/src/live.rs",
        &model_live,
        &["dead_method", "dead_live_array", "dead-array"],
    );
    assert!(!output.join("array_model/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_newtype_tuple_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/newtype_tuple_prune");
    let output = temp_path("slice-case-newtype-tuple-prune-output");
    let target_dir = temp_path("slice-case-newtype-tuple-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("newtype_tuple_prune fixture should slice");

    assert_eq!(report.packages, ["newtype_api", "newtype_model", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_newtype_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("newtype_api/Cargo.toml"));
    let api_root = read(output.join("newtype_api/src/lib.rs"));
    let api_live = read(output.join("newtype_api/src/live.rs"));
    let model_root = read(output.join("newtype_model/src/lib.rs"));
    let model_live = read(output.join("newtype_model/src/live.rs"));

    assert!(root_manifest.contains("../newtype_api"), "{root_manifest}");
    assert!(root_source.contains("pub fn selected_newtype_report"));
    assert!(root_source.contains("newtype_api::selected_newtype_report"));
    assert_absent("root/src/lib.rs", &root_source, &["dead_newtype_report"]);

    assert!(api_manifest.contains("../newtype_model"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_newtype_report"), "{api_root}");
    assert_absent(
        "newtype_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_newtype_report"],
    );
    assert!(api_live.contains("newtype_model::selected_newtype"));
    assert_absent(
        "newtype_api/src/live.rs",
        &api_live,
        &["dead_live_newtype_report"],
    );
    assert!(!output.join("newtype_api/src/dead.rs").exists());

    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(model_root.contains("selected_newtype"), "{model_root}");
    assert!(model_root.contains("NewtypeRecord"), "{model_root}");
    assert!(model_root.contains("NewtypeInner"), "{model_root}");
    assert_absent(
        "newtype_model/src/lib.rs",
        &model_root,
        &["mod dead", "DeadNewtype", "dead_newtype"],
    );
    assert!(model_live.contains("self.0.render()"), "{model_live}");
    assert_absent(
        "newtype_model/src/live.rs",
        &model_live,
        &[
            "dead_method",
            "dead_inner_method",
            "dead_live_newtype",
            "dead-newtype",
        ],
    );
    assert!(!output.join("newtype_model/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_lazy_parser_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/lazy_parser_prune");
    let output = temp_path("slice-case-lazy-parser-prune-output");
    let target_dir = temp_path("slice-case-lazy-parser-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("lazy_parser_prune fixture should slice");

    assert_eq!(report.packages, ["lazy_api", "lazy_model", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_lazy_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("lazy_api/Cargo.toml"));
    let api_root = read(output.join("lazy_api/src/lib.rs"));
    let api_live = read(output.join("lazy_api/src/live.rs"));
    let model_root = read(output.join("lazy_model/src/lib.rs"));
    let model_live = read(output.join("lazy_model/src/live.rs"));

    assert!(root_manifest.contains("../lazy_api"), "{root_manifest}");
    assert!(root_source.contains("pub fn selected_lazy_report"));
    assert!(root_source.contains("lazy_api::selected_lazy_report"));
    assert_absent("root/src/lib.rs", &root_source, &["dead_lazy_report"]);

    assert!(api_manifest.contains("../lazy_model"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_lazy_report"), "{api_root}");
    assert_absent(
        "lazy_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_lazy_report"],
    );
    assert!(api_live.contains("lazy_model::selected_lazy"));
    assert_absent(
        "lazy_api/src/live.rs",
        &api_live,
        &["dead_live_lazy_report"],
    );
    assert!(!output.join("lazy_api/src/dead.rs").exists());

    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(model_root.contains("selected_lazy"), "{model_root}");
    assert!(model_root.contains("LazyParser"), "{model_root}");
    assert_absent(
        "lazy_model/src/lib.rs",
        &model_root,
        &["mod dead", "DeadLazyParser", "dead_lazy"],
    );
    assert!(model_live.contains("static LIVE_PARSER"));
    assert!(model_live.contains("LazyToken"));
    assert!(model_live.contains("LazyLock::new"));
    assert!(model_live.contains("LIVE_PARSER.parse(raw).render()"));
    assert_absent(
        "lazy_model/src/live.rs",
        &model_live,
        &["dead_method", "dead_live_lazy", "dead-lazy", "dead-token"],
    );
    assert!(!output.join("lazy_model/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_global_mutex_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/global_mutex_prune");
    let output = temp_path("slice-case-global-mutex-prune-output");
    let target_dir = temp_path("slice-case-global-mutex-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("global_mutex_prune fixture should slice");

    assert_eq!(report.packages, ["mutex_api", "mutex_model", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_mutex_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("mutex_api/Cargo.toml"));
    let api_root = read(output.join("mutex_api/src/lib.rs"));
    let api_live = read(output.join("mutex_api/src/live.rs"));
    let model_root = read(output.join("mutex_model/src/lib.rs"));
    let model_live = read(output.join("mutex_model/src/live.rs"));

    assert!(root_manifest.contains("../mutex_api"), "{root_manifest}");
    assert!(root_source.contains("pub fn selected_mutex_report"));
    assert!(root_source.contains("mutex_api::selected_mutex_report"));
    assert_absent("root/src/lib.rs", &root_source, &["dead_mutex_report"]);

    assert!(api_manifest.contains("../mutex_model"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_mutex_report"), "{api_root}");
    assert_absent(
        "mutex_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_mutex_report"],
    );
    assert!(api_live.contains("mutex_model::selected_mutex"));
    assert_absent(
        "mutex_api/src/live.rs",
        &api_live,
        &["dead_live_mutex_report"],
    );
    assert!(!output.join("mutex_api/src/dead.rs").exists());

    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(model_root.contains("selected_mutex"), "{model_root}");
    assert!(model_root.contains("MutexEntry"), "{model_root}");
    assert_absent(
        "mutex_model/src/lib.rs",
        &model_root,
        &["mod dead", "DeadMutexEntry", "dead_mutex"],
    );
    assert!(model_live.contains("static LIVE_REGISTRY"));
    assert!(model_live.contains("Mutex<Vec<MutexEntry>>"));
    assert!(model_live.contains("guard.last().map(MutexEntry::render)"));
    assert_absent(
        "mutex_model/src/live.rs",
        &model_live,
        &["dead_method", "dead_live_mutex", "dead-mutex"],
    );
    assert!(!output.join("mutex_model/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_protocol_projection_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/protocol_projection_prune");
    let output = temp_path("slice-case-protocol-projection-prune-output");
    let target_dir = temp_path("slice-case-protocol-projection-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("protocol_projection_prune fixture should slice");

    assert_eq!(report.packages, ["protocol_api", "protocol_model", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_protocol_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("protocol_api/Cargo.toml"));
    let api_root = read(output.join("protocol_api/src/lib.rs"));
    let api_live = read(output.join("protocol_api/src/live.rs"));
    let model_root = read(output.join("protocol_model/src/lib.rs"));
    let model_live = read(output.join("protocol_model/src/live.rs"));

    assert!(root_manifest.contains("../protocol_api"), "{root_manifest}");
    assert!(root_source.contains("pub fn selected_protocol_report"));
    assert!(root_source.contains("protocol_api::selected_protocol_report"));
    assert_absent("root/src/lib.rs", &root_source, &["dead_protocol_report"]);

    assert!(api_manifest.contains("../protocol_model"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_protocol_report"), "{api_root}");
    assert_absent(
        "protocol_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_protocol_report"],
    );
    assert!(api_live.contains("protocol_model::selected_protocol"));
    assert_absent(
        "protocol_api/src/live.rs",
        &api_live,
        &["dead_live_protocol_report"],
    );
    assert!(!output.join("protocol_api/src/dead.rs").exists());

    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(model_root.contains("selected_protocol"), "{model_root}");
    assert!(model_root.contains("ProtocolPayload"), "{model_root}");
    assert_absent(
        "protocol_model/src/lib.rs",
        &model_root,
        &["mod dead", "DeadProtocol", "dead_protocol", "ProtocolEvent"],
    );
    assert!(model_live.contains("ProtocolEvent::Accepted"));
    assert!(model_live.contains("payload.render()"));
    assert_absent(
        "protocol_model/src/live.rs",
        &model_live,
        &[
            "dead_method",
            "dead_live_protocol",
            "dead-payload",
            "DeadProtocol",
        ],
    );
    assert!(!output.join("protocol_model/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_iterator_fold_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/iterator_fold_prune");
    let output = temp_path("slice-case-iterator-fold-prune-output");
    let target_dir = temp_path("slice-case-iterator-fold-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("iterator_fold_prune fixture should slice");

    assert_eq!(report.packages, ["fold_api", "fold_model", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_fold_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("fold_api/Cargo.toml"));
    let api_root = read(output.join("fold_api/src/lib.rs"));
    let api_live = read(output.join("fold_api/src/live.rs"));
    let model_root = read(output.join("fold_model/src/lib.rs"));
    let model_live = read(output.join("fold_model/src/live.rs"));

    assert!(root_manifest.contains("../fold_api"), "{root_manifest}");
    assert!(root_source.contains("pub fn selected_fold_report"));
    assert!(root_source.contains("fold_api::selected_fold_report"));
    assert_absent("root/src/lib.rs", &root_source, &["dead_fold_report"]);

    assert!(api_manifest.contains("../fold_model"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_fold_report"), "{api_root}");
    assert_absent(
        "fold_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_fold_report"],
    );
    assert!(api_live.contains("fold_model::selected_fold"));
    assert_absent(
        "fold_api/src/live.rs",
        &api_live,
        &["dead_live_fold_report"],
    );
    assert!(!output.join("fold_api/src/dead.rs").exists());

    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(model_root.contains("selected_fold"), "{model_root}");
    assert!(model_root.contains("FoldPart"), "{model_root}");
    assert!(model_root.contains("FoldSummary"), "{model_root}");
    assert_absent(
        "fold_model/src/lib.rs",
        &model_root,
        &["mod dead", "DeadFold", "dead_fold"],
    );
    assert!(model_live.contains(".fold(FoldSummary::new()"));
    assert!(model_live.contains("part.render()"));
    assert_absent(
        "fold_model/src/live.rs",
        &model_live,
        &["dead_method", "dead_live_fold", "dead-part", "dead-summary"],
    );
    assert!(!output.join("fold_model/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_iterator_filter_map_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/iterator_filter_map_prune");
    let output = temp_path("slice-case-iterator-filter-map-prune-output");
    let target_dir = temp_path("slice-case-iterator-filter-map-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("iterator_filter_map_prune fixture should slice");

    assert_eq!(report.packages, ["filter_api", "filter_model", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_filter_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("filter_api/Cargo.toml"));
    let api_root = read(output.join("filter_api/src/lib.rs"));
    let api_live = read(output.join("filter_api/src/live.rs"));
    let model_root = read(output.join("filter_model/src/lib.rs"));
    let model_live = read(output.join("filter_model/src/live.rs"));

    assert!(root_manifest.contains("../filter_api"), "{root_manifest}");
    assert!(root_source.contains("pub fn selected_filter_report"));
    assert!(root_source.contains("filter_api::selected_filter_report"));
    assert_absent("root/src/lib.rs", &root_source, &["dead_filter_report"]);

    assert!(api_manifest.contains("../filter_model"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_filter_report"), "{api_root}");
    assert_absent(
        "filter_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_filter_report"],
    );
    assert!(api_live.contains("filter_model::selected_filter"));
    assert_absent(
        "filter_api/src/live.rs",
        &api_live,
        &["dead_live_filter_report"],
    );
    assert!(!output.join("filter_api/src/dead.rs").exists());

    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(model_root.contains("selected_filter"), "{model_root}");
    assert_absent(
        "filter_model/src/lib.rs",
        &model_root,
        &["mod dead", "DeadFilterEntry", "dead_filter"],
    );
    assert!(model_live.contains("FilterEntry"), "{model_live}");
    assert!(model_live.contains("filter_map(|entry| entry.render_if_live())"));
    assert_absent(
        "filter_model/src/live.rs",
        &model_live,
        &["dead_method", "dead_live_filter", "dead-filter"],
    );
    assert!(!output.join("filter_model/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_iterator_any_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/iterator_any_prune");
    let output = temp_path("slice-case-iterator-any-prune-output");
    let target_dir = temp_path("slice-case-iterator-any-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("iterator_any_prune fixture should slice");

    assert_eq!(report.packages, ["any_api", "any_model", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_any_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("any_api/Cargo.toml"));
    let api_root = read(output.join("any_api/src/lib.rs"));
    let api_live = read(output.join("any_api/src/live.rs"));
    let model_root = read(output.join("any_model/src/lib.rs"));
    let model_live = read(output.join("any_model/src/live.rs"));

    assert!(root_manifest.contains("../any_api"), "{root_manifest}");
    assert!(root_source.contains("pub fn selected_any_report"));
    assert!(root_source.contains("any_api::selected_any_report"));
    assert_absent("root/src/lib.rs", &root_source, &["dead_any_report"]);

    assert!(api_manifest.contains("../any_model"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_any_report"), "{api_root}");
    assert_absent(
        "any_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_any_report"],
    );
    assert!(api_live.contains("any_model::selected_any"));
    assert_absent("any_api/src/live.rs", &api_live, &["dead_live_any_report"]);
    assert!(!output.join("any_api/src/dead.rs").exists());

    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(model_root.contains("selected_any"), "{model_root}");
    assert_absent(
        "any_model/src/lib.rs",
        &model_root,
        &["mod dead", "DeadAnyFlag", "dead_any"],
    );
    assert!(model_live.contains("AnyFlag"), "{model_live}");
    assert!(model_live.contains(".iter().any(|flag| flag.is_live())"));
    assert!(model_live.contains("pub fn render"), "{model_live}");
    assert_absent(
        "any_model/src/live.rs",
        &model_live,
        &["dead_method", "dead_live_any", "dead-any"],
    );
    assert!(!output.join("any_model/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_result_unwrap_or_else_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/result_unwrap_or_else_prune");
    let output = temp_path("slice-case-result-unwrap-or-else-prune-output");
    let target_dir = temp_path("slice-case-result-unwrap-or-else-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("result_unwrap_or_else_prune fixture should slice");

    assert_eq!(report.packages, ["root", "unwrap_api", "unwrap_model"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_unwrap_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("unwrap_api/Cargo.toml"));
    let api_root = read(output.join("unwrap_api/src/lib.rs"));
    let api_live = read(output.join("unwrap_api/src/live.rs"));
    let model_root = read(output.join("unwrap_model/src/lib.rs"));
    let model_live = read(output.join("unwrap_model/src/live.rs"));

    assert!(root_manifest.contains("../unwrap_api"), "{root_manifest}");
    assert!(root_source.contains("pub fn selected_unwrap_report"));
    assert!(root_source.contains("unwrap_api::selected_unwrap_report"));
    assert_absent("root/src/lib.rs", &root_source, &["dead_unwrap_report"]);

    assert!(api_manifest.contains("../unwrap_model"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_unwrap_report"), "{api_root}");
    assert_absent(
        "unwrap_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_unwrap_report"],
    );
    assert!(api_live.contains("unwrap_model::selected_unwrap"));
    assert_absent(
        "unwrap_api/src/live.rs",
        &api_live,
        &["dead_live_unwrap_report"],
    );
    assert!(!output.join("unwrap_api/src/dead.rs").exists());

    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(model_root.contains("selected_unwrap"), "{model_root}");
    assert_absent(
        "unwrap_model/src/lib.rs",
        &model_root,
        &["mod dead", "DeadUnwrap", "dead_unwrap"],
    );
    assert!(model_live.contains("UnwrapValue"), "{model_live}");
    assert!(model_live.contains("UnwrapError"), "{model_live}");
    assert!(model_live.contains("unwrap_or_else(|err| err.recover())"));
    assert_absent(
        "unwrap_model/src/live.rs",
        &model_live,
        &["dead_method", "dead_live_unwrap", "dead-error"],
    );
    assert!(!output.join("unwrap_model/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_retain_sort_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/retain_sort_prune");
    let output = temp_path("slice-case-retain-sort-prune-output");
    let target_dir = temp_path("slice-case-retain-sort-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("retain_sort_prune fixture should slice");

    assert_eq!(report.packages, ["retain_api", "retain_model", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_retain_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("retain_api/Cargo.toml"));
    let api_root = read(output.join("retain_api/src/lib.rs"));
    let api_live = read(output.join("retain_api/src/live.rs"));
    let model_root = read(output.join("retain_model/src/lib.rs"));
    let model_live = read(output.join("retain_model/src/live.rs"));

    assert!(root_manifest.contains("../retain_api"), "{root_manifest}");
    assert!(root_source.contains("pub fn selected_retain_report"));
    assert!(root_source.contains("retain_api::selected_retain_report"));
    assert_absent("root/src/lib.rs", &root_source, &["dead_retain_report"]);

    assert!(api_manifest.contains("../retain_model"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_retain_report"), "{api_root}");
    assert_absent(
        "retain_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_retain_report"],
    );
    assert!(api_live.contains("retain_model::selected_retain"));
    assert_absent(
        "retain_api/src/live.rs",
        &api_live,
        &["dead_live_retain_report"],
    );
    assert!(!output.join("retain_api/src/dead.rs").exists());

    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(model_root.contains("selected_retain"), "{model_root}");
    assert_absent(
        "retain_model/src/lib.rs",
        &model_root,
        &["mod dead", "DeadRetainItem", "dead_retain"],
    );
    assert!(model_live.contains("RetainItem"), "{model_live}");
    assert!(model_live.contains("items.retain(|item| item.keep())"));
    assert!(model_live.contains("items.sort_by_key(|item| item.sort_key())"));
    assert_absent(
        "retain_model/src/live.rs",
        &model_live,
        &["dead_method", "dead_live_retain", "dead-retain"],
    );
    assert!(!output.join("retain_model/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

#[test]
fn prunes_iterator_for_each_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_for_each_prune",
        root_fn: "selected_for_each_report",
        api_pkg: "foreach_api",
        model_pkg: "foreach_model",
        api_fn: "selected_for_each_report",
        model_fn: "selected_for_each",
        model_required: &[
            "ForEachEvent",
            ".for_each(|event| rendered.push(event.render()))",
            "pub fn render",
        ],
        model_absent: &[
            "mod dead",
            "DeadForEachEvent",
            "dead_for_each",
            "dead_method",
            "dead_live_for_each",
            "dead-each",
        ],
    });
}

#[test]
fn prunes_iterator_flat_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_flat_map_prune",
        root_fn: "selected_flat_map_report",
        api_pkg: "flat_api",
        model_pkg: "flat_model",
        api_fn: "selected_flat_map_report",
        model_fn: "selected_flat_map",
        model_required: &[
            "FlatSegment",
            ".flat_map(|segment| segment.expand())",
            "pub fn expand",
        ],
        model_absent: &[
            "mod dead",
            "DeadFlatSegment",
            "dead_flat_map",
            "dead_method",
            "dead_live_flat_map",
            "dead-flat",
        ],
    });
}

#[test]
fn prunes_iterator_map_while_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_map_while_prune",
        root_fn: "selected_map_while_report",
        api_pkg: "while_api",
        model_pkg: "while_model",
        api_fn: "selected_map_while_report",
        model_fn: "selected_map_while",
        model_required: &[
            "WhileStep",
            ".map_while(|step| step.next_render())",
            "pub fn next_render",
        ],
        model_absent: &[
            "mod dead",
            "DeadWhileStep",
            "dead_map_while",
            "dead_method",
            "dead_live_map_while",
            "dead-while",
        ],
    });
}

#[test]
fn prunes_iterator_try_for_each_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_try_for_each_prune",
        root_fn: "selected_try_for_each_report",
        api_pkg: "try_api",
        model_pkg: "try_model",
        api_fn: "selected_try_for_each_report",
        model_fn: "selected_try_for_each",
        model_required: &[
            "TryStep",
            "TryError",
            ".try_for_each(|step| step.append_to(&mut rendered))",
            "pub fn append_to",
            "pub fn render",
        ],
        model_absent: &[
            "mod dead",
            "DeadTryStep",
            "dead_try_for_each",
            "dead_method",
            "dead_live_try_for_each",
            "dead-try",
        ],
    });
}

struct SupportSliceFixture<'a> {
    fixture_name: &'a str,
    root_fn: &'a str,
    api_pkg: &'a str,
    model_pkg: &'a str,
    api_fn: &'a str,
    model_fn: &'a str,
    model_required: &'a [&'a str],
    model_absent: &'a [&'a str],
}

fn assert_support_slice_fixture(expect: SupportSliceFixture<'_>) {
    let fixture = repo_root().join(format!("fixtures/slice_cases/{}", expect.fixture_name));
    let output = temp_path(&format!("slice-case-{}-output", expect.fixture_name));
    let target_dir = temp_path(&format!("slice-case-{}-target", expect.fixture_name));

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .unwrap_or_else(|err| panic!("{} fixture should slice: {err}", expect.fixture_name));

    let mut expected_packages = vec![
        expect.api_pkg.to_string(),
        expect.model_pkg.to_string(),
        "root".to_string(),
    ];
    expected_packages.sort();
    assert_eq!(report.packages, expected_packages);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == format!("root::{}", expect.root_fn)),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join(format!("{}/Cargo.toml", expect.api_pkg)));
    let api_root = read(output.join(format!("{}/src/lib.rs", expect.api_pkg)));
    let api_live = read(output.join(format!("{}/src/live.rs", expect.api_pkg)));
    let model_root = read(output.join(format!("{}/src/lib.rs", expect.model_pkg)));
    let model_live = read(output.join(format!("{}/src/live.rs", expect.model_pkg)));
    let dead_root_fn = expect.root_fn.replacen("selected", "dead", 1);
    let dead_api_fn = expect.api_fn.replacen("selected", "dead", 1);

    assert!(
        root_manifest.contains(&format!("../{}", expect.api_pkg)),
        "{root_manifest}"
    );
    assert!(
        root_source.contains(&format!("pub fn {}", expect.root_fn)),
        "{root_source}"
    );
    assert!(
        root_source.contains(&format!("{}::{}", expect.api_pkg, expect.api_fn)),
        "{root_source}"
    );
    assert_absent("root/src/lib.rs", &root_source, &[dead_root_fn.as_str()]);

    assert!(
        api_manifest.contains(&format!("../{}", expect.model_pkg)),
        "{api_manifest}"
    );
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains(expect.api_fn), "{api_root}");
    assert_absent(
        &format!("{}/src/lib.rs", expect.api_pkg),
        &api_root,
        &["mod dead", dead_api_fn.as_str()],
    );
    assert!(
        api_live.contains(&format!("{}::{}", expect.model_pkg, expect.model_fn)),
        "{api_live}"
    );
    assert_absent(
        &format!("{}/src/live.rs", expect.api_pkg),
        &api_live,
        &["dead_live"],
    );
    assert!(!output
        .join(format!("{}/src/dead.rs", expect.api_pkg))
        .exists());

    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(model_root.contains(expect.model_fn), "{model_root}");
    assert_absent(
        &format!("{}/src/lib.rs", expect.model_pkg),
        &model_root,
        expect.model_absent,
    );
    for token in expect.model_required {
        assert!(
            model_live.contains(token),
            "{} should contain {token:?}\n{model_live}",
            expect.model_pkg
        );
    }
    assert_absent(
        &format!("{}/src/live.rs", expect.model_pkg),
        &model_live,
        expect.model_absent,
    );
    assert!(!output
        .join(format!("{}/src/dead.rs", expect.model_pkg))
        .exists());

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

fn assert_absent(label: &str, source: &str, tokens: &[&str]) {
    for token in tokens {
        assert!(
            !source.contains(token),
            "{label} should not contain {token:?}\n{source}"
        );
    }
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
