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
