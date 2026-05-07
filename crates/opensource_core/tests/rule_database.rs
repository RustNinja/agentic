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
fn prunes_public_glob_reexport_when_source_module_is_removed() {
    let workspace = temp_path("rule-dead-public-glob-workspace");
    let output = temp_path("rule-dead-public-glob-output");
    let target_dir = temp_path("rule-dead-public-glob-target");
    write_dead_public_glob_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("dead public glob rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("dead_public_glob_rule/src/lib.rs"));
    assert!(lib.contains("pub mod upstream"), "{lib}");
    assert!(!lib.contains("pub mod dead_api"), "{lib}");
    assert!(!lib.contains("pub use dead_api::*"), "{lib}");
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
fn retains_renamed_local_type_and_trait_imports_used_only_by_surfaces() {
    let workspace = temp_path("rule-renamed-surface-workspace");
    let output = temp_path("rule-renamed-surface-output");
    let target_dir = temp_path("rule-renamed-surface-target");
    write_renamed_surface_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("renamed surface rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("renamed_surface_rule/src/lib.rs"));
    assert!(lib.contains("LongName as PublicAlias"), "{lib}");
    assert!(lib.contains("LongTrait as TraitAlias"), "{lib}");
    assert!(lib.contains("pub struct LongName"), "{lib}");
    assert!(lib.contains("pub trait LongTrait"), "{lib}");
    assert!(lib.contains("impl TraitAlias for ApiSurface"), "{lib}");
    assert!(!lib.contains("DeadName as DeadAlias"), "{lib}");
    assert!(!lib.contains("pub struct DeadName"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_renamed_dependency_barrel_reexport_targets() {
    let workspace = temp_path("rule-renamed-dependency-barrel-workspace");
    let output = temp_path("rule-renamed-dependency-barrel-output");
    let target_dir = temp_path("rule-renamed-dependency-barrel-target");
    write_renamed_dependency_barrel_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("renamed dependency barrel rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let app = read(output.join("dependency_barrel_rule/src/lib.rs"));
    assert!(app.contains("SharedRecord as BarrelRecord"), "{app}");
    assert!(app.contains("SharedMode as BarrelMode"), "{app}");
    assert!(
        app.contains("use super::records::{BarrelMode, BarrelRecord}"),
        "{app}"
    );
    assert!(!app.contains("dead_shared_barrel"), "{app}");
    assert!(!app.contains("dead_barrel_helper"), "{app}");
    assert!(!app.contains("dead_helper"), "{app}");

    let shared = read(output.join("shared_rule/src/lib.rs"));
    assert!(shared.contains("pub struct SharedRecord"), "{shared}");
    assert!(shared.contains("pub enum SharedMode"), "{shared}");
    assert!(!shared.contains("pub struct DeadRecord"), "{shared}");
    assert!(!shared.contains("pub fn dead_shared"), "{shared}");
    assert_cargo_check(&output, &target_dir, &app);
}

#[test]
fn retains_multi_hop_dependency_barrel_reexports_without_dead_support_code() {
    let workspace = temp_path("rule-multi-hop-dependency-barrel-workspace");
    let output = temp_path("rule-multi-hop-dependency-barrel-output");
    let target_dir = temp_path("rule-multi-hop-dependency-barrel-target");
    write_multi_hop_dependency_barrel_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("multi-hop dependency barrel rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let app = read(output.join("multi_hop_barrel_app/src/lib.rs"));
    assert!(app.contains("LiveRecord as Record"), "{app}");
    assert!(app.contains("LiveMode as Mode"), "{app}");
    assert!(app.contains("score"), "{app}");
    assert!(!app.contains("DeadRecord"), "{app}");
    assert!(!app.contains("dead_score"), "{app}");
    assert!(!app.contains("dead_leaf_value"), "{app}");

    let support = read(output.join("barrel_support/src/lib.rs"));
    assert!(support.contains("LeafRecord as LiveRecord"), "{support}");
    assert!(support.contains("LeafMode as LiveMode"), "{support}");
    assert!(support.contains("pub fn score"), "{support}");
    assert!(!support.contains("DeadLeaf"), "{support}");
    assert!(!support.contains("dead_score"), "{support}");
    assert!(!support.contains("dead_support_root"), "{support}");
    assert!(!support.contains("dead_leaf_value"), "{support}");

    let leaf = read(output.join("barrel_leaf/src/lib.rs"));
    assert!(leaf.contains("pub struct LeafRecord"), "{leaf}");
    assert!(leaf.contains("pub enum LeafMode"), "{leaf}");
    assert!(!leaf.contains("pub struct DeadLeaf"), "{leaf}");
    assert!(!leaf.contains("pub fn dead_leaf_value"), "{leaf}");
    assert_cargo_check(&output, &target_dir, &app);
}

#[test]
fn prunes_module_scoped_imports_used_only_by_dead_items() {
    let workspace = temp_path("rule-module-import-liveness-workspace");
    let output = temp_path("rule-module-import-liveness-output");
    let target_dir = temp_path("rule-module-import-liveness-target");
    write_module_scoped_import_liveness_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("module import liveness rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("module_import_liveness_rule/src/lib.rs"));
    assert_eq!(lib.matches("use serde::Serialize").count(), 1, "{lib}");
    assert!(!lib.contains("DeserializeOwned"), "{lib}");
    assert!(!lib.contains("thiserror::Error"), "{lib}");
    assert!(lib.contains("#[derive(Serialize)]"), "{lib}");
    assert!(lib.contains("pub fn live_client"), "{lib}");
    assert!(!lib.contains("dead_client"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn prunes_thiserror_import_when_only_serde_json_error_paths_remain() {
    let workspace = temp_path("rule-thiserror-import-liveness-workspace");
    let output = temp_path("rule-thiserror-import-liveness-output");
    let target_dir = temp_path("rule-thiserror-import-liveness-target");
    write_thiserror_import_liveness_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("thiserror import liveness rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("thiserror_import_liveness_rule/src/lib.rs"));
    assert!(lib.contains("DeserializeOwned"), "{lib}");
    assert!(lib.contains("serde_json::Error"), "{lib}");
    assert!(
        lib.contains("#[derive(Debug, Clone, Deserialize)]"),
        "{lib}"
    );
    assert!(!lib.contains("thiserror::Error"), "{lib}");
    assert!(!lib.contains("DeadWireError"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn prunes_grouped_external_imports_after_private_field_pruning() {
    let workspace = temp_path("rule-private-field-import-liveness-workspace");
    let output = temp_path("rule-private-field-import-liveness-output");
    let target_dir = temp_path("rule-private-field-import-liveness-target");
    write_private_field_import_liveness_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("private field import liveness rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("private_field_import_liveness_rule/src/lib.rs"));
    assert!(lib.contains("use std::sync::Arc"), "{lib}");
    assert!(lib.contains("use tokio::sync::watch"), "{lib}");
    assert!(!lib.contains("RwLock"), "{lib}");
    assert!(!lib.contains("Handler"), "{lib}");
    assert!(lib.contains("pub fn selected"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn prunes_private_fields_mentioned_only_by_other_item_surfaces() {
    let workspace = temp_path("rule-private-field-module-mention-workspace");
    let output = temp_path("rule-private-field-module-mention-output");
    let target_dir = temp_path("rule-private-field-module-mention-target");
    write_private_field_module_mention_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("private field module mention rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("private_field_module_mention_rule/src/lib.rs"));
    assert!(lib.contains("pub enum HandoffAction"), "{lib}");
    assert!(
        lib.contains("SendTurn { transcript: String }"),
        "public enum surface should still keep its transcript field:\n{lib}",
    );
    assert!(!lib.contains("TranscriptBuffer"), "{lib}");
    assert!(!lib.contains("transcript: TranscriptBuffer"), "{lib}");
    assert!(lib.contains("action_queue: Vec<HandoffAction>"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_trait_default_methods_and_impl_associated_consts() {
    let workspace = temp_path("rule-default-trait-workspace");
    let output = temp_path("rule-default-trait-output");
    let target_dir = temp_path("rule-default-trait-target");
    write_default_trait_method_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("default trait method rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("default_trait_rule/src/lib.rs"));
    assert!(lib.contains("pub trait Fragment"), "{lib}");
    assert!(lib.contains("const KIND: &'static str"), "{lib}");
    assert!(lib.contains("fn render(&self) -> &'static str"), "{lib}");
    assert!(lib.contains("impl Fragment for EnvFragment"), "{lib}");
    assert!(lib.contains("const KIND: &'static str = \"env\""), "{lib}");
    assert!(!lib.contains("DeadFragment"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn reports_inline_callback_future_fields_as_dynamic_hazards() {
    let workspace = temp_path("rule-inline-callback-future-workspace");
    let output = temp_path("rule-inline-callback-future-output");
    write_inline_callback_future_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("inline callback future rule should reduce");

    assert_eq!(report.production.status, "requires_feedback");
    let trait_object = report
        .production
        .hazards
        .iter()
        .find(|hazard| hazard.code == "trait_object_surfaces" && hazard.severity == "warning")
        .expect("inline callback future should report trait object hazards");
    assert!(trait_object
        .details
        .iter()
        .any(|detail| detail.subject.contains("dyn Fn")));
    assert!(trait_object
        .details
        .iter()
        .any(|detail| detail.subject.contains("dyn Future")));

    let lib = read(output.join("inline_callback_future_rule/src/lib.rs"));
    let compact_lib = lib.split_whitespace().collect::<String>();
    assert!(
        compact_lib.contains("Pin<Box<dynFuture<Output=bool>+Send>>"),
        "{lib}"
    );
    assert!(compact_lib.contains("Arc<dynFn(&str)"), "{lib}");
    assert!(!lib.contains("DeadInlineCallback"), "{lib}");
}

#[test]
fn retains_lazy_lock_static_initializer_closure_dependencies() {
    let workspace = temp_path("rule-lazy-lock-workspace");
    let output = temp_path("rule-lazy-lock-output");
    let target_dir = temp_path("rule-lazy-lock-target");
    write_lazy_lock_static_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("lazy lock static rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("lazy_lock_rule/src/lib.rs"));
    assert!(lib.contains("static LIVE"), "{lib}");
    assert!(lib.contains("LazyLock::new(|| build_live())"), "{lib}");
    assert!(lib.contains("fn build_live() -> String"), "{lib}");
    assert!(!lib.contains("dead_build"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_once_lock_get_or_init_closure_dependencies() {
    let workspace = temp_path("rule-once-lock-workspace");
    let output = temp_path("rule-once-lock-output");
    let target_dir = temp_path("rule-once-lock-target");
    write_once_lock_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("once lock rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("once_lock_rule/src/lib.rs"));
    assert!(
        lib.contains("static CLIENT: OnceLock<Arc<Client>>"),
        "{lib}"
    );
    assert!(lib.contains("fn ensure_init"), "{lib}");
    assert!(lib.contains("Client::new()"), "{lib}");
    assert!(!lib.contains("dead_client"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_let_else_slice_pattern_enum_variants() {
    let workspace = temp_path("rule-let-else-slice-workspace");
    let output = temp_path("rule-let-else-slice-output");
    let target_dir = temp_path("rule-let-else-slice-target");
    write_let_else_slice_pattern_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("let-else slice pattern rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("let_else_slice_rule/src/lib.rs"));
    assert!(lib.contains("Live(u32)"), "{lib}");
    assert!(!lib.contains("dead_helper"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_const_to_const_dependencies_and_array_lengths() {
    let workspace = temp_path("rule-const-chain-workspace");
    let output = temp_path("rule-const-chain-output");
    let target_dir = temp_path("rule-const-chain-target");
    write_const_chain_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("const chain rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("const_chain_rule/src/lib.rs"));
    assert!(lib.contains("pub const ROWS: u32 = 2"), "{lib}");
    assert!(lib.contains("pub const WIDTH: u32 = 10"), "{lib}");
    assert!(lib.contains("pub const AREA: u32 = ROWS * WIDTH"), "{lib}");
    assert!(lib.contains("pub const NAMES"), "{lib}");
    assert!(!lib.contains("DEAD_CONST"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_format_capture_const_dependencies() {
    let workspace = temp_path("rule-format-capture-const-workspace");
    let output = temp_path("rule-format-capture-const-output");
    let target_dir = temp_path("rule-format-capture-const-target");
    write_format_capture_const_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("format capture const rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("format_capture_const_rule/src/lib.rs"));
    assert!(lib.contains("LIVE_PREFIX"), "{lib}");
    assert!(lib.contains("format!(\"{LIVE_PREFIX}:{value}\")"), "{lib}");
    assert!(!lib.contains("DEAD_PREFIX"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_macro_metavariable_enum_variant_paths() {
    let workspace = temp_path("rule-macro-variant-workspace");
    let output = temp_path("rule-macro-variant-output");
    let target_dir = temp_path("rule-macro-variant-target");
    write_macro_variant_path_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("macro variant path rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("macro_variant_rule/src/lib.rs"));
    assert!(lib.contains("macro_rules! req"), "{lib}");
    assert!(lib.contains("Ping { id: u32 }"), "{lib}");
    assert!(lib.contains("fn next_id() -> u32"), "{lib}");
    assert!(!lib.contains("dead_id"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_receiver_methods_inside_expression_macro_arguments() {
    let workspace = temp_path("rule-expression-macro-receiver-workspace");
    let output = temp_path("rule-expression-macro-receiver-output");
    let target_dir = temp_path("rule-expression-macro-receiver-target");
    write_expression_macro_receiver_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("expression macro receiver rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("expression_macro_receiver_rule/src/lib.rs"));
    assert!(lib.contains("macro_rules! passthrough"), "{lib}");
    assert!(lib.contains("fn score(&self) -> u32"), "{lib}");
    assert!(
        lib.contains("passthrough!(make_payload().score())"),
        "{lib}"
    );
    assert!(!lib.contains("dead_score"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_serde_serialize_and_deserialize_helper_paths_from_attrs() {
    let workspace = temp_path("rule-serde-hook-workspace");
    let output = temp_path("rule-serde-hook-output");
    let target_dir = temp_path("rule-serde-hook-target");
    write_serde_hook_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("serde hook rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("serde_hook_rule/src/lib.rs"));
    assert!(lib.contains("fn ser_live"), "{lib}");
    assert!(lib.contains("fn de_live"), "{lib}");
    assert!(lib.contains("serialize_with = \"ser_live\""), "{lib}");
    assert!(lib.contains("deserialize_with = \"de_live\""), "{lib}");
    assert!(!lib.contains("dead_hook"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_serde_default_function_paths_from_attrs() {
    let workspace = temp_path("rule-serde-default-fn-workspace");
    let output = temp_path("rule-serde-default-fn-output");
    let target_dir = temp_path("rule-serde-default-fn-target");
    write_serde_default_function_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("serde default function rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("serde_default_fn_rule/src/lib.rs"));
    assert!(lib.contains("fn default_mode() -> Mode"), "{lib}");
    assert!(lib.contains("default = \"default_mode\""), "{lib}");
    assert!(!lib.contains("dead_default"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_serde_with_module_helpers_from_attrs() {
    let workspace = temp_path("rule-serde-with-module-workspace");
    let output = temp_path("rule-serde-with-module-output");
    let target_dir = temp_path("rule-serde-with-module-target");
    write_serde_with_module_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("serde with module rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("serde_with_module_rule/src/lib.rs"));
    assert!(lib.contains("#[serde(with = \"codec\")]"), "{lib}");
    assert!(lib.contains("mod codec"), "{lib}");
    assert!(lib.contains("pub fn serialize"), "{lib}");
    assert!(lib.contains("pub fn deserialize"), "{lib}");
    assert!(!lib.contains("dead_codec"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn reports_async_trait_object_surfaces_with_macro_attr() {
    let workspace = temp_path("rule-async-trait-workspace");
    let output = temp_path("rule-async-trait-output");
    let target_dir = temp_path("rule-async-trait-target");
    write_async_trait_object_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("async trait object rule should reduce");

    assert_eq!(report.production.status, "requires_feedback");
    assert!(report.production.hazards.iter().any(|hazard| {
        hazard.code == "trait_object_surfaces"
            && hazard
                .details
                .iter()
                .any(|detail| detail.subject.contains("dyn Transport"))
    }));
    let lib = read(output.join("async_trait_rule/src/lib.rs"));
    assert!(lib.contains("pub trait Transport: Send + Sync"), "{lib}");
    assert!(!lib.contains("#[async_trait::async_trait]"), "{lib}");
    assert!(!lib.contains("async fn reconnect"), "{lib}");
    assert!(!lib.contains("DeadTransport"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_foreign_extern_functions_called_by_selected_roots() {
    let workspace = temp_path("rule-ffi-extern-workspace");
    let output = temp_path("rule-ffi-extern-output");
    let target_dir = temp_path("rule-ffi-extern-target");
    write_ffi_extern_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("ffi extern rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("ffi_extern_rule/src/lib.rs"));
    assert!(lib.contains("extern \"C\""), "{lib}");
    assert!(lib.contains("fn live_c() -> i32"), "{lib}");
    assert!(!lib.contains("dead_c"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn proves_returned_trait_object_surfaces_with_concrete_impls() {
    let workspace = temp_path("rule-returned-dyn-workspace");
    let output = temp_path("rule-returned-dyn-output");
    let target_dir = temp_path("rule-returned-dyn-target");
    write_returned_trait_object_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("returned dyn rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    assert!(!report
        .production
        .hazards
        .iter()
        .any(|hazard| hazard.code == "trait_object_surfaces"));
    let lib = read(output.join("returned_dyn_rule/src/lib.rs"));
    assert!(lib.contains("Box<dyn Reader>"), "{lib}");
    assert!(lib.contains("pub trait Reader"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn reports_forwarded_trait_object_returns_without_concrete_proof() {
    let workspace = temp_path("rule-forwarded-dyn-workspace");
    let output = temp_path("rule-forwarded-dyn-output");
    let target_dir = temp_path("rule-forwarded-dyn-target");
    write_forwarded_trait_object_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("forwarded dyn rule should reduce");

    assert_eq!(report.production.status, "requires_feedback");
    assert!(report.production.hazards.iter().any(|hazard| {
        hazard.code == "trait_object_surfaces"
            && hazard
                .details
                .iter()
                .any(|detail| detail.subject.contains("dyn Reader"))
    }));
    let lib = read(output.join("forwarded_dyn_rule/src/lib.rs"));
    assert!(lib.contains("Box<dyn Reader>"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_foreign_extern_static_symbols_called_by_selected_roots() {
    let workspace = temp_path("rule-ffi-static-workspace");
    let output = temp_path("rule-ffi-static-output");
    let target_dir = temp_path("rule-ffi-static-target");
    write_ffi_static_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("ffi static rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("ffi_static_rule/src/lib.rs"));
    assert!(lib.contains("static LIVE_FLAG: c_int"), "{lib}");
    assert!(!lib.contains("DEAD_FLAG"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn reports_ffi_callback_inputs_as_direct_boundary_warnings() {
    let workspace = temp_path("rule-ffi-callback-workspace");
    let output = temp_path("rule-ffi-callback-output");
    let target_dir = temp_path("rule-ffi-callback-target");
    write_ffi_callback_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("ffi callback rule should reduce");

    assert!(report.production.hazards.iter().any(|hazard| {
        hazard.code == "dynamic_callback_boundaries"
            && hazard.severity == "warning"
            && hazard
                .details
                .iter()
                .any(|detail| detail.subject.contains("extern \"C\" fn"))
    }));
    assert!(!report
        .production
        .hazards
        .iter()
        .any(|hazard| hazard.code == "function_pointer_surfaces" && hazard.severity == "error"));
    let lib = read(output.join("ffi_callback_rule/src/lib.rs"));
    assert!(lib.contains("extern \"C\" fn(*const c_char)"), "{lib}");
    assert!(!lib.contains("dead_register"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_zero_arg_macro_invocations_that_generate_reachable_items() {
    let workspace = temp_path("rule-zero-arg-macro-workspace");
    let output = temp_path("rule-zero-arg-macro-output");
    let target_dir = temp_path("rule-zero-arg-macro-target");
    write_zero_arg_macro_generated_item_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("zero-arg macro generated item rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("zero_arg_macro_rule/src/lib.rs"));
    assert!(lib.contains("macro_rules! setup_runtime"), "{lib}");
    assert!(lib.contains("setup_runtime!()"), "{lib}");
    assert!(lib.contains("fn helper_value() -> u32"), "{lib}");
    assert!(!lib.contains("setup_dead!()"), "{lib}");
    assert!(!lib.contains("dead_generated"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_external_source_trait_imports_by_actual_trait_methods() {
    let workspace = temp_path("rule-external-trait-import-workspace");
    let output = temp_path("rule-external-trait-import-output");
    let target_dir = temp_path("rule-external-trait-import-target");
    let support = temp_path("rule-external-trait-import-support");
    write_external_trait_import_rule_fixture(&workspace, &support);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("external trait import rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("external_trait_import_rule/src/lib.rs"));
    assert!(lib.contains("use support_rng::{Fixed, Rng}"), "{lib}");
    assert!(lib.contains("rng.gen()"), "{lib}");
    assert!(!lib.contains("DeadRng"), "{lib}");
    fs::rename(&support, support.with_extension("moved"))
        .expect("original support package should move away");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_derive_trait_impls_for_array_field_types() {
    let workspace = temp_path("rule-derive-array-workspace");
    let output = temp_path("rule-derive-array-output");
    let target_dir = temp_path("rule-derive-array-target");
    write_derive_array_field_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("derive array field rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("derive_array_rule/src/lib.rs"));
    assert!(lib.contains("#[derive(Clone)]"), "{lib}");
    assert!(lib.contains("pub items: [Inner; 1]"), "{lib}");
    assert!(lib.contains("impl Clone for Inner"), "{lib}");
    assert!(!lib.contains("DeadInner"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn propagates_associated_type_equality_bounds_into_method_chains() {
    let workspace = temp_path("rule-associated-equality-workspace");
    let output = temp_path("rule-associated-equality-output");
    let target_dir = temp_path("rule-associated-equality-target");
    write_associated_type_equality_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("associated type equality rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("associated_equality_rule/src/lib.rs"));
    assert!(lib.contains("type Item"), "{lib}");
    assert!(lib.contains("Source<Item = Payload>"), "{lib}");
    assert!(lib.contains("impl Payload"), "{lib}");
    assert!(!lib.contains("pub struct Other"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn infers_tuple_destructuring_local_receiver_types() {
    let workspace = temp_path("rule-tuple-destructure-workspace");
    let output = temp_path("rule-tuple-destructure-output");
    let target_dir = temp_path("rule-tuple-destructure-target");
    write_tuple_destructure_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("tuple destructuring rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("tuple_destructure_rule/src/lib.rs"));
    assert!(lib.contains("fn pair() -> (Payload, u32)"), "{lib}");
    assert!(lib.contains("impl Payload"), "{lib}");
    assert!(!lib.contains("pub struct Other"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn infers_struct_destructuring_local_receiver_types() {
    let workspace = temp_path("rule-struct-destructure-workspace");
    let output = temp_path("rule-struct-destructure-output");
    let target_dir = temp_path("rule-struct-destructure-target");
    write_struct_destructure_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("struct destructuring rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("struct_destructure_rule/src/lib.rs"));
    assert!(lib.contains("fn bundle() -> Bundle"), "{lib}");
    assert!(lib.contains("impl Payload"), "{lib}");
    assert!(!lib.contains("pub struct Other"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn infers_typed_function_parameter_destructuring_receiver_types() {
    let workspace = temp_path("rule-param-destructure-workspace");
    let output = temp_path("rule-param-destructure-output");
    let target_dir = temp_path("rule-param-destructure-target");
    write_param_destructure_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("parameter destructuring rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("param_destructure_rule/src/lib.rs"));
    assert!(lib.contains("impl Payload"), "{lib}");
    assert!(lib.contains("(payload, _): (Payload, u32)"), "{lib}");
    assert!(!lib.contains("pub struct Other"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn infers_for_loop_item_receiver_types_from_iterable_outputs() {
    let workspace = temp_path("rule-for-loop-item-workspace");
    let output = temp_path("rule-for-loop-item-output");
    let target_dir = temp_path("rule-for-loop-item-target");
    write_for_loop_item_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("for-loop item rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("for_loop_item_rule/src/lib.rs"));
    assert!(lib.contains("fn items() -> Vec<Payload>"), "{lib}");
    assert!(lib.contains("impl Payload"), "{lib}");
    assert!(!lib.contains("pub struct Other"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn types_free_function_closure_arguments_from_callable_inputs() {
    let workspace = temp_path("rule-free-closure-workspace");
    let output = temp_path("rule-free-closure-output");
    let target_dir = temp_path("rule-free-closure-target");
    write_free_function_closure_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("free function closure rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("free_closure_rule/src/lib.rs"));
    assert!(lib.contains("fn with_payload"), "{lib}");
    assert!(lib.contains("impl Payload"), "{lib}");
    assert!(!lib.contains("pub struct Other"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn infers_struct_match_pattern_receiver_types() {
    let workspace = temp_path("rule-struct-match-workspace");
    let output = temp_path("rule-struct-match-output");
    let target_dir = temp_path("rule-struct-match-target");
    write_struct_match_pattern_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("struct match pattern rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("struct_match_rule/src/lib.rs"));
    assert!(lib.contains("fn envelope() -> Envelope"), "{lib}");
    assert!(lib.contains("impl Payload"), "{lib}");
    assert!(!lib.contains("pub struct Other"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn types_option_map_payload_closures_from_receiver_arguments() {
    let workspace = temp_path("rule-option-map-workspace");
    let output = temp_path("rule-option-map-output");
    let target_dir = temp_path("rule-option-map-target");
    write_option_map_payload_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("option map payload rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("option_map_rule/src/lib.rs"));
    assert!(
        lib.contains("fn maybe_payload() -> Option<Payload>"),
        "{lib}"
    );
    assert!(lib.contains("impl Payload"), "{lib}");
    assert!(!lib.contains("pub struct Other"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn types_result_map_err_payload_closures_from_receiver_arguments() {
    let workspace = temp_path("rule-result-map-err-workspace");
    let output = temp_path("rule-result-map-err-output");
    let target_dir = temp_path("rule-result-map-err-target");
    write_result_map_err_payload_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("result map_err payload rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("result_map_err_rule/src/lib.rs"));
    assert!(
        lib.contains("fn maybe_payload() -> Result<u32, Payload>"),
        "{lib}"
    );
    assert!(lib.contains("impl Payload"), "{lib}");
    assert!(!lib.contains("pub struct Other"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_local_result_alias_named_like_prelude_type() {
    let workspace = temp_path("rule-local-result-alias-workspace");
    let output = temp_path("rule-local-result-alias-output");
    let target_dir = temp_path("rule-local-result-alias-target");
    write_local_result_alias_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("local result alias rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("local_result_alias_rule/src/lib.rs"));
    assert!(lib.contains("type Result<T>"), "{lib}");
    assert!(lib.contains("pub struct LocalError"), "{lib}");
    assert!(!lib.contains("DeadResult"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn lets_local_map_methods_drive_closure_payload_types() {
    let workspace = temp_path("rule-local-map-workspace");
    let output = temp_path("rule-local-map-output");
    let target_dir = temp_path("rule-local-map-target");
    write_local_map_method_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("local map method rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("local_map_rule/src/lib.rs"));
    assert!(lib.contains("pub struct Pipe<T>(T)"), "{lib}");
    assert!(lib.contains("impl<T> Pipe<T>"), "{lib}");
    assert!(lib.contains("impl Payload"), "{lib}");
    assert!(!lib.contains("pub struct Other"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn types_is_some_and_payloads_from_local_option_bindings() {
    let workspace = temp_path("rule-is-some-and-workspace");
    let output = temp_path("rule-is-some-and-output");
    let target_dir = temp_path("rule-is-some-and-target");
    write_is_some_and_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("is_some_and rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("is_some_and_rule/src/lib.rs"));
    assert!(lib.contains("let maybe: Option<Payload>"), "{lib}");
    assert!(lib.contains("impl Payload"), "{lib}");
    assert!(!lib.contains("pub struct Other"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn types_is_some_and_payloads_from_field_option_arc_bindings() {
    let workspace = temp_path("rule-field-option-arc-workspace");
    let output = temp_path("rule-field-option-arc-output");
    let target_dir = temp_path("rule-field-option-arc-target");
    write_field_option_arc_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("field Option<Arc<T>> payload rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("field_option_arc_rule/src/lib.rs"));
    assert!(lib.contains("client: Option<Arc<Client>>"), "{lib}");
    assert!(lib.contains("pub fn is_connected(&self) -> bool"), "{lib}");
    assert!(!lib.contains("pub fn dead(&self) -> bool"), "{lib}");
    assert!(!lib.contains("pub struct UnusedClient"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_dependencies_from_rendered_trait_impl_surfaces() {
    let workspace = temp_path("rule-trait-impl-surface-workspace");
    let output = temp_path("rule-trait-impl-surface-output");
    let target_dir = temp_path("rule-trait-impl-surface-target");
    write_trait_impl_surface_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("trait impl surface rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("trait_impl_surface_rule/src/lib.rs"));
    assert!(lib.contains("pub trait Marker"), "{lib}");
    assert!(lib.contains("pub struct Payload"), "{lib}");
    assert!(lib.contains("impl Marker for Api"), "{lib}");
    assert!(lib.contains("fn marker(&self) -> Self::Item"), "{lib}");
    assert!(!lib.contains("fn dead"), "{lib}");
    assert!(!lib.contains("pub struct Dead"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn prunes_trait_peer_impls_for_unrelated_receiver_types() {
    let workspace = temp_path("rule-trait-peer-receiver-workspace");
    let output = temp_path("rule-trait-peer-receiver-output");
    let target_dir = temp_path("rule-trait-peer-receiver-target");
    write_trait_peer_receiver_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("trait peer receiver rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("trait_peer_receiver_rule/src/lib.rs"));
    assert!(lib.contains("pub trait Work"), "{lib}");
    assert!(lib.contains("impl Work for Live"), "{lib}");
    assert!(!lib.contains("impl Work for Decoy"), "{lib}");
    assert!(!lib.contains("pub struct Decoy"), "{lib}");
    assert!(!lib.contains("pub struct DeadWork"), "{lib}");
    assert!(!lib.contains("dead_helper"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn preserves_serde_flatten_contract_fields_while_pruning_dead_private_fields() {
    let workspace = temp_path("rule-serde-flatten-workspace");
    let output = temp_path("rule-serde-flatten-output");
    let target_dir = temp_path("rule-serde-flatten-target");
    write_serde_flatten_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("serde flatten rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("serde_flatten_rule/src/lib.rs"));
    assert!(lib.contains("#[serde(flatten)]"), "{lib}");
    assert!(
        lib.contains("extra: BTreeMap<String, serde_json::Value>"),
        "{lib}"
    );
    assert!(!lib.contains("dead: Option<String>"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_nested_serde_flatten_payload_types() {
    let workspace = temp_path("rule-serde-flatten-nested-workspace");
    let output = temp_path("rule-serde-flatten-nested-output");
    let target_dir = temp_path("rule-serde-flatten-nested-target");
    write_serde_flatten_nested_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("serde flatten nested rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("serde_flatten_nested_rule/src/lib.rs"));
    assert!(lib.contains("nested: Nested"), "{lib}");
    assert!(lib.contains("pub struct Nested"), "{lib}");
    assert!(!lib.contains("DeadNested"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_serde_deserialize_with_private_wire_helpers() {
    let workspace = temp_path("rule-serde-deserialize-with-workspace");
    let output = temp_path("rule-serde-deserialize-with-output");
    let target_dir = temp_path("rule-serde-deserialize-with-target");
    write_serde_deserialize_with_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("serde deserialize_with rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("serde_deserialize_with_rule/src/lib.rs"));
    assert!(lib.contains("struct RawDto"), "{lib}");
    assert!(lib.contains("deserialize_with = \"parse_opt\""), "{lib}");
    assert!(lib.contains("fn parse_opt"), "{lib}");
    assert!(!lib.contains("struct DeadWire"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_serde_alias_private_wire_contract_fields() {
    let workspace = temp_path("rule-serde-alias-wire-workspace");
    let output = temp_path("rule-serde-alias-wire-output");
    let target_dir = temp_path("rule-serde-alias-wire-target");
    write_serde_alias_wire_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("serde alias wire rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("serde_alias_wire_rule/src/lib.rs"));
    assert!(lib.contains("struct PairPayload"), "{lib}");
    assert!(lib.contains("alias = \"hostname\""), "{lib}");
    assert!(lib.contains("alias = \"display_name\""), "{lib}");
    assert!(lib.contains("host_name: Option<String>"), "{lib}");
    assert!(!lib.contains("struct DeadPayload"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_serde_transparent_contract_fields() {
    let workspace = temp_path("rule-serde-transparent-workspace");
    let output = temp_path("rule-serde-transparent-output");
    let target_dir = temp_path("rule-serde-transparent-target");
    write_serde_transparent_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("serde transparent rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("serde_transparent_rule/src/lib.rs"));
    assert!(lib.contains("#[serde(transparent)]"), "{lib}");
    assert!(lib.contains("pub struct SessionId"), "{lib}");
    assert!(lib.contains("pub id: SessionId"), "{lib}");
    assert!(!lib.contains("DeadSessionId"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_serde_skip_serializing_if_contract_fields() {
    let workspace = temp_path("rule-serde-skip-serializing-workspace");
    let output = temp_path("rule-serde-skip-serializing-output");
    let target_dir = temp_path("rule-serde-skip-serializing-target");
    write_serde_skip_serializing_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("serde skip_serializing_if rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("serde_skip_serializing_rule/src/lib.rs"));
    assert!(
        lib.contains("skip_serializing_if = \"Option::is_none\""),
        "{lib}"
    );
    assert!(lib.contains("target: Option<String>"), "{lib}");
    assert!(!lib.contains("dead: Option<String>"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_serde_untagged_enum_contract_variants() {
    let workspace = temp_path("rule-serde-untagged-enum-workspace");
    let output = temp_path("rule-serde-untagged-enum-output");
    let target_dir = temp_path("rule-serde-untagged-enum-target");
    write_serde_untagged_enum_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("serde untagged enum rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("serde_untagged_enum_rule/src/lib.rs"));
    assert!(lib.contains("#[serde(untagged)]"), "{lib}");
    assert!(lib.contains("Index(usize)"), "{lib}");
    assert!(lib.contains("Key(String)"), "{lib}");
    assert!(!lib.contains("DeadSegment"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_serde_tagged_enum_contract_variants() {
    let workspace = temp_path("rule-serde-tagged-enum-workspace");
    let output = temp_path("rule-serde-tagged-enum-output");
    let target_dir = temp_path("rule-serde-tagged-enum-target");
    write_serde_tagged_enum_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("serde tagged enum rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("serde_tagged_enum_rule/src/lib.rs"));
    assert!(lib.contains("#[serde(tag = \"type\")]"), "{lib}");
    assert!(lib.contains("ApiKey"), "{lib}");
    assert!(lib.contains("Chatgpt"), "{lib}");
    assert!(lib.contains("api_key: String"), "{lib}");
    assert!(!lib.contains("DeadLogin"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_serde_default_enum_variant_contracts() {
    let workspace = temp_path("rule-serde-default-enum-workspace");
    let output = temp_path("rule-serde-default-enum-output");
    let target_dir = temp_path("rule-serde-default-enum-target");
    write_serde_default_enum_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("serde default enum rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("serde_default_enum_rule/src/lib.rs"));
    assert!(lib.contains("#[derive(Deserialize, Default)]"), "{lib}");
    assert!(lib.contains("#[default]"), "{lib}");
    assert!(lib.contains("Auto"), "{lib}");
    assert!(lib.contains("Manual"), "{lib}");
    assert!(!lib.contains("DeadMode"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn copies_support_path_bundle_without_absolute_leaks_or_dead_surfaces() {
    let workspace = temp_path("rule-support-path-bundle-workspace");
    let output = temp_path("rule-support-path-bundle-output");
    let target_dir = temp_path("rule-support-path-bundle-target");
    let external = temp_path("rule-support-path-bundle-external");
    let helper = external.join("external-helper");
    let leaf = external.join("external-leaf");
    let dead_leaf = external.join("dead-leaf");
    write_support_path_bundle_rule_fixture(&workspace, &helper, &leaf, &dead_leaf);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("support path bundle rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let app_manifest = read(output.join("support_path_app/Cargo.toml"));
    let helper_manifest = read(output.join("support/external-helper/Cargo.toml"));
    let leaf_manifest = read(output.join("support/external-leaf/Cargo.toml"));
    assert!(app_manifest.contains("path = \"../support/external-helper\""));
    assert!(helper_manifest.contains("path = \"../external-leaf\""));
    assert!(!helper_manifest.contains("dead-leaf"));
    assert!(leaf_manifest.contains("name = \"external-leaf\""));
    assert!(!app_manifest.contains(&manifest_path(&helper)));
    assert!(!helper_manifest.contains(&manifest_path(&leaf)));
    assert!(!app_manifest.contains("path = \"/"), "{app_manifest}");
    assert!(!helper_manifest.contains("path = \"/"), "{helper_manifest}");
    assert!(output.join("support/external-helper/src/lib.rs").exists());
    assert!(output.join("support/external-helper/src/live.rs").exists());
    assert!(output
        .join("support/external-helper/src/facade.rs")
        .exists());
    assert!(output
        .join("support/external-helper/src/facade/inner.rs")
        .exists());
    assert!(output
        .join("support/external-helper/src/live/atoms.rs")
        .exists());
    assert!(output.join("support/external-helper/src/live.txt").exists());
    assert!(output.join("support/external-leaf/src/lib.rs").exists());
    assert!(!output.join("support/dead-leaf").exists());
    let helper_lib = read(output.join("support/external-helper/src/lib.rs"));
    let helper_live = read(output.join("support/external-helper/src/live.rs"));
    let helper_facade = read(output.join("support/external-helper/src/facade.rs"));
    let helper_inner = read(output.join("support/external-helper/src/facade/inner.rs"));
    let helper_atoms = read(output.join("support/external-helper/src/live/atoms.rs"));
    let leaf_lib = read(output.join("support/external-leaf/src/lib.rs"));
    assert!(helper_lib.contains("pub fn decorate"), "{helper_lib}");
    assert!(
        helper_lib.contains("pub use external_leaf::LeafLive as LeafAlias"),
        "{helper_lib}"
    );
    assert!(helper_lib.contains("pub use facade::*"), "{helper_lib}");
    assert!(!helper_lib.contains("dead_decorate"), "{helper_lib}");
    assert!(!helper_lib.contains("dead_leaf"), "{helper_lib}");
    assert!(!helper_lib.contains("LeafDead"), "{helper_lib}");
    assert!(!helper_lib.contains("LeafDeadAlias"), "{helper_lib}");
    assert!(!helper_lib.contains("FacadeDead"), "{helper_lib}");
    assert!(!helper_lib.contains("FacadeLive"), "{helper_lib}");
    assert!(helper_live.contains("mod atoms"), "{helper_live}");
    assert!(helper_live.contains("use atoms::*"), "{helper_live}");
    assert!(helper_live.contains("pub fn decorate"), "{helper_live}");
    assert!(!helper_live.contains("dead_child"), "{helper_live}");
    assert!(!helper_live.contains("dead_leaf"), "{helper_live}");
    assert!(helper_facade.contains("mod inner"), "{helper_facade}");
    assert!(
        helper_facade.contains("pub use inner::*"),
        "{helper_facade}"
    );
    assert!(!helper_facade.contains("dead_facade"), "{helper_facade}");
    assert!(!helper_facade.contains("FacadeDead"), "{helper_facade}");
    assert!(!helper_facade.contains("FacadeLive"), "{helper_facade}");
    assert!(!helper_facade.contains("dead_leaf"), "{helper_facade}");
    assert!(
        helper_inner.contains("pub fn facade_decorate"),
        "{helper_inner}"
    );
    assert!(!helper_inner.contains("dead_inner"), "{helper_inner}");
    assert!(!helper_inner.contains("FacadeDead"), "{helper_inner}");
    assert!(!helper_inner.contains("FacadeLive"), "{helper_inner}");
    assert!(!helper_inner.contains("dead_leaf"), "{helper_inner}");
    assert!(
        helper_atoms.contains("pub fn suffix() -> &'static str"),
        "{helper_atoms}"
    );
    assert!(!helper_atoms.contains("dead_suffix"), "{helper_atoms}");
    assert!(!helper_atoms.contains("dead_leaf"), "{helper_atoms}");
    assert!(leaf_lib.contains("pub struct LeafLive"), "{leaf_lib}");
    assert!(leaf_lib.contains("pub fn suffix"), "{leaf_lib}");
    assert!(!leaf_lib.contains("LeafDead"), "{leaf_lib}");
    assert!(!leaf_lib.contains("dead_suffix"), "{leaf_lib}");
    assert!(!output.join("support/external-helper/src/bin").exists());
    assert!(!output.join("support/external-helper/examples").exists());
    assert!(!output.join("support/external-helper/tests").exists());
    assert!(!output.join("support/external-helper/benches").exists());
    assert!(!output.join("support/external-helper/fixtures").exists());
    assert!(!output
        .join("support/external-helper/src/test_only.rs")
        .exists());
    assert!(!output.join("support/external-helper/src/test.txt").exists());
    assert!(!output
        .join("support/external-helper/src/orphan.rs")
        .exists());
    assert!(!output
        .join("support/external-helper/src/orphan.txt")
        .exists());
    fs::rename(&helper, helper.with_extension("moved"))
        .expect("original helper package should move away");
    fs::rename(&leaf, leaf.with_extension("moved"))
        .expect("original leaf package should move away");
    fs::rename(&dead_leaf, dead_leaf.with_extension("moved"))
        .expect("original dead leaf package should move away");
    let lib = read(output.join("support_path_app/src/lib.rs"));
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn copies_support_path_packages_with_nonstandard_lib_roots() {
    let workspace = temp_path("rule-support-nonstandard-lib-workspace");
    let output = temp_path("rule-support-nonstandard-lib-output");
    let target_dir = temp_path("rule-support-nonstandard-lib-target");
    let helper = temp_path("rule-support-nonstandard-lib-helper");
    write_support_nonstandard_lib_rule_fixture(&workspace, &helper);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("support nonstandard lib rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    assert!(output
        .join("support/external-helper/src/dump/lib.rs")
        .exists());
    assert!(output
        .join("support/external-helper/src/dump/client.rs")
        .exists());
    assert!(!output.join("support/external-helper/src/lib.rs").exists());
    fs::rename(&helper, helper.with_extension("moved"))
        .expect("original helper package should move away");
    let lib = read(output.join("support_nonstandard_app/src/lib.rs"));
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

    assert_eq!(report.production.status, "requires_feedback");
    let trait_object = report
        .production
        .hazards
        .iter()
        .find(|hazard| hazard.code == "trait_object_surfaces" && hazard.severity == "warning")
        .expect("owned dyn field should be a feedback hazard");
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
        .find(|hazard| hazard.code == "function_pointer_surfaces" && hazard.severity == "warning")
        .expect("stored callback field should be a feedback hazard");
    assert!(function_pointer
        .details
        .iter()
        .any(|detail| detail.subject.contains("fn (u32) -> u32")));

    let lib = read(output.join("owned_dyn_rule/src/lib.rs"));
    assert!(lib.contains("Box<dyn Worker>"), "{lib}");
    assert!(lib.contains("pub type Callback = fn(u32) -> u32"), "{lib}");
}

#[test]
fn ignores_dynamic_hazards_on_pruned_private_fields() {
    let workspace = temp_path("rule-pruned-private-dyn-field-workspace");
    let output = temp_path("rule-pruned-private-dyn-field-output");
    let target_dir = temp_path("rule-pruned-private-dyn-field-target");
    write_pruned_private_dyn_field_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("pruned private dyn field rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("pruned_private_dyn_field_rule/src/lib.rs"));
    assert!(lib.contains("pub(crate) struct Api"), "{lib}");
    assert!(lib.contains("pub id: u32"), "{lib}");
    assert!(!lib.contains("callback"), "{lib}");
    assert!(!lib.contains("dyn Fn"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn prunes_dead_private_generic_fields_when_type_params_remain_used() {
    let workspace = temp_path("rule-private-generic-field-workspace");
    let output = temp_path("rule-private-generic-field-output");
    let target_dir = temp_path("rule-private-generic-field-target");
    write_private_generic_field_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("private generic field rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("private_generic_field_rule/src/lib.rs"));
    assert!(lib.contains("pub value: T"), "{lib}");
    assert!(!lib.contains("Heavy<T>"), "{lib}");
    assert!(!lib.contains("dead: Heavy"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
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
fn flags_option_env_inputs_as_production_blockers() {
    let workspace = temp_path("rule-option-env-workspace");
    let output = temp_path("rule-option-env-output");
    write_option_env_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("option_env rule should reduce");

    assert_eq!(report.production.status, "hazards_detected");
    assert!(report
        .production
        .hazards
        .iter()
        .any(|hazard| hazard.code == "compile_env_macros" && hazard.severity == "error"));
    let lib = read(output.join("option_env_rule/src/lib.rs"));
    assert!(lib.contains("option_env!(\"LITTER_PROFILE\")"), "{lib}");
    assert!(!lib.contains("DEAD_PROFILE"), "{lib}");
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

    assert_eq!(report.production.status, "requires_feedback");
    let trait_object = report
        .production
        .hazards
        .iter()
        .find(|hazard| hazard.code == "trait_object_surfaces" && hazard.severity == "warning")
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
    assert!(lib.contains("out_dir_generated_helper"), "{lib}");
    assert!(!lib.contains("dead_out_dir_helper"), "{lib}");
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
fn retains_fixed_name_item_macro_generated_dependencies() {
    let workspace = temp_path("rule-fixed-name-item-macro-workspace");
    let output = temp_path("rule-fixed-name-item-macro-output");
    let target_dir = temp_path("rule-fixed-name-item-macro-target");
    write_fixed_name_item_macro_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("fixed-name item macro rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("fixed_name_item_macro_rule/src/lib.rs"));
    assert!(lib.contains("macro_rules! declare_runtime"), "{lib}");
    assert!(lib.contains("declare_runtime!(build_live)"), "{lib}");
    assert!(lib.contains("fn build_live() -> u32"), "{lib}");
    assert!(!lib.contains("build_dead"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_item_macro_invocations_inside_inline_modules() {
    let workspace = temp_path("rule-inline-item-macro-workspace");
    let output = temp_path("rule-inline-item-macro-output");
    let target_dir = temp_path("rule-inline-item-macro-target");
    write_inline_item_macro_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("inline item macro rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("inline_item_macro_rule/src/lib.rs"));
    assert!(lib.contains("mod generated"), "{lib}");
    assert!(lib.contains("make_api!()"), "{lib}");
    assert!(lib.contains("fn live_helper() -> u32"), "{lib}");
    assert!(!lib.contains("dead_helper"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn resolves_shadowed_macro_rules_by_live_module_scope() {
    let workspace = temp_path("rule-shadowed-macro-scope-workspace");
    let output = temp_path("rule-shadowed-macro-scope-output");
    let target_dir = temp_path("rule-shadowed-macro-scope-target");
    write_shadowed_macro_scope_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("shadowed macro scope rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("shadowed_macro_scope_rule/src/lib.rs"));
    assert!(lib.contains("live_helper"), "{lib}");
    assert!(!lib.contains("dead_helper"), "{lib}");
    assert!(!lib.contains("mod dead"), "{lib}");
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
fn retains_bidirectional_from_impl_pair_for_boundary_roundtrips() {
    let workspace = temp_path("rule-bidirectional-from-workspace");
    let output = temp_path("rule-bidirectional-from-output");
    let target_dir = temp_path("rule-bidirectional-from-target");
    write_bidirectional_from_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("bidirectional From rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("bidirectional_from_rule/src/lib.rs"));
    assert!(lib.contains("impl From<Internal> for Wire"), "{lib}");
    assert!(lib.contains("impl From<Wire> for Internal"), "{lib}");
    assert!(!lib.contains("DeadWire"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_struct_literal_field_into_conversion_without_dead_siblings() {
    let workspace = temp_path("rule-struct-field-into-workspace");
    let output = temp_path("rule-struct-field-into-output");
    let target_dir = temp_path("rule-struct-field-into-target");
    write_struct_field_into_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("struct field into rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("struct_field_into_rule/src/lib.rs"));
    assert!(lib.contains("impl From<WireMode> for PublicMode"), "{lib}");
    assert!(!lib.contains("DeadWireMode"), "{lib}");
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

    assert_eq!(report.production.status, "requires_feedback");
    let trait_object = report
        .production
        .hazards
        .iter()
        .find(|hazard| hazard.code == "trait_object_surfaces" && hazard.severity == "warning")
        .expect("callback trait object should be a feedback hazard");
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
fn reports_nested_callback_store_trait_objects_as_dynamic_hazards() {
    let workspace = temp_path("rule-nested-callback-store-workspace");
    let output = temp_path("rule-nested-callback-store-output");
    write_nested_callback_store_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("nested callback store rule should reduce");

    assert_eq!(report.production.status, "requires_feedback");
    let trait_object = report
        .production
        .hazards
        .iter()
        .find(|hazard| hazard.code == "trait_object_surfaces" && hazard.severity == "warning")
        .expect("nested callback store should be a feedback hazard");
    assert!(trait_object.details.iter().any(|detail| {
        detail.subject.contains("dyn ReconnectCallback")
            && detail.subject.contains("Send")
            && detail.subject.contains("Sync")
    }));

    let lib = read(output.join("nested_callback_store_rule/src/lib.rs"));
    assert!(
        lib.contains("Arc<RwLock<Option<Arc<dyn ReconnectCallback + Send + Sync>>>>"),
        "{lib}"
    );
    assert!(!lib.contains("DeadCallback"), "{lib}");
}

#[test]
fn ignores_auto_trait_object_casts_while_retaining_concrete_sources() {
    let workspace = temp_path("rule-trait-object-cast-workspace");
    let output = temp_path("rule-trait-object-cast-output");
    let target_dir = temp_path("rule-trait-object-cast-target");
    write_trait_object_cast_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("trait object cast rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    assert!(!report
        .production
        .hazards
        .iter()
        .any(|hazard| hazard.code == "trait_object_surfaces"));
    let lib = read(output.join("trait_object_cast_rule/src/lib.rs"));
    assert!(lib.contains("pub struct Session"), "{lib}");
    assert!(lib.contains("Session::new()"), "{lib}");
    assert!(lib.contains("Arc<dyn Send + Sync>"), "{lib}");
    assert!(!lib.contains("DeadSession"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
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
fn reports_wrapped_borrowed_callback_boundaries_without_trait_object_noise() {
    let workspace = temp_path("rule-wrapped-callback-boundary-workspace");
    let output = temp_path("rule-wrapped-callback-boundary-output");
    let target_dir = temp_path("rule-wrapped-callback-boundary-target");
    write_wrapped_callback_boundary_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("wrapped callback boundary rule should reduce");

    assert_eq!(report.production.status, "requires_feedback");
    assert_no_error_hazards(&report.production.hazards);
    assert!(!report
        .production
        .hazards
        .iter()
        .any(|hazard| hazard.code == "trait_object_surfaces"));
    let boundary = report
        .production
        .hazards
        .iter()
        .find(|hazard| hazard.code == "dynamic_callback_boundaries")
        .expect("wrapped direct callback boundaries should be reported");
    assert!(boundary
        .details
        .iter()
        .any(|detail| detail.subject.contains("Option < & dyn Callback >")));
    assert!(boundary.details.iter().any(|detail| detail
        .subject
        .contains("Result < fn (u32) -> u32 , Error >")));

    let lib = read(output.join("wrapped_callback_boundary_rule/src/lib.rs"));
    assert!(lib.contains("handler: Option<&dyn Callback>"), "{lib}");
    assert!(
        lib.contains("callback: Result<fn(u32) -> u32, Error>"),
        "{lib}"
    );
    assert!(!lib.contains("DeadCallback"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn ignores_function_pointer_hazards_from_pruned_trait_surface_methods() {
    let workspace = temp_path("rule-pruned-trait-method-hazard-workspace");
    let output = temp_path("rule-pruned-trait-method-hazard-output");
    let target_dir = temp_path("rule-pruned-trait-method-hazard-target");
    write_pruned_trait_method_hazard_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("pruned trait method hazard rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("pruned_trait_method_hazard_rule/src/lib.rs"));
    assert!(lib.contains("pub trait Callback"), "{lib}");
    assert!(!lib.contains("dead_hook"), "{lib}");
    assert!(!lib.contains("fn(u32) -> u32"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn reports_bare_dyn_alias_inside_once_lock_arc_as_dynamic_hazard() {
    let workspace = temp_path("rule-bare-dyn-alias-once-lock-workspace");
    let output = temp_path("rule-bare-dyn-alias-once-lock-output");
    let target_dir = temp_path("rule-bare-dyn-alias-once-lock-target");
    write_bare_dyn_alias_once_lock_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("bare dyn alias once lock rule should reduce");

    assert_eq!(report.production.status, "requires_feedback");
    assert!(report.production.hazards.iter().any(|hazard| {
        hazard.code == "trait_object_surfaces"
            && hazard
                .details
                .iter()
                .any(|detail| detail.subject.contains("dyn Fn"))
    }));
    let lib = read(output.join("bare_dyn_alias_once_lock_rule/src/lib.rs"));
    assert!(lib.contains("type Observer = dyn Fn"), "{lib}");
    assert!(lib.contains("OnceLock<Arc<Observer>>"), "{lib}");
    assert!(!lib.contains("DeadObserver"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn reports_facade_reexported_boxed_io_alias_without_dead_siblings() {
    let workspace = temp_path("rule-boxed-io-alias-facade-workspace");
    let output = temp_path("rule-boxed-io-alias-facade-output");
    let target_dir = temp_path("rule-boxed-io-alias-facade-target");
    write_boxed_io_alias_facade_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("boxed io alias facade rule should reduce");

    assert_eq!(report.production.status, "requires_feedback");
    assert!(report.production.hazards.iter().any(|hazard| {
        hazard.code == "trait_object_surfaces"
            && hazard
                .details
                .iter()
                .any(|detail| detail.subject.contains("dyn Read"))
    }));
    let lib = read(output.join("boxed_io_alias_facade_rule/src/lib.rs"));
    assert!(lib.contains("pub use types::Input"), "{lib}");
    assert!(
        lib.contains("pub type Input = Box<dyn Read + Send>"),
        "{lib}"
    );
    assert!(lib.contains("use std::io::Read"), "{lib}");
    assert!(!lib.contains("Output"), "{lib}");
    assert!(!lib.contains("Write"), "{lib}");
    assert!(!lib.contains("DeadAlias"), "{lib}");
    assert!(!lib.contains("dead_selected"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn prunes_facade_const_alias_dead_siblings() {
    let workspace = temp_path("rule-facade-const-alias-workspace");
    let output = temp_path("rule-facade-const-alias-output");
    let target_dir = temp_path("rule-facade-const-alias-target");
    write_facade_const_alias_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("facade const alias rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("facade_const_alias_rule/src/lib.rs"));
    assert!(lib.contains("LIVE_LIMIT as LiveLimit"), "{lib}");
    assert!(lib.contains("pub const LIVE_LIMIT"), "{lib}");
    assert!(!lib.contains("DEAD_LIMIT"), "{lib}");
    assert!(!lib.contains("DeadLimit"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn prunes_facade_function_alias_dead_siblings() {
    let workspace = temp_path("rule-facade-function-alias-workspace");
    let output = temp_path("rule-facade-function-alias-output");
    let target_dir = temp_path("rule-facade-function-alias-target");
    write_facade_function_alias_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("facade function alias rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("facade_function_alias_rule/src/lib.rs"));
    assert!(lib.contains("live_value as public_live"), "{lib}");
    assert!(lib.contains("pub fn live_value"), "{lib}");
    assert!(!lib.contains("dead_value"), "{lib}");
    assert!(!lib.contains("public_dead"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_result_alias_error_and_payload_surfaces() {
    let workspace = temp_path("rule-result-alias-surface-workspace");
    let output = temp_path("rule-result-alias-surface-output");
    let target_dir = temp_path("rule-result-alias-surface-target");
    write_result_alias_surface_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("result alias surface rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("result_alias_surface_rule/src/lib.rs"));
    assert!(lib.contains("pub type AppResult<T>"), "{lib}");
    assert!(lib.contains("pub enum AppError"), "{lib}");
    assert!(lib.contains("pub struct StartResponse"), "{lib}");
    assert!(!lib.contains("DeadError"), "{lib}");
    assert!(!lib.contains("DeadResponse"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_arc_object_handle_inside_returned_record() {
    let workspace = temp_path("rule-arc-handle-record-workspace");
    let output = temp_path("rule-arc-handle-record-output");
    let target_dir = temp_path("rule-arc-handle-record-target");
    write_arc_handle_record_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("arc handle record rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("arc_handle_record_rule/src/lib.rs"));
    assert!(lib.contains("pub struct PairHostStartResult"), "{lib}");
    assert!(lib.contains("Arc<PairHostHandle>"), "{lib}");
    assert!(lib.contains("pub struct PairHostHandle"), "{lib}");
    assert!(lib.contains("pub struct PairHostInfo"), "{lib}");
    assert!(!lib.contains("DeadPairHostHandle"), "{lib}");
    assert!(!lib.contains("DeadPairHostStartResult"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_nested_option_vec_dto_surfaces_without_dead_siblings() {
    let workspace = temp_path("rule-nested-option-vec-dto-workspace");
    let output = temp_path("rule-nested-option-vec-dto-output");
    let target_dir = temp_path("rule-nested-option-vec-dto-target");
    write_nested_option_vec_dto_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("nested option vec dto rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("nested_option_vec_dto_rule/src/lib.rs"));
    assert!(lib.contains("Option<Vec<AppDynamicToolSpec>>"), "{lib}");
    assert!(lib.contains("pub struct AppDynamicToolSpec"), "{lib}");
    assert!(lib.contains("pub struct AppStartThreadRequest"), "{lib}");
    assert!(!lib.contains("DeadDynamicToolSpec"), "{lib}");
    assert!(!lib.contains("DeadStartThreadRequest"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_selected_mirror_try_from_conversion_without_dead_mirrors() {
    let workspace = temp_path("rule-mirror-try-from-workspace");
    let output = temp_path("rule-mirror-try-from-output");
    let target_dir = temp_path("rule-mirror-try-from-target");
    write_mirror_try_from_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("mirror try_from rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("mirror_try_from_rule/src/lib.rs"));
    assert!(lib.contains("impl TryFrom<AppStartThreadRequest>"), "{lib}");
    assert!(lib.contains("pub struct ThreadStartParams"), "{lib}");
    assert!(lib.contains("pub struct AppDynamicToolSpec"), "{lib}");
    assert!(lib.contains("fn parse_schema"), "{lib}");
    assert!(!lib.contains("AppResumeThreadRequest"), "{lib}");
    assert!(!lib.contains("ThreadResumeParams"), "{lib}");
    assert!(!lib.contains("dead_parse_schema"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn copies_selected_section_include_assets_only() {
    let workspace = temp_path("rule-section-include-workspace");
    let output = temp_path("rule-section-include-output");
    let target_dir = temp_path("rule-section-include-target");
    write_section_include_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("section include rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("section_include_rule/src/lib.rs"));
    assert!(lib.contains("include_str!(\"guides/live.md\")"), "{lib}");
    assert!(!lib.contains("include_str!(\"guides/dead.md\")"), "{lib}");
    assert!(output
        .join("section_include_rule/src/guides/live.md")
        .exists());
    assert!(!output
        .join("section_include_rule/src/guides/dead.md")
        .exists());
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_runtime_facade_singleton_without_dead_helpers() {
    let workspace = temp_path("rule-runtime-facade-workspace");
    let output = temp_path("rule-runtime-facade-output");
    let target_dir = temp_path("rule-runtime-facade-target");
    write_runtime_facade_singleton_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("runtime facade singleton rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("runtime_facade_singleton_rule/src/lib.rs"));
    assert!(
        lib.contains("shared_mobile_client as shared_client"),
        "{lib}"
    );
    assert!(lib.contains("pub fn shared_mobile_client"), "{lib}");
    assert!(lib.contains("OnceLock"), "{lib}");
    assert!(!lib.contains("dead_shared_mobile_client"), "{lib}");
    assert!(!lib.contains("dead_shared_runtime"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_map_payload_dto_surface_without_dead_snapshot_siblings() {
    let workspace = temp_path("rule-map-payload-dto-workspace");
    let output = temp_path("rule-map-payload-dto-output");
    let target_dir = temp_path("rule-map-payload-dto-target");
    write_map_payload_dto_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("map payload dto rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("map_payload_dto_rule/src/lib.rs"));
    assert!(lib.contains("BTreeMap<String, ThreadSnapshot>"), "{lib}");
    assert!(lib.contains("pub struct ThreadSnapshot"), "{lib}");
    assert!(!lib.contains("DeadThreadSnapshot"), "{lib}");
    assert!(!lib.contains("DeadAppSnapshot"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn prunes_dead_inherent_impl_methods_for_function_roots() {
    let workspace = temp_path("rule-dead-impl-method-workspace");
    let output = temp_path("rule-dead-impl-method-output");
    let target_dir = temp_path("rule-dead-impl-method-target");
    write_dead_impl_method_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("dead impl method rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("dead_impl_method_rule/src/lib.rs"));
    assert!(lib.contains("pub fn live_score"), "{lib}");
    assert!(lib.contains("fn private_seed"), "{lib}");
    assert!(!lib.contains("dead_score"), "{lib}");
    assert!(!lib.contains("dead_private_seed"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_type_alias_chain_surface_dependencies() {
    let workspace = temp_path("rule-type-alias-chain-workspace");
    let output = temp_path("rule-type-alias-chain-output");
    let target_dir = temp_path("rule-type-alias-chain-target");
    write_type_alias_chain_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("type alias chain rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("type_alias_chain_rule/src/lib.rs"));
    assert!(lib.contains("pub type PublicBatch"), "{lib}");
    assert!(lib.contains("pub type MessageBatch"), "{lib}");
    assert!(lib.contains("pub struct WireMessage"), "{lib}");
    assert!(!lib.contains("DeadWireMessage"), "{lib}");
    assert!(!lib.contains("DeadBatch"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_newtype_tuple_surface_dependencies() {
    let workspace = temp_path("rule-newtype-tuple-surface-workspace");
    let output = temp_path("rule-newtype-tuple-surface-output");
    let target_dir = temp_path("rule-newtype-tuple-surface-target");
    write_newtype_tuple_surface_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("newtype tuple surface rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("newtype_tuple_surface_rule/src/lib.rs"));
    assert!(lib.contains("pub struct SessionId"), "{lib}");
    assert!(lib.contains("pub struct StartResult"), "{lib}");
    assert!(!lib.contains("DeadSessionId"), "{lib}");
    assert!(!lib.contains("DeadStartResult"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_impl_trait_iterator_item_surface() {
    let workspace = temp_path("rule-impl-trait-iterator-workspace");
    let output = temp_path("rule-impl-trait-iterator-output");
    let target_dir = temp_path("rule-impl-trait-iterator-target");
    write_impl_trait_iterator_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("impl trait iterator rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("impl_trait_iterator_rule/src/lib.rs"));
    assert!(lib.contains("impl Iterator<Item = EventDto>"), "{lib}");
    assert!(lib.contains("pub struct EventDto"), "{lib}");
    assert!(lib.contains("fn make_event"), "{lib}");
    assert!(!lib.contains("DeadEventDto"), "{lib}");
    assert!(!lib.contains("dead_events"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_question_mark_error_conversion_impl() {
    let workspace = temp_path("rule-question-mark-error-workspace");
    let output = temp_path("rule-question-mark-error-output");
    let target_dir = temp_path("rule-question-mark-error-target");
    write_question_mark_error_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("question mark error rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("question_mark_error_rule/src/lib.rs"));
    assert!(lib.contains("impl From<ParseError> for AppError"), "{lib}");
    assert!(lib.contains("fn parse_wire"), "{lib}");
    assert!(!lib.contains("DeadError"), "{lib}");
    assert!(!lib.contains("dead_parse_wire"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_enum_variant_constructor_used_as_function() {
    let workspace = temp_path("rule-enum-variant-constructor-workspace");
    let output = temp_path("rule-enum-variant-constructor-output");
    let target_dir = temp_path("rule-enum-variant-constructor-target");
    write_enum_variant_constructor_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("enum variant constructor rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("enum_variant_constructor_rule/src/lib.rs"));
    assert!(lib.contains("pub enum Event"), "{lib}");
    assert!(lib.contains("pub struct Started"), "{lib}");
    assert!(lib.contains("map(Event::Started)"), "{lib}");
    assert!(!lib.contains("DeadStarted"), "{lib}");
    assert!(!lib.contains("DeadEvent"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_struct_update_default_surface() {
    let workspace = temp_path("rule-struct-update-default-workspace");
    let output = temp_path("rule-struct-update-default-output");
    let target_dir = temp_path("rule-struct-update-default-target");
    write_struct_update_default_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("struct update default rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("struct_update_default_rule/src/lib.rs"));
    assert!(lib.contains("#[derive(Default)]"), "{lib}");
    assert!(lib.contains("pub struct RuntimeConfig"), "{lib}");
    assert!(lib.contains("..Default::default()"), "{lib}");
    assert!(!lib.contains("DeadRuntimeConfig"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_method_references_used_as_iterator_functions() {
    let workspace = temp_path("rule-method-reference-map-workspace");
    let output = temp_path("rule-method-reference-map-output");
    let target_dir = temp_path("rule-method-reference-map-target");
    write_method_reference_map_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("method reference map rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("method_reference_map_rule/src/lib.rs"));
    assert!(lib.contains("Score::from_raw"), "{lib}");
    assert!(lib.contains("Score::value"), "{lib}");
    assert!(lib.contains("pub fn from_raw"), "{lib}");
    assert!(lib.contains("pub fn value"), "{lib}");
    assert!(!lib.contains("dead_value"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn reports_boxed_future_return_alias_as_dynamic_hazard() {
    let workspace = temp_path("rule-boxed-future-return-workspace");
    let output = temp_path("rule-boxed-future-return-output");
    let target_dir = temp_path("rule-boxed-future-return-target");
    write_boxed_future_return_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("boxed future return rule should reduce");

    assert_eq!(report.production.status, "requires_feedback");
    assert!(report.production.hazards.iter().any(|hazard| {
        hazard.code == "trait_object_surfaces"
            && hazard
                .details
                .iter()
                .any(|detail| detail.subject.contains("dyn Future"))
    }));
    let lib = read(output.join("boxed_future_return_rule/src/lib.rs"));
    assert!(lib.contains("type ResponseFuture"), "{lib}");
    assert!(lib.contains("dyn Future<Output = Response>"), "{lib}");
    assert!(lib.contains("pub struct Response"), "{lib}");
    assert!(!lib.contains("DeadResponse"), "{lib}");
    assert!(!lib.contains("dead_selected"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_const_generic_array_surface_dependency() {
    let workspace = temp_path("rule-const-generic-array-workspace");
    let output = temp_path("rule-const-generic-array-output");
    let target_dir = temp_path("rule-const-generic-array-target");
    write_const_generic_array_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("const generic array rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("const_generic_array_rule/src/lib.rs"));
    assert!(lib.contains("pub const LIVE_FRAME_LEN"), "{lib}");
    assert!(lib.contains("[u8; LIVE_FRAME_LEN]"), "{lib}");
    assert!(lib.contains("pub struct FrameBytes"), "{lib}");
    assert!(!lib.contains("DEAD_FRAME_LEN"), "{lib}");
    assert!(!lib.contains("DeadFrameBytes"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_nested_result_option_alias_surface() {
    let workspace = temp_path("rule-nested-result-option-alias-workspace");
    let output = temp_path("rule-nested-result-option-alias-output");
    let target_dir = temp_path("rule-nested-result-option-alias-target");
    write_nested_result_option_alias_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("nested result option alias rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("nested_result_option_alias_rule/src/lib.rs"));
    assert!(lib.contains("pub type MaybePayload"), "{lib}");
    assert!(lib.contains("pub type ApiResult"), "{lib}");
    assert!(lib.contains("pub struct Payload"), "{lib}");
    assert!(!lib.contains("DeadPayload"), "{lib}");
    assert!(!lib.contains("DeadResult"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_uniffi_callback_interface_facade_reexport() {
    let workspace = temp_path("rule-uniffi-callback-facade-workspace");
    let output = temp_path("rule-uniffi-callback-facade-output");
    let target_dir = temp_path("rule-uniffi-callback-facade-target");
    write_uniffi_callback_facade_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("uniffi callback facade rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("uniffi_callback_facade_rule/src/lib.rs"));
    assert!(
        lib.contains("pub use crate::boundary::{Credential, CredentialProvider}"),
        "{lib}"
    );
    assert!(lib.contains("pub trait CredentialProvider"), "{lib}");
    assert!(lib.contains("pub struct Credential"), "{lib}");
    assert!(!lib.contains("DeadCredential"), "{lib}");
    assert!(!lib.contains("DeadProvider"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_returned_subscription_object_exported_impl_surface() {
    let workspace = temp_path("rule-returned-subscription-workspace");
    let output = temp_path("rule-returned-subscription-output");
    let target_dir = temp_path("rule-returned-subscription-target");
    write_returned_subscription_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("returned subscription rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("returned_subscription_rule/src/lib.rs"));
    assert!(lib.contains("pub struct StreamSubscription"), "{lib}");
    assert!(lib.contains("pub async fn next_event"), "{lib}");
    assert!(lib.contains("pub struct StreamEvent"), "{lib}");
    assert!(lib.contains("pub enum StreamError"), "{lib}");
    assert!(!lib.contains("DeadSubscription"), "{lib}");
    assert!(!lib.contains("dead_event"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_split_exported_wrapper_private_impl_dependencies() {
    let workspace = temp_path("rule-split-wrapper-impl-workspace");
    let output = temp_path("rule-split-wrapper-impl-output");
    let target_dir = temp_path("rule-split-wrapper-impl-target");
    write_split_wrapper_impl_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("split wrapper impl rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("split_wrapper_impl_rule/src/lib.rs"));
    assert!(lib.contains("pub fn exported_wrapper"), "{lib}");
    assert!(lib.contains("fn private_helper"), "{lib}");
    assert!(lib.contains("fn convert"), "{lib}");
    assert!(!lib.contains("dead_helper"), "{lib}");
    assert!(!lib.contains("dead_export"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_nested_serde_tagged_patch_path_contract() {
    let workspace = temp_path("rule-serde-patch-path-workspace");
    let output = temp_path("rule-serde-patch-path-output");
    let target_dir = temp_path("rule-serde-patch-path-target");
    write_serde_patch_path_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("serde patch path rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("serde_patch_path_rule/src/lib.rs"));
    assert!(lib.contains("#[serde(tag = \"type\")]"), "{lib}");
    assert!(lib.contains("#[serde(untagged)]"), "{lib}");
    assert!(lib.contains("pub struct Patch"), "{lib}");
    assert!(lib.contains("Vec<PathSegment>"), "{lib}");
    assert!(!lib.contains("DeadPatch"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_request_method_typed_dispatch_unknown_fallback() {
    let workspace = temp_path("rule-method-dispatch-workspace");
    let output = temp_path("rule-method-dispatch-output");
    let target_dir = temp_path("rule-method-dispatch-target");
    write_method_dispatch_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("method dispatch rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("method_dispatch_rule/src/lib.rs"));
    assert!(lib.contains("Method::from_wire"), "{lib}");
    assert!(lib.contains("Start(StartParams)"), "{lib}");
    assert!(lib.contains("Unknown {"), "{lib}");
    assert!(lib.contains("pub struct StartParams"), "{lib}");
    assert!(!lib.contains("StopParams"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_try_from_option_vec_transpose_dependencies() {
    let workspace = temp_path("rule-try-from-transpose-workspace");
    let output = temp_path("rule-try-from-transpose-output");
    let target_dir = temp_path("rule-try-from-transpose-target");
    write_try_from_option_vec_transpose_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("try_from transpose rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("try_from_transpose_rule/src/lib.rs"));
    assert!(lib.contains(".transpose()?"), "{lib}");
    assert!(lib.contains("parse_schema"), "{lib}");
    assert!(lib.contains("pub struct DynamicToolSpec"), "{lib}");
    assert!(!lib.contains("DeadToolSpec"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn copies_function_local_static_include_bytes_asset() {
    let workspace = temp_path("rule-local-static-include-bytes-workspace");
    let output = temp_path("rule-local-static-include-bytes-output");
    let target_dir = temp_path("rule-local-static-include-bytes-target");
    write_local_static_include_bytes_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("local static include bytes rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("local_static_include_bytes_rule/src/lib.rs"));
    assert!(lib.contains("include_bytes!(\"certs/live.pem\")"), "{lib}");
    assert!(!lib.contains("include_bytes!(\"certs/dead.pem\")"), "{lib}");
    assert!(output
        .join("local_static_include_bytes_rule/src/certs/live.pem")
        .exists());
    assert!(!output
        .join("local_static_include_bytes_rule/src/certs/dead.pem")
        .exists());
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn flags_compile_time_env_inside_retained_macro_body() {
    let workspace = temp_path("rule-macro-env-workspace");
    let output = temp_path("rule-macro-env-output");
    write_macro_env_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("macro env rule should reduce");

    assert_eq!(report.production.status, "hazards_detected");
    assert!(report
        .production
        .hazards
        .iter()
        .any(|hazard| hazard.code == "compile_env_macros" && hazard.severity == "error"));
    let lib = read(output.join("macro_env_rule/src/lib.rs"));
    assert!(lib.contains("macro_rules! find_resource"), "{lib}");
    assert!(lib.contains("option_env!(\"BAZEL_PACKAGE\")"), "{lib}");
    assert!(!lib.contains("dead_resource"), "{lib}");
}

#[test]
fn retains_dependency_crate_alias_manifest_edge() {
    let workspace = temp_path("rule-dependency-crate-alias-workspace");
    let output = temp_path("rule-dependency-crate-alias-output");
    let target_dir = temp_path("rule-dependency-crate-alias-target");
    write_dependency_crate_alias_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("dependency crate alias rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let app = read(output.join("dependency_alias_app/src/lib.rs"));
    assert!(app.contains("use protocol_runtime as upstream"), "{app}");
    assert!(app.contains("upstream::LiveType"), "{app}");
    assert!(!app.contains("DeadType"), "{app}");
    let manifest = read(output.join("dependency_alias_app/Cargo.toml"));
    assert!(manifest.contains("protocol_runtime"), "{manifest}");
    assert_cargo_check(&output, &target_dir, &app);
}

#[test]
fn prunes_private_child_wildcard_with_selected_public_reexports() {
    let workspace = temp_path("rule-private-child-wildcard-workspace");
    let output = temp_path("rule-private-child-wildcard-output");
    let target_dir = temp_path("rule-private-child-wildcard-target");
    write_private_child_wildcard_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("private child wildcard rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("private_child_wildcard_rule/src/lib.rs"));
    assert!(lib.contains("pub use live::selected_helper"), "{lib}");
    assert!(lib.contains("use self::live::*"), "{lib}");
    assert!(!lib.contains("dead_helper"), "{lib}");
    assert!(!lib.contains("dead_value"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn prunes_nested_private_facade_reexport_chains_without_dead_siblings() {
    let workspace = temp_path("rule-nested-private-facade-workspace");
    let output = temp_path("rule-nested-private-facade-output");
    let target_dir = temp_path("rule-nested-private-facade-target");
    write_nested_private_facade_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("nested private facade rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("nested_private_facade_rule/src/lib.rs"));
    assert!(
        lib.contains("pub use crate::protocol::{live_factory, LiveClient}"),
        "{lib}"
    );
    assert!(
        lib.contains("pub use inner::{live_factory, LiveClient}"),
        "{lib}"
    );
    assert!(lib.contains("pub struct LiveClient"), "{lib}");
    assert!(!lib.contains("DeadClient"), "{lib}");
    assert!(!lib.contains("dead_factory"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_uniffi_enum_struct_variant_payload_surfaces() {
    let workspace = temp_path("rule-uniffi-struct-enum-workspace");
    let output = temp_path("rule-uniffi-struct-enum-output");
    let target_dir = temp_path("rule-uniffi-struct-enum-target");
    write_uniffi_struct_variant_enum_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("uniffi struct variant enum rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("uniffi_struct_enum_rule/src/lib.rs"));
    assert!(lib.contains("Started {"), "{lib}");
    assert!(lib.contains("Failed {"), "{lib}");
    assert!(lib.contains("pub struct SessionId"), "{lib}");
    assert!(lib.contains("pub struct StartConfig"), "{lib}");
    assert!(lib.contains("pub enum ClientError"), "{lib}");
    assert!(!lib.contains("DeadEvent"), "{lib}");
    assert!(!lib.contains("DeadPayload"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_nested_discriminated_serde_response_envelopes() {
    let workspace = temp_path("rule-serde-response-envelope-workspace");
    let output = temp_path("rule-serde-response-envelope-output");
    let target_dir = temp_path("rule-serde-response-envelope-target");
    write_serde_response_envelope_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("serde response envelope rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("serde_response_envelope_rule/src/lib.rs"));
    assert!(
        lib.contains("#[serde(tag = \"kind\", content = \"payload\")]"),
        "{lib}"
    );
    assert!(lib.contains("#[serde(tag = \"event\")]"), "{lib}");
    assert!(lib.contains("pub struct TokenPayload"), "{lib}");
    assert!(lib.contains("pub struct ErrorPayload"), "{lib}");
    assert!(!lib.contains("DeadEnvelope"), "{lib}");
    assert!(!lib.contains("DeadPayload"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_multiple_deserialize_with_helper_paths() {
    let workspace = temp_path("rule-serde-multi-helper-workspace");
    let output = temp_path("rule-serde-multi-helper-output");
    let target_dir = temp_path("rule-serde-multi-helper-target");
    write_serde_multi_helper_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("serde multi-helper rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("serde_multi_helper_rule/src/lib.rs"));
    assert!(
        lib.contains("deserialize_with = \"helpers::de_count\""),
        "{lib}"
    );
    assert!(
        lib.contains("deserialize_with = \"helpers::de_tags\""),
        "{lib}"
    );
    assert!(lib.contains("fn de_count"), "{lib}");
    assert!(lib.contains("fn de_tags"), "{lib}");
    assert!(!lib.contains("dead_helper"), "{lib}");
    assert!(!lib.contains("DeadWire"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_generic_try_into_error_bridge_dependencies() {
    let workspace = temp_path("rule-generic-try-into-bridge-workspace");
    let output = temp_path("rule-generic-try-into-bridge-output");
    let target_dir = temp_path("rule-generic-try-into-bridge-target");
    write_generic_try_into_bridge_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("generic try_into bridge rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("generic_try_into_bridge_rule/src/lib.rs"));
    assert!(lib.contains("fn bridge<T>(input: T)"), "{lib}");
    assert!(
        lib.contains("impl TryFrom<WireRequest> for InternalRequest"),
        "{lib}"
    );
    assert!(lib.contains("impl From<WireError> for ApiError"), "{lib}");
    assert!(
        lib.contains("impl From<InternalRequest> for PublicRequest"),
        "{lib}"
    );
    assert!(!lib.contains("DeadWireRequest"), "{lib}");
    assert!(!lib.contains("DeadInternalRequest"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_error_wire_fallback_conversion_dependencies() {
    let workspace = temp_path("rule-error-wire-fallback-workspace");
    let output = temp_path("rule-error-wire-fallback-output");
    let target_dir = temp_path("rule-error-wire-fallback-target");
    write_error_wire_fallback_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("error wire fallback rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("error_wire_fallback_rule/src/lib.rs"));
    assert!(lib.contains("Unknown {"), "{lib}");
    assert!(lib.contains("impl From<WireError> for ApiError"), "{lib}");
    assert!(
        lib.contains("parse_wire(value).map_err(ApiError::from)"),
        "{lib}"
    );
    assert!(!lib.contains("DeadError"), "{lib}");
    assert!(!lib.contains("dead_parse"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn copies_concat_include_str_array_assets_only() {
    let workspace = temp_path("rule-concat-include-array-workspace");
    let output = temp_path("rule-concat-include-array-output");
    let target_dir = temp_path("rule-concat-include-array-target");
    write_concat_include_array_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("concat include array rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("concat_include_array_rule/src/lib.rs"));
    assert!(
        lib.contains("include_str!(concat!(\"frames/\", \"intro.txt\"))"),
        "{lib}"
    );
    assert!(
        lib.contains("include_str!(concat!(\"frames/\", \"outro.txt\"))"),
        "{lib}"
    );
    assert!(!lib.contains("dead.txt"), "{lib}");
    assert!(output
        .join("concat_include_array_rule/src/frames/intro.txt")
        .exists());
    assert!(output
        .join("concat_include_array_rule/src/frames/outro.txt")
        .exists());
    assert!(!output
        .join("concat_include_array_rule/src/frames/dead.txt")
        .exists());
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn copies_once_lock_struct_initializer_include_asset_only() {
    let workspace = temp_path("rule-once-lock-include-workspace");
    let output = temp_path("rule-once-lock-include-output");
    let target_dir = temp_path("rule-once-lock-include-target");
    write_once_lock_include_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("once lock include rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("once_lock_include_rule/src/lib.rs"));
    assert!(lib.contains("OnceLock<Schema>"), "{lib}");
    assert!(lib.contains("include_str!(\"schema/live.json\")"), "{lib}");
    assert!(!lib.contains("schema/dead.json"), "{lib}");
    assert!(output
        .join("once_lock_include_rule/src/schema/live.json")
        .exists());
    assert!(!output
        .join("once_lock_include_rule/src/schema/dead.json")
        .exists());
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_method_facade_trait_object_boundary_as_warning() {
    let workspace = temp_path("rule-method-facade-dyn-boundary-workspace");
    let output = temp_path("rule-method-facade-dyn-boundary-output");
    let target_dir = temp_path("rule-method-facade-dyn-boundary-target");
    write_method_facade_dyn_boundary_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("method facade dyn boundary rule should reduce");

    assert_eq!(report.production.status, "requires_feedback");
    assert_no_error_hazards(&report.production.hazards);
    assert!(report.production.hazards.iter().any(|hazard| {
        hazard.code == "dynamic_callback_boundaries"
            && hazard
                .details
                .iter()
                .any(|detail| detail.subject.contains("& dyn Boundary"))
    }));
    let lib = read(output.join("method_facade_dyn_boundary_rule/src/lib.rs"));
    assert!(lib.contains("pub trait Boundary"), "{lib}");
    assert!(
        lib.contains("pub fn selected(handler: &dyn Boundary)"),
        "{lib}"
    );
    assert!(!lib.contains("DeadBoundary"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn prunes_local_facade_glob_reexports_to_selected_symbols() {
    let workspace = temp_path("rule-local-facade-glob-workspace");
    let output = temp_path("rule-local-facade-glob-output");
    let target_dir = temp_path("rule-local-facade-glob-target");
    write_local_facade_glob_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("local facade glob rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("local_facade_glob_rule/src/lib.rs"));
    assert!(lib.contains("pub use api::*"), "{lib}");
    assert!(lib.contains("pub fn selected_value"), "{lib}");
    assert!(!lib.contains("dead_value"), "{lib}");
    assert!(!lib.contains("DeadDto"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn flags_retained_source_include_macros_as_production_blockers() {
    let workspace = temp_path("rule-source-include-workspace");
    let output = temp_path("rule-source-include-output");
    let target_dir = temp_path("rule-source-include-target");
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
    assert!(lib.contains("helper_from_generated_expr"), "{lib}");
    assert!(!lib.contains("dead_generated"), "{lib}");
    assert!(!lib.contains("dead_helper"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn flags_fallback_retained_inline_module_source_includes() {
    let workspace = temp_path("rule-inline-source-include-workspace");
    let output = temp_path("rule-inline-source-include-output");
    write_inline_source_include_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("inline source include rule should reduce");

    assert_eq!(report.production.status, "hazards_detected");
    assert!(report
        .production
        .hazards
        .iter()
        .any(|hazard| { hazard.code == "source_include_macros" && hazard.severity == "error" }));
    let lib = read(output.join("inline_source_include_rule/src/lib.rs"));
    assert!(lib.contains("include!(\"generated_value.rs\")"), "{lib}");
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
fn retains_qualified_json_macro_dependency_without_dead_imports() {
    let workspace = temp_path("rule-qualified-json-macro-workspace");
    let output = temp_path("rule-qualified-json-macro-output");
    let target_dir = temp_path("rule-qualified-json-macro-target");
    write_qualified_json_macro_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("qualified json macro rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("qualified_json_macro_rule/src/lib.rs"));
    assert!(lib.contains("serde_json::json!"), "{lib}");
    assert!(!lib.contains("use serde_json::json"), "{lib}");
    assert!(!lib.contains("dead_payload"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
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

#[test]
fn retains_async_actor_channel_loop_without_dead_command_payloads() {
    let workspace = temp_path("rule-async-actor-loop-workspace");
    let output = temp_path("rule-async-actor-loop-output");
    let target_dir = temp_path("rule-async-actor-loop-target");
    write_async_actor_loop_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("async actor loop rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("async_actor_loop_rule/src/lib.rs"));
    assert!(lib.contains("tokio::spawn(worker_loop(rx))"), "{lib}");
    assert!(lib.contains("mpsc::Sender<Command>"), "{lib}");
    assert!(lib.contains("Live(Payload)"), "{lib}");
    assert!(!lib.contains("DeadPayload"), "{lib}");
    assert!(!lib.contains("dead_record"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_async_select_reconnect_loop_edges_without_dead_events() {
    let workspace = temp_path("rule-async-select-reconnect-workspace");
    let output = temp_path("rule-async-select-reconnect-output");
    let target_dir = temp_path("rule-async-select-reconnect-target");
    write_async_select_reconnect_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("async select reconnect rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("async_select_reconnect_rule/src/lib.rs"));
    assert!(lib.contains("tokio::select!"), "{lib}");
    assert!(lib.contains("broadcast::Sender<Event>"), "{lib}");
    assert!(lib.contains("watch::Receiver<bool>"), "{lib}");
    assert!(lib.contains("Event::Connected"), "{lib}");
    assert!(!lib.contains("DeadEvent"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn treats_unsafe_no_mangle_export_as_builtin_ffi_surface() {
    let workspace = temp_path("rule-ffi-unsafe-no-mangle-workspace");
    let output = temp_path("rule-ffi-unsafe-no-mangle-output");
    let target_dir = temp_path("rule-ffi-unsafe-no-mangle-target");
    write_ffi_unsafe_no_mangle_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("unsafe no_mangle FFI rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    assert!(!report
        .production
        .hazards
        .iter()
        .any(|hazard| hazard.code == "custom_attribute_macros"));
    let lib = read(output.join("ffi_unsafe_no_mangle_rule/src/lib.rs"));
    assert!(lib.contains("#[unsafe(no_mangle)]"), "{lib}");
    assert!(lib.contains("pub extern \"C\" fn selected_bridge"), "{lib}");
    assert!(lib.contains("fn live_offset"), "{lib}");
    assert!(!lib.contains("dead_bridge"), "{lib}");
    assert!(!lib.contains("dead_offset"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_jni_extern_system_entrypoint_without_dead_sibling_exports() {
    let workspace = temp_path("rule-jni-extern-system-workspace");
    let output = temp_path("rule-jni-extern-system-output");
    let target_dir = temp_path("rule-jni-extern-system-target");
    write_jni_extern_system_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("JNI extern system rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("jni_extern_system_rule/src/lib.rs"));
    assert!(
        lib.contains("pub extern \"system\" fn Java_demo_Native_live"),
        "{lib}"
    );
    assert!(lib.contains("use jni::objects::{JClass, JString}"), "{lib}");
    assert!(lib.contains("use jni::sys::jint"), "{lib}");
    assert!(lib.contains("fn live_probe"), "{lib}");
    assert!(!lib.contains("Java_demo_Native_dead"), "{lib}");
    assert!(!lib.contains("dead_probe"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_clap_nested_command_derive_contract() {
    let workspace = temp_path("rule-clap-command-workspace");
    let output = temp_path("rule-clap-command-output");
    let target_dir = temp_path("rule-clap-command-target");
    write_clap_command_contract_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("clap command contract rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("clap_command_rule/src/lib.rs"));
    assert!(lib.contains("#[derive(Parser)]"), "{lib}");
    assert!(lib.contains("#[derive(Subcommand)]"), "{lib}");
    assert!(lib.contains("#[derive(Args)]"), "{lib}");
    assert!(lib.contains("value_delimiter = ','"), "{lib}");
    assert!(lib.contains("conflicts_with = \"raw\""), "{lib}");
    assert!(!lib.contains("DeadCliHelper"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_thiserror_from_source_and_format_capture_contracts() {
    let workspace = temp_path("rule-thiserror-contract-workspace");
    let output = temp_path("rule-thiserror-contract-output");
    let target_dir = temp_path("rule-thiserror-contract-target");
    write_thiserror_contract_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("thiserror contract rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("thiserror_contract_rule/src/lib.rs"));
    assert!(lib.contains("#[derive(Debug, Error)]"), "{lib}");
    assert!(lib.contains("#[from]"), "{lib}");
    assert!(
        lib.contains("#[error(\"protocol {code}: {message}\")]"),
        "{lib}"
    );
    assert!(lib.contains("fn live_message"), "{lib}");
    assert!(!lib.contains("DeadBridgeError"), "{lib}");
    assert!(!lib.contains("dead_message"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_async_io_poll_impls_for_generic_stream_helpers() {
    let workspace = temp_path("rule-async-io-poll-workspace");
    let output = temp_path("rule-async-io-poll-output");
    let target_dir = temp_path("rule-async-io-poll-target");
    write_async_io_poll_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("async I/O poll rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("async_io_poll_rule/src/lib.rs"));
    assert!(lib.contains("impl AsyncRead for BridgeStream"), "{lib}");
    assert!(lib.contains("impl AsyncWrite for BridgeStream"), "{lib}");
    assert!(lib.contains("fn poll_read"), "{lib}");
    assert!(lib.contains("fn poll_shutdown"), "{lib}");
    assert!(!lib.contains("DeadStream"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_runtime_singleton_registry_initializers_without_dead_globals() {
    let workspace = temp_path("rule-runtime-singleton-registry-workspace");
    let output = temp_path("rule-runtime-singleton-registry-output");
    let target_dir = temp_path("rule-runtime-singleton-registry-target");
    write_runtime_singleton_registry_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("runtime singleton registry rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("runtime_singleton_registry_rule/src/lib.rs"));
    assert!(lib.contains("OnceLock<Arc<Runtime>>"), "{lib}");
    assert!(lib.contains("OnceLock<Mutex<Vec<String>>>"), "{lib}");
    assert!(lib.contains("RuntimeBuilder::new()"), "{lib}");
    assert!(lib.contains("fn stack_size"), "{lib}");
    assert!(!lib.contains("DEAD_RUNTIME"), "{lib}");
    assert!(!lib.contains("dead_stack_size"), "{lib}");
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn copies_platform_cfg_assets_and_keeps_extern_bundle_intact() {
    let workspace = temp_path("rule-platform-asset-extern-workspace");
    let output = temp_path("rule-platform-asset-extern-output");
    let target_dir = temp_path("rule-platform-asset-extern-target");
    write_platform_asset_extern_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("platform asset extern rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("platform_asset_extern_rule/src/lib.rs"));
    assert!(
        lib.contains("#[cfg(any(target_os = \"ios\", target_os = \"android\"))]"),
        "{lib}"
    );
    assert!(lib.contains("include_bytes!(\"cacert.pem\")"), "{lib}");
    assert!(lib.contains("unsafe extern \"C\""), "{lib}");
    assert!(lib.contains("fn init_tls_roots"), "{lib}");
    assert!(!lib.contains("dead_platform_probe"), "{lib}");
    assert!(output
        .join("platform_asset_extern_rule/src/cacert.pem")
        .exists());
    assert!(!output
        .join("platform_asset_extern_rule/src/dead.pem")
        .exists());
    assert_cargo_check(&output, &target_dir, &lib);
}

#[test]
fn retains_serde_adjacent_tag_custom_numeric_helpers() {
    let workspace = temp_path("rule-serde-custom-numeric-workspace");
    let output = temp_path("rule-serde-custom-numeric-output");
    let target_dir = temp_path("rule-serde-custom-numeric-target");
    write_serde_custom_numeric_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("serde custom numeric rule should reduce");

    assert_no_error_hazards(&report.production.hazards);
    let lib = read(output.join("serde_custom_numeric_rule/src/lib.rs"));
    assert!(lib.contains("tag = \"type\", content = \"value\""), "{lib}");
    assert!(lib.contains("deserialize_with = \"de_f64\""), "{lib}");
    assert!(lib.contains("serialize_with = \"ser_f64\""), "{lib}");
    assert!(lib.contains("deserialize_with = \"de_opt_u32\""), "{lib}");
    assert!(!lib.contains("dead_de_f64"), "{lib}");
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

fn write_dead_public_glob_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "dead_public_glob_rule",
        r#"use opensourced::opensourced;

pub mod upstream {
    pub struct Shared;
}

pub mod dead_api;
pub use dead_api::*;

#[opensourced]
pub struct Selected {
    pub field: upstream::Shared,
}
"#,
    );
    write(
        root.join("dead_public_glob_rule/src/dead_api.rs"),
        r#"pub use crate::upstream::Shared;

pub fn noise() -> u32 {
    2
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

fn write_renamed_surface_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "renamed_surface_rule",
        r#"use opensourced::opensourced;

mod types {
    pub struct LongName;

    pub trait LongTrait {
        fn mark(&self) -> u32;
    }

    pub struct DeadName;
}

use crate::types::{DeadName as DeadAlias, LongName as PublicAlias, LongTrait as TraitAlias};

pub struct ApiSurface {
    pub item: PublicAlias,
}

impl TraitAlias for ApiSurface {
    fn mark(&self) -> u32 {
        7
    }
}

#[opensourced]
pub fn selected() -> ApiSurface {
    ApiSurface { item: PublicAlias }
}

pub fn dead_api() -> DeadAlias {
    DeadAlias
}
"#,
    );
}

fn write_renamed_dependency_barrel_rule_fixture(root: &Path) {
    write(
        root.join("Cargo.toml"),
        r#"[workspace]
members = ["dependency_barrel_rule", "shared_rule"]
resolver = "2"
"#,
    );
    write(
        root.join("dependency_barrel_rule/Cargo.toml"),
        &format!(
            r#"[package]
name = "dependency_barrel_rule"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
shared_rule = {{ path = "../shared_rule" }}
"#,
            manifest_path(&repo_root().join("crates/opensourced"))
        ),
    );
    write(
        root.join("dependency_barrel_rule/src/lib.rs"),
        r#"use opensourced::opensourced;

#[allow(unused_imports)]
mod dependency_barrel {
    pub mod records {
        pub use shared_rule::dead_shared as dead_shared_barrel;
        pub use shared_rule::{SharedMode as BarrelMode, SharedRecord as BarrelRecord};
    }

    pub mod helpers {
        use super::records::{BarrelMode, BarrelRecord};

        pub fn score_record(record: BarrelRecord, mode: BarrelMode) -> u32 {
            record.value + mode.score()
        }

        pub fn dead_helper() -> u32 {
            99
        }
    }

    pub use helpers::dead_helper as dead_barrel_helper;
    pub use helpers::score_record;
    pub use records::*;
}

#[opensourced]
pub fn selected(value: u32) -> u32 {
    let record = dependency_barrel::BarrelRecord { value };
    let mode = dependency_barrel::BarrelMode::Fast(record.value);
    dependency_barrel::score_record(record, mode)
}
"#,
    );
    write(
        root.join("shared_rule/Cargo.toml"),
        r#"[package]
name = "shared_rule"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        root.join("shared_rule/src/lib.rs"),
        r#"
#[derive(Clone, Copy)]
pub struct SharedRecord {
    pub value: u32,
}

#[derive(Clone, Copy)]
pub enum SharedMode {
    Fast(u32),
    Slow,
}

impl SharedMode {
    pub fn score(&self) -> u32 {
        match self {
            SharedMode::Fast(value) => *value,
            SharedMode::Slow => 1,
        }
    }
}

pub struct DeadRecord;

pub fn dead_shared() -> u32 {
    99
}
"#,
    );
}

fn write_multi_hop_dependency_barrel_rule_fixture(root: &Path) {
    write(
        root.join("Cargo.toml"),
        r#"[workspace]
members = ["multi_hop_barrel_app", "barrel_support", "barrel_leaf"]
resolver = "2"
"#,
    );
    write(
        root.join("multi_hop_barrel_app/Cargo.toml"),
        &format!(
            r#"[package]
name = "multi_hop_barrel_app"
version = "0.1.0"
edition = "2021"

[dependencies]
barrel_support = {{ path = "../barrel_support" }}
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&repo_root().join("crates/opensourced"))
        ),
    );
    write(
        root.join("multi_hop_barrel_app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[allow(unused_imports)]
mod local_facade {
    pub use barrel_support::prelude::{
        dead_leaf_value as dead_leaf_value_alias, dead_score as dead_support_score,
        DeadRecord, LiveMode as Mode, LiveRecord as Record, score,
    };
}

#[opensourced]
pub fn selected(value: u32) -> u32 {
    let record = local_facade::Record { value };
    let mode = local_facade::Mode::Fast(record.value);
    local_facade::score(record, mode)
}

pub fn dead_selected() -> u32 {
    local_facade::dead_support_score() + local_facade::dead_leaf_value_alias()
}
"#,
    );
    write(
        root.join("barrel_support/Cargo.toml"),
        r#"[package]
name = "barrel_support"
version = "0.1.0"
edition = "2021"

[dependencies]
barrel_leaf = { path = "../barrel_leaf" }
"#,
    );
    write(
        root.join("barrel_support/src/lib.rs"),
        r#"pub mod prelude {
    pub use crate::helpers::{dead_score, score};
    pub use barrel_leaf::{
        dead_leaf_value, DeadLeaf as DeadRecord, LeafMode as LiveMode, LeafRecord as LiveRecord,
    };
}

pub mod helpers {
    use barrel_leaf::{dead_leaf_value, LeafMode, LeafRecord};

    pub fn score(record: LeafRecord, mode: LeafMode) -> u32 {
        record.value + mode.score()
    }

    pub fn dead_score() -> u32 {
        dead_leaf_value()
    }
}

pub fn dead_support_root() -> u32 {
    helpers::dead_score()
}
"#,
    );
    write(
        root.join("barrel_leaf/Cargo.toml"),
        r#"[package]
name = "barrel_leaf"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        root.join("barrel_leaf/src/lib.rs"),
        r#"#[derive(Clone, Copy)]
pub struct LeafRecord {
    pub value: u32,
}

#[derive(Clone, Copy)]
pub enum LeafMode {
    Fast(u32),
    Slow,
}

impl LeafMode {
    pub fn score(&self) -> u32 {
        match self {
            LeafMode::Fast(value) => *value,
            LeafMode::Slow => 1,
        }
    }
}

pub struct DeadLeaf;

pub fn dead_leaf_value() -> u32 {
    99
}
"#,
    );
}

fn write_module_scoped_import_liveness_rule_fixture(root: &Path) {
    write_workspace_with_dependencies(
        root,
        "module_import_liveness_rule",
        r#"serde = { version = "1", features = ["derive"] }
thiserror = "1"
"#,
        r#"use opensourced::opensourced;

mod client {
    use serde::{de::DeserializeOwned, Serialize};
    use thiserror::Error;

    pub fn live_client() -> u32 {
        7
    }

    #[derive(Debug, Error)]
    #[error("dead")]
    pub struct DeadError;

    pub fn dead_client<T: Serialize + DeserializeOwned>(value: &T) -> usize {
        core::mem::size_of_val(value)
    }
}

mod wire {
    use serde::Serialize;

    #[derive(Serialize)]
    pub struct LiveWire {
        pub id: u32,
    }
}

#[opensourced]
pub fn selected() -> wire::LiveWire {
    wire::LiveWire {
        id: client::live_client(),
    }
}
"#,
    );
}

fn write_thiserror_import_liveness_rule_fixture(root: &Path) {
    write_workspace_with_dependencies(
        root,
        "thiserror_import_liveness_rule",
        r#"serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
"#,
        r#"use opensourced::opensourced;
use serde::{de::DeserializeOwned, Deserialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Clone, Deserialize)]
pub struct LiveWire {
    pub id: u32,
}

#[derive(Debug, Error)]
#[error("dead wire error: {0}")]
pub struct DeadWireError(serde_json::Error);

fn deserialize_json_value<T>(value: &Value) -> Result<T, serde_json::Error>
where
    T: DeserializeOwned,
{
    T::deserialize(value)
}

#[opensourced]
pub fn selected(value: &Value) -> Result<LiveWire, serde_json::Error> {
    deserialize_json_value(value)
}
"#,
    );
}

fn write_private_field_import_liveness_rule_fixture(root: &Path) {
    write_workspace_with_dependencies(
        root,
        "private_field_import_liveness_rule",
        r#"tokio = { version = "1", features = ["sync"] }
"#,
        r#"use opensourced::opensourced;
use std::sync::Arc;
use tokio::sync::{RwLock, watch};

pub trait Handler: Send + Sync {}

pub struct ReconnectingClient {
    client: Arc<u32>,
    handler: Arc<RwLock<Option<Arc<dyn Handler>>>>,
    connection_tx: watch::Sender<bool>,
}

impl ReconnectingClient {
    pub fn shutdown(&self) -> bool {
        let _ = Arc::strong_count(&self.client);
        self.connection_tx.send_replace(false)
    }

    pub async fn set_handler(&self, handler: Arc<dyn Handler>) {
        let mut guard = self.handler.write().await;
        *guard = Some(handler);
    }
}

#[opensourced]
pub fn selected(client: &ReconnectingClient) -> bool {
    client.shutdown()
}
"#,
    );
}

fn write_private_field_module_mention_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "private_field_module_mention_rule",
        r#"use opensourced::opensourced;

pub enum HandoffAction {
    SendTurn { transcript: String },
    Drain,
}

pub struct TranscriptBuffer {
    text: String,
}

pub struct HandoffManager {
    inner: HandoffManagerInner,
}

struct HandoffManagerInner {
    action_queue: Vec<HandoffAction>,
    transcript: TranscriptBuffer,
}

impl HandoffManager {
    pub fn drain_actions(&mut self) -> Vec<HandoffAction> {
        std::mem::take(&mut self.inner.action_queue)
    }
}

#[opensourced]
pub fn selected(manager: &mut HandoffManager) -> Vec<HandoffAction> {
    manager.drain_actions()
}
"#,
    );
}

fn write_default_trait_method_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "default_trait_rule",
        r#"use opensourced::opensourced;

pub trait Fragment {
    const KIND: &'static str;

    fn render(&self) -> &'static str {
        Self::KIND
    }
}

pub struct EnvFragment;

impl Fragment for EnvFragment {
    const KIND: &'static str = "env";
}

pub struct DeadFragment;

impl Fragment for DeadFragment {
    const KIND: &'static str = "dead";
}

#[opensourced]
pub fn selected(fragment: &EnvFragment) -> &'static str {
    fragment.render()
}
"#,
    );
}

fn write_inline_callback_future_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "inline_callback_future_rule",
        r#"use opensourced::opensourced;
use std::{future::Future, pin::Pin, sync::Arc};

pub struct InlineRegistry {
    callback: Arc<dyn Fn(&str) -> Pin<Box<dyn Future<Output = bool> + Send>> + Send + Sync>,
}

impl InlineRegistry {
    pub fn new(
        callback: Arc<dyn Fn(&str) -> Pin<Box<dyn Future<Output = bool> + Send>> + Send + Sync>,
    ) -> Self {
        Self { callback }
    }
}

pub struct DeadInlineCallback;

#[opensourced]
pub fn selected(
    callback: Arc<dyn Fn(&str) -> Pin<Box<dyn Future<Output = bool> + Send>> + Send + Sync>,
) -> InlineRegistry {
    InlineRegistry::new(callback)
}
"#,
    );
}

fn write_lazy_lock_static_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "lazy_lock_rule",
        r#"use opensourced::opensourced;
use std::sync::LazyLock;

static LIVE: LazyLock<String> = LazyLock::new(|| build_live());

fn build_live() -> String {
    "live".to_string()
}

fn dead_build() -> String {
    "dead".to_string()
}

#[opensourced]
pub fn selected() -> usize {
    LIVE.len()
}
"#,
    );
}

fn write_once_lock_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "once_lock_rule",
        r#"use opensourced::opensourced;
use std::sync::{Arc, OnceLock};

static CLIENT: OnceLock<Arc<Client>> = OnceLock::new();

pub struct Client;

impl Client {
    pub fn new() -> Self {
        Self
    }

    pub fn live(&self) -> u32 {
        7
    }
}

fn ensure_init() {}

fn shared_client() -> Arc<Client> {
    ensure_init();
    CLIENT.get_or_init(|| Arc::new(Client::new())).clone()
}

fn dead_client() -> Client {
    Client
}

#[opensourced]
pub fn selected() -> u32 {
    shared_client().live()
}
"#,
    );
}

fn write_let_else_slice_pattern_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "let_else_slice_rule",
        r#"use opensourced::opensourced;

pub enum Event {
    Live(u32),
    Dead,
}

fn dead_helper() -> Event {
    Event::Dead
}

#[opensourced]
pub fn selected(events: &[Event]) -> u32 {
    let [Event::Live(value)] = events else {
        return 0;
    };
    *value
}
"#,
    );
}

fn write_const_chain_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "const_chain_rule",
        r#"use opensourced::opensourced;

pub const ROWS: u32 = 2;
pub const WIDTH: u32 = 10;
pub const AREA: u32 = ROWS * WIDTH;
pub const NAMES: [&str; ROWS as usize] = ["a", "b"];
pub const DEAD_CONST: u32 = 99;

#[opensourced]
pub fn selected() -> u32 {
    AREA + NAMES.len() as u32
}
"#,
    );
}

fn write_format_capture_const_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "format_capture_const_rule",
        r#"use opensourced::opensourced;

pub const LIVE_PREFIX: &str = "live";
pub const DEAD_PREFIX: &str = "dead";

#[opensourced]
pub fn selected(value: u32) -> String {
    format!("{LIVE_PREFIX}:{value}")
}
"#,
    );
}

fn write_macro_variant_path_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "macro_variant_rule",
        r#"use opensourced::opensourced;

pub enum Request {
    Ping { id: u32 },
    Dead,
}

macro_rules! req {
    ($variant:ident) => {
        Request::$variant { id: next_id() }
    };
}

fn next_id() -> u32 {
    1
}

fn dead_id() -> u32 {
    0
}

#[opensourced]
pub fn selected() -> Request {
    req!(Ping)
}
"#,
    );
}

fn write_expression_macro_receiver_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "expression_macro_receiver_rule",
        r#"use opensourced::opensourced;

macro_rules! passthrough {
    ($expr:expr) => {
        $expr
    };
}

pub struct Payload(u32);

impl Payload {
    pub fn score(&self) -> u32 {
        self.0
    }

    pub fn dead_score(&self) -> u32 {
        99
    }
}

fn make_payload() -> Payload {
    Payload(7)
}

#[opensourced]
pub fn selected() -> u32 {
    passthrough!(make_payload().score())
}
"#,
    );
}

fn write_serde_hook_rule_fixture(root: &Path) {
    write_workspace_with_dependencies(
        root,
        "serde_hook_rule",
        r#"serde = { version = "1", features = ["derive"] }
"#,
        r#"use opensourced::opensourced;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Wire {
    #[serde(serialize_with = "ser_live")]
    #[serde(default, deserialize_with = "de_live")]
    value: Option<u32>,
}

fn ser_live<S>(value: &Option<u32>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    value.serialize(serializer)
}

fn de_live<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<u32>::deserialize(deserializer)
}

fn dead_hook() -> Option<u32> {
    Some(99)
}

#[opensourced]
pub fn selected() -> Wire {
    Wire { value: None }
}
"#,
    );
}

fn write_serde_default_function_rule_fixture(root: &Path) {
    write_workspace_with_dependencies(
        root,
        "serde_default_fn_rule",
        r#"serde = { version = "1", features = ["derive"] }
"#,
        r#"use opensourced::opensourced;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    #[serde(default = "default_mode")]
    mode: Mode,
}

#[derive(Deserialize)]
pub enum Mode {
    Live,
    Dead,
}

fn default_mode() -> Mode {
    Mode::Live
}

fn dead_default() -> Mode {
    Mode::Dead
}

#[opensourced]
pub fn selected() -> Config {
    Config { mode: Mode::Live }
}
"#,
    );
}

fn write_serde_with_module_rule_fixture(root: &Path) {
    write_workspace_with_dependencies(
        root,
        "serde_with_module_rule",
        r#"serde = { version = "1", features = ["derive"] }
"#,
        r#"use opensourced::opensourced;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Wire {
    #[serde(with = "codec")]
    value: Value,
}

#[derive(Serialize, Deserialize)]
pub struct Value(u32);

mod codec {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::Value;

    pub fn serialize<S>(value: &Value, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        value.0.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        u32::deserialize(deserializer).map(Value)
    }
}

mod dead_codec {
    pub fn serialize() {}
}

#[opensourced]
pub fn selected() -> Wire {
    Wire { value: Value(7) }
}
"#,
    );
}

fn write_async_trait_object_rule_fixture(root: &Path) {
    write_workspace_with_dependencies(
        root,
        "async_trait_rule",
        r#"async-trait = "0.1"
"#,
        r#"use opensourced::opensourced;
use std::sync::Arc;

pub struct Live;

pub struct Error;

#[async_trait::async_trait]
pub trait Transport: Send + Sync {
    async fn reconnect(&self) -> Result<Live, Error>;
}

pub struct DeadTransport;

#[opensourced]
pub fn selected(handler: Arc<dyn Transport + Send + Sync>) -> Arc<dyn Transport + Send + Sync> {
    handler
}
"#,
    );
}

fn write_returned_trait_object_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "returned_dyn_rule",
        r#"
use opensourced::opensourced;

pub trait Reader {
    fn read(&self) -> u32;
}

pub struct LiveReader;

impl Reader for LiveReader {
    fn read(&self) -> u32 {
        1
    }
}

#[opensourced]
pub fn selected() -> Box<dyn Reader> {
    Box::new(LiveReader)
}
"#,
    );
}

fn write_forwarded_trait_object_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "forwarded_dyn_rule",
        r#"
use opensourced::opensourced;

pub trait Reader {
    fn read(&self) -> u32;
}

pub struct DeadReader;

impl Reader for DeadReader {
    fn read(&self) -> u32 {
        0
    }
}

#[opensourced]
pub fn selected(reader: Box<dyn Reader>) -> Box<dyn Reader> {
    {
        let _unused = DeadReader;
        reader
    }
}
"#,
    );
}

fn write_ffi_static_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "ffi_static_rule",
        r#"use std::os::raw::c_int;

use opensourced::opensourced;

extern "C" {
    static LIVE_FLAG: c_int;
    static DEAD_FLAG: c_int;
}

#[opensourced]
pub fn selected() -> i32 {
    unsafe { LIVE_FLAG as i32 }
}
"#,
    );
}

fn write_ffi_callback_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "ffi_callback_rule",
        r#"use std::os::raw::c_char;

use opensourced::opensourced;

#[opensourced]
#[no_mangle]
pub extern "C" fn register(cb: extern "C" fn(*const c_char), value: *const c_char) {
    cb(value);
}

#[no_mangle]
pub extern "C" fn dead_register(_cb: extern "C" fn(*const c_char)) {}
"#,
    );
}

fn write_ffi_extern_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "ffi_extern_rule",
        r#"use opensourced::opensourced;

extern "C" {
    fn live_c() -> i32;
    fn dead_c() -> i32;
}

#[opensourced]
#[no_mangle]
pub extern "C" fn selected() -> i32 {
    unsafe { live_c() }
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

fn write_pruned_private_dyn_field_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "pruned_private_dyn_field_rule",
        r#"use opensourced::opensourced;

pub(crate) struct Api {
    pub id: u32,
    callback: Box<dyn Fn() + Send + Sync>,
}

pub(crate) fn build() -> Api {
    Api {
        id: 7,
        callback: Box::new(|| {}),
    }
}

#[opensourced]
pub(crate) fn selected(api: &Api) -> u32 {
    api.id
}
"#,
    );
}

fn write_private_generic_field_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "private_generic_field_rule",
        r#"use opensourced::opensourced;

pub struct Heavy<T> {
    value: T,
}

pub struct Api<T> {
    pub value: T,
    dead: Heavy<T>,
}

impl<T> Api<T> {
    pub fn new(value: T, dead: T) -> Self {
        Self {
            value,
            dead: Heavy { value: dead },
        }
    }
}

#[opensourced]
pub fn selected(api: Api<u32>) -> u32 {
    api.value
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

fn write_option_env_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "option_env_rule",
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected() -> Option<&'static str> {
    option_env!("LITTER_PROFILE")
}

pub fn dead_profile() -> Option<&'static str> {
    option_env!("DEAD_PROFILE")
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

pub fn out_dir_generated_helper() -> u32 {
    42
}

pub fn dead_out_dir_helper() -> u32 {
    99
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
    fs::write(
        out_dir.join("generated.rs"),
        "pub fn generated_value() -> u32 { super::out_dir_generated_helper() }\n",
    )
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

fn write_fixed_name_item_macro_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "fixed_name_item_macro_rule",
        r#"use opensourced::opensourced;

macro_rules! declare_runtime {
    ($builder:ident) => {
        pub fn generated() -> u32 {
            $builder()
        }
    };
}

fn build_live() -> u32 {
    7
}

fn build_dead() -> u32 {
    99
}

declare_runtime!(build_live);

#[opensourced]
pub fn selected() -> u32 {
    generated()
}
"#,
    );
}

fn write_inline_item_macro_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "inline_item_macro_rule",
        r#"use opensourced::opensourced;

mod helpers {
    pub fn live_helper() -> u32 {
        7
    }

    pub fn dead_helper() -> u32 {
        99
    }
}

mod generated {
    macro_rules! make_api {
        () => {
            pub fn generated() -> u32 {
                crate::helpers::live_helper()
            }
        };
    }

    make_api!();
}

#[opensourced]
pub fn selected() -> u32 {
    generated::generated()
}
"#,
    );
}

fn write_shadowed_macro_scope_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "shadowed_macro_scope_rule",
        r#"use opensourced::opensourced;

mod live {
    macro_rules! declare {
        () => {
            pub fn generated() -> u32 {
                live_helper()
            }
        };
    }

    fn live_helper() -> u32 {
        7
    }

    declare!();
}

mod dead {
    macro_rules! declare {
        () => {
            pub fn generated() -> u32 {
                dead_helper()
            }
        };
    }

    fn dead_helper() -> u32 {
        99
    }

    declare!();
}

#[opensourced]
pub fn selected() -> u32 {
    live::generated()
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

fn write_bidirectional_from_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "bidirectional_from_rule",
        r#"use opensourced::opensourced;

pub struct Internal {
    pub id: u32,
}

impl Internal {
    pub fn new(id: u32) -> Self {
        Self { id }
    }
}

pub struct Wire {
    pub id: u32,
}

impl From<Internal> for Wire {
    fn from(value: Internal) -> Self {
        Self { id: value.id }
    }
}

impl From<Wire> for Internal {
    fn from(value: Wire) -> Self {
        Self { id: value.id }
    }
}

pub struct DeadWire {
    pub id: u32,
}

impl From<DeadWire> for Internal {
    fn from(value: DeadWire) -> Self {
        Self { id: value.id }
    }
}

#[opensourced]
pub fn selected(id: u32) -> u32 {
    let wire: Wire = Internal::new(id).into();
    let internal: Internal = wire.into();
    internal.id
}
"#,
    );
}

fn write_struct_field_into_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "struct_field_into_rule",
        r#"use opensourced::opensourced;

pub struct PublicAgent {
    pub name: String,
    pub mode: PublicMode,
}

pub enum PublicMode {
    Websocket,
    Jsonl,
}

struct WireAgent {
    name: String,
    mode: WireMode,
}

enum WireMode {
    Websocket,
    Jsonl,
}

enum DeadWireMode {
    Http,
}

impl From<WireMode> for PublicMode {
    fn from(value: WireMode) -> Self {
        match value {
            WireMode::Websocket => Self::Websocket,
            WireMode::Jsonl => Self::Jsonl,
        }
    }
}

impl From<DeadWireMode> for PublicMode {
    fn from(_: DeadWireMode) -> Self {
        Self::Websocket
    }
}

fn load_wire_agents() -> Vec<WireAgent> {
    vec![WireAgent {
        name: "codex".to_string(),
        mode: WireMode::Jsonl,
    }]
}

#[opensourced]
pub fn selected() -> Vec<PublicAgent> {
    load_wire_agents()
        .into_iter()
        .map(|agent| PublicAgent {
            name: agent.name,
            mode: agent.mode.into(),
        })
        .collect()
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

fn write_nested_callback_store_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "nested_callback_store_rule",
        r#"use opensourced::opensourced;
use std::sync::{Arc, RwLock};

pub trait ReconnectCallback {
    fn reconnect(&self, attempt: u32);
}

pub trait DeadCallback {
    fn reconnect(&self);
}

pub struct ReconnectStore {
    callback: Arc<RwLock<Option<Arc<dyn ReconnectCallback + Send + Sync>>>>,
}

impl ReconnectStore {
    pub fn new() -> Self {
        Self {
            callback: Arc::new(RwLock::new(None)),
        }
    }

    pub fn set(&self, callback: Arc<dyn ReconnectCallback + Send + Sync>) {
        *self.callback.write().unwrap() = Some(callback);
    }
}

#[opensourced]
pub fn selected() -> ReconnectStore {
    ReconnectStore::new()
}
"#,
    );
}

fn write_trait_object_cast_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "trait_object_cast_rule",
        r#"use opensourced::opensourced;
use std::sync::Arc;

pub struct Session;

impl Session {
    pub fn new() -> Self {
        Self
    }
}

pub struct DeadSession;

#[opensourced]
pub fn selected() -> Option<Arc<dyn Send + Sync>> {
    Some(Arc::new(Session::new()) as Arc<dyn Send + Sync>)
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

fn write_wrapped_callback_boundary_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "wrapped_callback_boundary_rule",
        r#"use opensourced::opensourced;

pub trait Callback {
    fn handle(&self, value: u32) -> u32;
}

pub trait DeadCallback {
    fn handle(&self) -> u32;
}

pub struct Error;

#[opensourced]
pub fn selected(handler: Option<&dyn Callback>, callback: Result<fn(u32) -> u32, Error>) -> u32 {
    let value = callback.map(|callback| callback(7)).unwrap_or(0);
    handler.map(|handler| handler.handle(value)).unwrap_or(value)
}
"#,
    );
}

fn write_pruned_trait_method_hazard_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "pruned_trait_method_hazard_rule",
        r#"use opensourced::opensourced;

pub trait Callback {
    fn live(&self) -> u32;

    fn dead_hook(&self, callback: fn(u32) -> u32) -> u32 {
        callback(1)
    }
}

#[opensourced]
pub fn selected(_handler: &dyn Callback) -> u32 {
    7
}
"#,
    );
}

fn write_bare_dyn_alias_once_lock_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "bare_dyn_alias_once_lock_rule",
        r#"use opensourced::opensourced;
use std::sync::{Arc, OnceLock};

pub enum Direction {
    In,
    Out,
}

pub type Observer = dyn Fn(Direction, &str) + Send + Sync + 'static;

static OBSERVER: OnceLock<Arc<Observer>> = OnceLock::new();

pub type DeadObserver = dyn Fn() + Send + Sync + 'static;

#[opensourced]
pub fn selected(observer: Arc<Observer>) -> bool {
    OBSERVER.set(observer).is_ok()
}
"#,
    );
}

fn write_boxed_io_alias_facade_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "boxed_io_alias_facade_rule",
        r#"use opensourced::opensourced;

mod io_facade {
    pub mod types {
        use std::io::{Read, Write};

        pub type Input = Box<dyn Read + Send>;

        pub type Output = Box<dyn Write + Send>;

        pub type DeadAlias = Output;
    }

    pub use types::{DeadAlias, Input, Output};
}

#[opensourced]
pub fn selected(_input: io_facade::Input) -> usize {
    0
}

pub fn dead_selected(_output: io_facade::Output) -> usize {
    99
}
"#,
    );
}

fn write_facade_const_alias_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "facade_const_alias_rule",
        r#"use opensourced::opensourced;

mod constants {
    pub const LIVE_LIMIT: u32 = 7;

    pub const DEAD_LIMIT: u32 = 99;
}

mod facade {
    pub use crate::constants::{DEAD_LIMIT as DeadLimit, LIVE_LIMIT as LiveLimit};
}

#[opensourced]
pub fn selected() -> u32 {
    facade::LiveLimit
}
"#,
    );
}

fn write_facade_function_alias_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "facade_function_alias_rule",
        r#"use opensourced::opensourced;

mod api {
    pub fn live_value() -> u32 {
        7
    }

    pub fn dead_value() -> u32 {
        99
    }
}

mod facade {
    pub use crate::api::{dead_value as public_dead, live_value as public_live};
}

#[opensourced]
pub fn selected() -> u32 {
    facade::public_live()
}
"#,
    );
}

fn write_result_alias_surface_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "result_alias_surface_rule",
        r#"use opensourced::opensourced;

pub enum AppError {
    Missing,
}

pub enum DeadError {
    Dead,
}

pub type AppResult<T> = Result<T, AppError>;

pub struct StartResponse {
    pub id: String,
}

pub struct DeadResponse {
    pub id: String,
}

#[opensourced]
pub fn selected(id: String) -> AppResult<StartResponse> {
    Ok(StartResponse { id })
}
"#,
    );
}

fn write_arc_handle_record_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "arc_handle_record_rule",
        r#"use opensourced::opensourced;
use std::sync::Arc;

pub struct PairHostHandle {
    id: String,
}

impl PairHostHandle {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

pub struct PairHostInfo {
    pub label: String,
}

pub struct PairHostStartResult {
    pub handle: Arc<PairHostHandle>,
    pub info: PairHostInfo,
}

pub struct DeadPairHostHandle;

pub struct DeadPairHostStartResult {
    pub handle: Arc<DeadPairHostHandle>,
}

#[opensourced]
pub fn selected(id: String) -> PairHostStartResult {
    PairHostStartResult {
        handle: Arc::new(PairHostHandle::new(id.clone())),
        info: PairHostInfo { label: id },
    }
}
"#,
    );
}

fn write_nested_option_vec_dto_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "nested_option_vec_dto_rule",
        r#"use opensourced::opensourced;

pub struct AppDynamicToolSpec {
    pub name: String,
}

pub struct DeadDynamicToolSpec {
    pub name: String,
}

pub struct AppStartThreadRequest {
    pub model: Option<String>,
    pub dynamic_tools: Option<Vec<AppDynamicToolSpec>>,
}

pub struct DeadStartThreadRequest {
    pub dynamic_tools: Option<Vec<DeadDynamicToolSpec>>,
}

#[opensourced]
pub fn selected(name: String) -> AppStartThreadRequest {
    AppStartThreadRequest {
        model: None,
        dynamic_tools: Some(vec![AppDynamicToolSpec { name }]),
    }
}
"#,
    );
}

fn write_mirror_try_from_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "mirror_try_from_rule",
        r#"use opensourced::opensourced;
use std::convert::{TryFrom, TryInto};

pub enum RpcClientError {
    Serialization(String),
}

pub struct AppDynamicToolSpec {
    pub name: String,
    pub input_schema_json: String,
}

pub struct AppStartThreadRequest {
    pub model: Option<String>,
    pub dynamic_tools: Option<Vec<AppDynamicToolSpec>>,
}

pub struct AppResumeThreadRequest {
    pub thread_id: String,
}

pub mod upstream {
    pub struct DynamicToolSpec {
        pub name: String,
        pub input_schema: String,
    }

    pub struct ThreadStartParams {
        pub model: Option<String>,
        pub dynamic_tools: Option<Vec<DynamicToolSpec>>,
    }

    pub struct ThreadResumeParams {
        pub thread_id: String,
    }
}

fn parse_schema(value: String) -> Result<String, RpcClientError> {
    Ok(value)
}

fn dead_parse_schema(value: String) -> Result<String, RpcClientError> {
    Ok(value)
}

impl TryFrom<AppStartThreadRequest> for upstream::ThreadStartParams {
    type Error = RpcClientError;

    fn try_from(value: AppStartThreadRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            model: value.model,
            dynamic_tools: value
                .dynamic_tools
                .map(|tools| {
                    tools
                        .into_iter()
                        .map(|spec| {
                            Ok(upstream::DynamicToolSpec {
                                name: spec.name,
                                input_schema: parse_schema(spec.input_schema_json)?,
                            })
                        })
                        .collect::<Result<Vec<_>, RpcClientError>>()
                })
                .transpose()?,
        })
    }
}

impl TryFrom<AppResumeThreadRequest> for upstream::ThreadResumeParams {
    type Error = RpcClientError;

    fn try_from(value: AppResumeThreadRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            thread_id: value.thread_id,
        })
    }
}

#[opensourced]
pub fn selected(request: AppStartThreadRequest) -> Result<upstream::ThreadStartParams, RpcClientError> {
    request.try_into()
}
"#,
    );
}

fn write_section_include_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "section_include_rule",
        r#"use opensourced::opensourced;

const LIVE_GUIDE: &str = include_str!("guides/live.md");
const DEAD_GUIDE: &str = include_str!("guides/dead.md");

fn section_content(section: &str) -> Option<&'static str> {
    match section {
        "live" => Some(LIVE_GUIDE),
        _ => None,
    }
}

pub fn dead_section_content() -> &'static str {
    DEAD_GUIDE
}

#[opensourced]
pub fn selected() -> Option<&'static str> {
    section_content("live")
}
"#,
    );
    write(
        root.join("section_include_rule/src/guides/live.md"),
        "live\n",
    );
    write(
        root.join("section_include_rule/src/guides/dead.md"),
        "dead\n",
    );
}

fn write_runtime_facade_singleton_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "runtime_facade_singleton_rule",
        r#"use opensourced::opensourced;
use std::sync::{Arc, OnceLock};

pub struct MobileClient {
    id: u32,
}

impl MobileClient {
    pub fn id(&self) -> u32 {
        self.id
    }
}

mod runtime {
    use super::{Arc, MobileClient, OnceLock};

    static CLIENT: OnceLock<Arc<MobileClient>> = OnceLock::new();

    pub fn shared_mobile_client() -> Arc<MobileClient> {
        Arc::clone(CLIENT.get_or_init(|| Arc::new(MobileClient { id: 7 })))
    }

    pub fn dead_shared_mobile_client() -> Arc<MobileClient> {
        Arc::new(MobileClient { id: 99 })
    }

    pub fn dead_shared_runtime() -> u32 {
        99
    }
}

mod ffi_shared {
    pub use crate::runtime::{
        dead_shared_mobile_client as dead_shared_client, shared_mobile_client as shared_client,
    };
}

#[opensourced]
pub fn selected() -> u32 {
    ffi_shared::shared_client().id()
}
"#,
    );
}

fn write_map_payload_dto_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "map_payload_dto_rule",
        r#"use opensourced::opensourced;
use std::collections::BTreeMap;

pub struct ThreadSnapshot {
    pub id: String,
}

pub struct DeadThreadSnapshot {
    pub id: String,
}

pub struct AppSnapshot {
    pub threads: BTreeMap<String, ThreadSnapshot>,
}

pub struct DeadAppSnapshot {
    pub threads: BTreeMap<String, DeadThreadSnapshot>,
}

#[opensourced]
pub fn selected(id: String) -> AppSnapshot {
    let mut threads = BTreeMap::new();
    threads.insert(id.clone(), ThreadSnapshot { id });
    AppSnapshot { threads }
}
"#,
    );
}

fn write_dead_impl_method_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "dead_impl_method_rule",
        r#"use opensourced::opensourced;

pub struct Store {
    value: u32,
}

impl Store {
    pub fn new(value: u32) -> Self {
        Self {
            value: Self::private_seed(value),
        }
    }

    fn private_seed(value: u32) -> u32 {
        value + 1
    }

    fn dead_private_seed() -> u32 {
        99
    }

    pub fn live_score(&self) -> u32 {
        self.value
    }

    pub fn dead_score(&self) -> u32 {
        Self::dead_private_seed()
    }
}

#[opensourced]
pub fn selected(value: u32) -> u32 {
    Store::new(value).live_score()
}
"#,
    );
}

fn write_type_alias_chain_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "type_alias_chain_rule",
        r#"use opensourced::opensourced;

pub struct WireMessage {
    pub text: String,
}

pub type MessageBatch = Vec<WireMessage>;

pub type PublicBatch = MessageBatch;

pub struct DeadWireMessage {
    pub text: String,
}

pub type DeadBatch = Vec<DeadWireMessage>;

#[opensourced]
pub fn selected(text: String) -> PublicBatch {
    vec![WireMessage { text }]
}
"#,
    );
}

fn write_newtype_tuple_surface_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "newtype_tuple_surface_rule",
        r#"use opensourced::opensourced;

pub struct SessionId(pub String);

pub struct StartResult(pub SessionId);

pub struct DeadSessionId(pub String);

pub struct DeadStartResult(pub DeadSessionId);

#[opensourced]
pub fn selected(id: String) -> StartResult {
    StartResult(SessionId(id))
}
"#,
    );
}

fn write_impl_trait_iterator_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "impl_trait_iterator_rule",
        r#"use opensourced::opensourced;

pub struct EventDto {
    pub id: String,
}

pub struct DeadEventDto {
    pub id: String,
}

fn make_event(id: String) -> EventDto {
    EventDto { id }
}

pub fn dead_events(id: String) -> impl Iterator<Item = DeadEventDto> {
    vec![DeadEventDto { id }].into_iter()
}

#[opensourced]
pub fn selected(id: String) -> impl Iterator<Item = EventDto> {
    vec![make_event(id)].into_iter()
}
"#,
    );
}

fn write_question_mark_error_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "question_mark_error_rule",
        r#"use opensourced::opensourced;

pub enum ParseError {
    Bad,
}

pub enum AppError {
    Parse(ParseError),
}

impl From<ParseError> for AppError {
    fn from(value: ParseError) -> Self {
        Self::Parse(value)
    }
}

pub enum DeadError {
    Dead,
}

impl From<DeadError> for AppError {
    fn from(_value: DeadError) -> Self {
        Self::Parse(ParseError::Bad)
    }
}

fn parse_wire(value: String) -> Result<u32, ParseError> {
    value.parse::<u32>().map_err(|_error| ParseError::Bad)
}

fn dead_parse_wire() -> Result<u32, DeadError> {
    Err(DeadError::Dead)
}

#[opensourced]
pub fn selected(value: String) -> Result<u32, AppError> {
    let parsed = parse_wire(value)?;
    Ok(parsed + 1)
}
"#,
    );
}

fn write_enum_variant_constructor_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "enum_variant_constructor_rule",
        r#"use opensourced::opensourced;

pub struct Started {
    pub id: String,
}

pub enum Event {
    Started(Started),
    Stopped,
}

pub struct DeadStarted {
    pub id: String,
}

pub enum DeadEvent {
    DeadStarted(DeadStarted),
}

#[opensourced]
pub fn selected(ids: Vec<String>) -> Vec<Event> {
    ids.into_iter()
        .map(|id| Started { id })
        .map(Event::Started)
        .collect()
}
"#,
    );
}

fn write_struct_update_default_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "struct_update_default_rule",
        r#"use opensourced::opensourced;

#[derive(Default)]
pub struct RuntimeConfig {
    pub enabled: bool,
    pub label: String,
}

#[derive(Default)]
pub struct DeadRuntimeConfig {
    pub enabled: bool,
}

#[opensourced]
pub fn selected(label: String) -> RuntimeConfig {
    RuntimeConfig {
        enabled: true,
        label,
        ..Default::default()
    }
}
"#,
    );
}

fn write_method_reference_map_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "method_reference_map_rule",
        r#"use opensourced::opensourced;

pub struct Score {
    value: u32,
}

impl Score {
    pub fn from_raw(value: u32) -> Self {
        Self { value }
    }

    pub fn value(self) -> u32 {
        self.value
    }

    pub fn dead_value(&self) -> u32 {
        99
    }
}

#[opensourced]
pub fn selected(values: Vec<u32>) -> Vec<u32> {
    values
        .into_iter()
        .map(Score::from_raw)
        .map(Score::value)
        .collect()
}
"#,
    );
}

fn write_boxed_future_return_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "boxed_future_return_rule",
        r#"use opensourced::opensourced;
use std::{future::Future, pin::Pin};

pub struct Response {
    pub ok: bool,
}

pub struct DeadResponse {
    pub ok: bool,
}

pub type ResponseFuture = Pin<Box<dyn Future<Output = Response> + Send>>;

pub type DeadFuture = Pin<Box<dyn Future<Output = DeadResponse> + Send>>;

#[opensourced]
pub fn selected() -> ResponseFuture {
    Box::pin(async { Response { ok: true } })
}

pub fn dead_selected() -> DeadFuture {
    Box::pin(async { DeadResponse { ok: false } })
}
"#,
    );
}

fn write_const_generic_array_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "const_generic_array_rule",
        r#"use opensourced::opensourced;

pub const LIVE_FRAME_LEN: usize = 4;

pub const DEAD_FRAME_LEN: usize = 8;

pub struct FrameBytes {
    pub bytes: [u8; LIVE_FRAME_LEN],
}

pub struct DeadFrameBytes {
    pub bytes: [u8; DEAD_FRAME_LEN],
}

#[opensourced]
pub fn selected(bytes: [u8; LIVE_FRAME_LEN]) -> FrameBytes {
    FrameBytes { bytes }
}
"#,
    );
}

fn write_nested_result_option_alias_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "nested_result_option_alias_rule",
        r#"use opensourced::opensourced;

pub struct Payload {
    pub id: String,
}

pub struct DeadPayload {
    pub id: String,
}

pub enum ApiError {
    Missing,
}

pub type MaybePayload = Option<Payload>;

pub type ApiResult = Result<MaybePayload, ApiError>;

pub type DeadResult = Result<Option<DeadPayload>, ApiError>;

#[opensourced]
pub fn selected(id: Option<String>) -> ApiResult {
    Ok(id.map(|id| Payload { id }))
}
"#,
    );
}

fn write_uniffi_callback_facade_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "uniffi_callback_facade_rule",
        r#"use opensourced::opensourced;

pub mod boundary {
    #[cfg_attr(feature = "ffi", derive(uniffi::Record))]
    pub struct Credential {
        pub user: String,
    }

    pub struct DeadCredential {
        pub user: String,
    }

    #[cfg_attr(feature = "ffi", uniffi::export(callback_interface))]
    pub trait CredentialProvider: Send + Sync {
        fn load(&self, key: String) -> Option<Credential>;
    }

    pub trait DeadProvider: Send + Sync {
        fn dead_load(&self) -> Option<DeadCredential>;
    }
}

pub mod ffi {
    pub use crate::boundary::{Credential, CredentialProvider};
    pub use crate::boundary::{DeadCredential, DeadProvider};
}

#[opensourced]
pub fn selected(provider: &dyn ffi::CredentialProvider, key: String) -> Option<ffi::Credential> {
    provider.load(key)
}
"#,
    );
}

fn write_returned_subscription_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "returned_subscription_rule",
        r#"use opensourced::opensourced;

#[cfg_attr(feature = "ffi", derive(uniffi::Object))]
pub struct Manager;

#[cfg_attr(feature = "ffi", derive(uniffi::Object))]
pub struct StreamSubscription {
    remaining: u32,
}

pub struct StreamEvent {
    pub value: u32,
}

pub enum StreamError {
    Closed,
}

#[cfg_attr(feature = "ffi", derive(uniffi::Object))]
pub struct DeadSubscription;

pub struct DeadEvent {
    pub value: u32,
}

#[cfg_attr(feature = "ffi", uniffi::export)]
impl Manager {
    pub fn subscribe(&self) -> StreamSubscription {
        StreamSubscription { remaining: 1 }
    }
}

#[cfg_attr(feature = "ffi", uniffi::export(async_runtime = "tokio"))]
impl StreamSubscription {
    pub async fn next_event(&self) -> Result<StreamEvent, StreamError> {
        if self.remaining == 0 {
            Err(StreamError::Closed)
        } else {
            Ok(StreamEvent {
                value: self.remaining,
            })
        }
    }
}

impl DeadSubscription {
    pub async fn dead_event(&self) -> DeadEvent {
        DeadEvent { value: 99 }
    }
}

#[opensourced]
pub fn selected(manager: &Manager) -> StreamSubscription {
    manager.subscribe()
}
"#,
    );
}

fn write_split_wrapper_impl_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "split_wrapper_impl_rule",
        r#"use opensourced::opensourced;

#[cfg_attr(feature = "ffi", derive(uniffi::Object))]
pub struct Manager {
    seed: u32,
}

pub struct Internal {
    value: u32,
}

impl Manager {
    pub fn new(seed: u32) -> Self {
        Self { seed }
    }

    fn private_helper(&self, value: Internal) -> u32 {
        self.seed + value.value
    }

    fn dead_helper(&self) -> u32 {
        99
    }
}

fn convert(value: u32) -> Internal {
    Internal { value }
}

#[cfg_attr(feature = "ffi", uniffi::export)]
impl Manager {
    pub fn exported_wrapper(&self, value: u32) -> u32 {
        self.private_helper(convert(value))
    }

    pub fn dead_export(&self) -> u32 {
        self.dead_helper()
    }
}

#[opensourced]
pub fn selected(seed: u32, value: u32) -> u32 {
    Manager::new(seed).exported_wrapper(value)
}
"#,
    );
}

fn write_serde_patch_path_rule_fixture(root: &Path) {
    write_workspace_with_dependencies(
        root,
        "serde_patch_path_rule",
        r#"serde = { version = "1", features = ["derive"] }
serde_json = "1"
"#,
        r#"use opensourced::opensourced;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum PathSegment {
    Index(usize),
    Key(String),
}

#[derive(Serialize, Deserialize)]
pub struct Patch {
    pub path: Vec<PathSegment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Change {
    Snapshot {
        #[serde(rename = "snapshot")]
        value: serde_json::Value,
    },
    Patches {
        patches: Vec<Patch>,
    },
}

#[derive(Serialize, Deserialize)]
pub struct Params {
    pub change: Change,
}

#[derive(Serialize, Deserialize)]
pub struct DeadPatch {
    pub path: Vec<String>,
}

#[opensourced]
pub fn selected(input: &str) -> serde_json::Result<Params> {
    serde_json::from_str::<Params>(input)
}
"#,
    );
}

fn write_method_dispatch_rule_fixture(root: &Path) {
    write_workspace_with_dependencies(
        root,
        "method_dispatch_rule",
        r#"serde = { version = "1", features = ["derive"] }
serde_json = "1"
"#,
        r#"use opensourced::opensourced;
use serde::Deserialize;

pub enum Method {
    Start,
    Stop,
    Unknown(String),
}

impl Method {
    pub fn from_wire(value: &str) -> Self {
        match value {
            "start" => Self::Start,
            "stop" => Self::Stop,
            other => Self::Unknown(other.to_string()),
        }
    }
}

pub struct Request {
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Deserialize)]
pub struct StartParams {
    pub model: String,
}

#[derive(Deserialize)]
pub struct StopParams {
    pub id: String,
}

pub enum TypedRequest {
    Start(StartParams),
    Unknown {
        method: String,
        params: serde_json::Value,
    },
}

impl TypedRequest {
    pub fn from_request(request: Request) -> serde_json::Result<Self> {
        match Method::from_wire(&request.method) {
            Method::Start => Ok(Self::Start(serde_json::from_value(request.params)?)),
            Method::Unknown(method) => Ok(Self::Unknown {
                method,
                params: request.params,
            }),
            Method::Stop => Ok(Self::Unknown {
                method: "stop".to_string(),
                params: request.params,
            }),
        }
    }
}

#[opensourced]
pub fn selected(request: Request) -> serde_json::Result<TypedRequest> {
    TypedRequest::from_request(request)
}
"#,
    );
}

fn write_try_from_option_vec_transpose_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "try_from_transpose_rule",
        r#"use opensourced::opensourced;
use std::convert::{TryFrom, TryInto};

pub enum RpcError {
    BadSchema,
}

pub struct AppToolSpec {
    pub name: String,
    pub schema_json: String,
}

pub struct AppRequest {
    pub tools: Option<Vec<AppToolSpec>>,
}

pub struct DeadToolSpec {
    pub name: String,
}

pub mod upstream {
    pub struct DynamicToolSpec {
        pub name: String,
        pub schema: String,
    }

    pub struct Params {
        pub tools: Option<Vec<DynamicToolSpec>>,
    }
}

fn parse_schema(value: String) -> Result<String, RpcError> {
    if value.is_empty() {
        Err(RpcError::BadSchema)
    } else {
        Ok(value)
    }
}

impl TryFrom<AppToolSpec> for upstream::DynamicToolSpec {
    type Error = RpcError;

    fn try_from(value: AppToolSpec) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value.name,
            schema: parse_schema(value.schema_json)?,
        })
    }
}

impl TryFrom<AppRequest> for upstream::Params {
    type Error = RpcError;

    fn try_from(value: AppRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            tools: value
                .tools
                .map(|tools| {
                    tools
                        .into_iter()
                        .map(|tool| tool.try_into())
                        .collect::<Result<Vec<_>, RpcError>>()
                })
                .transpose()?,
        })
    }
}

#[opensourced]
pub fn selected(request: AppRequest) -> Result<upstream::Params, RpcError> {
    request.try_into()
}
"#,
    );
}

fn write_local_static_include_bytes_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "local_static_include_bytes_rule",
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected() -> usize {
    static CERT: &[u8] = include_bytes!("certs/live.pem");
    CERT.len()
}

pub fn dead_cert_len() -> usize {
    static CERT: &[u8] = include_bytes!("certs/dead.pem");
    CERT.len()
}
"#,
    );
    write(
        root.join("local_static_include_bytes_rule/src/certs/live.pem"),
        "live",
    );
    write(
        root.join("local_static_include_bytes_rule/src/certs/dead.pem"),
        "dead",
    );
}

fn write_macro_env_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "macro_env_rule",
        r#"use opensourced::opensourced;

macro_rules! find_resource {
    () => {{
        option_env!("BAZEL_PACKAGE").unwrap_or(env!("CARGO_MANIFEST_DIR"))
    }};
}

#[opensourced]
pub fn selected() -> &'static str {
    find_resource!()
}

pub fn dead_resource() -> &'static str {
    env!("DEAD_RESOURCE")
}
"#,
    );
}

fn write_dependency_crate_alias_rule_fixture(root: &Path) {
    write(
        root.join("Cargo.toml"),
        r#"[workspace]
members = ["dependency_alias_app", "protocol-rule"]
resolver = "2"
"#,
    );
    write(
        root.join("dependency_alias_app/Cargo.toml"),
        &format!(
            r#"[package]
name = "dependency_alias_app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
protocol_runtime = {{ package = "protocol-rule", path = "../protocol-rule" }}
"#,
            manifest_path(&repo_root().join("crates/opensourced"))
        ),
    );
    write(
        root.join("dependency_alias_app/src/lib.rs"),
        r#"use opensourced::opensourced;
use protocol_runtime as upstream;

#[opensourced]
pub fn selected(id: String) -> upstream::LiveType {
    upstream::LiveType { id }
}

pub fn dead_selected(id: String) -> upstream::DeadType {
    upstream::DeadType { id }
}
"#,
    );
    write(
        root.join("protocol-rule/Cargo.toml"),
        r#"[package]
name = "protocol-rule"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        root.join("protocol-rule/src/lib.rs"),
        r#"pub struct LiveType {
    pub id: String,
}

pub struct DeadType {
    pub id: String,
}
"#,
    );
}

fn write_private_child_wildcard_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "private_child_wildcard_rule",
        r#"use opensourced::opensourced;

mod live {
    pub fn selected_helper() -> u32 {
        live_value()
    }

    pub fn live_value() -> u32 {
        7
    }

    pub fn dead_value() -> u32 {
        99
    }
}

mod dead {
    pub fn dead_helper() -> u32 {
        99
    }
}

use self::live::*;
pub use live::selected_helper;
pub use dead::dead_helper;

#[opensourced]
pub fn selected() -> u32 {
    selected_helper()
}
"#,
    );
}

fn write_nested_private_facade_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "nested_private_facade_rule",
        r#"use opensourced::opensourced;

mod protocol {
    mod inner {
        pub struct LiveClient {
            pub id: String,
        }

        pub struct DeadClient {
            pub id: String,
        }

        impl LiveClient {
            pub fn new(id: String) -> Self {
                Self { id }
            }
        }

        pub fn live_factory(id: String) -> LiveClient {
            LiveClient::new(id)
        }

        pub fn dead_factory(id: String) -> DeadClient {
            DeadClient { id }
        }
    }

    pub use inner::{dead_factory, DeadClient, live_factory, LiveClient};
}

pub mod facade {
    pub use crate::protocol::{dead_factory, DeadClient, live_factory, LiveClient};
}

#[opensourced]
pub fn selected(id: String) -> facade::LiveClient {
    facade::live_factory(id)
}
"#,
    );
}

fn write_uniffi_struct_variant_enum_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "uniffi_struct_enum_rule",
        r#"use opensourced::opensourced;

#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct SessionId {
    pub value: String,
}

#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct StartConfig {
    pub id: SessionId,
}

#[cfg_attr(feature = "ffi", derive(uniffi::Error))]
pub enum ClientError {
    Offline,
}

#[cfg_attr(feature = "ffi", derive(uniffi::Enum))]
pub enum ClientEvent {
    Started { config: StartConfig },
    Failed { error: ClientError },
}

pub enum DeadEvent {
    Dead { payload: DeadPayload },
}

pub struct DeadPayload {
    pub value: String,
}

#[opensourced]
pub fn selected(config: StartConfig) -> ClientEvent {
    ClientEvent::Started { config }
}
"#,
    );
}

fn write_serde_response_envelope_rule_fixture(root: &Path) {
    write_workspace_with_dependencies(
        root,
        "serde_response_envelope_rule",
        r#"serde = { version = "1", features = ["derive"] }
serde_json = "1"
"#,
        r#"use opensourced::opensourced;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum Envelope {
    Event(ResponseEvent),
    Error(ErrorPayload),
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum ResponseEvent {
    Token { token: TokenPayload },
    Done,
}

#[derive(Serialize, Deserialize)]
pub struct TokenPayload {
    pub text: String,
}

#[derive(Serialize, Deserialize)]
pub struct ErrorPayload {
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub enum DeadEnvelope {
    Dead(DeadPayload),
}

#[derive(Serialize, Deserialize)]
pub struct DeadPayload {
    pub value: String,
}

#[opensourced]
pub fn selected(input: &str) -> serde_json::Result<Envelope> {
    serde_json::from_str::<Envelope>(input)
}
"#,
    );
}

fn write_serde_multi_helper_rule_fixture(root: &Path) {
    write_workspace_with_dependencies(
        root,
        "serde_multi_helper_rule",
        r#"serde = { version = "1", features = ["derive"] }
serde_json = "1"
"#,
        r#"use opensourced::opensourced;
use serde::Deserialize;

#[derive(Deserialize)]
struct Wire {
    #[serde(default, deserialize_with = "helpers::de_count")]
    count: usize,
    #[serde(default, deserialize_with = "helpers::de_tags")]
    tags: Vec<String>,
}

#[derive(Deserialize)]
struct DeadWire {
    value: String,
}

mod helpers {
    use serde::{Deserialize, Deserializer};

    pub fn de_count<'de, D>(deserializer: D) -> Result<usize, D::Error>
    where
        D: Deserializer<'de>,
    {
        usize::deserialize(deserializer)
    }

    pub fn de_tags<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Vec::<String>::deserialize(deserializer)
    }

    pub fn dead_helper() -> usize {
        99
    }
}

#[opensourced]
pub fn selected(input: &str) -> usize {
    serde_json::from_str::<Wire>(input)
        .ok()
        .map(|wire| wire.count + wire.tags.len())
        .unwrap_or_default()
}
"#,
    );
}

fn write_generic_try_into_bridge_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "generic_try_into_bridge_rule",
        r#"use opensourced::opensourced;
use std::convert::{TryFrom, TryInto};

pub enum WireError {
    Bad,
}

pub enum ApiError {
    Wire(WireError),
}

impl From<WireError> for ApiError {
    fn from(error: WireError) -> Self {
        Self::Wire(error)
    }
}

pub struct WireRequest {
    pub value: String,
}

pub struct InternalRequest {
    value: String,
}

pub struct PublicRequest {
    pub value: String,
}

pub struct DeadWireRequest {
    pub value: String,
}

pub struct DeadInternalRequest {
    value: String,
}

impl TryFrom<WireRequest> for InternalRequest {
    type Error = WireError;

    fn try_from(value: WireRequest) -> Result<Self, Self::Error> {
        if value.value.is_empty() {
            Err(WireError::Bad)
        } else {
            Ok(Self { value: value.value })
        }
    }
}

impl TryFrom<DeadWireRequest> for DeadInternalRequest {
    type Error = WireError;

    fn try_from(value: DeadWireRequest) -> Result<Self, Self::Error> {
        Ok(Self { value: value.value })
    }
}

impl From<InternalRequest> for PublicRequest {
    fn from(value: InternalRequest) -> Self {
        Self { value: value.value }
    }
}

fn bridge<T>(input: T) -> Result<PublicRequest, ApiError>
where
    T: TryInto<InternalRequest, Error = WireError>,
{
    let internal = input.try_into()?;
    Ok(internal.into())
}

#[opensourced]
pub fn selected(input: WireRequest) -> Result<PublicRequest, ApiError> {
    bridge(input)
}
"#,
    );
}

fn write_error_wire_fallback_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "error_wire_fallback_rule",
        r#"use opensourced::opensourced;

pub enum WireError {
    Timeout,
    Unknown { code: String, message: String },
}

pub enum ApiError {
    Wire(WireError),
}

impl From<WireError> for ApiError {
    fn from(error: WireError) -> Self {
        Self::Wire(error)
    }
}

pub enum DeadError {
    Dead,
}

fn parse_wire(value: &str) -> Result<String, WireError> {
    match value {
        "timeout" => Err(WireError::Timeout),
        "" => Err(WireError::Unknown {
            code: "empty".to_string(),
            message: "empty input".to_string(),
        }),
        other => Ok(other.to_string()),
    }
}

fn dead_parse() -> Result<String, DeadError> {
    Err(DeadError::Dead)
}

#[opensourced]
pub fn selected(value: &str) -> Result<String, ApiError> {
    parse_wire(value).map_err(ApiError::from)
}
"#,
    );
}

fn write_concat_include_array_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "concat_include_array_rule",
        r#"use opensourced::opensourced;

const FRAMES: &[&str] = &[
    include_str!(concat!("frames/", "intro.txt")),
    include_str!(concat!("frames/", "outro.txt")),
];

const DEAD_FRAMES: &[&str] = &[include_str!(concat!("frames/", "dead.txt"))];

#[opensourced]
pub fn selected() -> usize {
    FRAMES.iter().map(|frame| frame.len()).sum()
}

pub fn dead_selected() -> usize {
    DEAD_FRAMES.len()
}
"#,
    );
    write(
        root.join("concat_include_array_rule/src/frames/intro.txt"),
        "intro",
    );
    write(
        root.join("concat_include_array_rule/src/frames/outro.txt"),
        "outro",
    );
    write(
        root.join("concat_include_array_rule/src/frames/dead.txt"),
        "dead",
    );
}

fn write_once_lock_include_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "once_lock_include_rule",
        r#"use opensourced::opensourced;
use std::sync::OnceLock;

pub struct Schema {
    pub raw: &'static str,
}

static SCHEMA: OnceLock<Schema> = OnceLock::new();

fn live_schema() -> &'static Schema {
    SCHEMA.get_or_init(|| Schema {
        raw: include_str!("schema/live.json"),
    })
}

fn dead_schema() -> Schema {
    Schema {
        raw: include_str!("schema/dead.json"),
    }
}

#[opensourced]
pub fn selected() -> &'static str {
    live_schema().raw
}
"#,
    );
    write(
        root.join("once_lock_include_rule/src/schema/live.json"),
        "{}",
    );
    write(
        root.join("once_lock_include_rule/src/schema/dead.json"),
        "{}",
    );
}

fn write_method_facade_dyn_boundary_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "method_facade_dyn_boundary_rule",
        r#"use opensourced::opensourced;

pub trait Boundary {
    fn score(&self) -> u32;
}

pub trait DeadBoundary {
    fn score(&self) -> u32;
}

pub mod facade {
    pub use crate::{Boundary, DeadBoundary};
}

#[opensourced]
pub fn selected(handler: &dyn Boundary) -> u32 {
    handler.score()
}
"#,
    );
}

fn write_local_facade_glob_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "local_facade_glob_rule",
        r#"use opensourced::opensourced;

mod api {
    pub struct LiveDto {
        pub value: u32,
    }

    pub struct DeadDto {
        pub value: u32,
    }

    pub fn selected_value() -> u32 {
        helper()
    }

    fn helper() -> u32 {
        7
    }

    pub fn dead_value() -> u32 {
        99
    }
}

pub use api::*;

#[opensourced]
pub fn selected() -> u32 {
    selected_value()
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

pub fn helper_from_generated_expr() -> u32 {
    41
}

pub fn dead_api() -> u32 {
    include!("dead_generated.rs")
}

pub fn dead_helper() -> u32 {
    99
}
"#,
    );
    write(
        root.join("source_include_rule/src/generated_expr.rs"),
        "helper_from_generated_expr()",
    );
    write(root.join("source_include_rule/src/dead_generated.rs"), "99");
}

fn write_inline_source_include_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "inline_source_include_rule",
        r#"use opensourced::opensourced;

pub mod generated {
    include!("generated_value.rs");
}

pub mod dead_generated {
    include!("dead_generated.rs");
}

#[opensourced]
pub fn selected() -> u32 {
    generated::VALUE
}
"#,
    );
    write(
        root.join("inline_source_include_rule/src/generated_value.rs"),
        "pub const VALUE: u32 = 41;\n",
    );
    write(
        root.join("inline_source_include_rule/src/dead_generated.rs"),
        "pub const VALUE: u32 = 99;\n",
    );
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

fn write_qualified_json_macro_rule_fixture(root: &Path) {
    write_workspace_with_dependencies(
        root,
        "qualified_json_macro_rule",
        r#"serde_json = "1"
"#,
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected() -> serde_json::Value {
    serde_json::json!({
        "kind": "live",
        "value": 7,
    })
}

pub fn dead_payload() -> serde_json::Value {
    use serde_json::json;
    json!({
        "kind": "dead",
    })
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

fn write_zero_arg_macro_generated_item_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "zero_arg_macro_rule",
        r#"
use opensourced::opensourced;

macro_rules! setup_runtime {
    () => {
        pub fn generated_value() -> u32 {
            helper_value()
        }
    };
}

macro_rules! setup_dead {
    () => {
        pub fn dead_generated() -> u32 {
            0
        }
    };
}

setup_runtime!();
setup_dead!();

fn helper_value() -> u32 {
    7
}

#[opensourced]
pub fn selected() -> u32 {
    generated_value()
}
"#,
    );
}

fn write_external_trait_import_rule_fixture(root: &Path, support: &Path) {
    write_workspace_with_dependencies(
        root,
        "external_trait_import_rule",
        &format!(r#"support-rng = {{ path = "{}" }}"#, manifest_path(support)),
        r#"
use opensourced::opensourced;
use support_rng::{Fixed, Rng};

#[opensourced]
pub fn selected(mut rng: Fixed) -> u32 {
    rng.gen()
}
"#,
    );
    write(
        support.join("Cargo.toml"),
        r#"[package]
name = "support-rng"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        support.join("src/lib.rs"),
        r#"
pub trait Rng {
    fn gen(&mut self) -> u32;
}

pub trait DeadRng {
    fn dead(&mut self) -> u32;
}

pub struct Fixed;

impl Rng for Fixed {
    fn gen(&mut self) -> u32 {
        7
    }
}
"#,
    );
}

fn write_derive_array_field_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "derive_array_rule",
        r#"
use opensourced::opensourced;

#[derive(Clone)]
pub struct Api {
    pub items: [Inner; 1],
}

pub struct Inner(u32);

impl Clone for Inner {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

pub struct DeadInner(u32);

impl Clone for DeadInner {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

#[opensourced]
pub fn selected(api: Api) -> Api {
    api.clone()
}
"#,
    );
}

fn write_associated_type_equality_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "associated_equality_rule",
        r#"
use opensourced::opensourced;

pub trait Source {
    type Item;

    fn item(&self) -> Self::Item;
}

pub struct Payload;

impl Payload {
    pub fn value(&self) -> u32 {
        1
    }
}

pub struct Other;

impl Other {
    pub fn value(&self) -> u32 {
        2
    }
}

#[opensourced]
pub fn selected<S: Source<Item = Payload>>(source: S) -> u32 {
    source.item().value()
}
"#,
    );
}

fn write_tuple_destructure_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "tuple_destructure_rule",
        r#"
use opensourced::opensourced;

pub struct Payload;

impl Payload {
    pub fn score(&self) -> u32 {
        1
    }
}

pub struct Other;

impl Other {
    pub fn score(&self) -> u32 {
        2
    }
}

fn pair() -> (Payload, u32) {
    (Payload, 0)
}

#[opensourced]
pub fn selected() -> u32 {
    let (payload, _) = pair();
    payload.score()
}
"#,
    );
}

fn write_struct_destructure_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "struct_destructure_rule",
        r#"
use opensourced::opensourced;

pub struct Payload;

impl Payload {
    pub fn score(&self) -> u32 {
        1
    }
}

pub struct Other;

impl Other {
    pub fn score(&self) -> u32 {
        2
    }
}

struct Bundle {
    payload: Payload,
    count: u32,
}

fn bundle() -> Bundle {
    Bundle {
        payload: Payload,
        count: 1,
    }
}

#[opensourced]
pub fn selected() -> u32 {
    let Bundle { payload, .. } = bundle();
    payload.score()
}
"#,
    );
}

fn write_param_destructure_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "param_destructure_rule",
        r#"
use opensourced::opensourced;

pub struct Payload;

impl Payload {
    pub fn live(&self) -> u32 {
        1
    }
}

pub struct Other;

impl Other {
    pub fn live(&self) -> u32 {
        2
    }
}

#[opensourced]
pub fn selected((payload, _): (Payload, u32)) -> u32 {
    payload.live()
}
"#,
    );
}

fn write_for_loop_item_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "for_loop_item_rule",
        r#"
use opensourced::opensourced;

pub struct Payload;

impl Payload {
    pub fn live(&self) -> u32 {
        1
    }
}

pub struct Other;

impl Other {
    pub fn live(&self) -> u32 {
        2
    }
}

fn items() -> Vec<Payload> {
    vec![Payload]
}

#[opensourced]
pub fn selected() -> u32 {
    let mut sum = 0;
    for item in items() {
        sum += item.live();
    }
    sum
}
"#,
    );
}

fn write_free_function_closure_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "free_closure_rule",
        r#"
use opensourced::opensourced;

pub struct Payload;

impl Payload {
    pub fn live(&self) -> u32 {
        1
    }
}

pub struct Other;

impl Other {
    pub fn live(&self) -> u32 {
        2
    }
}

fn with_payload(f: impl FnOnce(Payload) -> u32) -> u32 {
    f(Payload)
}

#[opensourced]
pub fn selected() -> u32 {
    with_payload(|payload| payload.live())
}
"#,
    );
}

fn write_struct_match_pattern_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "struct_match_rule",
        r#"
use opensourced::opensourced;

pub struct Payload;

impl Payload {
    pub fn live(&self) -> u32 {
        1
    }
}

pub struct Other;

impl Other {
    pub fn live(&self) -> u32 {
        2
    }
}

pub struct Envelope {
    payload: Payload,
}

fn envelope() -> Envelope {
    Envelope { payload: Payload }
}

#[opensourced]
pub fn selected() -> u32 {
    match envelope() {
        Envelope { payload } => payload.live(),
    }
}
"#,
    );
}

fn write_option_map_payload_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "option_map_rule",
        r#"
use opensourced::opensourced;

pub struct Payload;

impl Payload {
    pub fn live(&self) -> u32 {
        1
    }
}

pub struct Other;

impl Other {
    pub fn live(&self) -> u32 {
        2
    }
}

fn maybe_payload() -> Option<Payload> {
    Some(Payload)
}

#[opensourced]
pub fn selected() -> Option<u32> {
    maybe_payload().map(|payload| payload.live())
}
"#,
    );
}

fn write_result_map_err_payload_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "result_map_err_rule",
        r#"
use opensourced::opensourced;

pub struct Payload;

impl Payload {
    pub fn live(&self) -> u32 {
        1
    }
}

pub struct Other;

impl Other {
    pub fn live(&self) -> u32 {
        2
    }
}

fn maybe_payload() -> Result<u32, Payload> {
    Err(Payload)
}

#[opensourced]
pub fn selected() -> Result<u32, u32> {
    maybe_payload().map_err(|payload| payload.live())
}
"#,
    );
}

fn write_local_result_alias_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "local_result_alias_rule",
        r#"
use opensourced::opensourced;

#[derive(Debug)]
pub struct LocalError;

type Result<T> = std::result::Result<T, LocalError>;
type DeadResult<T> = std::result::Result<T, String>;

fn value() -> u32 {
    7
}

#[opensourced]
pub fn selected() -> Result<u32> {
    Ok(value())
}
"#,
    );
}

fn write_local_map_method_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "local_map_rule",
        r#"
use opensourced::opensourced;

pub struct Payload;

impl Payload {
    pub fn live(&self) -> u32 {
        1
    }
}

pub struct Other;

impl Other {
    pub fn live(&self) -> u32 {
        2
    }
}

pub struct Pipe<T>(T);

impl<T> Pipe<T> {
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> U {
        f(self.0)
    }
}

fn pipe() -> Pipe<Payload> {
    Pipe(Payload)
}

#[opensourced]
pub fn selected() -> u32 {
    pipe().map(|payload| payload.live())
}
"#,
    );
}

fn write_is_some_and_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "is_some_and_rule",
        r#"
use opensourced::opensourced;

pub struct Payload;

impl Payload {
    pub fn live(&self) -> bool {
        true
    }
}

pub struct Other;

impl Other {
    pub fn live(&self) -> bool {
        false
    }
}

#[opensourced]
pub fn selected() -> bool {
    let maybe: Option<Payload> = Some(Payload);
    maybe.is_some_and(|payload| payload.live())
}
"#,
    );
}

fn write_field_option_arc_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "field_option_arc_rule",
        r#"
use opensourced::opensourced;
use std::sync::Arc;

pub struct Client;

impl Client {
    pub fn is_connected(&self) -> bool {
        true
    }

    pub fn dead(&self) -> bool {
        false
    }
}

pub struct UnusedClient;

impl UnusedClient {
    pub fn is_connected(&self) -> bool {
        false
    }
}

pub struct Holder {
    client: Option<Arc<Client>>,
    unused: Option<Arc<UnusedClient>>,
}

impl Holder {
    #[opensourced]
    pub fn has_client(&self) -> bool {
        self.client.as_ref().is_some_and(|client| client.is_connected())
    }
}
"#,
    );
}

fn write_trait_impl_surface_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "trait_impl_surface_rule",
        r#"
use opensourced::opensourced;

pub trait Marker {
    type Item;

    fn marker(&self) -> Self::Item;

    fn dead(&self) -> Dead {
        Dead
    }
}

#[opensourced]
pub struct Api;

pub struct Payload;

pub struct Dead;

impl Marker for Api {
    type Item = Payload;

    fn marker(&self) -> Self::Item {
        Payload
    }

    fn dead(&self) -> Dead {
        Dead
    }
}
"#,
    );
}

fn write_trait_peer_receiver_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "trait_peer_receiver_rule",
        r#"use opensourced::opensourced;

pub trait Work {
    fn work(&self) -> u32;
}

pub struct Live;

pub struct Decoy;

pub struct DeadWork;

impl Work for Live {
    fn work(&self) -> u32 {
        live_helper()
    }
}

impl Work for Decoy {
    fn work(&self) -> u32 {
        dead_helper().0
    }
}

pub fn live_helper() -> u32 {
    7
}

pub fn dead_helper() -> DeadWork {
    DeadWork
}

#[opensourced]
pub fn selected(live: &Live) -> u32 {
    live.work()
}
"#,
    );
}

fn write_serde_flatten_rule_fixture(root: &Path) {
    write_workspace_with_dependencies(
        root,
        "serde_flatten_rule",
        r#"serde = { version = "1", features = ["derive"] }
serde_json = "1"
"#,
        r#"
use std::collections::BTreeMap;

use opensourced::opensourced;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Wire {
    pub id: String,
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
    dead: Option<String>,
}

#[opensourced]
pub fn selected(wire: Wire) -> String {
    wire.id
}
"#,
    );
}

fn write_serde_flatten_nested_rule_fixture(root: &Path) {
    write_workspace_with_dependencies(
        root,
        "serde_flatten_nested_rule",
        r#"serde = { version = "1", features = ["derive"] }
"#,
        r#"
use opensourced::opensourced;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Wire {
    pub id: String,
    #[serde(flatten)]
    nested: Nested,
}

#[derive(Deserialize)]
pub struct Nested {
    pub session_id: String,
}

#[derive(Deserialize)]
pub struct DeadNested {
    pub dead: String,
}

#[opensourced]
pub fn selected(wire: Wire) -> String {
    wire.id
}
"#,
    );
}

fn write_serde_deserialize_with_rule_fixture(root: &Path) {
    write_workspace_with_dependencies(
        root,
        "serde_deserialize_with_rule",
        r#"serde = { version = "1", features = ["derive"] }
serde_json = "1"
"#,
        r#"
use opensourced::opensourced;
use serde::{Deserialize, Deserializer};

#[derive(Deserialize)]
struct RawDto {
    #[serde(default, deserialize_with = "parse_opt")]
    value: Option<String>,
}

#[derive(Deserialize)]
struct DeadWire {
    value: String,
}

fn parse_opt<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer)
}

#[opensourced]
pub fn selected(input: &str) -> usize {
    serde_json::from_str::<RawDto>(input)
        .ok()
        .and_then(|raw| raw.value)
        .unwrap_or_default()
        .len()
}
"#,
    );
}

fn write_serde_alias_wire_rule_fixture(root: &Path) {
    write_workspace_with_dependencies(
        root,
        "serde_alias_wire_rule",
        r#"serde = { version = "1", features = ["derive"] }
serde_json = "1"
"#,
        r#"
use opensourced::opensourced;
use serde::Deserialize;

#[derive(Deserialize)]
struct PairPayload {
    id: String,
    #[serde(default, alias = "hostname", alias = "display_name")]
    host_name: Option<String>,
}

#[derive(Deserialize)]
struct DeadPayload {
    dead: String,
}

#[opensourced]
pub fn selected(input: &str) -> Option<String> {
    serde_json::from_str::<PairPayload>(input)
        .ok()
        .and_then(|payload| payload.host_name)
}
"#,
    );
}

fn write_serde_transparent_rule_fixture(root: &Path) {
    write_workspace_with_dependencies(
        root,
        "serde_transparent_rule",
        r#"serde = { version = "1", features = ["derive"] }
serde_json = "1"
"#,
        r#"
use opensourced::opensourced;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct SessionId(String);

#[derive(Serialize, Deserialize)]
pub struct Request {
    pub id: SessionId,
}

#[derive(Serialize, Deserialize)]
pub struct DeadSessionId(String);

#[opensourced]
pub fn selected(input: &str) -> String {
    serde_json::from_str::<Request>(input)
        .ok()
        .map(|request| request.id.0)
        .unwrap_or_default()
}
"#,
    );
}

fn write_serde_skip_serializing_rule_fixture(root: &Path) {
    write_workspace_with_dependencies(
        root,
        "serde_skip_serializing_rule",
        r#"serde = { version = "1", features = ["derive"] }
serde_json = "1"
"#,
        r#"
use opensourced::opensourced;
use serde::Serialize;

#[derive(Serialize)]
pub struct Request {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    target: Option<String>,
    dead: Option<String>,
}

#[opensourced]
pub fn selected(request: &Request) -> String {
    serde_json::to_string(request).unwrap_or_default()
}
"#,
    );
}

fn write_serde_tagged_enum_rule_fixture(root: &Path) {
    write_workspace_with_dependencies(
        root,
        "serde_tagged_enum_rule",
        r#"serde = { version = "1", features = ["derive"] }
serde_json = "1"
"#,
        r#"
use opensourced::opensourced;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Login {
    #[serde(rename = "apiKey", rename_all = "camelCase")]
    ApiKey {
        #[serde(rename = "apiKey")]
        api_key: String,
    },
    Chatgpt,
}

#[derive(Serialize, Deserialize)]
pub enum DeadLogin {
    Dead,
}

#[opensourced]
pub fn selected(input: &str) -> bool {
    serde_json::from_str::<Login>(input).is_ok()
}
"#,
    );
}

fn write_serde_default_enum_rule_fixture(root: &Path) {
    write_workspace_with_dependencies(
        root,
        "serde_default_enum_rule",
        r#"serde = { version = "1", features = ["derive"] }
serde_json = "1"
"#,
        r#"
use opensourced::opensourced;
use serde::Deserialize;

#[derive(Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    #[default]
    #[serde(rename = "auto")]
    Auto,
    Manual,
}

#[derive(Deserialize)]
pub enum DeadMode {
    Dead,
}

#[derive(Deserialize)]
pub struct Config {
    #[serde(default)]
    pub mode: Mode,
}

#[opensourced]
pub fn selected(input: &str) -> bool {
    serde_json::from_str::<Config>(input).is_ok()
}
"#,
    );
}

fn write_serde_untagged_enum_rule_fixture(root: &Path) {
    write_workspace_with_dependencies(
        root,
        "serde_untagged_enum_rule",
        r#"serde = { version = "1", features = ["derive"] }
"#,
        r#"
use opensourced::opensourced;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum PathSegment {
    Index(usize),
    Key(String),
}

#[derive(Serialize, Deserialize)]
pub enum DeadSegment {
    Dead,
}

pub struct Route {
    pub segments: Vec<PathSegment>,
}

#[opensourced]
pub fn selected(route: Route) -> usize {
    route.segments.len()
}
"#,
    );
}

fn write_support_path_bundle_rule_fixture(
    root: &Path,
    helper: &Path,
    leaf: &Path,
    dead_leaf: &Path,
) {
    write(
        root.join("Cargo.toml"),
        r#"[workspace]
members = ["support_path_app"]
resolver = "2"
"#,
    );
    write(
        root.join("support_path_app/Cargo.toml"),
        &format!(
            r#"[package]
name = "support_path_app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
external-helper = {{ path = "{}" }}
"#,
            manifest_path(&repo_root().join("crates/opensourced")),
            manifest_path(helper)
        ),
    );
    write(
        root.join("support_path_app/src/lib.rs"),
        r#"
use opensourced::opensourced;

#[opensourced]
pub fn selected(value: &str) -> String {
    let leaf = external_helper::LeafAlias::new(value);
    format!(
        "{}:{}:{}",
        external_helper::decorate(value),
        external_helper::facade_decorate(value),
        leaf.label()
    )
}
"#,
    );
    write(
        helper.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "external-helper"
version = "0.1.0"
edition = "2021"

[dependencies]
external-leaf = {{ path = "{}" }}
dead-leaf = {{ path = "{}" }}
"#,
            manifest_path(leaf),
            manifest_path(dead_leaf)
        ),
    );
    write(
        helper.join("src/lib.rs"),
        r#"
mod facade;
mod live;
pub use external_leaf::{LeafDead as LeafDeadAlias, LeafLive as LeafAlias};
pub use facade::*;

#[cfg(test)]
mod test_only;

pub fn decorate(value: &str) -> String {
    format!("{}{}", live::decorate(value), external_leaf::suffix())
}

pub fn dead_decorate(value: &str) -> String {
    format!("{value}{}", dead_leaf::suffix())
}
"#,
    );
    write(
        helper.join("src/facade.rs"),
        r#"
mod inner;
pub use inner::*;

pub fn dead_facade(value: &str) -> String {
    format!("{value}{}", dead_leaf::suffix())
}
"#,
    );
    write(
        helper.join("src/facade/inner.rs"),
        r#"
pub fn facade_decorate(value: &str) -> String {
    format!("{value}:facade")
}

pub struct FacadeLive {
    pub value: String,
}

pub struct FacadeDead {
    pub value: String,
}

impl FacadeDead {
    pub fn dead_value(&self) -> &str {
        &self.value
    }
}

pub fn dead_inner(value: &str) -> String {
    format!("{value}{}", dead_leaf::suffix())
}
"#,
    );
    write(
        helper.join("src/live.rs"),
        r#"
const LABEL: &str = include_str!("live.txt");
mod atoms;
use atoms::*;

pub fn decorate(value: &str) -> String {
    format!("{value}:{}:{}", LABEL.trim(), suffix())
}

pub fn dead_child(value: &str) -> String {
    format!("{value}{}", dead_leaf::suffix())
}
"#,
    );
    write(
        helper.join("src/live/atoms.rs"),
        r#"
pub fn suffix() -> &'static str {
    "atoms"
}

pub fn dead_suffix() -> &'static str {
    dead_leaf::suffix()
}
"#,
    );
    write(
        helper.join("src/test_only.rs"),
        r#"
const TEST_LABEL: &str = include_str!("test.txt");

pub fn test_value() -> &'static str {
    TEST_LABEL
}
"#,
    );
    write(
        helper.join("src/orphan.rs"),
        r#"
const ORPHAN_LABEL: &str = include_str!("orphan.txt");

pub fn orphan_value() -> &'static str {
    ORPHAN_LABEL
}
"#,
    );
    write(helper.join("src/live.txt"), "live");
    write(helper.join("src/test.txt"), "test");
    write(helper.join("src/orphan.txt"), "orphan");
    write(helper.join("src/bin/unused.rs"), "fn main() {}\n");
    write(helper.join("examples/unused.rs"), "fn main() {}\n");
    write(helper.join("tests/unused.rs"), "#[test]\nfn unused() {}\n");
    write(helper.join("benches/unused.rs"), "fn main() {}\n");
    write(helper.join("fixtures/dead.txt"), "dead");
    write(
        leaf.join("Cargo.toml"),
        r#"[package]
name = "external-leaf"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        leaf.join("src/lib.rs"),
        r#"
pub fn suffix() -> &'static str {
    ":leaf"
}

pub struct LeafLive {
    label: String,
}

impl LeafLive {
    pub fn new(value: &str) -> Self {
        Self {
            label: value.to_string(),
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }
}

pub struct LeafDead {
    label: String,
}

impl LeafDead {
    pub fn label(&self) -> &str {
        &self.label
    }
}

pub fn dead_suffix() -> &'static str {
    ":dead-leaf"
}
"#,
    );
    write(
        dead_leaf.join("Cargo.toml"),
        r#"[package]
name = "dead-leaf"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        dead_leaf.join("src/lib.rs"),
        r#"
pub fn suffix() -> &'static str {
    ":dead"
}
"#,
    );
}

fn write_support_nonstandard_lib_rule_fixture(root: &Path, helper: &Path) {
    write(
        root.join("Cargo.toml"),
        r#"[workspace]
members = ["support_nonstandard_app"]
resolver = "2"
"#,
    );
    write(
        root.join("support_nonstandard_app/Cargo.toml"),
        &format!(
            r#"[package]
name = "support_nonstandard_app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
external-helper = {{ path = "{}" }}
"#,
            manifest_path(&repo_root().join("crates/opensourced")),
            manifest_path(helper)
        ),
    );
    write(
        root.join("support_nonstandard_app/src/lib.rs"),
        r#"
use opensourced::opensourced;

#[opensourced]
pub fn selected(value: &str) -> String {
    external_helper::decorate(value)
}
"#,
    );
    write(
        helper.join("Cargo.toml"),
        r#"[package]
name = "external-helper"
version = "0.1.0"
edition = "2021"

[lib]
path = "src/dump/lib.rs"
"#,
    );
    write(
        helper.join("src/dump/lib.rs"),
        r#"
mod client;

pub fn decorate(value: &str) -> String {
    client::decorate(value)
}
"#,
    );
    write(
        helper.join("src/dump/client.rs"),
        r#"
pub fn decorate(value: &str) -> String {
    format!("{value}:client")
}
"#,
    );
    write(
        helper.join("src/lib.rs"),
        r#"
pub fn orphan() -> &'static str {
    "orphan"
}
"#,
    );
    write(helper.join("examples/unused.rs"), "fn main() {}\n");
}

fn write_async_actor_loop_rule_fixture(root: &Path) {
    write_workspace_with_dependencies(
        root,
        "async_actor_loop_rule",
        r#"tokio = { version = "1", features = ["rt", "sync"] }
"#,
        r#"use opensourced::opensourced;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

pub struct Payload {
    value: u32,
}

pub struct DeadPayload {
    value: u32,
}

enum Command {
    Live(Payload),
    Dead(DeadPayload),
}

pub struct WorkerHandle {
    tx: mpsc::Sender<Command>,
    task: JoinHandle<()>,
}

impl WorkerHandle {
    pub fn sender(&self) -> mpsc::Sender<Command> {
        self.tx.clone()
    }

    pub fn abort(self) {
        self.task.abort();
    }
}

async fn worker_loop(mut rx: mpsc::Receiver<Command>) {
    while let Some(command) = rx.recv().await {
        if let Command::Live(payload) = command {
            record(payload).await;
        }
    }
}

async fn record(payload: Payload) {
    let _ = payload.value;
}

async fn dead_record(payload: DeadPayload) {
    let _ = payload.value;
}

#[opensourced]
pub fn selected() -> WorkerHandle {
    let (tx, rx) = mpsc::channel(4);
    let task = tokio::spawn(worker_loop(rx));
    WorkerHandle { tx, task }
}
"#,
    );
}

fn write_async_select_reconnect_rule_fixture(root: &Path) {
    write_workspace_with_dependencies(
        root,
        "async_select_reconnect_rule",
        r#"tokio = { version = "1", features = ["macros", "rt", "sync", "time"] }
"#,
        r#"use opensourced::opensourced;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc, watch};
use tokio::task::JoinHandle;

#[derive(Clone)]
pub enum Event {
    Connected,
}

pub enum DeadEvent {
    Dead,
}

pub struct Reconnector {
    invalidation_tx: mpsc::UnboundedSender<()>,
    task: JoinHandle<()>,
}

impl Reconnector {
    pub fn invalidate(&self) {
        let _ = self.invalidation_tx.send(());
    }

    pub fn shutdown(self) {
        self.task.abort();
    }
}

async fn reconnect_loop(
    mut invalidate_rx: mpsc::UnboundedReceiver<()>,
    mut connected_rx: watch::Receiver<bool>,
    events: broadcast::Sender<Event>,
) {
    loop {
        tokio::select! {
            signal = invalidate_rx.recv() => {
                if signal.is_none() {
                    return;
                }
                let _ = events.send(Event::Connected);
                break;
            }
            changed = connected_rx.changed() => {
                if changed.is_err() {
                    return;
                }
                if *connected_rx.borrow() {
                    let _ = events.send(Event::Connected);
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(1)) => {
                break;
            }
        }
    }
}

fn dead_event() -> DeadEvent {
    DeadEvent::Dead
}

#[opensourced]
pub fn selected() -> Reconnector {
    let (invalidation_tx, invalidation_rx) = mpsc::unbounded_channel();
    let (connected_tx, connected_rx) = watch::channel(false);
    let (events, _) = broadcast::channel(4);
    drop(connected_tx);
    let task = tokio::spawn(reconnect_loop(invalidation_rx, connected_rx, events));
    Reconnector {
        invalidation_tx,
        task,
    }
}
"#,
    );
}

fn write_ffi_unsafe_no_mangle_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "ffi_unsafe_no_mangle_rule",
        r#"use opensourced::opensourced;
use std::os::raw::c_char;

fn live_offset() -> usize {
    1
}

fn dead_offset() -> usize {
    99
}

#[opensourced]
#[unsafe(no_mangle)]
pub extern "C" fn selected_bridge(ptr: *const c_char, len: usize) -> usize {
    if ptr.is_null() {
        0
    } else {
        len + live_offset()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn dead_bridge(_ptr: *const c_char) -> usize {
    dead_offset()
}
"#,
    );
}

fn write_jni_extern_system_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "jni_extern_system_rule",
        r#"use opensourced::opensourced;

mod jni {
    pub struct JNIEnv;

    impl JNIEnv {
        pub fn get_string(&mut self, _value: &objects::JString) -> Result<String, ()> {
            Ok("live".to_string())
        }
    }

    pub mod objects {
        pub struct JClass;
        pub struct JString;
    }

    pub mod sys {
        #[allow(non_camel_case_types)]
        pub type jint = i32;
    }
}

use jni::objects::{JClass, JString};
use jni::sys::jint;
use jni::JNIEnv;

fn live_probe(value: &str) -> jint {
    value.len() as jint
}

fn dead_probe() -> jint {
    -99
}

#[opensourced]
#[unsafe(no_mangle)]
pub extern "system" fn Java_demo_Native_live(
    mut env: JNIEnv,
    _class: JClass,
    value: JString,
) -> jint {
    match env.get_string(&value) {
        Ok(value) => live_probe(&value),
        Err(_) => -1,
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_demo_Native_dead(_env: JNIEnv, _class: JClass) -> jint {
    dead_probe()
}
"#,
    );
}

fn write_clap_command_contract_rule_fixture(root: &Path) {
    write_workspace_with_dependencies(
        root,
        "clap_command_rule",
        r#"clap = { version = "4", features = ["derive"] }
"#,
        r#"use clap::{Args, Parser, Subcommand};
use opensourced::opensourced;

#[derive(Parser)]
#[command(name = "debug-tool", about = "Debug entrypoint")]
pub struct Cli {
    #[command(subcommand)]
    command: Command,

    #[arg(long, default_value = "json")]
    format: String,
}

#[derive(Subcommand)]
enum Command {
    Live(LiveArgs),
    Serve {
        #[arg(long, value_delimiter = ',', default_value = "all")]
        methods: Vec<String>,
    },
}

#[derive(Args)]
pub struct LiveArgs {
    #[arg(long, conflicts_with = "raw")]
    pretty: bool,

    #[arg(long)]
    raw: bool,
}

pub struct DeadCliHelper;

impl LiveArgs {
    fn mode(&self) -> &'static str {
        if self.raw {
            "raw"
        } else if self.pretty {
            "pretty"
        } else {
            "compact"
        }
    }
}

#[opensourced]
pub fn selected<I, T>(iter: I) -> Result<String, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let cli = Cli::try_parse_from(iter)?;
    let command = match cli.command {
        Command::Live(args) => args.mode().to_string(),
        Command::Serve { methods } => methods.join(","),
    };
    Ok(format!("{}:{command}", cli.format))
}
"#,
    );
}

fn write_thiserror_contract_rule_fixture(root: &Path) {
    write_workspace_with_dependencies(
        root,
        "thiserror_contract_rule",
        r#"thiserror = "2"
"#,
        r#"use opensourced::opensourced;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BridgeError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("protocol {code}: {message}")]
    Protocol { code: String, message: String },
}

#[derive(Debug, Error)]
pub enum DeadBridgeError {
    #[error("dead")]
    Dead,
}

fn live_message() -> String {
    "bad frame".to_string()
}

fn dead_message() -> String {
    "dead".to_string()
}

#[opensourced]
pub fn selected(value: &str) -> Result<(), BridgeError> {
    if value.is_empty() {
        return Err(BridgeError::Protocol {
            code: "empty".to_string(),
            message: live_message(),
        });
    }
    Err(std::io::Error::from(std::io::ErrorKind::Other).into())
}
"#,
    );
}

fn write_async_io_poll_rule_fixture(root: &Path) {
    write_workspace_with_dependencies(
        root,
        "async_io_poll_rule",
        r#"tokio = { version = "1", features = ["io-util"] }
"#,
        r#"use opensourced::opensourced;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

pub struct BridgeStream {
    read: Vec<u8>,
    written: usize,
}

impl BridgeStream {
    fn new() -> Self {
        Self {
            read: vec![1, 2, 3],
            written: 0,
        }
    }
}

impl AsyncRead for BridgeStream {
    fn poll_read(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let this = self.get_mut();
        buf.put_slice(&this.read);
        this.read.clear();
        Poll::Ready(Ok(()))
    }
}

impl AsyncWrite for BridgeStream {
    fn poll_write(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        let this = self.get_mut();
        this.written += buf.len();
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

pub struct DeadStream;

fn generic_stream<S>(stream: S) -> S
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    stream
}

#[opensourced]
pub fn selected() -> BridgeStream {
    generic_stream(BridgeStream::new())
}
"#,
    );
}

fn write_runtime_singleton_registry_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "runtime_singleton_registry_rule",
        r#"use opensourced::opensourced;
use std::sync::{Arc, Mutex, OnceLock};

pub struct Runtime {
    stack: usize,
}

pub struct RuntimeBuilder {
    stack: usize,
}

impl RuntimeBuilder {
    fn new() -> Self {
        Self { stack: 0 }
    }

    fn thread_stack_size(mut self, stack: usize) -> Self {
        self.stack = stack;
        self
    }

    fn build(self) -> Runtime {
        Runtime { stack: self.stack }
    }
}

static RUNTIME: OnceLock<Arc<Runtime>> = OnceLock::new();
static REGISTRY: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
static DEAD_RUNTIME: OnceLock<Arc<Runtime>> = OnceLock::new();

fn stack_size() -> usize {
    1024
}

fn dead_stack_size() -> usize {
    1
}

fn shared_runtime() -> Arc<Runtime> {
    Arc::clone(RUNTIME.get_or_init(|| {
        Arc::new(
            RuntimeBuilder::new()
                .thread_stack_size(stack_size())
                .build(),
        )
    }))
}

fn registry() -> &'static Mutex<Vec<String>> {
    REGISTRY.get_or_init(|| Mutex::new(Vec::new()))
}

fn remember(name: String) -> usize {
    let mut guard = registry().lock().expect("registry lock");
    guard.push(name);
    guard.len()
}

#[opensourced]
pub fn selected(name: String) -> usize {
    remember(name) + shared_runtime().stack
}
"#,
    );
}

fn write_platform_asset_extern_rule_fixture(root: &Path) {
    write_workspace(
        root,
        "platform_asset_extern_rule",
        r#"use opensourced::opensourced;

fn live_common() -> usize {
    1
}

#[cfg(any(target_os = "ios", target_os = "android"))]
mod platform {
    static CACERT_PEM: &[u8] = include_bytes!("cacert.pem");

    #[cfg(target_os = "ios")]
    unsafe extern "C" {
        fn ios_tls_probe() -> i32;
    }

    #[cfg(target_os = "ios")]
    fn platform_probe() -> usize {
        unsafe { ios_tls_probe() as usize }
    }

    #[cfg(target_os = "android")]
    fn platform_probe() -> usize {
        7
    }

    pub fn init_tls_roots() -> usize {
        CACERT_PEM.len() + platform_probe()
    }

    pub fn dead_platform_probe() -> usize {
        include_bytes!("dead.pem").len()
    }
}

#[opensourced]
pub fn selected() -> usize {
    let total = live_common();
    #[cfg(any(target_os = "ios", target_os = "android"))]
    {
        return total + platform::init_tls_roots();
    }
    total
}
"#,
    );
    write(
        root.join("platform_asset_extern_rule/src/cacert.pem"),
        "cert",
    );
    write(root.join("platform_asset_extern_rule/src/dead.pem"), "dead");
}

fn write_serde_custom_numeric_rule_fixture(root: &Path) {
    write_workspace_with_dependencies(
        root,
        "serde_custom_numeric_rule",
        r#"serde = { version = "1", features = ["derive"] }
serde_json = "1"
"#,
        r#"use opensourced::opensourced;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub enum WireEvent {
    Number(#[serde(deserialize_with = "de_f64", serialize_with = "ser_f64")] f64),
    Counter {
        #[serde(default, deserialize_with = "de_opt_u32")]
        count: Option<u32>,
    },
}

fn de_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    value.parse().map_err(serde::de::Error::custom)
}

fn ser_f64<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&value.to_string())
}

fn de_opt_u32<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    value
        .map(|value| value.parse().map_err(serde::de::Error::custom))
        .transpose()
}

fn dead_de_f64<'de, D>(_deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(0.0)
}

#[opensourced]
pub fn selected(input: &str) -> Result<WireEvent, serde_json::Error> {
    serde_json::from_str(input)
}
"#,
    );
}

fn write_workspace(root: &Path, package: &str, lib: &str) {
    write_workspace_with_dependencies(root, package, "", lib);
}

fn write_workspace_with_dependencies(root: &Path, package: &str, dependencies: &str, lib: &str) {
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
{dependencies}
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
