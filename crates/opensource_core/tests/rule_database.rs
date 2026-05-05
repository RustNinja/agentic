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
    assert!(lib.contains("#[derive(Serialize)]"), "{lib}");
    assert!(lib.contains("pub fn live_client"), "{lib}");
    assert!(!lib.contains("dead_client"), "{lib}");
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

    assert_eq!(report.production.status, "hazards_detected");
    let trait_object = report
        .production
        .hazards
        .iter()
        .find(|hazard| hazard.code == "trait_object_surfaces" && hazard.severity == "error")
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

    assert_eq!(report.production.status, "hazards_detected");
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
fn reports_returned_trait_object_surfaces_as_dynamic_hazards() {
    let workspace = temp_path("rule-returned-dyn-workspace");
    let output = temp_path("rule-returned-dyn-output");
    let target_dir = temp_path("rule-returned-dyn-target");
    write_returned_trait_object_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("returned dyn rule should reduce");

    assert_eq!(report.production.status, "hazards_detected");
    assert!(report.production.hazards.iter().any(|hazard| {
        hazard.code == "trait_object_surfaces"
            && hazard
                .details
                .iter()
                .any(|detail| detail.subject.contains("dyn Reader"))
    }));
    let lib = read(output.join("returned_dyn_rule/src/lib.rs"));
    assert!(lib.contains("Box<dyn Reader>"), "{lib}");
    assert!(lib.contains("pub trait Reader"), "{lib}");
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
fn reports_nested_callback_store_trait_objects_as_dynamic_hazards() {
    let workspace = temp_path("rule-nested-callback-store-workspace");
    let output = temp_path("rule-nested-callback-store-output");
    write_nested_callback_store_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("nested callback store rule should reduce");

    assert_eq!(report.production.status, "hazards_detected");
    let trait_object = report
        .production
        .hazards
        .iter()
        .find(|hazard| hazard.code == "trait_object_surfaces" && hazard.severity == "error")
        .expect("nested callback store should be a hard production hazard");
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
fn reports_trait_object_casts_while_retaining_concrete_sources() {
    let workspace = temp_path("rule-trait-object-cast-workspace");
    let output = temp_path("rule-trait-object-cast-output");
    let target_dir = temp_path("rule-trait-object-cast-target");
    write_trait_object_cast_rule_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("trait object cast rule should reduce");

    assert_eq!(report.production.status, "hazards_detected");
    assert!(report.production.hazards.iter().any(|hazard| {
        hazard.code == "trait_object_surfaces"
            && hazard
                .details
                .iter()
                .any(|detail| detail.subject.contains("dyn Send + Sync"))
    }));
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

    assert_eq!(report.production.status, "hazards_detected");
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

fn write_module_scoped_import_liveness_rule_fixture(root: &Path) {
    write_workspace_with_dependencies(
        root,
        "module_import_liveness_rule",
        r#"serde = { version = "1", features = ["derive"] }
"#,
        r#"use opensourced::opensourced;

mod client {
    use serde::Serialize;

    pub fn live_client() -> u32 {
        7
    }

    pub fn dead_client<T: Serialize>(value: &T) -> usize {
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
