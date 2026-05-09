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

#[test]
fn prunes_iterator_take_while_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_take_while_prune",
        root_fn: "selected_take_while_report",
        api_pkg: "take_api",
        model_pkg: "take_model",
        api_fn: "selected_take_while_report",
        model_fn: "selected_take_while",
        model_required: &[
            "TakeItem",
            ".take_while(|item| item.keep_prefix())",
            "pub fn keep_prefix",
            "pub fn render",
        ],
        model_absent: &[
            "mod dead",
            "DeadTakeItem",
            "dead_take_while",
            "dead_method",
            "dead_live_take_while",
            "dead-take",
        ],
    });
}

#[test]
fn prunes_iterator_skip_while_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_skip_while_prune",
        root_fn: "selected_skip_while_report",
        api_pkg: "skip_api",
        model_pkg: "skip_model",
        api_fn: "selected_skip_while_report",
        model_fn: "selected_skip_while",
        model_required: &[
            "SkipItem",
            ".skip_while(|item| item.skip_prefix())",
            "pub fn skip_prefix",
            "pub fn render",
        ],
        model_absent: &[
            "mod dead",
            "DeadSkipItem",
            "dead_skip_while",
            "dead_method",
            "dead_live_skip_while",
            "dead-skip",
        ],
    });
}

#[test]
fn prunes_iterator_partition_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_partition_prune",
        root_fn: "selected_partition_report",
        api_pkg: "partition_api",
        model_pkg: "partition_model",
        api_fn: "selected_partition_report",
        model_fn: "selected_partition",
        model_required: &[
            "PartitionItem",
            ".partition(|item| item.is_selected())",
            "pub fn is_selected",
        ],
        model_absent: &[
            "mod dead",
            "DeadPartitionItem",
            "dead_partition",
            "dead_method",
            "dead_live_partition",
            "dead-partition",
        ],
    });
}

#[test]
fn prunes_iterator_try_fold_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_try_fold_prune",
        root_fn: "selected_try_fold_report",
        api_pkg: "try_fold_api",
        model_pkg: "try_fold_model",
        api_fn: "selected_try_fold_report",
        model_fn: "selected_try_fold",
        model_required: &[
            "TryFoldStep",
            "TryFoldState",
            "TryFoldError",
            ".try_fold(TryFoldState::new(), |state, step| step.append_to(state))",
            "pub fn append_to",
            "pub fn render",
        ],
        model_absent: &[
            "mod dead",
            "DeadTryFoldItem",
            "dead_try_fold",
            "dead_method",
            "dead_live_try_fold",
            "dead-try-fold",
        ],
    });
}

#[test]
fn prunes_iterator_cloned_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_cloned_prune",
        root_fn: "selected_cloned_report",
        api_pkg: "cloned_api",
        model_pkg: "cloned_model",
        api_fn: "selected_cloned_report",
        model_fn: "selected_cloned",
        model_required: &[
            "ClonedItem",
            ".cloned()",
            ".map(|item| item.render())",
            "pub fn render",
        ],
        model_absent: &[
            "mod dead",
            "DeadClonedItem",
            "dead_cloned",
            "dead_method",
            "dead_live_cloned",
            "dead-cloned",
        ],
    });
}

#[test]
fn prunes_iterator_chain_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_chain_prune",
        root_fn: "selected_chain_report",
        api_pkg: "chain_api",
        model_pkg: "chain_model",
        api_fn: "selected_chain_report",
        model_fn: "selected_chain",
        model_required: &[
            "ChainItem",
            ".chain(fallback.iter())",
            ".map(|item| item.render())",
            "pub fn render",
        ],
        model_absent: &[
            "mod dead",
            "DeadChainItem",
            "dead_chain",
            "dead_method",
            "dead_live_chain",
            "dead-chain",
        ],
    });
}

#[test]
fn prunes_iterator_enumerate_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_enumerate_prune",
        root_fn: "selected_enumerate_report",
        api_pkg: "enumerate_api",
        model_pkg: "enumerate_model",
        api_fn: "selected_enumerate_report",
        model_fn: "selected_enumerate",
        model_required: &[
            "EnumerateItem",
            ".enumerate()",
            ".map(|(index, item)| item.render_at(index))",
            "pub fn render_at",
        ],
        model_absent: &[
            "mod dead",
            "DeadEnumerateItem",
            "dead_enumerate",
            "dead_method",
            "dead_live_enumerate",
            "dead-enumerate",
        ],
    });
}

#[test]
fn prunes_iterator_zip_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_zip_prune",
        root_fn: "selected_zip_report",
        api_pkg: "zip_api",
        model_pkg: "zip_model",
        api_fn: "selected_zip_report",
        model_fn: "selected_zip",
        model_required: &[
            "ZipLeft",
            "ZipRight",
            ".zip(right.iter())",
            ".map(|(left, right)| left.render_with(right))",
            "pub fn render_with",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadZipItem",
            "dead_zip",
            "dead_method",
            "dead_live_zip",
            "dead-zip",
        ],
    });
}

#[test]
fn prunes_iterator_reduce_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_reduce_prune",
        root_fn: "selected_reduce_report",
        api_pkg: "reduce_api",
        model_pkg: "reduce_model",
        api_fn: "selected_reduce_report",
        model_fn: "selected_reduce",
        model_required: &[
            "ReduceItem",
            ".reduce(|left, right| left.merge(right))",
            "pub fn merge",
            "pub fn render",
        ],
        model_absent: &[
            "mod dead",
            "DeadReduceItem",
            "dead_reduce",
            "dead_method",
            "dead_live_reduce",
            "dead-reduce",
        ],
    });
}

#[test]
fn prunes_iterator_sort_by_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_sort_by_prune",
        root_fn: "selected_sort_by_report",
        api_pkg: "sort_api",
        model_pkg: "sort_model",
        api_fn: "selected_sort_by_report",
        model_fn: "selected_sort_by",
        model_required: &[
            "SortItem",
            "items.sort_by(|left, right| left.compare_rank(right))",
            "pub fn compare_rank",
            "pub fn render",
        ],
        model_absent: &[
            "mod dead",
            "DeadSortItem",
            "dead_sort_by",
            "dead_method",
            "dead_live_sort_by",
            "dead-sort",
        ],
    });
}

#[test]
fn prunes_iterator_dedup_by_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_dedup_by_prune",
        root_fn: "selected_dedup_by_report",
        api_pkg: "dedup_api",
        model_pkg: "dedup_model",
        api_fn: "selected_dedup_by_report",
        model_fn: "selected_dedup_by",
        model_required: &[
            "DedupItem",
            "items.dedup_by(|left, right| left.same_group(right))",
            "pub fn same_group",
            "pub fn render",
        ],
        model_absent: &[
            "mod dead",
            "DeadDedupItem",
            "dead_dedup_by",
            "dead_method",
            "dead_live_dedup_by",
            "dead-dedup",
        ],
    });
}

#[test]
fn prunes_iterator_scan_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_scan_prune",
        root_fn: "selected_scan_report",
        api_pkg: "scan_api",
        model_pkg: "scan_model",
        api_fn: "selected_scan_report",
        model_fn: "selected_scan",
        model_required: &[
            "ScanItem",
            "ScanState",
            ".scan(ScanState::new(), |state, item| state.accept(item))",
            "pub fn accept",
            "pub fn render",
        ],
        model_absent: &[
            "mod dead",
            "DeadScanItem",
            "dead_scan",
            "dead_method",
            "dead_live_scan",
            "dead-scan",
        ],
    });
}

#[test]
fn prunes_iterator_tuple_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_tuple_map_prune",
        root_fn: "selected_tuple_map_report",
        api_pkg: "tuple_map_api",
        model_pkg: "tuple_map_model",
        api_fn: "selected_tuple_map_report",
        model_fn: "selected_tuple_map",
        model_required: &[
            "TupleMapKey",
            "TupleMapValue",
            ".map(|(key, value)| key.render_with(value))",
            "pub fn render_with",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadTupleMapItem",
            "dead_tuple_map",
            "dead_method",
            "dead_live_tuple_map",
            "dead-tuple-map",
        ],
    });
}

#[test]
fn prunes_iterator_tuple_filter_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_tuple_filter_prune",
        root_fn: "selected_tuple_filter_report",
        api_pkg: "tuple_filter_api",
        model_pkg: "tuple_filter_model",
        api_fn: "selected_tuple_filter_report",
        model_fn: "selected_tuple_filter",
        model_required: &[
            "TupleFilterKey",
            "TupleFilterValue",
            ".filter(|(key, value)| key.accepts(value))",
            ".map(|(_, value)| value.render_label())",
            "pub fn accepts",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadTupleFilterItem",
            "dead_tuple_filter",
            "dead_method",
            "dead_live_tuple_filter",
            "dead-tuple-filter",
        ],
    });
}

#[test]
fn prunes_iterator_tuple_for_each_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_tuple_for_each_prune",
        root_fn: "selected_tuple_for_each_report",
        api_pkg: "tuple_for_each_api",
        model_pkg: "tuple_for_each_model",
        api_fn: "selected_tuple_for_each_report",
        model_fn: "selected_tuple_for_each",
        model_required: &[
            "TupleForEachKey",
            "TupleForEachValue",
            ".for_each(|(key, value)| rendered.push(key.render_with(value)))",
            "pub fn render_with",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadTupleForEachItem",
            "dead_tuple_for_each",
            "dead_method",
            "dead_live_tuple_for_each",
            "dead-tuple-for-each",
        ],
    });
}

#[test]
fn prunes_iterator_entry_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_entry_map_prune",
        root_fn: "selected_entry_map_report",
        api_pkg: "entry_map_api",
        model_pkg: "entry_map_model",
        api_fn: "selected_entry_map_report",
        model_fn: "selected_entry_map",
        model_required: &[
            "EntryMapKey",
            "EntryMapValue",
            "BTreeMap<EntryMapKey, EntryMapValue>",
            ".map(|(key, value)| key.render_entry(value))",
            "pub fn render_entry",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadEntryMapItem",
            "dead_entry_map",
            "dead_method",
            "dead_live_entry_map",
            "dead-entry-map",
        ],
    });
}

#[test]
fn prunes_iterator_tuple_find_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_tuple_find_map_prune",
        root_fn: "selected_tuple_find_map_report",
        api_pkg: "tuple_find_map_api",
        model_pkg: "tuple_find_map_model",
        api_fn: "selected_tuple_find_map_report",
        model_fn: "selected_tuple_find_map",
        model_required: &[
            "TupleFindMapKey",
            "TupleFindMapValue",
            ".find_map(|(key, value)| key.maybe_render(value))",
            "pub fn maybe_render",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadTupleFindMapItem",
            "dead_tuple_find_map",
            "dead_method",
            "dead_live_tuple_find_map",
            "dead-tuple-find-map",
        ],
    });
}

#[test]
fn prunes_iterator_tuple_partition_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_tuple_partition_prune",
        root_fn: "selected_tuple_partition_report",
        api_pkg: "tuple_partition_api",
        model_pkg: "tuple_partition_model",
        api_fn: "selected_tuple_partition_report",
        model_fn: "selected_tuple_partition",
        model_required: &[
            "TuplePartitionKey",
            "TuplePartitionValue",
            ".partition(|(key, value)| key.keep(value))",
            ".map(|(_, value)| value.render_label())",
            "pub fn keep",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadTuplePartitionItem",
            "dead_tuple_partition",
            "dead_method",
            "dead_live_tuple_partition",
            "dead-tuple-partition",
        ],
    });
}

#[test]
fn prunes_iterator_tuple_inspect_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_tuple_inspect_prune",
        root_fn: "selected_tuple_inspect_report",
        api_pkg: "tuple_inspect_api",
        model_pkg: "tuple_inspect_model",
        api_fn: "selected_tuple_inspect_report",
        model_fn: "selected_tuple_inspect",
        model_required: &[
            "TupleInspectKey",
            "TupleInspectValue",
            ".inspect(|(key, value)| audit.push(key.audit(value)))",
            ".map(|(_, value)| value.render_label())",
            "pub fn audit",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadTupleInspectItem",
            "dead_tuple_inspect",
            "dead_method",
            "dead_live_tuple_inspect",
            "dead-tuple-inspect",
        ],
    });
}

#[test]
fn prunes_iterator_tuple_sort_by_key_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_tuple_sort_by_key_prune",
        root_fn: "selected_tuple_sort_by_key_report",
        api_pkg: "tuple_sort_key_api",
        model_pkg: "tuple_sort_key_model",
        api_fn: "selected_tuple_sort_by_key_report",
        model_fn: "selected_tuple_sort_by_key",
        model_required: &[
            "TupleSortKey",
            "TupleSortValue",
            "entries.sort_by_key(|(key, value)| key.rank(value))",
            ".map(|(key, value)| key.render_with(value))",
            "pub fn rank",
            "pub fn render_with",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadTupleSortKeyItem",
            "dead_tuple_sort_by_key",
            "dead_method",
            "dead_live_tuple_sort_by_key",
            "dead-tuple-sort",
        ],
    });
}

#[test]
fn prunes_iterator_nested_tuple_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_nested_tuple_map_prune",
        root_fn: "selected_nested_tuple_map_report",
        api_pkg: "nested_tuple_api",
        model_pkg: "nested_tuple_model",
        api_fn: "selected_nested_tuple_map_report",
        model_fn: "selected_nested_tuple_map",
        model_required: &[
            "NestedTupleKey",
            "NestedTupleValue",
            "NestedTupleMeta",
            ".map(|((key, value), meta)| key.render_nested(value, meta))",
            "pub fn render_nested",
            "pub fn render_label",
            "pub fn render_tag",
        ],
        model_absent: &[
            "mod dead",
            "DeadNestedTupleItem",
            "dead_nested_tuple_map",
            "dead_method",
            "dead_live_nested_tuple_map",
            "dead-nested-tuple",
        ],
    });
}

#[test]
fn prunes_iterator_struct_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_struct_map_prune",
        root_fn: "selected_struct_map_report",
        api_pkg: "struct_map_api",
        model_pkg: "struct_map_model",
        api_fn: "selected_struct_map_report",
        model_fn: "selected_struct_map",
        model_required: &[
            "StructMapPayload",
            "StructMapKey",
            "StructMapValue",
            ".map(|StructMapPayload { key, value }| key.render_with(value))",
            "pub fn render_with",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadStructMapItem",
            "dead_struct_map",
            "dead_method",
            "dead_live_struct_map",
            "dead-struct-map",
        ],
    });
}

#[test]
fn prunes_iterator_struct_filter_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_struct_filter_prune",
        root_fn: "selected_struct_filter_report",
        api_pkg: "struct_filter_api",
        model_pkg: "struct_filter_model",
        api_fn: "selected_struct_filter_report",
        model_fn: "selected_struct_filter",
        model_required: &[
            "StructFilterPayload",
            "StructFilterKey",
            "StructFilterValue",
            ".filter(|StructFilterPayload { key, value }| key.accepts(value))",
            ".map(|StructFilterPayload { value, .. }| value.render_label())",
            "pub fn accepts",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadStructFilterItem",
            "dead_struct_filter",
            "dead_method",
            "dead_live_struct_filter",
            "dead-struct-filter",
        ],
    });
}

#[test]
fn prunes_iterator_struct_inspect_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_struct_inspect_prune",
        root_fn: "selected_struct_inspect_report",
        api_pkg: "struct_inspect_api",
        model_pkg: "struct_inspect_model",
        api_fn: "selected_struct_inspect_report",
        model_fn: "selected_struct_inspect",
        model_required: &[
            "StructInspectPayload",
            "StructInspectKey",
            "StructInspectValue",
            "StructInspectPayload { key, value }",
            "audit.push(key.audit(value))",
            ".map(|StructInspectPayload { value, .. }| value.render_label())",
            "pub fn audit",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadStructInspectItem",
            "dead_struct_inspect",
            "dead_method",
            "dead_live_struct_inspect",
            "dead-struct-inspect",
        ],
    });
}

#[test]
fn prunes_iterator_tuple_struct_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_tuple_struct_map_prune",
        root_fn: "selected_tuple_struct_report",
        api_pkg: "tuple_struct_api",
        model_pkg: "tuple_struct_model",
        api_fn: "selected_tuple_struct_report",
        model_fn: "selected_tuple_struct",
        model_required: &[
            "TupleStructPayload",
            "TupleStructKey",
            "TupleStructValue",
            ".map(|TupleStructPayload(key, value)| key.render_with(value))",
            "pub fn render_with",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadTupleStructItem",
            "dead_tuple_struct",
            "dead_method",
            "dead_live_tuple_struct",
            "dead-tuple-struct",
        ],
    });
}

#[test]
fn prunes_iterator_nested_struct_tuple_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_nested_struct_tuple_prune",
        root_fn: "selected_nested_struct_report",
        api_pkg: "nested_struct_api",
        model_pkg: "nested_struct_model",
        api_fn: "selected_nested_struct_report",
        model_fn: "selected_nested_struct",
        model_required: &[
            "NestedStructPayload",
            "NestedStructKey",
            "NestedStructValue",
            "NestedStructMeta",
            "NestedStructPayload { key, value }",
            "key.render_with(value, meta)",
            "pub fn render_with",
            "pub fn render_label",
            "pub fn render_tag",
        ],
        model_absent: &[
            "mod dead",
            "DeadNestedStructItem",
            "dead_nested_struct",
            "dead_method",
            "dead_live_nested_struct",
            "dead-nested-struct",
        ],
    });
}

#[test]
fn prunes_iterator_enum_struct_filter_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_enum_struct_filter_map_prune",
        root_fn: "selected_enum_struct_report",
        api_pkg: "enum_struct_api",
        model_pkg: "enum_struct_model",
        api_fn: "selected_enum_struct_report",
        model_fn: "selected_enum_struct",
        model_required: &[
            "EnumStructEvent",
            "EnumStructKey",
            "EnumStructValue",
            ".filter_map(|event| match event",
            "EnumStructEvent::Live { key, value }",
            "Some(key.render_with(value))",
            "pub fn render_with",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadEnumStructItem",
            "dead_enum_struct",
            "dead_method",
            "dead_live_enum_struct",
            "dead-enum-struct",
        ],
    });
}

#[test]
fn prunes_iterator_enum_tuple_find_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_enum_tuple_find_map_prune",
        root_fn: "selected_enum_tuple_report",
        api_pkg: "enum_tuple_api",
        model_pkg: "enum_tuple_model",
        api_fn: "selected_enum_tuple_report",
        model_fn: "selected_enum_tuple",
        model_required: &[
            "EnumTupleEvent",
            "EnumTupleKey",
            "EnumTupleValue",
            ".find_map(|event| match event",
            "EnumTupleEvent::Live(key, value)",
            "key.maybe_render(value)",
            "pub fn maybe_render",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadEnumTupleItem",
            "dead_enum_tuple",
            "dead_method",
            "dead_live_enum_tuple",
            "dead-enum-tuple",
        ],
    });
}

#[test]
fn prunes_iterator_enum_if_let_for_each_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_enum_if_let_for_each_prune",
        root_fn: "selected_enum_if_let_report",
        api_pkg: "enum_if_let_api",
        model_pkg: "enum_if_let_model",
        api_fn: "selected_enum_if_let_report",
        model_fn: "selected_enum_if_let",
        model_required: &[
            "EnumIfLetEvent",
            "EnumIfLetKey",
            "EnumIfLetValue",
            ".for_each(|event|",
            "if let EnumIfLetEvent::Live { key, value } = event",
            "rendered.push(key.render_with(value))",
            "pub fn render_with",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadEnumIfLetItem",
            "dead_enum_if_let",
            "dead_method",
            "dead_live_enum_if_let",
            "dead-enum-if-let",
        ],
    });
}

#[test]
fn prunes_iterator_enum_match_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_enum_match_map_prune",
        root_fn: "selected_enum_match_report",
        api_pkg: "enum_match_api",
        model_pkg: "enum_match_model",
        api_fn: "selected_enum_match_report",
        model_fn: "selected_enum_match",
        model_required: &[
            "EnumMatchEvent",
            "EnumMatchKey",
            "EnumMatchValue",
            ".map(|event| match event",
            "EnumMatchEvent::Live(key, value)",
            "key.render_with(value)",
            "pub fn render_with",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadEnumMatchItem",
            "dead_enum_match",
            "dead_method",
            "dead_live_enum_match",
            "dead-enum-match",
        ],
    });
}

#[test]
fn prunes_iterator_enum_flat_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_enum_flat_map_prune",
        root_fn: "selected_enum_flat_map_report",
        api_pkg: "enum_flat_map_api",
        model_pkg: "enum_flat_map_model",
        api_fn: "selected_enum_flat_map_report",
        model_fn: "selected_enum_flat_map",
        model_required: &[
            "EnumFlatMapEvent",
            "EnumFlatMapKey",
            "EnumFlatMapValue",
            ".flat_map(|event| match event",
            "EnumFlatMapEvent::Live { key, value }",
            "key.expand_with(value)",
            "pub fn expand_with",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadEnumFlatMapItem",
            "dead_enum_flat_map",
            "dead_method",
            "dead_live_enum_flat_map",
            "dead-enum-flat-map",
        ],
    });
}

#[test]
fn prunes_option_and_then_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "option_and_then_prune",
        root_fn: "selected_option_and_then_report",
        api_pkg: "option_and_api",
        model_pkg: "option_and_model",
        api_fn: "selected_option_and_then_report",
        model_fn: "selected_option_and_then",
        model_required: &[
            "OptionAndPayload",
            ".and_then(|payload| payload.expand())",
            "pub fn expand",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadOptionAndItem",
            "dead_option_and_then",
            "dead_method",
            "dead_live_option_and_then",
            "dead-option-and",
        ],
    });
}

#[test]
fn prunes_option_is_some_and_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "option_is_some_and_prune",
        root_fn: "selected_option_check_report",
        api_pkg: "option_check_api",
        model_pkg: "option_check_model",
        api_fn: "selected_option_check_report",
        model_fn: "selected_option_check",
        model_required: &[
            "OptionCheckPayload",
            ".is_some_and(|payload| payload.accepts())",
            "pub fn accepts",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadOptionCheckItem",
            "dead_option_check",
            "dead_method",
            "dead_live_option_check",
            "dead-option-check",
        ],
    });
}

#[test]
fn prunes_option_inspect_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "option_inspect_prune",
        root_fn: "selected_option_inspect_report",
        api_pkg: "option_inspect_api",
        model_pkg: "option_inspect_model",
        api_fn: "selected_option_inspect_report",
        model_fn: "selected_option_inspect",
        model_required: &[
            "OptionInspectPayload",
            ".inspect(|payload| audit.push(payload.audit()))",
            ".map(|payload| payload.render_label())",
            "pub fn audit",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadOptionInspectItem",
            "dead_option_inspect",
            "dead_method",
            "dead_live_option_inspect",
            "dead-option-inspect",
        ],
    });
}

#[test]
fn prunes_result_inspect_err_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "result_inspect_err_prune",
        root_fn: "selected_result_inspect_report",
        api_pkg: "result_inspect_api",
        model_pkg: "result_inspect_model",
        api_fn: "selected_result_inspect_report",
        model_fn: "selected_result_inspect",
        model_required: &[
            "ResultInspectPayload",
            "ResultInspectError",
            ".inspect_err(|err| audit.push(err.audit()))",
            ".map(|payload| payload.render_label())",
            "pub fn audit",
            "pub fn render_error",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadResultInspectItem",
            "dead_result_inspect",
            "dead_method",
            "dead_live_result_inspect",
            "dead-result-inspect",
        ],
    });
}

#[test]
fn prunes_result_or_else_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "result_or_else_prune",
        root_fn: "selected_result_or_report",
        api_pkg: "result_or_api",
        model_pkg: "result_or_model",
        api_fn: "selected_result_or_report",
        model_fn: "selected_result_or",
        model_required: &[
            "ResultOrPayload",
            "ResultOrError",
            ".or_else(|err| err.recover())",
            ".map(|payload| payload.render_label())",
            "pub fn recover",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadResultOrItem",
            "dead_result_or",
            "dead_method",
            "dead_live_result_or",
            "dead-result-or",
        ],
    });
}

#[test]
fn prunes_option_filter_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "option_filter_prune",
        root_fn: "selected_option_filter_report",
        api_pkg: "option_filter_api",
        model_pkg: "option_filter_model",
        api_fn: "selected_option_filter_report",
        model_fn: "selected_option_filter",
        model_required: &[
            "OptionFilterPayload",
            ".filter(|payload| payload.accepts())",
            ".map(|payload| payload.render_label())",
            "pub fn accepts",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadOptionFilterItem",
            "dead_option_filter",
            "dead_method",
            "dead_live_option_filter",
            "dead-option-filter",
        ],
    });
}

#[test]
fn prunes_option_ok_or_else_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "option_ok_or_else_prune",
        root_fn: "selected_option_ok_report",
        api_pkg: "option_ok_api",
        model_pkg: "option_ok_model",
        api_fn: "selected_option_ok_report",
        model_fn: "selected_option_ok",
        model_required: &[
            "OptionOkPayload",
            "OptionOkError",
            ".ok_or_else(|| OptionOkError::new(raw))",
            ".map(|payload| payload.render_label())",
            "pub fn render_error",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadOptionOkItem",
            "dead_option_ok",
            "dead_method",
            "dead_live_option_ok",
            "dead-option-ok",
        ],
    });
}

#[test]
fn prunes_option_as_ref_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "option_as_ref_map_prune",
        root_fn: "selected_option_ref_report",
        api_pkg: "option_ref_api",
        model_pkg: "option_ref_model",
        api_fn: "selected_option_ref_report",
        model_fn: "selected_option_ref",
        model_required: &[
            "OptionRefPayload",
            ".as_ref()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadOptionRefItem",
            "dead_option_ref",
            "dead_method",
            "dead_live_option_ref",
            "dead-option-ref",
        ],
    });
}

#[test]
fn prunes_result_and_then_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "result_and_then_prune",
        root_fn: "selected_result_and_report",
        api_pkg: "result_and_api",
        model_pkg: "result_and_model",
        api_fn: "selected_result_and_report",
        model_fn: "selected_result_and",
        model_required: &[
            "ResultAndPayload",
            "ResultAndError",
            ".and_then(|payload| payload.expand())",
            "pub fn expand",
            "pub fn render_error",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadResultAndItem",
            "dead_result_and",
            "dead_method",
            "dead_live_result_and",
            "dead-result-and",
        ],
    });
}

#[test]
fn prunes_result_inspect_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "result_inspect_prune",
        root_fn: "selected_result_inspect_ok_report",
        api_pkg: "result_inspect_ok_api",
        model_pkg: "result_inspect_ok_model",
        api_fn: "selected_result_inspect_ok_report",
        model_fn: "selected_result_inspect_ok",
        model_required: &[
            "ResultInspectOkPayload",
            "ResultInspectOkError",
            ".inspect(|payload| audit.push(payload.audit()))",
            ".map(|payload| payload.render_label())",
            "pub fn audit",
            "pub fn render_error",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadResultInspectOkItem",
            "dead_result_inspect_ok",
            "dead_method",
            "dead_live_result_inspect_ok",
            "dead-result-inspect-ok",
        ],
    });
}

#[test]
fn prunes_option_if_let_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "option_if_let_prune",
        root_fn: "selected_option_if_report",
        api_pkg: "option_if_api",
        model_pkg: "option_if_model",
        api_fn: "selected_option_if_report",
        model_fn: "selected_option_if",
        model_required: &[
            "OptionIfPayload",
            "if let Some(payload) = option_if_payload(raw)",
            "payload.render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadOptionIfItem",
            "dead_option_if",
            "dead_method",
            "dead_live_option_if",
            "dead-option-if",
        ],
    });
}

#[test]
fn prunes_result_match_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "result_match_prune",
        root_fn: "selected_result_match_report",
        api_pkg: "result_match_api",
        model_pkg: "result_match_model",
        api_fn: "selected_result_match_report",
        model_fn: "selected_result_match",
        model_required: &[
            "ResultMatchPayload",
            "ResultMatchError",
            "match result_match_payload(raw)",
            "Ok(payload) => payload.render_label()",
            "Err(err) => err.render_error()",
            "pub fn render_error",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadResultMatchItem",
            "dead_result_match",
            "dead_method",
            "dead_live_result_match",
            "dead-result-match",
        ],
    });
}

#[test]
fn prunes_while_let_payload_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "while_let_payload_prune",
        root_fn: "selected_while_let_report",
        api_pkg: "while_let_api",
        model_pkg: "while_let_model",
        api_fn: "selected_while_let_report",
        model_fn: "selected_while_let",
        model_required: &[
            "WhileLetPayload",
            "while let Some(payload) = items.next()",
            "rendered.push(payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadWhileLetItem",
            "dead_while_let",
            "dead_method",
            "dead_live_while_let",
            "dead-while-let",
        ],
    });
}

#[test]
fn prunes_for_loop_payload_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "for_loop_payload_prune",
        root_fn: "selected_for_loop_report",
        api_pkg: "for_loop_api",
        model_pkg: "for_loop_model",
        api_fn: "selected_for_loop_report",
        model_fn: "selected_for_loop",
        model_required: &[
            "ForLoopPayload",
            "for payload in for_loop_payloads(raw)",
            "rendered.push(payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadForLoopItem",
            "dead_for_loop",
            "dead_method",
            "dead_live_for_loop",
            "dead-for-loop",
        ],
    });
}

#[test]
fn prunes_matches_guard_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "matches_guard_prune",
        root_fn: "selected_matches_guard_report",
        api_pkg: "matches_guard_api",
        model_pkg: "matches_guard_model",
        api_fn: "selected_matches_guard_report",
        model_fn: "selected_matches_guard",
        model_required: &[
            "MatchesGuardPayload",
            "matches!(payload.as_ref(), Some(candidate) if candidate.accepts())",
            ".map(|candidate| candidate.render_label())",
            "pub fn accepts",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadMatchesGuardItem",
            "dead_matches_guard",
            "dead_method",
            "dead_live_matches_guard",
            "dead-matches-guard",
        ],
    });
}

#[test]
fn prunes_option_let_else_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "option_let_else_prune",
        root_fn: "selected_option_let_report",
        api_pkg: "option_let_api",
        model_pkg: "option_let_model",
        api_fn: "selected_option_let_report",
        model_fn: "selected_option_let",
        model_required: &[
            "OptionLetPayload",
            "let Some(payload) = option_let_payload(raw)",
            "payload.render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadOptionLetItem",
            "dead_option_let",
            "dead_method",
            "dead_live_option_let",
            "dead-option-let",
        ],
    });
}

#[test]
fn prunes_result_let_else_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "result_let_else_prune",
        root_fn: "selected_result_let_report",
        api_pkg: "result_let_api",
        model_pkg: "result_let_model",
        api_fn: "selected_result_let_report",
        model_fn: "selected_result_let",
        model_required: &[
            "ResultLetPayload",
            "ResultLetError",
            "let Ok(payload) = result_let_payload(raw)",
            "ResultLetError::new(raw).render_error()",
            "payload.render_label()",
            "pub fn render_error",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadResultLetItem",
            "dead_result_let",
            "dead_method",
            "dead_live_result_let",
            "dead-result-let",
        ],
    });
}

#[test]
fn prunes_nested_option_result_match_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "nested_option_result_match_prune",
        root_fn: "selected_nested_match_report",
        api_pkg: "nested_match_api",
        model_pkg: "nested_match_model",
        api_fn: "selected_nested_match_report",
        model_fn: "selected_nested_match",
        model_required: &[
            "NestedMatchPayload",
            "NestedMatchError",
            "Option<Result<NestedMatchPayload, NestedMatchError>>",
            "Some(Ok(payload)) => payload.render_label()",
            "Some(Err(err)) => err.render_error()",
            "pub fn render_error",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadNestedMatchItem",
            "dead_nested_match",
            "dead_method",
            "dead_live_nested_match",
            "dead-nested-match",
        ],
    });
}

#[test]
fn prunes_nested_option_result_if_let_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "nested_option_result_if_let_prune",
        root_fn: "selected_nested_if_report",
        api_pkg: "nested_if_api",
        model_pkg: "nested_if_model",
        api_fn: "selected_nested_if_report",
        model_fn: "selected_nested_if",
        model_required: &[
            "NestedIfPayload",
            "NestedIfError",
            "Option<Result<NestedIfPayload, NestedIfError>>",
            "if let Some(Ok(payload)) = nested_if_payload(raw)",
            "payload.render_label()",
            "err.render_error()",
            "pub fn render_error",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadNestedIfItem",
            "dead_nested_if",
            "dead_method",
            "dead_live_nested_if",
            "dead-nested-if",
        ],
    });
}

#[test]
fn prunes_matches_result_guard_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "matches_result_guard_prune",
        root_fn: "selected_matches_result_report",
        api_pkg: "matches_result_api",
        model_pkg: "matches_result_model",
        api_fn: "selected_matches_result_report",
        model_fn: "selected_matches_result",
        model_required: &[
            "MatchesResultPayload",
            "MatchesResultError",
            "matches!(result.as_ref(), Ok(payload) if payload.accepts())",
            ".map(|payload| payload.render_label())",
            "err.render_error()",
            "pub fn accepts",
            "pub fn render_error",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadMatchesResultItem",
            "dead_matches_result",
            "dead_method",
            "dead_live_matches_result",
            "dead-matches-result",
        ],
    });
}

#[test]
fn prunes_result_option_match_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "result_option_match_prune",
        root_fn: "selected_result_option_report",
        api_pkg: "result_option_api",
        model_pkg: "result_option_model",
        api_fn: "selected_result_option_report",
        model_fn: "selected_result_option",
        model_required: &[
            "ResultOptionPayload",
            "ResultOptionError",
            "Result<Option<ResultOptionPayload>, ResultOptionError>",
            "Ok(Some(payload)) => payload.render_label()",
            "Err(err) => err.render_error()",
            "pub fn render_error",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadResultOptionItem",
            "dead_result_option",
            "dead_method",
            "dead_live_result_option",
            "dead-result-option",
        ],
    });
}

#[test]
fn prunes_result_option_if_let_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "result_option_if_let_prune",
        root_fn: "selected_result_option_if_report",
        api_pkg: "result_option_if_api",
        model_pkg: "result_option_if_model",
        api_fn: "selected_result_option_if_report",
        model_fn: "selected_result_option_if",
        model_required: &[
            "ResultOptionIfPayload",
            "ResultOptionIfError",
            "if let Ok(Some(payload)) = result_option_if_payload(raw)",
            "payload.render_label()",
            "err.render_error()",
            "pub fn render_error",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadResultOptionIfItem",
            "dead_result_option_if",
            "dead_method",
            "dead_live_result_option_if",
            "dead-result-option-if",
        ],
    });
}

#[test]
fn prunes_option_struct_pattern_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "option_struct_pattern_prune",
        root_fn: "selected_option_struct_report",
        api_pkg: "option_struct_api",
        model_pkg: "option_struct_model",
        api_fn: "selected_option_struct_report",
        model_fn: "selected_option_struct",
        model_required: &[
            "OptionStructPayload",
            "OptionStructInner",
            "Some(OptionStructPayload { inner })",
            "inner.render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadOptionStructItem",
            "dead_option_struct",
            "dead_method",
            "dead_live_option_struct",
            "dead-option-struct",
        ],
    });
}

#[test]
fn prunes_option_tuple_pattern_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "option_tuple_pattern_prune",
        root_fn: "selected_option_tuple_report",
        api_pkg: "option_tuple_api",
        model_pkg: "option_tuple_model",
        api_fn: "selected_option_tuple_report",
        model_fn: "selected_option_tuple",
        model_required: &[
            "OptionTupleKey",
            "OptionTupleValue",
            "Some((key, value)) => key.render_with(value)",
            "pub fn render_with",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadOptionTupleItem",
            "dead_option_tuple",
            "dead_method",
            "dead_live_option_tuple",
            "dead-option-tuple",
        ],
    });
}

#[test]
fn prunes_option_tuple_struct_pattern_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "option_tuple_struct_pattern_prune",
        root_fn: "selected_option_tuple_struct_report",
        api_pkg: "option_tuple_struct_api",
        model_pkg: "option_tuple_struct_model",
        api_fn: "selected_option_tuple_struct_report",
        model_fn: "selected_option_tuple_struct",
        model_required: &[
            "OptionTupleStructPayload",
            "OptionTupleStructInner",
            "Some(OptionTupleStructPayload(inner))",
            "inner.render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadOptionTupleStructItem",
            "dead_option_tuple_struct",
            "dead_method",
            "dead_live_option_tuple_struct",
            "dead-option-tuple-struct",
        ],
    });
}

#[test]
fn prunes_option_enum_named_pattern_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "option_enum_named_pattern_prune",
        root_fn: "selected_option_enum_named_report",
        api_pkg: "option_enum_named_api",
        model_pkg: "option_enum_named_model",
        api_fn: "selected_option_enum_named_report",
        model_fn: "selected_option_enum_named",
        model_required: &[
            "OptionEnumNamedEvent",
            "OptionEnumNamedPayload",
            "Some(OptionEnumNamedEvent::Live { payload })",
            "payload.render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadOptionEnumNamedItem",
            "dead_option_enum_named",
            "dead_method",
            "dead_live_option_enum_named",
            "dead-option-enum-named",
        ],
    });
}

#[test]
fn prunes_option_map_or_else_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "option_map_or_else_prune",
        root_fn: "selected_option_map_or_report",
        api_pkg: "option_map_or_api",
        model_pkg: "option_map_or_model",
        api_fn: "selected_option_map_or_report",
        model_fn: "selected_option_map_or",
        model_required: &[
            "OptionMapOrPayload",
            "OptionMapOrFallback",
            ".map_or_else(",
            "OptionMapOrFallback::new(raw).render_missing()",
            "|payload| payload.render_label()",
            "pub fn render_missing",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadOptionMapOrItem",
            "dead_option_map_or",
            "dead_method",
            "dead_live_option_map_or",
            "dead-option-map-or",
        ],
    });
}

#[test]
fn prunes_result_map_or_else_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "result_map_or_else_prune",
        root_fn: "selected_result_map_or_report",
        api_pkg: "result_map_or_api",
        model_pkg: "result_map_or_model",
        api_fn: "selected_result_map_or_report",
        model_fn: "selected_result_map_or",
        model_required: &[
            "ResultMapOrPayload",
            "ResultMapOrError",
            ".map_or_else(|err| err.render_error(), |payload| payload.render_label())",
            "pub fn render_error",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadResultMapOrItem",
            "dead_result_map_or",
            "dead_method",
            "dead_live_result_map_or",
            "dead-result-map-or",
        ],
    });
}

#[test]
fn prunes_option_unwrap_or_else_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "option_unwrap_or_else_prune",
        root_fn: "selected_option_unwrap_report",
        api_pkg: "option_unwrap_api",
        model_pkg: "option_unwrap_model",
        api_fn: "selected_option_unwrap_report",
        model_fn: "selected_option_unwrap",
        model_required: &[
            "OptionUnwrapPayload",
            ".unwrap_or_else(|| OptionUnwrapPayload::fallback(raw))",
            ".render_label()",
            "pub fn fallback",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadOptionUnwrapItem",
            "dead_option_unwrap",
            "dead_method",
            "dead_live_option_unwrap",
            "dead-option-unwrap",
        ],
    });
}

#[test]
fn prunes_option_or_else_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "option_or_else_prune",
        root_fn: "selected_option_or_report",
        api_pkg: "option_or_api",
        model_pkg: "option_or_model",
        api_fn: "selected_option_or_report",
        model_fn: "selected_option_or",
        model_required: &[
            "OptionOrPayload",
            ".or_else(|| Some(OptionOrPayload::fallback(raw)))",
            ".map(|payload| payload.render_label())",
            "pub fn fallback",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadOptionOrItem",
            "dead_option_or",
            "dead_method",
            "dead_live_option_or",
            "dead-option-or",
        ],
    });
}

#[test]
fn prunes_result_is_ok_and_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "result_is_ok_and_prune",
        root_fn: "selected_result_ok_check_report",
        api_pkg: "result_ok_check_api",
        model_pkg: "result_ok_check_model",
        api_fn: "selected_result_ok_check_report",
        model_fn: "selected_result_ok_check",
        model_required: &[
            "ResultOkCheckPayload",
            ".is_ok_and(|payload| payload.accepts())",
            "pub fn accepts",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadResultOkCheckItem",
            "dead_result_ok_check",
            "dead_method",
            "dead_live_result_ok_check",
            "dead-result-ok-check",
        ],
    });
}

#[test]
fn prunes_result_is_err_and_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "result_is_err_and_prune",
        root_fn: "selected_result_err_check_report",
        api_pkg: "result_err_check_api",
        model_pkg: "result_err_check_model",
        api_fn: "selected_result_err_check_report",
        model_fn: "selected_result_err_check",
        model_required: &[
            "ResultErrCheckPayload",
            "ResultErrCheckError",
            ".is_err_and(|err| err.is_retryable())",
            "pub fn is_retryable",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadResultErrCheckItem",
            "dead_result_err_check",
            "dead_method",
            "dead_live_result_err_check",
            "dead-result-err-check",
        ],
    });
}

#[test]
fn prunes_option_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "option_map_prune",
        root_fn: "selected_option_map_report",
        api_pkg: "option_map_api",
        model_pkg: "option_map_model",
        api_fn: "selected_option_map_report",
        model_fn: "selected_option_map",
        model_required: &[
            "OptionMapPayload",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadOptionMapItem",
            "dead_option_map",
            "dead_method",
            "dead_live_option_map",
            "dead-option-map",
        ],
    });
}

#[test]
fn prunes_result_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "result_map_prune",
        root_fn: "selected_result_map_report",
        api_pkg: "result_map_api",
        model_pkg: "result_map_model",
        api_fn: "selected_result_map_report",
        model_fn: "selected_result_map",
        model_required: &[
            "ResultMapPayload",
            "ResultMapError",
            ".map(|payload| payload.render_label())",
            "pub fn render_error",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadResultMapItem",
            "dead_result_map",
            "dead_method",
            "dead_live_result_map",
            "dead-result-map",
        ],
    });
}

#[test]
fn prunes_option_map_or_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "option_map_or_prune",
        root_fn: "selected_option_map_or_value_report",
        api_pkg: "option_map_or_value_api",
        model_pkg: "option_map_or_value_model",
        api_fn: "selected_option_map_or_value_report",
        model_fn: "selected_option_map_or_value",
        model_required: &[
            "OptionMapOrValuePayload",
            "OptionMapOrValueDefault",
            ".map_or(",
            "OptionMapOrValueDefault::new(raw).render_missing()",
            "|payload| payload.render_label()",
            "pub fn render_missing",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadOptionMapOrValueItem",
            "dead_option_map_or_value",
            "dead_method",
            "dead_live_option_map_or_value",
            "dead-option-map-or-value",
        ],
    });
}

#[test]
fn prunes_result_map_or_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "result_map_or_prune",
        root_fn: "selected_result_map_or_value_report",
        api_pkg: "result_map_or_value_api",
        model_pkg: "result_map_or_value_model",
        api_fn: "selected_result_map_or_value_report",
        model_fn: "selected_result_map_or_value",
        model_required: &[
            "ResultMapOrValuePayload",
            "ResultMapOrValueError",
            "ResultMapOrValueDefault",
            ".map_or(",
            "ResultMapOrValueDefault::new(raw).render_missing()",
            "|payload| payload.render_label()",
            "pub fn render_missing",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadResultMapOrValueItem",
            "dead_result_map_or_value",
            "dead_method",
            "dead_live_result_map_or_value",
            "dead-result-map-or-value",
        ],
    });
}

#[test]
fn prunes_result_ok_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "result_ok_map_prune",
        root_fn: "selected_result_ok_map_report",
        api_pkg: "result_ok_map_api",
        model_pkg: "result_ok_map_model",
        api_fn: "selected_result_ok_map_report",
        model_fn: "selected_result_ok_map",
        model_required: &[
            "ResultOkMapPayload",
            "ResultOkMapError",
            ".ok()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadResultOkMapItem",
            "dead_result_ok_map",
            "dead_method",
            "dead_live_result_ok_map",
            "dead-result-ok-map",
        ],
    });
}

#[test]
fn prunes_result_err_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "result_err_map_prune",
        root_fn: "selected_result_err_map_report",
        api_pkg: "result_err_map_api",
        model_pkg: "result_err_map_model",
        api_fn: "selected_result_err_map_report",
        model_fn: "selected_result_err_map",
        model_required: &[
            "ResultErrMapPayload",
            "ResultErrMapError",
            ".err()",
            ".map(|err| err.render_error())",
            "pub fn render_error",
        ],
        model_absent: &[
            "mod dead",
            "DeadResultErrMapItem",
            "dead_result_err_map",
            "dead_method",
            "dead_live_result_err_map",
            "dead-result-err-map",
        ],
    });
}

#[test]
fn prunes_option_zip_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "option_zip_prune",
        root_fn: "selected_option_zip_report",
        api_pkg: "option_zip_api",
        model_pkg: "option_zip_model",
        api_fn: "selected_option_zip_report",
        model_fn: "selected_option_zip",
        model_required: &[
            "OptionZipLeft",
            "OptionZipRight",
            ".zip(option_zip_right(raw))",
            "left.render_left()",
            "right.render_right()",
            "pub fn render_left",
            "pub fn render_right",
        ],
        model_absent: &[
            "mod dead",
            "DeadOptionZipItem",
            "dead_option_zip",
            "dead_method",
            "dead_live_option_zip",
            "dead-option-zip",
        ],
    });
}

#[test]
fn prunes_iterator_all_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_all_prune",
        root_fn: "selected_all_report",
        api_pkg: "all_api",
        model_pkg: "all_model",
        api_fn: "selected_all_report",
        model_fn: "selected_all",
        model_required: &[
            "AllPayload",
            ".all(|payload| payload.accepts())",
            "pub fn accepts",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadAllItem",
            "dead_all",
            "dead_method",
            "dead_live_all",
            "dead-all",
        ],
    });
}

#[test]
fn prunes_iterator_find_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_find_prune",
        root_fn: "selected_find_report",
        api_pkg: "find_api",
        model_pkg: "find_model",
        api_fn: "selected_find_report",
        model_fn: "selected_find",
        model_required: &[
            "FindPayload",
            ".find(|payload| payload.accepts())",
            ".map(|payload| payload.render_label())",
            "pub fn accepts",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadFindItem",
            "dead_find",
            "dead_method",
            "dead_live_find",
            "dead-find",
        ],
    });
}

#[test]
fn prunes_iterator_max_by_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_max_by_prune",
        root_fn: "selected_max_by_report",
        api_pkg: "max_by_api",
        model_pkg: "max_by_model",
        api_fn: "selected_max_by_report",
        model_fn: "selected_max_by",
        model_required: &[
            "MaxByPayload",
            ".max_by(|left, right| left.rank().cmp(&right.rank()))",
            ".map(|payload| payload.render_label())",
            "pub fn rank",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadMaxByItem",
            "dead_max_by",
            "dead_method",
            "dead_live_max_by",
            "dead-max-by",
        ],
    });
}

#[test]
fn prunes_iterator_min_by_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_min_by_prune",
        root_fn: "selected_min_by_report",
        api_pkg: "min_by_api",
        model_pkg: "min_by_model",
        api_fn: "selected_min_by_report",
        model_fn: "selected_min_by",
        model_required: &[
            "MinByPayload",
            ".min_by(|left, right| left.rank().cmp(&right.rank()))",
            ".map(|payload| payload.render_label())",
            "pub fn rank",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadMinByItem",
            "dead_min_by",
            "dead_method",
            "dead_live_min_by",
            "dead-min-by",
        ],
    });
}

#[test]
fn prunes_iterator_max_by_key_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_max_by_key_prune",
        root_fn: "selected_max_by_key_report",
        api_pkg: "max_by_key_api",
        model_pkg: "max_by_key_model",
        api_fn: "selected_max_by_key_report",
        model_fn: "selected_max_by_key",
        model_required: &[
            "MaxByKeyPayload",
            ".max_by_key(|payload| payload.rank())",
            ".map(|payload| payload.render_label())",
            "pub fn rank",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadMaxByKeyItem",
            "dead_max_by_key",
            "dead_method",
            "dead_live_max_by_key",
            "dead-max-by-key",
        ],
    });
}

#[test]
fn prunes_iterator_min_by_key_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_min_by_key_prune",
        root_fn: "selected_min_by_key_report",
        api_pkg: "min_by_key_api",
        model_pkg: "min_by_key_model",
        api_fn: "selected_min_by_key_report",
        model_fn: "selected_min_by_key",
        model_required: &[
            "MinByKeyPayload",
            ".min_by_key(|payload| payload.rank())",
            ".map(|payload| payload.render_label())",
            "pub fn rank",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadMinByKeyItem",
            "dead_min_by_key",
            "dead_method",
            "dead_live_min_by_key",
            "dead-min-by-key",
        ],
    });
}

#[test]
fn prunes_iterator_rposition_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_rposition_prune",
        root_fn: "selected_rposition_report",
        api_pkg: "rposition_api",
        model_pkg: "rposition_model",
        api_fn: "selected_rposition_report",
        model_fn: "selected_rposition",
        model_required: &[
            "RPositionPayload",
            ".rposition(|payload| payload.accepts())",
            "RPositionPayload::new(&index.to_string()).render_label()",
            "pub fn accepts",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadRPositionItem",
            "dead_rposition",
            "dead_method",
            "dead_live_rposition",
            "dead-rposition",
        ],
    });
}

#[test]
fn prunes_option_xor_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "option_xor_prune",
        root_fn: "selected_option_xor_report",
        api_pkg: "option_xor_api",
        model_pkg: "option_xor_model",
        api_fn: "selected_option_xor_report",
        model_fn: "selected_option_xor",
        model_required: &[
            "OptionXorPayload",
            ".xor(option_xor_right(raw))",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadOptionXorItem",
            "dead_option_xor",
            "dead_method",
            "dead_live_option_xor",
            "dead-option-xor",
        ],
    });
}

#[test]
fn prunes_option_flatten_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "option_flatten_prune",
        root_fn: "selected_option_flatten_report",
        api_pkg: "option_flatten_api",
        model_pkg: "option_flatten_model",
        api_fn: "selected_option_flatten_report",
        model_fn: "selected_option_flatten",
        model_required: &[
            "OptionFlattenPayload",
            ".flatten()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadOptionFlattenItem",
            "dead_option_flatten",
            "dead_method",
            "dead_live_option_flatten",
            "dead-option-flatten",
        ],
    });
}

#[test]
fn prunes_bool_then_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "bool_then_prune",
        root_fn: "selected_bool_then_report",
        api_pkg: "bool_then_api",
        model_pkg: "bool_then_model",
        api_fn: "selected_bool_then_report",
        model_fn: "selected_bool_then",
        model_required: &[
            "BoolThenPayload",
            ".then(|| BoolThenPayload::new(raw))",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBoolThenItem",
            "dead_bool_then",
            "dead_method",
            "dead_live_bool_then",
            "dead-bool-then",
        ],
    });
}

#[test]
fn prunes_bool_then_some_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "bool_then_some_prune",
        root_fn: "selected_bool_then_some_report",
        api_pkg: "bool_then_some_api",
        model_pkg: "bool_then_some_model",
        api_fn: "selected_bool_then_some_report",
        model_fn: "selected_bool_then_some",
        model_required: &[
            "BoolThenSomePayload",
            ".then_some(BoolThenSomePayload::new(raw))",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBoolThenSomeItem",
            "dead_bool_then_some",
            "dead_method",
            "dead_live_bool_then_some",
            "dead-bool-then-some",
        ],
    });
}

#[test]
fn prunes_option_expect_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "option_expect_prune",
        root_fn: "selected_option_expect_report",
        api_pkg: "option_expect_api",
        model_pkg: "option_expect_model",
        api_fn: "selected_option_expect_report",
        model_fn: "selected_option_expect",
        model_required: &[
            "OptionExpectPayload",
            ".expect(\"fixture payload should exist\")",
            ".render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadOptionExpectItem",
            "dead_option_expect",
            "dead_method",
            "dead_live_option_expect",
            "dead-option-expect",
        ],
    });
}

#[test]
fn prunes_result_expect_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "result_expect_prune",
        root_fn: "selected_result_expect_report",
        api_pkg: "result_expect_api",
        model_pkg: "result_expect_model",
        api_fn: "selected_result_expect_report",
        model_fn: "selected_result_expect",
        model_required: &[
            "ResultExpectPayload",
            "ResultExpectError",
            ".expect(\"fixture payload should exist\")",
            ".render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadResultExpectItem",
            "dead_result_expect",
            "dead_method",
            "dead_live_result_expect",
            "dead-result-expect",
        ],
    });
}

#[test]
fn prunes_option_unwrap_direct_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "option_unwrap_prune",
        root_fn: "selected_option_unwrap_direct_report",
        api_pkg: "option_unwrap_direct_api",
        model_pkg: "option_unwrap_direct_model",
        api_fn: "selected_option_unwrap_direct_report",
        model_fn: "selected_option_unwrap_direct",
        model_required: &[
            "OptionUnwrapDirectPayload",
            ".unwrap().render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadOptionUnwrapDirectItem",
            "dead_option_unwrap_direct",
            "dead_method",
            "dead_live_option_unwrap_direct",
            "dead-option-unwrap-direct",
        ],
    });
}

#[test]
fn prunes_option_unwrap_or_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "option_unwrap_or_prune",
        root_fn: "selected_option_unwrap_or_value_report",
        api_pkg: "option_unwrap_or_value_api",
        model_pkg: "option_unwrap_or_value_model",
        api_fn: "selected_option_unwrap_or_value_report",
        model_fn: "selected_option_unwrap_or_value",
        model_required: &[
            "OptionUnwrapOrValuePayload",
            ".unwrap_or(OptionUnwrapOrValuePayload::fallback(raw))",
            ".render_label()",
            "pub fn fallback",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadOptionUnwrapOrValueItem",
            "dead_option_unwrap_or_value",
            "dead_method",
            "dead_live_option_unwrap_or_value",
            "dead-option-unwrap-or-value",
        ],
    });
}

#[test]
fn prunes_result_unwrap_direct_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "result_unwrap_prune",
        root_fn: "selected_result_unwrap_direct_report",
        api_pkg: "result_unwrap_direct_api",
        model_pkg: "result_unwrap_direct_model",
        api_fn: "selected_result_unwrap_direct_report",
        model_fn: "selected_result_unwrap_direct",
        model_required: &[
            "ResultUnwrapDirectPayload",
            "ResultUnwrapDirectError",
            ".unwrap().render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadResultUnwrapDirectItem",
            "dead_result_unwrap_direct",
            "dead_method",
            "dead_live_result_unwrap_direct",
            "dead-result-unwrap-direct",
        ],
    });
}

#[test]
fn prunes_result_unwrap_or_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "result_unwrap_or_prune",
        root_fn: "selected_result_unwrap_or_value_report",
        api_pkg: "result_unwrap_or_value_api",
        model_pkg: "result_unwrap_or_value_model",
        api_fn: "selected_result_unwrap_or_value_report",
        model_fn: "selected_result_unwrap_or_value",
        model_required: &[
            "ResultUnwrapOrValuePayload",
            "ResultUnwrapOrValueError",
            ".unwrap_or(ResultUnwrapOrValuePayload::fallback(raw))",
            ".render_label()",
            "pub fn fallback",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadResultUnwrapOrValueItem",
            "dead_result_unwrap_or_value",
            "dead_method",
            "dead_live_result_unwrap_or_value",
            "dead-result-unwrap-or-value",
        ],
    });
}

#[test]
fn prunes_iterator_last_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_last_prune",
        root_fn: "selected_last_report",
        api_pkg: "last_api",
        model_pkg: "last_model",
        api_fn: "selected_last_report",
        model_fn: "selected_last",
        model_required: &[
            "LastPayload",
            ".last()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadLastItem",
            "dead_last",
            "dead_method",
            "dead_live_last",
            "dead-last",
        ],
    });
}

#[test]
fn prunes_iterator_nth_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_nth_prune",
        root_fn: "selected_nth_report",
        api_pkg: "nth_api",
        model_pkg: "nth_model",
        api_fn: "selected_nth_report",
        model_fn: "selected_nth",
        model_required: &[
            "NthPayload",
            ".nth(1)",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadNthItem",
            "dead_nth",
            "dead_method",
            "dead_live_nth",
            "dead-nth",
        ],
    });
}

#[test]
fn prunes_iterator_rev_last_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_rev_last_prune",
        root_fn: "selected_rev_last_report",
        api_pkg: "rev_last_api",
        model_pkg: "rev_last_model",
        api_fn: "selected_rev_last_report",
        model_fn: "selected_rev_last",
        model_required: &[
            "RevLastPayload",
            ".rev()",
            ".last()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadRevLastItem",
            "dead_rev_last",
            "dead_method",
            "dead_live_rev_last",
            "dead-rev-last",
        ],
    });
}

#[test]
fn prunes_iterator_peekable_nth_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_peekable_nth_prune",
        root_fn: "selected_peekable_nth_report",
        api_pkg: "peekable_nth_api",
        model_pkg: "peekable_nth_model",
        api_fn: "selected_peekable_nth_report",
        model_fn: "selected_peekable_nth",
        model_required: &[
            "PeekableNthPayload",
            ".peekable()",
            ".nth(1)",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadPeekableNthItem",
            "dead_peekable_nth",
            "dead_method",
            "dead_live_peekable_nth",
            "dead-peekable-nth",
        ],
    });
}

#[test]
fn prunes_iterator_fuse_last_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_fuse_last_prune",
        root_fn: "selected_fuse_last_report",
        api_pkg: "fuse_last_api",
        model_pkg: "fuse_last_model",
        api_fn: "selected_fuse_last_report",
        model_fn: "selected_fuse_last",
        model_required: &[
            "FuseLastPayload",
            ".fuse()",
            ".last()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadFuseLastItem",
            "dead_fuse_last",
            "dead_method",
            "dead_live_fuse_last",
            "dead-fuse-last",
        ],
    });
}

#[test]
fn prunes_iterator_cycle_nth_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iterator_cycle_nth_prune",
        root_fn: "selected_cycle_nth_report",
        api_pkg: "cycle_nth_api",
        model_pkg: "cycle_nth_model",
        api_fn: "selected_cycle_nth_report",
        model_fn: "selected_cycle_nth",
        model_required: &[
            "CycleNthPayload",
            ".cycle()",
            ".nth(1)",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadCycleNthItem",
            "dead_cycle_nth",
            "dead_method",
            "dead_live_cycle_nth",
            "dead-cycle-nth",
        ],
    });
}

#[test]
fn prunes_vec_first_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "vec_first_prune",
        root_fn: "selected_vec_first_report",
        api_pkg: "vec_first_api",
        model_pkg: "vec_first_model",
        api_fn: "selected_vec_first_report",
        model_fn: "selected_vec_first",
        model_required: &[
            "VecFirstPayload",
            ".first()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadVecFirstItem",
            "dead_vec_first",
            "dead_method",
            "dead_live_vec_first",
            "dead-vec-first",
        ],
    });
}

#[test]
fn prunes_vec_last_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "vec_last_prune",
        root_fn: "selected_vec_last_report",
        api_pkg: "vec_last_api",
        model_pkg: "vec_last_model",
        api_fn: "selected_vec_last_report",
        model_fn: "selected_vec_last",
        model_required: &[
            "VecLastPayload",
            ".last()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadVecLastItem",
            "dead_vec_last",
            "dead_method",
            "dead_live_vec_last",
            "dead-vec-last",
        ],
    });
}

#[test]
fn prunes_vec_get_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "vec_get_prune",
        root_fn: "selected_vec_get_report",
        api_pkg: "vec_get_api",
        model_pkg: "vec_get_model",
        api_fn: "selected_vec_get_report",
        model_fn: "selected_vec_get",
        model_required: &[
            "VecGetPayload",
            ".get(1)",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadVecGetItem",
            "dead_vec_get",
            "dead_method",
            "dead_live_vec_get",
            "dead-vec-get",
        ],
    });
}

#[test]
fn prunes_vec_pop_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "vec_pop_prune",
        root_fn: "selected_vec_pop_report",
        api_pkg: "vec_pop_api",
        model_pkg: "vec_pop_model",
        api_fn: "selected_vec_pop_report",
        model_fn: "selected_vec_pop",
        model_required: &[
            "VecPopPayload",
            ".pop()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadVecPopItem",
            "dead_vec_pop",
            "dead_method",
            "dead_live_vec_pop",
            "dead-vec-pop",
        ],
    });
}

#[test]
fn prunes_vec_remove_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "vec_remove_prune",
        root_fn: "selected_vec_remove_report",
        api_pkg: "vec_remove_api",
        model_pkg: "vec_remove_model",
        api_fn: "selected_vec_remove_report",
        model_fn: "selected_vec_remove",
        model_required: &[
            "VecRemovePayload",
            ".remove(0).render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadVecRemoveItem",
            "dead_vec_remove",
            "dead_method",
            "dead_live_vec_remove",
            "dead-vec-remove",
        ],
    });
}

#[test]
fn prunes_vec_swap_remove_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "vec_swap_remove_prune",
        root_fn: "selected_vec_swap_remove_report",
        api_pkg: "vec_swap_remove_api",
        model_pkg: "vec_swap_remove_model",
        api_fn: "selected_vec_swap_remove_report",
        model_fn: "selected_vec_swap_remove",
        model_required: &[
            "VecSwapRemovePayload",
            ".swap_remove(0).render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadVecSwapRemoveItem",
            "dead_vec_swap_remove",
            "dead_method",
            "dead_live_vec_swap_remove",
            "dead-vec-swap-remove",
        ],
    });
}

#[test]
fn prunes_vecdeque_front_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "vecdeque_front_prune",
        root_fn: "selected_vecdeque_front_report",
        api_pkg: "vecdeque_front_api",
        model_pkg: "vecdeque_front_model",
        api_fn: "selected_vecdeque_front_report",
        model_fn: "selected_vecdeque_front",
        model_required: &[
            "VecdequeFrontPayload",
            ".front()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadVecdequeFrontItem",
            "dead_vecdeque_front",
            "dead_method",
            "dead_live_vecdeque_front",
            "dead-vecdeque-front",
        ],
    });
}

#[test]
fn prunes_vecdeque_back_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "vecdeque_back_prune",
        root_fn: "selected_vecdeque_back_report",
        api_pkg: "vecdeque_back_api",
        model_pkg: "vecdeque_back_model",
        api_fn: "selected_vecdeque_back_report",
        model_fn: "selected_vecdeque_back",
        model_required: &[
            "VecdequeBackPayload",
            ".back()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadVecdequeBackItem",
            "dead_vecdeque_back",
            "dead_method",
            "dead_live_vecdeque_back",
            "dead-vecdeque-back",
        ],
    });
}

#[test]
fn prunes_vecdeque_pop_front_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "vecdeque_pop_front_prune",
        root_fn: "selected_vecdeque_pop_front_report",
        api_pkg: "vecdeque_pop_front_api",
        model_pkg: "vecdeque_pop_front_model",
        api_fn: "selected_vecdeque_pop_front_report",
        model_fn: "selected_vecdeque_pop_front",
        model_required: &[
            "VecdequePopFrontPayload",
            ".pop_front()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadVecdequePopFrontItem",
            "dead_vecdeque_pop_front",
            "dead_method",
            "dead_live_vecdeque_pop_front",
            "dead-vecdeque-pop-front",
        ],
    });
}

#[test]
fn prunes_vecdeque_pop_back_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "vecdeque_pop_back_prune",
        root_fn: "selected_vecdeque_pop_back_report",
        api_pkg: "vecdeque_pop_back_api",
        model_pkg: "vecdeque_pop_back_model",
        api_fn: "selected_vecdeque_pop_back_report",
        model_fn: "selected_vecdeque_pop_back",
        model_required: &[
            "VecdequePopBackPayload",
            ".pop_back()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadVecdequePopBackItem",
            "dead_vecdeque_pop_back",
            "dead_method",
            "dead_live_vecdeque_pop_back",
            "dead-vecdeque-pop-back",
        ],
    });
}

#[test]
fn prunes_hashmap_get_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "hashmap_get_prune",
        root_fn: "selected_hashmap_get_report",
        api_pkg: "hashmap_get_api",
        model_pkg: "hashmap_get_model",
        api_fn: "selected_hashmap_get_report",
        model_fn: "selected_hashmap_get",
        model_required: &[
            "HashmapGetKey",
            "HashmapGetPayload",
            ".get(&HashmapGetKey::live())",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadHashmapGetItem",
            "dead_hashmap_get",
            "dead_method",
            "dead_live_hashmap_get",
            "dead-hashmap-get",
        ],
    });
}

#[test]
fn prunes_hashmap_get_mut_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "hashmap_get_mut_prune",
        root_fn: "selected_hashmap_get_mut_report",
        api_pkg: "hashmap_get_mut_api",
        model_pkg: "hashmap_get_mut_model",
        api_fn: "selected_hashmap_get_mut_report",
        model_fn: "selected_hashmap_get_mut",
        model_required: &[
            "HashmapGetMutKey",
            "HashmapGetMutPayload",
            ".get_mut(&HashmapGetMutKey::live())",
            ".map(|payload| payload.mark_live().render_label())",
            "pub fn mark_live",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadHashmapGetMutItem",
            "dead_hashmap_get_mut",
            "dead_method",
            "dead_live_hashmap_get_mut",
            "dead-hashmap-get-mut",
        ],
    });
}

#[test]
fn prunes_hashmap_remove_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "hashmap_remove_prune",
        root_fn: "selected_hashmap_remove_report",
        api_pkg: "hashmap_remove_api",
        model_pkg: "hashmap_remove_model",
        api_fn: "selected_hashmap_remove_report",
        model_fn: "selected_hashmap_remove",
        model_required: &[
            "HashmapRemoveKey",
            "HashmapRemovePayload",
            ".remove(&HashmapRemoveKey::live())",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadHashmapRemoveItem",
            "dead_hashmap_remove",
            "dead_method",
            "dead_live_hashmap_remove",
            "dead-hashmap-remove",
        ],
    });
}

#[test]
fn prunes_hashmap_values_next_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "hashmap_values_next_prune",
        root_fn: "selected_hashmap_values_next_report",
        api_pkg: "hashmap_values_next_api",
        model_pkg: "hashmap_values_next_model",
        api_fn: "selected_hashmap_values_next_report",
        model_fn: "selected_hashmap_values_next",
        model_required: &[
            "HashmapValuesNextKey",
            "HashmapValuesNextPayload",
            ".values()",
            ".next()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadHashmapValuesNextItem",
            "dead_hashmap_values_next",
            "dead_method",
            "dead_live_hashmap_values_next",
            "dead-hashmap-values-next",
        ],
    });
}

#[test]
fn prunes_hashmap_into_values_next_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "hashmap_into_values_next_prune",
        root_fn: "selected_hashmap_into_values_next_report",
        api_pkg: "hashmap_into_values_next_api",
        model_pkg: "hashmap_into_values_next_model",
        api_fn: "selected_hashmap_into_values_next_report",
        model_fn: "selected_hashmap_into_values_next",
        model_required: &[
            "HashmapIntoValuesNextKey",
            "HashmapIntoValuesNextPayload",
            ".into_values()",
            ".next()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadHashmapIntoValuesNextItem",
            "dead_hashmap_into_values_next",
            "dead_method",
            "dead_live_hashmap_into_values_next",
            "dead-hashmap-into-values-next",
        ],
    });
}

#[test]
fn prunes_btreemap_get_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "btreemap_get_prune",
        root_fn: "selected_btreemap_get_report",
        api_pkg: "btreemap_get_api",
        model_pkg: "btreemap_get_model",
        api_fn: "selected_btreemap_get_report",
        model_fn: "selected_btreemap_get",
        model_required: &[
            "BtreemapGetKey",
            "BtreemapGetPayload",
            ".get(&BtreemapGetKey::live())",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBtreemapGetItem",
            "dead_btreemap_get",
            "dead_method",
            "dead_live_btreemap_get",
            "dead-btreemap-get",
        ],
    });
}

#[test]
fn prunes_btreemap_get_mut_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "btreemap_get_mut_prune",
        root_fn: "selected_btreemap_get_mut_report",
        api_pkg: "btreemap_get_mut_api",
        model_pkg: "btreemap_get_mut_model",
        api_fn: "selected_btreemap_get_mut_report",
        model_fn: "selected_btreemap_get_mut",
        model_required: &[
            "BtreemapGetMutKey",
            "BtreemapGetMutPayload",
            ".get_mut(&BtreemapGetMutKey::live())",
            ".map(|payload| payload.mark_live().render_label())",
            "pub fn mark_live",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBtreemapGetMutItem",
            "dead_btreemap_get_mut",
            "dead_method",
            "dead_live_btreemap_get_mut",
            "dead-btreemap-get-mut",
        ],
    });
}

#[test]
fn prunes_btreemap_remove_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "btreemap_remove_prune",
        root_fn: "selected_btreemap_remove_report",
        api_pkg: "btreemap_remove_api",
        model_pkg: "btreemap_remove_model",
        api_fn: "selected_btreemap_remove_report",
        model_fn: "selected_btreemap_remove",
        model_required: &[
            "BtreemapRemoveKey",
            "BtreemapRemovePayload",
            ".remove(&BtreemapRemoveKey::live())",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBtreemapRemoveItem",
            "dead_btreemap_remove",
            "dead_method",
            "dead_live_btreemap_remove",
            "dead-btreemap-remove",
        ],
    });
}

#[test]
fn prunes_btreemap_values_last_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "btreemap_values_last_prune",
        root_fn: "selected_btreemap_values_last_report",
        api_pkg: "btreemap_values_last_api",
        model_pkg: "btreemap_values_last_model",
        api_fn: "selected_btreemap_values_last_report",
        model_fn: "selected_btreemap_values_last",
        model_required: &[
            "BtreemapValuesLastKey",
            "BtreemapValuesLastPayload",
            ".values()",
            ".last()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBtreemapValuesLastItem",
            "dead_btreemap_values_last",
            "dead_method",
            "dead_live_btreemap_values_last",
            "dead-btreemap-values-last",
        ],
    });
}

#[test]
fn prunes_btreemap_into_values_next_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "btreemap_into_values_next_prune",
        root_fn: "selected_btreemap_into_values_next_report",
        api_pkg: "btreemap_into_values_next_api",
        model_pkg: "btreemap_into_values_next_model",
        api_fn: "selected_btreemap_into_values_next_report",
        model_fn: "selected_btreemap_into_values_next",
        model_required: &[
            "BtreemapIntoValuesNextKey",
            "BtreemapIntoValuesNextPayload",
            ".into_values()",
            ".next()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBtreemapIntoValuesNextItem",
            "dead_btreemap_into_values_next",
            "dead_method",
            "dead_live_btreemap_into_values_next",
            "dead-btreemap-into-values-next",
        ],
    });
}

#[test]
fn prunes_hashmap_entry_or_insert_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "hashmap_entry_or_insert_prune",
        root_fn: "selected_hashmap_entry_or_insert_report",
        api_pkg: "hashmap_entry_or_insert_api",
        model_pkg: "hashmap_entry_or_insert_model",
        api_fn: "selected_hashmap_entry_or_insert_report",
        model_fn: "selected_hashmap_entry_or_insert",
        model_required: &[
            "HashmapEntryOrInsertKey",
            "HashmapEntryOrInsertPayload",
            ".entry(HashmapEntryOrInsertKey::live())",
            ".or_insert(HashmapEntryOrInsertPayload::new(raw))",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadHashmapEntryOrInsertItem",
            "dead_hashmap_entry_or_insert",
            "pub fn mark_live",
            "dead_method",
            "dead_live_hashmap_entry_or_insert",
            "dead-hashmap-entry-or-insert",
        ],
    });
}

#[test]
fn prunes_hashmap_entry_or_insert_with_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "hashmap_entry_or_insert_with_prune",
        root_fn: "selected_hashmap_entry_or_insert_with_report",
        api_pkg: "hashmap_entry_or_insert_with_api",
        model_pkg: "hashmap_entry_or_insert_with_model",
        api_fn: "selected_hashmap_entry_or_insert_with_report",
        model_fn: "selected_hashmap_entry_or_insert_with",
        model_required: &[
            "HashmapEntryOrInsertWithKey",
            "HashmapEntryOrInsertWithPayload",
            ".entry(HashmapEntryOrInsertWithKey::live())",
            ".or_insert_with(|| HashmapEntryOrInsertWithPayload::new(raw))",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadHashmapEntryOrInsertWithItem",
            "dead_hashmap_entry_or_insert_with",
            "pub fn mark_live",
            "dead_method",
            "dead_live_hashmap_entry_or_insert_with",
            "dead-hashmap-entry-or-insert-with",
        ],
    });
}

#[test]
fn prunes_hashmap_entry_and_modify_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "hashmap_entry_and_modify_prune",
        root_fn: "selected_hashmap_entry_and_modify_report",
        api_pkg: "hashmap_entry_and_modify_api",
        model_pkg: "hashmap_entry_and_modify_model",
        api_fn: "selected_hashmap_entry_and_modify_report",
        model_fn: "selected_hashmap_entry_and_modify",
        model_required: &[
            "HashmapEntryAndModifyKey",
            "HashmapEntryAndModifyPayload",
            ".entry(HashmapEntryAndModifyKey::live())",
            ".and_modify(|payload| {",
            "payload.mark_live();",
            "pub fn mark_live",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadHashmapEntryAndModifyItem",
            "dead_hashmap_entry_and_modify",
            "dead_method",
            "dead_live_hashmap_entry_and_modify",
            "dead-hashmap-entry-and-modify",
        ],
    });
}

#[test]
fn prunes_hashmap_entry_or_default_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "hashmap_entry_or_default_prune",
        root_fn: "selected_hashmap_entry_or_default_report",
        api_pkg: "hashmap_entry_or_default_api",
        model_pkg: "hashmap_entry_or_default_model",
        api_fn: "selected_hashmap_entry_or_default_report",
        model_fn: "selected_hashmap_entry_or_default",
        model_required: &[
            "HashmapEntryOrDefaultKey",
            "HashmapEntryOrDefaultPayload",
            ".entry(HashmapEntryOrDefaultKey::live())",
            ".or_default()",
            "impl Default for HashmapEntryOrDefaultPayload",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadHashmapEntryOrDefaultItem",
            "dead_hashmap_entry_or_default",
            "pub fn mark_live",
            "dead_method",
            "dead_live_hashmap_entry_or_default",
            "dead-hashmap-entry-or-default",
        ],
    });
}

#[test]
fn prunes_hashmap_entry_or_insert_with_key_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "hashmap_entry_or_insert_with_key_prune",
        root_fn: "selected_hashmap_entry_or_insert_with_key_report",
        api_pkg: "hashmap_entry_or_insert_with_key_api",
        model_pkg: "hashmap_entry_or_insert_with_key_model",
        api_fn: "selected_hashmap_entry_or_insert_with_key_report",
        model_fn: "selected_hashmap_entry_or_insert_with_key",
        model_required: &[
            "HashmapEntryOrInsertWithKeyKey",
            "HashmapEntryOrInsertWithKeyPayload",
            ".entry(HashmapEntryOrInsertWithKeyKey::live())",
            ".or_insert_with_key(|_| HashmapEntryOrInsertWithKeyPayload::new(raw))",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadHashmapEntryOrInsertWithKeyItem",
            "dead_hashmap_entry_or_insert_with_key",
            "pub fn mark_live",
            "dead_method",
            "dead_live_hashmap_entry_or_insert_with_key",
            "dead-hashmap-entry-or-insert-with-key",
        ],
    });
}

#[test]
fn prunes_btreemap_entry_or_insert_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "btreemap_entry_or_insert_prune",
        root_fn: "selected_btreemap_entry_or_insert_report",
        api_pkg: "btreemap_entry_or_insert_api",
        model_pkg: "btreemap_entry_or_insert_model",
        api_fn: "selected_btreemap_entry_or_insert_report",
        model_fn: "selected_btreemap_entry_or_insert",
        model_required: &[
            "BtreemapEntryOrInsertKey",
            "BtreemapEntryOrInsertPayload",
            ".entry(BtreemapEntryOrInsertKey::live())",
            ".or_insert(BtreemapEntryOrInsertPayload::new(raw))",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBtreemapEntryOrInsertItem",
            "dead_btreemap_entry_or_insert",
            "pub fn mark_live",
            "dead_method",
            "dead_live_btreemap_entry_or_insert",
            "dead-btreemap-entry-or-insert",
        ],
    });
}

#[test]
fn prunes_btreemap_entry_or_insert_with_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "btreemap_entry_or_insert_with_prune",
        root_fn: "selected_btreemap_entry_or_insert_with_report",
        api_pkg: "btreemap_entry_or_insert_with_api",
        model_pkg: "btreemap_entry_or_insert_with_model",
        api_fn: "selected_btreemap_entry_or_insert_with_report",
        model_fn: "selected_btreemap_entry_or_insert_with",
        model_required: &[
            "BtreemapEntryOrInsertWithKey",
            "BtreemapEntryOrInsertWithPayload",
            ".entry(BtreemapEntryOrInsertWithKey::live())",
            ".or_insert_with(|| BtreemapEntryOrInsertWithPayload::new(raw))",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBtreemapEntryOrInsertWithItem",
            "dead_btreemap_entry_or_insert_with",
            "pub fn mark_live",
            "dead_method",
            "dead_live_btreemap_entry_or_insert_with",
            "dead-btreemap-entry-or-insert-with",
        ],
    });
}

#[test]
fn prunes_btreemap_entry_and_modify_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "btreemap_entry_and_modify_prune",
        root_fn: "selected_btreemap_entry_and_modify_report",
        api_pkg: "btreemap_entry_and_modify_api",
        model_pkg: "btreemap_entry_and_modify_model",
        api_fn: "selected_btreemap_entry_and_modify_report",
        model_fn: "selected_btreemap_entry_and_modify",
        model_required: &[
            "BtreemapEntryAndModifyKey",
            "BtreemapEntryAndModifyPayload",
            ".entry(BtreemapEntryAndModifyKey::live())",
            ".and_modify(|payload| {",
            "payload.mark_live();",
            "pub fn mark_live",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBtreemapEntryAndModifyItem",
            "dead_btreemap_entry_and_modify",
            "dead_method",
            "dead_live_btreemap_entry_and_modify",
            "dead-btreemap-entry-and-modify",
        ],
    });
}

#[test]
fn prunes_btreemap_entry_or_default_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "btreemap_entry_or_default_prune",
        root_fn: "selected_btreemap_entry_or_default_report",
        api_pkg: "btreemap_entry_or_default_api",
        model_pkg: "btreemap_entry_or_default_model",
        api_fn: "selected_btreemap_entry_or_default_report",
        model_fn: "selected_btreemap_entry_or_default",
        model_required: &[
            "BtreemapEntryOrDefaultKey",
            "BtreemapEntryOrDefaultPayload",
            ".entry(BtreemapEntryOrDefaultKey::live())",
            ".or_default()",
            "impl Default for BtreemapEntryOrDefaultPayload",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBtreemapEntryOrDefaultItem",
            "dead_btreemap_entry_or_default",
            "pub fn mark_live",
            "dead_method",
            "dead_live_btreemap_entry_or_default",
            "dead-btreemap-entry-or-default",
        ],
    });
}

#[test]
fn prunes_btreemap_entry_or_insert_with_key_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "btreemap_entry_or_insert_with_key_prune",
        root_fn: "selected_btreemap_entry_or_insert_with_key_report",
        api_pkg: "btreemap_entry_or_insert_with_key_api",
        model_pkg: "btreemap_entry_or_insert_with_key_model",
        api_fn: "selected_btreemap_entry_or_insert_with_key_report",
        model_fn: "selected_btreemap_entry_or_insert_with_key",
        model_required: &[
            "BtreemapEntryOrInsertWithKeyKey",
            "BtreemapEntryOrInsertWithKeyPayload",
            ".entry(BtreemapEntryOrInsertWithKeyKey::live())",
            ".or_insert_with_key(|_| BtreemapEntryOrInsertWithKeyPayload::new(raw))",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBtreemapEntryOrInsertWithKeyItem",
            "dead_btreemap_entry_or_insert_with_key",
            "pub fn mark_live",
            "dead_method",
            "dead_live_btreemap_entry_or_insert_with_key",
            "dead-btreemap-entry-or-insert-with-key",
        ],
    });
}

#[test]
fn prunes_hashmap_iter_pairs_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "hashmap_iter_pairs_prune",
        root_fn: "selected_hashmap_iter_pairs_report",
        api_pkg: "hashmap_iter_pairs_api",
        model_pkg: "hashmap_iter_pairs_model",
        api_fn: "selected_hashmap_iter_pairs_report",
        model_fn: "selected_hashmap_iter_pairs",
        model_required: &[
            "HashmapIterPairsKey",
            "HashmapIterPairsPayload",
            ".iter()",
            ".map(|(_key, payload)| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadHashmapIterPairsItem",
            "dead_hashmap_iter_pairs",
            "pub fn is_live",
            "pub fn render_key",
            "pub fn mark_live",
            "dead_method",
            "dead_live_hashmap_iter_pairs",
            "dead-hashmap-iter-pairs",
        ],
    });
}

#[test]
fn prunes_hashmap_iter_mut_pairs_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "hashmap_iter_mut_pairs_prune",
        root_fn: "selected_hashmap_iter_mut_pairs_report",
        api_pkg: "hashmap_iter_mut_pairs_api",
        model_pkg: "hashmap_iter_mut_pairs_model",
        api_fn: "selected_hashmap_iter_mut_pairs_report",
        model_fn: "selected_hashmap_iter_mut_pairs",
        model_required: &[
            "HashmapIterMutPairsKey",
            "HashmapIterMutPairsPayload",
            ".iter_mut()",
            ".for_each(|(_key, payload)| {",
            "payload.mark_live();",
            "pub fn mark_live",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadHashmapIterMutPairsItem",
            "dead_hashmap_iter_mut_pairs",
            "pub fn is_live",
            "pub fn render_key",
            "dead_method",
            "dead_live_hashmap_iter_mut_pairs",
            "dead-hashmap-iter-mut-pairs",
        ],
    });
}

#[test]
fn prunes_hashmap_into_iter_pairs_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "hashmap_into_iter_pairs_prune",
        root_fn: "selected_hashmap_into_iter_pairs_report",
        api_pkg: "hashmap_into_iter_pairs_api",
        model_pkg: "hashmap_into_iter_pairs_model",
        api_fn: "selected_hashmap_into_iter_pairs_report",
        model_fn: "selected_hashmap_into_iter_pairs",
        model_required: &[
            "HashmapIntoIterPairsKey",
            "HashmapIntoIterPairsPayload",
            ".into_iter()",
            ".map(|(_key, payload)| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadHashmapIntoIterPairsItem",
            "dead_hashmap_into_iter_pairs",
            "pub fn is_live",
            "pub fn render_key",
            "pub fn mark_live",
            "dead_method",
            "dead_live_hashmap_into_iter_pairs",
            "dead-hashmap-into-iter-pairs",
        ],
    });
}

#[test]
fn prunes_hashmap_drain_pairs_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "hashmap_drain_pairs_prune",
        root_fn: "selected_hashmap_drain_pairs_report",
        api_pkg: "hashmap_drain_pairs_api",
        model_pkg: "hashmap_drain_pairs_model",
        api_fn: "selected_hashmap_drain_pairs_report",
        model_fn: "selected_hashmap_drain_pairs",
        model_required: &[
            "HashmapDrainPairsKey",
            "HashmapDrainPairsPayload",
            ".drain()",
            ".map(|(_key, payload)| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadHashmapDrainPairsItem",
            "dead_hashmap_drain_pairs",
            "pub fn is_live",
            "pub fn render_key",
            "pub fn mark_live",
            "dead_method",
            "dead_live_hashmap_drain_pairs",
            "dead-hashmap-drain-pairs",
        ],
    });
}

#[test]
fn prunes_hashmap_keys_find_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "hashmap_keys_find_prune",
        root_fn: "selected_hashmap_keys_find_report",
        api_pkg: "hashmap_keys_find_api",
        model_pkg: "hashmap_keys_find_model",
        api_fn: "selected_hashmap_keys_find_report",
        model_fn: "selected_hashmap_keys_find",
        model_required: &[
            "HashmapKeysFindKey",
            "HashmapKeysFindPayload",
            ".keys()",
            ".find(|key| key.is_live())",
            ".map(|key| key.render_key())",
            "pub fn is_live",
            "pub fn render_key",
        ],
        model_absent: &[
            "mod dead",
            "DeadHashmapKeysFindItem",
            "dead_hashmap_keys_find",
            "pub fn mark_live",
            "pub fn render_label",
            "dead_method",
            "dead_live_hashmap_keys_find",
            "dead-hashmap-keys-find",
        ],
    });
}

#[test]
fn prunes_btreemap_iter_pairs_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "btreemap_iter_pairs_prune",
        root_fn: "selected_btreemap_iter_pairs_report",
        api_pkg: "btreemap_iter_pairs_api",
        model_pkg: "btreemap_iter_pairs_model",
        api_fn: "selected_btreemap_iter_pairs_report",
        model_fn: "selected_btreemap_iter_pairs",
        model_required: &[
            "BtreemapIterPairsKey",
            "BtreemapIterPairsPayload",
            ".iter()",
            ".map(|(_key, payload)| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBtreemapIterPairsItem",
            "dead_btreemap_iter_pairs",
            "pub fn is_live",
            "pub fn render_key",
            "pub fn mark_live",
            "dead_method",
            "dead_live_btreemap_iter_pairs",
            "dead-btreemap-iter-pairs",
        ],
    });
}

#[test]
fn prunes_btreemap_iter_mut_pairs_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "btreemap_iter_mut_pairs_prune",
        root_fn: "selected_btreemap_iter_mut_pairs_report",
        api_pkg: "btreemap_iter_mut_pairs_api",
        model_pkg: "btreemap_iter_mut_pairs_model",
        api_fn: "selected_btreemap_iter_mut_pairs_report",
        model_fn: "selected_btreemap_iter_mut_pairs",
        model_required: &[
            "BtreemapIterMutPairsKey",
            "BtreemapIterMutPairsPayload",
            ".iter_mut()",
            ".for_each(|(_key, payload)| {",
            "payload.mark_live();",
            "pub fn mark_live",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBtreemapIterMutPairsItem",
            "dead_btreemap_iter_mut_pairs",
            "pub fn is_live",
            "pub fn render_key",
            "dead_method",
            "dead_live_btreemap_iter_mut_pairs",
            "dead-btreemap-iter-mut-pairs",
        ],
    });
}

#[test]
fn prunes_btreemap_into_iter_pairs_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "btreemap_into_iter_pairs_prune",
        root_fn: "selected_btreemap_into_iter_pairs_report",
        api_pkg: "btreemap_into_iter_pairs_api",
        model_pkg: "btreemap_into_iter_pairs_model",
        api_fn: "selected_btreemap_into_iter_pairs_report",
        model_fn: "selected_btreemap_into_iter_pairs",
        model_required: &[
            "BtreemapIntoIterPairsKey",
            "BtreemapIntoIterPairsPayload",
            ".into_iter()",
            ".map(|(_key, payload)| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBtreemapIntoIterPairsItem",
            "dead_btreemap_into_iter_pairs",
            "pub fn is_live",
            "pub fn render_key",
            "pub fn mark_live",
            "dead_method",
            "dead_live_btreemap_into_iter_pairs",
            "dead-btreemap-into-iter-pairs",
        ],
    });
}

#[test]
fn prunes_btreemap_range_pairs_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "btreemap_range_pairs_prune",
        root_fn: "selected_btreemap_range_pairs_report",
        api_pkg: "btreemap_range_pairs_api",
        model_pkg: "btreemap_range_pairs_model",
        api_fn: "selected_btreemap_range_pairs_report",
        model_fn: "selected_btreemap_range_pairs",
        model_required: &[
            "BtreemapRangePairsKey",
            "BtreemapRangePairsPayload",
            ".range(BtreemapRangePairsKey::live()..)",
            ".map(|(_key, payload)| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBtreemapRangePairsItem",
            "dead_btreemap_range_pairs",
            "pub fn is_live",
            "pub fn render_key",
            "pub fn mark_live",
            "dead_method",
            "dead_live_btreemap_range_pairs",
            "dead-btreemap-range-pairs",
        ],
    });
}

#[test]
fn prunes_btreemap_range_mut_pairs_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "btreemap_range_mut_pairs_prune",
        root_fn: "selected_btreemap_range_mut_pairs_report",
        api_pkg: "btreemap_range_mut_pairs_api",
        model_pkg: "btreemap_range_mut_pairs_model",
        api_fn: "selected_btreemap_range_mut_pairs_report",
        model_fn: "selected_btreemap_range_mut_pairs",
        model_required: &[
            "BtreemapRangeMutPairsKey",
            "BtreemapRangeMutPairsPayload",
            ".range_mut(BtreemapRangeMutPairsKey::live()..)",
            "payload.mark_live();",
            "pub fn mark_live",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBtreemapRangeMutPairsItem",
            "dead_btreemap_range_mut_pairs",
            "pub fn is_live",
            "pub fn render_key",
            "dead_method",
            "dead_live_btreemap_range_mut_pairs",
            "dead-btreemap-range-mut-pairs",
        ],
    });
}

#[test]
fn prunes_hashset_get_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "hashset_get_prune",
        root_fn: "selected_hashset_get_report",
        api_pkg: "hashset_get_api",
        model_pkg: "hashset_get_model",
        api_fn: "selected_hashset_get_report",
        model_fn: "selected_hashset_get",
        model_required: &[
            "HashsetGetItem",
            "HashSet<HashsetGetItem>",
            ".get(&HashsetGetItem::live())",
            ".map(|item| item.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadHashsetGetItem",
            "dead_hashset_get",
            "pub fn is_live",
            "dead_method",
            "dead_live_hashset_get",
            "dead-hashset-get",
        ],
    });
}

#[test]
fn prunes_hashset_take_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "hashset_take_prune",
        root_fn: "selected_hashset_take_report",
        api_pkg: "hashset_take_api",
        model_pkg: "hashset_take_model",
        api_fn: "selected_hashset_take_report",
        model_fn: "selected_hashset_take",
        model_required: &[
            "HashsetTakeItem",
            "HashSet<HashsetTakeItem>",
            ".take(&HashsetTakeItem::live())",
            ".map(|item| item.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadHashsetTakeItem",
            "dead_hashset_take",
            "pub fn is_live",
            "dead_method",
            "dead_live_hashset_take",
            "dead-hashset-take",
        ],
    });
}

#[test]
fn prunes_hashset_replace_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "hashset_replace_prune",
        root_fn: "selected_hashset_replace_report",
        api_pkg: "hashset_replace_api",
        model_pkg: "hashset_replace_model",
        api_fn: "selected_hashset_replace_report",
        model_fn: "selected_hashset_replace",
        model_required: &[
            "HashsetReplaceItem",
            "HashSet<HashsetReplaceItem>",
            ".replace(HashsetReplaceItem::live())",
            ".map(|item| item.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadHashsetReplaceItem",
            "dead_hashset_replace",
            "pub fn is_live",
            "dead_method",
            "dead_live_hashset_replace",
            "dead-hashset-replace",
        ],
    });
}

#[test]
fn prunes_hashset_iter_find_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "hashset_iter_find_prune",
        root_fn: "selected_hashset_iter_find_report",
        api_pkg: "hashset_iter_find_api",
        model_pkg: "hashset_iter_find_model",
        api_fn: "selected_hashset_iter_find_report",
        model_fn: "selected_hashset_iter_find",
        model_required: &[
            "HashsetIterFindItem",
            "HashSet<HashsetIterFindItem>",
            ".iter()",
            ".find(|item| item.is_live())",
            ".map(|item| item.render_label())",
            "pub fn is_live",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadHashsetIterFindItem",
            "dead_hashset_iter_find",
            "dead_method",
            "dead_live_hashset_iter_find",
            "dead-hashset-iter-find",
        ],
    });
}

#[test]
fn prunes_hashset_drain_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "hashset_drain_prune",
        root_fn: "selected_hashset_drain_report",
        api_pkg: "hashset_drain_api",
        model_pkg: "hashset_drain_model",
        api_fn: "selected_hashset_drain_report",
        model_fn: "selected_hashset_drain",
        model_required: &[
            "HashsetDrainItem",
            "HashSet<HashsetDrainItem>",
            ".drain()",
            ".map(|item| item.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadHashsetDrainItem",
            "dead_hashset_drain",
            "pub fn is_live",
            "dead_method",
            "dead_live_hashset_drain",
            "dead-hashset-drain",
        ],
    });
}

#[test]
fn prunes_hashset_into_iter_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "hashset_into_iter_prune",
        root_fn: "selected_hashset_into_iter_report",
        api_pkg: "hashset_into_iter_api",
        model_pkg: "hashset_into_iter_model",
        api_fn: "selected_hashset_into_iter_report",
        model_fn: "selected_hashset_into_iter",
        model_required: &[
            "HashsetIntoIterItem",
            "HashSet<HashsetIntoIterItem>",
            ".into_iter()",
            ".map(|item| item.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadHashsetIntoIterItem",
            "dead_hashset_into_iter",
            "pub fn is_live",
            "dead_method",
            "dead_live_hashset_into_iter",
            "dead-hashset-into-iter",
        ],
    });
}

#[test]
fn prunes_btreeset_get_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "btreeset_get_prune",
        root_fn: "selected_btreeset_get_report",
        api_pkg: "btreeset_get_api",
        model_pkg: "btreeset_get_model",
        api_fn: "selected_btreeset_get_report",
        model_fn: "selected_btreeset_get",
        model_required: &[
            "BtreesetGetItem",
            "BTreeSet<BtreesetGetItem>",
            ".get(&BtreesetGetItem::live())",
            ".map(|item| item.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBtreesetGetItem",
            "dead_btreeset_get",
            "pub fn is_live",
            "dead_method",
            "dead_live_btreeset_get",
            "dead-btreeset-get",
        ],
    });
}

#[test]
fn prunes_btreeset_take_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "btreeset_take_prune",
        root_fn: "selected_btreeset_take_report",
        api_pkg: "btreeset_take_api",
        model_pkg: "btreeset_take_model",
        api_fn: "selected_btreeset_take_report",
        model_fn: "selected_btreeset_take",
        model_required: &[
            "BtreesetTakeItem",
            "BTreeSet<BtreesetTakeItem>",
            ".take(&BtreesetTakeItem::live())",
            ".map(|item| item.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBtreesetTakeItem",
            "dead_btreeset_take",
            "pub fn is_live",
            "dead_method",
            "dead_live_btreeset_take",
            "dead-btreeset-take",
        ],
    });
}

#[test]
fn prunes_btreeset_replace_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "btreeset_replace_prune",
        root_fn: "selected_btreeset_replace_report",
        api_pkg: "btreeset_replace_api",
        model_pkg: "btreeset_replace_model",
        api_fn: "selected_btreeset_replace_report",
        model_fn: "selected_btreeset_replace",
        model_required: &[
            "BtreesetReplaceItem",
            "BTreeSet<BtreesetReplaceItem>",
            ".replace(BtreesetReplaceItem::live())",
            ".map(|item| item.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBtreesetReplaceItem",
            "dead_btreeset_replace",
            "pub fn is_live",
            "dead_method",
            "dead_live_btreeset_replace",
            "dead-btreeset-replace",
        ],
    });
}

#[test]
fn prunes_btreeset_range_find_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "btreeset_range_find_prune",
        root_fn: "selected_btreeset_range_find_report",
        api_pkg: "btreeset_range_find_api",
        model_pkg: "btreeset_range_find_model",
        api_fn: "selected_btreeset_range_find_report",
        model_fn: "selected_btreeset_range_find",
        model_required: &[
            "BtreesetRangeFindItem",
            "BTreeSet<BtreesetRangeFindItem>",
            ".range(BtreesetRangeFindItem::live()..)",
            ".find(|item| item.is_live())",
            ".map(|item| item.render_label())",
            "pub fn is_live",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBtreesetRangeFindItem",
            "dead_btreeset_range_find",
            "dead_method",
            "dead_live_btreeset_range_find",
            "dead-btreeset-range-find",
        ],
    });
}

#[test]
fn prunes_btreeset_pop_first_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "btreeset_pop_first_prune",
        root_fn: "selected_btreeset_pop_first_report",
        api_pkg: "btreeset_pop_first_api",
        model_pkg: "btreeset_pop_first_model",
        api_fn: "selected_btreeset_pop_first_report",
        model_fn: "selected_btreeset_pop_first",
        model_required: &[
            "BtreesetPopFirstItem",
            "BTreeSet<BtreesetPopFirstItem>",
            ".pop_first()",
            ".map(|item| item.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBtreesetPopFirstItem",
            "dead_btreeset_pop_first",
            "pub fn is_live",
            "dead_method",
            "dead_live_btreeset_pop_first",
            "dead-btreeset-pop-first",
        ],
    });
}

#[test]
fn prunes_btreeset_pop_last_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "btreeset_pop_last_prune",
        root_fn: "selected_btreeset_pop_last_report",
        api_pkg: "btreeset_pop_last_api",
        model_pkg: "btreeset_pop_last_model",
        api_fn: "selected_btreeset_pop_last_report",
        model_fn: "selected_btreeset_pop_last",
        model_required: &[
            "BtreesetPopLastItem",
            "BTreeSet<BtreesetPopLastItem>",
            ".pop_last()",
            ".map(|item| item.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBtreesetPopLastItem",
            "dead_btreeset_pop_last",
            "pub fn is_live",
            "dead_method",
            "dead_live_btreeset_pop_last",
            "dead-btreeset-pop-last",
        ],
    });
}

#[test]
fn prunes_binaryheap_peek_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "binaryheap_peek_prune",
        root_fn: "selected_binaryheap_peek_report",
        api_pkg: "binaryheap_peek_api",
        model_pkg: "binaryheap_peek_model",
        api_fn: "selected_binaryheap_peek_report",
        model_fn: "selected_binaryheap_peek",
        model_required: &[
            "BinaryheapPeekItem",
            "BinaryHeap<BinaryheapPeekItem>",
            ".peek()",
            ".map(|item| item.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBinaryheapPeekItem",
            "dead_binaryheap_peek",
            "pub fn is_live",
            "dead_method",
            "dead_live_binaryheap_peek",
            "dead-binaryheap-peek",
        ],
    });
}

#[test]
fn prunes_binaryheap_pop_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "binaryheap_pop_prune",
        root_fn: "selected_binaryheap_pop_report",
        api_pkg: "binaryheap_pop_api",
        model_pkg: "binaryheap_pop_model",
        api_fn: "selected_binaryheap_pop_report",
        model_fn: "selected_binaryheap_pop",
        model_required: &[
            "BinaryheapPopItem",
            "BinaryHeap<BinaryheapPopItem>",
            ".pop()",
            ".map(|item| item.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBinaryheapPopItem",
            "dead_binaryheap_pop",
            "pub fn is_live",
            "dead_method",
            "dead_live_binaryheap_pop",
            "dead-binaryheap-pop",
        ],
    });
}

#[test]
fn prunes_binaryheap_iter_find_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "binaryheap_iter_find_prune",
        root_fn: "selected_binaryheap_iter_find_report",
        api_pkg: "binaryheap_iter_find_api",
        model_pkg: "binaryheap_iter_find_model",
        api_fn: "selected_binaryheap_iter_find_report",
        model_fn: "selected_binaryheap_iter_find",
        model_required: &[
            "BinaryheapIterFindItem",
            "BinaryHeap<BinaryheapIterFindItem>",
            ".iter()",
            ".find(|item| item.is_live())",
            ".map(|item| item.render_label())",
            "pub fn is_live",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBinaryheapIterFindItem",
            "dead_binaryheap_iter_find",
            "dead_method",
            "dead_live_binaryheap_iter_find",
            "dead-binaryheap-iter-find",
        ],
    });
}

#[test]
fn prunes_binaryheap_drain_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "binaryheap_drain_prune",
        root_fn: "selected_binaryheap_drain_report",
        api_pkg: "binaryheap_drain_api",
        model_pkg: "binaryheap_drain_model",
        api_fn: "selected_binaryheap_drain_report",
        model_fn: "selected_binaryheap_drain",
        model_required: &[
            "BinaryheapDrainItem",
            "BinaryHeap<BinaryheapDrainItem>",
            ".drain()",
            ".map(|item| item.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBinaryheapDrainItem",
            "dead_binaryheap_drain",
            "pub fn is_live",
            "dead_method",
            "dead_live_binaryheap_drain",
            "dead-binaryheap-drain",
        ],
    });
}

#[test]
fn prunes_binaryheap_into_iter_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "binaryheap_into_iter_prune",
        root_fn: "selected_binaryheap_into_iter_report",
        api_pkg: "binaryheap_into_iter_api",
        model_pkg: "binaryheap_into_iter_model",
        api_fn: "selected_binaryheap_into_iter_report",
        model_fn: "selected_binaryheap_into_iter",
        model_required: &[
            "BinaryheapIntoIterItem",
            "BinaryHeap<BinaryheapIntoIterItem>",
            ".into_iter()",
            ".map(|item| item.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBinaryheapIntoIterItem",
            "dead_binaryheap_into_iter",
            "pub fn is_live",
            "dead_method",
            "dead_live_binaryheap_into_iter",
            "dead-binaryheap-into-iter",
        ],
    });
}

#[test]
fn prunes_binaryheap_into_sorted_vec_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "binaryheap_into_sorted_vec_prune",
        root_fn: "selected_binaryheap_into_sorted_vec_report",
        api_pkg: "binaryheap_into_sorted_vec_api",
        model_pkg: "binaryheap_into_sorted_vec_model",
        api_fn: "selected_binaryheap_into_sorted_vec_report",
        model_fn: "selected_binaryheap_into_sorted_vec",
        model_required: &[
            "BinaryheapIntoSortedVecItem",
            "BinaryHeap<BinaryheapIntoSortedVecItem>",
            ".into_sorted_vec()",
            ".into_iter()",
            ".map(|item| item.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBinaryheapIntoSortedVecItem",
            "dead_binaryheap_into_sorted_vec",
            "pub fn is_live",
            "dead_method",
            "dead_live_binaryheap_into_sorted_vec",
            "dead-binaryheap-into-sorted-vec",
        ],
    });
}

#[test]
fn prunes_linkedlist_front_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "linkedlist_front_prune",
        root_fn: "selected_linkedlist_front_report",
        api_pkg: "linkedlist_front_api",
        model_pkg: "linkedlist_front_model",
        api_fn: "selected_linkedlist_front_report",
        model_fn: "selected_linkedlist_front",
        model_required: &[
            "LinkedlistFrontItem",
            "LinkedList<LinkedlistFrontItem>",
            ".front()",
            ".map(|item| item.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadLinkedlistFrontItem",
            "dead_linkedlist_front",
            "pub fn is_live",
            "dead_method",
            "dead_live_linkedlist_front",
            "dead-linkedlist-front",
        ],
    });
}

#[test]
fn prunes_linkedlist_back_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "linkedlist_back_prune",
        root_fn: "selected_linkedlist_back_report",
        api_pkg: "linkedlist_back_api",
        model_pkg: "linkedlist_back_model",
        api_fn: "selected_linkedlist_back_report",
        model_fn: "selected_linkedlist_back",
        model_required: &[
            "LinkedlistBackItem",
            "LinkedList<LinkedlistBackItem>",
            ".back()",
            ".map(|item| item.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadLinkedlistBackItem",
            "dead_linkedlist_back",
            "pub fn is_live",
            "dead_method",
            "dead_live_linkedlist_back",
            "dead-linkedlist-back",
        ],
    });
}

#[test]
fn prunes_linkedlist_pop_front_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "linkedlist_pop_front_prune",
        root_fn: "selected_linkedlist_pop_front_report",
        api_pkg: "linkedlist_pop_front_api",
        model_pkg: "linkedlist_pop_front_model",
        api_fn: "selected_linkedlist_pop_front_report",
        model_fn: "selected_linkedlist_pop_front",
        model_required: &[
            "LinkedlistPopFrontItem",
            "LinkedList<LinkedlistPopFrontItem>",
            ".pop_front()",
            ".map(|item| item.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadLinkedlistPopFrontItem",
            "dead_linkedlist_pop_front",
            "pub fn is_live",
            "dead_method",
            "dead_live_linkedlist_pop_front",
            "dead-linkedlist-pop-front",
        ],
    });
}

#[test]
fn prunes_linkedlist_pop_back_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "linkedlist_pop_back_prune",
        root_fn: "selected_linkedlist_pop_back_report",
        api_pkg: "linkedlist_pop_back_api",
        model_pkg: "linkedlist_pop_back_model",
        api_fn: "selected_linkedlist_pop_back_report",
        model_fn: "selected_linkedlist_pop_back",
        model_required: &[
            "LinkedlistPopBackItem",
            "LinkedList<LinkedlistPopBackItem>",
            ".pop_back()",
            ".map(|item| item.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadLinkedlistPopBackItem",
            "dead_linkedlist_pop_back",
            "pub fn is_live",
            "dead_method",
            "dead_live_linkedlist_pop_back",
            "dead-linkedlist-pop-back",
        ],
    });
}

#[test]
fn prunes_linkedlist_iter_find_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "linkedlist_iter_find_prune",
        root_fn: "selected_linkedlist_iter_find_report",
        api_pkg: "linkedlist_iter_find_api",
        model_pkg: "linkedlist_iter_find_model",
        api_fn: "selected_linkedlist_iter_find_report",
        model_fn: "selected_linkedlist_iter_find",
        model_required: &[
            "LinkedlistIterFindItem",
            "LinkedList<LinkedlistIterFindItem>",
            ".iter()",
            ".find(|item| item.is_live())",
            ".map(|item| item.render_label())",
            "pub fn is_live",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadLinkedlistIterFindItem",
            "dead_linkedlist_iter_find",
            "dead_method",
            "dead_live_linkedlist_iter_find",
            "dead-linkedlist-iter-find",
        ],
    });
}

#[test]
fn prunes_linkedlist_into_iter_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "linkedlist_into_iter_prune",
        root_fn: "selected_linkedlist_into_iter_report",
        api_pkg: "linkedlist_into_iter_api",
        model_pkg: "linkedlist_into_iter_model",
        api_fn: "selected_linkedlist_into_iter_report",
        model_fn: "selected_linkedlist_into_iter",
        model_required: &[
            "LinkedlistIntoIterItem",
            "LinkedList<LinkedlistIntoIterItem>",
            ".into_iter()",
            ".map(|item| item.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadLinkedlistIntoIterItem",
            "dead_linkedlist_into_iter",
            "pub fn is_live",
            "dead_method",
            "dead_live_linkedlist_into_iter",
            "dead-linkedlist-into-iter",
        ],
    });
}

#[test]
fn prunes_vec_iter_find_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "vec_iter_find_prune",
        root_fn: "selected_vec_iter_find_report",
        api_pkg: "vec_iter_find_api",
        model_pkg: "vec_iter_find_model",
        api_fn: "selected_vec_iter_find_report",
        model_fn: "selected_vec_iter_find",
        model_required: &[
            "VecIterFindItem",
            "Vec<VecIterFindItem>",
            ".iter()",
            ".find(|item| item.is_live())",
            ".map(|item| item.render_label())",
            "pub fn is_live",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadVecIterFindItem",
            "dead_vec_iter_find",
            "dead_method",
            "dead_live_vec_iter_find",
            "dead-vec-iter-find",
        ],
    });
}

#[test]
fn prunes_vec_drain_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "vec_drain_prune",
        root_fn: "selected_vec_drain_report",
        api_pkg: "vec_drain_api",
        model_pkg: "vec_drain_model",
        api_fn: "selected_vec_drain_report",
        model_fn: "selected_vec_drain",
        model_required: &[
            "VecDrainItem",
            "Vec<VecDrainItem>",
            ".drain(..)",
            ".map(|item| item.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadVecDrainItem",
            "dead_vec_drain",
            "pub fn is_live",
            "dead_method",
            "dead_live_vec_drain",
            "dead-vec-drain",
        ],
    });
}

#[test]
fn prunes_vec_into_iter_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "vec_into_iter_prune",
        root_fn: "selected_vec_into_iter_report",
        api_pkg: "vec_into_iter_api",
        model_pkg: "vec_into_iter_model",
        api_fn: "selected_vec_into_iter_report",
        model_fn: "selected_vec_into_iter",
        model_required: &[
            "VecIntoIterItem",
            "Vec<VecIntoIterItem>",
            ".into_iter()",
            ".map(|item| item.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadVecIntoIterItem",
            "dead_vec_into_iter",
            "pub fn is_live",
            "dead_method",
            "dead_live_vec_into_iter",
            "dead-vec-into-iter",
        ],
    });
}

#[test]
fn prunes_vecdeque_iter_find_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "vecdeque_iter_find_prune",
        root_fn: "selected_vecdeque_iter_find_report",
        api_pkg: "vecdeque_iter_find_api",
        model_pkg: "vecdeque_iter_find_model",
        api_fn: "selected_vecdeque_iter_find_report",
        model_fn: "selected_vecdeque_iter_find",
        model_required: &[
            "VecdequeIterFindItem",
            "VecDeque<VecdequeIterFindItem>",
            ".iter()",
            ".find(|item| item.is_live())",
            ".map(|item| item.render_label())",
            "pub fn is_live",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadVecdequeIterFindItem",
            "dead_vecdeque_iter_find",
            "dead_method",
            "dead_live_vecdeque_iter_find",
            "dead-vecdeque-iter-find",
        ],
    });
}

#[test]
fn prunes_vecdeque_drain_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "vecdeque_drain_prune",
        root_fn: "selected_vecdeque_drain_report",
        api_pkg: "vecdeque_drain_api",
        model_pkg: "vecdeque_drain_model",
        api_fn: "selected_vecdeque_drain_report",
        model_fn: "selected_vecdeque_drain",
        model_required: &[
            "VecdequeDrainItem",
            "VecDeque<VecdequeDrainItem>",
            ".drain(..)",
            ".map(|item| item.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadVecdequeDrainItem",
            "dead_vecdeque_drain",
            "pub fn is_live",
            "dead_method",
            "dead_live_vecdeque_drain",
            "dead-vecdeque-drain",
        ],
    });
}

#[test]
fn prunes_vecdeque_into_iter_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "vecdeque_into_iter_prune",
        root_fn: "selected_vecdeque_into_iter_report",
        api_pkg: "vecdeque_into_iter_api",
        model_pkg: "vecdeque_into_iter_model",
        api_fn: "selected_vecdeque_into_iter_report",
        model_fn: "selected_vecdeque_into_iter",
        model_required: &[
            "VecdequeIntoIterItem",
            "VecDeque<VecdequeIntoIterItem>",
            ".into_iter()",
            ".map(|item| item.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadVecdequeIntoIterItem",
            "dead_vecdeque_into_iter",
            "pub fn is_live",
            "dead_method",
            "dead_live_vecdeque_into_iter",
            "dead-vecdeque-into-iter",
        ],
    });
}

#[test]
fn prunes_slice_chunks_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "slice_chunks_prune",
        root_fn: "selected_slice_chunks_report",
        api_pkg: "slice_chunks_api",
        model_pkg: "slice_chunks_model",
        api_fn: "selected_slice_chunks_report",
        model_fn: "selected_slice_chunks",
        model_required: &[
            "SliceChunksItem",
            "Vec<SliceChunksItem>",
            ".chunks(2)",
            ".first()",
            ".map(|item| item.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadSliceChunksItem",
            "dead_slice_chunks",
            "pub fn is_break",
            "pub fn bump",
            "dead_method",
            "dead_live_slice_chunks",
            "dead-slice-chunks",
        ],
    });
}

#[test]
fn prunes_slice_chunks_exact_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "slice_chunks_exact_prune",
        root_fn: "selected_slice_chunks_exact_report",
        api_pkg: "slice_chunks_exact_api",
        model_pkg: "slice_chunks_exact_model",
        api_fn: "selected_slice_chunks_exact_report",
        model_fn: "selected_slice_chunks_exact",
        model_required: &[
            "SliceChunksExactItem",
            "Vec<SliceChunksExactItem>",
            ".chunks_exact(2)",
            ".iter()",
            ".map(|item| item.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadSliceChunksExactItem",
            "dead_slice_chunks_exact",
            "pub fn is_break",
            "pub fn bump",
            "dead_method",
            "dead_live_slice_chunks_exact",
            "dead-slice-chunks-exact",
        ],
    });
}

#[test]
fn prunes_slice_rchunks_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "slice_rchunks_prune",
        root_fn: "selected_slice_rchunks_report",
        api_pkg: "slice_rchunks_api",
        model_pkg: "slice_rchunks_model",
        api_fn: "selected_slice_rchunks_report",
        model_fn: "selected_slice_rchunks",
        model_required: &[
            "SliceRchunksItem",
            "Vec<SliceRchunksItem>",
            ".rchunks(2)",
            ".last()",
            ".map(|item| item.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadSliceRchunksItem",
            "dead_slice_rchunks",
            "pub fn is_break",
            "pub fn bump",
            "dead_method",
            "dead_live_slice_rchunks",
            "dead-slice-rchunks",
        ],
    });
}

#[test]
fn prunes_slice_rchunks_exact_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "slice_rchunks_exact_prune",
        root_fn: "selected_slice_rchunks_exact_report",
        api_pkg: "slice_rchunks_exact_api",
        model_pkg: "slice_rchunks_exact_model",
        api_fn: "selected_slice_rchunks_exact_report",
        model_fn: "selected_slice_rchunks_exact",
        model_required: &[
            "SliceRchunksExactItem",
            "Vec<SliceRchunksExactItem>",
            ".rchunks_exact(2)",
            ".iter()",
            ".map(|item| item.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadSliceRchunksExactItem",
            "dead_slice_rchunks_exact",
            "pub fn is_break",
            "pub fn bump",
            "dead_method",
            "dead_live_slice_rchunks_exact",
            "dead-slice-rchunks-exact",
        ],
    });
}

#[test]
fn prunes_slice_windows_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "slice_windows_prune",
        root_fn: "selected_slice_windows_report",
        api_pkg: "slice_windows_api",
        model_pkg: "slice_windows_model",
        api_fn: "selected_slice_windows_report",
        model_fn: "selected_slice_windows",
        model_required: &[
            "SliceWindowsItem",
            "Vec<SliceWindowsItem>",
            ".windows(2)",
            ".iter()",
            ".map(|item| item.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadSliceWindowsItem",
            "dead_slice_windows",
            "pub fn is_break",
            "pub fn bump",
            "dead_method",
            "dead_live_slice_windows",
            "dead-slice-windows",
        ],
    });
}

#[test]
fn prunes_slice_split_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "slice_split_prune",
        root_fn: "selected_slice_split_report",
        api_pkg: "slice_split_api",
        model_pkg: "slice_split_model",
        api_fn: "selected_slice_split_report",
        model_fn: "selected_slice_split",
        model_required: &[
            "SliceSplitItem",
            "Vec<SliceSplitItem>",
            ".split(|item| item.is_break())",
            ".first()",
            ".map(|item| item.render_label())",
            "pub fn is_break",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadSliceSplitItem",
            "dead_slice_split",
            "pub fn bump",
            "dead_method",
            "dead_live_slice_split",
            "dead-slice-split",
        ],
    });
}

#[test]
fn prunes_slice_split_inclusive_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "slice_split_inclusive_prune",
        root_fn: "selected_slice_split_inclusive_report",
        api_pkg: "slice_split_inclusive_api",
        model_pkg: "slice_split_inclusive_model",
        api_fn: "selected_slice_split_inclusive_report",
        model_fn: "selected_slice_split_inclusive",
        model_required: &[
            "SliceSplitInclusiveItem",
            "Vec<SliceSplitInclusiveItem>",
            ".split_inclusive(|item| item.is_break())",
            ".last()",
            ".map(|item| item.render_label())",
            "pub fn is_break",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadSliceSplitInclusiveItem",
            "dead_slice_split_inclusive",
            "pub fn bump",
            "dead_method",
            "dead_live_slice_split_inclusive",
            "dead-slice-split-inclusive",
        ],
    });
}

#[test]
fn prunes_slice_rsplit_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "slice_rsplit_prune",
        root_fn: "selected_slice_rsplit_report",
        api_pkg: "slice_rsplit_api",
        model_pkg: "slice_rsplit_model",
        api_fn: "selected_slice_rsplit_report",
        model_fn: "selected_slice_rsplit",
        model_required: &[
            "SliceRsplitItem",
            "Vec<SliceRsplitItem>",
            ".rsplit(|item| item.is_break())",
            ".first()",
            ".map(|item| item.render_label())",
            "pub fn is_break",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadSliceRsplitItem",
            "dead_slice_rsplit",
            "pub fn bump",
            "dead_method",
            "dead_live_slice_rsplit",
            "dead-slice-rsplit",
        ],
    });
}

#[test]
fn prunes_slice_splitn_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "slice_splitn_prune",
        root_fn: "selected_slice_splitn_report",
        api_pkg: "slice_splitn_api",
        model_pkg: "slice_splitn_model",
        api_fn: "selected_slice_splitn_report",
        model_fn: "selected_slice_splitn",
        model_required: &[
            "SliceSplitnItem",
            "Vec<SliceSplitnItem>",
            ".splitn(2, |item| item.is_break())",
            ".iter()",
            ".map(|item| item.render_label())",
            "pub fn is_break",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadSliceSplitnItem",
            "dead_slice_splitn",
            "pub fn bump",
            "dead_method",
            "dead_live_slice_splitn",
            "dead-slice-splitn",
        ],
    });
}

#[test]
fn prunes_slice_rsplitn_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "slice_rsplitn_prune",
        root_fn: "selected_slice_rsplitn_report",
        api_pkg: "slice_rsplitn_api",
        model_pkg: "slice_rsplitn_model",
        api_fn: "selected_slice_rsplitn_report",
        model_fn: "selected_slice_rsplitn",
        model_required: &[
            "SliceRsplitnItem",
            "Vec<SliceRsplitnItem>",
            ".rsplitn(2, |item| item.is_break())",
            ".iter()",
            ".map(|item| item.render_label())",
            "pub fn is_break",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadSliceRsplitnItem",
            "dead_slice_rsplitn",
            "pub fn bump",
            "dead_method",
            "dead_live_slice_rsplitn",
            "dead-slice-rsplitn",
        ],
    });
}

#[test]
fn prunes_slice_iter_mut_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "slice_iter_mut_prune",
        root_fn: "selected_slice_iter_mut_report",
        api_pkg: "slice_iter_mut_api",
        model_pkg: "slice_iter_mut_model",
        api_fn: "selected_slice_iter_mut_report",
        model_fn: "selected_slice_iter_mut",
        model_required: &[
            "SliceIterMutItem",
            "Vec<SliceIterMutItem>",
            ".iter_mut()",
            ".map(|item| item.bump().render_label())",
            "pub fn bump",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadSliceIterMutItem",
            "dead_slice_iter_mut",
            "pub fn is_break",
            "dead_method",
            "dead_live_slice_iter_mut",
            "dead-slice-iter-mut",
        ],
    });
}

#[test]
fn prunes_array_iter_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "array_iter_prune",
        root_fn: "selected_array_iter_report",
        api_pkg: "array_iter_api",
        model_pkg: "array_iter_model",
        api_fn: "selected_array_iter_report",
        model_fn: "selected_array_iter",
        model_required: &[
            "ArrayIterItem",
            "[ArrayIterItem; 2]",
            ".iter()",
            ".find(|item| item.is_live())",
            ".map(|item| item.render_label())",
            "pub fn is_live",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadArrayIterItem",
            "dead_array_iter",
            "pub fn is_break",
            "pub fn bump",
            "dead_method",
            "dead_live_array_iter",
            "dead-array-iter",
        ],
    });
}

#[test]
fn prunes_vec_retain_mut_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "vec_retain_mut_prune",
        root_fn: "selected_vec_retain_mut_report",
        api_pkg: "vec_retain_mut_api",
        model_pkg: "vec_retain_mut_model",
        api_fn: "selected_vec_retain_mut_report",
        model_fn: "selected_vec_retain_mut",
        model_required: &[
            "VecRetainMutItem",
            "Vec<VecRetainMutItem>",
            ".retain_mut(|item| item.bump().is_live())",
            ".map(|item| item.render_label())",
            "pub fn bump",
            "pub fn is_live",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadVecRetainMutItem",
            "dead_vec_retain_mut",
            "pub fn sort_key",
            "pub fn compare(&self",
            "pub fn compare_key",
            "pub fn unused_helper",
            "dead_method",
            "dead-vec-retain-mut",
        ],
    });
}

#[test]
fn prunes_vec_dedup_by_key_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "vec_dedup_by_key_prune",
        root_fn: "selected_vec_dedup_by_key_report",
        api_pkg: "vec_dedup_by_key_api",
        model_pkg: "vec_dedup_by_key_model",
        api_fn: "selected_vec_dedup_by_key_report",
        model_fn: "selected_vec_dedup_by_key",
        model_required: &[
            "VecDedupByKeyItem",
            "Vec<VecDedupByKeyItem>",
            ".dedup_by_key(|item| item.sort_key())",
            ".map(|item| item.render_label())",
            "pub fn sort_key",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadVecDedupByKeyItem",
            "dead_vec_dedup_by_key",
            "pub fn bump",
            "pub fn is_live",
            "pub fn compare(&self",
            "pub fn compare_key",
            "pub fn unused_helper",
            "dead_method",
            "dead-vec-dedup-by-key",
        ],
    });
}

#[test]
fn prunes_vec_sort_by_cached_key_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "vec_sort_by_cached_key_prune",
        root_fn: "selected_vec_sort_by_cached_key_report",
        api_pkg: "vec_sort_by_cached_key_api",
        model_pkg: "vec_sort_by_cached_key_model",
        api_fn: "selected_vec_sort_by_cached_key_report",
        model_fn: "selected_vec_sort_by_cached_key",
        model_required: &[
            "VecSortByCachedKeyItem",
            "Vec<VecSortByCachedKeyItem>",
            ".sort_by_cached_key(|item| item.sort_key())",
            ".map(|item| item.render_label())",
            "pub fn sort_key",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadVecSortByCachedKeyItem",
            "dead_vec_sort_by_cached_key",
            "pub fn bump",
            "pub fn is_live",
            "pub fn compare(&self",
            "pub fn compare_key",
            "pub fn unused_helper",
            "dead_method",
            "dead-vec-sort-by-cached-key",
        ],
    });
}

#[test]
fn prunes_vec_sort_unstable_by_key_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "vec_sort_unstable_by_key_prune",
        root_fn: "selected_vec_sort_unstable_by_key_report",
        api_pkg: "vec_sort_unstable_by_key_api",
        model_pkg: "vec_sort_unstable_by_key_model",
        api_fn: "selected_vec_sort_unstable_by_key_report",
        model_fn: "selected_vec_sort_unstable_by_key",
        model_required: &[
            "VecSortUnstableByKeyItem",
            "Vec<VecSortUnstableByKeyItem>",
            ".sort_unstable_by_key(|item| item.sort_key())",
            ".map(|item| item.render_label())",
            "pub fn sort_key",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadVecSortUnstableByKeyItem",
            "dead_vec_sort_unstable_by_key",
            "pub fn bump",
            "pub fn is_live",
            "pub fn compare(&self",
            "pub fn compare_key",
            "pub fn unused_helper",
            "dead_method",
            "dead-vec-sort-unstable-by-key",
        ],
    });
}

#[test]
fn prunes_slice_binary_search_by_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "slice_binary_search_by_prune",
        root_fn: "selected_slice_binary_search_by_report",
        api_pkg: "slice_binary_search_by_api",
        model_pkg: "slice_binary_search_by_model",
        api_fn: "selected_slice_binary_search_by_report",
        model_fn: "selected_slice_binary_search_by",
        model_required: &[
            "SliceBinarySearchByItem",
            "Vec<SliceBinarySearchByItem>",
            ".binary_search_by(|item| item.compare_key(raw))",
            ".map(|item| item.render_label())",
            "pub fn sort_key",
            "pub fn compare_key",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadSliceBinarySearchByItem",
            "dead_slice_binary_search_by",
            "pub fn bump",
            "pub fn is_live",
            "pub fn compare(&self",
            "pub fn unused_helper",
            "dead_method",
            "dead-slice-binary-search-by",
        ],
    });
}

#[test]
fn prunes_slice_binary_search_by_key_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "slice_binary_search_by_key_prune",
        root_fn: "selected_slice_binary_search_by_key_report",
        api_pkg: "slice_binary_search_by_key_api",
        model_pkg: "slice_binary_search_by_key_model",
        api_fn: "selected_slice_binary_search_by_key_report",
        model_fn: "selected_slice_binary_search_by_key",
        model_required: &[
            "SliceBinarySearchByKeyItem",
            "Vec<SliceBinarySearchByKeyItem>",
            ".binary_search_by_key(&needle, |item| item.sort_key())",
            ".map(|item| item.render_label())",
            "pub fn sort_key",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadSliceBinarySearchByKeyItem",
            "dead_slice_binary_search_by_key",
            "pub fn bump",
            "pub fn is_live",
            "pub fn compare(&self",
            "pub fn compare_key",
            "pub fn unused_helper",
            "dead_method",
            "dead-slice-binary-search-by-key",
        ],
    });
}

#[test]
fn prunes_slice_partition_point_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "slice_partition_point_prune",
        root_fn: "selected_slice_partition_point_report",
        api_pkg: "slice_partition_point_api",
        model_pkg: "slice_partition_point_model",
        api_fn: "selected_slice_partition_point_report",
        model_fn: "selected_slice_partition_point",
        model_required: &[
            "SlicePartitionPointItem",
            "Vec<SlicePartitionPointItem>",
            ".partition_point(|item| item.sort_key() <= raw.len())",
            ".map(|item| item.render_label())",
            "pub fn sort_key",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadSlicePartitionPointItem",
            "dead_slice_partition_point",
            "pub fn bump",
            "pub fn is_live",
            "pub fn compare(&self",
            "pub fn compare_key",
            "pub fn unused_helper",
            "dead_method",
            "dead-slice-partition-point",
        ],
    });
}

#[test]
fn prunes_slice_sort_unstable_by_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "slice_sort_unstable_by_prune",
        root_fn: "selected_slice_sort_unstable_by_report",
        api_pkg: "slice_sort_unstable_by_api",
        model_pkg: "slice_sort_unstable_by_model",
        api_fn: "selected_slice_sort_unstable_by_report",
        model_fn: "selected_slice_sort_unstable_by",
        model_required: &[
            "SliceSortUnstableByItem",
            "Vec<SliceSortUnstableByItem>",
            ".sort_unstable_by(|left, right| left.compare(right))",
            ".map(|item| item.render_label())",
            "pub fn compare",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadSliceSortUnstableByItem",
            "dead_slice_sort_unstable_by",
            "pub fn bump",
            "pub fn is_live",
            "pub fn sort_key",
            "pub fn compare_key",
            "pub fn unused_helper",
            "dead_method",
            "dead-slice-sort-unstable-by",
        ],
    });
}

#[test]
fn prunes_slice_select_nth_unstable_by_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "slice_select_nth_unstable_by_prune",
        root_fn: "selected_slice_select_nth_unstable_by_report",
        api_pkg: "slice_select_nth_unstable_by_api",
        model_pkg: "slice_select_nth_unstable_by_model",
        api_fn: "selected_slice_select_nth_unstable_by_report",
        model_fn: "selected_slice_select_nth_unstable_by",
        model_required: &[
            "SliceSelectNthUnstableByItem",
            "Vec<SliceSelectNthUnstableByItem>",
            ".select_nth_unstable_by(1, |left, right| left.compare(right))",
            ".map(|item| item.render_label())",
            "pub fn compare",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadSliceSelectNthUnstableByItem",
            "dead_slice_select_nth_unstable_by",
            "pub fn bump",
            "pub fn is_live",
            "pub fn sort_key",
            "pub fn compare_key",
            "pub fn unused_helper",
            "dead_method",
            "dead-slice-select-nth-unstable-by",
        ],
    });
}

#[test]
fn prunes_slice_select_nth_unstable_by_key_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "slice_select_nth_unstable_by_key_prune",
        root_fn: "selected_slice_select_nth_unstable_by_key_report",
        api_pkg: "slice_select_nth_unstable_by_key_api",
        model_pkg: "slice_select_nth_unstable_by_key_model",
        api_fn: "selected_slice_select_nth_unstable_by_key_report",
        model_fn: "selected_slice_select_nth_unstable_by_key",
        model_required: &[
            "SliceSelectNthUnstableByKeyItem",
            "Vec<SliceSelectNthUnstableByKeyItem>",
            ".select_nth_unstable_by_key(1, |item| item.sort_key())",
            ".map(|item| item.render_label())",
            "pub fn sort_key",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadSliceSelectNthUnstableByKeyItem",
            "dead_slice_select_nth_unstable_by_key",
            "pub fn bump",
            "pub fn is_live",
            "pub fn compare(&self",
            "pub fn compare_key",
            "pub fn unused_helper",
            "dead_method",
            "dead-slice-select-nth-unstable-by-key",
        ],
    });
}

#[test]
fn prunes_hashmap_retain_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "hashmap_retain_prune",
        root_fn: "selected_hashmap_retain_report",
        api_pkg: "hashmap_retain_api",
        model_pkg: "hashmap_retain_model",
        api_fn: "selected_hashmap_retain_report",
        model_fn: "selected_hashmap_retain",
        model_required: &[
            "HashMap<",
            "HashMapRetainKey",
            "HashMapRetainItem",
            "entries.retain",
            "key.keep()",
            "item.is_live()",
            ".values()",
            "pub fn keep",
            "pub fn is_live",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadHashMapRetainItem",
            "dead_hashmap_retain",
            "pub fn sort_key",
            "pub fn bump",
            "pub fn compare(&self",
            "pub fn unused_helper",
            "dead_key_method",
            "dead_method",
            "dead-hashmap-retain",
        ],
    });
}

#[test]
fn prunes_btreemap_retain_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "btreemap_retain_prune",
        root_fn: "selected_btreemap_retain_report",
        api_pkg: "btreemap_retain_api",
        model_pkg: "btreemap_retain_model",
        api_fn: "selected_btreemap_retain_report",
        model_fn: "selected_btreemap_retain",
        model_required: &[
            "BTreeMap<",
            "BTreeMapRetainKey",
            "BTreeMapRetainItem",
            "entries.retain",
            "key.keep()",
            "item.is_live()",
            ".values()",
            "pub fn keep",
            "pub fn is_live",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBTreeMapRetainItem",
            "dead_btreemap_retain",
            "pub fn sort_key",
            "pub fn bump",
            "pub fn compare(&self",
            "pub fn unused_helper",
            "dead_key_method",
            "dead_method",
            "dead-btreemap-retain",
        ],
    });
}

#[test]
fn prunes_hashset_retain_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "hashset_retain_prune",
        root_fn: "selected_hashset_retain_report",
        api_pkg: "hashset_retain_api",
        model_pkg: "hashset_retain_model",
        api_fn: "selected_hashset_retain_report",
        model_fn: "selected_hashset_retain",
        model_required: &[
            "HashSet<",
            "HashSetRetainItem",
            "entries.retain",
            "item.is_live()",
            ".iter()",
            "pub fn is_live",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadHashSetRetainItem",
            "dead_hashset_retain",
            "pub fn sort_key",
            "pub fn bump",
            "pub fn compare(&self",
            "pub fn unused_helper",
            "dead_method",
            "dead-hashset-retain",
        ],
    });
}

#[test]
fn prunes_btreeset_retain_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "btreeset_retain_prune",
        root_fn: "selected_btreeset_retain_report",
        api_pkg: "btreeset_retain_api",
        model_pkg: "btreeset_retain_model",
        api_fn: "selected_btreeset_retain_report",
        model_fn: "selected_btreeset_retain",
        model_required: &[
            "BTreeSet<",
            "BTreeSetRetainItem",
            "entries.retain",
            "item.is_live()",
            ".iter()",
            "pub fn is_live",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBTreeSetRetainItem",
            "dead_btreeset_retain",
            "pub fn sort_key",
            "pub fn bump",
            "pub fn compare(&self",
            "pub fn unused_helper",
            "dead_method",
            "dead-btreeset-retain",
        ],
    });
}

#[test]
fn prunes_binaryheap_retain_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "binaryheap_retain_prune",
        root_fn: "selected_binaryheap_retain_report",
        api_pkg: "binaryheap_retain_api",
        model_pkg: "binaryheap_retain_model",
        api_fn: "selected_binaryheap_retain_report",
        model_fn: "selected_binaryheap_retain",
        model_required: &[
            "BinaryHeap<",
            "BinaryHeapRetainItem",
            "entries.retain",
            "item.is_live()",
            ".iter()",
            "pub fn is_live",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBinaryHeapRetainItem",
            "dead_binaryheap_retain",
            "pub fn sort_key",
            "pub fn bump",
            "pub fn compare(&self",
            "pub fn unused_helper",
            "dead_method",
            "dead-binaryheap-retain",
        ],
    });
}

#[test]
fn prunes_vecdeque_retain_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "vecdeque_retain_prune",
        root_fn: "selected_vecdeque_retain_report",
        api_pkg: "vecdeque_retain_api",
        model_pkg: "vecdeque_retain_model",
        api_fn: "selected_vecdeque_retain_report",
        model_fn: "selected_vecdeque_retain",
        model_required: &[
            "VecDeque<",
            "VecDequeRetainItem",
            "entries.retain",
            "item.is_live()",
            ".iter()",
            "pub fn is_live",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadVecDequeRetainItem",
            "dead_vecdeque_retain",
            "pub fn sort_key",
            "pub fn bump",
            "pub fn compare(&self",
            "pub fn unused_helper",
            "dead_method",
            "dead-vecdeque-retain",
        ],
    });
}

#[test]
fn prunes_vecdeque_make_contiguous_sort_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "vecdeque_make_contiguous_sort_prune",
        root_fn: "selected_vecdeque_make_contiguous_sort_report",
        api_pkg: "vecdeque_make_contiguous_sort_api",
        model_pkg: "vecdeque_make_contiguous_sort_model",
        api_fn: "selected_vecdeque_make_contiguous_sort_report",
        model_fn: "selected_vecdeque_make_contiguous_sort",
        model_required: &[
            "VecDeque<",
            "VecDequeMakeContiguousSortItem",
            ".make_contiguous().sort_by_key",
            "item.sort_key()",
            ".iter()",
            "pub fn sort_key",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadVecDequeMakeContiguousSortItem",
            "dead_vecdeque_make_contiguous_sort",
            "pub fn bump",
            "pub fn compare(&self",
            "pub fn unused_helper",
            "dead_method",
            "dead-vecdeque-make-contiguous-sort",
        ],
    });
}

#[test]
fn prunes_vec_as_slice_iter_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "vec_as_slice_iter_prune",
        root_fn: "selected_vec_as_slice_iter_report",
        api_pkg: "vec_as_slice_iter_api",
        model_pkg: "vec_as_slice_iter_model",
        api_fn: "selected_vec_as_slice_iter_report",
        model_fn: "selected_vec_as_slice_iter",
        model_required: &[
            "Vec<VecAsSliceIterItem>",
            ".as_slice()",
            ".find(|item| item.is_live())",
            "pub fn is_live",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadVecAsSliceIterItem",
            "dead_vec_as_slice_iter",
            "pub fn sort_key",
            "pub fn bump",
            "pub fn compare(&self",
            "pub fn unused_helper",
            "dead_method",
            "dead-vec-as-slice-iter",
        ],
    });
}

#[test]
fn prunes_vec_as_mut_slice_iter_mut_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "vec_as_mut_slice_iter_mut_prune",
        root_fn: "selected_vec_as_mut_slice_iter_mut_report",
        api_pkg: "vec_as_mut_slice_iter_mut_api",
        model_pkg: "vec_as_mut_slice_iter_mut_model",
        api_fn: "selected_vec_as_mut_slice_iter_mut_report",
        model_fn: "selected_vec_as_mut_slice_iter_mut",
        model_required: &[
            "Vec<VecAsMutSliceIterMutItem>",
            ".as_mut_slice()",
            ".iter_mut()",
            "item.bump().render_label()",
            "pub fn bump",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadVecAsMutSliceIterMutItem",
            "dead_vec_as_mut_slice_iter_mut",
            "pub fn sort_key",
            "pub fn compare(&self",
            "pub fn unused_helper",
            "dead_method",
            "dead-vec-as-mut-slice-iter-mut",
        ],
    });
}

#[test]
fn prunes_vec_as_slice_chunks_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "vec_as_slice_chunks_prune",
        root_fn: "selected_vec_as_slice_chunks_report",
        api_pkg: "vec_as_slice_chunks_api",
        model_pkg: "vec_as_slice_chunks_model",
        api_fn: "selected_vec_as_slice_chunks_report",
        model_fn: "selected_vec_as_slice_chunks",
        model_required: &[
            "Vec<VecAsSliceChunksItem>",
            ".as_slice()",
            ".chunks(2)",
            ".map(|item| item.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadVecAsSliceChunksItem",
            "dead_vec_as_slice_chunks",
            "pub fn sort_key",
            "pub fn bump",
            "pub fn compare(&self",
            "pub fn unused_helper",
            "dead_method",
            "dead-vec-as-slice-chunks",
        ],
    });
}

#[test]
fn prunes_collect_hashmap_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "collect_hashmap_prune",
        root_fn: "selected_collect_hashmap_report",
        api_pkg: "collect_hashmap_api",
        model_pkg: "collect_hashmap_model",
        api_fn: "selected_collect_hashmap_report",
        model_fn: "selected_collect_hashmap",
        model_required: &[
            "collect::<HashMap<CollectHashMapKey, CollectHashMapItem>>()",
            "CollectHashMapKey::from_source",
            "CollectHashMapItem::from_source",
            ".values()",
            "pub fn key_seed",
            "pub fn value_seed",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadCollectHashMapItem",
            "dead_collect_hashmap",
            "pub fn is_live_source",
            "pub fn render_key",
            "dead_key_method",
            "pub fn compare(&self",
            "pub fn unused_helper",
            "dead_method",
            "dead-collect-hashmap",
        ],
    });
}

#[test]
fn prunes_collect_btreemap_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "collect_btreemap_prune",
        root_fn: "selected_collect_btreemap_report",
        api_pkg: "collect_btreemap_api",
        model_pkg: "collect_btreemap_model",
        api_fn: "selected_collect_btreemap_report",
        model_fn: "selected_collect_btreemap",
        model_required: &[
            "collect::<BTreeMap<CollectBTreeMapKey, CollectBTreeMapItem>>()",
            "CollectBTreeMapKey::from_source",
            "CollectBTreeMapItem::from_source",
            ".values()",
            "pub fn key_seed",
            "pub fn value_seed",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadCollectBTreeMapItem",
            "dead_collect_btreemap",
            "pub fn is_live_source",
            "pub fn render_key",
            "dead_key_method",
            "pub fn compare(&self",
            "pub fn unused_helper",
            "dead_method",
            "dead-collect-btreemap",
        ],
    });
}

#[test]
fn prunes_collect_hashset_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "collect_hashset_prune",
        root_fn: "selected_collect_hashset_report",
        api_pkg: "collect_hashset_api",
        model_pkg: "collect_hashset_model",
        api_fn: "selected_collect_hashset_report",
        model_fn: "selected_collect_hashset",
        model_required: &[
            "collect::<HashSet<CollectHashSetItem>>()",
            ".filter(|source| source.is_live_source())",
            "CollectHashSetItem::from_source",
            ".iter()",
            "pub fn is_live_source",
            "pub fn value_seed",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadCollectHashSetItem",
            "dead_collect_hashset",
            "pub fn key_seed",
            "pub fn compare(&self",
            "pub fn unused_helper",
            "dead_method",
            "dead-collect-hashset",
        ],
    });
}

#[test]
fn prunes_collect_btreeset_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "collect_btreeset_prune",
        root_fn: "selected_collect_btreeset_report",
        api_pkg: "collect_btreeset_api",
        model_pkg: "collect_btreeset_model",
        api_fn: "selected_collect_btreeset_report",
        model_fn: "selected_collect_btreeset",
        model_required: &[
            "collect::<BTreeSet<CollectBTreeSetItem>>()",
            ".filter(|source| source.is_live_source())",
            "CollectBTreeSetItem::from_source",
            ".iter()",
            "pub fn is_live_source",
            "pub fn value_seed",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadCollectBTreeSetItem",
            "dead_collect_btreeset",
            "pub fn key_seed",
            "pub fn compare(&self",
            "pub fn unused_helper",
            "dead_method",
            "dead-collect-btreeset",
        ],
    });
}

#[test]
fn prunes_collect_binaryheap_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "collect_binaryheap_prune",
        root_fn: "selected_collect_binaryheap_report",
        api_pkg: "collect_binaryheap_api",
        model_pkg: "collect_binaryheap_model",
        api_fn: "selected_collect_binaryheap_report",
        model_fn: "selected_collect_binaryheap",
        model_required: &[
            "collect::<BinaryHeap<CollectBinaryHeapItem>>()",
            ".filter(|source| source.is_live_source())",
            "CollectBinaryHeapItem::from_source",
            ".iter()",
            "pub fn is_live_source",
            "pub fn value_seed",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadCollectBinaryHeapItem",
            "dead_collect_binaryheap",
            "pub fn key_seed",
            "pub fn compare(&self",
            "pub fn unused_helper",
            "dead_method",
            "dead-collect-binaryheap",
        ],
    });
}

#[test]
fn prunes_collect_vecdeque_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "collect_vecdeque_prune",
        root_fn: "selected_collect_vecdeque_report",
        api_pkg: "collect_vecdeque_api",
        model_pkg: "collect_vecdeque_model",
        api_fn: "selected_collect_vecdeque_report",
        model_fn: "selected_collect_vecdeque",
        model_required: &[
            "collect::<VecDeque<CollectVecDequeItem>>()",
            ".filter(|source| source.is_live_source())",
            "CollectVecDequeItem::from_source",
            ".iter()",
            "pub fn is_live_source",
            "pub fn value_seed",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadCollectVecDequeItem",
            "dead_collect_vecdeque",
            "pub fn key_seed",
            "pub fn compare(&self",
            "pub fn unused_helper",
            "dead_method",
            "dead-collect-vecdeque",
        ],
    });
}

#[test]
fn prunes_collect_linkedlist_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "collect_linkedlist_prune",
        root_fn: "selected_collect_linkedlist_report",
        api_pkg: "collect_linkedlist_api",
        model_pkg: "collect_linkedlist_model",
        api_fn: "selected_collect_linkedlist_report",
        model_fn: "selected_collect_linkedlist",
        model_required: &[
            "collect::<LinkedList<CollectLinkedListItem>>()",
            ".filter(|source| source.is_live_source())",
            "CollectLinkedListItem::from_source",
            ".iter()",
            "pub fn is_live_source",
            "pub fn value_seed",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadCollectLinkedListItem",
            "dead_collect_linkedlist",
            "pub fn key_seed",
            "pub fn compare(&self",
            "pub fn unused_helper",
            "dead_method",
            "dead-collect-linkedlist",
        ],
    });
}

#[test]
fn prunes_collect_vec_tuple_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "collect_vec_tuple_prune",
        root_fn: "selected_collect_vec_tuple_report",
        api_pkg: "collect_vec_tuple_api",
        model_pkg: "collect_vec_tuple_model",
        api_fn: "selected_collect_vec_tuple_report",
        model_fn: "selected_collect_vec_tuple",
        model_required: &[
            "collect::<Vec<(CollectVecTupleKey, CollectVecTupleItem)>>()",
            "CollectVecTupleKey::from_source",
            "CollectVecTupleItem::from_source",
            "pub fn key_seed",
            "pub fn value_seed",
            "pub fn render_key",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadCollectVecTupleItem",
            "dead_collect_vec_tuple",
            "pub fn is_live_source",
            "dead_key_method",
            "pub fn compare(&self",
            "pub fn unused_helper",
            "dead_method",
            "dead-collect-vec-tuple",
        ],
    });
}

#[test]
fn prunes_collect_result_vec_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "collect_result_vec_prune",
        root_fn: "selected_collect_result_vec_report",
        api_pkg: "collect_result_vec_api",
        model_pkg: "collect_result_vec_model",
        api_fn: "selected_collect_result_vec_report",
        model_fn: "selected_collect_result_vec",
        model_required: &[
            "collect::<Result<Vec<CollectResultVecItem>, CollectResultVecError>>()",
            "collect_result_vec_convert",
            "Ok(CollectResultVecItem::from_source",
            "Err(CollectResultVecError::from_source",
            "pub fn key_seed",
            "pub fn value_seed",
            "pub fn render_label",
            "pub fn render_error",
        ],
        model_absent: &[
            "mod dead",
            "DeadCollectResultVecItem",
            "dead_collect_result_vec",
            "pub fn compare(&self",
            "pub fn unused_helper",
            "dead_method",
            "dead_error_method",
            "dead-collect-result-vec",
        ],
    });
}

#[test]
fn prunes_collect_option_vec_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "collect_option_vec_prune",
        root_fn: "selected_collect_option_vec_report",
        api_pkg: "collect_option_vec_api",
        model_pkg: "collect_option_vec_model",
        api_fn: "selected_collect_option_vec_report",
        model_fn: "selected_collect_option_vec",
        model_required: &[
            "collect::<Option<Vec<CollectOptionVecItem>>>()",
            "collect_option_vec_convert",
            ".then(|| CollectOptionVecItem::from_source",
            "pub fn is_live_source",
            "pub fn value_seed",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadCollectOptionVecItem",
            "dead_collect_option_vec",
            "pub fn key_seed",
            "pub fn compare(&self",
            "pub fn unused_helper",
            "dead_method",
            "dead-collect-option-vec",
        ],
    });
}

#[test]
fn prunes_collect_annotated_hashmap_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "collect_annotated_hashmap_prune",
        root_fn: "selected_collect_annotated_hashmap_report",
        api_pkg: "collect_annotated_hashmap_api",
        model_pkg: "collect_annotated_hashmap_model",
        api_fn: "selected_collect_annotated_hashmap_report",
        model_fn: "selected_collect_annotated_hashmap",
        model_required: &[
            "let collected: HashMap<",
            "CollectAnnotatedHashMapKey::from_source",
            "CollectAnnotatedHashMapItem::from_source",
            ".values()",
            "pub fn key_seed",
            "pub fn value_seed",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadCollectAnnotatedHashMapItem",
            "dead_collect_annotated_hashmap",
            "pub fn is_live_source",
            "pub fn render_key",
            "dead_key_method",
            "pub fn compare(&self",
            "pub fn unused_helper",
            "dead_method",
            "dead-collect-annotated-hashmap",
        ],
    });
}

#[test]
fn prunes_collect_annotated_btreemap_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "collect_annotated_btreemap_prune",
        root_fn: "selected_collect_annotated_btreemap_report",
        api_pkg: "collect_annotated_btreemap_api",
        model_pkg: "collect_annotated_btreemap_model",
        api_fn: "selected_collect_annotated_btreemap_report",
        model_fn: "selected_collect_annotated_btreemap",
        model_required: &[
            "let collected: BTreeMap<",
            "CollectAnnotatedBTreeMapKey::from_source",
            "CollectAnnotatedBTreeMapItem::from_source",
            ".values()",
            "pub fn key_seed",
            "pub fn value_seed",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadCollectAnnotatedBTreeMapItem",
            "dead_collect_annotated_btreemap",
            "pub fn is_live_source",
            "pub fn render_key",
            "dead_key_method",
            "pub fn compare(&self",
            "pub fn unused_helper",
            "dead_method",
            "dead-collect-annotated-btreemap",
        ],
    });
}

#[test]
fn prunes_collect_annotated_hashset_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "collect_annotated_hashset_prune",
        root_fn: "selected_collect_annotated_hashset_report",
        api_pkg: "collect_annotated_hashset_api",
        model_pkg: "collect_annotated_hashset_model",
        api_fn: "selected_collect_annotated_hashset_report",
        model_fn: "selected_collect_annotated_hashset",
        model_required: &[
            "let collected: HashSet<",
            ".filter(|source| source.is_live_source())",
            "CollectAnnotatedHashSetItem::from_source",
            ".iter()",
            "pub fn is_live_source",
            "pub fn value_seed",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadCollectAnnotatedHashSetItem",
            "dead_collect_annotated_hashset",
            "pub fn key_seed",
            "pub fn compare(&self",
            "pub fn unused_helper",
            "dead_method",
            "dead-collect-annotated-hashset",
        ],
    });
}

#[test]
fn prunes_collect_annotated_btreeset_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "collect_annotated_btreeset_prune",
        root_fn: "selected_collect_annotated_btreeset_report",
        api_pkg: "collect_annotated_btreeset_api",
        model_pkg: "collect_annotated_btreeset_model",
        api_fn: "selected_collect_annotated_btreeset_report",
        model_fn: "selected_collect_annotated_btreeset",
        model_required: &[
            "let collected: BTreeSet<",
            ".filter(|source| source.is_live_source())",
            "CollectAnnotatedBTreeSetItem::from_source",
            ".iter()",
            "pub fn is_live_source",
            "pub fn value_seed",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadCollectAnnotatedBTreeSetItem",
            "dead_collect_annotated_btreeset",
            "pub fn key_seed",
            "pub fn compare(&self",
            "pub fn unused_helper",
            "dead_method",
            "dead-collect-annotated-btreeset",
        ],
    });
}

#[test]
fn prunes_collect_annotated_binaryheap_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "collect_annotated_binaryheap_prune",
        root_fn: "selected_collect_annotated_binaryheap_report",
        api_pkg: "collect_annotated_binaryheap_api",
        model_pkg: "collect_annotated_binaryheap_model",
        api_fn: "selected_collect_annotated_binaryheap_report",
        model_fn: "selected_collect_annotated_binaryheap",
        model_required: &[
            "let collected: BinaryHeap<",
            ".filter(|source| source.is_live_source())",
            "CollectAnnotatedBinaryHeapItem::from_source",
            ".iter()",
            "pub fn is_live_source",
            "pub fn value_seed",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadCollectAnnotatedBinaryHeapItem",
            "dead_collect_annotated_binaryheap",
            "pub fn key_seed",
            "pub fn compare(&self",
            "pub fn unused_helper",
            "dead_method",
            "dead-collect-annotated-binaryheap",
        ],
    });
}

#[test]
fn prunes_collect_annotated_vecdeque_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "collect_annotated_vecdeque_prune",
        root_fn: "selected_collect_annotated_vecdeque_report",
        api_pkg: "collect_annotated_vecdeque_api",
        model_pkg: "collect_annotated_vecdeque_model",
        api_fn: "selected_collect_annotated_vecdeque_report",
        model_fn: "selected_collect_annotated_vecdeque",
        model_required: &[
            "let collected: VecDeque<",
            ".filter(|source| source.is_live_source())",
            "CollectAnnotatedVecDequeItem::from_source",
            ".iter()",
            "pub fn is_live_source",
            "pub fn value_seed",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadCollectAnnotatedVecDequeItem",
            "dead_collect_annotated_vecdeque",
            "pub fn key_seed",
            "pub fn compare(&self",
            "pub fn unused_helper",
            "dead_method",
            "dead-collect-annotated-vecdeque",
        ],
    });
}

#[test]
fn prunes_collect_annotated_linkedlist_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "collect_annotated_linkedlist_prune",
        root_fn: "selected_collect_annotated_linkedlist_report",
        api_pkg: "collect_annotated_linkedlist_api",
        model_pkg: "collect_annotated_linkedlist_model",
        api_fn: "selected_collect_annotated_linkedlist_report",
        model_fn: "selected_collect_annotated_linkedlist",
        model_required: &[
            "let collected: LinkedList<",
            ".filter(|source| source.is_live_source())",
            "CollectAnnotatedLinkedListItem::from_source",
            ".iter()",
            "pub fn is_live_source",
            "pub fn value_seed",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadCollectAnnotatedLinkedListItem",
            "dead_collect_annotated_linkedlist",
            "pub fn key_seed",
            "pub fn compare(&self",
            "pub fn unused_helper",
            "dead_method",
            "dead-collect-annotated-linkedlist",
        ],
    });
}

#[test]
fn prunes_collect_annotated_vec_tuple_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "collect_annotated_vec_tuple_prune",
        root_fn: "selected_collect_annotated_vec_tuple_report",
        api_pkg: "collect_annotated_vec_tuple_api",
        model_pkg: "collect_annotated_vec_tuple_model",
        api_fn: "selected_collect_annotated_vec_tuple_report",
        model_fn: "selected_collect_annotated_vec_tuple",
        model_required: &[
            "let collected: Vec<(",
            "CollectAnnotatedVecTupleKey::from_source",
            "CollectAnnotatedVecTupleItem::from_source",
            "pub fn key_seed",
            "pub fn value_seed",
            "pub fn render_key",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadCollectAnnotatedVecTupleItem",
            "dead_collect_annotated_vec_tuple",
            "pub fn is_live_source",
            "dead_key_method",
            "pub fn compare(&self",
            "pub fn unused_helper",
            "dead_method",
            "dead-collect-annotated-vec-tuple",
        ],
    });
}

#[test]
fn prunes_collect_annotated_result_vec_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "collect_annotated_result_vec_prune",
        root_fn: "selected_collect_annotated_result_vec_report",
        api_pkg: "collect_annotated_result_vec_api",
        model_pkg: "collect_annotated_result_vec_model",
        api_fn: "selected_collect_annotated_result_vec_report",
        model_fn: "selected_collect_annotated_result_vec",
        model_required: &[
            "let collected: Result<",
            "Vec<CollectAnnotatedResultVecItem>",
            "collect_annotated_result_vec_convert",
            "Ok(CollectAnnotatedResultVecItem::from_source",
            "Err(CollectAnnotatedResultVecError::from_source",
            "pub fn key_seed",
            "pub fn value_seed",
            "pub fn render_label",
            "pub fn render_error",
        ],
        model_absent: &[
            "mod dead",
            "DeadCollectAnnotatedResultVecItem",
            "dead_collect_annotated_result_vec",
            "pub fn compare(&self",
            "pub fn unused_helper",
            "dead_method",
            "dead_error_method",
            "dead-collect-annotated-result-vec",
        ],
    });
}

#[test]
fn prunes_collect_annotated_option_vec_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "collect_annotated_option_vec_prune",
        root_fn: "selected_collect_annotated_option_vec_report",
        api_pkg: "collect_annotated_option_vec_api",
        model_pkg: "collect_annotated_option_vec_model",
        api_fn: "selected_collect_annotated_option_vec_report",
        model_fn: "selected_collect_annotated_option_vec",
        model_required: &[
            "let collected: Option<Vec<",
            "collect_annotated_option_vec_convert",
            ".then(|| CollectAnnotatedOptionVecItem::from_source",
            "pub fn is_live_source",
            "pub fn value_seed",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadCollectAnnotatedOptionVecItem",
            "dead_collect_annotated_option_vec",
            "pub fn key_seed",
            "pub fn compare(&self",
            "pub fn unused_helper",
            "dead_method",
            "dead-collect-annotated-option-vec",
        ],
    });
}

#[test]
fn prunes_option_as_deref_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "option_as_deref_map_prune",
        root_fn: "selected_option_as_deref_map_report",
        api_pkg: "option_as_deref_api",
        model_pkg: "option_as_deref_model",
        api_fn: "selected_option_as_deref_map_report",
        model_fn: "selected_option_as_deref_map",
        model_required: &[".as_deref()", "pub fn render_label"],
        model_absent: &[
            "mod dead",
            "DeadOptionAsDerefMapItem",
            "dead_option_as_deref_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-option-as-deref-map",
        ],
    });
}

#[test]
fn prunes_option_as_deref_mut_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "option_as_deref_mut_map_prune",
        root_fn: "selected_option_as_deref_mut_map_report",
        api_pkg: "option_as_deref_mut_api",
        model_pkg: "option_as_deref_mut_model",
        api_fn: "selected_option_as_deref_mut_map_report",
        model_fn: "selected_option_as_deref_mut_map",
        model_required: &[".as_deref_mut()", "pub fn bump_and_render"],
        model_absent: &[
            "mod dead",
            "DeadOptionAsDerefMutMapItem",
            "dead_option_as_deref_mut_map",
            "pub fn render_label",
            "dead_method",
            "dead-option-as-deref-mut-map",
        ],
    });
}

#[test]
fn prunes_result_as_ref_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "result_as_ref_map_prune",
        root_fn: "selected_result_as_ref_map_report",
        api_pkg: "result_as_ref_api",
        model_pkg: "result_as_ref_model",
        api_fn: "selected_result_as_ref_map_report",
        model_fn: "selected_result_as_ref_map",
        model_required: &[".as_ref()", "pub fn render_label", "pub fn render_error"],
        model_absent: &[
            "mod dead",
            "DeadResultAsRefMapItem",
            "dead_result_as_ref_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead_error_method",
            "dead-result-as-ref-map",
        ],
    });
}

#[test]
fn prunes_result_as_deref_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "result_as_deref_map_prune",
        root_fn: "selected_result_as_deref_map_report",
        api_pkg: "result_as_deref_api",
        model_pkg: "result_as_deref_model",
        api_fn: "selected_result_as_deref_map_report",
        model_fn: "selected_result_as_deref_map",
        model_required: &[".as_deref()", "pub fn render_label", "pub fn render_error"],
        model_absent: &[
            "mod dead",
            "DeadResultAsDerefMapItem",
            "dead_result_as_deref_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead_error_method",
            "dead-result-as-deref-map",
        ],
    });
}

#[test]
fn prunes_result_as_deref_mut_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "result_as_deref_mut_map_prune",
        root_fn: "selected_result_as_deref_mut_map_report",
        api_pkg: "result_as_deref_mut_api",
        model_pkg: "result_as_deref_mut_model",
        api_fn: "selected_result_as_deref_mut_map_report",
        model_fn: "selected_result_as_deref_mut_map",
        model_required: &[
            ".as_deref_mut()",
            "pub fn bump_and_render",
            "pub fn render_error",
        ],
        model_absent: &[
            "mod dead",
            "DeadResultAsDerefMutMapItem",
            "dead_result_as_deref_mut_map",
            "pub fn render_label",
            "dead_method",
            "dead_error_method",
            "dead-result-as-deref-mut-map",
        ],
    });
}

#[test]
fn prunes_box_as_ref_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "box_as_ref_map_prune",
        root_fn: "selected_box_as_ref_map_report",
        api_pkg: "box_as_ref_api",
        model_pkg: "box_as_ref_model",
        api_fn: "selected_box_as_ref_map_report",
        model_fn: "selected_box_as_ref_map",
        model_required: &[
            "Box::new",
            ".as_ref().render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBoxAsRefMapItem",
            "dead_box_as_ref_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-box-as-ref-map",
        ],
    });
}

#[test]
fn prunes_arc_as_ref_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "arc_as_ref_map_prune",
        root_fn: "selected_arc_as_ref_map_report",
        api_pkg: "arc_as_ref_api",
        model_pkg: "arc_as_ref_model",
        api_fn: "selected_arc_as_ref_map_report",
        model_fn: "selected_arc_as_ref_map",
        model_required: &[
            "use std::sync::Arc;",
            "Arc::new",
            ".as_ref().render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadArcAsRefMapItem",
            "dead_arc_as_ref_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-arc-as-ref-map",
        ],
    });
}

#[test]
fn prunes_rc_as_ref_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "rc_as_ref_map_prune",
        root_fn: "selected_rc_as_ref_map_report",
        api_pkg: "rc_as_ref_api",
        model_pkg: "rc_as_ref_model",
        api_fn: "selected_rc_as_ref_map_report",
        model_fn: "selected_rc_as_ref_map",
        model_required: &[
            "use std::rc::Rc;",
            "Rc::new",
            ".as_ref().render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadRcAsRefMapItem",
            "dead_rc_as_ref_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-rc-as-ref-map",
        ],
    });
}

#[test]
fn prunes_vec_box_iter_as_ref_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "vec_box_iter_as_ref_map_prune",
        root_fn: "selected_vec_box_iter_as_ref_map_report",
        api_pkg: "vec_box_iter_as_ref_api",
        model_pkg: "vec_box_iter_as_ref_model",
        api_fn: "selected_vec_box_iter_as_ref_map_report",
        model_fn: "selected_vec_box_iter_as_ref_map",
        model_required: &[
            "Vec<Box<VecBoxIterAsRefMapPayload>>",
            ".iter()",
            ".as_ref().render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadVecBoxIterAsRefMapItem",
            "dead_vec_box_iter_as_ref_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-vec-box-iter-as-ref-map",
        ],
    });
}

#[test]
fn prunes_option_copied_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "option_copied_map_prune",
        root_fn: "selected_option_copied_map_report",
        api_pkg: "option_copied_api",
        model_pkg: "option_copied_model",
        api_fn: "selected_option_copied_map_report",
        model_fn: "selected_option_copied_map",
        model_required: &[".copied()", "pub fn render_label"],
        model_absent: &[
            "mod dead",
            "DeadOptionCopiedMapItem",
            "dead_option_copied_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-option-copied-map",
        ],
    });
}

#[test]
fn prunes_refcell_borrow_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "refcell_borrow_map_prune",
        root_fn: "selected_refcell_borrow_map_report",
        api_pkg: "refcell_borrow_api",
        model_pkg: "refcell_borrow_model",
        api_fn: "selected_refcell_borrow_map_report",
        model_fn: "selected_refcell_borrow_map",
        model_required: &[
            "use std::cell::RefCell;",
            ".borrow().render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadRefCellBorrowMapItem",
            "dead_refcell_borrow_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-refcell-borrow-map",
        ],
    });
}

#[test]
fn prunes_refcell_borrow_mut_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "refcell_borrow_mut_map_prune",
        root_fn: "selected_refcell_borrow_mut_map_report",
        api_pkg: "refcell_borrow_mut_api",
        model_pkg: "refcell_borrow_mut_model",
        api_fn: "selected_refcell_borrow_mut_map_report",
        model_fn: "selected_refcell_borrow_mut_map",
        model_required: &[
            "use std::cell::RefCell;",
            ".borrow_mut().bump_and_render()",
            "pub fn bump_and_render",
        ],
        model_absent: &[
            "mod dead",
            "DeadRefCellBorrowMutMapItem",
            "dead_refcell_borrow_mut_map",
            "pub fn render_label",
            "dead_method",
            "dead-refcell-borrow-mut-map",
        ],
    });
}

#[test]
fn prunes_cell_get_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "cell_get_map_prune",
        root_fn: "selected_cell_get_map_report",
        api_pkg: "cell_get_api",
        model_pkg: "cell_get_model",
        api_fn: "selected_cell_get_map_report",
        model_fn: "selected_cell_get_map",
        model_required: &[
            "use std::cell::Cell;",
            ".get().render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadCellGetMapItem",
            "dead_cell_get_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-cell-get-map",
        ],
    });
}

#[test]
fn prunes_once_lock_get_or_init_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "once_lock_get_or_init_map_prune",
        root_fn: "selected_once_lock_get_or_init_map_report",
        api_pkg: "once_lock_get_or_init_api",
        model_pkg: "once_lock_get_or_init_model",
        api_fn: "selected_once_lock_get_or_init_map_report",
        model_fn: "selected_once_lock_get_or_init_map",
        model_required: &[
            "use std::sync::OnceLock;",
            ".get_or_init(||",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadOnceLockGetOrInitMapItem",
            "dead_once_lock_get_or_init_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-once-lock-get-or-init-map",
        ],
    });
}

#[test]
fn prunes_mutex_lock_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "mutex_lock_map_prune",
        root_fn: "selected_mutex_lock_map_report",
        api_pkg: "mutex_lock_api",
        model_pkg: "mutex_lock_model",
        api_fn: "selected_mutex_lock_map_report",
        model_fn: "selected_mutex_lock_map",
        model_required: &[
            "use std::sync::Mutex;",
            ".lock()",
            "guard.render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadMutexLockMapItem",
            "dead_mutex_lock_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-mutex-lock-map",
        ],
    });
}

#[test]
fn prunes_mutex_lock_mut_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "mutex_lock_mut_map_prune",
        root_fn: "selected_mutex_lock_mut_map_report",
        api_pkg: "mutex_lock_mut_api",
        model_pkg: "mutex_lock_mut_model",
        api_fn: "selected_mutex_lock_mut_map_report",
        model_fn: "selected_mutex_lock_mut_map",
        model_required: &[
            "use std::sync::Mutex;",
            ".lock()",
            "guard.bump_and_render()",
            "pub fn bump_and_render",
        ],
        model_absent: &[
            "mod dead",
            "DeadMutexLockMutMapItem",
            "dead_mutex_lock_mut_map",
            "pub fn render_label",
            "dead_method",
            "dead-mutex-lock-mut-map",
        ],
    });
}

#[test]
fn prunes_rwlock_read_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "rwlock_read_map_prune",
        root_fn: "selected_rwlock_read_map_report",
        api_pkg: "rwlock_read_api",
        model_pkg: "rwlock_read_model",
        api_fn: "selected_rwlock_read_map_report",
        model_fn: "selected_rwlock_read_map",
        model_required: &[
            "use std::sync::RwLock;",
            ".read()",
            "guard.render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadRwLockReadMapItem",
            "dead_rwlock_read_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-rwlock-read-map",
        ],
    });
}

#[test]
fn prunes_rwlock_write_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "rwlock_write_map_prune",
        root_fn: "selected_rwlock_write_map_report",
        api_pkg: "rwlock_write_api",
        model_pkg: "rwlock_write_model",
        api_fn: "selected_rwlock_write_map_report",
        model_fn: "selected_rwlock_write_map",
        model_required: &[
            "use std::sync::RwLock;",
            ".write()",
            "guard.bump_and_render()",
            "pub fn bump_and_render",
        ],
        model_absent: &[
            "mod dead",
            "DeadRwLockWriteMapItem",
            "dead_rwlock_write_map",
            "pub fn render_label",
            "dead_method",
            "dead-rwlock-write-map",
        ],
    });
}

#[test]
fn prunes_option_refcell_borrow_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "option_refcell_borrow_map_prune",
        root_fn: "selected_option_refcell_borrow_map_report",
        api_pkg: "option_refcell_borrow_api",
        model_pkg: "option_refcell_borrow_model",
        api_fn: "selected_option_refcell_borrow_map_report",
        model_fn: "selected_option_refcell_borrow_map",
        model_required: &[
            "use std::cell::RefCell;",
            ".as_ref()",
            ".borrow().render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadOptionRefCellBorrowMapItem",
            "dead_option_refcell_borrow_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-option-refcell-borrow-map",
        ],
    });
}

#[test]
fn prunes_arc_mutex_lock_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "arc_mutex_lock_map_prune",
        root_fn: "selected_arc_mutex_lock_map_report",
        api_pkg: "arc_mutex_lock_api",
        model_pkg: "arc_mutex_lock_model",
        api_fn: "selected_arc_mutex_lock_map_report",
        model_fn: "selected_arc_mutex_lock_map",
        model_required: &[
            "use std::sync::{Arc, Mutex};",
            "Arc::new(Mutex::new",
            ".lock()",
            "guard.render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadArcMutexLockMapItem",
            "dead_arc_mutex_lock_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-arc-mutex-lock-map",
        ],
    });
}

#[test]
fn prunes_rc_refcell_borrow_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "rc_refcell_borrow_map_prune",
        root_fn: "selected_rc_refcell_borrow_map_report",
        api_pkg: "rc_refcell_borrow_api",
        model_pkg: "rc_refcell_borrow_model",
        api_fn: "selected_rc_refcell_borrow_map_report",
        model_fn: "selected_rc_refcell_borrow_map",
        model_required: &[
            "use std::cell::RefCell;",
            "use std::rc::Rc;",
            "Rc::new(RefCell::new",
            ".borrow().render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadRcRefCellBorrowMapItem",
            "dead_rc_refcell_borrow_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-rc-refcell-borrow-map",
        ],
    });
}

#[test]
fn prunes_cow_borrowed_as_ref_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "cow_borrowed_as_ref_map_prune",
        root_fn: "selected_cow_borrowed_as_ref_map_report",
        api_pkg: "cow_borrowed_as_ref_api",
        model_pkg: "cow_borrowed_as_ref_model",
        api_fn: "selected_cow_borrowed_as_ref_map_report",
        model_fn: "selected_cow_borrowed_as_ref_map",
        model_required: &[
            "use std::borrow::Cow;",
            "Cow::Borrowed",
            ".as_ref().render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadCowBorrowedAsRefMapItem",
            "dead_cow_borrowed_as_ref_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-cow-borrowed-as-ref-map",
        ],
    });
}

#[test]
fn prunes_cow_owned_into_owned_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "cow_owned_into_owned_map_prune",
        root_fn: "selected_cow_owned_into_owned_map_report",
        api_pkg: "cow_owned_into_owned_api",
        model_pkg: "cow_owned_into_owned_model",
        api_fn: "selected_cow_owned_into_owned_map_report",
        model_fn: "selected_cow_owned_into_owned_map",
        model_required: &[
            "use std::borrow::Cow;",
            "Cow::Owned",
            ".into_owned().render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadCowOwnedIntoOwnedMapItem",
            "dead_cow_owned_into_owned_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-cow-owned-into-owned-map",
        ],
    });
}

#[test]
fn prunes_cow_to_mut_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "cow_to_mut_map_prune",
        root_fn: "selected_cow_to_mut_map_report",
        api_pkg: "cow_to_mut_api",
        model_pkg: "cow_to_mut_model",
        api_fn: "selected_cow_to_mut_map_report",
        model_fn: "selected_cow_to_mut_map",
        model_required: &[
            "use std::borrow::Cow;",
            ".to_mut().bump_and_render()",
            "pub fn bump_and_render",
        ],
        model_absent: &[
            "mod dead",
            "DeadCowToMutMapItem",
            "dead_cow_to_mut_map",
            "pub fn render_label",
            "dead_method",
            "dead-cow-to-mut-map",
        ],
    });
}

#[test]
fn prunes_borrow_trait_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "borrow_trait_map_prune",
        root_fn: "selected_borrow_trait_map_report",
        api_pkg: "borrow_trait_api",
        model_pkg: "borrow_trait_model",
        api_fn: "selected_borrow_trait_map_report",
        model_fn: "selected_borrow_trait_map",
        model_required: &[
            "use std::borrow::Borrow;",
            "impl Borrow<BorrowTraitMapPayload>",
            ".borrow()",
            "payload.render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBorrowTraitMapItem",
            "dead_borrow_trait_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead_wrapper_method",
            "dead-borrow-trait-map",
        ],
    });
}

#[test]
fn prunes_as_ref_trait_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "as_ref_trait_map_prune",
        root_fn: "selected_as_ref_trait_map_report",
        api_pkg: "as_ref_trait_api",
        model_pkg: "as_ref_trait_model",
        api_fn: "selected_as_ref_trait_map_report",
        model_fn: "selected_as_ref_trait_map",
        model_required: &[
            "impl AsRef<AsRefTraitMapPayload>",
            ".as_ref().render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadAsRefTraitMapItem",
            "dead_as_ref_trait_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead_wrapper_method",
            "dead-as-ref-trait-map",
        ],
    });
}

#[test]
fn prunes_as_mut_trait_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "as_mut_trait_map_prune",
        root_fn: "selected_as_mut_trait_map_report",
        api_pkg: "as_mut_trait_api",
        model_pkg: "as_mut_trait_model",
        api_fn: "selected_as_mut_trait_map_report",
        model_fn: "selected_as_mut_trait_map",
        model_required: &[
            "impl AsMut<AsMutTraitMapPayload>",
            ".as_mut().bump_and_render()",
            "pub fn bump_and_render",
        ],
        model_absent: &[
            "mod dead",
            "DeadAsMutTraitMapItem",
            "dead_as_mut_trait_map",
            "pub fn render_label",
            "dead_method",
            "dead_wrapper_method",
            "dead-as-mut-trait-map",
        ],
    });
}

#[test]
fn prunes_deref_mut_trait_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "deref_mut_trait_map_prune",
        root_fn: "selected_deref_mut_trait_map_report",
        api_pkg: "deref_mut_trait_api",
        model_pkg: "deref_mut_trait_model",
        api_fn: "selected_deref_mut_trait_map_report",
        model_fn: "selected_deref_mut_trait_map",
        model_required: &[
            "use std::ops::{Deref, DerefMut};",
            "impl Deref for DerefMutTraitMapWrapper",
            "impl DerefMut for DerefMutTraitMapWrapper",
            "wrapper.bump_and_render()",
            "pub fn bump_and_render",
        ],
        model_absent: &[
            "mod dead",
            "DeadDerefMutTraitMapItem",
            "dead_deref_mut_trait_map",
            "pub fn render_label",
            "dead_method",
            "dead_wrapper_method",
            "dead-deref-mut-trait-map",
        ],
    });
}

#[test]
fn prunes_pin_box_as_ref_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "pin_box_as_ref_map_prune",
        root_fn: "selected_pin_box_as_ref_map_report",
        api_pkg: "pin_box_as_ref_api",
        model_pkg: "pin_box_as_ref_model",
        api_fn: "selected_pin_box_as_ref_map_report",
        model_fn: "selected_pin_box_as_ref_map",
        model_required: &[
            "use std::pin::Pin;",
            "Pin::new(Box::new",
            ".as_ref().get_ref().render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadPinBoxAsRefMapItem",
            "dead_pin_box_as_ref_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-pin-box-as-ref-map",
        ],
    });
}

#[test]
fn prunes_pin_box_as_mut_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "pin_box_as_mut_map_prune",
        root_fn: "selected_pin_box_as_mut_map_report",
        api_pkg: "pin_box_as_mut_api",
        model_pkg: "pin_box_as_mut_model",
        api_fn: "selected_pin_box_as_mut_map_report",
        model_fn: "selected_pin_box_as_mut_map",
        model_required: &[
            "use std::pin::Pin;",
            "Pin::new(Box::new",
            ".as_mut().get_mut().bump_and_render()",
            "pub fn bump_and_render",
        ],
        model_absent: &[
            "mod dead",
            "DeadPinBoxAsMutMapItem",
            "dead_pin_box_as_mut_map",
            "pub fn render_label",
            "dead_method",
            "dead-pin-box-as-mut-map",
        ],
    });
}

#[test]
fn prunes_phantomdata_surface_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "phantomdata_surface_map_prune",
        root_fn: "selected_phantomdata_surface_map_report",
        api_pkg: "phantomdata_surface_api",
        model_pkg: "phantomdata_surface_model",
        api_fn: "selected_phantomdata_surface_map_report",
        model_fn: "selected_phantomdata_surface_map",
        model_required: &[
            "use std::marker::PhantomData;",
            "PhantomData<T>",
            "PhantomDataSurfaceMapMarker",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadPhantomDataSurfaceMapItem",
            "DeadPhantomDataSurfaceMapMarker",
            "dead_phantomdata_surface_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead_wrapper_method",
            "dead-phantomdata-surface-map",
        ],
    });
}

#[test]
fn prunes_weak_upgrade_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "weak_upgrade_map_prune",
        root_fn: "selected_weak_upgrade_map_report",
        api_pkg: "weak_upgrade_map_api",
        model_pkg: "weak_upgrade_map_model",
        api_fn: "selected_weak_upgrade_map_report",
        model_fn: "selected_weak_upgrade_map",
        model_required: &[
            "use std::sync::{Arc, Weak};",
            "Arc::downgrade",
            ".upgrade()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadWeakUpgradeMapItem",
            "dead_weak_upgrade_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-weak-upgrade-map",
        ],
    });
}

#[test]
fn prunes_rc_weak_upgrade_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "rc_weak_upgrade_map_prune",
        root_fn: "selected_rc_weak_upgrade_map_report",
        api_pkg: "rc_weak_upgrade_map_api",
        model_pkg: "rc_weak_upgrade_map_model",
        api_fn: "selected_rc_weak_upgrade_map_report",
        model_fn: "selected_rc_weak_upgrade_map",
        model_required: &[
            "use std::rc::{Rc, Weak};",
            "Rc::downgrade",
            ".upgrade()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadRcWeakUpgradeMapItem",
            "dead_rc_weak_upgrade_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-rc-weak-upgrade-map",
        ],
    });
}

#[test]
fn prunes_option_take_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "option_take_map_prune",
        root_fn: "selected_option_take_map_report",
        api_pkg: "option_take_map_api",
        model_pkg: "option_take_map_model",
        api_fn: "selected_option_take_map_report",
        model_fn: "selected_option_take_map",
        model_required: &[
            "pub struct OptionTakeMapSlot",
            ".take()",
            "pub fn take_render",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadOptionTakeMapItem",
            "dead_option_take_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead_slot_method",
            "dead-option-take-map",
        ],
    });
}

#[test]
fn prunes_option_get_or_insert_with_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "option_get_or_insert_with_map_prune",
        root_fn: "selected_option_get_or_insert_with_map_report",
        api_pkg: "option_get_or_insert_with_map_api",
        model_pkg: "option_get_or_insert_with_map_model",
        api_fn: "selected_option_get_or_insert_with_map_report",
        model_fn: "selected_option_get_or_insert_with_map",
        model_required: &[
            "pub struct OptionGetOrInsertWithMapSlot",
            ".get_or_insert_with(",
            "pub fn ensure_render",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadOptionGetOrInsertWithMapItem",
            "dead_option_get_or_insert_with_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead_slot_method",
            "dead-option-get-or-insert-with-map",
        ],
    });
}

#[test]
fn prunes_option_insert_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "option_insert_map_prune",
        root_fn: "selected_option_insert_map_report",
        api_pkg: "option_insert_map_api",
        model_pkg: "option_insert_map_model",
        api_fn: "selected_option_insert_map_report",
        model_fn: "selected_option_insert_map",
        model_required: &[
            "pub struct OptionInsertMapSlot",
            ".insert(",
            "pub fn insert_render",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadOptionInsertMapItem",
            "dead_option_insert_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead_slot_method",
            "dead-option-insert-map",
        ],
    });
}

#[test]
fn prunes_mem_take_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "mem_take_map_prune",
        root_fn: "selected_mem_take_map_report",
        api_pkg: "mem_take_map_api",
        model_pkg: "mem_take_map_model",
        api_fn: "selected_mem_take_map_report",
        model_fn: "selected_mem_take_map",
        model_required: &[
            "use std::mem;",
            "mem::take(&mut payload)",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadMemTakeMapItem",
            "dead_mem_take_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-mem-take-map",
        ],
    });
}

#[test]
fn prunes_mem_replace_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "mem_replace_map_prune",
        root_fn: "selected_mem_replace_map_report",
        api_pkg: "mem_replace_map_api",
        model_pkg: "mem_replace_map_model",
        api_fn: "selected_mem_replace_map_report",
        model_fn: "selected_mem_replace_map",
        model_required: &[
            "use std::mem;",
            "mem::replace(&mut payload",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadMemReplaceMapItem",
            "dead_mem_replace_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-mem-replace-map",
        ],
    });
}

#[test]
fn prunes_cell_replace_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "cell_replace_map_prune",
        root_fn: "selected_cell_replace_map_report",
        api_pkg: "cell_replace_map_api",
        model_pkg: "cell_replace_map_model",
        api_fn: "selected_cell_replace_map_report",
        model_fn: "selected_cell_replace_map",
        model_required: &[
            "use std::cell::Cell;",
            "Cell::new",
            ".replace(",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadCellReplaceMapItem",
            "dead_cell_replace_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-cell-replace-map",
        ],
    });
}

#[test]
fn prunes_refcell_replace_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "refcell_replace_map_prune",
        root_fn: "selected_refcell_replace_map_report",
        api_pkg: "refcell_replace_map_api",
        model_pkg: "refcell_replace_map_model",
        api_fn: "selected_refcell_replace_map_report",
        model_fn: "selected_refcell_replace_map",
        model_required: &[
            "use std::cell::RefCell;",
            "RefCell::new",
            ".replace(",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadRefCellReplaceMapItem",
            "dead_refcell_replace_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-refcell-replace-map",
        ],
    });
}

#[test]
fn prunes_once_lock_get_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "once_lock_get_map_prune",
        root_fn: "selected_once_lock_get_map_report",
        api_pkg: "once_lock_get_map_api",
        model_pkg: "once_lock_get_map_model",
        api_fn: "selected_once_lock_get_map_report",
        model_fn: "selected_once_lock_get_map",
        model_required: &[
            "use std::sync::OnceLock;",
            "OnceLock::new",
            ".get()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadOnceLockGetMapItem",
            "dead_once_lock_get_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-once-lock-get-map",
        ],
    });
}

#[test]
fn prunes_iter_from_fn_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iter_from_fn_map_prune",
        root_fn: "selected_iter_from_fn_map_report",
        api_pkg: "iter_from_fn_map_api",
        model_pkg: "iter_from_fn_map_model",
        api_fn: "selected_iter_from_fn_map_report",
        model_fn: "selected_iter_from_fn_map",
        model_required: &[
            "use std::iter;",
            "iter::from_fn",
            ".next()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadIterFromFnMapItem",
            "dead_iter_from_fn_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-iter-from-fn-map",
        ],
    });
}

#[test]
fn prunes_iter_successors_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "iter_successors_map_prune",
        root_fn: "selected_iter_successors_map_report",
        api_pkg: "iter_successors_map_api",
        model_pkg: "iter_successors_map_model",
        api_fn: "selected_iter_successors_map_report",
        model_fn: "selected_iter_successors_map",
        model_required: &[
            "use std::iter;",
            "iter::successors",
            ".next()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadIterSuccessorsMapItem",
            "dead_iter_successors_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-iter-successors-map",
        ],
    });
}

#[test]
fn prunes_option_as_mut_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "option_as_mut_map_prune",
        root_fn: "selected_option_as_mut_map_report",
        api_pkg: "option_as_mut_map_api",
        model_pkg: "option_as_mut_map_model",
        api_fn: "selected_option_as_mut_map_report",
        model_fn: "selected_option_as_mut_map",
        model_required: &[
            ".as_mut()",
            ".map(|payload| payload.bump_and_render())",
            "pub fn bump_and_render",
        ],
        model_absent: &[
            "mod dead",
            "DeadOptionAsMutMapItem",
            "dead_option_as_mut_map",
            "pub fn render_label",
            "dead_method",
            "dead-option-as-mut-map",
        ],
    });
}

#[test]
fn prunes_result_as_mut_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "result_as_mut_map_prune",
        root_fn: "selected_result_as_mut_map_report",
        api_pkg: "result_as_mut_map_api",
        model_pkg: "result_as_mut_map_model",
        api_fn: "selected_result_as_mut_map_report",
        model_fn: "selected_result_as_mut_map",
        model_required: &[
            "ResultAsMutMapError",
            ".as_mut()",
            ".map(|payload| payload.bump_and_render())",
            "pub fn bump_and_render",
            "pub fn render_error",
        ],
        model_absent: &[
            "mod dead",
            "DeadResultAsMutMapItem",
            "dead_result_as_mut_map",
            "pub fn render_label",
            "dead_method",
            "dead_error_method",
            "dead-result-as-mut-map",
        ],
    });
}

#[test]
fn prunes_option_take_if_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "option_take_if_map_prune",
        root_fn: "selected_option_take_if_map_report",
        api_pkg: "option_take_if_map_api",
        model_pkg: "option_take_if_map_model",
        api_fn: "selected_option_take_if_map_report",
        model_fn: "selected_option_take_if_map",
        model_required: &[
            ".take_if(|payload| payload.allow())",
            ".map(|payload| payload.render_label())",
            "pub fn allow",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadOptionTakeIfMapItem",
            "dead_option_take_if_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-option-take-if-map",
        ],
    });
}

#[test]
fn prunes_mem_swap_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "mem_swap_map_prune",
        root_fn: "selected_mem_swap_map_report",
        api_pkg: "mem_swap_map_api",
        model_pkg: "mem_swap_map_model",
        api_fn: "selected_mem_swap_map_report",
        model_fn: "selected_mem_swap_map",
        model_required: &[
            "use std::mem;",
            "mem::swap(&mut left, &mut right)",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadMemSwapMapItem",
            "dead_mem_swap_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-mem-swap-map",
        ],
    });
}

#[test]
fn prunes_maybeuninit_assume_init_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "maybeuninit_assume_init_map_prune",
        root_fn: "selected_maybeuninit_assume_init_map_report",
        api_pkg: "maybeuninit_assume_init_map_api",
        model_pkg: "maybeuninit_assume_init_map_model",
        api_fn: "selected_maybeuninit_assume_init_map_report",
        model_fn: "selected_maybeuninit_assume_init_map",
        model_required: &[
            "use std::mem::MaybeUninit;",
            "MaybeUninit<MaybeUninitAssumeInitMapPayload>",
            ".assume_init().render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadMaybeUninitAssumeInitMapItem",
            "dead_maybeuninit_assume_init_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-maybeuninit-assume-init-map",
        ],
    });
}

#[test]
fn prunes_manuallydrop_into_inner_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "manuallydrop_into_inner_map_prune",
        root_fn: "selected_manuallydrop_into_inner_map_report",
        api_pkg: "manuallydrop_into_inner_map_api",
        model_pkg: "manuallydrop_into_inner_map_model",
        api_fn: "selected_manuallydrop_into_inner_map_report",
        model_fn: "selected_manuallydrop_into_inner_map",
        model_required: &[
            "use std::mem::ManuallyDrop;",
            "ManuallyDrop<ManuallyDropIntoInnerMapPayload>",
            "ManuallyDrop::into_inner(payload).render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadManuallyDropIntoInnerMapItem",
            "dead_manuallydrop_into_inner_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-manuallydrop-into-inner-map",
        ],
    });
}

#[test]
fn prunes_nonnull_as_ref_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "nonnull_as_ref_map_prune",
        root_fn: "selected_nonnull_as_ref_map_report",
        api_pkg: "nonnull_as_ref_map_api",
        model_pkg: "nonnull_as_ref_map_model",
        api_fn: "selected_nonnull_as_ref_map_report",
        model_fn: "selected_nonnull_as_ref_map",
        model_required: &[
            "use std::ptr::NonNull;",
            "NonNull<NonNullAsRefMapPayload>",
            "pointer.as_ref().render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadNonNullAsRefMapItem",
            "dead_nonnull_as_ref_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-nonnull-as-ref-map",
        ],
    });
}

#[test]
fn prunes_box_pin_as_ref_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "box_pin_as_ref_map_prune",
        root_fn: "selected_box_pin_as_ref_map_report",
        api_pkg: "box_pin_as_ref_map_api",
        model_pkg: "box_pin_as_ref_map_model",
        api_fn: "selected_box_pin_as_ref_map_report",
        model_fn: "selected_box_pin_as_ref_map",
        model_required: &[
            "use std::pin::Pin;",
            "Pin<Box<BoxPinAsRefMapPayload>>",
            "Box::pin",
            "pinned.as_ref().get_ref().render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBoxPinAsRefMapItem",
            "dead_box_pin_as_ref_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-box-pin-as-ref-map",
        ],
    });
}

#[test]
fn prunes_arc_make_mut_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "arc_make_mut_map_prune",
        root_fn: "selected_arc_make_mut_map_report",
        api_pkg: "arc_make_mut_map_api",
        model_pkg: "arc_make_mut_map_model",
        api_fn: "selected_arc_make_mut_map_report",
        model_fn: "selected_arc_make_mut_map",
        model_required: &[
            "use std::sync::Arc;",
            "Arc::make_mut(&mut payload).bump_and_render()",
            "pub fn bump_and_render",
        ],
        model_absent: &[
            "mod dead",
            "DeadArcMakeMutMapItem",
            "dead_arc_make_mut_map",
            "pub fn render_label",
            "dead_method",
            "dead-arc-make-mut-map",
        ],
    });
}

#[test]
fn prunes_rc_make_mut_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "rc_make_mut_map_prune",
        root_fn: "selected_rc_make_mut_map_report",
        api_pkg: "rc_make_mut_map_api",
        model_pkg: "rc_make_mut_map_model",
        api_fn: "selected_rc_make_mut_map_report",
        model_fn: "selected_rc_make_mut_map",
        model_required: &[
            "use std::rc::Rc;",
            "Rc::make_mut(&mut payload).bump_and_render()",
            "pub fn bump_and_render",
        ],
        model_absent: &[
            "mod dead",
            "DeadRcMakeMutMapItem",
            "dead_rc_make_mut_map",
            "pub fn render_label",
            "dead_method",
            "dead-rc-make-mut-map",
        ],
    });
}

#[test]
fn prunes_control_flow_continue_match_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "control_flow_continue_match_map_prune",
        root_fn: "selected_control_flow_continue_match_map_report",
        api_pkg: "control_flow_continue_match_map_api",
        model_pkg: "control_flow_continue_match_map_model",
        api_fn: "selected_control_flow_continue_match_map_report",
        model_fn: "selected_control_flow_continue_match_map",
        model_required: &[
            "use std::ops::ControlFlow;",
            "ControlFlow<(), ControlFlowContinueMatchMapPayload>",
            "ControlFlow::Continue(payload) => payload.render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadControlFlowContinueMatchMapItem",
            "dead_control_flow_continue_match_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-control-flow-continue-match-map",
        ],
    });
}

#[test]
fn prunes_control_flow_break_match_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "control_flow_break_match_map_prune",
        root_fn: "selected_control_flow_break_match_map_report",
        api_pkg: "control_flow_break_match_map_api",
        model_pkg: "control_flow_break_match_map_model",
        api_fn: "selected_control_flow_break_match_map_report",
        model_fn: "selected_control_flow_break_match_map",
        model_required: &[
            "use std::ops::ControlFlow;",
            "ControlFlow<ControlFlowBreakMatchMapPayload, ()>",
            "ControlFlow::Break(payload) => payload.render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadControlFlowBreakMatchMapItem",
            "dead_control_flow_break_match_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-control-flow-break-match-map",
        ],
    });
}

#[test]
fn prunes_option_iter_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "option_iter_map_prune",
        root_fn: "selected_option_iter_map_report",
        api_pkg: "option_iter_map_api",
        model_pkg: "option_iter_map_model",
        api_fn: "selected_option_iter_map_report",
        model_fn: "selected_option_iter_map",
        model_required: &[
            ".iter()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadOptionIterMapItem",
            "dead_option_iter_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-option-iter-map",
        ],
    });
}

#[test]
fn prunes_option_iter_mut_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "option_iter_mut_map_prune",
        root_fn: "selected_option_iter_mut_map_report",
        api_pkg: "option_iter_mut_map_api",
        model_pkg: "option_iter_mut_map_model",
        api_fn: "selected_option_iter_mut_map_report",
        model_fn: "selected_option_iter_mut_map",
        model_required: &[
            ".iter_mut()",
            ".map(|payload| payload.bump_and_render())",
            "pub fn bump_and_render",
        ],
        model_absent: &[
            "mod dead",
            "DeadOptionIterMutMapItem",
            "dead_option_iter_mut_map",
            "pub fn render_label",
            "dead_method",
            "dead-option-iter-mut-map",
        ],
    });
}

#[test]
fn prunes_result_iter_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "result_iter_map_prune",
        root_fn: "selected_result_iter_map_report",
        api_pkg: "result_iter_map_api",
        model_pkg: "result_iter_map_model",
        api_fn: "selected_result_iter_map_report",
        model_fn: "selected_result_iter_map",
        model_required: &[
            "ResultIterMapError",
            ".iter()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
            "pub fn render_error",
        ],
        model_absent: &[
            "mod dead",
            "DeadResultIterMapItem",
            "dead_result_iter_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead_error_method",
            "dead-result-iter-map",
        ],
    });
}

#[test]
fn prunes_result_iter_mut_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "result_iter_mut_map_prune",
        root_fn: "selected_result_iter_mut_map_report",
        api_pkg: "result_iter_mut_map_api",
        model_pkg: "result_iter_mut_map_model",
        api_fn: "selected_result_iter_mut_map_report",
        model_fn: "selected_result_iter_mut_map",
        model_required: &[
            "ResultIterMutMapError",
            ".iter_mut()",
            ".map(|payload| payload.bump_and_render())",
            "pub fn bump_and_render",
            "pub fn render_error",
        ],
        model_absent: &[
            "mod dead",
            "DeadResultIterMutMapItem",
            "dead_result_iter_mut_map",
            "pub fn render_label",
            "dead_method",
            "dead_error_method",
            "dead-result-iter-mut-map",
        ],
    });
}

#[test]
fn prunes_option_as_slice_iter_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "option_as_slice_iter_prune",
        root_fn: "selected_option_as_slice_iter_report",
        api_pkg: "option_as_slice_iter_api",
        model_pkg: "option_as_slice_iter_model",
        api_fn: "selected_option_as_slice_iter_report",
        model_fn: "selected_option_as_slice_iter",
        model_required: &[
            ".as_slice()",
            ".iter()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadOptionAsSliceIterItem",
            "dead_option_as_slice_iter",
            "pub fn bump_and_render",
            "dead_method",
            "dead-option-as-slice-iter",
        ],
    });
}

#[test]
fn prunes_option_as_mut_slice_iter_mut_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "option_as_mut_slice_iter_mut_prune",
        root_fn: "selected_option_as_mut_slice_iter_mut_report",
        api_pkg: "option_as_mut_slice_iter_mut_api",
        model_pkg: "option_as_mut_slice_iter_mut_model",
        api_fn: "selected_option_as_mut_slice_iter_mut_report",
        model_fn: "selected_option_as_mut_slice_iter_mut",
        model_required: &[
            ".as_mut_slice()",
            ".iter_mut()",
            ".map(|payload| payload.bump_and_render())",
            "pub fn bump_and_render",
        ],
        model_absent: &[
            "mod dead",
            "DeadOptionAsMutSliceIterMutItem",
            "dead_option_as_mut_slice_iter_mut",
            "pub fn render_label",
            "dead_method",
            "dead-option-as-mut-slice-iter-mut",
        ],
    });
}

#[test]
fn prunes_option_replace_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "option_replace_map_prune",
        root_fn: "selected_option_replace_map_report",
        api_pkg: "option_replace_map_api",
        model_pkg: "option_replace_map_model",
        api_fn: "selected_option_replace_map_report",
        model_fn: "selected_option_replace_map",
        model_required: &[
            ".replace(",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadOptionReplaceMapItem",
            "dead_option_replace_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-option-replace-map",
        ],
    });
}

#[test]
fn prunes_btreemap_first_key_value_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "btreemap_first_key_value_prune",
        root_fn: "selected_btreemap_first_key_value_report",
        api_pkg: "btreemap_first_key_value_api",
        model_pkg: "btreemap_first_key_value_model",
        api_fn: "selected_btreemap_first_key_value_report",
        model_fn: "selected_btreemap_first_key_value",
        model_required: &[
            "BTreeMap",
            ".first_key_value()",
            "pub fn render_key",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBtreemapFirstKeyValueItem",
            "dead_btreemap_first_key_value",
            "dead_key_method",
            "dead_method",
            "dead-btreemap-first-key-value",
        ],
    });
}

#[test]
fn prunes_btreemap_last_key_value_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "btreemap_last_key_value_prune",
        root_fn: "selected_btreemap_last_key_value_report",
        api_pkg: "btreemap_last_key_value_api",
        model_pkg: "btreemap_last_key_value_model",
        api_fn: "selected_btreemap_last_key_value_report",
        model_fn: "selected_btreemap_last_key_value",
        model_required: &[
            "BTreeMap",
            ".last_key_value()",
            "pub fn render_key",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBtreemapLastKeyValueItem",
            "dead_btreemap_last_key_value",
            "dead_key_method",
            "dead_method",
            "dead-btreemap-last-key-value",
        ],
    });
}

#[test]
fn prunes_btreemap_pop_first_pair_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "btreemap_pop_first_pair_prune",
        root_fn: "selected_btreemap_pop_first_pair_report",
        api_pkg: "btreemap_pop_first_pair_api",
        model_pkg: "btreemap_pop_first_pair_model",
        api_fn: "selected_btreemap_pop_first_pair_report",
        model_fn: "selected_btreemap_pop_first_pair",
        model_required: &[
            "BTreeMap",
            ".pop_first()",
            "pub fn render_key",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBtreemapPopFirstPairItem",
            "dead_btreemap_pop_first_pair",
            "dead_key_method",
            "dead_method",
            "dead-btreemap-pop-first-pair",
        ],
    });
}

#[test]
fn prunes_btreemap_pop_last_pair_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "btreemap_pop_last_pair_prune",
        root_fn: "selected_btreemap_pop_last_pair_report",
        api_pkg: "btreemap_pop_last_pair_api",
        model_pkg: "btreemap_pop_last_pair_model",
        api_fn: "selected_btreemap_pop_last_pair_report",
        model_fn: "selected_btreemap_pop_last_pair",
        model_required: &[
            "BTreeMap",
            ".pop_last()",
            "pub fn render_key",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBtreemapPopLastPairItem",
            "dead_btreemap_pop_last_pair",
            "dead_key_method",
            "dead_method",
            "dead-btreemap-pop-last-pair",
        ],
    });
}

#[test]
fn prunes_array_from_fn_iter_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "array_from_fn_iter_prune",
        root_fn: "selected_array_from_fn_iter_report",
        api_pkg: "array_from_fn_iter_api",
        model_pkg: "array_from_fn_iter_model",
        api_fn: "selected_array_from_fn_iter_report",
        model_fn: "selected_array_from_fn_iter",
        model_required: &[
            "std::array::from_fn",
            ".iter()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadArrayFromFnIterItem",
            "dead_array_from_fn_iter",
            "pub fn bump_and_render",
            "dead_method",
            "dead-array-from-fn-iter",
        ],
    });
}

#[test]
fn prunes_vec_split_off_into_iter_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "vec_split_off_into_iter_prune",
        root_fn: "selected_vec_split_off_into_iter_report",
        api_pkg: "vec_split_off_into_iter_api",
        model_pkg: "vec_split_off_into_iter_model",
        api_fn: "selected_vec_split_off_into_iter_report",
        model_fn: "selected_vec_split_off_into_iter",
        model_required: &[
            ".split_off(1)",
            ".into_iter()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadVecSplitOffIntoIterItem",
            "dead_vec_split_off_into_iter",
            "pub fn bump_and_render",
            "dead_method",
            "dead-vec-split-off-into-iter",
        ],
    });
}

#[test]
fn prunes_vec_splice_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "vec_splice_map_prune",
        root_fn: "selected_vec_splice_map_report",
        api_pkg: "vec_splice_map_api",
        model_pkg: "vec_splice_map_model",
        api_fn: "selected_vec_splice_map_report",
        model_fn: "selected_vec_splice_map",
        model_required: &[
            ".splice(0..1",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadVecSpliceMapItem",
            "dead_vec_splice_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-vec-splice-map",
        ],
    });
}

#[test]
fn prunes_vec_into_boxed_slice_iter_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "vec_into_boxed_slice_iter_prune",
        root_fn: "selected_vec_into_boxed_slice_iter_report",
        api_pkg: "vec_into_boxed_slice_iter_api",
        model_pkg: "vec_into_boxed_slice_iter_model",
        api_fn: "selected_vec_into_boxed_slice_iter_report",
        model_fn: "selected_vec_into_boxed_slice_iter",
        model_required: &[
            ".into_boxed_slice()",
            ".iter()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadVecIntoBoxedSliceIterItem",
            "dead_vec_into_boxed_slice_iter",
            "pub fn bump_and_render",
            "dead_method",
            "dead-vec-into-boxed-slice-iter",
        ],
    });
}

#[test]
fn prunes_vec_leak_iter_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "vec_leak_iter_prune",
        root_fn: "selected_vec_leak_iter_report",
        api_pkg: "vec_leak_iter_api",
        model_pkg: "vec_leak_iter_model",
        api_fn: "selected_vec_leak_iter_report",
        model_fn: "selected_vec_leak_iter",
        model_required: &[
            ".leak()",
            ".iter()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadVecLeakIterItem",
            "dead_vec_leak_iter",
            "pub fn bump_and_render",
            "dead_method",
            "dead-vec-leak-iter",
        ],
    });
}

#[test]
fn prunes_vec_resize_with_last_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "vec_resize_with_last_prune",
        root_fn: "selected_vec_resize_with_last_report",
        api_pkg: "vec_resize_with_last_api",
        model_pkg: "vec_resize_with_last_model",
        api_fn: "selected_vec_resize_with_last_report",
        model_fn: "selected_vec_resize_with_last",
        model_required: &[
            ".resize_with(2",
            ".last()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadVecResizeWithLastItem",
            "dead_vec_resize_with_last",
            "pub fn bump_and_render",
            "dead_method",
            "dead-vec-resize-with-last",
        ],
    });
}

#[test]
fn prunes_vec_extend_from_slice_iter_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "vec_extend_from_slice_iter_prune",
        root_fn: "selected_vec_extend_from_slice_iter_report",
        api_pkg: "vec_extend_from_slice_iter_api",
        model_pkg: "vec_extend_from_slice_iter_model",
        api_fn: "selected_vec_extend_from_slice_iter_report",
        model_fn: "selected_vec_extend_from_slice_iter",
        model_required: &[
            ".extend_from_slice(&extras)",
            ".iter()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadVecExtendFromSliceIterItem",
            "dead_vec_extend_from_slice_iter",
            "pub fn bump_and_render",
            "dead_method",
            "dead-vec-extend-from-slice-iter",
        ],
    });
}

#[test]
fn prunes_vecdeque_split_off_into_iter_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "vecdeque_split_off_into_iter_prune",
        root_fn: "selected_vecdeque_split_off_into_iter_report",
        api_pkg: "vecdeque_split_off_into_iter_api",
        model_pkg: "vecdeque_split_off_into_iter_model",
        api_fn: "selected_vecdeque_split_off_into_iter_report",
        model_fn: "selected_vecdeque_split_off_into_iter",
        model_required: &[
            "VecDeque",
            ".split_off(1)",
            ".into_iter()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadVecdequeSplitOffIntoIterItem",
            "dead_vecdeque_split_off_into_iter",
            "pub fn bump_and_render",
            "dead_method",
            "dead-vecdeque-split-off-into-iter",
        ],
    });
}

#[test]
fn prunes_linkedlist_split_off_into_iter_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "linkedlist_split_off_into_iter_prune",
        root_fn: "selected_linkedlist_split_off_into_iter_report",
        api_pkg: "linkedlist_split_off_into_iter_api",
        model_pkg: "linkedlist_split_off_into_iter_model",
        api_fn: "selected_linkedlist_split_off_into_iter_report",
        model_fn: "selected_linkedlist_split_off_into_iter",
        model_required: &[
            "LinkedList",
            ".split_off(1)",
            ".into_iter()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadLinkedlistSplitOffIntoIterItem",
            "dead_linkedlist_split_off_into_iter",
            "pub fn bump_and_render",
            "dead_method",
            "dead-linkedlist-split-off-into-iter",
        ],
    });
}

#[test]
fn prunes_btreeset_split_off_into_iter_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "btreeset_split_off_into_iter_prune",
        root_fn: "selected_btreeset_split_off_into_iter_report",
        api_pkg: "btreeset_split_off_into_iter_api",
        model_pkg: "btreeset_split_off_into_iter_model",
        api_fn: "selected_btreeset_split_off_into_iter_report",
        model_fn: "selected_btreeset_split_off_into_iter",
        model_required: &[
            "BTreeSet",
            ".split_off(",
            ".into_iter()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBtreesetSplitOffIntoIterItem",
            "dead_btreeset_split_off_into_iter",
            "pub fn bump_and_render",
            "dead_method",
            "dead-btreeset-split-off-into-iter",
        ],
    });
}

#[test]
fn prunes_btreemap_split_off_into_values_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "btreemap_split_off_into_values_prune",
        root_fn: "selected_btreemap_split_off_into_values_report",
        api_pkg: "btreemap_split_off_into_values_api",
        model_pkg: "btreemap_split_off_into_values_model",
        api_fn: "selected_btreemap_split_off_into_values_report",
        model_fn: "selected_btreemap_split_off_into_values",
        model_required: &[
            "BTreeMap",
            ".split_off(",
            ".into_values()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBtreemapSplitOffIntoValuesItem",
            "dead_btreemap_split_off_into_values",
            "dead_key_method",
            "dead_method",
            "dead-btreemap-split-off-into-values",
        ],
    });
}

#[test]
fn prunes_btreemap_append_values_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "btreemap_append_values_prune",
        root_fn: "selected_btreemap_append_values_report",
        api_pkg: "btreemap_append_values_api",
        model_pkg: "btreemap_append_values_model",
        api_fn: "selected_btreemap_append_values_report",
        model_fn: "selected_btreemap_append_values",
        model_required: &[
            "BTreeMap",
            ".append(&mut extras)",
            ".values()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBtreemapAppendValuesItem",
            "dead_btreemap_append_values",
            "dead_key_method",
            "dead_method",
            "dead-btreemap-append-values",
        ],
    });
}

#[test]
fn prunes_binaryheap_append_into_sorted_vec_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "binaryheap_append_into_sorted_vec_prune",
        root_fn: "selected_binaryheap_append_into_sorted_vec_report",
        api_pkg: "binaryheap_append_into_sorted_vec_api",
        model_pkg: "binaryheap_append_into_sorted_vec_model",
        api_fn: "selected_binaryheap_append_into_sorted_vec_report",
        model_fn: "selected_binaryheap_append_into_sorted_vec",
        model_required: &[
            "BinaryHeap",
            ".append(&mut extras)",
            ".into_sorted_vec()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadBinaryheapAppendIntoSortedVecItem",
            "dead_binaryheap_append_into_sorted_vec",
            "pub fn bump_and_render",
            "dead_method",
            "dead-binaryheap-append-into-sorted-vec",
        ],
    });
}

#[test]
fn prunes_hashset_intersection_support_chain_with_default_analyzer() {
    assert_set_algebra_support_fixture(
        "hashset_intersection_prune",
        "hashset_intersection",
        "HashSet",
        "intersection",
        "DeadHashsetIntersectionItem",
        "dead-hashset-intersection",
    );
}

#[test]
fn prunes_hashset_union_support_chain_with_default_analyzer() {
    assert_set_algebra_support_fixture(
        "hashset_union_prune",
        "hashset_union",
        "HashSet",
        "union",
        "DeadHashsetUnionItem",
        "dead-hashset-union",
    );
}

#[test]
fn prunes_hashset_difference_support_chain_with_default_analyzer() {
    assert_set_algebra_support_fixture(
        "hashset_difference_prune",
        "hashset_difference",
        "HashSet",
        "difference",
        "DeadHashsetDifferenceItem",
        "dead-hashset-difference",
    );
}

#[test]
fn prunes_hashset_symmetric_difference_support_chain_with_default_analyzer() {
    assert_set_algebra_support_fixture(
        "hashset_symmetric_difference_prune",
        "hashset_symmetric_difference",
        "HashSet",
        "symmetric_difference",
        "DeadHashsetSymmetricDifferenceItem",
        "dead-hashset-symmetric-difference",
    );
}

#[test]
fn prunes_btreeset_intersection_support_chain_with_default_analyzer() {
    assert_set_algebra_support_fixture(
        "btreeset_intersection_prune",
        "btreeset_intersection",
        "BTreeSet",
        "intersection",
        "DeadBtreesetIntersectionItem",
        "dead-btreeset-intersection",
    );
}

#[test]
fn prunes_btreeset_union_support_chain_with_default_analyzer() {
    assert_set_algebra_support_fixture(
        "btreeset_union_prune",
        "btreeset_union",
        "BTreeSet",
        "union",
        "DeadBtreesetUnionItem",
        "dead-btreeset-union",
    );
}

#[test]
fn prunes_btreeset_difference_support_chain_with_default_analyzer() {
    assert_set_algebra_support_fixture(
        "btreeset_difference_prune",
        "btreeset_difference",
        "BTreeSet",
        "difference",
        "DeadBtreesetDifferenceItem",
        "dead-btreeset-difference",
    );
}

#[test]
fn prunes_btreeset_symmetric_difference_support_chain_with_default_analyzer() {
    assert_set_algebra_support_fixture(
        "btreeset_symmetric_difference_prune",
        "btreeset_symmetric_difference",
        "BTreeSet",
        "symmetric_difference",
        "DeadBtreesetSymmetricDifferenceItem",
        "dead-btreeset-symmetric-difference",
    );
}

#[test]
fn prunes_hashmap_values_mut_support_chain_with_default_analyzer() {
    assert_mut_map_values_support_fixture(
        "hashmap_values_mut_prune",
        "hashmap_values_mut",
        "HashMap",
        "DeadHashmapValuesMutItem",
        "dead-hashmap-values-mut",
    );
}

#[test]
fn prunes_btreemap_values_mut_support_chain_with_default_analyzer() {
    assert_mut_map_values_support_fixture(
        "btreemap_values_mut_prune",
        "btreemap_values_mut",
        "BTreeMap",
        "DeadBtreemapValuesMutItem",
        "dead-btreemap-values-mut",
    );
}

#[test]
fn prunes_vec_append_iter_support_chain_with_default_analyzer() {
    assert_append_iter_support_fixture(
        "vec_append_iter_prune",
        "vec_append_iter",
        "",
        "DeadVecAppendIterItem",
        "dead-vec-append-iter",
    );
}

#[test]
fn prunes_vecdeque_append_iter_support_chain_with_default_analyzer() {
    assert_append_iter_support_fixture(
        "vecdeque_append_iter_prune",
        "vecdeque_append_iter",
        "VecDeque",
        "DeadVecdequeAppendIterItem",
        "dead-vecdeque-append-iter",
    );
}

#[test]
fn prunes_slice_strip_prefix_iter_support_chain_with_default_analyzer() {
    assert_iter_adapter_support_fixture(
        "slice_strip_prefix_iter_prune",
        "slice_strip_prefix_iter",
        ".strip_prefix(&prefix)",
        "DeadSliceStripPrefixIterItem",
        "dead-slice-strip-prefix-iter",
        Some(".unwrap_or(&payloads)"),
    );
}

#[test]
fn prunes_slice_strip_suffix_iter_support_chain_with_default_analyzer() {
    assert_iter_adapter_support_fixture(
        "slice_strip_suffix_iter_prune",
        "slice_strip_suffix_iter",
        ".strip_suffix(&suffix)",
        "DeadSliceStripSuffixIterItem",
        "dead-slice-strip-suffix-iter",
        Some(".unwrap_or(&payloads)"),
    );
}

#[test]
fn prunes_vec_truncate_iter_support_chain_with_default_analyzer() {
    assert_iter_adapter_support_fixture(
        "vec_truncate_iter_prune",
        "vec_truncate_iter",
        ".truncate(2)",
        "DeadVecTruncateIterItem",
        "dead-vec-truncate-iter",
        None,
    );
}

#[test]
fn prunes_vec_reverse_iter_support_chain_with_default_analyzer() {
    assert_iter_adapter_support_fixture(
        "vec_reverse_iter_prune",
        "vec_reverse_iter",
        ".reverse()",
        "DeadVecReverseIterItem",
        "dead-vec-reverse-iter",
        None,
    );
}

#[test]
fn prunes_vec_rotate_left_iter_support_chain_with_default_analyzer() {
    assert_iter_adapter_support_fixture(
        "vec_rotate_left_iter_prune",
        "vec_rotate_left_iter",
        ".rotate_left(1)",
        "DeadVecRotateLeftIterItem",
        "dead-vec-rotate-left-iter",
        None,
    );
}

#[test]
fn prunes_vec_rotate_right_iter_support_chain_with_default_analyzer() {
    assert_iter_adapter_support_fixture(
        "vec_rotate_right_iter_prune",
        "vec_rotate_right_iter",
        ".rotate_right(1)",
        "DeadVecRotateRightIterItem",
        "dead-vec-rotate-right-iter",
        None,
    );
}

#[test]
fn prunes_vec_swap_iter_support_chain_with_default_analyzer() {
    assert_iter_adapter_support_fixture(
        "vec_swap_iter_prune",
        "vec_swap_iter",
        ".swap(0, 1)",
        "DeadVecSwapIterItem",
        "dead-vec-swap-iter",
        None,
    );
}

#[test]
fn prunes_vec_fill_with_iter_support_chain_with_default_analyzer() {
    assert_iter_adapter_support_fixture(
        "vec_fill_with_iter_prune",
        "vec_fill_with_iter",
        ".fill_with(||",
        "DeadVecFillWithIterItem",
        "dead-vec-fill-with-iter",
        None,
    );
}

#[test]
fn prunes_slice_reverse_iter_support_chain_with_default_analyzer() {
    assert_iter_adapter_support_fixture(
        "slice_reverse_iter_prune",
        "slice_reverse_iter",
        ".reverse()",
        "DeadSliceReverseIterItem",
        "dead-slice-reverse-iter",
        None,
    );
}

#[test]
fn prunes_slice_rotate_left_iter_support_chain_with_default_analyzer() {
    assert_iter_adapter_support_fixture(
        "slice_rotate_left_iter_prune",
        "slice_rotate_left_iter",
        ".rotate_left(1)",
        "DeadSliceRotateLeftIterItem",
        "dead-slice-rotate-left-iter",
        None,
    );
}

#[test]
fn prunes_slice_rotate_right_iter_support_chain_with_default_analyzer() {
    assert_iter_adapter_support_fixture(
        "slice_rotate_right_iter_prune",
        "slice_rotate_right_iter",
        ".rotate_right(1)",
        "DeadSliceRotateRightIterItem",
        "dead-slice-rotate-right-iter",
        None,
    );
}

#[test]
fn prunes_slice_swap_iter_support_chain_with_default_analyzer() {
    assert_iter_adapter_support_fixture(
        "slice_swap_iter_prune",
        "slice_swap_iter",
        ".swap(0, 1)",
        "DeadSliceSwapIterItem",
        "dead-slice-swap-iter",
        None,
    );
}

#[test]
fn prunes_slice_split_first_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "slice_split_first_map_prune",
        "slice_split_first_map",
        &[
            ".split_first()",
            ".map(|(payload, _tail)| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadSliceSplitFirstMapItem",
        "dead-slice-split-first-map",
        "pub fn bump_and_render",
    );
}

#[test]
fn prunes_slice_split_last_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "slice_split_last_map_prune",
        "slice_split_last_map",
        &[
            ".split_last()",
            ".map(|(payload, _tail)| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadSliceSplitLastMapItem",
        "dead-slice-split-last-map",
        "pub fn bump_and_render",
    );
}

#[test]
fn prunes_slice_split_first_mut_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "slice_split_first_mut_map_prune",
        "slice_split_first_mut_map",
        &[
            ".split_first_mut()",
            ".map(|(payload, _tail)| payload.bump_and_render())",
            "pub fn bump_and_render",
        ],
        "DeadSliceSplitFirstMutMapItem",
        "dead-slice-split-first-mut-map",
        "pub fn render_label",
    );
}

#[test]
fn prunes_slice_split_last_mut_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "slice_split_last_mut_map_prune",
        "slice_split_last_mut_map",
        &[
            ".split_last_mut()",
            ".map(|(payload, _tail)| payload.bump_and_render())",
            "pub fn bump_and_render",
        ],
        "DeadSliceSplitLastMutMapItem",
        "dead-slice-split-last-mut-map",
        "pub fn render_label",
    );
}

#[test]
fn prunes_vec_split_first_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "vec_split_first_map_prune",
        "vec_split_first_map",
        &[
            ".split_first()",
            ".map(|(payload, _tail)| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadVecSplitFirstMapItem",
        "dead-vec-split-first-map",
        "pub fn bump_and_render",
    );
}

#[test]
fn prunes_vec_split_last_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "vec_split_last_map_prune",
        "vec_split_last_map",
        &[
            ".split_last()",
            ".map(|(payload, _tail)| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadVecSplitLastMapItem",
        "dead-vec-split-last-map",
        "pub fn bump_and_render",
    );
}

#[test]
fn prunes_vec_split_first_mut_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "vec_split_first_mut_map_prune",
        "vec_split_first_mut_map",
        &[
            ".split_first_mut()",
            ".map(|(payload, _tail)| payload.bump_and_render())",
            "pub fn bump_and_render",
        ],
        "DeadVecSplitFirstMutMapItem",
        "dead-vec-split-first-mut-map",
        "pub fn render_label",
    );
}

#[test]
fn prunes_vec_split_last_mut_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "vec_split_last_mut_map_prune",
        "vec_split_last_mut_map",
        &[
            ".split_last_mut()",
            ".map(|(payload, _tail)| payload.bump_and_render())",
            "pub fn bump_and_render",
        ],
        "DeadVecSplitLastMutMapItem",
        "dead-vec-split-last-mut-map",
        "pub fn render_label",
    );
}

#[test]
fn prunes_slice_split_at_tail_iter_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "slice_split_at_tail_iter_prune",
        "slice_split_at_tail_iter",
        &[
            ".split_at(1)",
            "tail.iter()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadSliceSplitAtTailIterItem",
        "dead-slice-split-at-tail-iter",
        "pub fn bump_and_render",
    );
}

#[test]
fn prunes_slice_split_at_mut_tail_iter_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "slice_split_at_mut_tail_iter_prune",
        "slice_split_at_mut_tail_iter",
        &[
            ".split_at_mut(1)",
            "tail.iter_mut()",
            ".map(|payload| payload.bump_and_render())",
            "pub fn bump_and_render",
        ],
        "DeadSliceSplitAtMutTailIterItem",
        "dead-slice-split-at-mut-tail-iter",
        "pub fn render_label",
    );
}

#[test]
fn prunes_vecdeque_as_slices_iter_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "vecdeque_as_slices_iter_prune",
        "vecdeque_as_slices_iter",
        &[
            "VecDeque",
            ".as_slices()",
            ".iter()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadVecdequeAsSlicesIterItem",
        "dead-vecdeque-as-slices-iter",
        "pub fn bump_and_render",
    );
}

#[test]
fn prunes_vecdeque_as_mut_slices_iter_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "vecdeque_as_mut_slices_iter_prune",
        "vecdeque_as_mut_slices_iter",
        &[
            "VecDeque",
            ".as_mut_slices()",
            ".iter_mut()",
            ".map(|payload| payload.bump_and_render())",
            "pub fn bump_and_render",
        ],
        "DeadVecdequeAsMutSlicesIterItem",
        "dead-vecdeque-as-mut-slices-iter",
        "pub fn render_label",
    );
}

fn assert_set_algebra_support_fixture(
    fixture_name: &str,
    stem: &str,
    collection: &str,
    operation: &str,
    dead_item: &str,
    dead_token: &str,
) {
    let root_fn = format!("selected_{stem}_report");
    let api_pkg = format!("{stem}_api");
    let model_pkg = format!("{stem}_model");
    let api_fn = format!("selected_{stem}_report");
    let model_fn = format!("selected_{stem}");
    let operation_token = format!(".{operation}(&right)");
    let dead_fn = format!("dead_{stem}");
    let model_required = [
        collection,
        operation_token.as_str(),
        ".map(|payload| payload.render_label())",
        "pub fn render_label",
    ];
    let model_absent = [
        "mod dead",
        dead_item,
        dead_fn.as_str(),
        "dead_method",
        dead_token,
    ];

    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name,
        root_fn: root_fn.as_str(),
        api_pkg: api_pkg.as_str(),
        model_pkg: model_pkg.as_str(),
        api_fn: api_fn.as_str(),
        model_fn: model_fn.as_str(),
        model_required: &model_required,
        model_absent: &model_absent,
    });
}

fn assert_iter_adapter_support_fixture(
    fixture_name: &str,
    stem: &str,
    operation: &str,
    dead_item: &str,
    dead_token: &str,
    extra_operation: Option<&str>,
) {
    let root_fn = format!("selected_{stem}_report");
    let api_pkg = format!("{stem}_api");
    let model_pkg = format!("{stem}_model");
    let api_fn = format!("selected_{stem}_report");
    let model_fn = format!("selected_{stem}");
    let dead_fn = format!("dead_{stem}");
    let mut model_required = vec![
        operation,
        ".iter()",
        ".map(|payload| payload.render_label())",
        "pub fn render_label",
    ];
    if let Some(extra_operation) = extra_operation {
        model_required.push(extra_operation);
    }
    let model_absent = [
        "mod dead",
        dead_item,
        dead_fn.as_str(),
        "pub fn bump_and_render",
        "dead_method",
        dead_token,
    ];

    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name,
        root_fn: root_fn.as_str(),
        api_pkg: api_pkg.as_str(),
        model_pkg: model_pkg.as_str(),
        api_fn: api_fn.as_str(),
        model_fn: model_fn.as_str(),
        model_required: &model_required,
        model_absent: &model_absent,
    });
}

fn assert_tuple_adapter_support_fixture(
    fixture_name: &str,
    stem: &str,
    required: &[&str],
    dead_item: &str,
    dead_token: &str,
    absent_live_method: &str,
) {
    let root_fn = format!("selected_{stem}_report");
    let api_pkg = format!("{stem}_api");
    let model_pkg = format!("{stem}_model");
    let api_fn = format!("selected_{stem}_report");
    let model_fn = format!("selected_{stem}");
    let dead_fn = format!("dead_{stem}");
    let model_absent = [
        "mod dead",
        dead_item,
        dead_fn.as_str(),
        absent_live_method,
        "dead_method",
        dead_token,
    ];

    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name,
        root_fn: root_fn.as_str(),
        api_pkg: api_pkg.as_str(),
        model_pkg: model_pkg.as_str(),
        api_fn: api_fn.as_str(),
        model_fn: model_fn.as_str(),
        model_required: required,
        model_absent: &model_absent,
    });
}

fn assert_mut_map_values_support_fixture(
    fixture_name: &str,
    stem: &str,
    collection: &str,
    dead_item: &str,
    dead_token: &str,
) {
    let root_fn = format!("selected_{stem}_report");
    let api_pkg = format!("{stem}_api");
    let model_pkg = format!("{stem}_model");
    let api_fn = format!("selected_{stem}_report");
    let model_fn = format!("selected_{stem}");
    let dead_fn = format!("dead_{stem}");
    let model_required = [
        collection,
        ".values_mut()",
        ".map(|payload| payload.bump_and_render())",
        "pub fn bump_and_render",
    ];
    let model_absent = [
        "mod dead",
        dead_item,
        dead_fn.as_str(),
        "pub fn render_label",
        "dead_key_method",
        "dead_method",
        dead_token,
    ];

    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name,
        root_fn: root_fn.as_str(),
        api_pkg: api_pkg.as_str(),
        model_pkg: model_pkg.as_str(),
        api_fn: api_fn.as_str(),
        model_fn: model_fn.as_str(),
        model_required: &model_required,
        model_absent: &model_absent,
    });
}

fn assert_append_iter_support_fixture(
    fixture_name: &str,
    stem: &str,
    collection: &str,
    dead_item: &str,
    dead_token: &str,
) {
    let root_fn = format!("selected_{stem}_report");
    let api_pkg = format!("{stem}_api");
    let model_pkg = format!("{stem}_model");
    let api_fn = format!("selected_{stem}_report");
    let model_fn = format!("selected_{stem}");
    let dead_fn = format!("dead_{stem}");
    let mut required = vec![
        ".append(&mut extras)",
        ".iter()",
        ".map(|payload| payload.render_label())",
        "pub fn render_label",
    ];
    if !collection.is_empty() {
        required.push(collection);
    }
    let model_absent = [
        "mod dead",
        dead_item,
        dead_fn.as_str(),
        "pub fn bump_and_render",
        "dead_method",
        dead_token,
    ];

    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name,
        root_fn: root_fn.as_str(),
        api_pkg: api_pkg.as_str(),
        model_pkg: model_pkg.as_str(),
        api_fn: api_fn.as_str(),
        model_fn: model_fn.as_str(),
        model_required: &required,
        model_absent: &model_absent,
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
