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
