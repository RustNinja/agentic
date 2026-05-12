use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use opensource_core::{generate_with_analyzer, AnalyzerMode, GenerateOptions, GenerateReport};

#[test]
#[should_panic(expected = "generated source exposes public reexports")]
fn public_reexport_contract_rejects_prunable_local_targets() {
    let mut rendered = RenderedSymbols::default();
    rendered.module_paths.insert(Vec::new());
    rendered.module_paths.insert(vec!["facade".to_string()]);
    rendered.public_reexports.insert(RenderedPublicReexport {
        package: "root".to_string(),
        module_path: Vec::new(),
        visible: "Dead".to_string(),
        target: vec!["facade".to_string(), "Dead".to_string()],
    });

    assert_public_reexport_contract(
        &rendered,
        &BTreeSet::new(),
        &BTreeSet::new(),
        &BTreeSet::new(),
        &BTreeSet::from(["root::facade::Dead(Struct)".to_string()]),
    );
}

#[test]
#[should_panic(
    expected = "generated source exposes public reexports to unclassified local targets"
)]
fn public_reexport_contract_rejects_unclassified_local_targets() {
    let mut rendered = RenderedSymbols::default();
    rendered.module_paths.insert(Vec::new());
    rendered.module_paths.insert(vec!["facade".to_string()]);
    rendered.public_reexports.insert(RenderedPublicReexport {
        package: "root".to_string(),
        module_path: Vec::new(),
        visible: "Escaped".to_string(),
        target: vec!["facade".to_string(), "Escaped".to_string()],
    });

    assert_public_reexport_contract(
        &rendered,
        &BTreeSet::new(),
        &BTreeSet::new(),
        &BTreeSet::new(),
        &BTreeSet::new(),
    );
}

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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
}

#[test]
fn prunes_public_support_field_name_collisions_to_concrete_struct_use() {
    let fixture = repo_root().join("fixtures/slice_cases/public_field_collision_prune");
    let output = temp_path("slice-case-public-field-collision-prune-output");
    let target_dir = temp_path("slice-case-public-field-collision-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("public_field_collision_prune fixture should slice");

    assert_eq!(
        report.packages,
        ["audit_record", "field_api", "live_record", "root"]
    );
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_summary"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_source = read(output.join("root/src/lib.rs"));
    let field_api = read(output.join("field_api/src/lib.rs"));
    let live_record = read(output.join("live_record/src/lib.rs"));
    let audit_record = read(output.join("audit_record/src/lib.rs"));

    assert!(
        root_source.contains("pub fn selected_summary"),
        "{root_source}"
    );
    assert!(!root_source.contains("dead_summary"), "{root_source}");
    assert!(field_api.contains("pub fn selected"), "{field_api}");
    assert!(field_api.contains("LiveRecord"), "{field_api}");
    assert!(field_api.contains(".value"), "{field_api}");
    assert!(field_api.contains("AuditRecord::label_only"), "{field_api}");
    assert!(!field_api.contains("pub fn dead"), "{field_api}");

    assert!(
        live_record.contains("pub struct LiveRecord"),
        "{live_record}"
    );
    assert!(live_record.contains("pub value: u32"), "{live_record}");
    assert!(!live_record.contains("dead_note"), "{live_record}");
    assert!(!live_record.contains("dead_live"), "{live_record}");

    assert!(
        audit_record.contains("pub struct AuditRecord"),
        "{audit_record}"
    );
    assert!(audit_record.contains("pub fn label_only"), "{audit_record}");
    assert!(
        !audit_record.contains("pub value: u32"),
        "unrelated public support field with same name as a live field must be pruned\n{audit_record}"
    );
    assert!(
        !audit_record.contains("pub label: String"),
        "{audit_record}"
    );
    assert!(!audit_record.contains("dead_value"), "{audit_record}");
    assert!(!audit_record.contains("dead_audit"), "{audit_record}");

    assert_cargo_check(&output, &target_dir, &root_source, &report);
}

#[test]
fn prunes_public_support_field_name_collisions_inside_same_package() {
    let fixture =
        repo_root().join("fixtures/slice_cases/public_field_same_package_collision_prune");
    let output = temp_path("slice-case-public-field-same-package-collision-prune-output");
    let target_dir = temp_path("slice-case-public-field-same-package-collision-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("public_field_same_package_collision_prune fixture should slice");

    assert_eq!(report.packages, ["root", "support_records"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_summary"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_source = read(output.join("root/src/lib.rs"));
    let support = read(output.join("support_records/src/lib.rs"));

    assert!(
        root_source.contains("pub fn selected_summary"),
        "{root_source}"
    );
    assert!(!root_source.contains("dead_summary"), "{root_source}");

    assert!(support.contains("pub struct LiveRecord"), "{support}");
    assert!(support.contains("pub value: u32"), "{support}");
    assert!(!support.contains("dead_note"), "{support}");
    assert!(support.contains("pub struct AuditRecord"), "{support}");
    assert!(
        support.contains("pub struct AuditRecord {\n    pub label: String,\n}"),
        "same-package support record should keep only concrete live label field\n{support}"
    );
    assert!(!support.contains("pub fn dead_value"), "{support}");
    assert!(!support.contains("pub fn dead_summary"), "{support}");

    assert_cargo_check(&output, &target_dir, &root_source, &report);
}

#[test]
fn retains_public_support_fields_used_through_typed_param_patterns() {
    let fixture = repo_root().join("fixtures/slice_cases/public_field_typed_param_pattern_prune");
    let output = temp_path("slice-case-public-field-typed-param-pattern-prune-output");
    let target_dir = temp_path("slice-case-public-field-typed-param-pattern-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("public_field_typed_param_pattern_prune fixture should slice");

    assert_eq!(report.packages, ["root", "support_records"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_summary"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_source = read(output.join("root/src/lib.rs"));
    let support = read(output.join("support_records/src/lib.rs"));

    assert!(
        root_source.contains("pub fn selected_summary"),
        "{root_source}"
    );
    assert!(!root_source.contains("dead_summary"), "{root_source}");

    assert!(support.contains("pub struct LiveRecord"), "{support}");
    assert!(support.contains("pub value: u32"), "{support}");
    assert!(!support.contains("dead_note"), "{support}");
    assert!(support.contains("fn read_value"), "{support}");
    assert!(support.contains("{ value, .. }"), "{support}");
    assert!(
        support.contains("pub struct AuditRecord {\n    pub label: String,\n}"),
        "same-module sibling should keep only its concrete live label field\n{support}"
    );
    assert!(!support.contains("pub fn dead_value"), "{support}");
    assert!(!support.contains("pub fn dead_summary"), "{support}");

    assert_cargo_check(&output, &target_dir, &root_source, &report);
}

#[test]
fn retains_public_support_fields_used_through_typed_params() {
    let fixture = repo_root().join("fixtures/slice_cases/public_field_typed_param_prune");
    let output = temp_path("slice-case-public-field-typed-param-prune-output");
    let target_dir = temp_path("slice-case-public-field-typed-param-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("public_field_typed_param_prune fixture should slice");

    assert_eq!(report.packages, ["audit_record", "field_api", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_score"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_source = read(output.join("root/src/lib.rs"));
    let field_api = read(output.join("field_api/src/lib.rs"));
    let audit_record = read(output.join("audit_record/src/lib.rs"));

    assert!(
        root_source.contains("pub fn selected_score"),
        "{root_source}"
    );
    assert!(!root_source.contains("dead_score"), "{root_source}");

    assert!(field_api.contains("pub fn selected_score"), "{field_api}");
    assert!(field_api.contains("fn read_value"), "{field_api}");
    assert!(field_api.contains("record.value"), "{field_api}");
    assert!(!field_api.contains("dead_note"), "{field_api}");
    assert!(!field_api.contains("pub fn dead_score"), "{field_api}");

    assert!(
        audit_record.contains("pub struct AuditRecord"),
        "{audit_record}"
    );
    assert!(audit_record.contains("pub value: u32"), "{audit_record}");
    assert!(!audit_record.contains("dead_note"), "{audit_record}");
    assert!(
        audit_record.contains("pub fn placeholder"),
        "{audit_record}"
    );
    assert!(!audit_record.contains("dead_record"), "{audit_record}");

    assert_cargo_check(&output, &target_dir, &root_source, &report);
}

#[test]
fn retains_public_support_fields_used_through_map_get() {
    let fixture = repo_root().join("fixtures/slice_cases/public_field_map_get_prune");
    let output = temp_path("slice-case-public-field-map-get-prune-output");
    let target_dir = temp_path("slice-case-public-field-map-get-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("public_field_map_get_prune fixture should slice");

    assert_eq!(report.packages, ["audit_record", "field_api", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_summary"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_source = read(output.join("root/src/lib.rs"));
    let field_api = read(output.join("field_api/src/lib.rs"));
    let audit_record = read(output.join("audit_record/src/lib.rs"));

    assert!(
        root_source.contains("pub fn selected_summary"),
        "{root_source}"
    );
    assert!(!root_source.contains("dead_summary"), "{root_source}");

    assert!(field_api.contains("pub fn selected_summary"), "{field_api}");
    assert!(field_api.contains(".records.get"), "{field_api}");
    assert!(field_api.contains("record.label"), "{field_api}");
    assert!(field_api.contains("record.value"), "{field_api}");
    assert!(!field_api.contains("unused_note"), "{field_api}");
    assert!(!field_api.contains("pub fn dead_summary"), "{field_api}");

    assert!(
        audit_record.contains("pub struct AuditSnapshot"),
        "{audit_record}"
    );
    assert!(audit_record.contains("pub records:"), "{audit_record}");
    assert!(!audit_record.contains("unused_note"), "{audit_record}");
    assert!(
        audit_record.contains("pub struct AuditRecord"),
        "{audit_record}"
    );
    assert!(audit_record.contains("pub value:"), "{audit_record}");
    assert!(audit_record.contains("pub label:"), "{audit_record}");
    assert!(audit_record.contains("Option"), "{audit_record}");
    assert!(!audit_record.contains("dead_note"), "{audit_record}");
    assert!(audit_record.contains("pub fn snapshot"), "{audit_record}");
    assert!(!audit_record.contains("dead_snapshot"), "{audit_record}");

    assert_cargo_check(&output, &target_dir, &root_source, &report);
}

#[test]
fn retains_public_support_fields_used_through_vec_iter_closure() {
    let fixture = repo_root().join("fixtures/slice_cases/public_field_vec_iter_closure_prune");
    let output = temp_path("slice-case-public-field-vec-iter-closure-prune-output");
    let target_dir = temp_path("slice-case-public-field-vec-iter-closure-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("public_field_vec_iter_closure_prune fixture should slice");

    assert_eq!(report.packages, ["audit_record", "field_api", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_summary"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_source = read(output.join("root/src/lib.rs"));
    let field_api = read(output.join("field_api/src/lib.rs"));
    let audit_record = read(output.join("audit_record/src/lib.rs"));

    assert!(
        root_source.contains("pub fn selected_summary"),
        "{root_source}"
    );
    assert!(!root_source.contains("dead_summary"), "{root_source}");

    assert!(field_api.contains("pub fn selected_summary"), "{field_api}");
    assert!(field_api.contains(".records"), "{field_api}");
    assert!(field_api.contains(".iter()"), "{field_api}");
    assert!(field_api.contains("record.label"), "{field_api}");
    assert!(field_api.contains("record.value"), "{field_api}");
    assert!(!field_api.contains("unused_note"), "{field_api}");
    assert!(!field_api.contains("pub fn dead_summary"), "{field_api}");

    assert!(
        audit_record.contains("pub struct AuditSnapshot"),
        "{audit_record}"
    );
    assert!(audit_record.contains("pub records:"), "{audit_record}");
    assert!(!audit_record.contains("unused_note"), "{audit_record}");
    assert!(
        audit_record.contains("pub struct AuditRecord"),
        "{audit_record}"
    );
    assert!(audit_record.contains("pub value:"), "{audit_record}");
    assert!(audit_record.contains("pub label:"), "{audit_record}");
    assert!(!audit_record.contains("dead_note"), "{audit_record}");
    assert!(audit_record.contains("pub fn snapshot"), "{audit_record}");
    assert!(!audit_record.contains("dead_snapshot"), "{audit_record}");

    assert_cargo_check(&output, &target_dir, &root_source, &report);
}

#[test]
fn retains_public_support_fields_used_through_iterator_closure_surfaces() {
    let fixture =
        repo_root().join("fixtures/slice_cases/public_field_iterator_closure_surfaces_prune");
    let output = temp_path("slice-case-public-field-iterator-closure-surfaces-prune-output");
    let target_dir = temp_path("slice-case-public-field-iterator-closure-surfaces-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("public_field_iterator_closure_surfaces_prune fixture should slice");

    assert_eq!(report.packages, ["audit_record", "field_api", "root"]);

    let root_source = read(output.join("root/src/lib.rs"));
    let field_api = read(output.join("field_api/src/lib.rs"));
    let audit_record = read(output.join("audit_record/src/lib.rs"));

    assert!(
        root_source.contains("pub fn selected_summary"),
        "{root_source}"
    );
    assert!(!root_source.contains("dead_summary"), "{root_source}");

    assert!(field_api.contains("pub fn selected_summary"), "{field_api}");
    assert!(field_api.contains("sort_by"), "{field_api}");
    assert!(field_api.contains("left.label"), "{field_api}");
    assert!(field_api.contains("right.label"), "{field_api}");
    assert!(field_api.contains("record.value"), "{field_api}");
    assert!(field_api.contains("record.tags"), "{field_api}");
    assert!(field_api.contains("record.active"), "{field_api}");
    assert!(!field_api.contains("dead_note"), "{field_api}");
    assert!(!field_api.contains("unused_note"), "{field_api}");
    assert!(!field_api.contains("pub fn dead_summary"), "{field_api}");

    assert!(
        audit_record.contains("pub struct AuditSnapshot"),
        "{audit_record}"
    );
    assert!(audit_record.contains("pub records:"), "{audit_record}");
    assert!(!audit_record.contains("unused_note"), "{audit_record}");
    assert!(
        audit_record.contains("pub struct AuditRecord"),
        "{audit_record}"
    );
    assert!(audit_record.contains("pub value:"), "{audit_record}");
    assert!(audit_record.contains("pub label:"), "{audit_record}");
    assert!(audit_record.contains("pub tags:"), "{audit_record}");
    assert!(audit_record.contains("pub active:"), "{audit_record}");
    assert!(!audit_record.contains("dead_note"), "{audit_record}");
    assert!(audit_record.contains("pub fn snapshot"), "{audit_record}");
    assert!(!audit_record.contains("dead_snapshot"), "{audit_record}");

    assert_cargo_check(&output, &target_dir, &root_source, &report);
}

#[test]
fn retains_public_support_fields_used_through_iterator_tuple_shapes() {
    let fixture = repo_root().join("fixtures/slice_cases/public_field_iterator_tuple_shape_prune");
    let output = temp_path("slice-case-public-field-iterator-tuple-shape-prune-output");
    let target_dir = temp_path("slice-case-public-field-iterator-tuple-shape-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("public_field_iterator_tuple_shape_prune fixture should slice");

    assert_eq!(
        report.packages,
        ["field_api", "left_record", "right_record", "root"]
    );

    let root_source = read(output.join("root/src/lib.rs"));
    let field_api = read(output.join("field_api/src/lib.rs"));
    let left_record = read(output.join("left_record/src/lib.rs"));
    let right_record = read(output.join("right_record/src/lib.rs"));

    assert!(
        root_source.contains("pub fn selected_summary"),
        "{root_source}"
    );
    assert!(!root_source.contains("dead_summary"), "{root_source}");

    assert!(field_api.contains("pub fn selected_summary"), "{field_api}");
    assert!(field_api.contains("enumerate()"), "{field_api}");
    assert!(
        field_api.contains("zip(right.entries.iter())"),
        "{field_api}"
    );
    assert!(field_api.contains("record.label"), "{field_api}");
    assert!(field_api.contains("left.value"), "{field_api}");
    assert!(field_api.contains("right.weight"), "{field_api}");
    assert!(field_api.contains("entry.code"), "{field_api}");
    assert!(field_api.contains("record.loop_label"), "{field_api}");
    assert!(field_api.contains("left.loop_value"), "{field_api}");
    assert!(field_api.contains("right.loop_weight"), "{field_api}");
    assert!(field_api.contains("right.loop_code"), "{field_api}");
    assert!(!field_api.contains("dead_note"), "{field_api}");
    assert!(!field_api.contains("unused_note"), "{field_api}");
    assert!(!field_api.contains("pub fn dead_summary"), "{field_api}");

    assert!(
        left_record.contains("pub struct LeftSnapshot"),
        "{left_record}"
    );
    assert!(left_record.contains("pub records:"), "{left_record}");
    assert!(!left_record.contains("unused_note"), "{left_record}");
    assert!(
        left_record.contains("pub struct LeftRecord"),
        "{left_record}"
    );
    assert!(left_record.contains("pub label:"), "{left_record}");
    assert!(left_record.contains("pub value:"), "{left_record}");
    assert!(left_record.contains("pub loop_label:"), "{left_record}");
    assert!(left_record.contains("pub loop_value:"), "{left_record}");
    assert!(!left_record.contains("dead_note"), "{left_record}");
    assert!(left_record.contains("pub fn snapshot"), "{left_record}");
    assert!(!left_record.contains("dead_snapshot"), "{left_record}");

    assert!(
        right_record.contains("pub struct RightSnapshot"),
        "{right_record}"
    );
    assert!(right_record.contains("pub entries:"), "{right_record}");
    assert!(!right_record.contains("unused_note"), "{right_record}");
    assert!(
        right_record.contains("pub struct RightRecord"),
        "{right_record}"
    );
    assert!(right_record.contains("pub code:"), "{right_record}");
    assert!(right_record.contains("pub weight:"), "{right_record}");
    assert!(right_record.contains("pub loop_code:"), "{right_record}");
    assert!(right_record.contains("pub loop_weight:"), "{right_record}");
    assert!(!right_record.contains("dead_note"), "{right_record}");
    assert!(right_record.contains("pub fn snapshot"), "{right_record}");
    assert!(!right_record.contains("dead_snapshot"), "{right_record}");

    assert_cargo_check(&output, &target_dir, &root_source, &report);
}

#[test]
fn retains_public_support_fields_used_through_iterator_transform_shapes() {
    let fixture =
        repo_root().join("fixtures/slice_cases/public_field_iterator_transform_shape_prune");
    let output = temp_path("slice-case-public-field-iterator-transform-shape-prune-output");
    let target_dir = temp_path("slice-case-public-field-iterator-transform-shape-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("public_field_iterator_transform_shape_prune fixture should slice");

    assert!(
        report.production.hazards.iter().all(|hazard| !matches!(
            hazard.code.as_str(),
            "semantic_unresolved_method_calls"
                | "syntactic_method_fallback_cap"
                | "trait_object_surfaces"
        )),
        "project-local iterator methods and returned dyn Iterator<Item = ProjectType> surfaces should be fully proven for this fixture: {:?}",
        report.production.hazards
    );

    assert_eq!(
        report.packages,
        ["field_api", "root", "source_record", "view_record"]
    );

    let root_source = read(output.join("root/src/lib.rs"));
    let field_api = read(output.join("field_api/src/lib.rs"));
    let source_record = read(output.join("source_record/src/lib.rs"));
    let view_record = read(output.join("view_record/src/lib.rs"));

    assert!(
        root_source.contains("pub fn selected_summary"),
        "{root_source}"
    );
    assert!(!root_source.contains("dead_summary"), "{root_source}");

    assert!(field_api.contains("pub fn selected_summary"), "{field_api}");
    assert!(field_api.contains(".map(|record|"), "{field_api}");
    assert!(field_api.contains(".filter_map(|record|"), "{field_api}");
    assert!(field_api.contains(".flat_map(|record|"), "{field_api}");
    assert!(field_api.contains("view.score"), "{field_api}");
    assert!(field_api.contains("view.title"), "{field_api}");
    assert!(field_api.contains("view.loop_title"), "{field_api}");
    assert!(field_api.contains("view.code"), "{field_api}");
    assert!(field_api.contains("view.weight"), "{field_api}");
    assert!(field_api.contains("child.child_weight"), "{field_api}");
    assert!(field_api.contains("child.child_label"), "{field_api}");
    assert!(field_api.contains("local_views"), "{field_api}");
    assert!(field_api.contains("view.local_title"), "{field_api}");
    assert!(field_api.contains("view.local_score"), "{field_api}");
    assert!(field_api.contains("lazy_views"), "{field_api}");
    assert!(field_api.contains("view.lazy_title"), "{field_api}");
    assert!(field_api.contains("view.lazy_score"), "{field_api}");
    assert!(field_api.contains("build_returned_views"), "{field_api}");
    assert!(field_api.contains("returned_views"), "{field_api}");
    assert!(field_api.contains("view.returned_title"), "{field_api}");
    assert!(field_api.contains("view.returned_score"), "{field_api}");
    assert!(field_api.contains("method_views"), "{field_api}");
    assert!(field_api.contains("view.method_title"), "{field_api}");
    assert!(field_api.contains("view.method_score"), "{field_api}");
    assert!(field_api.contains("summarize_param_views"), "{field_api}");
    assert!(field_api.contains("view.param_title"), "{field_api}");
    assert!(field_api.contains("view.param_score"), "{field_api}");
    assert!(field_api.contains("impl_iter_views"), "{field_api}");
    assert!(field_api.contains("view.impl_title"), "{field_api}");
    assert!(field_api.contains("view.impl_score"), "{field_api}");
    assert!(field_api.contains("dyn_iter_views"), "{field_api}");
    assert!(field_api.contains("view.dyn_title"), "{field_api}");
    assert!(field_api.contains("view.dyn_score"), "{field_api}");
    assert!(field_api.contains("view.branch_title"), "{field_api}");
    assert!(field_api.contains("view.branch_score"), "{field_api}");
    assert!(field_api.contains("view.if_title"), "{field_api}");
    assert!(field_api.contains("view.if_score"), "{field_api}");
    assert!(field_api.contains("if_collection_views"), "{field_api}");
    assert!(
        field_api.contains("view.if_collection_title"),
        "{field_api}"
    );
    assert!(
        field_api.contains("view.if_collection_score"),
        "{field_api}"
    );
    assert!(field_api.contains("match_collection_views"), "{field_api}");
    assert!(
        field_api.contains("view.match_collection_title"),
        "{field_api}"
    );
    assert!(
        field_api.contains("view.match_collection_score"),
        "{field_api}"
    );
    assert!(field_api.contains("summarize_alias_views"), "{field_api}");
    assert!(field_api.contains("view.alias_title"), "{field_api}");
    assert!(field_api.contains("view.alias_score"), "{field_api}");
    assert!(field_api.contains("tuple_views"), "{field_api}");
    assert!(field_api.contains("view.tuple_title"), "{field_api}");
    assert!(field_api.contains("view.tuple_score"), "{field_api}");
    assert!(field_api.contains("bag_views"), "{field_api}");
    assert!(field_api.contains("view.bag_title"), "{field_api}");
    assert!(field_api.contains("view.bag_score"), "{field_api}");
    assert!(
        field_api.contains("summarize_generic_alias_views"),
        "{field_api}"
    );
    assert!(field_api.contains("view.generic_title"), "{field_api}");
    assert!(field_api.contains("view.generic_score"), "{field_api}");
    assert!(field_api.contains("newtype_views"), "{field_api}");
    assert!(field_api.contains("view.newtype_title"), "{field_api}");
    assert!(field_api.contains("view.newtype_score"), "{field_api}");
    assert!(field_api.contains("local_alias_views"), "{field_api}");
    assert!(field_api.contains("view.local_alias_title"), "{field_api}");
    assert!(field_api.contains("view.local_alias_score"), "{field_api}");
    assert!(!field_api.contains("dead_view_note"), "{field_api}");
    assert!(!field_api.contains("dead_filter_note"), "{field_api}");
    assert!(!field_api.contains("dead_child_view_note"), "{field_api}");
    assert!(!field_api.contains("dead_local_note"), "{field_api}");
    assert!(!field_api.contains("dead_lazy_note"), "{field_api}");
    assert!(!field_api.contains("dead_returned_note"), "{field_api}");
    assert!(!field_api.contains("dead_method_note"), "{field_api}");
    assert!(!field_api.contains("dead_param_note"), "{field_api}");
    assert!(!field_api.contains("dead_impl_note"), "{field_api}");
    assert!(!field_api.contains("dead_dyn_note"), "{field_api}");
    assert!(!field_api.contains("dead_branch_note"), "{field_api}");
    assert!(!field_api.contains("dead_if_note"), "{field_api}");
    assert!(
        !field_api.contains("dead_if_collection_note"),
        "{field_api}"
    );
    assert!(
        !field_api.contains("dead_match_collection_note"),
        "{field_api}"
    );
    assert!(!field_api.contains("dead_alias_note"), "{field_api}");
    assert!(!field_api.contains("dead_tuple_note"), "{field_api}");
    assert!(!field_api.contains("dead_bag_note"), "{field_api}");
    assert!(!field_api.contains("dead_bag_views"), "{field_api}");
    assert!(!field_api.contains("dead_generic_note"), "{field_api}");
    assert!(!field_api.contains("dead_newtype_note"), "{field_api}");
    assert!(!field_api.contains("dead_local_alias_note"), "{field_api}");
    assert!(!field_api.contains("pub fn dead_summary"), "{field_api}");

    assert!(
        source_record.contains("pub struct SourceSnapshot"),
        "{source_record}"
    );
    assert!(source_record.contains("pub records:"), "{source_record}");
    assert!(!source_record.contains("unused_note"), "{source_record}");
    assert!(
        source_record.contains("pub struct SourceRecord"),
        "{source_record}"
    );
    assert!(source_record.contains("pub raw_label:"), "{source_record}");
    assert!(source_record.contains("pub raw_value:"), "{source_record}");
    assert!(
        source_record.contains("pub filter_code:"),
        "{source_record}"
    );
    assert!(source_record.contains("pub children:"), "{source_record}");
    assert!(!source_record.contains("dead_note"), "{source_record}");
    assert!(
        source_record.contains("pub struct SourceChild"),
        "{source_record}"
    );
    assert!(
        source_record.contains("pub child_label:"),
        "{source_record}"
    );
    assert!(
        source_record.contains("pub child_weight:"),
        "{source_record}"
    );
    assert!(
        !source_record.contains("dead_child_note"),
        "{source_record}"
    );
    assert!(source_record.contains("pub fn snapshot"), "{source_record}");
    assert!(!source_record.contains("dead_snapshot"), "{source_record}");

    assert!(
        view_record.contains("pub struct ViewRecord"),
        "{view_record}"
    );
    assert!(view_record.contains("pub title:"), "{view_record}");
    assert!(view_record.contains("pub score:"), "{view_record}");
    assert!(view_record.contains("pub loop_title:"), "{view_record}");
    assert!(!view_record.contains("dead_view_note"), "{view_record}");
    assert!(
        view_record.contains("pub struct FilterRecord"),
        "{view_record}"
    );
    assert!(view_record.contains("pub code:"), "{view_record}");
    assert!(view_record.contains("pub weight:"), "{view_record}");
    assert!(!view_record.contains("dead_filter_note"), "{view_record}");
    assert!(
        view_record.contains("pub struct ChildView"),
        "{view_record}"
    );
    assert!(view_record.contains("pub child_label:"), "{view_record}");
    assert!(view_record.contains("pub child_weight:"), "{view_record}");
    assert!(
        !view_record.contains("dead_child_view_note"),
        "{view_record}"
    );
    assert!(
        view_record.contains("pub struct LocalView"),
        "{view_record}"
    );
    assert!(view_record.contains("pub local_title:"), "{view_record}");
    assert!(view_record.contains("pub local_score:"), "{view_record}");
    assert!(!view_record.contains("dead_local_note"), "{view_record}");
    assert!(view_record.contains("pub struct LazyView"), "{view_record}");
    assert!(view_record.contains("pub lazy_title:"), "{view_record}");
    assert!(view_record.contains("pub lazy_score:"), "{view_record}");
    assert!(!view_record.contains("dead_lazy_note"), "{view_record}");
    assert!(
        view_record.contains("pub struct ReturnedView"),
        "{view_record}"
    );
    assert!(view_record.contains("pub returned_title:"), "{view_record}");
    assert!(view_record.contains("pub returned_score:"), "{view_record}");
    assert!(!view_record.contains("dead_returned_note"), "{view_record}");
    assert!(
        view_record.contains("pub struct MethodView"),
        "{view_record}"
    );
    assert!(view_record.contains("pub method_title:"), "{view_record}");
    assert!(view_record.contains("pub method_score:"), "{view_record}");
    assert!(!view_record.contains("dead_method_note"), "{view_record}");
    assert!(
        view_record.contains("pub struct ParamView"),
        "{view_record}"
    );
    assert!(view_record.contains("pub param_title:"), "{view_record}");
    assert!(view_record.contains("pub param_score:"), "{view_record}");
    assert!(!view_record.contains("dead_param_note"), "{view_record}");
    assert!(
        view_record.contains("pub struct ImplIterView"),
        "{view_record}"
    );
    assert!(view_record.contains("pub impl_title:"), "{view_record}");
    assert!(view_record.contains("pub impl_score:"), "{view_record}");
    assert!(!view_record.contains("dead_impl_note"), "{view_record}");
    assert!(
        view_record.contains("pub struct DynIterView"),
        "{view_record}"
    );
    assert!(view_record.contains("pub dyn_title:"), "{view_record}");
    assert!(view_record.contains("pub dyn_score:"), "{view_record}");
    assert!(!view_record.contains("dead_dyn_note"), "{view_record}");
    assert!(
        view_record.contains("pub struct BranchView"),
        "{view_record}"
    );
    assert!(view_record.contains("pub branch_title:"), "{view_record}");
    assert!(view_record.contains("pub branch_score:"), "{view_record}");
    assert!(!view_record.contains("dead_branch_note"), "{view_record}");
    assert!(view_record.contains("pub struct IfView"), "{view_record}");
    assert!(view_record.contains("pub if_title:"), "{view_record}");
    assert!(view_record.contains("pub if_score:"), "{view_record}");
    assert!(!view_record.contains("dead_if_note"), "{view_record}");
    assert!(
        view_record.contains("pub struct IfCollectionView"),
        "{view_record}"
    );
    assert!(
        view_record.contains("pub if_collection_title:"),
        "{view_record}"
    );
    assert!(
        view_record.contains("pub if_collection_score:"),
        "{view_record}"
    );
    assert!(
        !view_record.contains("dead_if_collection_note"),
        "{view_record}"
    );
    assert!(
        view_record.contains("pub struct MatchCollectionView"),
        "{view_record}"
    );
    assert!(
        view_record.contains("pub match_collection_title:"),
        "{view_record}"
    );
    assert!(
        view_record.contains("pub match_collection_score:"),
        "{view_record}"
    );
    assert!(
        !view_record.contains("dead_match_collection_note"),
        "{view_record}"
    );
    assert!(
        view_record.contains("pub type AliasViews = Vec < AliasView >")
            || view_record.contains("pub type AliasViews = Vec<AliasView>"),
        "{view_record}"
    );
    assert!(
        view_record.contains("pub struct AliasView"),
        "{view_record}"
    );
    assert!(view_record.contains("pub alias_title:"), "{view_record}");
    assert!(view_record.contains("pub alias_score:"), "{view_record}");
    assert!(!view_record.contains("dead_alias_note"), "{view_record}");
    assert!(
        view_record.contains("pub struct TupleView"),
        "{view_record}"
    );
    assert!(view_record.contains("pub tuple_title:"), "{view_record}");
    assert!(view_record.contains("pub tuple_score:"), "{view_record}");
    assert!(!view_record.contains("dead_tuple_note"), "{view_record}");
    assert!(
        view_record.contains("pub struct StructBag"),
        "{view_record}"
    );
    assert!(view_record.contains("pub bag_views:"), "{view_record}");
    assert!(!view_record.contains("dead_bag_views"), "{view_record}");
    assert!(
        view_record.contains("pub struct StructBagView"),
        "{view_record}"
    );
    assert!(view_record.contains("pub bag_title:"), "{view_record}");
    assert!(view_record.contains("pub bag_score:"), "{view_record}");
    assert!(!view_record.contains("dead_bag_note"), "{view_record}");
    assert!(
        view_record.contains("pub type GenericViews<T> = Vec<T>;"),
        "{view_record}"
    );
    assert!(
        view_record.contains("pub struct GenericAliasView"),
        "{view_record}"
    );
    assert!(view_record.contains("pub generic_title:"), "{view_record}");
    assert!(view_record.contains("pub generic_score:"), "{view_record}");
    assert!(!view_record.contains("dead_generic_note"), "{view_record}");
    assert!(
        view_record.contains("pub struct NewtypeViews"),
        "{view_record}"
    );
    assert!(view_record.contains("pub fn iter(&self)"), "{view_record}");
    assert!(
        view_record.contains("pub struct NewtypeView"),
        "{view_record}"
    );
    assert!(view_record.contains("pub newtype_title:"), "{view_record}");
    assert!(view_record.contains("pub newtype_score:"), "{view_record}");
    assert!(!view_record.contains("dead_newtype_note"), "{view_record}");
    assert!(
        view_record.contains("pub struct LocalAliasViews"),
        "{view_record}"
    );
    assert!(
        view_record.contains("let views = & self . 0 ;")
            || view_record.contains("let views = &self.0;"),
        "{view_record}"
    );
    assert!(
        view_record.contains("pub struct LocalAliasView"),
        "{view_record}"
    );
    assert!(
        view_record.contains("pub local_alias_title:"),
        "{view_record}"
    );
    assert!(
        view_record.contains("pub local_alias_score:"),
        "{view_record}"
    );
    assert!(
        !view_record.contains("dead_local_alias_note"),
        "{view_record}"
    );

    assert_cargo_check(&output, &target_dir, &root_source, &report);
}

#[test]
fn retains_public_support_fields_used_through_typed_closure_params() {
    let fixture = repo_root().join("fixtures/slice_cases/public_field_typed_closure_prune");
    let output = temp_path("slice-case-public-field-typed-closure-prune-output");
    let target_dir = temp_path("slice-case-public-field-typed-closure-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("public_field_typed_closure_prune fixture should slice");

    assert_eq!(report.packages, ["audit_record", "field_api", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_sum"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_source = read(output.join("root/src/lib.rs"));
    let field_api = read(output.join("field_api/src/lib.rs"));
    let audit_record = read(output.join("audit_record/src/lib.rs"));

    assert!(root_source.contains("pub fn selected_sum"), "{root_source}");
    assert!(!root_source.contains("dead_sum"), "{root_source}");

    assert!(field_api.contains("pub fn selected_sum"), "{field_api}");
    assert!(field_api.contains("record.value"), "{field_api}");
    assert!(!field_api.contains("dead_note"), "{field_api}");
    assert!(!field_api.contains("pub fn dead_sum"), "{field_api}");

    assert!(
        audit_record.contains("pub struct AuditRecord"),
        "{audit_record}"
    );
    assert!(audit_record.contains("pub value: u32"), "{audit_record}");
    assert!(!audit_record.contains("dead_note"), "{audit_record}");
    assert!(
        audit_record.contains("pub fn placeholder"),
        "{audit_record}"
    );
    assert!(!audit_record.contains("dead_record"), "{audit_record}");

    assert_cargo_check(&output, &target_dir, &root_source, &report);
}

#[test]
fn retains_public_support_fields_used_through_typed_closure_patterns() {
    let fixture = repo_root().join("fixtures/slice_cases/public_field_typed_closure_pattern_prune");
    let output = temp_path("slice-case-public-field-typed-closure-pattern-prune-output");
    let target_dir = temp_path("slice-case-public-field-typed-closure-pattern-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("public_field_typed_closure_pattern_prune fixture should slice");

    assert_eq!(report.packages, ["audit_record", "field_api", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_sum"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_source = read(output.join("root/src/lib.rs"));
    let field_api = read(output.join("field_api/src/lib.rs"));
    let audit_record = read(output.join("audit_record/src/lib.rs"));

    assert!(root_source.contains("pub fn selected_sum"), "{root_source}");
    assert!(!root_source.contains("dead_sum"), "{root_source}");

    assert!(field_api.contains("pub fn selected_sum"), "{field_api}");
    assert!(field_api.contains("{ value, .. }"), "{field_api}");
    assert!(!field_api.contains("dead_note"), "{field_api}");
    assert!(!field_api.contains("pub fn dead_sum"), "{field_api}");

    assert!(
        audit_record.contains("pub struct AuditRecord"),
        "{audit_record}"
    );
    assert!(audit_record.contains("pub value: u32"), "{audit_record}");
    assert!(!audit_record.contains("dead_note"), "{audit_record}");
    assert!(
        audit_record.contains("pub fn placeholder"),
        "{audit_record}"
    );
    assert!(!audit_record.contains("dead_record"), "{audit_record}");

    assert_cargo_check(&output, &target_dir, &root_source, &report);
}

#[test]
#[cfg(feature = "ra-hir")]
fn ra_hir_proves_support_sub_dependency_pruning_contract() {
    let (output, target_dir, report) = generate_slice_case_fixture(
        "sub_dependency_prune",
        "slice-case-sub-dependency-prune-ra-output",
        "slice-case-sub-dependency-prune-ra-target",
        AnalyzerMode::RustAnalyzerHir,
    );

    assert_eq!(report.packages, ["adapter", "leaf", "root"]);
    assert_ra_pruning_proof_complete(&report);
    assert_ra_promoted_reference_edges(&report);

    let root_source = read(output.join("root/src/lib.rs"));
    assert_cargo_check(&output, &target_dir, &root_source, &report);
}

#[test]
fn prunes_litter_theme_health_support_package_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/litter_theme_health_prune");
    let output = temp_path("slice-case-litter-theme-health-prune-output");
    let target_dir = temp_path("slice-case-litter-theme-health-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("litter_theme_health_prune fixture should slice");

    assert_eq!(report.packages, ["mobile-client", "tui"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "tui::theme::health_color"),
        "health_color root should be recorded: {:?}",
        report.roots
    );
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "tui::theme::health_symbol"),
        "health_symbol root should be recorded: {:?}",
        report.roots
    );

    let root_source = read(output.join("tui/src/lib.rs"));
    let theme_source = read(output.join("tui/src/theme.rs"));
    let mobile_root = read(output.join("mobile-client/src/lib.rs"));
    let store_mod = read(output.join("mobile-client/src/store/mod.rs"));
    let snapshot = read(output.join("mobile-client/src/store/snapshot.rs"));

    assert!(root_source.contains("pub mod theme"), "{root_source}");
    assert!(!root_source.contains("dead_preview"), "{root_source}");
    assert!(theme_source.contains("pub enum Color"), "{theme_source}");
    assert!(
        theme_source.contains("pub fn health_color"),
        "{theme_source}"
    );
    assert!(
        theme_source.contains("pub fn health_symbol"),
        "{theme_source}"
    );
    assert!(theme_source.contains("pub const SUCCESS"), "{theme_source}");
    assert!(!theme_source.contains("pub const ACCENT"), "{theme_source}");
    assert!(!theme_source.contains("pub fn accent"), "{theme_source}");
    assert!(
        !theme_source.contains("dead_status_label"),
        "{theme_source}"
    );
    assert!(!theme_source.contains("dead_name"), "{theme_source}");

    assert!(mobile_root.contains("pub mod store"), "{mobile_root}");
    assert!(!mobile_root.contains("dead_api"), "{mobile_root}");
    assert!(store_mod.contains("pub mod snapshot"), "{store_mod}");
    assert!(
        store_mod.contains("pub use snapshot::ServerHealthSnapshot"),
        "{store_mod}"
    );
    assert!(!store_mod.contains("AppSnapshot"), "{store_mod}");
    assert!(!store_mod.contains("DeadHealthSnapshot"), "{store_mod}");
    assert!(!store_mod.contains("dead_health_label"), "{store_mod}");
    assert!(!output.join("mobile-client/src/store/private.rs").exists());
    assert!(
        snapshot.contains("pub enum ServerHealthSnapshot"),
        "{snapshot}"
    );
    assert!(!snapshot.contains("AppSnapshot"), "{snapshot}");
    assert!(!snapshot.contains("DeadHealthSnapshot"), "{snapshot}");
    assert!(!snapshot.contains("dead_snapshot"), "{snapshot}");
    assert!(!snapshot.contains("is_connected"), "{snapshot}");
    assert!(!snapshot.contains("dead_label"), "{snapshot}");

    assert_cargo_check(&output, &target_dir, &root_source, &report);
}

#[test]
#[cfg(feature = "ra-hir")]
fn ra_hir_proves_litter_theme_support_pruning_contract() {
    let (output, target_dir, report) = generate_slice_case_fixture(
        "litter_theme_health_prune",
        "slice-case-litter-theme-health-prune-ra-output",
        "slice-case-litter-theme-health-prune-ra-target",
        AnalyzerMode::RustAnalyzerHir,
    );

    assert_eq!(report.packages, ["mobile-client", "tui"]);
    assert_ra_pruning_proof_complete(&report);
    assert_ra_promoted_reference_edges(&report);

    let root_source = read(output.join("tui/src/lib.rs"));
    assert_cargo_check(&output, &target_dir, &root_source, &report);
}

#[test]
fn prunes_litter_conversation_render_support_packages_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/litter_conversation_render_prune");
    let output = temp_path("slice-case-litter-conversation-render-prune-output");
    let target_dir = temp_path("slice-case-litter-conversation-render-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("litter_conversation_render_prune fixture should slice");

    assert_eq!(
        report.packages,
        ["codex-core", "codex-protocol", "codex-tui"]
    );
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "codex-tui::screens::conversation::render"),
        "conversation render root should be recorded: {:?}",
        report.roots
    );

    let tui_root = read(output.join("codex-tui/src/lib.rs"));
    let screens_root = read(output.join("codex-tui/src/screens/mod.rs"));
    let conversation = read(output.join("codex-tui/src/screens/conversation.rs"));
    let core_root = read(output.join("codex-core/src/lib.rs"));
    let core_live = read(output.join("codex-core/src/live.rs"));
    let protocol_root = read(output.join("codex-protocol/src/lib.rs"));
    let protocol_live = read(output.join("codex-protocol/src/live.rs"));

    assert!(tui_root.contains("pub mod screens"), "{tui_root}");
    assert_absent(
        "codex-tui/src/lib.rs",
        &tui_root,
        &["theme", "dead_tui_entry"],
    );
    assert!(
        screens_root.contains("pub mod conversation"),
        "{screens_root}"
    );
    assert_absent("codex-tui/src/screens/mod.rs", &screens_root, &["settings"]);
    assert!(conversation.contains("pub fn render"), "{conversation}");
    for token in [
        "fn render_header",
        "fn render_event",
        "fn render_message",
        "fn role_label",
        "fn render_footer",
    ] {
        assert!(
            conversation.contains(token),
            "missing {token:?}\n{conversation}"
        );
    }
    assert_absent(
        "codex-tui/src/screens/conversation.rs",
        &conversation,
        &["dead_conversation_panel", "dead_summary"],
    );
    assert!(!output.join("codex-tui/src/screens/settings.rs").exists());
    assert!(!output.join("codex-tui/src/theme.rs").exists());

    assert!(core_root.contains("mod live"), "{core_root}");
    assert!(core_root.contains("ConversationState"), "{core_root}");
    assert_absent(
        "codex-core/src/lib.rs",
        &core_root,
        &["mod dead", "DeadState"],
    );
    assert!(
        core_live.contains("pub struct ConversationState"),
        "{core_live}"
    );
    assert!(core_live.contains("pub struct ScreenStats"), "{core_live}");
    assert!(core_live.contains("pub fn visible_events"), "{core_live}");
    assert_absent(
        "codex-core/src/live.rs",
        &core_live,
        &["dead_live_state", "dead_summary"],
    );
    assert!(!output.join("codex-core/src/dead.rs").exists());

    assert!(protocol_root.contains("mod live"), "{protocol_root}");
    assert!(
        protocol_root.contains("ConversationEvent"),
        "{protocol_root}"
    );
    assert_absent(
        "codex-protocol/src/lib.rs",
        &protocol_root,
        &["mod dead", "DeadEvent"],
    );
    assert!(
        protocol_live.contains("pub enum ConversationEvent"),
        "{protocol_live}"
    );
    assert!(
        protocol_live.contains("pub struct Message"),
        "{protocol_live}"
    );
    assert!(protocol_live.contains("pub enum Role"), "{protocol_live}");
    assert_absent(
        "codex-protocol/src/live.rs",
        &protocol_live,
        &["dead_live_event", "dead_kind", "dead_render"],
    );
    assert!(!output.join("codex-protocol/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &tui_root, &report);
}

#[test]
fn prunes_litter_conversation_state_serde_support_package_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/litter_conversation_state_serde_prune");
    let output = temp_path("slice-case-litter-conversation-state-serde-prune-output");
    let target_dir = temp_path("slice-case-litter-conversation-state-serde-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("litter_conversation_state_serde_prune fixture should slice");

    assert_eq!(report.packages, ["codex-ipc", "codex-state"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "codex-ipc::conversation_preview"),
        "conversation preview root should be recorded: {:?}",
        report.roots
    );

    let ipc_root = read(output.join("codex-ipc/src/lib.rs"));
    let state_manifest = read(output.join("codex-state/Cargo.toml"));
    let state_root = read(output.join("codex-state/src/lib.rs"));
    let state_live = read(output.join("codex-state/src/live.rs"));

    assert!(
        ipc_root.contains("pub fn conversation_preview"),
        "{ipc_root}"
    );
    assert_absent(
        "codex-ipc/src/lib.rs",
        &ipc_root,
        &["dead_conversation_preview"],
    );

    for dependency in ["serde", "serde_json", "thiserror"] {
        assert!(
            state_manifest.contains(dependency),
            "codex-state manifest should retain dependency {dependency:?}\n{state_manifest}"
        );
    }
    assert!(state_root.contains("mod live"), "{state_root}");
    assert!(
        state_root.contains("pub use live::{conversation_preview, ConversationError}"),
        "{state_root}"
    );
    assert_absent(
        "codex-state/src/lib.rs",
        &state_root,
        &[
            "mod dead",
            "DeadConversationError",
            "dead_conversation_preview",
        ],
    );
    assert!(!output.join("codex-state/src/dead.rs").exists());

    for token in [
        "use serde::Deserialize",
        "use thiserror::Error",
        "#[derive(Debug, Error)]",
        "#[error(\"deserialize conversation state: {0}\")]",
        "Deserialize(#[from] serde_json::Error)",
        "#[derive(Debug, Deserialize)]",
        "#[serde(rename_all = \"camelCase\")]",
        "struct DesktopConversationState",
        "struct ThreadState",
        "#[serde(tag = \"type\", rename_all = \"camelCase\")]",
        "enum TurnState",
        "UserMessage",
        "AgentMessage",
        "ToolResult",
        "struct PendingApproval",
        "pub fn conversation_preview",
    ] {
        assert!(
            state_live.contains(token),
            "missing {token:?}\n{state_live}"
        );
    }
    assert_absent(
        "codex-state/src/live.rs",
        &state_live,
        &[
            "dead_conversation_preview",
            "dead_live_helper",
            "DeadConversationError",
            "DeadConversationState",
        ],
    );

    assert_cargo_check(&output, &target_dir, &ipc_root, &report);
}

#[test]
fn prunes_litter_mobile_session_support_packages_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/litter_mobile_session_prune");
    let output = temp_path("slice-case-litter-mobile-session-prune-output");
    let target_dir = temp_path("slice-case-litter-mobile-session-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("litter_mobile_session_prune fixture should slice");

    assert_eq!(
        report.packages,
        [
            "codex-mobile-client",
            "session-api",
            "session-core",
            "session-protocol"
        ]
    );
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string()
                == "codex-mobile-client::ffi::session::connect_session_status"),
        "mobile session root should be recorded: {:?}",
        report.roots
    );

    let mobile_root = read(output.join("codex-mobile-client/src/lib.rs"));
    let ffi_root = read(output.join("codex-mobile-client/src/ffi/mod.rs"));
    let session_source = read(output.join("codex-mobile-client/src/ffi/session.rs"));
    let api_root = read(output.join("session-api/src/lib.rs"));
    let api_live = read(output.join("session-api/src/live.rs"));
    let core_root = read(output.join("session-core/src/lib.rs"));
    let core_live = read(output.join("session-core/src/live.rs"));
    let protocol_root = read(output.join("session-protocol/src/lib.rs"));
    let protocol_live = read(output.join("session-protocol/src/live.rs"));

    assert!(mobile_root.contains("pub mod ffi"), "{mobile_root}");
    assert_absent(
        "codex-mobile-client/src/lib.rs",
        &mobile_root,
        &["settings", "dead_mobile_entry"],
    );
    assert!(ffi_root.contains("pub mod session"), "{ffi_root}");
    assert_absent(
        "codex-mobile-client/src/ffi/mod.rs",
        &ffi_root,
        &["voice_handoff"],
    );
    assert!(
        session_source.contains("pub fn connect_session_status"),
        "{session_source}"
    );
    assert_absent(
        "codex-mobile-client/src/ffi/session.rs",
        &session_source,
        &["dead_session_status", "dead_summary"],
    );
    assert!(!output.join("codex-mobile-client/src/settings.rs").exists());
    assert!(!output
        .join("codex-mobile-client/src/ffi/voice_handoff.rs")
        .exists());

    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("SessionHandle"), "{api_root}");
    assert_absent(
        "session-api/src/lib.rs",
        &api_root,
        &["mod dead", "DeadSessionHandle", "dead_api_report"],
    );
    for token in [
        "pub struct SessionRequest",
        "pub struct SessionHandle",
        "pub struct SessionStatusDto",
        "pub fn connect",
        "pub fn status",
        "pub fn from_status",
    ] {
        assert!(api_live.contains(token), "missing {token:?}\n{api_live}");
    }
    assert_absent(
        "session-api/src/live.rs",
        &api_live,
        &[
            "dead_exported_status",
            "dead_connect",
            "DeadSessionApi",
            "dead_live_api",
            "dead_render",
        ],
    );
    assert!(!output.join("session-api/src/dead.rs").exists());

    assert!(core_root.contains("mod live"), "{core_root}");
    assert!(core_root.contains("SessionEngine"), "{core_root}");
    assert_absent(
        "session-core/src/lib.rs",
        &core_root,
        &["mod dead", "DeadSessionEngine", "dead_core_report"],
    );
    for token in [
        "pub struct SessionEngine",
        "pub struct SessionStatus",
        "pub fn connect",
        "pub fn status",
        "pub fn new",
        "pub fn label",
        "pub fn kind",
        "fn normalize_label",
    ] {
        assert!(core_live.contains(token), "missing {token:?}\n{core_live}");
    }
    assert_absent(
        "session-core/src/live.rs",
        &core_live,
        &["dead_debug", "dead_label", "dead_live_core"],
    );
    assert!(!output.join("session-core/src/dead.rs").exists());

    assert!(protocol_root.contains("mod live"), "{protocol_root}");
    assert!(protocol_root.contains("WireSession"), "{protocol_root}");
    assert_absent(
        "session-protocol/src/lib.rs",
        &protocol_root,
        &["mod dead", "DeadWireSession", "dead_protocol_report"],
    );
    for token in [
        "pub enum WireStatusKind",
        "pub struct WireSession",
        "pub fn from_status",
        "pub fn label",
        "pub fn kind",
    ] {
        assert!(
            protocol_live.contains(token),
            "missing {token:?}\n{protocol_live}"
        );
    }
    assert_absent(
        "session-protocol/src/live.rs",
        &protocol_live,
        &["dead_summary", "dead_live_protocol"],
    );
    assert!(!output.join("session-protocol/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &mobile_root, &report);
}

#[test]
fn prunes_litter_bridge_ipc_support_packages_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/litter_bridge_ipc_prune");
    let output = temp_path("slice-case-litter-bridge-ipc-prune-output");
    let target_dir = temp_path("slice-case-litter-bridge-ipc-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("litter_bridge_ipc_prune fixture should slice");

    assert_no_production_hazard(&report, "semantic_unresolved_paths");

    assert_eq!(
        report.packages,
        ["bridge-core", "bridge-protocol", "codex-bridge"]
    );
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "codex-bridge::ipc::handle_frame"),
        "bridge ipc root should be recorded: {:?}",
        report.roots
    );

    let bridge_root = read(output.join("codex-bridge/src/lib.rs"));
    let ipc_source = read(output.join("codex-bridge/src/ipc.rs"));
    let core_root = read(output.join("bridge-core/src/lib.rs"));
    let core_live = read(output.join("bridge-core/src/live.rs"));
    let protocol_root = read(output.join("bridge-protocol/src/lib.rs"));
    let protocol_live = read(output.join("bridge-protocol/src/live.rs"));

    assert!(bridge_root.contains("pub mod ipc"), "{bridge_root}");
    assert_absent(
        "codex-bridge/src/lib.rs",
        &bridge_root,
        &["pub mod ssh", "dead_bridge_entry"],
    );
    assert!(ipc_source.contains("pub fn handle_frame"), "{ipc_source}");
    assert_absent(
        "codex-bridge/src/ipc.rs",
        &ipc_source,
        &["dead_handle_frame", "BridgeError::dead"],
    );
    assert!(!output.join("codex-bridge/src/ssh.rs").exists());

    assert!(core_root.contains("mod live"), "{core_root}");
    assert!(core_root.contains("BridgeSession"), "{core_root}");
    assert_absent(
        "bridge-core/src/lib.rs",
        &core_root,
        &["mod dead", "DeadBridgeSession", "dead_core_report"],
    );
    for token in [
        "pub enum SessionState",
        "pub struct BridgeSession",
        "pub fn new",
        "pub fn id",
        "pub fn state",
        "pub fn dispatch_method",
        "fn normalize_id",
        "fn normalize_payload",
        "fn state_label",
    ] {
        assert!(core_live.contains(token), "missing {token:?}\n{core_live}");
    }
    assert_absent(
        "bridge-core/src/live.rs",
        &core_live,
        &["dead_debug", "dead_live_dispatch"],
    );
    assert!(!output.join("bridge-core/src/dead.rs").exists());

    assert!(protocol_root.contains("mod live"), "{protocol_root}");
    assert!(protocol_root.contains("WireFrame"), "{protocol_root}");
    assert_absent(
        "bridge-protocol/src/lib.rs",
        &protocol_root,
        &["mod dead", "DeadWireFrame", "dead_protocol_report"],
    );
    for token in [
        "pub enum Method",
        "pub struct WireFrame",
        "pub struct ResponseFrame",
        "pub enum BridgeError",
        "pub fn from_wire",
        "pub fn decode",
        "pub fn method",
        "pub fn session_id",
        "pub fn payload",
        "pub fn ok",
        "pub fn core",
    ] {
        assert!(
            protocol_live.contains(token),
            "missing {token:?}\n{protocol_live}"
        );
    }
    assert_absent(
        "bridge-protocol/src/live.rs",
        &protocol_live,
        &[
            "dead_name",
            "dead_wire_debug",
            "dead_response",
            "pub fn dead(",
            "dead_live_protocol",
        ],
    );
    assert!(!output.join("bridge-protocol/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &bridge_root, &report);
}

#[test]
fn prunes_litter_event_callback_support_packages_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/litter_event_callback_prune");
    let output = temp_path("slice-case-litter-event-callback-prune-output");
    let target_dir = temp_path("slice-case-litter-event-callback-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("litter_event_callback_prune fixture should slice");

    assert_eq!(
        report.packages,
        [
            "codex-mobile-client",
            "event-api",
            "event-core",
            "event-protocol"
        ]
    );
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string()
                == "codex-mobile-client::ffi::events::register_event_callback"),
        "event callback root should be recorded: {:?}",
        report.roots
    );
    assert_production_hazard(&report, "trait_object_surfaces", "warning");

    let mobile_root = read(output.join("codex-mobile-client/src/lib.rs"));
    let ffi_root = read(output.join("codex-mobile-client/src/ffi/mod.rs"));
    let events_source = read(output.join("codex-mobile-client/src/ffi/events.rs"));
    let api_root = read(output.join("event-api/src/lib.rs"));
    let api_live = read(output.join("event-api/src/live.rs"));
    let core_root = read(output.join("event-core/src/lib.rs"));
    let core_live = read(output.join("event-core/src/live.rs"));
    let protocol_root = read(output.join("event-protocol/src/lib.rs"));
    let protocol_live = read(output.join("event-protocol/src/live.rs"));

    assert!(mobile_root.contains("pub mod ffi"), "{mobile_root}");
    assert_absent(
        "codex-mobile-client/src/lib.rs",
        &mobile_root,
        &["diagnostics", "dead_mobile_event_entry"],
    );
    assert!(ffi_root.contains("pub mod events"), "{ffi_root}");
    assert_absent("codex-mobile-client/src/ffi/mod.rs", &ffi_root, &["voice"]);
    assert!(
        events_source.contains("pub fn register_event_callback"),
        "{events_source}"
    );
    assert_absent(
        "codex-mobile-client/src/ffi/events.rs",
        &events_source,
        &["dead_event_callback", "dead_summary"],
    );
    assert!(!output
        .join("codex-mobile-client/src/diagnostics.rs")
        .exists());
    assert!(!output.join("codex-mobile-client/src/ffi/voice.rs").exists());

    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("EventCallbackHandle"), "{api_root}");
    assert_absent(
        "event-api/src/lib.rs",
        &api_root,
        &["mod dead", "DeadEventApi", "dead_event_api_report"],
    );
    for token in [
        "pub struct EventRegistration",
        "pub struct EventCallbackHandle",
        "pub struct EventSnapshotDto",
        "pub fn register",
        "pub fn emit_preview",
        "pub fn from_wire",
        "struct DefaultCallback",
        "impl EventCallback for DefaultCallback",
    ] {
        assert!(api_live.contains(token), "missing {token:?}\n{api_live}");
    }
    assert_absent(
        "event-api/src/live.rs",
        &api_live,
        &[
            "dead_exported_preview",
            "dead_register",
            "DeadEventApi",
            "dead_live_event_api",
            "dead_render",
        ],
    );
    assert!(!output.join("event-api/src/dead.rs").exists());

    assert!(core_root.contains("mod live"), "{core_root}");
    assert!(core_root.contains("EventBus"), "{core_root}");
    assert_absent(
        "event-core/src/lib.rs",
        &core_root,
        &["mod dead", "DeadEventBus", "dead_event_core_report"],
    );
    for token in [
        "pub enum EventKind",
        "pub struct EventEnvelope",
        "pub trait EventCallback",
        "pub struct EventBus",
        "pub fn new",
        "pub fn set_callback",
        "pub fn emit",
        "fn classify_payload",
        "fn normalize_label",
        "fn normalize_payload",
    ] {
        assert!(core_live.contains(token), "missing {token:?}\n{core_live}");
    }
    assert!(core_live.contains("dyn EventCallback"), "{core_live}");
    assert_absent(
        "event-core/src/live.rs",
        &core_live,
        &[
            "dead_render",
            "dead_trait_method",
            "dead_debug",
            "dead_live_event_core",
        ],
    );
    assert!(!output.join("event-core/src/dead.rs").exists());

    assert!(protocol_root.contains("mod live"), "{protocol_root}");
    assert!(protocol_root.contains("WireEvent"), "{protocol_root}");
    assert_absent(
        "event-protocol/src/lib.rs",
        &protocol_root,
        &["mod dead", "DeadWireEvent", "dead_event_protocol_report"],
    );
    for token in [
        "pub enum WireEventKind",
        "pub struct WireEvent",
        "pub fn connected",
        "pub fn message",
        "pub fn closed",
        "pub fn missing",
        "pub fn label",
        "pub fn body",
        "pub fn kind",
        "fn new",
    ] {
        assert!(
            protocol_live.contains(token),
            "missing {token:?}\n{protocol_live}"
        );
    }
    assert_absent(
        "event-protocol/src/live.rs",
        &protocol_live,
        &["dead_summary", "dead_live_event_protocol"],
    );
    assert!(!output.join("event-protocol/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &mobile_root, &report);
}

#[test]
fn prunes_litter_out_dir_codegen_support_packages_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/litter_out_dir_codegen_prune");
    let output = temp_path("slice-case-litter-out-dir-codegen-prune-output");
    let target_dir = temp_path("slice-case-litter-out-dir-codegen-prune-target");
    seed_out_dir_generated_source(
        "litter_codegen_bindings.rs",
        "pub fn generated_event(label: &str) -> super::GeneratedEvent { super::generated_event_helper(label) }\n",
    );

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("litter_out_dir_codegen_prune fixture should slice");

    assert_eq!(
        report.packages,
        ["codex-core", "codex-protocol-codegen", "codex-tui"]
    );
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "codex-tui::wire::render_generated_event"),
        "out-dir codegen root should be recorded: {:?}",
        report.roots
    );
    assert_production_hazard(&report, "retained_build_scripts", "error");
    assert_production_hazard(&report, "out_dir_source_include_macros", "error");
    assert_no_production_hazard(&report, "semantic_unresolved_paths");

    let tui_root = read(output.join("codex-tui/src/lib.rs"));
    let wire_source = read(output.join("codex-tui/src/wire.rs"));
    let core_root = read(output.join("codex-core/src/lib.rs"));
    let core_live = read(output.join("codex-core/src/live.rs"));
    let codegen_manifest = read(output.join("codex-protocol-codegen/Cargo.toml"));
    let codegen_root = read(output.join("codex-protocol-codegen/src/lib.rs"));
    let codegen_live = read(output.join("codex-protocol-codegen/src/live.rs"));

    assert!(tui_root.contains("pub mod wire"), "{tui_root}");
    assert_absent(
        "codex-tui/src/lib.rs",
        &tui_root,
        &["pub mod panels", "dead_tui_codegen_entry"],
    );
    assert!(
        wire_source.contains("pub fn render_generated_event"),
        "{wire_source}"
    );
    assert_absent(
        "codex-tui/src/wire.rs",
        &wire_source,
        &["dead_generated_event", "dead_core_codegen"],
    );
    assert!(!output.join("codex-tui/src/panels.rs").exists());

    assert!(core_root.contains("mod live"), "{core_root}");
    assert!(core_root.contains("CoreGeneratedEvent"), "{core_root}");
    assert_absent(
        "codex-core/src/lib.rs",
        &core_root,
        &["mod dead", "DeadGeneratedCore", "dead_core_codegen"],
    );
    for token in [
        "pub struct CoreGeneratedEvent",
        "pub fn new",
        "pub fn label",
        "pub fn render_core_generated_event",
        "fn normalize_core_label",
    ] {
        assert!(core_live.contains(token), "missing {token:?}\n{core_live}");
    }
    assert_absent(
        "codex-core/src/live.rs",
        &core_live,
        &["dead_summary", "dead_live_core_codegen"],
    );
    assert!(!output.join("codex-core/src/dead.rs").exists());

    assert!(codegen_manifest.contains("build = \"build.rs\""));
    assert!(output.join("codex-protocol-codegen/build.rs").exists());
    assert!(codegen_root.contains("mod live"), "{codegen_root}");
    assert!(codegen_root.contains("GeneratedEvent"), "{codegen_root}");
    assert_absent(
        "codex-protocol-codegen/src/lib.rs",
        &codegen_root,
        &[
            "mod dead",
            "DeadGeneratedProtocol",
            "dead_protocol_codegen_report",
        ],
    );
    assert!(
        codegen_live.contains("include!(concat!(env!(\"OUT_DIR\")"),
        "{codegen_live}"
    );
    for token in [
        "pub struct GeneratedEvent",
        "pub fn new",
        "pub fn label",
        "pub fn payload",
        "pub fn render",
        "pub fn selected_wire_event",
        "pub fn generated_event_helper",
        "fn normalize_generated_payload",
    ] {
        assert!(
            codegen_live.contains(token),
            "missing {token:?}\n{codegen_live}"
        );
    }
    assert_absent(
        "codex-protocol-codegen/src/live.rs",
        &codegen_live,
        &["dead_debug", "dead_wire_event", "dead_generated_helper"],
    );
    assert!(!output.join("codex-protocol-codegen/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &tui_root, &report);
}

#[test]
fn prunes_litter_reconnect_grouped_import_support_package_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/litter_reconnect_import_prune");
    let output = temp_path("slice-case-litter-reconnect-import-prune-output");
    let target_dir = temp_path("slice-case-litter-reconnect-import-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("litter_reconnect_import_prune fixture should slice");

    assert_eq!(report.packages, ["codex-client", "codex-ipc"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "codex-ipc::reconnect_summary"),
        "reconnect root should be recorded: {:?}",
        report.roots
    );

    let ipc_root = read(output.join("codex-ipc/src/lib.rs"));
    let client_root = read(output.join("codex-client/src/lib.rs"));
    let client_live = read(output.join("codex-client/src/live.rs"));

    assert!(ipc_root.contains("pub fn reconnect_summary"), "{ipc_root}");
    assert!(
        ipc_root.contains("codex_client::build_reconnect"),
        "{ipc_root}"
    );
    assert_absent(
        "codex-ipc/src/lib.rs",
        &ipc_root,
        &["dead_reconnect_summary", "dead_reconnect"],
    );

    assert!(client_root.contains("mod live"), "{client_root}");
    assert!(
        client_root.contains("pub use live::{build_reconnect, ReconnectState}"),
        "{client_root}"
    );
    assert_absent(
        "codex-client/src/lib.rs",
        &client_root,
        &["mod dead", "DeadReconnectState", "dead_reconnect"],
    );
    assert!(!output.join("codex-client/src/dead.rs").exists());

    for token in [
        "pub struct ReconnectState",
        "id: String",
        "pub fn new",
        "pub fn summary",
        "pub fn build_reconnect",
        "fn normalize_reconnect_id",
    ] {
        assert!(
            client_live.contains(token),
            "missing {token:?}\n{client_live}"
        );
    }
    assert_absent(
        "codex-client/src/live.rs",
        &client_live,
        &[
            "use std::",
            "PathBuf",
            "Arc",
            "Duration",
            "Cow",
            "socket_path",
            "retry_after",
            "fallback_label",
            "dead_details",
            "dead_live_reconnect",
        ],
    );

    assert_cargo_check(&output, &target_dir, &ipc_root, &report);
}

#[test]
fn prunes_litter_reconnect_callback_support_package_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/litter_reconnect_callback_prune");
    let output = temp_path("slice-case-litter-reconnect-callback-prune-output");
    let target_dir = temp_path("slice-case-litter-reconnect-callback-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("litter_reconnect_callback_prune fixture should slice");

    assert_eq!(report.packages, ["codex-client", "codex-ipc"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "codex-ipc::reconnect_callback_summary"),
        "reconnect callback root should be recorded: {:?}",
        report.roots
    );
    assert_production_hazard(&report, "trait_object_surfaces", "warning");

    let ipc_root = read(output.join("codex-ipc/src/lib.rs"));
    let client_root = read(output.join("codex-client/src/lib.rs"));
    let client_live = read(output.join("codex-client/src/live.rs"));

    assert!(
        ipc_root.contains("pub fn reconnect_callback_summary"),
        "{ipc_root}"
    );
    assert_absent(
        "codex-ipc/src/lib.rs",
        &ipc_root,
        &["dead_reconnect_callback_summary", "dead_reconnect"],
    );

    assert!(client_root.contains("mod live"), "{client_root}");
    assert!(
        client_root.contains(
            "pub use live::{selected_reconnect, ClientResponse, ReconnectController, RequestHandler}"
        ),
        "{client_root}"
    );
    assert_absent(
        "codex-client/src/lib.rs",
        &client_root,
        &["mod dead", "DeadReconnectController", "dead_reconnect"],
    );
    assert!(!output.join("codex-client/src/dead.rs").exists());

    for token in [
        "pub type ConnectFuture",
        "pub type Connector",
        "pub struct IpcClient",
        "pub struct IpcError",
        "pub struct ClientRequest",
        "pub struct ClientResponse",
        "pub trait RequestHandler",
        "pub struct ReconnectController",
        "Arc<RwLock<Option<Arc<dyn RequestHandler>>>>",
        "dyn Fn() -> ConnectFuture",
        "pub fn new",
        "pub fn set_handler",
        "pub fn summarize",
        "struct EchoHandler",
        "impl RequestHandler for EchoHandler",
        "fn normalize_id",
        "pub fn selected_reconnect",
    ] {
        assert!(
            client_live.contains(token),
            "missing {token:?}\n{client_live}"
        );
    }
    assert_absent(
        "codex-client/src/live.rs",
        &client_live,
        &[
            "dead_client_debug",
            "dead_response",
            "dead_handler_method",
            "dead_controller_debug",
            "dead_live_reconnect",
        ],
    );

    assert_cargo_check(&output, &target_dir, &ipc_root, &report);
}

#[test]
#[cfg(feature = "ra-hir")]
fn ra_hir_proves_high_risk_fixture_pruning_matrix() {
    let cases = [
        RaHardFixture {
            fixture: "proc_macro_surface_prune",
            packages: &["api", "macro_support", "model", "root"],
            hazards: &[
                ("custom_attribute_macros", "warning"),
                ("custom_derive_macros", "warning"),
            ],
        },
        RaHardFixture {
            fixture: "macro_generated_prune",
            packages: &["macro_api", "macro_support", "root"],
            hazards: &[("custom_macro_invocations", "warning")],
        },
        RaHardFixture {
            fixture: "litter_conversation_render_prune",
            packages: &["codex-core", "codex-protocol", "codex-tui"],
            hazards: &[],
        },
        RaHardFixture {
            fixture: "litter_conversation_state_serde_prune",
            packages: &["codex-ipc", "codex-state"],
            hazards: &[],
        },
        RaHardFixture {
            fixture: "litter_mobile_session_prune",
            packages: &[
                "codex-mobile-client",
                "session-api",
                "session-core",
                "session-protocol",
            ],
            hazards: &[
                ("custom_attribute_macros", "warning"),
                ("conditional_compilation_attrs", "warning"),
                ("custom_derive_macros", "warning"),
            ],
        },
        RaHardFixture {
            fixture: "litter_bridge_ipc_prune",
            packages: &["bridge-core", "bridge-protocol", "codex-bridge"],
            hazards: &[],
        },
        RaHardFixture {
            fixture: "litter_event_callback_prune",
            packages: &[
                "codex-mobile-client",
                "event-api",
                "event-core",
                "event-protocol",
            ],
            hazards: &[
                ("custom_attribute_macros", "warning"),
                ("conditional_compilation_attrs", "warning"),
                ("custom_derive_macros", "warning"),
                ("trait_object_surfaces", "warning"),
            ],
        },
        RaHardFixture {
            fixture: "litter_out_dir_codegen_prune",
            packages: &["codex-core", "codex-protocol-codegen", "codex-tui"],
            hazards: &[
                ("out_dir_source_include_macros", "error"),
                ("retained_build_scripts", "error"),
            ],
        },
        RaHardFixture {
            fixture: "litter_reconnect_import_prune",
            packages: &["codex-client", "codex-ipc"],
            hazards: &[],
        },
        RaHardFixture {
            fixture: "litter_reconnect_callback_prune",
            packages: &["codex-client", "codex-ipc"],
            hazards: &[("trait_object_surfaces", "warning")],
        },
        RaHardFixture {
            fixture: "macro_receiver_prune",
            packages: &["macro_api", "macro_support", "root"],
            hazards: &[("custom_macro_invocations", "warning")],
        },
        RaHardFixture {
            fixture: "cfg_attr_uniffi_prune",
            packages: &["cfg_api", "cfg_model", "root", "uniffi"],
            hazards: &[
                ("custom_attribute_macros", "warning"),
                ("custom_derive_macros", "warning"),
                ("conditional_compilation_attrs", "warning"),
            ],
        },
        RaHardFixture {
            fixture: "returned_dyn_trait_prune",
            packages: &["dyn_api", "dyn_model", "root"],
            hazards: &[("trait_object_surfaces", "warning")],
        },
        RaHardFixture {
            fixture: "callback_store_prune",
            packages: &["callback_store_api", "callback_store_support", "root"],
            hazards: &[("trait_object_surfaces", "warning")],
        },
        RaHardFixture {
            fixture: "ffi_export_prune",
            packages: &["ffi_api", "ffi_support", "root"],
            hazards: &[],
        },
        RaHardFixture {
            fixture: "static_registry_prune",
            packages: &["registry_api", "registry_support", "root"],
            hazards: &[],
        },
        RaHardFixture {
            fixture: "poll_adapter_prune",
            packages: &["poll_api", "poll_support", "root"],
            hazards: &[],
        },
        RaHardFixture {
            fixture: "uniffi_runtime_prune",
            packages: &["root", "runtime_api", "runtime_support"],
            hazards: &[],
        },
        RaHardFixture {
            fixture: "out_dir_generated_prune",
            packages: &["generated_support", "root"],
            hazards: &[
                ("out_dir_source_include_macros", "error"),
                ("retained_build_scripts", "error"),
            ],
        },
    ];

    for case in cases {
        let (output, target_dir, report) = generate_slice_case_fixture(
            case.fixture,
            &format!("slice-case-{}-ra-output", case.fixture),
            &format!("slice-case-{}-ra-target", case.fixture),
            AnalyzerMode::RustAnalyzerHir,
        );

        let actual_packages = report
            .packages
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        assert_eq!(
            actual_packages, case.packages,
            "{} retained unexpected packages",
            case.fixture
        );
        assert_ra_pruning_proof_complete(&report);
        assert_expected_production_hazards(&report, case.hazards);

        let diagnostic_source = diagnostic_source_for_report(&output, &report);
        assert_cargo_check(&output, &target_dir, &diagnostic_source, &report);
    }
}

#[test]
fn prunes_out_dir_generated_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/out_dir_generated_prune");
    let output = temp_path("slice-case-out-dir-generated-prune-output");
    let target_dir = temp_path("slice-case-out-dir-generated-prune-target");
    seed_out_dir_generated_source(
        "slice_case_generated.rs",
        "pub fn generated_value() -> u32 { super::out_dir_generated_helper() }\n",
    );

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("out_dir_generated_prune fixture should slice");

    assert_eq!(report.packages, ["generated_support", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_generated_total"),
        "selected root should be recorded: {:?}",
        report.roots
    );
    assert_production_hazard(&report, "retained_build_scripts", "error");
    assert_production_hazard(&report, "out_dir_source_include_macros", "error");
    assert_no_production_hazard(&report, "semantic_unresolved_paths");

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let support_manifest = read(output.join("generated_support/Cargo.toml"));
    let support_source = read(output.join("generated_support/src/lib.rs"));

    assert!(
        root_manifest.contains("../generated_support"),
        "{root_manifest}"
    );
    assert!(
        root_source.contains("pub fn selected_generated_total"),
        "{root_source}"
    );
    assert!(
        root_source.contains("generated_support::selected_generated_value"),
        "{root_source}"
    );
    assert_absent(
        "root/src/lib.rs",
        &root_source,
        &["dead_generated_total", "dead_generated_value"],
    );

    assert!(support_manifest.contains("build = \"build.rs\""));
    assert!(output.join("generated_support/build.rs").exists());
    assert!(support_source.contains("include!(concat!(env!(\"OUT_DIR\")"));
    assert!(
        support_source.contains("pub fn selected_generated_value"),
        "{support_source}"
    );
    assert!(
        support_source.contains("pub fn out_dir_generated_helper"),
        "{support_source}"
    );
    assert_absent(
        "generated_support/src/lib.rs",
        &support_source,
        &["dead_generated_value", "dead_out_dir_helper"],
    );

    assert_cargo_check(&output, &target_dir, &root_source, &report);
}

#[test]
fn prunes_cfg_attr_uniffi_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/cfg_attr_uniffi_prune");
    let output = temp_path("slice-case-cfg-attr-uniffi-prune-output");
    let target_dir = temp_path("slice-case-cfg-attr-uniffi-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("cfg_attr_uniffi_prune fixture should slice");

    assert_eq!(report.packages, ["cfg_api", "cfg_model", "root", "uniffi"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_cfg_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );
    assert_production_hazard(&report, "conditional_compilation_attrs", "warning");
    assert_production_hazard(&report, "custom_attribute_macros", "warning");
    assert_production_hazard(&report, "custom_derive_macros", "warning");

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("cfg_api/Cargo.toml"));
    let api_root = read(output.join("cfg_api/src/lib.rs"));
    let api_live = read(output.join("cfg_api/src/live.rs"));
    let model_root = read(output.join("cfg_model/src/lib.rs"));
    let model_live = read(output.join("cfg_model/src/live.rs"));

    assert!(root_manifest.contains("../cfg_api"), "{root_manifest}");
    assert!(root_source.contains("pub fn selected_cfg_report"));
    assert!(root_source.contains("cfg_api::selected_cfg_report"));
    assert_absent("root/src/lib.rs", &root_source, &["dead_cfg_report"]);

    assert!(api_manifest.contains("../cfg_model"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_cfg_report"), "{api_root}");
    assert_absent(
        "cfg_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_cfg_report"],
    );
    assert!(api_live.contains("cfg_model::selected_cfg_record"));
    assert_absent("cfg_api/src/live.rs", &api_live, &["dead_live_cfg_report"]);
    assert!(!output.join("cfg_api/src/dead.rs").exists());

    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(model_root.contains("selected_cfg_record"), "{model_root}");
    assert!(model_root.contains("LiveCfgRecord"), "{model_root}");
    assert!(model_root.contains("LiveCfgStatus"), "{model_root}");
    assert_absent(
        "cfg_model/src/lib.rs",
        &model_root,
        &["mod dead", "DeadCfgRecord", "dead_cfg_record"],
    );
    assert!(model_live.contains("pub struct LiveCfgRecord"));
    assert!(model_live.contains("pub enum LiveCfgStatus"));
    assert!(model_live.contains("pub fn render"));
    assert!(model_live.contains("pub fn label"));
    assert!(model_live.contains("pub fn selected_cfg_record"));
    assert_absent(
        "cfg_model/src/live.rs",
        &model_live,
        &["dead_method", "dead_live_cfg_record", "dead-cfg"],
    );
    assert!(!output.join("cfg_model/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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
        &[
            "DEAD_ASSET",
            "dead_live_asset",
            "dead-asset",
            "test_only_fixture_asset",
            "test_only.txt",
        ],
    );
    assert!(output.join("asset_codec/src/assets/header.txt").exists());
    assert!(output.join("asset_codec/src/assets/body.txt").exists());
    assert!(!output.join("asset_codec/src/assets/dead.txt").exists());
    assert!(!output.join("asset_codec/src/assets/test_only.txt").exists());
    assert!(!output.join("asset_codec/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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
    assert!(
        api_live.contains("use facade_support::facade::*;"),
        "{api_live}"
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
    assert!(support_root.contains("pub use nested::*"), "{support_root}");
    assert!(support_root.contains("build_live"), "{support_root}");
    assert!(support_root.contains("LiveRecord"), "{support_root}");
    assert_absent(
        "facade_support/src/lib.rs",
        &support_root,
        &[
            "mod dead",
            "pub use facade::*",
            "DeadRecord",
            "dead_factory",
        ],
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_production_hazard(&report, "custom_macro_invocations", "warning");
    assert_no_production_hazard(&report, "semantic_usage_mapping_incomplete");

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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
}

#[test]
fn prunes_global_mutex_multi_registry_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/global_mutex_multi_registry_prune");
    let output = temp_path("slice-case-global-mutex-multi-registry-prune-output");
    let target_dir = temp_path("slice-case-global-mutex-multi-registry-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("global_mutex_multi_registry_prune fixture should slice");

    assert_eq!(report.packages, ["registry_api", "registry_store", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_registry_lookup"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("registry_api/Cargo.toml"));
    let api_root = read(output.join("registry_api/src/lib.rs"));
    let api_live = read(output.join("registry_api/src/live.rs"));
    let store_root = read(output.join("registry_store/src/lib.rs"));
    let store_live = read(output.join("registry_store/src/live.rs"));

    assert!(root_manifest.contains("../registry_api"), "{root_manifest}");
    assert!(
        root_source.contains("pub fn selected_registry_lookup"),
        "{root_source}"
    );
    assert!(
        root_source.contains("registry_api::selected_registry_lookup"),
        "{root_source}"
    );
    assert_absent("root/src/lib.rs", &root_source, &["dead_registry_lookup"]);

    assert!(api_manifest.contains("../registry_store"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_registry_lookup"), "{api_root}");
    assert_absent(
        "registry_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_registry_lookup"],
    );
    assert!(api_live.contains("registry_store::register_entry"));
    assert!(api_live.contains("registry_store::lookup_entry"));
    assert!(api_live.contains(".map(|entry| entry.render())"));
    assert_absent(
        "registry_api/src/live.rs",
        &api_live,
        &[
            "dead_live_registry_lookup",
            "remove_entry",
            "list_entries",
            "dead_render",
        ],
    );
    assert!(!output.join("registry_api/src/dead.rs").exists());

    assert!(store_root.contains("mod live"), "{store_root}");
    assert!(store_root.contains("register_entry"), "{store_root}");
    assert!(store_root.contains("lookup_entry"), "{store_root}");
    assert!(store_root.contains("RegistryEntry"), "{store_root}");
    assert_absent(
        "registry_store/src/lib.rs",
        &store_root,
        &[
            "mod dead",
            "DeadRegistryEntry",
            "dead_registry_metric",
            "remove_entry",
            "list_entries",
        ],
    );
    assert!(store_live.contains("use std::collections::BTreeMap"));
    assert!(store_live.contains("use std::sync::{Arc, Mutex, OnceLock}"));
    assert!(store_live.contains("static REGISTRY"));
    assert!(
        store_live.contains("pub struct RegistryEntry"),
        "{store_live}"
    );
    assert!(store_live.contains("pub fn new"), "{store_live}");
    assert!(store_live.contains("pub fn render"), "{store_live}");
    assert!(store_live.contains("fn registry"), "{store_live}");
    assert!(store_live.contains("pub fn register_entry"), "{store_live}");
    assert!(store_live.contains("pub fn lookup_entry"), "{store_live}");
    assert_absent(
        "registry_store/src/live.rs",
        &store_live,
        &[
            "pub fn remove_entry",
            "pub fn list_entries",
            "dead_live_registry",
            "dead_render",
            "dead-registry-entry",
        ],
    );
    assert!(!output.join("registry_store/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
}

#[test]
fn prunes_request_json_patch_state_machine_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/request_json_patch_state_machine_prune");
    let output = temp_path("slice-case-request-json-patch-state-machine-prune-output");
    let target_dir = temp_path("slice-case-request-json-patch-state-machine-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("request_json_patch_state_machine_prune fixture should slice");

    assert_eq!(report.packages, ["request_api", "request_state", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_patch_summary"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("request_api/Cargo.toml"));
    let api_root = read(output.join("request_api/src/lib.rs"));
    let api_live = read(output.join("request_api/src/live.rs"));
    let state_manifest = read(output.join("request_state/Cargo.toml"));
    let state_root = read(output.join("request_state/src/lib.rs"));
    let state_live = read(output.join("request_state/src/live.rs"));

    assert!(root_manifest.contains("../request_api"), "{root_manifest}");
    assert!(
        root_source.contains("pub fn selected_patch_summary"),
        "{root_source}"
    );
    assert!(
        root_source.contains("request_api::selected_patch_summary"),
        "{root_source}"
    );
    assert_absent("root/src/lib.rs", &root_source, &["dead_patch_summary"]);

    assert!(api_manifest.contains("../request_state"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_patch_summary"), "{api_root}");
    assert_absent(
        "request_api/src/lib.rs",
        &api_root,
        &["mod dead", "DeadPatchRequest", "dead_patch_summary"],
    );
    assert!(
        api_live.contains("request_state::apply_patch_request"),
        "{api_live}"
    );
    assert_absent(
        "request_api/src/live.rs",
        &api_live,
        &["dead_live_patch_summary"],
    );
    assert!(!output.join("request_api/src/dead.rs").exists());

    for dependency in ["serde", "serde_json"] {
        assert!(
            state_manifest.contains(dependency),
            "request_state manifest should retain dependency {dependency:?}\n{state_manifest}"
        );
    }
    assert!(state_root.contains("mod live"), "{state_root}");
    assert!(state_root.contains("apply_patch_request"), "{state_root}");
    assert!(state_root.contains("PatchCommand"), "{state_root}");
    assert_absent(
        "request_state/src/lib.rs",
        &state_root,
        &["mod dead", "DeadState", "dead_state_summary"],
    );
    assert!(!output.join("request_state/src/dead.rs").exists());

    for token in [
        "use serde::{Deserialize, Serialize}",
        "use serde_json::{Map, Value}",
        "#[serde(rename_all = \"camelCase\")]",
        "#[serde(tag = \"op\", content = \"value\", rename_all = \"camelCase\")]",
        "pub enum PatchCommand",
        "Add(PatchPayload)",
        "Replace(PatchPayload)",
        "Remove(RemovePayload)",
        "Test(TestPayload)",
        "pub struct PatchPayload",
        "pub struct RemovePayload",
        "pub struct TestPayload",
        "pub struct PatchSegment",
        "fn path_label",
        "fn render",
        "pub fn apply_patch_request",
    ] {
        assert!(
            state_live.contains(token),
            "missing {token:?}\n{state_live}"
        );
    }
    assert_absent(
        "request_state/src/live.rs",
        &state_live,
        &[
            "dead_live_state_summary",
            "dead_payload_debug",
            "dead_segment_debug",
            "DeadState",
            "dead-state",
        ],
    );

    assert_cargo_check(&output, &target_dir, &root_source, &report);
}

#[test]
fn prunes_serde_adjacent_contract_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/serde_adjacent_contract_prune");
    let output = temp_path("slice-case-serde-adjacent-contract-prune-output");
    let target_dir = temp_path("slice-case-serde-adjacent-contract-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("serde_adjacent_contract_prune fixture should slice");

    assert_eq!(
        report.packages,
        ["root", "serde_adjacent_api", "serde_adjacent_model"]
    );
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_adjacent_summary"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("serde_adjacent_api/Cargo.toml"));
    let api_root = read(output.join("serde_adjacent_api/src/lib.rs"));
    let api_live = read(output.join("serde_adjacent_api/src/live.rs"));
    let model_manifest = read(output.join("serde_adjacent_model/Cargo.toml"));
    let model_root = read(output.join("serde_adjacent_model/src/lib.rs"));
    let model_live = read(output.join("serde_adjacent_model/src/live.rs"));

    assert!(
        root_manifest.contains("../serde_adjacent_api"),
        "{root_manifest}"
    );
    assert!(
        root_source.contains("pub fn selected_adjacent_summary"),
        "{root_source}"
    );
    assert!(
        root_source.contains("serde_adjacent_api::selected_adjacent_summary"),
        "{root_source}"
    );
    assert_absent("root/src/lib.rs", &root_source, &["dead_adjacent_summary"]);

    assert!(
        api_manifest.contains("../serde_adjacent_model"),
        "{api_manifest}"
    );
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_adjacent_summary"), "{api_root}");
    assert_absent(
        "serde_adjacent_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_adjacent_summary"],
    );
    assert!(
        api_live.contains("serde_adjacent_model::parse_live_command"),
        "{api_live}"
    );
    assert_absent(
        "serde_adjacent_api/src/live.rs",
        &api_live,
        &["dead_live_adjacent_summary"],
    );
    assert!(!output.join("serde_adjacent_api/src/dead.rs").exists());

    for dependency in ["serde", "serde_json"] {
        assert!(
            model_manifest.contains(dependency),
            "serde_adjacent_model manifest should retain dependency {dependency:?}\n{model_manifest}"
        );
    }
    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(model_root.contains("parse_live_command"), "{model_root}");
    assert!(model_root.contains("LiveEnvelope"), "{model_root}");
    assert_absent(
        "serde_adjacent_model/src/lib.rs",
        &model_root,
        &["mod dead", "DeadEnvelope", "dead_contract_summary"],
    );
    assert!(!output.join("serde_adjacent_model/src/dead.rs").exists());

    for token in [
        "use serde::{Deserialize, Serialize}",
        "#[serde(tag = \"kind\", content = \"payload\", rename_all = \"camelCase\")]",
        "pub enum LiveEnvelope",
        "Started(StartedPayload)",
        "Update {",
        "Failed(FailurePayload)",
        "pub struct StartedPayload",
        "pub labels: Vec<String>",
        "pub struct LiveMessage",
        "pub metadata: Vec<String>",
        "pub struct FailurePayload",
        "pub recoverable: bool",
        "pub fn parse_live_command",
    ] {
        assert!(
            model_live.contains(token),
            "missing {token:?}\n{model_live}"
        );
    }
    assert_absent(
        "serde_adjacent_model/src/live.rs",
        &model_live,
        &[
            "dead_live_contract_summary",
            "DeadPayload",
            "dead_payload_debug",
            "dead-content-meta-word",
            "dead-kind-meta-word",
            "dead-payload-meta-word",
            "pub fn content",
            "pub fn kind",
            "pub fn payload",
        ],
    );

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
}

#[test]
fn prunes_uniffi_async_runtime_object_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/uniffi_async_runtime_object_prune");
    let output = temp_path("slice-case-uniffi-async-runtime-object-prune-output");
    let target_dir = temp_path("slice-case-uniffi-async-runtime-object-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("uniffi_async_runtime_object_prune fixture should slice");

    assert_eq!(
        report.packages,
        ["root", "runtime_api", "runtime_support", "uniffi"]
    );
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_async_runtime_status"),
        "selected root should be recorded: {:?}",
        report.roots
    );
    assert_production_hazard(&report, "conditional_compilation_attrs", "warning");
    assert_production_hazard(&report, "custom_attribute_macros", "warning");
    assert_production_hazard(&report, "custom_derive_macros", "warning");

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("runtime_api/Cargo.toml"));
    let api_root = read(output.join("runtime_api/src/lib.rs"));
    let api_live = read(output.join("runtime_api/src/live.rs"));
    let support_root = read(output.join("runtime_support/src/lib.rs"));
    let support_live = read(output.join("runtime_support/src/live.rs"));

    assert!(root_manifest.contains("../runtime_api"), "{root_manifest}");
    assert!(
        root_source.contains("pub async fn selected_async_runtime_status"),
        "{root_source}"
    );
    assert!(
        root_source.contains("runtime_api::selected_async_runtime_status(raw).await"),
        "{root_source}"
    );
    assert_absent(
        "root/src/lib.rs",
        &root_source,
        &["dead_async_runtime_status"],
    );

    assert!(
        api_manifest.contains("../runtime_support"),
        "{api_manifest}"
    );
    assert!(api_manifest.contains("../uniffi"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(
        api_root.contains("selected_async_runtime_status"),
        "{api_root}"
    );
    assert_absent(
        "runtime_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_async_runtime_status"],
    );
    assert!(api_live.contains("derive(uniffi::Object)"), "{api_live}");
    assert!(api_live.contains("derive(uniffi::Record)"), "{api_live}");
    assert!(
        api_live.contains("uniffi::export(async_runtime = \"tokio\")"),
        "{api_live}"
    );
    assert!(api_live.contains("pub struct RuntimeBridge"), "{api_live}");
    assert!(
        api_live.contains("pub struct RuntimeStatusDto"),
        "{api_live}"
    );
    assert!(api_live.contains("pub fn from_snapshot"), "{api_live}");
    assert!(api_live.contains("pub fn render"), "{api_live}");
    assert!(api_live.contains("pub fn shared"), "{api_live}");
    assert!(
        api_live.contains("pub async fn current_status"),
        "{api_live}"
    );
    assert!(
        api_live.contains("pub async fn selected_async_runtime_status"),
        "{api_live}"
    );
    assert_absent(
        "runtime_api/src/live.rs",
        &api_live,
        &[
            "dead_exported_status",
            "dead_local_bridge",
            "dead_live_async_runtime_status",
            "dead_render",
            "dead-status",
            "dead-bridge",
        ],
    );
    assert!(!output.join("runtime_api/src/dead.rs").exists());

    assert!(support_root.contains("mod live"), "{support_root}");
    assert!(support_root.contains("shared_runtime"), "{support_root}");
    assert!(support_root.contains("RuntimeCore"), "{support_root}");
    assert!(support_root.contains("RuntimeSnapshot"), "{support_root}");
    assert_absent(
        "runtime_support/src/lib.rs",
        &support_root,
        &["mod dead", "DeadRuntime", "dead_runtime_report"],
    );
    assert!(support_live.contains("use std::sync::{Arc, OnceLock}"));
    assert!(support_live.contains("static SHARED_RUNTIME"));
    assert!(support_live.contains("pub struct RuntimeCore"));
    assert!(support_live.contains("pub fn new"));
    assert!(support_live.contains("pub async fn status"));
    assert!(support_live.contains("pub struct RuntimeSnapshot"));
    assert!(support_live.contains("pub fn label"));
    assert!(support_live.contains("pub fn ready"));
    assert!(support_live.contains("pub fn shared_runtime"));
    assert_absent(
        "runtime_support/src/live.rs",
        &support_live,
        &[
            "dead_status",
            "dead_snapshot",
            "dead_live_runtime_report",
            "dead-runtime-status",
            "dead-snapshot",
        ],
    );
    assert!(!output.join("runtime_support/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_production_hazard(&report, "custom_macro_invocations", "warning");
    assert_no_production_hazard(&report, "semantic_usage_mapping_incomplete");

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
    assert_absent(
        "macro_support/src/live.rs",
        &support_live,
        &["pub(crate) use render_record"],
    );
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    let public_reexports = &report.usage.public_reexports;
    assert_eq!(public_reexports.status, "proven", "{public_reexports:#?}");
    assert!(
        public_reexports.summary.facade_chain_targets >= 1,
        "returned_api facade chain should be proved through returned_model: {public_reexports:#?}"
    );
    assert!(
        public_reexports.entries.iter().any(|entry| {
            entry.package == "returned_api"
                && entry.visible == "ReturnedSubscription"
                && entry
                    .resolved_targets
                    .iter()
                    .any(|target| target == "returned_model::ReturnedSubscription")
                && entry.classification == "retained"
        }),
        "returned support facade should resolve to retained model item: {public_reexports:#?}"
    );
    assert_eq!(public_reexports.summary.prunable_targets, 0);
    assert_eq!(public_reexports.summary.unclassified_targets, 0);

    assert_cargo_check(&output, &target_dir, &root_source, &report);
}

#[test]
fn prunes_returned_dyn_trait_support_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/returned_dyn_trait_prune");
    let output = temp_path("slice-case-returned-dyn-trait-prune-output");
    let target_dir = temp_path("slice-case-returned-dyn-trait-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("returned_dyn_trait_prune fixture should slice");

    assert_eq!(report.packages, ["dyn_api", "dyn_model", "root"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_dyn_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );
    assert_production_hazard(&report, "trait_object_surfaces", "warning");

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("dyn_api/Cargo.toml"));
    let api_root = read(output.join("dyn_api/src/lib.rs"));
    let api_live = read(output.join("dyn_api/src/live.rs"));
    let model_root = read(output.join("dyn_model/src/lib.rs"));
    let model_live = read(output.join("dyn_model/src/live.rs"));

    assert!(root_manifest.contains("../dyn_api"), "{root_manifest}");
    assert!(root_source.contains("pub fn selected_dyn_report"));
    assert!(root_source.contains("dyn_api::selected_dyn_report"));
    assert_absent("root/src/lib.rs", &root_source, &["dead_dyn_report"]);

    assert!(api_manifest.contains("../dyn_model"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_dyn_report"), "{api_root}");
    assert_absent(
        "dyn_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_dyn_report"],
    );
    assert!(api_live.contains("dyn_model::selected_reader"));
    assert!(api_live.contains("dyn_model::render_reader"));
    assert_absent("dyn_api/src/live.rs", &api_live, &["dead_live_dyn_report"]);
    assert!(!output.join("dyn_api/src/dead.rs").exists());

    assert!(model_root.contains("mod live"), "{model_root}");
    assert!(model_root.contains("selected_reader"), "{model_root}");
    assert!(model_root.contains("render_reader"), "{model_root}");
    assert!(model_root.contains("LiveReader"), "{model_root}");
    assert!(model_root.contains("Reader"), "{model_root}");
    assert_absent(
        "dyn_model/src/lib.rs",
        &model_root,
        &["mod dead", "DeadReader", "dead_dyn_summary"],
    );
    assert!(model_live.contains("pub trait Reader"), "{model_live}");
    assert!(model_live.contains("pub struct LiveReader"), "{model_live}");
    assert!(
        model_live.contains("impl Reader for LiveReader"),
        "{model_live}"
    );
    assert!(
        model_live.contains("pub fn selected_reader"),
        "{model_live}"
    );
    assert!(model_live.contains("Box < dyn Reader >") || model_live.contains("Box<dyn Reader>"));
    assert!(model_live.contains("pub fn render_reader"), "{model_live}");
    assert_absent(
        "dyn_model/src/live.rs",
        &model_live,
        &[
            "dead_default",
            "dead_method",
            "dead_live_dyn_summary",
            "dead-reader",
        ],
    );
    assert!(!output.join("dyn_model/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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
        &[
            "Stop(StopParams)",
            "List(ListParams)",
            "Notify(DeadNotification)",
            "pub struct StopParams",
            "pub struct ListParams",
            "pub enum DeadNotification",
            "unsupported",
            "dead_method",
            "dead_live_dispatch",
            "dead-dispatch",
        ],
    );
    assert!(!output.join("dispatch_model/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
}

#[test]
fn prunes_lazy_regex_support_dependency_chain_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/lazy_regex_prune");
    let output = temp_path("slice-case-lazy-regex-prune-output");
    let target_dir = temp_path("slice-case-lazy-regex-prune-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("lazy_regex_prune fixture should slice");

    assert_eq!(
        report.packages,
        ["parser_support", "regex_api", "regex_lite", "root"]
    );
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_route_report"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let api_manifest = read(output.join("regex_api/Cargo.toml"));
    let api_root = read(output.join("regex_api/src/lib.rs"));
    let api_live = read(output.join("regex_api/src/live.rs"));
    let parser_manifest = read(output.join("parser_support/Cargo.toml"));
    let parser_root = read(output.join("parser_support/src/lib.rs"));
    let parser_live = read(output.join("parser_support/src/live.rs"));
    let regex_root = read(output.join("regex_lite/src/lib.rs"));
    let regex_live = read(output.join("regex_lite/src/live.rs"));

    assert!(root_manifest.contains("../regex_api"), "{root_manifest}");
    assert!(root_source.contains("pub fn selected_route_report"));
    assert!(root_source.contains("regex_api::selected_route_report"));
    assert_absent("root/src/lib.rs", &root_source, &["dead_route_report"]);

    assert!(api_manifest.contains("../parser_support"), "{api_manifest}");
    assert!(api_root.contains("mod live"), "{api_root}");
    assert!(api_root.contains("selected_route_report"), "{api_root}");
    assert_absent(
        "regex_api/src/lib.rs",
        &api_root,
        &["mod dead", "dead_route_report"],
    );
    assert!(api_live.contains("parser_support::selected_route_id"));
    assert_absent(
        "regex_api/src/live.rs",
        &api_live,
        &["dead_live_route_report"],
    );
    assert!(!output.join("regex_api/src/dead.rs").exists());

    assert!(
        parser_manifest.contains("../regex_lite"),
        "{parser_manifest}"
    );
    assert!(parser_root.contains("mod live"), "{parser_root}");
    assert!(parser_root.contains("selected_route_id"), "{parser_root}");
    assert_absent(
        "parser_support/src/lib.rs",
        &parser_root,
        &["mod dead", "dead_route_id"],
    );
    assert!(parser_live.contains("use regex_lite::Regex"));
    assert!(parser_live.contains("use std::sync::LazyLock"));
    assert!(parser_live.contains("static ROUTE_RE"));
    assert!(parser_live.contains("LazyLock::new"));
    assert!(parser_live.contains("Regex::new(\"/session/\")"));
    assert!(parser_live.contains("error.message()"));
    assert!(parser_live.contains(".captures(raw)"));
    assert!(parser_live.contains("captures.name(\"id\")"));
    assert!(parser_live.contains("route_match.as_str()"));
    assert_absent(
        "parser_support/src/live.rs",
        &parser_live,
        &[
            "dead_live_route_id",
            "replace_all",
            "dead_message",
            "\"/dead/\"",
        ],
    );
    assert!(!output.join("parser_support/src/dead.rs").exists());

    assert!(regex_root.contains("mod live"), "{regex_root}");
    assert!(regex_root.contains("Regex"), "{regex_root}");
    assert!(regex_root.contains("Captures"), "{regex_root}");
    assert!(regex_root.contains("Match"), "{regex_root}");
    assert!(regex_root.contains("RegexError"), "{regex_root}");
    assert_absent(
        "regex_lite/src/lib.rs",
        &regex_root,
        &["mod dead", "DeadRegex", "dead_regex_debug"],
    );
    assert!(regex_live.contains("pub struct Regex"), "{regex_live}");
    assert!(regex_live.contains("pub fn new"), "{regex_live}");
    assert!(regex_live.contains("pub fn captures"), "{regex_live}");
    assert!(regex_live.contains("pub struct Captures"), "{regex_live}");
    assert!(regex_live.contains("pub fn name"), "{regex_live}");
    assert!(regex_live.contains("pub struct Match"), "{regex_live}");
    assert!(regex_live.contains("pub fn as_str"), "{regex_live}");
    assert!(regex_live.contains("pub struct RegexError"), "{regex_live}");
    assert!(regex_live.contains("pub fn message"), "{regex_live}");
    assert_absent(
        "regex_lite/src/live.rs",
        &regex_live,
        &[
            "pub fn replace_all",
            "dead_summary",
            "dead_value",
            "dead_message",
            "dead_live_regex_report",
            "dead-regex",
        ],
    );
    assert!(!output.join("regex_lite/src/dead.rs").exists());

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
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
fn prunes_iterator_flatten_option_array_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "iterator_flatten_option_array_prune",
        "iterator_flatten_option_array",
        &[
            "[Some(IteratorFlattenOptionArrayPayload::new(raw)), None]",
            ".into_iter()",
            ".flatten()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadIteratorFlattenOptionArrayItem",
        "dead-iterator-flatten-option-array",
        "pub fn bump_and_render",
        &[],
    );
}

#[test]
fn prunes_iterator_flatten_option_refs_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "iterator_flatten_option_refs_prune",
        "iterator_flatten_option_refs",
        &[
            "[&left, &right]",
            ".into_iter()",
            ".flatten()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadIteratorFlattenOptionRefsItem",
        "dead-iterator-flatten-option-refs",
        "pub fn bump_and_render",
        &[],
    );
}

#[test]
fn prunes_iterator_flatten_result_vec_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "iterator_flatten_result_vec_prune",
        "iterator_flatten_result_vec",
        &[
            "Result<IteratorFlattenResultVecPayload, IteratorFlattenResultVecError>",
            ".into_iter()",
            ".flatten()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadIteratorFlattenResultVecItem",
        "dead-iterator-flatten-result-vec",
        "pub fn bump_and_render",
        &["dead_error_method"],
    );
}

#[test]
fn prunes_iterator_flatten_result_refs_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "iterator_flatten_result_refs_prune",
        "iterator_flatten_result_refs",
        &[
            "[&left, &right]",
            ".into_iter()",
            ".flatten()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadIteratorFlattenResultRefsItem",
        "dead-iterator-flatten-result-refs",
        "pub fn bump_and_render",
        &["dead_error_method"],
    );
}

#[test]
fn prunes_iterator_flatten_vec_vec_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "iterator_flatten_vec_vec_prune",
        "iterator_flatten_vec_vec",
        &[
            "Vec<Vec<IteratorFlattenVecVecPayload>>",
            ".into_iter()",
            ".flatten()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadIteratorFlattenVecVecItem",
        "dead-iterator-flatten-vec-vec",
        "pub fn bump_and_render",
        &[],
    );
}

#[test]
fn prunes_iterator_flatten_take_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "iterator_flatten_take_map_prune",
        "iterator_flatten_take_map",
        &[
            "Vec<Option<IteratorFlattenTakeMapPayload>>",
            ".flatten()",
            ".take(1)",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadIteratorFlattenTakeMapItem",
        "dead-iterator-flatten-take-map",
        "pub fn bump_and_render",
        &[],
    );
}

#[test]
fn prunes_iterator_filter_map_result_ok_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "iterator_filter_map_result_ok_prune",
        "iterator_filter_map_result_ok",
        &[
            "Result::ok",
            ".filter_map(Result::ok)",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadIteratorFilterMapResultOkItem",
        "dead-iterator-filter-map-result-ok",
        "pub fn bump_and_render",
        &["dead_error_method"],
    );
}

#[test]
fn prunes_iterator_filter_map_result_err_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "iterator_filter_map_result_err_prune",
        "iterator_filter_map_result_err",
        &[
            "Result::err",
            ".filter_map(Result::err)",
            ".map(|err| err.render_error())",
            "pub fn render_error",
        ],
        "DeadIteratorFilterMapResultErrItem",
        "dead-iterator-filter-map-result-err",
        "pub fn render_label",
        &["dead_error_method"],
    );
}

#[test]
fn prunes_iterator_find_map_result_ok_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "iterator_find_map_result_ok_prune",
        "iterator_find_map_result_ok",
        &[
            ".find_map(Result::ok)",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadIteratorFindMapResultOkItem",
        "dead-iterator-find-map-result-ok",
        "pub fn bump_and_render",
        &["dead_error_method"],
    );
}

#[test]
fn prunes_iterator_filter_map_option_identity_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "iterator_filter_map_option_identity_prune",
        "iterator_filter_map_option_identity",
        &[
            "Vec<Option<IteratorFilterMapOptionIdentityPayload>>",
            ".filter_map(|payload| payload)",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadIteratorFilterMapOptionIdentityItem",
        "dead-iterator-filter-map-option-identity",
        "pub fn bump_and_render",
        &[],
    );
}

#[test]
fn prunes_iterator_filter_map_match_result_ok_question_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "iterator_filter_map_match_result_ok_question_prune",
        "iterator_filter_map_match_result_ok_question",
        &[
            ".filter_map(|request| match request.method()",
            ".ok()?",
            ".render_label()",
            "pub fn try_parse",
            "pub fn render_label",
        ],
        "DeadIteratorFilterMapMatchResultOkQuestionItem",
        "dead-iterator-filter-map-match-result-ok-question",
        "pub fn unused_label",
        &["dead_error_method"],
    );
}

#[test]
fn prunes_iterator_filter_map_match_option_question_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "iterator_filter_map_match_option_question_prune",
        "iterator_filter_map_match_option_question",
        &[
            ".filter_map(|request| match request.method()",
            "lookup_payload(request.raw())?",
            "pub fn render_label",
        ],
        "DeadIteratorFilterMapMatchOptionQuestionItem",
        "dead-iterator-filter-map-match-option-question",
        "pub fn unused_label",
        &[],
    );
}

#[test]
fn prunes_iterator_find_map_method_ref_option_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "iterator_find_map_method_ref_option_prune",
        "iterator_find_map_method_ref_option",
        &[
            ".find_map(IteratorFindMapMethodRefOptionPayload::maybe_label)",
            "pub fn maybe_label",
            "pub fn render_label",
        ],
        "DeadIteratorFindMapMethodRefOptionItem",
        "dead-iterator-find-map-method-ref-option",
        "pub fn unused_label",
        &[],
    );
}

#[test]
fn prunes_hashmap_values_find_map_method_ref_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "hashmap_values_find_map_method_ref_prune",
        "hashmap_values_find_map_method_ref",
        &[
            "use std::collections::HashMap;",
            ".values()",
            ".find_map(HashmapValuesFindMapMethodRefPayload::maybe_label)",
            "pub fn maybe_label",
        ],
        "DeadHashmapValuesFindMapMethodRefItem",
        "dead-hashmap-values-find-map-method-ref",
        "pub fn unused_label",
        &[],
    );
}

#[test]
fn prunes_btreemap_values_find_map_method_ref_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "btreemap_values_find_map_method_ref_prune",
        "btreemap_values_find_map_method_ref",
        &[
            "use std::collections::BTreeMap;",
            ".values()",
            ".find_map(BtreemapValuesFindMapMethodRefPayload::maybe_label)",
            "pub fn maybe_label",
        ],
        "DeadBtreemapValuesFindMapMethodRefItem",
        "dead-btreemap-values-find-map-method-ref",
        "pub fn unused_label",
        &[],
    );
}

#[test]
fn prunes_iterator_find_map_enum_match_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "iterator_find_map_enum_match_prune",
        "iterator_find_map_enum_match",
        &[
            ".find_map(|event| match event",
            "IteratorFindMapEnumMatchEvent::Live(payload)",
            "pub fn render_label",
        ],
        "DeadIteratorFindMapEnumMatchItem",
        "dead-iterator-find-map-enum-match",
        "pub fn unused_label",
        &[],
    );
}

#[test]
fn prunes_iterator_filter_map_borrowed_field_match_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "iterator_filter_map_borrowed_field_match_prune",
        "iterator_filter_map_borrowed_field_match",
        &[
            "match &entry.path",
            "Some(payload) => Some(payload.render_label())",
            "pub fn render_label",
        ],
        "DeadIteratorFilterMapBorrowedFieldMatchItem",
        "dead-iterator-filter-map-borrowed-field-match",
        "pub fn unused_label",
        &["ShadowPayload"],
    );
}

#[test]
fn prunes_iterator_filter_map_let_else_option_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "iterator_filter_map_let_else_option_prune",
        "iterator_filter_map_let_else_option",
        &[
            "let Some(payload) = entry.path.as_ref() else",
            "Some(payload.render_label())",
            "pub fn render_label",
        ],
        "DeadIteratorFilterMapLetElseOptionItem",
        "dead-iterator-filter-map-let-else-option",
        "pub fn unused_label",
        &[],
    );
}

#[test]
fn prunes_iterator_flat_map_match_enum_vec_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "iterator_flat_map_match_enum_vec_prune",
        "iterator_flat_map_match_enum_vec",
        &[
            ".flat_map(|command| match command",
            "vec![payload]",
            ".map(|payload| payload.render_label())",
        ],
        "DeadIteratorFlatMapMatchEnumVecItem",
        "dead-iterator-flat-map-match-enum-vec",
        "pub fn unused_label",
        &[],
    );
}

#[test]
fn prunes_iterator_find_map_nested_ok_question_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "iterator_find_map_nested_ok_question_prune",
        "iterator_find_map_nested_ok_question",
        &[
            ".find_map(|part| {",
            ".ok()?;",
            "Some(payload.render_label())",
        ],
        "DeadIteratorFindMapNestedOkQuestionItem",
        "dead-iterator-find-map-nested-ok-question",
        "pub fn unused_label",
        &["dead_error_method"],
    );
}

#[test]
fn prunes_hashmap_iter_filter_map_match_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "hashmap_iter_filter_map_match_prune",
        "hashmap_iter_filter_map_match",
        &[
            "use std::collections::HashMap;",
            ".filter_map(|(key, payload)| match key.as_str()",
            "Some(payload.render_label())",
        ],
        "DeadHashmapIterFilterMapMatchItem",
        "dead-hashmap-iter-filter-map-match",
        "pub fn unused_label",
        &[],
    );
}

#[test]
fn prunes_iterator_filter_map_method_ref_option_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "iterator_filter_map_method_ref_option_prune",
        "iterator_filter_map_method_ref_option",
        &[
            ".filter_map(IteratorFilterMapMethodRefOptionPayload::maybe_label)",
            "pub fn maybe_label",
            "pub fn render_label",
        ],
        "DeadIteratorFilterMapMethodRefOptionItem",
        "dead-iterator-filter-map-method-ref-option",
        "pub fn unused_label",
        &[],
    );
}

#[test]
fn prunes_iterator_for_loop_flatten_option_refs_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "iterator_for_loop_flatten_option_refs_prune",
        "iterator_for_loop_flatten_option_refs",
        &[
            "for payload in [&left, &right].into_iter().flatten()",
            "payload.render_label()",
            "pub fn render_label",
        ],
        "DeadIteratorForLoopFlattenOptionRefsItem",
        "dead-iterator-for-loop-flatten-option-refs",
        "pub fn bump_and_render",
        &[],
    );
}

#[test]
fn prunes_iterator_for_loop_flatten_result_vec_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "iterator_for_loop_flatten_result_vec_prune",
        "iterator_for_loop_flatten_result_vec",
        &[
            "for payload in results.into_iter().flatten()",
            "payload.render_label()",
            "pub fn render_label",
        ],
        "DeadIteratorForLoopFlattenResultVecItem",
        "dead-iterator-for-loop-flatten-result-vec",
        "pub fn bump_and_render",
        &["dead_error_method"],
    );
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

#[test]
fn prunes_option_get_or_insert_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "option_get_or_insert_map_prune",
        "option_get_or_insert_map",
        &[
            ".get_or_insert(",
            ".bump_and_render()",
            "pub fn bump_and_render",
        ],
        "DeadOptionGetOrInsertMapItem",
        "dead-option-get-or-insert-map",
        "pub fn render_label",
    );
}

#[test]
fn prunes_option_get_or_insert_default_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "option_get_or_insert_default_map_prune",
        "option_get_or_insert_default_map",
        &[
            ".get_or_insert_default()",
            "impl Default for OptionGetOrInsertDefaultMapPayload",
            ".bump_and_render()",
            "pub fn bump_and_render",
        ],
        "DeadOptionGetOrInsertDefaultMapItem",
        "dead-option-get-or-insert-default-map",
        "pub fn render_label",
    );
}

#[test]
fn prunes_option_unwrap_or_default_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "option_unwrap_or_default_map_prune",
        "option_unwrap_or_default_map",
        &[
            ".unwrap_or_default()",
            "impl Default for OptionUnwrapOrDefaultMapPayload",
            ".render_label()",
            "pub fn render_label",
        ],
        "DeadOptionUnwrapOrDefaultMapItem",
        "dead-option-unwrap-or-default-map",
        "pub fn bump_and_render",
    );
}

#[test]
fn prunes_result_unwrap_or_default_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "result_unwrap_or_default_map_prune",
        "result_unwrap_or_default_map",
        &[
            "Result<ResultUnwrapOrDefaultMapPayload, ResultUnwrapOrDefaultMapError>",
            ".unwrap_or_default()",
            "impl Default for ResultUnwrapOrDefaultMapPayload",
            ".render_label()",
            "pub fn render_label",
        ],
        "DeadResultUnwrapOrDefaultMapItem",
        "dead-result-unwrap-or-default-map",
        "pub fn bump_and_render",
    );
}

#[test]
fn prunes_option_ok_or_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "option_ok_or_map_prune",
        "option_ok_or_map",
        &[
            ".ok_or(",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
            "pub fn render_error",
        ],
        "DeadOptionOkOrMapItem",
        "dead-option-ok-or-map",
        "pub fn bump_and_render",
    );
}

#[test]
fn prunes_option_cloned_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "option_cloned_map_prune",
        "option_cloned_map",
        &[
            ".cloned()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadOptionClonedMapItem",
        "dead-option-cloned-map",
        "pub fn bump_and_render",
    );
}

#[test]
fn prunes_result_cloned_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "result_cloned_map_prune",
        "result_cloned_map",
        &[
            "Result<&ResultClonedMapPayload, ResultClonedMapError>",
            ".cloned()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
            "pub fn render_error",
        ],
        "DeadResultClonedMapItem",
        "dead-result-cloned-map",
        "pub fn bump_and_render",
    );
}

#[test]
fn prunes_result_copied_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "result_copied_map_prune",
        "result_copied_map",
        &[
            "#[derive(Clone, Copy)]",
            "Result<&ResultCopiedMapPayload, ResultCopiedMapError>",
            ".copied()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
            "pub fn render_error",
        ],
        "DeadResultCopiedMapItem",
        "dead-result-copied-map",
        "pub fn bump_and_render",
    );
}

#[test]
fn prunes_option_transpose_unwrap_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "option_transpose_unwrap_map_prune",
        "option_transpose_unwrap_map",
        &[
            "let optional: Option<",
            "Result<OptionTransposeUnwrapMapPayload, OptionTransposeUnwrapMapError>",
            ".transpose()",
            "let _ = err.render_error();",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
            "pub fn render_error",
        ],
        "DeadOptionTransposeUnwrapMapItem",
        "dead-option-transpose-unwrap-map",
        "pub fn bump_and_render",
    );
}

#[test]
fn prunes_result_transpose_unwrap_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "result_transpose_unwrap_map_prune",
        "result_transpose_unwrap_map",
        &[
            "let result: Result<",
            "Option<ResultTransposeUnwrapMapPayload>",
            "ResultTransposeUnwrapMapError",
            ".transpose()",
            "Ok(ResultTransposeUnwrapMapPayload::new(\"fallback\"))",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
            "pub fn render_error",
        ],
        "DeadResultTransposeUnwrapMapItem",
        "dead-result-transpose-unwrap-map",
        "pub fn bump_and_render",
    );
}

#[test]
fn prunes_option_unzip_pair_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "option_unzip_pair_map_prune",
        "option_unzip_pair_map",
        &[
            ".unzip()",
            "let (left, right)",
            ".map(|payload| payload.render_left())",
            ".map(|payload| payload.render_right())",
            "pub fn render_left",
            "pub fn render_right",
        ],
        "DeadOptionUnzipPairMapItem",
        "dead-option-unzip-pair-map",
        "pub fn render_label",
    );
}

#[test]
fn prunes_iterator_unzip_pair_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "iterator_unzip_pair_map_prune",
        "iterator_unzip_pair_map",
        &[
            ".into_iter().unzip()",
            "let (left, right): (Vec<_>, Vec<_>)",
            ".map(|payload| payload.render_left())",
            ".map(|payload| payload.render_right())",
            "pub fn render_left",
            "pub fn render_right",
        ],
        "DeadIteratorUnzipPairMapItem",
        "dead-iterator-unzip-pair-map",
        "pub fn render_label",
    );
}

#[test]
fn prunes_iterator_skip_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "iterator_skip_map_prune",
        "iterator_skip_map",
        &[
            ".skip(1)",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadIteratorSkipMapItem",
        "dead-iterator-skip-map",
        "pub fn bump_and_render",
    );
}

#[test]
fn prunes_iterator_take_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "iterator_take_map_prune",
        "iterator_take_map",
        &[
            ".take(1)",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadIteratorTakeMapItem",
        "dead-iterator-take-map",
        "pub fn bump_and_render",
    );
}

#[test]
fn prunes_iterator_step_by_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "iterator_step_by_map_prune",
        "iterator_step_by_map",
        &[
            ".step_by(2)",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadIteratorStepByMapItem",
        "dead-iterator-step-by-map",
        "pub fn bump_and_render",
    );
}

#[test]
fn prunes_iterator_filter_predicate_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "iterator_filter_predicate_map_prune",
        "iterator_filter_predicate_map",
        &[
            ".filter(|payload| payload.is_live())",
            "pub fn is_live",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadIteratorFilterPredicateMapItem",
        "dead-iterator-filter-predicate-map",
        "pub fn bump_and_render",
    );
}

#[test]
fn prunes_iterator_inspect_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "iterator_inspect_map_prune",
        "iterator_inspect_map",
        &[
            ".inspect(|payload| payload.touch())",
            "pub fn touch",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadIteratorInspectMapItem",
        "dead-iterator-inspect-map",
        "pub fn bump_and_render",
    );
}

#[test]
fn prunes_iterator_by_ref_take_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "iterator_by_ref_take_map_prune",
        "iterator_by_ref_take_map",
        &[
            ".by_ref()",
            ".take(1)",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadIteratorByRefTakeMapItem",
        "dead-iterator-by-ref-take-map",
        "pub fn bump_and_render",
    );
}

#[test]
fn prunes_iterator_copied_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "iterator_copied_map_prune",
        "iterator_copied_map",
        &[
            "#[derive(Clone, Copy)]",
            ".copied()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadIteratorCopiedMapItem",
        "dead-iterator-copied-map",
        "pub fn bump_and_render",
    );
}

#[test]
fn prunes_slice_iter_copied_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "slice_iter_copied_map_prune",
        "slice_iter_copied_map",
        &[
            "#[derive(Clone, Copy)]",
            "let slice = &items[..];",
            ".copied()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadSliceIterCopiedMapItem",
        "dead-slice-iter-copied-map",
        "pub fn bump_and_render",
    );
}

#[test]
fn prunes_result_iter_copied_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "result_iter_copied_map_prune",
        "result_iter_copied_map",
        &[
            "#[derive(Clone, Copy)]",
            "Result<ResultIterCopiedMapPayload, ResultIterCopiedMapError>",
            ".iter()",
            ".copied()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadResultIterCopiedMapItem",
        "dead-result-iter-copied-map",
        "pub fn bump_and_render",
    );
}

#[test]
fn prunes_option_iter_cloned_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "option_iter_cloned_map_prune",
        "option_iter_cloned_map",
        &[
            ".iter()",
            ".cloned()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadOptionIterClonedMapItem",
        "dead-option-iter-cloned-map",
        "pub fn bump_and_render",
    );
}

#[test]
fn prunes_vec_into_iter_take_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "vec_into_iter_take_map_prune",
        "vec_into_iter_take_map",
        &[
            ".into_iter()",
            ".take(1)",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadVecIntoIterTakeMapItem",
        "dead-vec-into-iter-take-map",
        "pub fn bump_and_render",
    );
}

#[test]
fn prunes_array_into_iter_skip_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "array_into_iter_skip_map_prune",
        "array_into_iter_skip_map",
        &[
            ".into_iter()",
            ".skip(1)",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadArrayIntoIterSkipMapItem",
        "dead-array-into-iter-skip-map",
        "pub fn bump_and_render",
    );
}

#[test]
fn prunes_str_split_once_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "str_split_once_map_prune",
        "str_split_once_map",
        &[
            ".split_once('=')",
            ".map(|(_, value)| StrSplitOnceMapPayload::new(value).render_label())",
            "pub fn render_label",
        ],
        "DeadStrSplitOnceMapItem",
        "dead-str-split-once-map",
        "pub fn unused_label",
    );
}

#[test]
fn prunes_str_strip_prefix_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "str_strip_prefix_map_prune",
        "str_strip_prefix_map",
        &[
            ".strip_prefix(\"live:\")",
            ".map(StrStripPrefixMapPayload::new)",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadStrStripPrefixMapItem",
        "dead-str-strip-prefix-map",
        "pub fn unused_label",
    );
}

#[test]
fn prunes_str_lines_filter_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "str_lines_filter_map_prune",
        "str_lines_filter_map",
        &[
            ".lines()",
            ".filter_map(|line|",
            "StrLinesFilterMapPayload::new(line).render_label()",
            "pub fn render_label",
        ],
        "DeadStrLinesFilterMapItem",
        "dead-str-lines-filter-map",
        "pub fn unused_label",
    );
}

#[test]
fn prunes_str_split_whitespace_find_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "str_split_whitespace_find_map_prune",
        "str_split_whitespace_find_map",
        &[
            ".split_whitespace()",
            ".find_map(|part|",
            ".ok()",
            ".map(|payload| payload.render_label())",
            "pub fn try_parse",
            "pub fn render_label",
        ],
        "DeadStrSplitWhitespaceFindMapItem",
        "dead-str-split-whitespace-find-map",
        "pub fn unused_label",
    );
}

#[test]
fn prunes_iterator_map_while_result_ok_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "iterator_map_while_result_ok_prune",
        "iterator_map_while_result_ok",
        &[
            ".map_while(|part| IteratorMapWhileResultOkPayload::try_parse(part).ok())",
            ".map(|payload| payload.render_label())",
            "pub fn try_parse",
            "pub fn render_label",
        ],
        "DeadIteratorMapWhileResultOkItem",
        "dead-iterator-map-while-result-ok",
        "pub fn unused_label",
    );
}

#[test]
fn prunes_iterator_scan_stateful_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "iterator_scan_stateful_map_prune",
        "iterator_scan_stateful_map",
        &[
            ".scan(",
            "0usize",
            "let tagged = format!(\"{count}:{part}\");",
            "IteratorScanStatefulMapPayload::new(&tagged).render_label()",
            "pub fn render_label",
        ],
        "DeadIteratorScanStatefulMapItem",
        "dead-iterator-scan-stateful-map",
        "pub fn unused_label",
    );
}

#[test]
fn prunes_iterator_flatten_option_result_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "iterator_flatten_option_result_prune",
        "iterator_flatten_option_result",
        &[
            "let items = vec![",
            "Some(Ok(IteratorFlattenOptionResultPayload::new(raw)))",
            ".flatten()",
            ".filter_map(Result::ok)",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadIteratorFlattenOptionResultItem",
        "dead-iterator-flatten-option-result",
        "pub fn unused_label",
    );
}

#[test]
fn prunes_option_zip_tuple_render_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "option_zip_tuple_render_prune",
        "option_zip_tuple_render",
        &[
            "left.zip(right)",
            ".map(|(left, right)|",
            "left.render_label()",
            "right.render_label()",
            "pub fn render_label",
        ],
        "DeadOptionZipTupleRenderItem",
        "dead-option-zip-tuple-render",
        "pub fn unused_label",
    );
}

#[test]
fn prunes_slice_windows_filter_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "slice_windows_filter_map_prune",
        "slice_windows_filter_map",
        &[
            ".windows(2)",
            ".filter_map(|window| window.first().map(|payload| payload.render_label()))",
            "pub fn render_label",
        ],
        "DeadSliceWindowsFilterMapItem",
        "dead-slice-windows-filter-map",
        "pub fn unused_label",
    );
}

#[test]
fn prunes_vec_drain_filter_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "vec_drain_filter_map_prune",
        "vec_drain_filter_map",
        &[
            ".drain(..)",
            ".filter_map(VecDrainFilterMapPayload::maybe_label)",
            "pub fn maybe_label",
            "pub fn render_label",
        ],
        "DeadVecDrainFilterMapItem",
        "dead-vec-drain-filter-map",
        "pub fn unused_label",
    );
}

#[test]
fn prunes_hashmap_entry_match_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "hashmap_entry_match_prune",
        "hashmap_entry_match",
        &[
            "use std::collections::HashMap;",
            ".entry(\"live\".to_string())",
            "std::collections::hash_map::Entry::Occupied",
            "std::collections::hash_map::Entry::Vacant",
            "pub fn render_label",
        ],
        "DeadHashmapEntryMatchItem",
        "dead-hashmap-entry-match",
        "pub fn unused_label",
    );
}

#[test]
fn prunes_btreemap_range_find_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "btreemap_range_find_map_prune",
        "btreemap_range_find_map",
        &[
            "use std::collections::BTreeMap;",
            ".range(0..=2)",
            ".find_map(|(_, payload)| payload.maybe_label())",
            "pub fn maybe_label",
            "pub fn render_label",
        ],
        "DeadBtreemapRangeFindMapItem",
        "dead-btreemap-range-find-map",
        "pub fn unused_label",
    );
}

#[test]
fn prunes_str_rsplit_once_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "str_rsplit_once_map_prune",
        "str_rsplit_once_map",
        &[
            ".rsplit_once('/')",
            ".map(|(_, value)| StrRsplitOnceMapPayload::new(value).render_label())",
            "pub fn render_label",
        ],
        "DeadStrRsplitOnceMapItem",
        "dead-str-rsplit-once-map",
        "pub fn unused_label",
    );
}

#[test]
fn prunes_str_splitn_find_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "str_splitn_find_map_prune",
        "str_splitn_find_map",
        &[
            ".splitn(3, ',')",
            ".map(StrSplitnFindMapPayload::new)",
            ".find_map(|payload| payload.maybe_label())",
            "pub fn maybe_label",
            "pub fn render_label",
        ],
        "DeadStrSplitnFindMapItem",
        "dead-str-splitn-find-map",
        "pub fn unused_label",
    );
}

#[test]
fn prunes_str_split_terminator_filter_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "str_split_terminator_filter_map_prune",
        "str_split_terminator_filter_map",
        &[
            ".split_terminator(';')",
            ".filter_map(|part|",
            "StrSplitTerminatorFilterMapPayload::new(part).render_label()",
            "pub fn render_label",
        ],
        "DeadStrSplitTerminatorFilterMapItem",
        "dead-str-split-terminator-filter-map",
        "pub fn unused_label",
    );
}

#[test]
fn prunes_str_char_indices_filter_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "str_char_indices_filter_map_prune",
        "str_char_indices_filter_map",
        &[
            ".char_indices()",
            ".is_alphabetic()",
            "format!(\"{index}:{ch}\")",
            "pub fn render_label",
        ],
        "DeadStrCharIndicesFilterMapItem",
        "dead-str-char-indices-filter-map",
        "pub fn unused_label",
    );
}

#[test]
fn prunes_str_match_indices_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "str_match_indices_map_prune",
        "str_match_indices_map",
        &[
            ".match_indices(\"live\")",
            "format!(\"{index}:{part}\")",
            ".map(|(index, part)|",
            "pub fn render_label",
        ],
        "DeadStrMatchIndicesMapItem",
        "dead-str-match-indices-map",
        "pub fn unused_label",
    );
}

#[test]
fn prunes_path_components_filter_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "path_components_filter_map_prune",
        "path_components_filter_map",
        &[
            "use std::path::{Component, Path};",
            "Path::new(raw)",
            ".components()",
            "Component::Normal(part)",
            "part.to_string_lossy()",
            "pub fn render_label",
        ],
        "DeadPathComponentsFilterMapItem",
        "dead-path-components-filter-map",
        "pub fn unused_label",
    );
}

#[test]
fn prunes_path_file_name_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "path_file_name_map_prune",
        "path_file_name_map",
        &[
            "use std::path::Path;",
            "Path::new(raw)",
            ".file_name()",
            ".and_then(|name| name.to_str())",
            ".map(PathFileNameMapPayload::new)",
            "pub fn render_label",
        ],
        "DeadPathFileNameMapItem",
        "dead-path-file-name-map",
        "pub fn unused_label",
    );
}

#[test]
fn prunes_osstr_to_str_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "osstr_to_str_map_prune",
        "osstr_to_str_map",
        &[
            "use std::ffi::OsStr;",
            "OsStr::new(raw)",
            ".to_str()",
            ".map(OsstrToStrMapPayload::new)",
            "pub fn render_label",
        ],
        "DeadOsstrToStrMapItem",
        "dead-osstr-to-str-map",
        "pub fn unused_label",
    );
}

#[test]
fn prunes_hashmap_get_key_value_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "hashmap_get_key_value_map_prune",
        "hashmap_get_key_value_map",
        &[
            "use std::collections::HashMap;",
            ".get_key_value(\"live\")",
            "format!(\"{key}:{}\", payload.render_label())",
            "pub fn render_label",
        ],
        "DeadHashmapGetKeyValueMapItem",
        "dead-hashmap-get-key-value-map",
        "pub fn unused_label",
    );
}

#[test]
fn prunes_btreemap_range_mut_find_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "btreemap_range_mut_find_map_prune",
        "btreemap_range_mut_find_map",
        &[
            "use std::collections::BTreeMap;",
            ".range_mut(0..=3)",
            ".find_map(|(_, payload)| payload.maybe_label_mut())",
            "pub fn maybe_label_mut",
            "pub fn render_label",
        ],
        "DeadBtreemapRangeMutFindMapItem",
        "dead-btreemap-range-mut-find-map",
        "pub fn unused_label",
    );
}

#[test]
fn prunes_iterator_peekable_peek_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "iterator_peekable_peek_map_prune",
        "iterator_peekable_peek_map",
        &[
            ".peekable()",
            "iter.peek()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadIteratorPeekablePeekMapItem",
        "dead-iterator-peekable-peek-map",
        "pub fn unused_label",
    );
}

#[test]
fn prunes_iter_repeat_with_take_map_support_chain_with_default_analyzer() {
    assert_tuple_adapter_support_fixture(
        "iter_repeat_with_take_map_prune",
        "iter_repeat_with_take_map",
        &[
            "std::iter::repeat_with(|| IterRepeatWithTakeMapPayload::new(raw))",
            ".take(2)",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadIterRepeatWithTakeMapItem",
        "dead-iter-repeat-with-take-map",
        "pub fn unused_label",
    );
}

#[test]
fn prunes_str_chars_filter_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "str_chars_filter_map_prune",
        "str_chars_filter_map",
        &[".chars()", ".filter_map(|ch|", "pub fn render_label"],
        "DeadStrCharsFilterMapItem",
        "dead-str-chars-filter-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_str_bytes_filter_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "str_bytes_filter_map_prune",
        "str_bytes_filter_map",
        &[".bytes()", ".filter_map(|byte|", "pub fn render_label"],
        "DeadStrBytesFilterMapItem",
        "dead-str-bytes-filter-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_str_split_inclusive_filter_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "str_split_inclusive_filter_map_prune",
        "str_split_inclusive_filter_map",
        &[
            ".split_inclusive(';')",
            ".filter_map(|part|",
            "pub fn render_label",
        ],
        "DeadStrSplitInclusiveFilterMapItem",
        "dead-str-split-inclusive-filter-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_str_rsplitn_find_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "str_rsplitn_find_map_prune",
        "str_rsplitn_find_map",
        &[
            ".rsplitn(3, ':')",
            ".find_map(|part|",
            "pub fn render_label",
        ],
        "DeadStrRsplitnFindMapItem",
        "dead-str-rsplitn-find-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_str_trim_matches_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "str_trim_matches_map_prune",
        "str_trim_matches_map",
        &["trim_matches('/')", ".map(|part|", "pub fn render_label"],
        "DeadStrTrimMatchesMapItem",
        "dead-str-trim-matches-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_str_strip_suffix_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "str_strip_suffix_map_prune",
        "str_strip_suffix_map",
        &[
            ".strip_suffix(\".json\")",
            ".map(|part|",
            "pub fn render_label",
        ],
        "DeadStrStripSuffixMapItem",
        "dead-str-strip-suffix-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_path_parent_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "path_parent_map_prune",
        "path_parent_map",
        &[
            "Path::new(raw)",
            ".parent()",
            ".and_then(|path| path.to_str())",
            "pub fn render_label",
        ],
        "DeadPathParentMapItem",
        "dead-path-parent-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_path_ancestors_find_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "path_ancestors_find_map_prune",
        "path_ancestors_find_map",
        &[
            ".ancestors()",
            ".find_map(|path|",
            "file_name()",
            "pub fn render_label",
        ],
        "DeadPathAncestorsFindMapItem",
        "dead-path-ancestors-find-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_path_extension_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "path_extension_map_prune",
        "path_extension_map",
        &[
            ".extension()",
            ".and_then(OsStr::to_str)",
            "pub fn render_label",
        ],
        "DeadPathExtensionMapItem",
        "dead-path-extension-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_path_file_stem_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "path_file_stem_map_prune",
        "path_file_stem_map",
        &[
            ".file_stem()",
            ".and_then(OsStr::to_str)",
            "pub fn render_label",
        ],
        "DeadPathFileStemMapItem",
        "dead-path-file-stem-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_path_strip_prefix_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "path_strip_prefix_map_prune",
        "path_strip_prefix_map",
        &[
            ".strip_prefix(\"/tmp\")",
            ".ok()",
            ".and_then(|path| path.to_str())",
            "pub fn render_label",
        ],
        "DeadPathStripPrefixMapItem",
        "dead-path-strip-prefix-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_hashmap_values_filter_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "hashmap_values_filter_map_prune",
        "hashmap_values_filter_map",
        &[
            "HashMap::new()",
            ".values()",
            ".filter_map(|payload|",
            "pub fn render_label",
        ],
        "DeadHashmapValuesFilterMapItem",
        "dead-hashmap-values-filter-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_hashmap_keys_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "hashmap_keys_map_prune",
        "hashmap_keys_map",
        &[
            "HashMap::new()",
            ".keys()",
            ".map(|key|",
            "pub fn render_label",
        ],
        "DeadHashmapKeysMapItem",
        "dead-hashmap-keys-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_btreemap_values_filter_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "btreemap_values_filter_map_prune",
        "btreemap_values_filter_map",
        &[
            "BTreeMap::new()",
            ".values()",
            ".filter_map(|payload|",
            "pub fn render_label",
        ],
        "DeadBtreemapValuesFilterMapItem",
        "dead-btreemap-values-filter-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_btreemap_keys_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "btreemap_keys_map_prune",
        "btreemap_keys_map",
        &[
            "BTreeMap::new()",
            ".keys()",
            ".map(|key|",
            "pub fn render_label",
        ],
        "DeadBtreemapKeysMapItem",
        "dead-btreemap-keys-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_btreeset_first_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "btreeset_first_map_prune",
        "btreeset_first_map",
        &[
            "BTreeSet::new()",
            ".first()",
            ".map(|value|",
            "pub fn render_label",
        ],
        "DeadBtreesetFirstMapItem",
        "dead-btreeset-first-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_btreeset_last_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "btreeset_last_map_prune",
        "btreeset_last_map",
        &[
            "BTreeSet::new()",
            ".last()",
            ".map(|value|",
            "pub fn render_label",
        ],
        "DeadBtreesetLastMapItem",
        "dead-btreeset-last-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_binaryheap_peek_mut_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "binaryheap_peek_mut_map_prune",
        "binaryheap_peek_mut_map",
        &[
            "BinaryHeap::from",
            ".peek_mut()",
            ".map(|mut value|",
            "pub fn render_label",
        ],
        "DeadBinaryheapPeekMutMapItem",
        "dead-binaryheap-peek-mut-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_vecdeque_range_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vecdeque_range_map_prune",
        "vecdeque_range_map",
        &[
            "VecDeque::from",
            ".range(0..1)",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadVecdequeRangeMapItem",
        "dead-vecdeque-range-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_linkedlist_iter_mut_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "linkedlist_iter_mut_map_prune",
        "linkedlist_iter_mut_map",
        &[
            "LinkedList::new()",
            ".iter_mut()",
            ".map(|payload| payload.bump_and_render())",
            "pub fn bump_and_render",
        ],
        "DeadLinkedlistIterMutMapItem",
        "dead-linkedlist-iter-mut-map",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_str_split_ascii_whitespace_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "str_split_ascii_whitespace_map_prune",
        "str_split_ascii_whitespace_map",
        &[
            ".split_ascii_whitespace()",
            ".map(|part|",
            "pub fn render_label",
        ],
        "DeadStrSplitAsciiWhitespaceMapItem",
        "dead-str-split-ascii-whitespace-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_str_rmatch_indices_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "str_rmatch_indices_map_prune",
        "str_rmatch_indices_map",
        &[
            ".rmatch_indices('/')",
            ".map(|(idx, part)|",
            "pub fn render_label",
        ],
        "DeadStrRmatchIndicesMapItem",
        "dead-str-rmatch-indices-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_str_encode_utf16_filter_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "str_encode_utf16_filter_map_prune",
        "str_encode_utf16_filter_map",
        &[
            ".encode_utf16()",
            ".filter_map(|unit|",
            "pub fn render_label",
        ],
        "DeadStrEncodeUtf16FilterMapItem",
        "dead-str-encode-utf16-filter-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_str_char_escape_default_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "str_char_escape_default_map_prune",
        "str_char_escape_default_map",
        &[
            ".flat_map(char::escape_default)",
            ".map(|ch|",
            "pub fn render_label",
        ],
        "DeadStrCharEscapeDefaultMapItem",
        "dead-str-char-escape-default-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_path_iter_filter_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "path_iter_filter_map_prune",
        "path_iter_filter_map",
        &[
            "Path::new(raw)",
            ".iter()",
            ".filter_map(OsStr::to_str)",
            "pub fn render_label",
        ],
        "DeadPathIterFilterMapItem",
        "dead-path-iter-filter-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_path_with_file_name_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "path_with_file_name_map_prune",
        "path_with_file_name_map",
        &[
            ".with_file_name(\"live.txt\")",
            ".to_str()",
            "pub fn render_label",
        ],
        "DeadPathWithFileNameMapItem",
        "dead-path-with-file-name-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_path_with_extension_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "path_with_extension_map_prune",
        "path_with_extension_map",
        &[
            ".with_extension(\"slice\")",
            ".to_str()",
            "pub fn render_label",
        ],
        "DeadPathWithExtensionMapItem",
        "dead-path-with-extension-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_hashmap_into_keys_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "hashmap_into_keys_map_prune",
        "hashmap_into_keys_map",
        &[
            "HashMap::new()",
            ".into_keys()",
            ".map(|key|",
            "pub fn render_label",
        ],
        "DeadHashmapIntoKeysMapItem",
        "dead-hashmap-into-keys-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_btreemap_into_keys_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "btreemap_into_keys_map_prune",
        "btreemap_into_keys_map",
        &[
            "BTreeMap::new()",
            ".into_keys()",
            ".map(|key|",
            "pub fn render_label",
        ],
        "DeadBtreemapIntoKeysMapItem",
        "dead-btreemap-into-keys-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_vec_first_mut_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vec_first_mut_map_prune",
        "vec_first_mut_map",
        &[
            ".first_mut()",
            ".map(|payload| payload.bump_and_render())",
            "pub fn bump_and_render",
        ],
        "DeadVecFirstMutMapItem",
        "dead-vec-first-mut-map",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_vec_last_mut_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vec_last_mut_map_prune",
        "vec_last_mut_map",
        &[
            ".last_mut()",
            ".map(|payload| payload.bump_and_render())",
            "pub fn bump_and_render",
        ],
        "DeadVecLastMutMapItem",
        "dead-vec-last-mut-map",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_vec_get_mut_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vec_get_mut_map_prune",
        "vec_get_mut_map",
        &[
            ".get_mut(0)",
            ".map(|payload| payload.bump_and_render())",
            "pub fn bump_and_render",
        ],
        "DeadVecGetMutMapItem",
        "dead-vec-get-mut-map",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_vecdeque_get_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vecdeque_get_map_prune",
        "vecdeque_get_map",
        &[
            "VecDeque::from",
            ".get(0)",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadVecdequeGetMapItem",
        "dead-vecdeque-get-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_vecdeque_get_mut_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vecdeque_get_mut_map_prune",
        "vecdeque_get_mut_map",
        &[
            "VecDeque::from",
            ".get_mut(0)",
            ".map(|payload| payload.bump_and_render())",
            "pub fn bump_and_render",
        ],
        "DeadVecdequeGetMutMapItem",
        "dead-vecdeque-get-mut-map",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_vecdeque_remove_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vecdeque_remove_map_prune",
        "vecdeque_remove_map",
        &[
            "VecDeque::from",
            ".remove(0)",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadVecdequeRemoveMapItem",
        "dead-vecdeque-remove-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_linkedlist_front_mut_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "linkedlist_front_mut_map_prune",
        "linkedlist_front_mut_map",
        &[
            "LinkedList::new()",
            ".front_mut()",
            ".map(|payload| payload.bump_and_render())",
            "pub fn bump_and_render",
        ],
        "DeadLinkedlistFrontMutMapItem",
        "dead-linkedlist-front-mut-map",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_linkedlist_back_mut_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "linkedlist_back_mut_map_prune",
        "linkedlist_back_mut_map",
        &[
            "LinkedList::new()",
            ".back_mut()",
            ".map(|payload| payload.bump_and_render())",
            "pub fn bump_and_render",
        ],
        "DeadLinkedlistBackMutMapItem",
        "dead-linkedlist-back-mut-map",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_btreemap_first_entry_mut_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "btreemap_first_entry_mut_map_prune",
        "btreemap_first_entry_mut_map",
        &[
            "BTreeMap::new()",
            ".first_entry()",
            ".map(|mut entry| entry.get_mut().bump_and_render())",
            "pub fn bump_and_render",
        ],
        "DeadBtreemapFirstEntryMutMapItem",
        "dead-btreemap-first-entry-mut-map",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_btreemap_last_entry_mut_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "btreemap_last_entry_mut_map_prune",
        "btreemap_last_entry_mut_map",
        &[
            "BTreeMap::new()",
            ".last_entry()",
            ".map(|mut entry| entry.get_mut().bump_and_render())",
            "pub fn bump_and_render",
        ],
        "DeadBtreemapLastEntryMutMapItem",
        "dead-btreemap-last-entry-mut-map",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_binaryheap_push_peek_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "binaryheap_push_peek_map_prune",
        "binaryheap_push_peek_map",
        &[
            "BinaryHeap::new()",
            ".push(raw.to_string())",
            ".peek()",
            "pub fn render_label",
        ],
        "DeadBinaryheapPushPeekMapItem",
        "dead-binaryheap-push-peek-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_hashmap_remove_entry_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "hashmap_remove_entry_map_prune",
        "hashmap_remove_entry_map",
        &[
            "HashMap::new()",
            ".remove_entry(raw)",
            ".map(|(_, payload)| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadHashmapRemoveEntryMapItem",
        "dead-hashmap-remove-entry-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_btreemap_remove_entry_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "btreemap_remove_entry_map_prune",
        "btreemap_remove_entry_map",
        &[
            "BTreeMap::new()",
            ".remove_entry(raw)",
            ".map(|(_, payload)| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadBtreemapRemoveEntryMapItem",
        "dead-btreemap-remove-entry-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_hashmap_entry_remove_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "hashmap_entry_remove_map_prune",
        "hashmap_entry_remove_map",
        &[
            "hash_map::Entry",
            "Entry::Occupied(entry)",
            "entry.remove_entry()",
            "Entry::Vacant(entry)",
            "pub fn render_label",
        ],
        "DeadHashmapEntryRemoveMapItem",
        "dead-hashmap-entry-remove-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_btreemap_entry_remove_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "btreemap_entry_remove_map_prune",
        "btreemap_entry_remove_map",
        &[
            "btree_map::Entry",
            "Entry::Occupied(entry)",
            "entry.remove_entry()",
            "Entry::Vacant(entry)",
            "pub fn render_label",
        ],
        "DeadBtreemapEntryRemoveMapItem",
        "dead-btreemap-entry-remove-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_btreeset_range_rev_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "btreeset_range_rev_map_prune",
        "btreeset_range_rev_map",
        &[
            "BTreeSet::new()",
            ".range(..=raw.to_string())",
            ".rev()",
            "pub fn render_label",
        ],
        "DeadBtreesetRangeRevMapItem",
        "dead-btreeset-range-rev-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_btreeset_iter_next_back_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "btreeset_iter_next_back_map_prune",
        "btreeset_iter_next_back_map",
        &[
            "BTreeSet::new()",
            ".iter()",
            ".next_back()",
            "pub fn render_label",
        ],
        "DeadBtreesetIterNextBackMapItem",
        "dead-btreeset-iter-next-back-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_vecdeque_rotate_left_front_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vecdeque_rotate_left_front_map_prune",
        "vecdeque_rotate_left_front_map",
        &[
            "VecDeque::from",
            ".rotate_left(1)",
            ".front()",
            "pub fn render_label",
        ],
        "DeadVecdequeRotateLeftFrontMapItem",
        "dead-vecdeque-rotate-left-front-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_vecdeque_rotate_right_back_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vecdeque_rotate_right_back_map_prune",
        "vecdeque_rotate_right_back_map",
        &[
            "VecDeque::from",
            ".rotate_right(1)",
            ".back()",
            "pub fn render_label",
        ],
        "DeadVecdequeRotateRightBackMapItem",
        "dead-vecdeque-rotate-right-back-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_vecdeque_swap_remove_front_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vecdeque_swap_remove_front_map_prune",
        "vecdeque_swap_remove_front_map",
        &[
            "VecDeque::from",
            ".swap_remove_front(0)",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadVecdequeSwapRemoveFrontMapItem",
        "dead-vecdeque-swap-remove-front-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_vecdeque_swap_remove_back_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vecdeque_swap_remove_back_map_prune",
        "vecdeque_swap_remove_back_map",
        &[
            "VecDeque::from",
            ".swap_remove_back(1)",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadVecdequeSwapRemoveBackMapItem",
        "dead-vecdeque-swap-remove-back-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_slice_chunks_mut_filter_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "slice_chunks_mut_filter_map_prune",
        "slice_chunks_mut_filter_map",
        &[
            ".chunks_mut(1)",
            ".filter_map(|chunk| chunk.first_mut())",
            ".map(|payload| payload.bump_and_render())",
            "pub fn bump_and_render",
        ],
        "DeadSliceChunksMutFilterMapItem",
        "dead-slice-chunks-mut-filter-map",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_slice_rchunks_mut_filter_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "slice_rchunks_mut_filter_map_prune",
        "slice_rchunks_mut_filter_map",
        &[
            ".rchunks_mut(1)",
            ".filter_map(|chunk| chunk.first_mut())",
            ".map(|payload| payload.bump_and_render())",
            "pub fn bump_and_render",
        ],
        "DeadSliceRchunksMutFilterMapItem",
        "dead-slice-rchunks-mut-filter-map",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_slice_split_mut_filter_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "slice_split_mut_filter_map_prune",
        "slice_split_mut_filter_map",
        &[
            ".split_mut(|payload| payload.value == \"skip\")",
            ".filter_map(|chunk| chunk.first_mut())",
            ".map(|payload| payload.bump_and_render())",
            "pub fn bump_and_render",
        ],
        "DeadSliceSplitMutFilterMapItem",
        "dead-slice-split-mut-filter-map",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_slice_splitn_mut_filter_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "slice_splitn_mut_filter_map_prune",
        "slice_splitn_mut_filter_map",
        &[
            ".splitn_mut(2, |payload| payload.value == \"skip\")",
            ".filter_map(|chunk| chunk.first_mut())",
            ".map(|payload| payload.bump_and_render())",
            "pub fn bump_and_render",
        ],
        "DeadSliceSplitnMutFilterMapItem",
        "dead-slice-splitn-mut-filter-map",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_str_lines_rev_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "str_lines_rev_map_prune",
        "str_lines_rev_map",
        &[
            "raw.lines()",
            ".rev()",
            ".map(StrLinesRevMapPayload::new)",
            "pub fn render_label",
        ],
        "DeadStrLinesRevMapItem",
        "dead-str-lines-rev-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_str_matches_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "str_matches_map_prune",
        "str_matches_map",
        &[
            "raw.matches('a')",
            ".map(StrMatchesMapPayload::new)",
            "pub fn render_label",
        ],
        "DeadStrMatchesMapItem",
        "dead-str-matches-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_str_split_rev_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "str_split_rev_map_prune",
        "str_split_rev_map",
        &[
            "raw.split(':')",
            ".rev()",
            ".map(StrSplitRevMapPayload::new)",
            "pub fn render_label",
        ],
        "DeadStrSplitRevMapItem",
        "dead-str-split-rev-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_pathbuf_push_to_str_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "pathbuf_push_to_str_map_prune",
        "pathbuf_push_to_str_map",
        &[
            "PathBuf::from(raw)",
            "path.push(\"tail\")",
            "path.to_str()",
            "pub fn render_label",
        ],
        "DeadPathbufPushToStrMapItem",
        "dead-pathbuf-push-to-str-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_pathbuf_set_extension_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "pathbuf_set_extension_map_prune",
        "pathbuf_set_extension_map",
        &[
            "PathBuf::from(raw)",
            "path.set_extension(\"log\")",
            "path.to_str()",
            "pub fn render_label",
        ],
        "DeadPathbufSetExtensionMapItem",
        "dead-pathbuf-set-extension-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_osstring_into_string_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "osstring_into_string_map_prune",
        "osstring_into_string_map",
        &[
            "OsString::from(raw)",
            ".into_string()",
            ".ok()",
            "pub fn render_label",
        ],
        "DeadOsstringIntoStringMapItem",
        "dead-osstring-into-string-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_string_drain_collect_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "string_drain_collect_map_prune",
        "string_drain_collect_map",
        &[
            "value.drain(..)",
            ".collect::<String>()",
            "pub fn render_label",
        ],
        "DeadStringDrainCollectMapItem",
        "dead-string-drain-collect-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_string_pop_char_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "string_pop_char_map_prune",
        "string_pop_char_map",
        &[
            ".pop()",
            "StringPopCharMapPayload::new(&ch.to_string())",
            "pub fn render_label",
        ],
        "DeadStringPopCharMapItem",
        "dead-string-pop-char-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_string_remove_char_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "string_remove_char_map_prune",
        "string_remove_char_map",
        &[
            "value.remove(0)",
            "StringRemoveCharMapPayload::new(&ch.to_string())",
            "pub fn render_label",
        ],
        "DeadStringRemoveCharMapItem",
        "dead-string-remove-char-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_string_replace_range_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "string_replace_range_map_prune",
        "string_replace_range_map",
        &[
            "value.replace_range(..0, \"head-\")",
            "StringReplaceRangeMapPayload::new(&value)",
            "pub fn render_label",
        ],
        "DeadStringReplaceRangeMapItem",
        "dead-string-replace-range-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_str_escape_debug_flat_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "str_escape_debug_flat_map_prune",
        "str_escape_debug_flat_map",
        &[
            ".flat_map(char::escape_debug)",
            "StrEscapeDebugFlatMapPayload::new(&ch.to_string())",
            "pub fn render_label",
        ],
        "DeadStrEscapeDebugFlatMapItem",
        "dead-str-escape-debug-flat-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_str_bytes_enumerate_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "str_bytes_enumerate_map_prune",
        "str_bytes_enumerate_map",
        &[
            "raw.bytes()",
            ".enumerate()",
            "StrBytesEnumerateMapPayload::new(&format!",
            "pub fn render_label",
        ],
        "DeadStrBytesEnumerateMapItem",
        "dead-str-bytes-enumerate-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_string_from_utf8_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "string_from_utf8_map_prune",
        "string_from_utf8_map",
        &[
            "String::from_utf8(raw.as_bytes().to_vec())",
            ".ok()",
            "StringFromUtf8MapPayload::new(&value)",
            "pub fn render_label",
        ],
        "DeadStringFromUtf8MapItem",
        "dead-string-from-utf8-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_string_from_utf16_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "string_from_utf16_map_prune",
        "string_from_utf16_map",
        &[
            "raw.encode_utf16().collect::<Vec<_>>()",
            "String::from_utf16(&units)",
            "StringFromUtf16MapPayload::new(&value)",
            "pub fn render_label",
        ],
        "DeadStringFromUtf16MapItem",
        "dead-string-from-utf16-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_cstring_into_string_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "cstring_into_string_map_prune",
        "cstring_into_string_map",
        &[
            "CString::new(raw)",
            ".and_then(|value| value.into_string().ok())",
            "CstringIntoStringMapPayload::new(&value)",
            "pub fn render_label",
        ],
        "DeadCstringIntoStringMapItem",
        "dead-cstring-into-string-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_osstring_push_into_string_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "osstring_push_into_string_map_prune",
        "osstring_push_into_string_map",
        &[
            "OsString::from(raw)",
            "value.push(\"tail\")",
            ".into_string()",
            "pub fn render_label",
        ],
        "DeadOsstringPushIntoStringMapItem",
        "dead-osstring-push-into-string-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_pathbuf_pop_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "pathbuf_pop_map_prune",
        "pathbuf_pop_map",
        &[
            "PathBuf::from(raw)",
            "path.push(\"tail\")",
            "path.pop()",
            "PathbufPopMapPayload::new",
            "pub fn render_label",
        ],
        "DeadPathbufPopMapItem",
        "dead-pathbuf-pop-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_pathbuf_set_file_name_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "pathbuf_set_file_name_map_prune",
        "pathbuf_set_file_name_map",
        &[
            "PathBuf::from(raw)",
            "path.set_file_name(\"renamed.txt\")",
            "path.to_str()",
            "pub fn render_label",
        ],
        "DeadPathbufSetFileNameMapItem",
        "dead-pathbuf-set-file-name-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_vecdeque_front_mut_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vecdeque_front_mut_map_prune",
        "vecdeque_front_mut_map",
        &[
            "VecDeque::from",
            ".front_mut()",
            ".map(|payload| payload.bump_and_render())",
            "pub fn bump_and_render",
        ],
        "DeadVecdequeFrontMutMapItem",
        "dead-vecdeque-front-mut-map",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_vecdeque_back_mut_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vecdeque_back_mut_map_prune",
        "vecdeque_back_mut_map",
        &[
            "VecDeque::from",
            ".back_mut()",
            ".map(|payload| payload.bump_and_render())",
            "pub fn bump_and_render",
        ],
        "DeadVecdequeBackMutMapItem",
        "dead-vecdeque-back-mut-map",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_vecdeque_swap_front_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vecdeque_swap_front_map_prune",
        "vecdeque_swap_front_map",
        &[
            "VecDeque::from",
            "values.swap(0, 1)",
            ".front()",
            "pub fn render_label",
        ],
        "DeadVecdequeSwapFrontMapItem",
        "dead-vecdeque-swap-front-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_vec_insert_get_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vec_insert_get_map_prune",
        "vec_insert_get_map",
        &[
            "values.insert(0, VecInsertGetMapPayload::new(raw))",
            ".get(0)",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadVecInsertGetMapItem",
        "dead-vec-insert-get-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_vec_resize_with_pop_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vec_resize_with_pop_map_prune",
        "vec_resize_with_pop_map",
        &[
            "values.resize_with(2, || VecResizeWithPopMapPayload::new(raw))",
            ".pop()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadVecResizeWithPopMapItem",
        "dead-vec-resize-with-pop-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_hashmap_drain_filter_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "hashmap_drain_filter_map_prune",
        "hashmap_drain_filter_map",
        &[
            "HashMap::new()",
            ".drain()",
            ".filter_map(|(_, payload)| Some(payload.render_label()))",
            "pub fn render_label",
        ],
        "DeadHashmapDrainFilterMapItem",
        "dead-hashmap-drain-filter-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_btreemap_split_off_keys_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "btreemap_split_off_keys_map_prune",
        "btreemap_split_off_keys_map",
        &[
            "BTreeMap::new()",
            "values.split_off(raw)",
            ".keys()",
            "BtreemapSplitOffKeysMapPayload::new(key)",
            "pub fn render_label",
        ],
        "DeadBtreemapSplitOffKeysMapItem",
        "dead-btreemap-split-off-keys-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_binaryheap_from_iter_peek_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "binaryheap_from_iter_peek_map_prune",
        "binaryheap_from_iter_peek_map",
        &[
            "BinaryHeap::from([raw.to_string()])",
            ".peek()",
            "BinaryheapFromIterPeekMapPayload::new(value)",
            "pub fn render_label",
        ],
        "DeadBinaryheapFromIterPeekMapItem",
        "dead-binaryheap-from-iter-peek-map",
        "pub fn unused_label",
        &["pub fn bump_and_render"],
    );
}

#[test]
fn prunes_vec_dedup_by_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vec_dedup_by_map_prune",
        "vec_dedup_by_map",
        &[
            "values.dedup_by(",
            ".iter()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadVecDedupByMapItem",
        "dead-vec-dedup-by-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_vec_sort_by_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vec_sort_by_map_prune",
        "vec_sort_by_map",
        &[
            "values.sort_by(",
            ".iter()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadVecSortByMapItem",
        "dead-vec-sort-by-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_vec_select_nth_unstable_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vec_select_nth_unstable_map_prune",
        "vec_select_nth_unstable_map",
        &[
            ".select_nth_unstable_by(",
            "payload.render_label()",
            "pub fn render_label",
        ],
        "DeadVecSelectNthUnstableMapItem",
        "dead-vec-select-nth-unstable-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_vec_split_at_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vec_split_at_map_prune",
        "vec_split_at_map",
        &[
            ".split_at(1)",
            "head.iter()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadVecSplitAtMapItem",
        "dead-vec-split-at-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_vec_split_at_mut_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vec_split_at_mut_map_prune",
        "vec_split_at_mut_map",
        &[
            ".split_at_mut(1)",
            "head.iter_mut()",
            ".map(|payload| payload.bump_and_render())",
            "pub fn bump_and_render",
        ],
        "DeadVecSplitAtMutMapItem",
        "dead-vec-split-at-mut-map",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_vec_chunks_exact_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vec_chunks_exact_map_prune",
        "vec_chunks_exact_map",
        &[
            ".chunks_exact(1)",
            ".filter_map(|chunk| chunk.first())",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadVecChunksExactMapItem",
        "dead-vec-chunks-exact-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_vec_windows_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vec_windows_map_prune",
        "vec_windows_map",
        &[
            ".windows(1)",
            ".filter_map(|chunk| chunk.first())",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadVecWindowsMapItem",
        "dead-vec-windows-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_vec_retain_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vec_retain_map_prune",
        "vec_retain_map",
        &[
            "values.retain(",
            ".iter()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadVecRetainMapItem",
        "dead-vec-retain-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_string_split_off_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "string_split_off_map_prune",
        "string_split_off_map",
        &[
            "value.split_off(5)",
            "StringSplitOffMapPayload::new(&tail)",
            "pub fn render_label",
        ],
        "DeadStringSplitOffMapItem",
        "dead-string-split-off-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_string_truncate_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "string_truncate_map_prune",
        "string_truncate_map",
        &[
            "value.truncate(raw.len())",
            "StringTruncateMapPayload::new(&value)",
            "pub fn render_label",
        ],
        "DeadStringTruncateMapItem",
        "dead-string-truncate-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_string_clear_push_str_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "string_clear_push_str_map_prune",
        "string_clear_push_str_map",
        &[
            "value.clear()",
            "value.push_str(raw)",
            "StringClearPushStrMapPayload::new(&value)",
            "pub fn render_label",
        ],
        "DeadStringClearPushStrMapItem",
        "dead-string-clear-push-str-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_string_retain_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "string_retain_map_prune",
        "string_retain_map",
        &[
            "value.retain(",
            "StringRetainMapPayload::new(&value)",
            "pub fn render_label",
        ],
        "DeadStringRetainMapItem",
        "dead-string-retain-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_str_trim_start_matches_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "str_trim_start_matches_map_prune",
        "str_trim_start_matches_map",
        &[
            "raw.trim_start_matches('a')",
            "StrTrimStartMatchesMapPayload::new(segment)",
            "pub fn render_label",
        ],
        "DeadStrTrimStartMatchesMapItem",
        "dead-str-trim-start-matches-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_str_trim_end_matches_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "str_trim_end_matches_map_prune",
        "str_trim_end_matches_map",
        &[
            "raw.trim_end_matches('z')",
            "StrTrimEndMatchesMapPayload::new(segment)",
            "pub fn render_label",
        ],
        "DeadStrTrimEndMatchesMapItem",
        "dead-str-trim-end-matches-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_str_split_inclusive_rev_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "str_split_inclusive_rev_map_prune",
        "str_split_inclusive_rev_map",
        &[
            "raw.split_inclusive(':')",
            ".rev()",
            ".map(StrSplitInclusiveRevMapPayload::new)",
            "pub fn render_label",
        ],
        "DeadStrSplitInclusiveRevMapItem",
        "dead-str-split-inclusive-rev-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_str_rsplit_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "str_rsplit_map_prune",
        "str_rsplit_map",
        &[
            "raw.rsplit(':')",
            ".map(StrRsplitMapPayload::new)",
            "pub fn render_label",
        ],
        "DeadStrRsplitMapItem",
        "dead-str-rsplit-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_pathbuf_join_file_name_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "pathbuf_join_file_name_map_prune",
        "pathbuf_join_file_name_map",
        &[
            "PathBuf::from(raw).join(\"tail.txt\")",
            "path.file_name()",
            ".and_then(|name| name.to_str())",
            "pub fn render_label",
        ],
        "DeadPathbufJoinFileNameMapItem",
        "dead-pathbuf-join-file-name-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_pathbuf_with_extension_to_str_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "pathbuf_with_extension_to_str_map_prune",
        "pathbuf_with_extension_to_str_map",
        &[
            "PathBuf::from(raw).with_extension(\"log\")",
            "path.to_str()",
            "pub fn render_label",
        ],
        "DeadPathbufWithExtensionToStrMapItem",
        "dead-pathbuf-with-extension-to-str-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_hashmap_extend_values_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "hashmap_extend_values_map_prune",
        "hashmap_extend_values_map",
        &[
            "HashMap::new()",
            "values.extend(",
            ".values()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadHashmapExtendValuesMapItem",
        "dead-hashmap-extend-values-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_vecdeque_truncate_front_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vecdeque_truncate_front_map_prune",
        "vecdeque_truncate_front_map",
        &[
            "VecDeque::from",
            "values.truncate(1)",
            ".front()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadVecdequeTruncateFrontMapItem",
        "dead-vecdeque-truncate-front-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_vec_swap_get_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vec_swap_get_map_prune",
        "vec_swap_get_map",
        &[
            "values.swap(0, 1)",
            ".get(0)",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadVecSwapGetMapItem",
        "dead-vec-swap-get-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_vec_clear_extend_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vec_clear_extend_map_prune",
        "vec_clear_extend_map",
        &[
            "values.clear()",
            "values.extend([",
            ".iter()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadVecClearExtendMapItem",
        "dead-vec-clear-extend-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_vec_extend_from_within_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vec_extend_from_within_map_prune",
        "vec_extend_from_within_map",
        &[
            "#[derive(Clone)]",
            "values.extend_from_within(0..1)",
            ".iter()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadVecExtendFromWithinMapItem",
        "dead-vec-extend-from-within-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_vec_resize_clone_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vec_resize_clone_map_prune",
        "vec_resize_clone_map",
        &[
            "#[derive(Clone)]",
            "values.resize(2, VecResizeCloneMapPayload::new(raw))",
            ".last()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadVecResizeCloneMapItem",
        "dead-vec-resize-clone-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_slice_fill_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "slice_fill_map_prune",
        "slice_fill_map",
        &[
            "#[derive(Clone)]",
            ".as_mut_slice().fill(",
            ".iter()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadSliceFillMapItem",
        "dead-slice-fill-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_slice_fill_with_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "slice_fill_with_map_prune",
        "slice_fill_with_map",
        &[
            ".as_mut_slice()",
            ".fill_with(|| SliceFillWithMapPayload::new(raw))",
            ".iter()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadSliceFillWithMapItem",
        "dead-slice-fill-with-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_string_push_char_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "string_push_char_map_prune",
        "string_push_char_map",
        &[
            "String::new()",
            "value.push('x')",
            "value.push_str(raw)",
            "StringPushCharMapPayload::new(&value)",
            "pub fn render_label",
        ],
        "DeadStringPushCharMapItem",
        "dead-string-push-char-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_string_push_str_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "string_push_str_map_prune",
        "string_push_str_map",
        &[
            "String::from(raw)",
            "value.push_str(\"-tail\")",
            "StringPushStrMapPayload::new(&value)",
            "pub fn render_label",
        ],
        "DeadStringPushStrMapItem",
        "dead-string-push-str-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_string_insert_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "string_insert_map_prune",
        "string_insert_map",
        &[
            "raw.to_string()",
            "value.insert(0, 'x')",
            "StringInsertMapPayload::new(&value)",
            "pub fn render_label",
        ],
        "DeadStringInsertMapItem",
        "dead-string-insert-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_string_insert_str_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "string_insert_str_map_prune",
        "string_insert_str_map",
        &[
            "raw.to_string()",
            "value.insert_str(0, \"head-\")",
            "StringInsertStrMapPayload::new(&value)",
            "pub fn render_label",
        ],
        "DeadStringInsertStrMapItem",
        "dead-string-insert-str-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_string_extend_chars_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "string_extend_chars_map_prune",
        "string_extend_chars_map",
        &[
            "String::from(\"head-\")",
            "value.extend(raw.chars())",
            "StringExtendCharsMapPayload::new(&value)",
            "pub fn render_label",
        ],
        "DeadStringExtendCharsMapItem",
        "dead-string-extend-chars-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_str_replace_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "str_replace_map_prune",
        "str_replace_map",
        &[
            "raw.replace('a', \"b\")",
            "StrReplaceMapPayload::new(&value)",
            "pub fn render_label",
        ],
        "DeadStrReplaceMapItem",
        "dead-str-replace-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_str_replacen_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "str_replacen_map_prune",
        "str_replacen_map",
        &[
            "raw.replacen('a', \"b\", 1)",
            "StrReplacenMapPayload::new(&value)",
            "pub fn render_label",
        ],
        "DeadStrReplacenMapItem",
        "dead-str-replacen-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_str_to_lowercase_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "str_to_lowercase_map_prune",
        "str_to_lowercase_map",
        &[
            "raw.to_lowercase()",
            "StrToLowercaseMapPayload::new(&value)",
            "pub fn render_label",
        ],
        "DeadStrToLowercaseMapItem",
        "dead-str-to-lowercase-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_str_to_uppercase_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "str_to_uppercase_map_prune",
        "str_to_uppercase_map",
        &[
            "raw.to_uppercase()",
            "StrToUppercaseMapPayload::new(&value)",
            "pub fn render_label",
        ],
        "DeadStrToUppercaseMapItem",
        "dead-str-to-uppercase-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_pathbuf_clear_push_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "pathbuf_clear_push_map_prune",
        "pathbuf_clear_push_map",
        &[
            "PathBuf::from(\"dead\")",
            "path.clear()",
            "path.push(raw)",
            "path.to_str()",
            "pub fn render_label",
        ],
        "DeadPathbufClearPushMapItem",
        "dead-pathbuf-clear-push-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_pathbuf_reserve_push_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "pathbuf_reserve_push_map_prune",
        "pathbuf_reserve_push_map",
        &[
            "PathBuf::with_capacity(raw.len() + 4)",
            "path.reserve(4)",
            "path.push(raw)",
            "path.to_str()",
            "pub fn render_label",
        ],
        "DeadPathbufReservePushMapItem",
        "dead-pathbuf-reserve-push-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_hashmap_reserve_insert_values_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "hashmap_reserve_insert_values_map_prune",
        "hashmap_reserve_insert_values_map",
        &[
            "HashMap::new()",
            "values.reserve(1)",
            "values.insert(",
            ".values()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadHashmapReserveInsertValuesMapItem",
        "dead-hashmap-reserve-insert-values-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_btreemap_append_keys_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "btreemap_append_keys_map_prune",
        "btreemap_append_keys_map",
        &[
            "BTreeMap::new()",
            "values.append(&mut extras)",
            ".keys()",
            "BtreemapAppendKeysMapPayload::new(key).render_label()",
            "pub fn render_label",
        ],
        "DeadBtreemapAppendKeysMapItem",
        "dead-btreemap-append-keys-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_binaryheap_append_peek_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "binaryheap_append_peek_map_prune",
        "binaryheap_append_peek_map",
        &[
            "BinaryHeap::from([raw.to_string()])",
            "values.append(&mut extras)",
            ".peek()",
            "BinaryheapAppendPeekMapPayload::new(value).render_label()",
            "pub fn render_label",
        ],
        "DeadBinaryheapAppendPeekMapItem",
        "dead-binaryheap-append-peek-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_vec_reserve_push_get_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vec_reserve_push_get_map_prune",
        "vec_reserve_push_get_map",
        &[
            "Vec::with_capacity(1)",
            "values.reserve(1)",
            "values.push(",
            ".get(0)",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadVecReservePushGetMapItem",
        "dead-vec-reserve-push-get-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_vec_shrink_to_fit_last_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vec_shrink_to_fit_last_map_prune",
        "vec_shrink_to_fit_last_map",
        &[
            "vec![VecShrinkToFitLastMapPayload::new(raw)]",
            "values.shrink_to_fit()",
            ".last()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadVecShrinkToFitLastMapItem",
        "dead-vec-shrink-to-fit-last-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_vec_truncate_get_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vec_truncate_get_map_prune",
        "vec_truncate_get_map",
        &[
            "values.truncate(1)",
            ".get(0)",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadVecTruncateGetMapItem",
        "dead-vec-truncate-get-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_vec_split_off_last_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vec_split_off_last_map_prune",
        "vec_split_off_last_map",
        &[
            "values.split_off(1)",
            "tail.last()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadVecSplitOffLastMapItem",
        "dead-vec-split-off-last-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_slice_sort_by_key_first_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "slice_sort_by_key_first_map_prune",
        "slice_sort_by_key_first_map",
        &[
            ".as_mut_slice().sort_by_key(|payload| payload.rank())",
            "pub fn rank",
            ".first()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadSliceSortByKeyFirstMapItem",
        "dead-slice-sort-by-key-first-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_slice_rotate_left_first_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "slice_rotate_left_first_map_prune",
        "slice_rotate_left_first_map",
        &[
            "values.rotate_left(1)",
            ".first()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadSliceRotateLeftFirstMapItem",
        "dead-slice-rotate-left-first-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_slice_rotate_right_last_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "slice_rotate_right_last_map_prune",
        "slice_rotate_right_last_map",
        &[
            "values.rotate_right(1)",
            ".last()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadSliceRotateRightLastMapItem",
        "dead-slice-rotate-right-last-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_string_reserve_push_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "string_reserve_push_map_prune",
        "string_reserve_push_map",
        &[
            "String::with_capacity(raw.len() + 4)",
            "value.reserve(4)",
            "value.push_str(raw)",
            "value.push('!')",
            "StringReservePushMapPayload::new(&value)",
            "pub fn render_label",
        ],
        "DeadStringReservePushMapItem",
        "dead-string-reserve-push-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_string_shrink_to_fit_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "string_shrink_to_fit_map_prune",
        "string_shrink_to_fit_map",
        &[
            "raw.to_string()",
            "value.push_str(\"-tail\")",
            "value.shrink_to_fit()",
            "StringShrinkToFitMapPayload::new(&value)",
            "pub fn render_label",
        ],
        "DeadStringShrinkToFitMapItem",
        "dead-string-shrink-to-fit-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_str_as_bytes_first_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "str_as_bytes_first_map_prune",
        "str_as_bytes_first_map",
        &[
            "raw.as_bytes()",
            ".first()",
            "StrAsBytesFirstMapPayload::new(&byte.to_string()).render_label()",
            "pub fn render_label",
        ],
        "DeadStrAsBytesFirstMapItem",
        "dead-str-as-bytes-first-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_str_chars_next_back_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "str_chars_next_back_map_prune",
        "str_chars_next_back_map",
        &[
            "raw.chars()",
            ".next_back()",
            "StrCharsNextBackMapPayload::new(&ch.to_string()).render_label()",
            "pub fn render_label",
        ],
        "DeadStrCharsNextBackMapItem",
        "dead-str-chars-next-back-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_pathbuf_push_pop_push_to_str_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "pathbuf_push_pop_push_to_str_map_prune",
        "pathbuf_push_pop_push_to_str_map",
        &[
            "PathBuf::from(raw)",
            "path.push(\"child\")",
            "path.pop()",
            "path.push(\"live\")",
            "path.to_str()",
            "pub fn render_label",
        ],
        "DeadPathbufPushPopPushToStrMapItem",
        "dead-pathbuf-push-pop-push-to-str-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_osstr_to_string_lossy_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "osstr_to_string_lossy_map_prune",
        "osstr_to_string_lossy_map",
        &[
            "OsStr::new(raw).to_string_lossy()",
            "OsstrToStringLossyMapPayload::new(&value)",
            "pub fn render_label",
        ],
        "DeadOsstrToStringLossyMapItem",
        "dead-osstr-to-string-lossy-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_cstr_to_str_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "cstr_to_str_map_prune",
        "cstr_to_str_map",
        &[
            "CString::new(raw).unwrap_or_else",
            ".as_c_str()",
            ".to_str()",
            ".ok()",
            "CstrToStrMapPayload::new(text).render_label()",
            "pub fn render_label",
        ],
        "DeadCstrToStrMapItem",
        "dead-cstr-to-str-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_hashmap_shrink_to_fit_values_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "hashmap_shrink_to_fit_values_map_prune",
        "hashmap_shrink_to_fit_values_map",
        &[
            "HashMap::new()",
            "values.shrink_to_fit()",
            ".values()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadHashmapShrinkToFitValuesMapItem",
        "dead-hashmap-shrink-to-fit-values-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_hashmap_keys_cloned_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "hashmap_keys_cloned_map_prune",
        "hashmap_keys_cloned_map",
        &[
            "HashMap::new()",
            ".keys()",
            ".cloned()",
            "HashmapKeysClonedMapPayload::new(&key).render_label()",
            "pub fn render_label",
        ],
        "DeadHashmapKeysClonedMapItem",
        "dead-hashmap-keys-cloned-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_hashset_reserve_insert_iter_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "hashset_reserve_insert_iter_map_prune",
        "hashset_reserve_insert_iter_map",
        &[
            "HashSet::new()",
            "values.reserve(1)",
            "values.insert(raw.to_string())",
            ".iter()",
            "HashsetReserveInsertIterMapPayload::new(value).render_label()",
            "pub fn render_label",
        ],
        "DeadHashsetReserveInsertIterMapItem",
        "dead-hashset-reserve-insert-iter-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_hashset_extend_iter_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "hashset_extend_iter_map_prune",
        "hashset_extend_iter_map",
        &[
            "HashSet::new()",
            "values.extend([raw.to_string()])",
            ".iter()",
            "HashsetExtendIterMapPayload::new(value).render_label()",
            "pub fn render_label",
        ],
        "DeadHashsetExtendIterMapItem",
        "dead-hashset-extend-iter-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_btreemap_append_values_last_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "btreemap_append_values_last_map_prune",
        "btreemap_append_values_last_map",
        &[
            "BTreeMap::new()",
            "values.append(&mut extras)",
            ".values()",
            ".last()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadBtreemapAppendValuesLastMapItem",
        "dead-btreemap-append-values-last-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_btreemap_clear_insert_values_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "btreemap_clear_insert_values_map_prune",
        "btreemap_clear_insert_values_map",
        &[
            "BTreeMap::new()",
            "values.clear()",
            "values.insert(1usize",
            ".values()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadBtreemapClearInsertValuesMapItem",
        "dead-btreemap-clear-insert-values-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_btreeset_append_iter_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "btreeset_append_iter_map_prune",
        "btreeset_append_iter_map",
        &[
            "BTreeSet::new()",
            "values.append(&mut extras)",
            ".iter()",
            "BtreesetAppendIterMapPayload::new(value).render_label()",
            "pub fn render_label",
        ],
        "DeadBtreesetAppendIterMapItem",
        "dead-btreeset-append-iter-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_binaryheap_push_pop_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "binaryheap_push_pop_map_prune",
        "binaryheap_push_pop_map",
        &[
            "BinaryHeap::new()",
            "values.push(raw.to_string())",
            ".pop()",
            "BinaryheapPushPopMapPayload::new(&value).render_label()",
            "pub fn render_label",
        ],
        "DeadBinaryheapPushPopMapItem",
        "dead-binaryheap-push-pop-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_binaryheap_into_sorted_vec_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "binaryheap_into_sorted_vec_map_prune",
        "binaryheap_into_sorted_vec_map",
        &[
            "BinaryHeap::from([raw.to_string()])",
            ".into_sorted_vec()",
            ".into_iter()",
            "BinaryheapIntoSortedVecMapPayload::new(&value).render_label()",
            "pub fn render_label",
        ],
        "DeadBinaryheapIntoSortedVecMapItem",
        "dead-binaryheap-into-sorted-vec-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_linkedlist_push_back_front_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "linkedlist_push_back_front_map_prune",
        "linkedlist_push_back_front_map",
        &[
            "LinkedList::new()",
            "values.push_back(",
            ".front()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadLinkedlistPushBackFrontMapItem",
        "dead-linkedlist-push-back-front-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_linkedlist_append_back_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "linkedlist_append_back_map_prune",
        "linkedlist_append_back_map",
        &[
            "LinkedList::new()",
            "values.append(&mut extras)",
            ".back()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadLinkedlistAppendBackMapItem",
        "dead-linkedlist-append-back-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_vecdeque_push_front_back_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vecdeque_push_front_back_map_prune",
        "vecdeque_push_front_back_map",
        &[
            "VecDeque::new()",
            "values.push_front(",
            ".back()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadVecdequePushFrontBackMapItem",
        "dead-vecdeque-push-front-back-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_vecdeque_resize_with_back_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vecdeque_resize_with_back_map_prune",
        "vecdeque_resize_with_back_map",
        &[
            "VecDeque::new()",
            "values.resize_with(2, || VecdequeResizeWithBackMapPayload::new(raw))",
            ".back()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadVecdequeResizeWithBackMapItem",
        "dead-vecdeque-resize-with-back-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_arc_get_mut_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "arc_get_mut_map_prune",
        "arc_get_mut_map",
        &[
            "use std::sync::Arc;",
            "Arc::new",
            "Arc::get_mut(&mut payload)",
            ".map(|payload| payload.bump_and_render())",
            "pub fn bump_and_render",
        ],
        "DeadArcGetMutMapItem",
        "dead-arc-get-mut-map",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_rc_get_mut_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "rc_get_mut_map_prune",
        "rc_get_mut_map",
        &[
            "use std::rc::Rc;",
            "Rc::new",
            "Rc::get_mut(&mut payload)",
            ".map(|payload| payload.bump_and_render())",
            "pub fn bump_and_render",
        ],
        "DeadRcGetMutMapItem",
        "dead-rc-get-mut-map",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_mutex_get_mut_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "mutex_get_mut_map_prune",
        "mutex_get_mut_map",
        &[
            "use std::sync::Mutex;",
            "Mutex::new",
            ".get_mut()",
            ".map(|payload| payload.bump_and_render())",
            "pub fn bump_and_render",
        ],
        "DeadMutexGetMutMapItem",
        "dead-mutex-get-mut-map",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_rwlock_get_mut_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "rwlock_get_mut_map_prune",
        "rwlock_get_mut_map",
        &[
            "use std::sync::RwLock;",
            "RwLock::new",
            ".get_mut()",
            ".map(|payload| payload.bump_and_render())",
            "pub fn bump_and_render",
        ],
        "DeadRwlockGetMutMapItem",
        "dead-rwlock-get-mut-map",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_refcell_get_mut_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "refcell_get_mut_map_prune",
        "refcell_get_mut_map",
        &[
            "use std::cell::RefCell;",
            "RefCell::new",
            "payload.get_mut().bump_and_render()",
            "pub fn bump_and_render",
        ],
        "DeadRefcellGetMutMapItem",
        "dead-refcell-get-mut-map",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_cell_set_get_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "cell_set_get_map_prune",
        "cell_set_get_map",
        &[
            "use std::cell::Cell;",
            "Cell::new(raw.len())",
            "value.set(value.get() + 1)",
            "CellSetGetMapPayload::new(&value.get().to_string()).render_label()",
            "pub fn render_label",
        ],
        "DeadCellSetGetMapItem",
        "dead-cell-set-get-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_once_lock_set_get_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "once_lock_set_get_map_prune",
        "once_lock_set_get_map",
        &[
            "use std::sync::OnceLock;",
            "OnceLock::new",
            "slot.set(OnceLockSetGetMapPayload::new(raw))",
            "slot.get()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadOnceLockSetGetMapItem",
        "dead-once-lock-set-get-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_lazy_lock_force_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "lazy_lock_force_map_prune",
        "lazy_lock_force_map",
        &[
            "use std::sync::LazyLock;",
            "static PAYLOAD: LazyLock<LazyLockForceMapPayload>",
            "LazyLock::new(|| LazyLockForceMapPayload::new",
            "\"lazy\"",
            "PAYLOAD.render_label()",
            "pub fn render_label",
        ],
        "DeadLazyLockForceMapItem",
        "dead-lazy-lock-force-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_box_leak_mut_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "box_leak_mut_map_prune",
        "box_leak_mut_map",
        &[
            "Box::leak(Box::new(BoxLeakMutMapPayload::new(raw)))",
            "payload.bump_and_render()",
            "pub fn bump_and_render",
        ],
        "DeadBoxLeakMutMapItem",
        "dead-box-leak-mut-map",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_pin_into_inner_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "pin_into_inner_map_prune",
        "pin_into_inner_map",
        &[
            "use std::pin::Pin;",
            "Box::pin(PinIntoInnerMapPayload::new(raw))",
            "Pin::into_inner(pinned).render_label()",
            "pub fn render_label",
        ],
        "DeadPinIntoInnerMapItem",
        "dead-pin-into-inner-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_maybeuninit_write_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "maybeuninit_write_map_prune",
        "maybeuninit_write_map",
        &[
            "use std::mem::MaybeUninit;",
            "MaybeUninit::uninit()",
            "slot.write(MaybeuninitWriteMapPayload::new(raw))",
            "payload.bump_and_render()",
            "pub fn bump_and_render",
        ],
        "DeadMaybeuninitWriteMapItem",
        "dead-maybeuninit-write-map",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_nonnull_as_mut_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "nonnull_as_mut_map_prune",
        "nonnull_as_mut_map",
        &[
            "use std::ptr::NonNull;",
            "NonNull::from(leaked)",
            "unsafe { ptr.as_mut().bump_and_render() }",
            "pub fn bump_and_render",
        ],
        "DeadNonnullAsMutMapItem",
        "dead-nonnull-as-mut-map",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_result_or_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "result_or_map_prune",
        "result_or_map",
        &[
            "let value: Result<ResultOrMapPayload, ResultOrMapPayload>",
            ".or(",
            "Ok::<",
            "ResultOrMapPayload",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadResultOrMapItem",
        "dead-result-or-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_option_into_iter_next_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "option_into_iter_next_map_prune",
        "option_into_iter_next_map",
        &[
            "Some(OptionIntoIterNextMapPayload::new(raw))",
            ".into_iter()",
            ".next()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadOptionIntoIterNextMapItem",
        "dead-option-into-iter-next-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_result_into_iter_next_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "result_into_iter_next_map_prune",
        "result_into_iter_next_map",
        &[
            "Ok::<_, ()>(ResultIntoIterNextMapPayload::new(raw))",
            ".into_iter()",
            ".next()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadResultIntoIterNextMapItem",
        "dead-result-into-iter-next-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_array_map_payload_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "array_map_payload_map_prune",
        "array_map_payload_map",
        &[
            "ArrayMapPayloadMapPayload::new(raw)",
            ".map(|payload| payload.render_label())",
            ".join(\"|\")",
            "pub fn render_label",
        ],
        "DeadArrayMapPayloadMapItem",
        "dead-array-map-payload-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_vecdeque_push_back_front_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vecdeque_push_back_front_map_prune",
        "vecdeque_push_back_front_map",
        &[
            "use std::collections::VecDeque;",
            "VecDeque::new()",
            "values.push_back(",
            ".front()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadVecdequePushBackFrontMapItem",
        "dead-vecdeque-push-back-front-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_linkedlist_push_front_back_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "linkedlist_push_front_back_map_prune",
        "linkedlist_push_front_back_map",
        &[
            "use std::collections::LinkedList;",
            "LinkedList::new()",
            "values.push_front(",
            ".back()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadLinkedlistPushFrontBackMapItem",
        "dead-linkedlist-push-front-back-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_binaryheap_shrink_to_fit_peek_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "binaryheap_shrink_to_fit_peek_map_prune",
        "binaryheap_shrink_to_fit_peek_map",
        &[
            "use std::collections::BinaryHeap;",
            "BinaryHeap::from([raw.to_string()])",
            "values.shrink_to_fit()",
            ".peek()",
            "BinaryheapShrinkToFitPeekMapPayload::new(value).render_label()",
            "pub fn render_label",
        ],
        "DeadBinaryheapShrinkToFitPeekMapItem",
        "dead-binaryheap-shrink-to-fit-peek-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_btreeset_split_off_iter_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "btreeset_split_off_iter_map_prune",
        "btreeset_split_off_iter_map",
        &[
            "use std::collections::BTreeSet;",
            "BTreeSet::new()",
            "values.split_off(raw)",
            "tail.iter()",
            "BtreesetSplitOffIterMapPayload::new(value).render_label()",
            "pub fn render_label",
        ],
        "DeadBtreesetSplitOffIterMapItem",
        "dead-btreeset-split-off-iter-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_hashset_shrink_to_fit_iter_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "hashset_shrink_to_fit_iter_map_prune",
        "hashset_shrink_to_fit_iter_map",
        &[
            "use std::collections::HashSet;",
            "HashSet::new()",
            "values.shrink_to_fit()",
            ".iter()",
            "HashsetShrinkToFitIterMapPayload::new(value).render_label()",
            "pub fn render_label",
        ],
        "DeadHashsetShrinkToFitIterMapItem",
        "dead-hashset-shrink-to-fit-iter-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_hashmap_clear_insert_get_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "hashmap_clear_insert_get_map_prune",
        "hashmap_clear_insert_get_map",
        &[
            "use std::collections::HashMap;",
            "HashMap::new()",
            "values.clear()",
            "values.insert(raw.to_string()",
            ".get(raw)",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadHashmapClearInsertGetMapItem",
        "dead-hashmap-clear-insert-get-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_pathbuf_as_path_file_name_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "pathbuf_as_path_file_name_map_prune",
        "pathbuf_as_path_file_name_map",
        &[
            "use std::path::PathBuf;",
            "PathBuf::from(raw).join(\"file.txt\")",
            "path.as_path()",
            ".file_name()",
            ".and_then(|name| name.to_str())",
            "PathbufAsPathFileNameMapPayload::new(value).render_label()",
            "pub fn render_label",
        ],
        "DeadPathbufAsPathFileNameMapItem",
        "dead-pathbuf-as-path-file-name-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_osstring_clear_push_into_string_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "osstring_clear_push_into_string_map_prune",
        "osstring_clear_push_into_string_map",
        &[
            "use std::ffi::OsString;",
            "OsString::from(\"dead\")",
            "value.clear()",
            "value.push(raw)",
            ".into_string()",
            "OsstringClearPushIntoStringMapPayload::new(&text).render_label()",
            "pub fn render_label",
        ],
        "DeadOsstringClearPushIntoStringMapItem",
        "dead-osstring-clear-push-into-string-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_cstring_as_bytes_first_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "cstring_as_bytes_first_map_prune",
        "cstring_as_bytes_first_map",
        &[
            "use std::ffi::CString;",
            "CString::new(raw).unwrap_or_else",
            ".as_bytes()",
            ".first()",
            "CstringAsBytesFirstMapPayload::new(&byte.to_string()).render_label()",
            "pub fn render_label",
        ],
        "DeadCstringAsBytesFirstMapItem",
        "dead-cstring-as-bytes-first-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_string_into_bytes_first_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "string_into_bytes_first_map_prune",
        "string_into_bytes_first_map",
        &[
            "String::from(raw)",
            ".into_bytes()",
            ".into_iter()",
            ".next()",
            "StringIntoBytesFirstMapPayload::new(&byte.to_string()).render_label()",
            "pub fn render_label",
        ],
        "DeadStringIntoBytesFirstMapItem",
        "dead-string-into-bytes-first-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_arc_try_unwrap_ok_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "arc_try_unwrap_ok_map_prune",
        "arc_try_unwrap_ok_map",
        &[
            "use std::sync::Arc;",
            "Arc::try_unwrap(payload)",
            ".ok()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadArcTryUnwrapOkMapItem",
        "dead-arc-try-unwrap-ok-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_rc_try_unwrap_ok_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "rc_try_unwrap_ok_map_prune",
        "rc_try_unwrap_ok_map",
        &[
            "use std::rc::Rc;",
            "Rc::try_unwrap(payload)",
            ".ok()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadRcTryUnwrapOkMapItem",
        "dead-rc-try-unwrap-ok-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_arc_unwrap_or_clone_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "arc_unwrap_or_clone_map_prune",
        "arc_unwrap_or_clone_map",
        &[
            "use std::sync::Arc;",
            "Arc::unwrap_or_clone(payload).render_label()",
            "pub fn render_label",
        ],
        "DeadArcUnwrapOrCloneMapItem",
        "dead-arc-unwrap-or-clone-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_rc_unwrap_or_clone_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "rc_unwrap_or_clone_map_prune",
        "rc_unwrap_or_clone_map",
        &[
            "use std::rc::Rc;",
            "Rc::unwrap_or_clone(payload).render_label()",
            "pub fn render_label",
        ],
        "DeadRcUnwrapOrCloneMapItem",
        "dead-rc-unwrap-or-clone-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_mutex_into_inner_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "mutex_into_inner_map_prune",
        "mutex_into_inner_map",
        &[
            "use std::sync::Mutex;",
            "Mutex::into_inner(payload)",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadMutexIntoInnerMapItem",
        "dead-mutex-into-inner-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_rwlock_into_inner_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "rwlock_into_inner_map_prune",
        "rwlock_into_inner_map",
        &[
            "use std::sync::RwLock;",
            "RwLock::into_inner(payload)",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadRwlockIntoInnerMapItem",
        "dead-rwlock-into-inner-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_cell_into_inner_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "cell_into_inner_map_prune",
        "cell_into_inner_map",
        &[
            "use std::cell::Cell;",
            "Cell::new(CellIntoInnerMapPayload::new(raw))",
            "payload.into_inner().render_label()",
            "pub fn render_label",
        ],
        "DeadCellIntoInnerMapItem",
        "dead-cell-into-inner-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_refcell_into_inner_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "refcell_into_inner_map_prune",
        "refcell_into_inner_map",
        &[
            "use std::cell::RefCell;",
            "RefCell::new(RefcellIntoInnerMapPayload::new(raw))",
            "payload.into_inner().render_label()",
            "pub fn render_label",
        ],
        "DeadRefcellIntoInnerMapItem",
        "dead-refcell-into-inner-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_once_lock_into_inner_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "once_lock_into_inner_map_prune",
        "once_lock_into_inner_map",
        &[
            "use std::sync::OnceLock;",
            "let _ = slot.set(OnceLockIntoInnerMapPayload::new(raw))",
            "slot.into_inner()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadOnceLockIntoInnerMapItem",
        "dead-once-lock-into-inner-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_once_lock_get_mut_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "once_lock_get_mut_map_prune",
        "once_lock_get_mut_map",
        &[
            "use std::sync::OnceLock;",
            "let mut slot: OnceLock<OnceLockGetMutMapPayload>",
            "slot.get_mut()",
            ".map(|payload| payload.bump_and_render())",
            "pub fn bump_and_render",
        ],
        "DeadOnceLockGetMutMapItem",
        "dead-once-lock-get-mut-map",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_once_lock_take_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "once_lock_take_map_prune",
        "once_lock_take_map",
        &[
            "use std::sync::OnceLock;",
            "let mut slot: OnceLock<OnceLockTakeMapPayload>",
            "slot.take()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadOnceLockTakeMapItem",
        "dead-once-lock-take-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_option_unwrap_unchecked_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "option_unwrap_unchecked_map_prune",
        "option_unwrap_unchecked_map",
        &[
            "Some(OptionUnwrapUncheckedMapPayload::new(raw))",
            "unsafe { value.unwrap_unchecked().render_label() }",
            "pub fn render_label",
        ],
        "DeadOptionUnwrapUncheckedMapItem",
        "dead-option-unwrap-unchecked-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_result_unwrap_unchecked_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "result_unwrap_unchecked_map_prune",
        "result_unwrap_unchecked_map",
        &[
            "let value: Result<ResultUnwrapUncheckedMapPayload, ()>",
            "unsafe { value.unwrap_unchecked().render_label() }",
            "pub fn render_label",
        ],
        "DeadResultUnwrapUncheckedMapItem",
        "dead-result-unwrap-unchecked-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_result_unwrap_err_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "result_unwrap_err_map_prune",
        "result_unwrap_err_map",
        &[
            "let value: Result<(), ResultUnwrapErrMapPayload>",
            "value.unwrap_err().render_label()",
            "pub fn render_label",
        ],
        "DeadResultUnwrapErrMapItem",
        "dead-result-unwrap-err-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_result_expect_err_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "result_expect_err_map_prune",
        "result_expect_err_map",
        &[
            "let value: Result<(), ResultExpectErrMapPayload>",
            "value.expect_err(\"expected payload\").render_label()",
            "pub fn render_label",
        ],
        "DeadResultExpectErrMapItem",
        "dead-result-expect-err-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_result_unwrap_err_unchecked_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "result_unwrap_err_unchecked_map_prune",
        "result_unwrap_err_unchecked_map",
        &[
            "let value: Result<(), ResultUnwrapErrUncheckedMapPayload>",
            "unsafe { value.unwrap_err_unchecked().render_label() }",
            "pub fn render_label",
        ],
        "DeadResultUnwrapErrUncheckedMapItem",
        "dead-result-unwrap-err-unchecked-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_box_into_raw_from_raw_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "box_into_raw_from_raw_map_prune",
        "box_into_raw_from_raw_map",
        &[
            "Box::into_raw(Box::new(BoxIntoRawFromRawMapPayload::new(raw)))",
            "unsafe { Box::from_raw(ptr).render_label() }",
            "pub fn render_label",
        ],
        "DeadBoxIntoRawFromRawMapItem",
        "dead-box-into-raw-from-raw-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_vec_into_boxed_slice_into_vec_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vec_into_boxed_slice_into_vec_map_prune",
        "vec_into_boxed_slice_into_vec_map",
        &[
            ".into_boxed_slice()",
            ".into_vec()",
            ".into_iter()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadVecIntoBoxedSliceIntoVecMapItem",
        "dead-vec-into-boxed-slice-into-vec-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_boxed_slice_iter_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "boxed_slice_iter_map_prune",
        "boxed_slice_iter_map",
        &[
            "let boxed: Box<[BoxedSliceIterMapPayload]>",
            ".iter()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadBoxedSliceIterMapItem",
        "dead-boxed-slice-iter-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_vec_from_array_into_iter_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vec_from_array_into_iter_map_prune",
        "vec_from_array_into_iter_map",
        &[
            "Vec::from([VecFromArrayIntoIterMapPayload::new(raw)])",
            ".into_iter()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadVecFromArrayIntoIterMapItem",
        "dead-vec-from-array-into-iter-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_arc_into_inner_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "arc_into_inner_map_prune",
        "arc_into_inner_map",
        &[
            "use std::sync::Arc;",
            "Arc::into_inner(payload)",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadArcIntoInnerMapItem",
        "dead-arc-into-inner-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_rc_into_inner_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "rc_into_inner_map_prune",
        "rc_into_inner_map",
        &[
            "use std::rc::Rc;",
            "Rc::into_inner(payload)",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadRcIntoInnerMapItem",
        "dead-rc-into-inner-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_arc_try_unwrap_unwrap_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "arc_try_unwrap_unwrap_map_prune",
        "arc_try_unwrap_unwrap_map",
        &[
            "use std::sync::Arc;",
            "Arc::try_unwrap(payload).unwrap().render_label()",
            "pub fn render_label",
        ],
        "DeadArcTryUnwrapUnwrapMapItem",
        "dead-arc-try-unwrap-unwrap-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_rc_try_unwrap_unwrap_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "rc_try_unwrap_unwrap_map_prune",
        "rc_try_unwrap_unwrap_map",
        &[
            "use std::rc::Rc;",
            "Rc::try_unwrap(payload).unwrap().render_label()",
            "pub fn render_label",
        ],
        "DeadRcTryUnwrapUnwrapMapItem",
        "dead-rc-try-unwrap-unwrap-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_mutex_try_lock_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "mutex_try_lock_map_prune",
        "mutex_try_lock_map",
        &[
            "use std::sync::Mutex;",
            ".try_lock()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadMutexTryLockMapItem",
        "dead-mutex-try-lock-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_mutex_try_lock_unwrap_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "mutex_try_lock_unwrap_map_prune",
        "mutex_try_lock_unwrap_map",
        &[
            "use std::sync::Mutex;",
            "payload.try_lock().unwrap().render_label()",
            "pub fn render_label",
        ],
        "DeadMutexTryLockUnwrapMapItem",
        "dead-mutex-try-lock-unwrap-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_rwlock_try_read_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "rwlock_try_read_map_prune",
        "rwlock_try_read_map",
        &[
            "use std::sync::RwLock;",
            ".try_read()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadRwlockTryReadMapItem",
        "dead-rwlock-try-read-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_rwlock_try_write_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "rwlock_try_write_map_prune",
        "rwlock_try_write_map",
        &[
            "use std::sync::RwLock;",
            ".try_write()",
            ".map(|mut payload| payload.bump_and_render())",
            "pub fn bump_and_render",
        ],
        "DeadRwlockTryWriteMapItem",
        "dead-rwlock-try-write-map",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_rwlock_try_read_unwrap_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "rwlock_try_read_unwrap_map_prune",
        "rwlock_try_read_unwrap_map",
        &[
            "use std::sync::RwLock;",
            "payload.try_read().unwrap().render_label()",
            "pub fn render_label",
        ],
        "DeadRwlockTryReadUnwrapMapItem",
        "dead-rwlock-try-read-unwrap-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_rwlock_try_write_unwrap_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "rwlock_try_write_unwrap_map_prune",
        "rwlock_try_write_unwrap_map",
        &[
            "use std::sync::RwLock;",
            "payload.try_write().unwrap().bump_and_render()",
            "pub fn bump_and_render",
        ],
        "DeadRwlockTryWriteUnwrapMapItem",
        "dead-rwlock-try-write-unwrap-map",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_refcell_try_borrow_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "refcell_try_borrow_map_prune",
        "refcell_try_borrow_map",
        &[
            "use std::cell::RefCell;",
            ".try_borrow()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadRefcellTryBorrowMapItem",
        "dead-refcell-try-borrow-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_refcell_try_borrow_mut_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "refcell_try_borrow_mut_map_prune",
        "refcell_try_borrow_mut_map",
        &[
            "use std::cell::RefCell;",
            ".try_borrow_mut()",
            ".map(|mut payload| payload.bump_and_render())",
            "pub fn bump_and_render",
        ],
        "DeadRefcellTryBorrowMutMapItem",
        "dead-refcell-try-borrow-mut-map",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_refcell_try_borrow_unwrap_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "refcell_try_borrow_unwrap_map_prune",
        "refcell_try_borrow_unwrap_map",
        &[
            "use std::cell::RefCell;",
            "payload.try_borrow().unwrap().render_label()",
            "pub fn render_label",
        ],
        "DeadRefcellTryBorrowUnwrapMapItem",
        "dead-refcell-try-borrow-unwrap-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_refcell_try_borrow_mut_unwrap_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "refcell_try_borrow_mut_unwrap_map_prune",
        "refcell_try_borrow_mut_unwrap_map",
        &[
            "use std::cell::RefCell;",
            "payload.try_borrow_mut().unwrap().bump_and_render()",
            "pub fn bump_and_render",
        ],
        "DeadRefcellTryBorrowMutUnwrapMapItem",
        "dead-refcell-try-borrow-mut-unwrap-map",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_cell_take_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "cell_take_map_prune",
        "cell_take_map",
        &[
            "use std::cell::Cell;",
            "payload.take().render_label()",
            "pub fn render_label",
        ],
        "DeadCellTakeMapItem",
        "dead-cell-take-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_refcell_take_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "refcell_take_map_prune",
        "refcell_take_map",
        &[
            "use std::cell::RefCell;",
            "payload.take().render_label()",
            "pub fn render_label",
        ],
        "DeadRefcellTakeMapItem",
        "dead-refcell-take-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_mutex_into_inner_unwrap_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "mutex_into_inner_unwrap_map_prune",
        "mutex_into_inner_unwrap_map",
        &[
            "use std::sync::Mutex;",
            "Mutex::into_inner(payload).unwrap().render_label()",
            "pub fn render_label",
        ],
        "DeadMutexIntoInnerUnwrapMapItem",
        "dead-mutex-into-inner-unwrap-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_rwlock_into_inner_unwrap_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "rwlock_into_inner_unwrap_map_prune",
        "rwlock_into_inner_unwrap_map",
        &[
            "use std::sync::RwLock;",
            "RwLock::into_inner(payload).unwrap().render_label()",
            "pub fn render_label",
        ],
        "DeadRwlockIntoInnerUnwrapMapItem",
        "dead-rwlock-into-inner-unwrap-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_once_lock_into_inner_unwrap_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "once_lock_into_inner_unwrap_map_prune",
        "once_lock_into_inner_unwrap_map",
        &[
            "use std::sync::OnceLock;",
            "slot.into_inner().unwrap().render_label()",
            "pub fn render_label",
        ],
        "DeadOnceLockIntoInnerUnwrapMapItem",
        "dead-once-lock-into-inner-unwrap-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_vecdeque_make_contiguous_first_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vecdeque_make_contiguous_first_map_prune",
        "vecdeque_make_contiguous_first_map",
        &[
            "use std::collections::VecDeque;",
            ".make_contiguous()",
            ".first()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadVecdequeMakeContiguousFirstMapItem",
        "dead-vecdeque-make-contiguous-first-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_vecdeque_make_contiguous_last_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vecdeque_make_contiguous_last_map_prune",
        "vecdeque_make_contiguous_last_map",
        &[
            "use std::collections::VecDeque;",
            ".make_contiguous()",
            ".last()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadVecdequeMakeContiguousLastMapItem",
        "dead-vecdeque-make-contiguous-last-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_vecdeque_make_contiguous_first_mut_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vecdeque_make_contiguous_first_mut_map_prune",
        "vecdeque_make_contiguous_first_mut_map",
        &[
            "use std::collections::VecDeque;",
            ".make_contiguous()",
            ".first_mut()",
            ".map(|payload| payload.bump_and_render())",
            "pub fn bump_and_render",
        ],
        "DeadVecdequeMakeContiguousFirstMutMapItem",
        "dead-vecdeque-make-contiguous-first-mut-map",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_vecdeque_make_contiguous_last_mut_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vecdeque_make_contiguous_last_mut_map_prune",
        "vecdeque_make_contiguous_last_mut_map",
        &[
            "use std::collections::VecDeque;",
            ".make_contiguous()",
            ".last_mut()",
            ".map(|payload| payload.bump_and_render())",
            "pub fn bump_and_render",
        ],
        "DeadVecdequeMakeContiguousLastMutMapItem",
        "dead-vecdeque-make-contiguous-last-mut-map",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_mpsc_recv_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "mpsc_recv_map_prune",
        "mpsc_recv_map",
        &[
            "use std::sync::mpsc::{self, Receiver, Sender};",
            "Receiver<MpscRecvMapPayload>",
            ".recv()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadMpscRecvMapItem",
        "dead-mpsc-recv-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_mpsc_try_recv_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "mpsc_try_recv_map_prune",
        "mpsc_try_recv_map",
        &[
            ".try_recv()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadMpscTryRecvMapItem",
        "dead-mpsc-try-recv-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_mpsc_recv_unwrap_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "mpsc_recv_unwrap_map_prune",
        "mpsc_recv_unwrap_map",
        &["rx.recv().unwrap().render_label()", "pub fn render_label"],
        "DeadMpscRecvUnwrapMapItem",
        "dead-mpsc-recv-unwrap-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_mpsc_try_recv_unwrap_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "mpsc_try_recv_unwrap_map_prune",
        "mpsc_try_recv_unwrap_map",
        &[
            "rx.try_recv().unwrap().render_label()",
            "pub fn render_label",
        ],
        "DeadMpscTryRecvUnwrapMapItem",
        "dead-mpsc-try-recv-unwrap-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_mpsc_recv_timeout_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "mpsc_recv_timeout_map_prune",
        "mpsc_recv_timeout_map",
        &[
            "Duration::from_millis(1)",
            ".recv_timeout(",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadMpscRecvTimeoutMapItem",
        "dead-mpsc-recv-timeout-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_mpsc_iter_next_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "mpsc_iter_next_map_prune",
        "mpsc_iter_next_map",
        &[
            ".iter()",
            ".next()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadMpscIterNextMapItem",
        "dead-mpsc-iter-next-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_mpsc_try_iter_next_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "mpsc_try_iter_next_map_prune",
        "mpsc_try_iter_next_map",
        &[
            ".try_iter()",
            ".next()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadMpscTryIterNextMapItem",
        "dead-mpsc-try-iter-next-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_mpsc_into_iter_next_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "mpsc_into_iter_next_map_prune",
        "mpsc_into_iter_next_map",
        &[
            ".into_iter()",
            ".next()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadMpscIntoIterNextMapItem",
        "dead-mpsc-into-iter-next-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_sync_mpsc_recv_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "sync_mpsc_recv_map_prune",
        "sync_mpsc_recv_map",
        &[
            "SyncSender<SyncMpscRecvMapPayload>",
            "mpsc::sync_channel",
            ".recv()",
            "pub fn render_label",
        ],
        "DeadSyncMpscRecvMapItem",
        "dead-sync-mpsc-recv-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_sync_mpsc_try_recv_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "sync_mpsc_try_recv_map_prune",
        "sync_mpsc_try_recv_map",
        &[
            "mpsc::sync_channel",
            ".try_recv()",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadSyncMpscTryRecvMapItem",
        "dead-sync-mpsc-try-recv-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_iter_once_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "iter_once_map_prune",
        "iter_once_map",
        &[
            "iter::once(IterOnceMapPayload::new(raw))",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadIterOnceMapItem",
        "dead-iter-once-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_iter_once_with_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "iter_once_with_map_prune",
        "iter_once_with_map",
        &[
            "iter::once_with(|| IterOnceWithMapPayload::new(raw))",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadIterOnceWithMapItem",
        "dead-iter-once-with-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_iter_repeat_n_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "iter_repeat_n_map_prune",
        "iter_repeat_n_map",
        &[
            "iter::repeat_n(IterRepeatNMapPayload::new(raw), 1)",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadIterRepeatNMapItem",
        "dead-iter-repeat-n-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_iter_empty_chain_once_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "iter_empty_chain_once_map_prune",
        "iter_empty_chain_once_map",
        &[
            "iter::empty::<IterEmptyChainOnceMapPayload>()",
            ".chain(iter::once(",
            ".map(|payload| payload.render_label())",
            "pub fn render_label",
        ],
        "DeadIterEmptyChainOnceMapItem",
        "dead-iter-empty-chain-once-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_slice_chunk_by_flatten_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "slice_chunk_by_flatten_map_prune",
        "slice_chunk_by_flatten_map",
        &[
            ".chunk_by(|left, right| left.group_key() == right.group_key())",
            ".flatten()",
            ".map(|payload| payload.render_label())",
            "pub fn group_key",
            "pub fn render_label",
        ],
        "DeadSliceChunkByFlattenMapItem",
        "dead-slice-chunk-by-flatten-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_slice_chunk_by_mut_flatten_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "slice_chunk_by_mut_flatten_map_prune",
        "slice_chunk_by_mut_flatten_map",
        &[
            ".chunk_by_mut(|left, right| left.group_key() == right.group_key())",
            ".flatten()",
            ".map(|payload| payload.bump_and_render())",
            "pub fn group_key",
            "pub fn bump_and_render",
        ],
        "DeadSliceChunkByMutFlattenMapItem",
        "dead-slice-chunk-by-mut-flatten-map",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}
#[test]
fn prunes_typed_array_pattern_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "typed_array_pattern_prune",
        "typed_array_pattern",
        &[
            "let [payload, _other]: [TypedArrayPatternPayload; 2]",
            "payload.render_label()",
            "pub fn render_label",
        ],
        "DeadTypedArrayPatternItem",
        "dead-typed-array-pattern",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_typed_array_ref_pattern_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "typed_array_ref_pattern_prune",
        "typed_array_ref_pattern",
        &[
            "let [payload, _other]: &[TypedArrayRefPatternPayload; 2]",
            "payload.render_label()",
            "pub fn render_label",
        ],
        "DeadTypedArrayRefPatternItem",
        "dead-typed-array-ref-pattern",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_typed_array_mut_pattern_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "typed_array_mut_pattern_prune",
        "typed_array_mut_pattern",
        &[
            "let [payload, _other]: &mut [TypedArrayMutPatternPayload; 2]",
            "payload.bump_and_render()",
            "pub fn bump_and_render",
        ],
        "DeadTypedArrayMutPatternItem",
        "dead-typed-array-mut-pattern",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_slice_let_else_pattern_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "slice_let_else_pattern_prune",
        "slice_let_else_pattern",
        &[
            "let [payload, ..] = items.as_slice()",
            "payload.render_label()",
            "pub fn render_label",
        ],
        "DeadSliceLetElsePatternItem",
        "dead-slice-let-else-pattern",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_slice_if_let_pattern_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "slice_if_let_pattern_prune",
        "slice_if_let_pattern",
        &[
            "if let [payload, ..] = items.as_slice()",
            "payload.render_label()",
            "pub fn render_label",
        ],
        "DeadSliceIfLetPatternItem",
        "dead-slice-if-let-pattern",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_slice_mut_let_else_pattern_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "slice_mut_let_else_pattern_prune",
        "slice_mut_let_else_pattern",
        &[
            "let [payload, ..] = items.as_mut_slice()",
            "payload.bump_and_render()",
            "pub fn bump_and_render",
        ],
        "DeadSliceMutLetElsePatternItem",
        "dead-slice-mut-let-else-pattern",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_slice_mut_if_let_pattern_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "slice_mut_if_let_pattern_prune",
        "slice_mut_if_let_pattern",
        &[
            "if let [payload, ..] = items.as_mut_slice()",
            "payload.bump_and_render()",
            "pub fn bump_and_render",
        ],
        "DeadSliceMutIfLetPatternItem",
        "dead-slice-mut-if-let-pattern",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_typed_slice_ref_pattern_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "typed_slice_ref_pattern_prune",
        "typed_slice_ref_pattern",
        &[
            "let [payload, ..]: &[TypedSliceRefPatternPayload] = items.as_slice()",
            "payload.render_label()",
            "pub fn render_label",
        ],
        "DeadTypedSliceRefPatternItem",
        "dead-typed-slice-ref-pattern",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_typed_slice_mut_pattern_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "typed_slice_mut_pattern_prune",
        "typed_slice_mut_pattern",
        &[
            "let [payload, ..]: &mut [TypedSliceMutPatternPayload] = items.as_mut_slice()",
            "payload.bump_and_render()",
            "pub fn bump_and_render",
        ],
        "DeadTypedSliceMutPatternItem",
        "dead-typed-slice-mut-pattern",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_typed_slice_tuple_pattern_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "typed_slice_tuple_pattern_prune",
        "typed_slice_tuple_pattern",
        &[
            "let [(payload, _flag), ..]: &[(TypedSliceTuplePatternPayload, bool)]",
            "payload.render_label()",
            "pub fn render_label",
        ],
        "DeadTypedSliceTuplePatternItem",
        "dead-typed-slice-tuple-pattern",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_typed_slice_tuple_mut_pattern_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "typed_slice_tuple_mut_pattern_prune",
        "typed_slice_tuple_mut_pattern",
        &[
            "let [(payload, _flag), ..]: &mut [(TypedSliceTupleMutPatternPayload, bool)]",
            "payload.bump_and_render()",
            "pub fn bump_and_render",
        ],
        "DeadTypedSliceTupleMutPatternItem",
        "dead-typed-slice-tuple-mut-pattern",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_typed_slice_named_struct_pattern_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "typed_slice_named_struct_pattern_prune",
        "typed_slice_named_struct_pattern",
        &[
            "struct TypedSliceNamedStructPatternEnvelope",
            "payload.render_label()",
            "pub fn render_label",
        ],
        "DeadTypedSliceNamedStructPatternItem",
        "dead-typed-slice-named-struct-pattern",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_typed_slice_tuple_struct_pattern_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "typed_slice_tuple_struct_pattern_prune",
        "typed_slice_tuple_struct_pattern",
        &[
            "struct TypedSliceTupleStructPatternEnvelope",
            "payload.render_label()",
            "pub fn render_label",
        ],
        "DeadTypedSliceTupleStructPatternItem",
        "dead-typed-slice-tuple-struct-pattern",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_typed_slice_enum_tuple_pattern_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "typed_slice_enum_tuple_pattern_prune",
        "typed_slice_enum_tuple_pattern",
        &[
            "enum TypedSliceEnumTuplePatternEnvelope",
            "payload.render_label()",
            "pub fn render_label",
        ],
        "DeadTypedSliceEnumTuplePatternItem",
        "dead-typed-slice-enum-tuple-pattern",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_typed_slice_enum_struct_pattern_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "typed_slice_enum_struct_pattern_prune",
        "typed_slice_enum_struct_pattern",
        &[
            "enum TypedSliceEnumStructPatternEnvelope",
            "payload.render_label()",
            "pub fn render_label",
        ],
        "DeadTypedSliceEnumStructPatternItem",
        "dead-typed-slice-enum-struct-pattern",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_slice_chunks_for_pattern_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "slice_chunks_for_pattern_prune",
        "slice_chunks_for_pattern",
        &[
            "for chunk in items.chunks(2)",
            "let [payload, ..] = chunk",
            "payload.render_label()",
            "pub fn render_label",
        ],
        "DeadSliceChunksForPatternItem",
        "dead-slice-chunks-for-pattern",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_slice_chunks_mut_for_pattern_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "slice_chunks_mut_for_pattern_prune",
        "slice_chunks_mut_for_pattern",
        &[
            "for chunk in items.chunks_mut(2)",
            "let [payload, ..] = chunk",
            "payload.bump_and_render()",
            "pub fn bump_and_render",
        ],
        "DeadSliceChunksMutForPatternItem",
        "dead-slice-chunks-mut-for-pattern",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}
#[test]
fn prunes_iterator_rfold_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "iterator_rfold_prune",
        "iterator_rfold",
        &[
            ".rfold(",
            "|mut acc, payload|",
            "payload.render_label()",
            "pub fn render_label",
        ],
        "DeadIteratorRfoldItem",
        "dead-iterator-rfold",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_iterator_try_rfold_result_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "iterator_try_rfold_result_prune",
        "iterator_try_rfold_result",
        &[
            ".try_rfold(",
            "|mut acc, payload|",
            "payload.render_label()",
            "pub fn render_label",
        ],
        "DeadIteratorTryRfoldResultItem",
        "dead-iterator-try-rfold-result",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_iterator_rfind_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "iterator_rfind_prune",
        "iterator_rfind",
        &[
            ".rfind(|payload| payload.is_match())",
            "payload.render_label()",
            "pub fn is_match",
            "pub fn render_label",
        ],
        "DeadIteratorRfindItem",
        "dead-iterator-rfind",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_iterator_position_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "iterator_position_prune",
        "iterator_position",
        &[".position(|payload| payload.is_match())", "pub fn is_match"],
        "DeadIteratorPositionItem",
        "dead-iterator-position",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_iterator_next_back_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "iterator_next_back_map_prune",
        "iterator_next_back_map",
        &[
            ".next_back()",
            "payload.render_label()",
            "pub fn render_label",
        ],
        "DeadIteratorNextBackMapItem",
        "dead-iterator-next-back-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_iterator_nth_back_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "iterator_nth_back_map_prune",
        "iterator_nth_back_map",
        &[
            ".nth_back(0)",
            "payload.render_label()",
            "pub fn render_label",
        ],
        "DeadIteratorNthBackMapItem",
        "dead-iterator-nth-back-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_iterator_max_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "iterator_max_map_prune",
        "iterator_max_map",
        &[".max()", "payload.render_label()", "pub fn render_label"],
        "DeadIteratorMaxMapItem",
        "dead-iterator-max-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_iterator_min_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "iterator_min_map_prune",
        "iterator_min_map",
        &[".min()", "payload.render_label()", "pub fn render_label"],
        "DeadIteratorMinMapItem",
        "dead-iterator-min-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_iterator_peekable_next_if_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "iterator_peekable_next_if_prune",
        "iterator_peekable_next_if",
        &[
            ".peekable()",
            ".next_if(|payload| payload.is_match())",
            "payload.render_label()",
            "pub fn is_match",
            "pub fn render_label",
        ],
        "DeadIteratorPeekableNextIfItem",
        "dead-iterator-peekable-next-if",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_iterator_peekable_peek_mut_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "iterator_peekable_peek_mut_prune",
        "iterator_peekable_peek_mut",
        &[
            ".iter_mut().peekable()",
            ".peek_mut()",
            "payload.bump_and_render()",
            "pub fn bump_and_render",
        ],
        "DeadIteratorPeekablePeekMutItem",
        "dead-iterator-peekable-peek-mut",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_vec_iter_next_back_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vec_iter_next_back_map_prune",
        "vec_iter_next_back_map",
        &[
            ".next_back()",
            "payload.render_label()",
            "pub fn render_label",
        ],
        "DeadVecIterNextBackMapItem",
        "dead-vec-iter-next-back-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_vecdeque_iter_next_back_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "vecdeque_iter_next_back_map_prune",
        "vecdeque_iter_next_back_map",
        &[
            "VecDeque::new()",
            ".next_back()",
            "payload.render_label()",
            "pub fn render_label",
        ],
        "DeadVecdequeIterNextBackMapItem",
        "dead-vecdeque-iter-next-back-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_btreemap_values_next_back_map_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "btreemap_values_next_back_map_prune",
        "btreemap_values_next_back_map",
        &[
            "BTreeMap::new()",
            ".values()",
            ".next_back()",
            "payload.render_label()",
            "pub fn render_label",
        ],
        "DeadBtreemapValuesNextBackMapItem",
        "dead-btreemap-values-next-back-map",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_slice_rchunks_for_pattern_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "slice_rchunks_for_pattern_prune",
        "slice_rchunks_for_pattern",
        &[
            "for chunk in items.rchunks(2)",
            "let [payload, ..] = chunk",
            "payload.render_label()",
            "pub fn render_label",
        ],
        "DeadSliceRchunksForPatternItem",
        "dead-slice-rchunks-for-pattern",
        "pub fn bump_and_render",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_slice_rchunks_mut_for_pattern_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "slice_rchunks_mut_for_pattern_prune",
        "slice_rchunks_mut_for_pattern",
        &[
            "for chunk in items.rchunks_mut(2)",
            "let [payload, ..] = chunk",
            "payload.bump_and_render()",
            "pub fn bump_and_render",
        ],
        "DeadSliceRchunksMutForPatternItem",
        "dead-slice-rchunks-mut-for-pattern",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_slice_chunks_exact_mut_for_pattern_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "slice_chunks_exact_mut_for_pattern_prune",
        "slice_chunks_exact_mut_for_pattern",
        &[
            "for chunk in items.chunks_exact_mut(2)",
            "let [payload, _tail] = chunk",
            "payload.bump_and_render()",
            "pub fn bump_and_render",
        ],
        "DeadSliceChunksExactMutForPatternItem",
        "dead-slice-chunks-exact-mut-for-pattern",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_slice_rchunks_exact_mut_for_pattern_support_chain_with_default_analyzer() {
    assert_extended_tuple_adapter_support_fixture(
        "slice_rchunks_exact_mut_for_pattern_prune",
        "slice_rchunks_exact_mut_for_pattern",
        &[
            "for chunk in items.rchunks_exact_mut(2)",
            "let [payload, _tail] = chunk",
            "payload.bump_and_render()",
            "pub fn bump_and_render",
        ],
        "DeadSliceRchunksExactMutForPatternItem",
        "dead-slice-rchunks-exact-mut-for-pattern",
        "pub fn render_label",
        &["pub fn unused_label"],
    );
}

#[test]
fn prunes_slice_first_chunk_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "slice_first_chunk_map_prune",
        root_fn: "selected_slice_first_chunk_map_report",
        api_pkg: "slice_first_chunk_map_api",
        model_pkg: "slice_first_chunk_map_model",
        api_fn: "selected_slice_first_chunk_map_report",
        model_fn: "selected_slice_first_chunk_map",
        model_required: &[
            ".first_chunk::<2>()",
            ".map(|[head, _tail]| head.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadSliceFirstChunkMapItem",
            "dead_slice_first_chunk_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-slice-first-chunk-map",
        ],
    });
}

#[test]
fn prunes_slice_first_chunk_mut_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "slice_first_chunk_mut_map_prune",
        root_fn: "selected_slice_first_chunk_mut_map_report",
        api_pkg: "slice_first_chunk_mut_map_api",
        model_pkg: "slice_first_chunk_mut_map_model",
        api_fn: "selected_slice_first_chunk_mut_map_report",
        model_fn: "selected_slice_first_chunk_mut_map",
        model_required: &[
            ".first_chunk_mut::<2>()",
            ".map(|[head, _tail]| head.bump_and_render())",
            "pub fn bump_and_render",
        ],
        model_absent: &[
            "mod dead",
            "DeadSliceFirstChunkMutMapItem",
            "dead_slice_first_chunk_mut_map",
            "pub fn render_label",
            "dead_method",
            "dead-slice-first-chunk-mut-map",
        ],
    });
}

#[test]
fn prunes_slice_last_chunk_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "slice_last_chunk_map_prune",
        root_fn: "selected_slice_last_chunk_map_report",
        api_pkg: "slice_last_chunk_map_api",
        model_pkg: "slice_last_chunk_map_model",
        api_fn: "selected_slice_last_chunk_map_report",
        model_fn: "selected_slice_last_chunk_map",
        model_required: &[
            ".last_chunk::<2>()",
            ".map(|[head, _tail]| head.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadSliceLastChunkMapItem",
            "dead_slice_last_chunk_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-slice-last-chunk-map",
        ],
    });
}

#[test]
fn prunes_slice_last_chunk_mut_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "slice_last_chunk_mut_map_prune",
        root_fn: "selected_slice_last_chunk_mut_map_report",
        api_pkg: "slice_last_chunk_mut_map_api",
        model_pkg: "slice_last_chunk_mut_map_model",
        api_fn: "selected_slice_last_chunk_mut_map_report",
        model_fn: "selected_slice_last_chunk_mut_map",
        model_required: &[
            ".last_chunk_mut::<2>()",
            ".map(|[head, _tail]| head.bump_and_render())",
            "pub fn bump_and_render",
        ],
        model_absent: &[
            "mod dead",
            "DeadSliceLastChunkMutMapItem",
            "dead_slice_last_chunk_mut_map",
            "pub fn render_label",
            "dead_method",
            "dead-slice-last-chunk-mut-map",
        ],
    });
}

#[test]
fn prunes_slice_split_first_chunk_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "slice_split_first_chunk_map_prune",
        root_fn: "selected_slice_split_first_chunk_map_report",
        api_pkg: "slice_split_first_chunk_map_api",
        model_pkg: "slice_split_first_chunk_map_model",
        api_fn: "selected_slice_split_first_chunk_map_report",
        model_fn: "selected_slice_split_first_chunk_map",
        model_required: &[
            ".split_first_chunk::<2>()",
            "head.render_label()",
            "tail.len()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadSliceSplitFirstChunkMapItem",
            "dead_slice_split_first_chunk_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-slice-split-first-chunk-map",
        ],
    });
}

#[test]
fn prunes_slice_split_first_chunk_mut_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "slice_split_first_chunk_mut_map_prune",
        root_fn: "selected_slice_split_first_chunk_mut_map_report",
        api_pkg: "slice_split_first_chunk_mut_map_api",
        model_pkg: "slice_split_first_chunk_mut_map_model",
        api_fn: "selected_slice_split_first_chunk_mut_map_report",
        model_fn: "selected_slice_split_first_chunk_mut_map",
        model_required: &[
            ".split_first_chunk_mut::<2>()",
            "head.bump_and_render()",
            "tail.len()",
            "pub fn bump_and_render",
        ],
        model_absent: &[
            "mod dead",
            "DeadSliceSplitFirstChunkMutMapItem",
            "dead_slice_split_first_chunk_mut_map",
            "pub fn render_label",
            "dead_method",
            "dead-slice-split-first-chunk-mut-map",
        ],
    });
}

#[test]
fn prunes_slice_split_last_chunk_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "slice_split_last_chunk_map_prune",
        root_fn: "selected_slice_split_last_chunk_map_report",
        api_pkg: "slice_split_last_chunk_map_api",
        model_pkg: "slice_split_last_chunk_map_model",
        api_fn: "selected_slice_split_last_chunk_map_report",
        model_fn: "selected_slice_split_last_chunk_map",
        model_required: &[
            ".split_last_chunk::<2>()",
            "tail.render_label()",
            "head.len()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadSliceSplitLastChunkMapItem",
            "dead_slice_split_last_chunk_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-slice-split-last-chunk-map",
        ],
    });
}

#[test]
fn prunes_slice_split_last_chunk_mut_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "slice_split_last_chunk_mut_map_prune",
        root_fn: "selected_slice_split_last_chunk_mut_map_report",
        api_pkg: "slice_split_last_chunk_mut_map_api",
        model_pkg: "slice_split_last_chunk_mut_map_model",
        api_fn: "selected_slice_split_last_chunk_mut_map_report",
        model_fn: "selected_slice_split_last_chunk_mut_map",
        model_required: &[
            ".split_last_chunk_mut::<2>()",
            "tail.bump_and_render()",
            "head.len()",
            "pub fn bump_and_render",
        ],
        model_absent: &[
            "mod dead",
            "DeadSliceSplitLastChunkMutMapItem",
            "dead_slice_split_last_chunk_mut_map",
            "pub fn render_label",
            "dead_method",
            "dead-slice-split-last-chunk-mut-map",
        ],
    });
}

#[test]
fn prunes_slice_as_chunks_iter_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "slice_as_chunks_iter_prune",
        root_fn: "selected_slice_as_chunks_iter_report",
        api_pkg: "slice_as_chunks_iter_api",
        model_pkg: "slice_as_chunks_iter_model",
        api_fn: "selected_slice_as_chunks_iter_report",
        model_fn: "selected_slice_as_chunks_iter",
        model_required: &[
            ".as_chunks::<2>()",
            ".iter()",
            ".map(|[head, _tail]| head.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadSliceAsChunksIterItem",
            "dead_slice_as_chunks_iter",
            "pub fn bump_and_render",
            "dead_method",
            "dead-slice-as-chunks-iter",
        ],
    });
}

#[test]
fn prunes_slice_as_chunks_mut_iter_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "slice_as_chunks_mut_iter_prune",
        root_fn: "selected_slice_as_chunks_mut_iter_report",
        api_pkg: "slice_as_chunks_mut_iter_api",
        model_pkg: "slice_as_chunks_mut_iter_model",
        api_fn: "selected_slice_as_chunks_mut_iter_report",
        model_fn: "selected_slice_as_chunks_mut_iter",
        model_required: &[
            ".as_chunks_mut::<2>()",
            ".iter_mut()",
            ".map(|[head, _tail]| head.bump_and_render())",
            "pub fn bump_and_render",
        ],
        model_absent: &[
            "mod dead",
            "DeadSliceAsChunksMutIterItem",
            "dead_slice_as_chunks_mut_iter",
            "pub fn render_label",
            "dead_method",
            "dead-slice-as-chunks-mut-iter",
        ],
    });
}

#[test]
fn prunes_slice_as_rchunks_iter_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "slice_as_rchunks_iter_prune",
        root_fn: "selected_slice_as_rchunks_iter_report",
        api_pkg: "slice_as_rchunks_iter_api",
        model_pkg: "slice_as_rchunks_iter_model",
        api_fn: "selected_slice_as_rchunks_iter_report",
        model_fn: "selected_slice_as_rchunks_iter",
        model_required: &[
            ".as_rchunks::<2>()",
            ".iter()",
            ".map(|[head, _tail]| head.render_label())",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadSliceAsRchunksIterItem",
            "dead_slice_as_rchunks_iter",
            "pub fn bump_and_render",
            "dead_method",
            "dead-slice-as-rchunks-iter",
        ],
    });
}

#[test]
fn prunes_slice_as_rchunks_mut_iter_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "slice_as_rchunks_mut_iter_prune",
        root_fn: "selected_slice_as_rchunks_mut_iter_report",
        api_pkg: "slice_as_rchunks_mut_iter_api",
        model_pkg: "slice_as_rchunks_mut_iter_model",
        api_fn: "selected_slice_as_rchunks_mut_iter_report",
        model_fn: "selected_slice_as_rchunks_mut_iter",
        model_required: &[
            ".as_rchunks_mut::<2>()",
            ".iter_mut()",
            ".map(|[head, _tail]| head.bump_and_render())",
            "pub fn bump_and_render",
        ],
        model_absent: &[
            "mod dead",
            "DeadSliceAsRchunksMutIterItem",
            "dead_slice_as_rchunks_mut_iter",
            "pub fn render_label",
            "dead_method",
            "dead-slice-as-rchunks-mut-iter",
        ],
    });
}

#[test]
fn prunes_slice_split_at_checked_tail_iter_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "slice_split_at_checked_tail_iter_prune",
        root_fn: "selected_slice_split_at_checked_tail_iter_report",
        api_pkg: "slice_split_at_checked_tail_iter_api",
        model_pkg: "slice_split_at_checked_tail_iter_model",
        api_fn: "selected_slice_split_at_checked_tail_iter_report",
        model_fn: "selected_slice_split_at_checked_tail_iter",
        model_required: &[
            ".split_at_checked(1)",
            "tail.iter()",
            "payload.render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadSliceSplitAtCheckedTailIterItem",
            "dead_slice_split_at_checked_tail_iter",
            "pub fn bump_and_render",
            "dead_method",
            "dead-slice-split-at-checked-tail-iter",
        ],
    });
}

#[test]
fn prunes_slice_split_at_mut_checked_tail_iter_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "slice_split_at_mut_checked_tail_iter_prune",
        root_fn: "selected_slice_split_at_mut_checked_tail_iter_report",
        api_pkg: "slice_split_at_mut_checked_tail_iter_api",
        model_pkg: "slice_split_at_mut_checked_tail_iter_model",
        api_fn: "selected_slice_split_at_mut_checked_tail_iter_report",
        model_fn: "selected_slice_split_at_mut_checked_tail_iter",
        model_required: &[
            ".split_at_mut_checked(1)",
            "tail.iter_mut()",
            "payload.bump_and_render()",
            "pub fn bump_and_render",
        ],
        model_absent: &[
            "mod dead",
            "DeadSliceSplitAtMutCheckedTailIterItem",
            "dead_slice_split_at_mut_checked_tail_iter",
            "pub fn render_label",
            "dead_method",
            "dead-slice-split-at-mut-checked-tail-iter",
        ],
    });
}

#[test]
fn prunes_option_as_pin_ref_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "option_as_pin_ref_map_prune",
        root_fn: "selected_option_as_pin_ref_map_report",
        api_pkg: "option_as_pin_ref_map_api",
        model_pkg: "option_as_pin_ref_map_model",
        api_fn: "selected_option_as_pin_ref_map_report",
        model_fn: "selected_option_as_pin_ref_map",
        model_required: &[
            "use std::pin::Pin;",
            "Pin::new(&slot)",
            ".as_pin_ref()",
            "payload.render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadOptionAsPinRefMapItem",
            "dead_option_as_pin_ref_map",
            "pub fn bump_and_render",
            "dead_method",
            "dead-option-as-pin-ref-map",
        ],
    });
}

#[test]
fn prunes_option_as_pin_mut_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "option_as_pin_mut_map_prune",
        root_fn: "selected_option_as_pin_mut_map_report",
        api_pkg: "option_as_pin_mut_map_api",
        model_pkg: "option_as_pin_mut_map_model",
        api_fn: "selected_option_as_pin_mut_map_report",
        model_fn: "selected_option_as_pin_mut_map",
        model_required: &[
            "use std::pin::Pin;",
            "Pin::new(&mut slot)",
            ".as_pin_mut()",
            "payload.bump_and_render()",
            "pub fn bump_and_render",
        ],
        model_absent: &[
            "mod dead",
            "DeadOptionAsPinMutMapItem",
            "dead_option_as_pin_mut_map",
            "pub fn render_label",
            "dead_method",
            "dead-option-as-pin-mut-map",
        ],
    });
}

#[test]
fn prunes_vecdeque_range_mut_map_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "vecdeque_range_mut_map_prune",
        root_fn: "selected_vecdeque_range_mut_map_report",
        api_pkg: "vecdeque_range_mut_map_api",
        model_pkg: "vecdeque_range_mut_map_model",
        api_fn: "selected_vecdeque_range_mut_map_report",
        model_fn: "selected_vecdeque_range_mut_map",
        model_required: &[
            "use std::collections::VecDeque;",
            ".range_mut(0..2)",
            ".map(|payload| payload.bump_and_render())",
            "pub fn bump_and_render",
        ],
        model_absent: &[
            "mod dead",
            "DeadVecdequeRangeMutMapItem",
            "dead_vecdeque_range_mut_map",
            "pub fn render_label",
            "dead_method",
            "dead-vecdeque-range-mut-map",
        ],
    });
}

#[test]
fn prunes_for_hashmap_iter_pair_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "for_hashmap_iter_pair_prune",
        root_fn: "selected_for_hashmap_iter_pair_report",
        api_pkg: "for_hashmap_iter_pair_api",
        model_pkg: "for_hashmap_iter_pair_model",
        api_fn: "selected_for_hashmap_iter_pair_report",
        model_fn: "selected_for_hashmap_iter_pair",
        model_required: &[
            "use std::collections::HashMap;",
            "for (_key, payload) in items.iter()",
            "payload.render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadForHashmapIterPairPayload",
            "dead_for_hashmap_iter_pair",
            "pub fn bump_and_render",
            "unused_label",
            "dead-for-hashmap-iter-pair",
        ],
    });
}

#[test]
fn prunes_for_hashmap_iter_mut_pair_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "for_hashmap_iter_mut_pair_prune",
        root_fn: "selected_for_hashmap_iter_mut_pair_report",
        api_pkg: "for_hashmap_iter_mut_pair_api",
        model_pkg: "for_hashmap_iter_mut_pair_model",
        api_fn: "selected_for_hashmap_iter_mut_pair_report",
        model_fn: "selected_for_hashmap_iter_mut_pair",
        model_required: &[
            "use std::collections::HashMap;",
            "for (_key, payload) in items.iter_mut()",
            "payload.bump_and_render()",
            "pub fn bump_and_render",
        ],
        model_absent: &[
            "mod dead",
            "DeadForHashmapIterMutPairPayload",
            "dead_for_hashmap_iter_mut_pair",
            "pub fn render_label",
            "unused_label",
            "dead-for-hashmap-iter-mut-pair",
        ],
    });
}

#[test]
fn prunes_for_hashmap_into_iter_pair_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "for_hashmap_into_iter_pair_prune",
        root_fn: "selected_for_hashmap_into_iter_pair_report",
        api_pkg: "for_hashmap_into_iter_pair_api",
        model_pkg: "for_hashmap_into_iter_pair_model",
        api_fn: "selected_for_hashmap_into_iter_pair_report",
        model_fn: "selected_for_hashmap_into_iter_pair",
        model_required: &[
            "use std::collections::HashMap;",
            "for (_key, payload) in items",
            "payload.render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadForHashmapIntoIterPairPayload",
            "dead_for_hashmap_into_iter_pair",
            "pub fn bump_and_render",
            "unused_label",
            "dead-for-hashmap-into-iter-pair",
        ],
    });
}

#[test]
fn prunes_for_hashmap_drain_pair_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "for_hashmap_drain_pair_prune",
        root_fn: "selected_for_hashmap_drain_pair_report",
        api_pkg: "for_hashmap_drain_pair_api",
        model_pkg: "for_hashmap_drain_pair_model",
        api_fn: "selected_for_hashmap_drain_pair_report",
        model_fn: "selected_for_hashmap_drain_pair",
        model_required: &[
            "use std::collections::HashMap;",
            "for (_key, payload) in items.drain()",
            "payload.render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadForHashmapDrainPairPayload",
            "dead_for_hashmap_drain_pair",
            "pub fn bump_and_render",
            "unused_label",
            "dead-for-hashmap-drain-pair",
        ],
    });
}

#[test]
fn prunes_for_btreemap_iter_pair_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "for_btreemap_iter_pair_prune",
        root_fn: "selected_for_btreemap_iter_pair_report",
        api_pkg: "for_btreemap_iter_pair_api",
        model_pkg: "for_btreemap_iter_pair_model",
        api_fn: "selected_for_btreemap_iter_pair_report",
        model_fn: "selected_for_btreemap_iter_pair",
        model_required: &[
            "use std::collections::BTreeMap;",
            "for (_key, payload) in items.iter()",
            "payload.render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadForBtreemapIterPairPayload",
            "dead_for_btreemap_iter_pair",
            "pub fn bump_and_render",
            "unused_label",
            "dead-for-btreemap-iter-pair",
        ],
    });
}

#[test]
fn prunes_for_btreemap_iter_mut_pair_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "for_btreemap_iter_mut_pair_prune",
        root_fn: "selected_for_btreemap_iter_mut_pair_report",
        api_pkg: "for_btreemap_iter_mut_pair_api",
        model_pkg: "for_btreemap_iter_mut_pair_model",
        api_fn: "selected_for_btreemap_iter_mut_pair_report",
        model_fn: "selected_for_btreemap_iter_mut_pair",
        model_required: &[
            "use std::collections::BTreeMap;",
            "for (_key, payload) in items.iter_mut()",
            "payload.bump_and_render()",
            "pub fn bump_and_render",
        ],
        model_absent: &[
            "mod dead",
            "DeadForBtreemapIterMutPairPayload",
            "dead_for_btreemap_iter_mut_pair",
            "pub fn render_label",
            "unused_label",
            "dead-for-btreemap-iter-mut-pair",
        ],
    });
}

#[test]
fn prunes_for_btreemap_into_iter_pair_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "for_btreemap_into_iter_pair_prune",
        root_fn: "selected_for_btreemap_into_iter_pair_report",
        api_pkg: "for_btreemap_into_iter_pair_api",
        model_pkg: "for_btreemap_into_iter_pair_model",
        api_fn: "selected_for_btreemap_into_iter_pair_report",
        model_fn: "selected_for_btreemap_into_iter_pair",
        model_required: &[
            "use std::collections::BTreeMap;",
            "for (_key, payload) in items",
            "payload.render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadForBtreemapIntoIterPairPayload",
            "dead_for_btreemap_into_iter_pair",
            "pub fn bump_and_render",
            "unused_label",
            "dead-for-btreemap-into-iter-pair",
        ],
    });
}

#[test]
fn prunes_for_btreemap_range_pair_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "for_btreemap_range_pair_prune",
        root_fn: "selected_for_btreemap_range_pair_report",
        api_pkg: "for_btreemap_range_pair_api",
        model_pkg: "for_btreemap_range_pair_model",
        api_fn: "selected_for_btreemap_range_pair_report",
        model_fn: "selected_for_btreemap_range_pair",
        model_required: &[
            "use std::collections::BTreeMap;",
            "for (_key, payload) in items.range(0..=3)",
            "payload.render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadForBtreemapRangePairPayload",
            "dead_for_btreemap_range_pair",
            "pub fn bump_and_render",
            "unused_label",
            "dead-for-btreemap-range-pair",
        ],
    });
}

#[test]
fn prunes_for_btreemap_range_mut_pair_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "for_btreemap_range_mut_pair_prune",
        root_fn: "selected_for_btreemap_range_mut_pair_report",
        api_pkg: "for_btreemap_range_mut_pair_api",
        model_pkg: "for_btreemap_range_mut_pair_model",
        api_fn: "selected_for_btreemap_range_mut_pair_report",
        model_fn: "selected_for_btreemap_range_mut_pair",
        model_required: &[
            "use std::collections::BTreeMap;",
            "for (_key, payload) in items.range_mut(0..=3)",
            "payload.bump_and_render()",
            "pub fn bump_and_render",
        ],
        model_absent: &[
            "mod dead",
            "DeadForBtreemapRangeMutPairPayload",
            "dead_for_btreemap_range_mut_pair",
            "pub fn render_label",
            "unused_label",
            "dead-for-btreemap-range-mut-pair",
        ],
    });
}

#[test]
fn prunes_for_vec_ref_payload_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "for_vec_ref_payload_prune",
        root_fn: "selected_for_vec_ref_payload_report",
        api_pkg: "for_vec_ref_payload_api",
        model_pkg: "for_vec_ref_payload_model",
        api_fn: "selected_for_vec_ref_payload_report",
        model_fn: "selected_for_vec_ref_payload",
        model_required: &[
            "for payload in &items",
            "payload.render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadForVecRefPayloadPayload",
            "dead_for_vec_ref_payload",
            "pub fn bump_and_render",
            "unused_label",
            "dead-for-vec-ref-payload",
        ],
    });
}

#[test]
fn prunes_for_vec_mut_payload_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "for_vec_mut_payload_prune",
        root_fn: "selected_for_vec_mut_payload_report",
        api_pkg: "for_vec_mut_payload_api",
        model_pkg: "for_vec_mut_payload_model",
        api_fn: "selected_for_vec_mut_payload_report",
        model_fn: "selected_for_vec_mut_payload",
        model_required: &[
            "for payload in &mut items",
            "payload.bump_and_render()",
            "pub fn bump_and_render",
        ],
        model_absent: &[
            "mod dead",
            "DeadForVecMutPayloadPayload",
            "dead_for_vec_mut_payload",
            "pub fn render_label",
            "unused_label",
            "dead-for-vec-mut-payload",
        ],
    });
}

#[test]
fn prunes_for_array_ref_payload_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "for_array_ref_payload_prune",
        root_fn: "selected_for_array_ref_payload_report",
        api_pkg: "for_array_ref_payload_api",
        model_pkg: "for_array_ref_payload_model",
        api_fn: "selected_for_array_ref_payload_report",
        model_fn: "selected_for_array_ref_payload",
        model_required: &[
            "for payload in &items",
            "payload.render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadForArrayRefPayloadPayload",
            "dead_for_array_ref_payload",
            "pub fn bump_and_render",
            "unused_label",
            "dead-for-array-ref-payload",
        ],
    });
}

#[test]
fn prunes_for_option_iter_payload_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "for_option_iter_payload_prune",
        root_fn: "selected_for_option_iter_payload_report",
        api_pkg: "for_option_iter_payload_api",
        model_pkg: "for_option_iter_payload_model",
        api_fn: "selected_for_option_iter_payload_report",
        model_fn: "selected_for_option_iter_payload",
        model_required: &[
            "for payload in maybe.iter()",
            "payload.render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadForOptionIterPayloadPayload",
            "dead_for_option_iter_payload",
            "pub fn bump_and_render",
            "unused_label",
            "dead-for-option-iter-payload",
        ],
    });
}

#[test]
fn prunes_for_option_iter_mut_payload_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "for_option_iter_mut_payload_prune",
        root_fn: "selected_for_option_iter_mut_payload_report",
        api_pkg: "for_option_iter_mut_payload_api",
        model_pkg: "for_option_iter_mut_payload_model",
        api_fn: "selected_for_option_iter_mut_payload_report",
        model_fn: "selected_for_option_iter_mut_payload",
        model_required: &[
            "for payload in maybe.iter_mut()",
            "payload.bump_and_render()",
            "pub fn bump_and_render",
        ],
        model_absent: &[
            "mod dead",
            "DeadForOptionIterMutPayloadPayload",
            "dead_for_option_iter_mut_payload",
            "pub fn render_label",
            "unused_label",
            "dead-for-option-iter-mut-payload",
        ],
    });
}

#[test]
fn prunes_for_result_iter_payload_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "for_result_iter_payload_prune",
        root_fn: "selected_for_result_iter_payload_report",
        api_pkg: "for_result_iter_payload_api",
        model_pkg: "for_result_iter_payload_model",
        api_fn: "selected_for_result_iter_payload_report",
        model_fn: "selected_for_result_iter_payload",
        model_required: &[
            "for payload in result.iter()",
            "payload.render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadForResultIterPayloadPayload",
            "dead_for_result_iter_payload",
            "pub fn bump_and_render",
            "unused_label",
            "dead-for-result-iter-payload",
        ],
    });
}

#[test]
fn prunes_for_result_iter_mut_payload_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "for_result_iter_mut_payload_prune",
        root_fn: "selected_for_result_iter_mut_payload_report",
        api_pkg: "for_result_iter_mut_payload_api",
        model_pkg: "for_result_iter_mut_payload_model",
        api_fn: "selected_for_result_iter_mut_payload_report",
        model_fn: "selected_for_result_iter_mut_payload",
        model_required: &[
            "for payload in result.iter_mut()",
            "payload.bump_and_render()",
            "pub fn bump_and_render",
        ],
        model_absent: &[
            "mod dead",
            "DeadForResultIterMutPayloadPayload",
            "dead_for_result_iter_mut_payload",
            "pub fn render_label",
            "unused_label",
            "dead-for-result-iter-mut-payload",
        ],
    });
}

#[test]
fn prunes_for_vecdeque_ref_payload_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "for_vecdeque_ref_payload_prune",
        root_fn: "selected_for_vecdeque_ref_payload_report",
        api_pkg: "for_vecdeque_ref_payload_api",
        model_pkg: "for_vecdeque_ref_payload_model",
        api_fn: "selected_for_vecdeque_ref_payload_report",
        model_fn: "selected_for_vecdeque_ref_payload",
        model_required: &[
            "use std::collections::VecDeque;",
            "for payload in &items",
            "payload.render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadForVecdequeRefPayloadPayload",
            "dead_for_vecdeque_ref_payload",
            "pub fn bump_and_render",
            "unused_label",
            "dead-for-vecdeque-ref-payload",
        ],
    });
}

#[test]
fn prunes_for_linkedlist_ref_payload_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "for_linkedlist_ref_payload_prune",
        root_fn: "selected_for_linkedlist_ref_payload_report",
        api_pkg: "for_linkedlist_ref_payload_api",
        model_pkg: "for_linkedlist_ref_payload_model",
        api_fn: "selected_for_linkedlist_ref_payload_report",
        model_fn: "selected_for_linkedlist_ref_payload",
        model_required: &[
            "use std::collections::LinkedList;",
            "for payload in &items",
            "payload.render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadForLinkedlistRefPayloadPayload",
            "dead_for_linkedlist_ref_payload",
            "pub fn bump_and_render",
            "unused_label",
            "dead-for-linkedlist-ref-payload",
        ],
    });
}

#[test]
fn prunes_for_hashset_ref_payload_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "for_hashset_ref_payload_prune",
        root_fn: "selected_for_hashset_ref_payload_report",
        api_pkg: "for_hashset_ref_payload_api",
        model_pkg: "for_hashset_ref_payload_model",
        api_fn: "selected_for_hashset_ref_payload_report",
        model_fn: "selected_for_hashset_ref_payload",
        model_required: &[
            "use std::collections::HashSet;",
            "for payload in &items",
            "payload.render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadForHashsetRefPayloadPayload",
            "dead_for_hashset_ref_payload",
            "pub fn bump_and_render",
            "unused_label",
            "dead-for-hashset-ref-payload",
        ],
    });
}

#[test]
fn prunes_for_btreeset_ref_payload_support_chain_with_default_analyzer() {
    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name: "for_btreeset_ref_payload_prune",
        root_fn: "selected_for_btreeset_ref_payload_report",
        api_pkg: "for_btreeset_ref_payload_api",
        model_pkg: "for_btreeset_ref_payload_model",
        api_fn: "selected_for_btreeset_ref_payload_report",
        model_fn: "selected_for_btreeset_ref_payload",
        model_required: &[
            "use std::collections::BTreeSet;",
            "for payload in &items",
            "payload.render_label()",
            "pub fn render_label",
        ],
        model_absent: &[
            "mod dead",
            "DeadForBtreesetRefPayloadPayload",
            "dead_for_btreeset_ref_payload",
            "pub fn bump_and_render",
            "unused_label",
            "dead-for-btreeset-ref-payload",
        ],
    });
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
    assert_extended_tuple_adapter_support_fixture(
        fixture_name,
        stem,
        required,
        dead_item,
        dead_token,
        absent_live_method,
        &[],
    );
}

fn assert_extended_tuple_adapter_support_fixture(
    fixture_name: &str,
    stem: &str,
    required: &[&str],
    dead_item: &str,
    dead_token: &str,
    absent_live_method: &str,
    extra_absent: &[&str],
) {
    let root_fn = format!("selected_{stem}_report");
    let api_pkg = format!("{stem}_api");
    let model_pkg = format!("{stem}_model");
    let api_fn = format!("selected_{stem}_report");
    let model_fn = format!("selected_{stem}");
    let dead_fn = format!("dead_{stem}");
    let mut model_absent = vec![
        "mod dead",
        dead_item,
        dead_fn.as_str(),
        absent_live_method,
        "dead_method",
        dead_token,
    ];
    model_absent.extend_from_slice(extra_absent);

    assert_support_slice_fixture(SupportSliceFixture {
        fixture_name,
        root_fn: root_fn.as_str(),
        api_pkg: api_pkg.as_str(),
        model_pkg: model_pkg.as_str(),
        api_fn: api_fn.as_str(),
        model_fn: model_fn.as_str(),
        model_required: required,
        model_absent: model_absent.as_slice(),
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

    assert_cargo_check(&output, &target_dir, &root_source, &report);
}

fn assert_cargo_check(
    workspace: &Path,
    target_dir: &Path,
    root_source: &str,
    report: &GenerateReport,
) {
    let output = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(workspace)
        .env("CARGO_TARGET_DIR", target_dir)
        .env("RUSTFLAGS", fixture_rustflags_with_denied_unused_imports())
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
    let package_roots = assert_package_inventory_contract(workspace, report);
    assert_usage_contract(&package_roots, report);
}

fn assert_package_inventory_contract(
    workspace: &Path,
    report: &GenerateReport,
) -> BTreeMap<String, PathBuf> {
    let expected = report.packages.iter().cloned().collect::<BTreeSet<_>>();
    let package_roots = collect_rendered_package_roots(workspace);
    let actual = package_roots.keys().cloned().collect::<BTreeSet<_>>();
    assert_eq!(
        actual, expected,
        "generated workspace package manifests must match reported retained packages"
    );
    package_roots
}

fn collect_rendered_package_roots(workspace: &Path) -> BTreeMap<String, PathBuf> {
    let mut manifests = Vec::new();
    collect_cargo_manifests(workspace, &mut manifests);
    let mut packages = BTreeMap::new();
    for manifest in manifests {
        let source = read(&manifest);
        let value = source.parse::<toml::Value>().unwrap_or_else(|err| {
            panic!(
                "generated manifest should parse: {}\n{err}",
                manifest.display()
            )
        });
        let Some(package) = value.get("package") else {
            continue;
        };
        let Some(name) = package.get("name").and_then(toml::Value::as_str) else {
            continue;
        };
        let package_root = manifest
            .parent()
            .expect("generated manifest should have a parent")
            .to_path_buf();
        assert!(
            packages.insert(name.to_string(), package_root).is_none(),
            "generated workspace should not contain duplicate package name {name:?}; duplicate at {}",
            manifest.display()
        );
    }
    packages
}

fn collect_cargo_manifests(root: &Path, manifests: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root).unwrap_or_else(|err| {
        panic!(
            "generated workspace directory should be readable: {}\n{err}",
            root.display()
        )
    }) {
        let entry = entry.expect("generated workspace directory entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            collect_cargo_manifests(&path, manifests);
        } else if path.file_name().and_then(|name| name.to_str()) == Some("Cargo.toml") {
            manifests.push(path);
        }
    }
}

#[cfg(feature = "ra-hir")]
fn generate_slice_case_fixture(
    fixture_name: &str,
    output_name: &str,
    target_name: &str,
    analyzer: AnalyzerMode,
) -> (PathBuf, PathBuf, GenerateReport) {
    prepare_slice_case_fixture(fixture_name);

    let fixture = repo_root().join("fixtures/slice_cases").join(fixture_name);
    let output = temp_path(output_name);
    let target_dir = temp_path(target_name);

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        analyzer,
    )
    .unwrap_or_else(|err| panic!("{fixture_name} fixture should slice with {analyzer:?}: {err}"));

    (output, target_dir, report)
}

#[cfg(feature = "ra-hir")]
fn prepare_slice_case_fixture(fixture_name: &str) {
    if fixture_name == "out_dir_generated_prune" {
        seed_out_dir_generated_source(
            "slice_case_generated.rs",
            "pub fn generated_value() -> u32 { super::out_dir_generated_helper() }\n",
        );
    }
    if fixture_name == "litter_out_dir_codegen_prune" {
        seed_out_dir_generated_source(
            "litter_codegen_bindings.rs",
            "pub fn generated_event(label: &str) -> super::GeneratedEvent { super::generated_event_helper(label) }\n",
        );
    }
}

#[cfg(feature = "ra-hir")]
fn assert_ra_pruning_proof_complete(report: &GenerateReport) {
    assert_eq!(report.analyzer.mode, AnalyzerMode::RustAnalyzerHir);
    assert!(
        report.analyzer.loaded,
        "RA analyzer should be loaded: {:?}",
        report.analyzer
    );
    let semantic_usage = report
        .analyzer
        .semantic_usage
        .as_ref()
        .expect("RA-backed slice should report semantic usage mapping");
    assert!(
        semantic_usage.reference_queries > 0,
        "RA semantic usage should query references: {:?}",
        semantic_usage
    );

    let proof = &report.usage.semantic_proof;
    assert!(
        proof.summary.analyzer_available,
        "semantic proof should record analyzer availability: {:?}",
        proof
    );
    assert!(
        proof.summary.proof_required_callables + proof.summary.proof_required_items > 0,
        "fixture should require retained-package pruning proof: {:?}",
        proof
    );
    assert_eq!(
        proof.status, "complete_for_retained_packages",
        "RA pruning proof should be complete: {:?}",
        proof
    );
    assert_eq!(
        proof.summary.unproven_callables, 0,
        "no prunable callable should remain unproven: {:?}",
        proof.unproven.callables
    );
    assert_eq!(
        proof.summary.unproven_items, 0,
        "no prunable item should remain unproven: {:?}",
        proof.unproven.items
    );
}

#[cfg(feature = "ra-hir")]
fn assert_ra_promoted_reference_edges(report: &GenerateReport) {
    let semantic_usage = report
        .analyzer
        .semantic_usage
        .as_ref()
        .expect("RA-backed slice should report semantic usage mapping");
    assert!(
        semantic_usage.callable_reference_edges + semantic_usage.item_reference_edges > 0,
        "RA semantic usage should promote reference edges: {:?}",
        semantic_usage
    );
}

#[cfg(feature = "ra-hir")]
fn assert_expected_production_hazards(
    report: &GenerateReport,
    expected: &[(&'static str, &'static str)],
) {
    let actual = report
        .production
        .hazards
        .iter()
        .map(|hazard| (hazard.code.as_str(), hazard.severity.as_str()))
        .collect::<BTreeSet<_>>();
    let expected = expected.iter().copied().collect::<BTreeSet<_>>();

    assert_eq!(
        actual, expected,
        "RA hard fixture should report exactly the expected scoped hazards"
    );
}

#[cfg(feature = "ra-hir")]
fn diagnostic_source_for_report(workspace: &Path, report: &GenerateReport) -> String {
    let mut source = String::new();
    for package in &report.packages {
        let path = workspace.join(package).join("src/lib.rs");
        if !path.exists() {
            continue;
        }
        source.push_str("\n// ");
        source.push_str(&path.display().to_string());
        source.push('\n');
        source.push_str(&read(path));
    }
    source
}

fn fixture_rustflags_with_denied_unused_imports() -> String {
    match std::env::var("RUSTFLAGS") {
        Ok(flags) if !rustflags_deny_lint(&flags, "unused-imports") => {
            format!("{flags} -D unused-imports")
        }
        Ok(flags) => flags,
        Err(_) => "-D unused-imports".to_string(),
    }
}

fn rustflags_deny_lint(flags: &str, lint: &str) -> bool {
    let mut previous = "";
    for flag in flags.split_whitespace() {
        if flag == format!("-D{lint}") || (previous == "-D" && flag == lint) {
            return true;
        }
        previous = flag;
    }
    false
}

fn assert_production_hazard(report: &GenerateReport, code: &str, severity: &str) {
    assert!(
        report
            .production
            .hazards
            .iter()
            .any(|hazard| hazard.code == code && hazard.severity == severity),
        "expected production hazard {code:?} with severity {severity:?}: {:?}",
        report.production.hazards
    );
}

fn assert_no_production_hazard(report: &GenerateReport, code: &str) {
    assert!(
        report
            .production
            .hazards
            .iter()
            .all(|hazard| hazard.code != code),
        "expected no production hazard {code:?}: {:?}",
        report.production.hazards
    );
}

fn assert_usage_contract(package_roots: &BTreeMap<String, PathBuf>, report: &GenerateReport) {
    let reachable_callables = report
        .reachable
        .iter()
        .map(ToString::to_string)
        .collect::<BTreeSet<_>>();
    let reachable_items = report
        .reachable_items
        .iter()
        .map(ToString::to_string)
        .collect::<BTreeSet<_>>();
    let used_callables = report
        .usage
        .used
        .callables
        .iter()
        .map(ToString::to_string)
        .collect::<BTreeSet<_>>();
    let used_items = report
        .usage
        .used
        .items
        .iter()
        .map(ToString::to_string)
        .collect::<BTreeSet<_>>();
    let blocked_callables = report
        .usage
        .blocked_by_unknown
        .callables
        .iter()
        .map(ToString::to_string)
        .collect::<BTreeSet<_>>();
    let blocked_items = report
        .usage
        .blocked_by_unknown
        .items
        .iter()
        .map(ToString::to_string)
        .collect::<BTreeSet<_>>();
    let prunable_callables = report
        .usage
        .prunable
        .callables
        .iter()
        .map(ToString::to_string)
        .collect::<BTreeSet<_>>();
    let prunable_items = report
        .usage
        .prunable
        .items
        .iter()
        .map(ToString::to_string)
        .collect::<BTreeSet<_>>();
    let unused_callables = report
        .usage
        .unused
        .callables
        .iter()
        .map(ToString::to_string)
        .collect::<BTreeSet<_>>();
    let unused_items = report
        .usage
        .unused
        .items
        .iter()
        .map(ToString::to_string)
        .collect::<BTreeSet<_>>();

    let retained_callables = used_callables
        .union(&blocked_callables)
        .cloned()
        .collect::<BTreeSet<_>>();
    let retained_items = used_items
        .union(&blocked_items)
        .cloned()
        .collect::<BTreeSet<_>>();

    assert_eq!(
        retained_callables, reachable_callables,
        "every rendered callable must be classified as used or blocked_by_unknown"
    );
    assert_eq!(
        retained_items, reachable_items,
        "every rendered item must be classified as used or blocked_by_unknown"
    );
    assert!(
        prunable_callables.is_disjoint(&reachable_callables),
        "prunable callables must not be reachable/rendered: {:?}",
        prunable_callables
            .intersection(&reachable_callables)
            .collect::<Vec<_>>()
    );
    assert!(
        prunable_items.is_disjoint(&reachable_items),
        "prunable items must not be reachable/rendered: {:?}",
        prunable_items
            .intersection(&reachable_items)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        unused_callables, prunable_callables,
        "public usage.unused must contain only the removable prunable callables"
    );
    assert_eq!(
        unused_items, prunable_items,
        "public usage.unused must contain only the removable prunable items"
    );

    let rendered = collect_rendered_symbols(package_roots, &report.packages);
    let rendered_decisions = &report.usage.rendered_decision_map;
    let rendered_decision_callables = rendered_decisions
        .callables
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    let rendered_decision_items = rendered_decisions
        .items
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    assert_eq!(
        rendered_decision_callables, rendered.callables,
        "final rendered callable decision map must match generated source symbols"
    );
    assert_eq!(
        rendered_decision_items, rendered.items,
        "final rendered item decision map must match generated source symbols"
    );
    assert_rendered_decisions_only_used_or_unknown("callables", &rendered_decisions.callables);
    assert_rendered_decisions_only_used_or_unknown("items", &rendered_decisions.items);
    assert_rendered_decisions_only_used_or_unknown("members", &rendered_decisions.members);
    assert_rendered_decisions_only_used_or_unknown("assoc_items", &rendered_decisions.assoc_items);
    assert_rendered_decisions_only_used_or_unknown(
        "trait_default_methods",
        &rendered_decisions.trait_default_methods,
    );

    let retained_rendered_items = retained_items
        .union(&rendered.structural_module_items)
        .cloned()
        .collect::<BTreeSet<_>>();
    let non_structural_rendered_items = rendered
        .items
        .difference(&rendered.structural_module_items)
        .cloned()
        .collect::<BTreeSet<_>>();
    let unclassified_rendered_callables = rendered
        .callables
        .difference(&retained_callables)
        .collect::<Vec<_>>();
    let unclassified_rendered_items = rendered
        .items
        .difference(&retained_rendered_items)
        .collect::<Vec<_>>();
    assert!(
        unclassified_rendered_callables.is_empty(),
        "generated source declares callables that are not classified as used or blocked_by_unknown: {:?}",
        unclassified_rendered_callables
    );
    assert!(
        unclassified_rendered_items.is_empty(),
        "generated source declares items that are not classified as used or blocked_by_unknown: {:?}",
        unclassified_rendered_items
    );
    assert!(
        rendered.callables.is_disjoint(&prunable_callables),
        "generated source still declares prunable callables: {:?}",
        rendered
            .callables
            .intersection(&prunable_callables)
            .collect::<Vec<_>>()
    );
    assert!(
        non_structural_rendered_items.is_disjoint(&prunable_items),
        "generated source still declares prunable items: {:?}",
        non_structural_rendered_items
            .intersection(&prunable_items)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        report.usage.rendered_symbols.summary.prunable_members, 0,
        "generated source still declares struct fields or enum variants classified as prunable: {:?}",
        report
            .usage
            .rendered_symbols
            .entries
            .iter()
            .filter(|entry| entry.kind == "member" && entry.classification == "prunable")
            .collect::<Vec<_>>()
    );
    assert_eq!(
        report.usage.rendered_symbols.summary.unclassified_members, 0,
        "generated source declares struct fields or enum variants missing used/unknown classification: {:?}",
        report
            .usage
            .rendered_symbols
            .entries
            .iter()
            .filter(|entry| entry.kind == "member" && entry.classification == "unclassified")
            .collect::<Vec<_>>()
    );
    assert_eq!(
        report.usage.rendered_symbols.summary.prunable_assoc_items,
        0,
        "generated source still declares associated items classified as prunable: {:?}",
        report
            .usage
            .rendered_symbols
            .entries
            .iter()
            .filter(|entry| entry.kind == "assoc_item" && entry.classification == "prunable")
            .collect::<Vec<_>>()
    );
    assert_eq!(
        report
            .usage
            .rendered_symbols
            .summary
            .unclassified_assoc_items,
        0,
        "generated source declares associated items missing used/unknown classification: {:?}",
        report
            .usage
            .rendered_symbols
            .entries
            .iter()
            .filter(|entry| entry.kind == "assoc_item" && entry.classification == "unclassified")
            .collect::<Vec<_>>()
    );
    assert_public_reexport_contract(
        &rendered,
        &retained_callables,
        &retained_items,
        &prunable_callables,
        &prunable_items,
    );

    let retained_trait_default_methods = retained_trait_default_methods(&rendered, &blocked_items);
    let unproven_trait_default_methods = rendered
        .trait_default_methods
        .difference(&retained_trait_default_methods)
        .collect::<Vec<_>>();
    assert!(
        unproven_trait_default_methods.is_empty(),
        "generated source retained trait default methods that are not referenced by retained code \
         and not blocked_by_unknown: {:?}",
        unproven_trait_default_methods
    );
}

fn assert_rendered_decisions_only_used_or_unknown(
    label: &str,
    decisions: &BTreeMap<String, String>,
) {
    let invalid = decisions
        .iter()
        .filter(|(_, decision)| {
            decision.as_str() != "used" && decision.as_str() != "blocked_by_unknown"
        })
        .collect::<Vec<_>>();
    assert!(
        invalid.is_empty(),
        "final rendered {label} decision map must contain only used or blocked_by_unknown entries: {:?}",
        invalid
    );
}

#[derive(Default)]
struct RenderedSymbols {
    callables: BTreeSet<String>,
    items: BTreeSet<String>,
    module_items: BTreeSet<String>,
    structural_module_items: BTreeSet<String>,
    module_paths: BTreeSet<Vec<String>>,
    public_reexports: BTreeSet<RenderedPublicReexport>,
    trait_default_methods: BTreeSet<RenderedTraitDefaultMethod>,
    direct_call_references: BTreeSet<String>,
    trait_default_method_references: BTreeMap<RenderedTraitDefaultMethod, BTreeSet<String>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct RenderedPublicReexport {
    package: String,
    module_path: Vec<String>,
    visible: String,
    target: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct RenderedTraitDefaultMethod {
    trait_item: String,
    method: String,
}

#[cfg(feature = "ra-hir")]
struct RaHardFixture {
    fixture: &'static str,
    packages: &'static [&'static str],
    hazards: &'static [(&'static str, &'static str)],
}

fn collect_rendered_symbols(
    package_roots: &BTreeMap<String, PathBuf>,
    packages: &[String],
) -> RenderedSymbols {
    let mut symbols = RenderedSymbols::default();
    for package in packages {
        let source_root = package_roots
            .get(package)
            .unwrap_or_else(|| panic!("reported package {package:?} should have a manifest"))
            .join("src");
        if !source_root.exists() {
            continue;
        }
        for file in rust_files_under(&source_root) {
            let module_path = module_path_from_source_file(&source_root, &file);
            insert_module_path_with_parents(&mut symbols.module_paths, &module_path);
            let source = read(&file);
            let syntax = syn::parse_file(&source).unwrap_or_else(|err| {
                panic!("generated source should parse: {}\n{err}", file.display())
            });
            let aliases = rendered_aliases_from_items(&syntax.items, None);
            collect_rendered_items(package, &module_path, &syntax.items, &aliases, &mut symbols);
        }
    }
    symbols
}

fn collect_rendered_items(
    package: &str,
    module_path: &[String],
    items: &[syn::Item],
    aliases: &BTreeMap<String, Vec<String>>,
    symbols: &mut RenderedSymbols,
) {
    for item in items {
        match item {
            syn::Item::Fn(function) => {
                record_rendered_structural_modules(package, module_path, symbols);
                symbols.callables.insert(symbol_path(
                    package,
                    module_path,
                    &function.sig.ident.to_string(),
                ));
                symbols
                    .direct_call_references
                    .extend(call_references_in_block(&function.block));
            }
            syn::Item::Struct(item) => {
                symbols.items.insert(item_symbol_path(
                    package,
                    module_path,
                    &item.ident.to_string(),
                    "Struct",
                ));
                record_rendered_structural_modules(package, module_path, symbols);
            }
            syn::Item::Enum(item) => {
                symbols.items.insert(item_symbol_path(
                    package,
                    module_path,
                    &item.ident.to_string(),
                    "Enum",
                ));
                record_rendered_structural_modules(package, module_path, symbols);
            }
            syn::Item::Union(item) => {
                symbols.items.insert(item_symbol_path(
                    package,
                    module_path,
                    &item.ident.to_string(),
                    "Union",
                ));
                record_rendered_structural_modules(package, module_path, symbols);
            }
            syn::Item::Type(item) => {
                symbols.items.insert(item_symbol_path(
                    package,
                    module_path,
                    &item.ident.to_string(),
                    "Type",
                ));
                record_rendered_structural_modules(package, module_path, symbols);
            }
            syn::Item::Trait(item) => {
                record_rendered_structural_modules(package, module_path, symbols);
                let trait_item =
                    item_symbol_path(package, module_path, &item.ident.to_string(), "Trait");
                symbols.items.insert(item_symbol_path(
                    package,
                    module_path,
                    &item.ident.to_string(),
                    "Trait",
                ));
                for trait_item_fn in &item.items {
                    let syn::TraitItem::Fn(method) = trait_item_fn else {
                        continue;
                    };
                    let Some(block) = &method.default else {
                        continue;
                    };
                    let default_method = RenderedTraitDefaultMethod {
                        trait_item: trait_item.clone(),
                        method: method.sig.ident.to_string(),
                    };
                    symbols.trait_default_methods.insert(default_method.clone());
                    symbols
                        .trait_default_method_references
                        .insert(default_method, call_references_in_block(block));
                }
            }
            syn::Item::Const(item) => {
                symbols.items.insert(item_symbol_path(
                    package,
                    module_path,
                    &item.ident.to_string(),
                    "Const",
                ));
                record_rendered_structural_modules(package, module_path, symbols);
            }
            syn::Item::Static(item) => {
                symbols.items.insert(item_symbol_path(
                    package,
                    module_path,
                    &item.ident.to_string(),
                    "Static",
                ));
                record_rendered_structural_modules(package, module_path, symbols);
            }
            syn::Item::Macro(item) => {
                if let Some(ident) = &item.ident {
                    symbols.items.insert(item_symbol_path(
                        package,
                        module_path,
                        &ident.to_string(),
                        "Macro",
                    ));
                    record_rendered_structural_modules(package, module_path, symbols);
                }
                if item_macro_is_source_include(item) {
                    record_rendered_structural_modules(package, module_path, symbols);
                }
            }
            syn::Item::Mod(item) => {
                let name = item.ident.to_string();
                let module_item = item_symbol_path(package, module_path, &name, "Mod");
                symbols.items.insert(module_item.clone());
                symbols.module_items.insert(module_item);
                if let Some((_, nested)) = &item.content {
                    let mut nested_module_path = module_path.to_vec();
                    nested_module_path.push(name);
                    insert_module_path_with_parents(&mut symbols.module_paths, &nested_module_path);
                    let nested_aliases = rendered_aliases_from_items(nested, Some(aliases));
                    collect_rendered_items(
                        package,
                        &nested_module_path,
                        nested,
                        &nested_aliases,
                        symbols,
                    );
                }
            }
            syn::Item::Use(item) => {
                if is_public_visibility(&item.vis) {
                    record_rendered_structural_modules(package, module_path, symbols);
                    collect_rendered_public_reexports(
                        package,
                        module_path,
                        &item.tree,
                        Vec::new(),
                        symbols,
                    );
                }
            }
            syn::Item::Impl(item) => {
                if let Some(type_path) =
                    rendered_impl_type_path(module_path, &item.self_ty, aliases)
                {
                    let trait_path = item
                        .trait_
                        .as_ref()
                        .map(|(_, path, _)| rendered_normalized_path(module_path, path, aliases));
                    let trait_input_type_paths = item
                        .trait_
                        .as_ref()
                        .map(|(_, path, _)| {
                            rendered_trait_input_type_paths(module_path, path, aliases)
                        })
                        .unwrap_or_default();
                    for impl_item in &item.items {
                        if let syn::ImplItem::Fn(method) = impl_item {
                            record_rendered_structural_modules(package, module_path, symbols);
                            symbols.callables.insert(rendered_method_symbol_path(
                                package,
                                &type_path,
                                trait_path.as_deref(),
                                &trait_input_type_paths,
                                &method.sig.ident.to_string(),
                            ));
                            symbols
                                .direct_call_references
                                .extend(call_references_in_block(&method.block));
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

fn record_rendered_structural_modules(
    package: &str,
    module_path: &[String],
    symbols: &mut RenderedSymbols,
) {
    symbols
        .structural_module_items
        .extend(rendered_module_item_parents(package, module_path));
}

fn rendered_module_item_parents(package: &str, module_path: &[String]) -> BTreeSet<String> {
    let mut modules = BTreeSet::new();
    for index in 1..=module_path.len() {
        let parent = &module_path[..index - 1];
        let name = &module_path[index - 1];
        modules.insert(item_symbol_path(package, parent, name, "Mod"));
    }
    modules
}

fn rendered_aliases_from_items(
    items: &[syn::Item],
    parent: Option<&BTreeMap<String, Vec<String>>>,
) -> BTreeMap<String, Vec<String>> {
    let mut aliases = parent.cloned().unwrap_or_default();
    for item in items {
        if let syn::Item::Use(item_use) = item {
            collect_rendered_use_tree(&item_use.tree, Vec::new(), &mut aliases);
        }
    }
    aliases
}

fn collect_rendered_use_tree(
    tree: &syn::UseTree,
    mut prefix: Vec<String>,
    aliases: &mut BTreeMap<String, Vec<String>>,
) {
    match tree {
        syn::UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            collect_rendered_use_tree(&path.tree, prefix, aliases);
        }
        syn::UseTree::Name(name) => {
            let ident = name.ident.to_string();
            let mut target = prefix;
            target.push(ident.clone());
            aliases.insert(ident, target);
        }
        syn::UseTree::Rename(rename) => {
            let mut target = prefix;
            target.push(rename.ident.to_string());
            aliases.insert(rename.rename.to_string(), target);
        }
        syn::UseTree::Group(group) => {
            for nested in &group.items {
                collect_rendered_use_tree(nested, prefix.clone(), aliases);
            }
        }
        syn::UseTree::Glob(_) => {}
    }
}

fn collect_rendered_public_reexports(
    package: &str,
    module_path: &[String],
    tree: &syn::UseTree,
    mut prefix: Vec<String>,
    symbols: &mut RenderedSymbols,
) {
    match tree {
        syn::UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            collect_rendered_public_reexports(package, module_path, &path.tree, prefix, symbols);
        }
        syn::UseTree::Name(name) => {
            if let Some((visible, target)) = rendered_reexport_leaf(prefix, name.ident.to_string())
            {
                symbols.public_reexports.insert(RenderedPublicReexport {
                    package: package.to_string(),
                    module_path: module_path.to_vec(),
                    visible,
                    target,
                });
            }
        }
        syn::UseTree::Rename(rename) => {
            if let Some((_, target)) = rendered_reexport_leaf(prefix, rename.ident.to_string()) {
                symbols.public_reexports.insert(RenderedPublicReexport {
                    package: package.to_string(),
                    module_path: module_path.to_vec(),
                    visible: rename.rename.to_string(),
                    target,
                });
            }
        }
        syn::UseTree::Group(group) => {
            for nested in &group.items {
                collect_rendered_public_reexports(
                    package,
                    module_path,
                    nested,
                    prefix.clone(),
                    symbols,
                );
            }
        }
        syn::UseTree::Glob(_) => {}
    }
}

fn rendered_reexport_leaf(mut prefix: Vec<String>, ident: String) -> Option<(String, Vec<String>)> {
    if ident == "self" {
        let visible = prefix.last()?.clone();
        Some((visible, prefix))
    } else {
        prefix.push(ident.clone());
        Some((ident, prefix))
    }
}

fn item_macro_is_source_include(item: &syn::ItemMacro) -> bool {
    item.mac.path.is_ident("include")
}

fn is_public_visibility(visibility: &syn::Visibility) -> bool {
    matches!(visibility, syn::Visibility::Public(_))
}

fn insert_module_path_with_parents(modules: &mut BTreeSet<Vec<String>>, module_path: &[String]) {
    modules.insert(Vec::new());
    for index in 1..=module_path.len() {
        modules.insert(module_path[..index].to_vec());
    }
}

fn assert_public_reexport_contract(
    rendered: &RenderedSymbols,
    retained_callables: &BTreeSet<String>,
    retained_items: &BTreeSet<String>,
    prunable_callables: &BTreeSet<String>,
    prunable_items: &BTreeSet<String>,
) {
    let retained_symbols = symbol_base_paths(retained_callables, retained_items);
    let prunable_symbols = symbol_base_paths(prunable_callables, prunable_items);
    let known_symbols = retained_symbols
        .union(&prunable_symbols)
        .cloned()
        .collect::<BTreeSet<_>>();
    let exposed_reexports = rendered_exposed_reexports(rendered);
    let mut stale_reexports = Vec::new();
    let mut unclassified_reexports = Vec::new();

    for reexport in &rendered.public_reexports {
        let candidates = public_reexport_target_closure(
            reexport,
            &rendered.module_paths,
            &known_symbols,
            &exposed_reexports,
        );
        if candidates
            .iter()
            .any(|candidate| retained_symbols.contains(candidate))
        {
            continue;
        }

        let prunable_hits = candidates
            .iter()
            .filter(|candidate| prunable_symbols.contains(*candidate))
            .cloned()
            .collect::<Vec<_>>();
        if !prunable_hits.is_empty() {
            stale_reexports.push((reexport.clone(), prunable_hits));
        } else if !candidates.is_empty() {
            unclassified_reexports.push((reexport.clone(), candidates));
        }
    }

    assert!(
        stale_reexports.is_empty(),
        "generated source exposes public reexports to prunable local targets: {:?}",
        stale_reexports
    );
    assert!(
        unclassified_reexports.is_empty(),
        "generated source exposes public reexports to unclassified local targets: {:?}",
        unclassified_reexports
    );
}

fn rendered_exposed_reexports(
    rendered: &RenderedSymbols,
) -> BTreeMap<String, Vec<RenderedPublicReexport>> {
    let mut reexports = BTreeMap::<String, Vec<RenderedPublicReexport>>::new();
    for reexport in &rendered.public_reexports {
        let mut exposed_path = reexport.module_path.clone();
        exposed_path.push(reexport.visible.clone());
        reexports
            .entry(qualified_symbol_path(&reexport.package, &exposed_path))
            .or_default()
            .push(reexport.clone());
    }
    reexports
}

fn public_reexport_target_closure(
    reexport: &RenderedPublicReexport,
    module_paths: &BTreeSet<Vec<String>>,
    known_symbols: &BTreeSet<String>,
    exposed_reexports: &BTreeMap<String, Vec<RenderedPublicReexport>>,
) -> BTreeSet<String> {
    let mut candidates = BTreeSet::new();
    let mut pending = public_reexport_local_candidates(reexport, module_paths, known_symbols)
        .into_iter()
        .collect::<Vec<_>>();
    if let Some(target) = exposed_raw_reexport_target(&reexport.target, exposed_reexports) {
        pending.push(target);
    }

    while let Some(candidate) = pending.pop() {
        if !candidates.insert(candidate.clone()) {
            continue;
        }
        let Some(reexports) = exposed_reexports.get(&candidate) else {
            continue;
        };
        for exposed in reexports {
            for next in public_reexport_local_candidates(exposed, module_paths, known_symbols) {
                if !candidates.contains(&next) {
                    pending.push(next);
                }
            }
            if let Some(next) = exposed_raw_reexport_target(&exposed.target, exposed_reexports) {
                if !candidates.contains(&next) {
                    pending.push(next);
                }
            }
        }
    }

    candidates
}

fn symbol_base_paths(callables: &BTreeSet<String>, items: &BTreeSet<String>) -> BTreeSet<String> {
    let mut paths = callables.clone();
    paths.extend(items.iter().map(|item| item_base_path(item)));
    paths
}

fn item_base_path(item: &str) -> String {
    item.rfind('(')
        .map(|index| item[..index].to_string())
        .unwrap_or_else(|| item.to_string())
}

fn public_reexport_local_candidates(
    reexport: &RenderedPublicReexport,
    module_paths: &BTreeSet<Vec<String>>,
    known_symbols: &BTreeSet<String>,
) -> BTreeSet<String> {
    let mut candidates = BTreeSet::new();
    if let Some(target) = known_raw_reexport_target(&reexport.target, known_symbols) {
        candidates.insert(target);
    }
    if let Some(local_segments) =
        normalize_public_reexport_target(&reexport.module_path, &reexport.target)
    {
        let explicit_local = reexport
            .target
            .first()
            .is_some_and(|segment| segment == "crate" || segment == "self" || segment == "super");
        if local_reexport_candidate_is_local(
            &reexport.package,
            &local_segments,
            module_paths,
            known_symbols,
            explicit_local,
        ) {
            candidates.insert(qualified_symbol_path(&reexport.package, &local_segments));
        }
    }

    if reexport
        .target
        .first()
        .is_some_and(|segment| segment != "crate" && segment != "self" && segment != "super")
        && local_reexport_candidate_is_local(
            &reexport.package,
            &reexport.target,
            module_paths,
            known_symbols,
            false,
        )
    {
        candidates.insert(qualified_symbol_path(&reexport.package, &reexport.target));
    }

    candidates
}

fn known_raw_reexport_target(
    target: &[String],
    known_symbols: &BTreeSet<String>,
) -> Option<String> {
    let path = raw_reexport_target(target)?;
    known_symbols.contains(&path).then_some(path)
}

fn exposed_raw_reexport_target(
    target: &[String],
    exposed_reexports: &BTreeMap<String, Vec<RenderedPublicReexport>>,
) -> Option<String> {
    let path = raw_reexport_target(target)?;
    exposed_reexports.contains_key(&path).then_some(path)
}

fn raw_reexport_target(target: &[String]) -> Option<String> {
    if target.is_empty() {
        return None;
    }
    Some(target.join("::"))
}

fn local_reexport_candidate_is_local(
    package: &str,
    segments: &[String],
    module_paths: &BTreeSet<Vec<String>>,
    known_symbols: &BTreeSet<String>,
    explicit_local: bool,
) -> bool {
    if segments.is_empty() {
        return false;
    }
    let symbol = qualified_symbol_path(package, segments);
    if known_symbols.contains(&symbol) || module_paths.contains(segments) {
        return true;
    }
    if explicit_local {
        return true;
    }
    if segments.len() == 1 {
        return false;
    }
    let parent = &segments[..segments.len() - 1];
    module_paths.contains(parent)
}

fn normalize_public_reexport_target(
    module_path: &[String],
    target: &[String],
) -> Option<Vec<String>> {
    let first = target.first()?;
    if first == "crate" {
        return Some(target[1..].to_vec());
    }

    let mut normalized = module_path.to_vec();
    let mut index = 0;
    if first == "self" {
        index = 1;
    } else {
        while target.get(index).is_some_and(|segment| segment == "super") {
            normalized.pop();
            index += 1;
        }
    }
    normalized.extend_from_slice(&target[index..]);
    Some(normalized)
}

fn qualified_symbol_path(package: &str, segments: &[String]) -> String {
    let mut path = package.to_string();
    for segment in segments {
        path.push_str("::");
        path.push_str(segment);
    }
    path
}

fn retained_trait_default_methods(
    rendered: &RenderedSymbols,
    blocked_items: &BTreeSet<String>,
) -> BTreeSet<RenderedTraitDefaultMethod> {
    let mut retained = BTreeSet::new();
    let mut referenced_methods = rendered.direct_call_references.clone();

    loop {
        let before = retained.len();
        for method in &rendered.trait_default_methods {
            if blocked_items.contains(&method.trait_item)
                || referenced_methods.contains(&method.method)
            {
                retained.insert(method.clone());
                if let Some(references) = rendered.trait_default_method_references.get(method) {
                    referenced_methods.extend(references.iter().cloned());
                }
            }
        }
        if retained.len() == before {
            break;
        }
    }

    retained
}

fn call_references_in_block(block: &syn::Block) -> BTreeSet<String> {
    let mut collector = CallReferenceCollector::default();
    syn::visit::Visit::visit_block(&mut collector, block);
    collector.references
}

#[derive(Default)]
struct CallReferenceCollector {
    references: BTreeSet<String>,
}

impl<'ast> syn::visit::Visit<'ast> for CallReferenceCollector {
    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        self.references.insert(node.method.to_string());
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let syn::Expr::Path(path) = node.func.as_ref() {
            if let Some(segment) = path.path.segments.last() {
                self.references.insert(segment.ident.to_string());
            }
        }
        syn::visit::visit_expr_call(self, node);
    }
}

fn rust_files_under(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_rust_files(root, &mut files);
    files.sort();
    files
}

fn collect_rust_files(root: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root).unwrap_or_else(|err| {
        panic!(
            "generated source directory should be readable: {}\n{err}",
            root.display()
        )
    }) {
        let entry = entry.expect("generated source directory entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, files);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}

fn module_path_from_source_file(source_root: &Path, file: &Path) -> Vec<String> {
    let relative = file
        .strip_prefix(source_root)
        .expect("generated Rust file should live below source root");
    let mut components = relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>();
    let Some(file_name) = components.pop() else {
        return Vec::new();
    };
    match file_name.as_str() {
        "lib.rs" | "main.rs" => Vec::new(),
        "mod.rs" => components,
        _ => {
            let stem = Path::new(&file_name)
                .file_stem()
                .expect("Rust source file should have a stem")
                .to_string_lossy()
                .to_string();
            components.push(stem);
            components
        }
    }
}

fn rendered_impl_type_path(
    module_path: &[String],
    ty: &syn::Type,
    aliases: &BTreeMap<String, Vec<String>>,
) -> Option<Vec<String>> {
    let syn::Type::Path(path) = ty else {
        return None;
    };
    if path.qself.is_some() {
        return None;
    }
    rendered_normalized_type_path(module_path, ty, aliases)
}

fn rendered_normalized_path(
    module_path: &[String],
    path: &syn::Path,
    aliases: &BTreeMap<String, Vec<String>>,
) -> Vec<String> {
    let segments = rendered_apply_alias(
        path.segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect(),
        aliases,
    );
    rendered_normalize_segments(module_path, segments).unwrap_or_default()
}

fn rendered_normalized_type_path(
    module_path: &[String],
    ty: &syn::Type,
    aliases: &BTreeMap<String, Vec<String>>,
) -> Option<Vec<String>> {
    let syn::Type::Path(type_path) = ty else {
        return None;
    };
    let segments = rendered_apply_alias(
        type_path
            .path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect(),
        aliases,
    );
    rendered_normalize_segments(module_path, segments)
}

fn rendered_normalize_segments(
    module_path: &[String],
    segments: Vec<String>,
) -> Option<Vec<String>> {
    let Some(first) = segments.first() else {
        return None;
    };
    if first == "crate" {
        return Some(segments[1..].to_vec());
    }
    if first == "self" {
        let mut path = module_path.to_vec();
        path.extend_from_slice(&segments[1..]);
        return Some(path);
    }
    if first == "super" {
        let mut path = module_path.to_vec();
        path.pop();
        path.extend_from_slice(&segments[1..]);
        return Some(path);
    }

    let mut path = module_path.to_vec();
    path.extend(segments);
    Some(path)
}

fn rendered_apply_alias(
    mut segments: Vec<String>,
    aliases: &BTreeMap<String, Vec<String>>,
) -> Vec<String> {
    let Some(first) = segments.first() else {
        return segments;
    };
    let Some(target) = aliases.get(first) else {
        return segments;
    };
    let mut resolved = target.clone();
    resolved.extend(segments.drain(1..));
    resolved
}

fn rendered_trait_input_type_paths(
    module_path: &[String],
    path: &syn::Path,
    aliases: &BTreeMap<String, Vec<String>>,
) -> Vec<Vec<String>> {
    let mut type_paths = Vec::new();
    for segment in &path.segments {
        if let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments {
            for argument in &arguments.args {
                if let syn::GenericArgument::Type(ty) = argument {
                    collect_rendered_type_paths(module_path, ty, aliases, &mut type_paths);
                }
            }
        }
    }
    type_paths.sort();
    type_paths.dedup();
    type_paths
}

fn collect_rendered_type_paths(
    module_path: &[String],
    ty: &syn::Type,
    aliases: &BTreeMap<String, Vec<String>>,
    type_paths: &mut Vec<Vec<String>>,
) {
    match ty {
        syn::Type::Path(type_path) => {
            if let Some(path) = rendered_normalized_type_path(module_path, ty, aliases) {
                type_paths.push(path);
            }
            for segment in &type_path.path.segments {
                if let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments {
                    for argument in &arguments.args {
                        if let syn::GenericArgument::Type(ty) = argument {
                            collect_rendered_type_paths(module_path, ty, aliases, type_paths);
                        }
                    }
                }
            }
        }
        syn::Type::Reference(reference) => {
            collect_rendered_type_paths(module_path, &reference.elem, aliases, type_paths);
        }
        _ => {}
    }
}

fn symbol_path(package: &str, module_path: &[String], name: &str) -> String {
    let mut path = package.to_string();
    for segment in module_path {
        path.push_str("::");
        path.push_str(segment);
    }
    path.push_str("::");
    path.push_str(name);
    path
}

fn rendered_method_symbol_path(
    package: &str,
    type_path: &[String],
    trait_path: Option<&[String]>,
    trait_input_type_paths: &[Vec<String>],
    method: &str,
) -> String {
    if let Some(trait_path) = trait_path {
        let mut path = format!("{package}::<");
        push_segments(&mut path, type_path);
        path.push_str(" as ");
        push_segments(&mut path, trait_path);
        if !trait_input_type_paths.is_empty() {
            path.push('<');
            for (index, input_path) in trait_input_type_paths.iter().enumerate() {
                if index > 0 {
                    path.push_str(", ");
                }
                push_segments(&mut path, input_path);
            }
            path.push('>');
        }
        path.push_str(">::");
        path.push_str(method);
        path
    } else {
        method_symbol_path(package, type_path, method)
    }
}

fn push_segments(output: &mut String, segments: &[String]) {
    for (index, segment) in segments.iter().enumerate() {
        if index > 0 {
            output.push_str("::");
        }
        output.push_str(segment);
    }
}

fn method_symbol_path(package: &str, type_path: &[String], method: &str) -> String {
    let mut path = package.to_string();
    for segment in type_path {
        path.push_str("::");
        path.push_str(segment);
    }
    path.push_str("::");
    path.push_str(method);
    path
}

fn item_symbol_path(package: &str, module_path: &[String], name: &str, kind: &str) -> String {
    format!("{}({kind})", symbol_path(package, module_path, name))
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

fn seed_out_dir_generated_source(file_name: &str, source: &str) {
    let out_dir = std::env::var_os("OUT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let path = temp_path("slice-case-seeded-out-dir");
            fs::create_dir_all(&path).expect("seeded OUT_DIR should be created");
            std::env::set_var("OUT_DIR", &path);
            path
        });
    fs::write(out_dir.join(file_name), source).expect("OUT_DIR generated source should be seeded");
}

fn read(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path.as_ref())
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.as_ref().display()))
}
