use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use opensource_core::{generate, GenerateOptions};

const TARGET_RULE_COUNT: usize = 1_200;
const GENERATED_BATCH_SIZE: usize = 50;

#[derive(Clone, Debug)]
struct Axis {
    id: &'static str,
    purpose: &'static str,
}

#[derive(Clone, Debug)]
struct RuleCatalogEntry {
    id: String,
    group: &'static str,
    root_kind: &'static str,
    edge_shape: &'static str,
    dependency_surface: &'static str,
    validation_mode: &'static str,
    expected_behavior: &'static str,
    failure_mode: &'static str,
    coverage: CoverageStrategy,
    promotion_hint: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CoverageStrategy {
    family: &'static str,
    executable_seed: &'static str,
    enforcement: &'static str,
}

const GROUPS: &[Axis] = &[
    Axis {
        id: "import",
        purpose: "use, pub use, glob, rename, shadowing, and reexport cleanup",
    },
    Axis {
        id: "macro",
        purpose: "macro_rules, item macros, derives, attributes, and helper attrs",
    },
    Axis {
        id: "trait",
        purpose: "trait impls, associated items, blanket impls, UFCS, and projections",
    },
    Axis {
        id: "dyn",
        purpose: "trait objects, callback registries, function pointers, and async callbacks",
    },
    Axis {
        id: "include",
        purpose: "include, include_str, include_bytes, and asset copying",
    },
    Axis {
        id: "build",
        purpose: "build scripts, generated Rust, compile env, and OUT_DIR surfaces",
    },
    Axis {
        id: "uniffi",
        purpose: "FFI records, enums, objects, callbacks, setup, and helper attributes",
    },
    Axis {
        id: "manifest",
        purpose: "workspace members, path dependencies, patches, locks, and target tables",
    },
    Axis {
        id: "repair",
        purpose: "compiler-feedback cleanup, repeated diagnostics, and malformed remnants",
    },
    Axis {
        id: "cfg",
        purpose: "cfg gates moved intact or fail-closed under bounded validation",
    },
];

const ROOT_KINDS: &[Axis] = &[
    Axis {
        id: "free_fn",
        purpose: "selected free function or exported function",
    },
    Axis {
        id: "method",
        purpose: "selected inherent or trait method",
    },
    Axis {
        id: "struct",
        purpose: "selected struct, field, constructor, or retained state type",
    },
    Axis {
        id: "enum",
        purpose: "selected enum, data variant, or pattern surface",
    },
    Axis {
        id: "trait",
        purpose: "selected trait definition or object boundary",
    },
    Axis {
        id: "module",
        purpose: "selected module, facade, prelude, or external file module",
    },
    Axis {
        id: "binary",
        purpose: "selected bin target or main function",
    },
    Axis {
        id: "test_target",
        purpose: "retained integration test or test-only boundary validation",
    },
    Axis {
        id: "example_target",
        purpose: "retained example or required-feature target",
    },
    Axis {
        id: "build_script",
        purpose: "retained build script or generated input boundary",
    },
    Axis {
        id: "ffi_export",
        purpose: "selected FFI-facing export, callback, or object method",
    },
];

const EDGE_SHAPES: &[Axis] = &[
    Axis {
        id: "direct_path",
        purpose: "direct path or unqualified local reference",
    },
    Axis {
        id: "receiver_method",
        purpose: "method call through a typed receiver",
    },
    Axis {
        id: "ufcs_method",
        purpose: "UFCS or explicit trait method call",
    },
    Axis {
        id: "trait_bound_method",
        purpose: "method reached through a generic trait bound",
    },
    Axis {
        id: "associated_projection",
        purpose: "associated type, const, or projection reference",
    },
    Axis {
        id: "public_reexport",
        purpose: "public reexport chain or facade hub",
    },
    Axis {
        id: "glob_reexport",
        purpose: "glob reexport with mixed live and dead names",
    },
    Axis {
        id: "renamed_import",
        purpose: "renamed dependency or local alias",
    },
    Axis {
        id: "macro_body",
        purpose: "dependency visible only inside retained macro tokens",
    },
    Axis {
        id: "derive_helper_attr",
        purpose: "derive helper attribute path or callback",
    },
    Axis {
        id: "attribute_macro_surface",
        purpose: "retained custom attribute macro surface",
    },
    Axis {
        id: "item_macro_invocation",
        purpose: "retained item macro that declares reachable items",
    },
    Axis {
        id: "dyn_trait_surface",
        purpose: "retained dyn Trait input, output, field, or alias",
    },
    Axis {
        id: "fn_pointer_surface",
        purpose: "retained fn pointer input, output, field, or alias",
    },
    Axis {
        id: "include_asset",
        purpose: "literal or concat include asset path",
    },
    Axis {
        id: "source_include",
        purpose: "Rust source include boundary",
    },
    Axis {
        id: "compile_env",
        purpose: "env or option_env macro boundary",
    },
    Axis {
        id: "build_script_output",
        purpose: "build script emitted file, cfg, env, or link input",
    },
    Axis {
        id: "cfg_gate_move_intact",
        purpose: "cfg attribute retained on a live item without matrix expansion",
    },
    Axis {
        id: "manifest_dependency",
        purpose: "Cargo dependency, feature, patch, target table, or toolchain edge",
    },
];

const DEPENDENCY_SURFACES: &[Axis] = &[
    Axis {
        id: "none",
        purpose: "single-package local source only",
    },
    Axis {
        id: "local_module",
        purpose: "same package external module file",
    },
    Axis {
        id: "workspace_package",
        purpose: "workspace member package dependency",
    },
    Axis {
        id: "external_crate",
        purpose: "registry dependency or renamed dependency",
    },
    Axis {
        id: "path_package",
        purpose: "local non-workspace path package",
    },
    Axis {
        id: "build_dependency",
        purpose: "build-dependency table or build-script-only dependency",
    },
    Axis {
        id: "proc_macro_dependency",
        purpose: "derive, attribute, or item proc-macro dependency",
    },
    Axis {
        id: "optional_feature_dep",
        purpose: "optional dependency reached through feature cfg",
    },
    Axis {
        id: "target_dependency",
        purpose: "target-specific dependency table",
    },
    Axis {
        id: "support_asset",
        purpose: "non-Rust asset copied with retained source",
    },
];

const VALIDATION_MODES: &[Axis] = &[
    Axis {
        id: "syn",
        purpose: "syntactic slicer fallback",
    },
    Axis {
        id: "ra_hir",
        purpose: "bounded rust-analyzer semantic edge collection",
    },
    Axis {
        id: "ra_feedback",
        purpose: "copy/prove/cut rust-analyzer feedback mode",
    },
    Axis {
        id: "production",
        purpose: "production gate with compiler feedback",
    },
    Axis {
        id: "no_default_features",
        purpose: "no-default-feature validation",
    },
    Axis {
        id: "feature_target",
        purpose: "explicit feature or target validation",
    },
];

const EXPECTED_BEHAVIORS: &[Axis] = &[
    Axis {
        id: "retain_live",
        purpose: "retain the live item and transitive support closure",
    },
    Axis {
        id: "prune_dead",
        purpose: "prune dead sibling names, modules, items, assets, or dependencies",
    },
    Axis {
        id: "copy_asset",
        purpose: "copy only retained asset paths",
    },
    Axis {
        id: "copy_support_package",
        purpose: "copy only retained support package closure",
    },
    Axis {
        id: "report_warning",
        purpose: "report review-required warning with structured details",
    },
    Axis {
        id: "report_error",
        purpose: "report production-blocking hazard with structured details",
    },
    Axis {
        id: "repair_feedback",
        purpose: "repair compiler-reported unused imports or malformed remnants",
    },
    Axis {
        id: "move_cfg_intact",
        purpose: "preserve live cfg gate exactly unless validation proves it inactive",
    },
];

const FAILURE_MODES: &[Axis] = &[
    Axis {
        id: "missing_live_edge",
        purpose: "needed item, impl, module, macro, or dependency was pruned",
    },
    Axis {
        id: "dead_retention",
        purpose: "dead item, module, asset, package, or manifest entry survived",
    },
    Axis {
        id: "malformed_source",
        purpose: "rendered Rust contains empty use groups, path remnants, or orphan attrs",
    },
    Axis {
        id: "missing_asset",
        purpose: "retained include asset was not copied",
    },
    Axis {
        id: "overcopied_support",
        purpose: "support package or path dependency closure is too broad",
    },
    Axis {
        id: "unmodeled_macro",
        purpose: "macro expansion or helper attr dependency is not semantically modeled",
    },
    Axis {
        id: "unmodeled_dynamic_dispatch",
        purpose: "dyn Trait or callback call target cannot be proven",
    },
    Axis {
        id: "unmodeled_generated_source",
        purpose: "build script or include generated Rust cannot be modeled",
    },
    Axis {
        id: "cfg_not_proven",
        purpose: "cfg gate is retained but not proven by validation args",
    },
    Axis {
        id: "feedback_no_progress",
        purpose: "compiler-feedback loop repeats without reducing diagnostics",
    },
];

fn build_catalog() -> Vec<RuleCatalogEntry> {
    (0..TARGET_RULE_COUNT)
        .map(|index| {
            let group = axis_at(GROUPS, index, 1, 0);
            let root_kind = axis_at(ROOT_KINDS, index, 3, 1);
            let edge_shape = axis_at(EDGE_SHAPES, index, 7, 2);
            let dependency_surface = axis_at(DEPENDENCY_SURFACES, index, 9, 3);
            let validation_mode = axis_at(VALIDATION_MODES, index, 11, 4);
            let expected_behavior = axis_at(EXPECTED_BEHAVIORS, index, 13, 5);
            let failure_mode = axis_at(FAILURE_MODES, index, 17, 6);
            let coverage = coverage_strategy(
                group.id,
                root_kind.id,
                edge_shape.id,
                dependency_surface.id,
                expected_behavior.id,
                failure_mode.id,
            );
            RuleCatalogEntry {
                id: format!(
                    "{}.{}.{}.{}.{}.{}",
                    group.id,
                    root_kind.id,
                    edge_shape.id,
                    dependency_surface.id,
                    validation_mode.id,
                    sequence(index)
                ),
                group: group.id,
                root_kind: root_kind.id,
                edge_shape: edge_shape.id,
                dependency_surface: dependency_surface.id,
                validation_mode: validation_mode.id,
                expected_behavior: expected_behavior.id,
                failure_mode: failure_mode.id,
                coverage,
                promotion_hint: format!(
                    "promote:{}:{}:{}:{}",
                    group.id, root_kind.id, edge_shape.id, failure_mode.id
                ),
            }
        })
        .collect()
}

fn coverage_strategy(
    group: &'static str,
    root_kind: &'static str,
    edge_shape: &'static str,
    dependency_surface: &'static str,
    expected_behavior: &'static str,
    failure_mode: &'static str,
) -> CoverageStrategy {
    match group {
        "import" => strategy(
            "import.reexport_and_alias_pruning",
            "import.reexport.grouped.001",
            "executable_fixture",
        ),
        "macro" => {
            let seed = match edge_shape {
                "derive_helper_attr" | "attribute_macro_surface" => {
                    "macro.proc_attr_import_retention.001"
                }
                "item_macro_invocation" => "macro.item_invocation.generated_api.001",
                _ => "macro.metavariable_method.001",
            };
            strategy(
                "macro.expansion_surface_closure",
                seed,
                "executable_fixture",
            )
        }
        "trait" => strategy(
            "trait.impl_projection_and_conversion_closure",
            "trait.associated_projection.001",
            "executable_fixture",
        ),
        "dyn" => {
            let seed = if edge_shape == "fn_pointer_surface" {
                "ffi.callback_direct_input.001"
            } else {
                "dyn.callback.future_alias.001"
            };
            strategy(
                "dyn.dispatch_and_callback_boundaries",
                seed,
                "production_hazard_fixture",
            )
        }
        "include" => {
            let seed = if edge_shape == "source_include" {
                "include.source.static.001"
            } else {
                "include.str.static_concat.001"
            };
            let enforcement = if edge_shape == "source_include" {
                "production_hazard_fixture"
            } else {
                "executable_fixture"
            };
            strategy("include.assets_and_source_boundaries", seed, enforcement)
        }
        "build" => strategy(
            "build.generated_source_and_env_hazards",
            "build.out_dir_source_include.001",
            "production_hazard_fixture",
        ),
        "uniffi" => strategy(
            "uniffi.exported_surface_closure",
            "uniffi.object.split_impl_wrapper.001",
            "executable_fixture",
        ),
        "manifest" => strategy(
            "manifest.support_package_pruning",
            "manifest.support_dependency_item_pruning.001",
            "executable_fixture",
        ),
        "repair" => strategy(
            "repair.compiler_feedback_convergence",
            "repair.feedback_loop_guards.001",
            "catalog_guard",
        ),
        "cfg" => strategy(
            "cfg.move_intact_or_fail_closed",
            "catalog.cfg_move_intact_policy.001",
            "catalog_guard",
        ),
        _ if root_kind == "ffi_export" => strategy(
            "uniffi.exported_surface_closure",
            "uniffi.object.split_impl_wrapper.001",
            "executable_fixture",
        ),
        _ if dependency_surface == "path_package"
            || dependency_surface == "workspace_package"
            || dependency_surface == "target_dependency"
            || dependency_surface == "optional_feature_dep"
            || failure_mode == "overcopied_support"
            || expected_behavior == "copy_support_package" =>
        {
            strategy(
                "manifest.support_package_pruning",
                "manifest.support_dependency_item_pruning.001",
                "executable_fixture",
            )
        }
        _ => strategy(
            "core.direct_item_surface_closure",
            "struct.private_generic_field_usage.001",
            "executable_fixture",
        ),
    }
}

fn strategy(
    family: &'static str,
    executable_seed: &'static str,
    enforcement: &'static str,
) -> CoverageStrategy {
    CoverageStrategy {
        family,
        executable_seed,
        enforcement,
    }
}

fn axis_at(axes: &[Axis], index: usize, multiplier: usize, offset: usize) -> &Axis {
    &axes[(index * multiplier + offset) % axes.len()]
}

fn sequence(index: usize) -> String {
    format!("{:04}", index + 1)
}

#[test]
fn catalog_generates_at_least_one_thousand_generic_rules() {
    let catalog = build_catalog();

    assert!(catalog.len() >= 1_000, "catalog has {}", catalog.len());

    let ids: BTreeSet<_> = catalog.iter().map(|entry| entry.id.as_str()).collect();
    assert_eq!(ids.len(), catalog.len(), "catalog rule ids must be unique");

    for entry in &catalog {
        assert_valid_identifier(&entry.id);
        assert!(!entry.group.is_empty(), "{entry:?}");
        assert!(!entry.root_kind.is_empty(), "{entry:?}");
        assert!(!entry.edge_shape.is_empty(), "{entry:?}");
        assert!(!entry.dependency_surface.is_empty(), "{entry:?}");
        assert!(!entry.validation_mode.is_empty(), "{entry:?}");
        assert!(!entry.expected_behavior.is_empty(), "{entry:?}");
        assert!(!entry.failure_mode.is_empty(), "{entry:?}");
        assert_valid_identifier(entry.coverage.family);
        assert_valid_identifier(entry.coverage.executable_seed);
        assert!(
            is_valid_enforcement(entry.coverage.enforcement),
            "{entry:?}"
        );
        assert!(entry.promotion_hint.starts_with("promote:"), "{entry:?}");
    }
}

#[test]
fn catalog_covers_every_declared_axis_value() {
    let catalog = build_catalog();

    assert_axis_coverage(&catalog, "group", GROUPS, |entry| entry.group);
    assert_axis_coverage(&catalog, "root_kind", ROOT_KINDS, |entry| entry.root_kind);
    assert_axis_coverage(&catalog, "edge_shape", EDGE_SHAPES, |entry| {
        entry.edge_shape
    });
    assert_axis_coverage(
        &catalog,
        "dependency_surface",
        DEPENDENCY_SURFACES,
        |entry| entry.dependency_surface,
    );
    assert_axis_coverage(&catalog, "validation_mode", VALIDATION_MODES, |entry| {
        entry.validation_mode
    });
    assert_axis_coverage(&catalog, "expected_behavior", EXPECTED_BEHAVIORS, |entry| {
        entry.expected_behavior
    });
    assert_axis_coverage(&catalog, "failure_mode", FAILURE_MODES, |entry| {
        entry.failure_mode
    });
}

#[test]
fn catalog_entries_are_not_tied_to_known_real_project_names() {
    let catalog = build_catalog();
    let forbidden_terms = [
        "litter",
        "codex",
        "handoff",
        "conversation",
        "rust_bridge",
        "rust-bridge",
        "mobile_client",
        "ipc",
    ];

    for entry in &catalog {
        let haystack = format!(
            "{} {} {} {} {} {} {} {}",
            entry.id,
            entry.group,
            entry.root_kind,
            entry.edge_shape,
            entry.dependency_surface,
            entry.validation_mode,
            entry.expected_behavior,
            entry.promotion_hint
        );
        for term in forbidden_terms {
            assert!(
                !haystack.contains(term),
                "catalog entry is project-specific: term={term} entry={entry:?}"
            );
        }
    }
}

#[test]
fn catalog_maps_every_record_to_an_enforceable_coverage_family() {
    let catalog = build_catalog();

    for entry in &catalog {
        assert!(!entry.coverage.family.is_empty(), "{entry:?}");
        assert!(!entry.coverage.executable_seed.is_empty(), "{entry:?}");
        assert!(
            is_valid_enforcement(entry.coverage.enforcement),
            "{entry:?}"
        );
    }

    let family_counts = count_entries_by(&catalog, |entry| entry.coverage.family);
    assert!(
        family_counts.len() >= 10,
        "coverage families should stay broad but varied: {family_counts:?}"
    );
    assert!(
        family_counts
            .values()
            .all(|count| *count <= TARGET_RULE_COUNT / 3),
        "one coverage family is hiding too much rule variety: {family_counts:?}"
    );

    let executable_backed = catalog
        .iter()
        .filter(|entry| {
            matches!(
                entry.coverage.enforcement,
                "executable_fixture" | "production_hazard_fixture"
            )
        })
        .count();
    assert!(
        executable_backed >= 900,
        "most catalog rules should map to executable or hazard fixture families, got {executable_backed}"
    );
}

#[test]
fn catalog_keeps_cfg_as_bounded_move_intact_work_not_matrix_explosion() {
    let catalog = build_catalog();
    let cfg_entries: Vec<_> = catalog
        .iter()
        .filter(|entry| {
            entry.group == "cfg"
                || entry.edge_shape == "cfg_gate_move_intact"
                || entry.expected_behavior == "move_cfg_intact"
                || entry.failure_mode == "cfg_not_proven"
        })
        .collect();

    assert!(
        cfg_entries.len() >= 100,
        "cfg needs broad catalog representation without generating a feature matrix"
    );
    assert!(cfg_entries
        .iter()
        .any(|entry| entry.expected_behavior == "move_cfg_intact"));
    assert!(cfg_entries
        .iter()
        .any(|entry| entry.failure_mode == "cfg_not_proven"));

    let validation_counts = count_by(&cfg_entries, |entry| entry.validation_mode);
    assert!(validation_counts.contains_key("production"));
    assert!(validation_counts.contains_key("feature_target"));
}

#[test]
fn catalog_has_readable_promotion_hints_for_executable_fixtures() {
    let catalog = build_catalog();

    let candidates: Vec<_> = catalog
        .iter()
        .filter(|entry| {
            matches!(
                entry.failure_mode,
                "missing_live_edge"
                    | "dead_retention"
                    | "malformed_source"
                    | "missing_asset"
                    | "overcopied_support"
            )
        })
        .collect();
    assert!(
        candidates.len() >= 500,
        "expected enough executable-fixture candidates, got {}",
        candidates.len()
    );

    for entry in candidates.iter().take(50) {
        let parts: Vec<_> = entry.promotion_hint.split(':').collect();
        assert_eq!(parts.len(), 5, "{entry:?}");
        assert_eq!(parts[0], "promote", "{entry:?}");
        assert_eq!(parts[1], entry.group, "{entry:?}");
        assert_eq!(parts[2], entry.root_kind, "{entry:?}");
        assert_eq!(parts[3], entry.edge_shape, "{entry:?}");
        assert_eq!(parts[4], entry.failure_mode, "{entry:?}");
    }
}

#[test]
fn catalog_executes_all_generated_rules_as_real_slices() {
    let catalog = build_catalog();
    let base = temp_path("rule-catalog-executable-slices");
    let source_base = base.join("source");
    let output_base = base.join("output");
    let selected = selected_catalog_entries(&catalog);

    let mut failures = Vec::new();
    let mut executed = 0;
    for (batch_index, batch) in selected.chunks(GENERATED_BATCH_SIZE).enumerate() {
        let package = format!("catalog_batch_{batch_index:04}");
        let workspace = source_base.join(format!("batch-{batch_index:04}"));
        let output = output_base.join(format!("batch-{batch_index:04}"));
        let expectations = write_catalog_batch_fixture(&workspace, &package, batch);

        match generate(GenerateOptions {
            workspace_root: workspace.clone(),
            output_root: output.clone(),
        }) {
            Ok(report) => {
                assert_catalog_batch(&output, &report, &expectations, &mut failures);
            }
            Err(error) => {
                failures.push(format!(
                    "batch {batch_index:04} failed to generate: {error}; rows={}",
                    batch
                        .iter()
                        .map(|(_, entry)| entry.id.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        }
        executed += batch.len();
    }

    if std::env::var_os("OS_RULE_CATALOG_FILTER").is_none() {
        assert_eq!(executed, TARGET_RULE_COUNT);
    } else {
        assert!(executed > 0, "catalog filter selected no generated rules");
    }
    assert!(
        failures.is_empty(),
        "generated catalog slicing failures ({}):\n{}",
        failures.len(),
        failures.join("\n\n")
    );
}

fn assert_valid_identifier(id: &str) {
    assert!(
        id.chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_' || ch == '.'),
        "invalid rule id: {id}"
    );
    assert!(!id.contains(".."), "invalid rule id: {id}");
    assert!(!id.starts_with('.'), "invalid rule id: {id}");
    assert!(!id.ends_with('.'), "invalid rule id: {id}");
}

fn is_valid_enforcement(enforcement: &str) -> bool {
    matches!(
        enforcement,
        "executable_fixture" | "production_hazard_fixture" | "catalog_guard"
    )
}

fn assert_axis_coverage(
    catalog: &[RuleCatalogEntry],
    axis_name: &str,
    expected: &[Axis],
    value: impl Fn(&RuleCatalogEntry) -> &'static str,
) {
    let actual: BTreeSet<_> = catalog.iter().map(value).collect();
    let missing: Vec<_> = expected
        .iter()
        .filter(|axis| !actual.contains(axis.id))
        .map(|axis| format!("{} ({})", axis.id, axis.purpose))
        .collect();
    assert!(missing.is_empty(), "missing {axis_name} axes: {missing:?}");
}

fn count_entries_by(
    entries: &[RuleCatalogEntry],
    value: impl Fn(&RuleCatalogEntry) -> &'static str,
) -> BTreeMap<&'static str, usize> {
    let mut counts = BTreeMap::new();
    for entry in entries {
        *counts.entry(value(entry)).or_insert(0) += 1;
    }
    counts
}

fn count_by<'a>(
    entries: &[&'a RuleCatalogEntry],
    value: impl Fn(&'a RuleCatalogEntry) -> &'static str,
) -> BTreeMap<&'static str, usize> {
    let mut counts = BTreeMap::new();
    for entry in entries {
        *counts.entry(value(entry)).or_insert(0) += 1;
    }
    counts
}

fn selected_catalog_entries(catalog: &[RuleCatalogEntry]) -> Vec<(usize, &RuleCatalogEntry)> {
    let filter = std::env::var("OS_RULE_CATALOG_FILTER").ok();
    catalog
        .iter()
        .enumerate()
        .filter(|(_, entry)| {
            filter.as_ref().is_none_or(|filter| {
                entry.id.contains(filter)
                    || entry.group.contains(filter)
                    || entry.root_kind.contains(filter)
                    || entry.edge_shape.contains(filter)
                    || entry.dependency_surface.contains(filter)
                    || entry.coverage.family.contains(filter)
            })
        })
        .collect()
}

#[derive(Debug)]
struct GeneratedCatalogExpectation {
    row_id: String,
    family: &'static str,
    package: String,
    keep: Vec<String>,
    drop: Vec<String>,
    hazard_codes: Vec<&'static str>,
    copied_files: Vec<PathBuf>,
    omitted_files: Vec<PathBuf>,
}

#[derive(Debug)]
struct RenderedCatalogCase {
    module: String,
    source: String,
    expectation: GeneratedCatalogExpectation,
    dependency: Option<String>,
    support_member: Option<String>,
    build_env: Option<(String, String)>,
}

#[derive(Debug)]
struct CaseNames {
    module: String,
    selected: String,
    live_fn: String,
    dead_fn: String,
    helper_fn: String,
    live_type: String,
    dead_type: String,
    live_trait: String,
    dead_trait: String,
    live_macro: String,
    dead_macro: String,
    root_owner: String,
    root_struct: String,
    root_enum: String,
    root_trait: String,
    root_mod: String,
    support_pkg: String,
    env_var: String,
}

fn write_catalog_batch_fixture(
    root: &Path,
    package: &str,
    entries: &[(usize, &RuleCatalogEntry)],
) -> Vec<GeneratedCatalogExpectation> {
    let mut members = vec![package.to_string()];
    let mut dependencies = Vec::new();
    let mut build_env = Vec::new();
    let mut lib = String::from("#![allow(unexpected_cfgs)]\n\n");
    let mut expectations = Vec::new();

    for (index, entry) in entries {
        let rendered = render_catalog_case(root, package, entry, *index);
        if let Some(member) = rendered.support_member {
            members.push(member);
        }
        if let Some(dependency) = rendered.dependency {
            dependencies.push(dependency);
        }
        if let Some(env) = rendered.build_env {
            build_env.push(env);
        }
        lib.push_str(&format!("// {}\n", rendered.module));
        lib.push_str(&rendered.source);
        lib.push('\n');
        expectations.push(rendered.expectation);
    }

    members.sort();
    members.dedup();
    let members = members
        .iter()
        .map(|member| format!("{member:?}"))
        .collect::<Vec<_>>()
        .join(", ");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[workspace]
members = [{members}]
resolver = "2"
"#
        ),
    );

    let build_line = if build_env.is_empty() {
        ""
    } else {
        "build = \"build.rs\"\n"
    };
    write(
        root.join(format!("{package}/Cargo.toml")),
        &format!(
            r#"[package]
name = {package:?}
version = "0.1.0"
edition = "2021"
{build_line}
[dependencies]
opensourced = {{ path = "{}" }}
{}
"#,
            manifest_path(&repo_root().join("crates/opensourced")),
            dependencies.join("")
        ),
    );
    write(root.join(format!("{package}/src/lib.rs")), &lib);

    if !build_env.is_empty() {
        let mut build_rs = String::from("fn main() {\n");
        for (name, value) in build_env {
            build_rs.push_str(&format!(
                "    println!(\"cargo:rustc-env={name}={value}\");\n"
            ));
        }
        build_rs.push_str("}\n");
        write(root.join(format!("{package}/build.rs")), &build_rs);
    }

    expectations
}

fn render_catalog_case(
    workspace: &Path,
    package: &str,
    entry: &RuleCatalogEntry,
    index: usize,
) -> RenderedCatalogCase {
    match entry.group {
        "import" => render_import_case(package, entry, index),
        "macro" => render_macro_case(package, entry, index),
        "trait" => render_trait_case(package, entry, index),
        "dyn" => render_dyn_case(package, entry, index),
        "include" => render_include_case(workspace, package, entry, index),
        "build" => render_build_case(package, entry, index),
        "uniffi" => render_uniffi_case(package, entry, index),
        "manifest" => render_manifest_case(workspace, package, entry, index),
        "repair" => render_repair_case(package, entry, index),
        "cfg" => render_cfg_case(package, entry, index),
        _ => unreachable!("catalog group is closed: {}", entry.group),
    }
}

fn render_import_case(
    package: &str,
    entry: &RuleCatalogEntry,
    index: usize,
) -> RenderedCatalogCase {
    let names = case_names(index);
    let (use_line, selected_expr, keep_extra) = match entry.edge_shape {
        "glob_reexport" => (
            format!("pub use api_{}::*;", index),
            format!(
                "let item = {live_type} {{ value: {live_fn}() }}; item.value",
                live_type = names.live_type,
                live_fn = names.live_fn
            ),
            names.live_fn.clone(),
        ),
        "renamed_import" => (
            format!(
                "pub use api_{index}::{{{dead_fn} as dead_alias_{index:04}, {dead_type} as DeadAlias{index:04}, {live_fn} as live_alias_{index:04}, {live_type} as LiveAlias{index:04}}};",
                dead_fn = names.dead_fn,
                dead_type = names.dead_type,
                live_fn = names.live_fn,
                live_type = names.live_type
            ),
            format!("let item = LiveAlias{index:04} {{ value: live_alias_{index:04}() }}; item.value"),
            format!("live_alias_{index:04}"),
        ),
        _ => (
            format!(
                "pub use api_{index}::{{{dead_fn}, {dead_type}, {live_fn}, {live_type}}};",
                dead_fn = names.dead_fn,
                dead_type = names.dead_type,
                live_fn = names.live_fn,
                live_type = names.live_type
            ),
            format!(
                "let item = {live_type} {{ value: {live_fn}() }}; item.value",
                live_type = names.live_type,
                live_fn = names.live_fn
            ),
            names.live_fn.clone(),
        ),
    };
    let root_marker = root_kind_marker(entry, &names);
    let source = format!(
        r#"
pub mod {module} {{
    pub mod api_{index} {{
        pub struct {live_type} {{
            pub value: u32,
        }}

        pub struct {dead_type} {{
            pub value: u32,
        }}

        pub fn {live_fn}() -> u32 {{
            7
        }}

        pub fn {dead_fn}() -> u32 {{
            99
        }}
    }}

    {use_line}

    #[opensourced::opensourced]
    pub fn {selected}() -> u32 {{
        {selected_expr}
    }}

    {root_marker}
}}
"#,
        module = names.module,
        index = index,
        live_type = names.live_type,
        dead_type = names.dead_type,
        live_fn = names.live_fn,
        dead_fn = names.dead_fn,
        use_line = use_line,
        selected = names.selected,
        selected_expr = selected_expr,
        root_marker = root_marker,
    );
    catalog_case(
        package,
        entry,
        names,
        source,
        vec![keep_extra],
        vec![format!("dead_alias_{index:04}")],
        vec![],
        vec![],
        vec![],
    )
}

fn render_macro_case(package: &str, entry: &RuleCatalogEntry, index: usize) -> RenderedCatalogCase {
    let names = case_names(index);
    let root_marker = root_kind_marker(entry, &names);
    let source = if entry.edge_shape == "item_macro_invocation" {
        format!(
            r#"
pub mod {module} {{
    macro_rules! {live_macro} {{
        () => {{
            pub fn {live_fn}() -> u32 {{
                7
            }}
        }};
    }}

    macro_rules! {dead_macro} {{
        () => {{
            pub fn {dead_fn}() -> u32 {{
                99
            }}
        }};
    }}

    {live_macro}!();

    #[opensourced::opensourced]
    pub fn {selected}() -> u32 {{
        {live_fn}()
    }}

    {root_marker}
}}
"#,
            module = names.module,
            live_macro = names.live_macro,
            dead_macro = names.dead_macro,
            live_fn = names.live_fn,
            dead_fn = names.dead_fn,
            selected = names.selected,
            root_marker = root_marker,
        )
    } else {
        format!(
            r#"
pub mod {module} {{
    macro_rules! {live_macro} {{
        ($value:expr) => {{
            {helper_fn}($value)
        }};
    }}

    macro_rules! {dead_macro} {{
        () => {{
            {dead_fn}()
        }};
    }}

    fn {helper_fn}(value: u32) -> u32 {{
        value + 1
    }}

    fn {dead_fn}() -> u32 {{
        99
    }}

    #[opensourced::opensourced]
    pub fn {selected}() -> u32 {{
        {live_macro}!(7)
    }}

    {root_marker}
}}
"#,
            module = names.module,
            live_macro = names.live_macro,
            dead_macro = names.dead_macro,
            helper_fn = names.helper_fn,
            dead_fn = names.dead_fn,
            selected = names.selected,
            root_marker = root_marker,
        )
    };
    let keep = vec![names.live_macro.clone()];
    catalog_case(
        package,
        entry,
        names,
        source,
        keep,
        vec![],
        vec![],
        vec![],
        vec![],
    )
}

fn render_trait_case(package: &str, entry: &RuleCatalogEntry, index: usize) -> RenderedCatalogCase {
    let names = case_names(index);
    let root_marker = root_kind_marker(entry, &names);
    let source = format!(
        r#"
pub mod {module} {{
    pub trait {live_trait} {{
        type Output;
        const BASE: u32;

        fn score(&self) -> Self::Output;
    }}

    pub struct {live_type};
    pub struct {dead_type};

    impl {live_trait} for {live_type} {{
        type Output = u32;
        const BASE: u32 = 7;

        fn score(&self) -> Self::Output {{
            Self::BASE
        }}
    }}

    impl {live_trait} for {dead_type} {{
        type Output = u32;
        const BASE: u32 = 99;

        fn score(&self) -> Self::Output {{
            {dead_fn}()
        }}
    }}

    fn {dead_fn}() -> u32 {{
        99
    }}

    #[opensourced::opensourced]
    pub fn {selected}() -> u32 {{
        <{live_type} as {live_trait}>::score(&{live_type})
    }}

    {root_marker}
}}
"#,
        module = names.module,
        live_trait = names.live_trait,
        live_type = names.live_type,
        dead_type = names.dead_type,
        dead_fn = names.dead_fn,
        selected = names.selected,
        root_marker = root_marker,
    );
    let keep = vec![names.live_trait.clone(), names.live_type.clone()];
    catalog_case(
        package,
        entry,
        names,
        source,
        keep,
        vec![],
        vec![],
        vec![],
        vec![],
    )
}

fn render_dyn_case(package: &str, entry: &RuleCatalogEntry, index: usize) -> RenderedCatalogCase {
    let names = case_names(index);
    let root_marker = root_kind_marker(entry, &names);
    let (source, hazards, drop_extra) = if entry.edge_shape == "fn_pointer_surface" {
        (
            format!(
                r#"
pub mod {module} {{
    pub type Handler{index:04} = fn(u32) -> u32;
    pub type DeadHandler{index:04} = fn(u32) -> u32;

    fn {dead_fn}(value: u32) -> u32 {{
        value + 99
    }}

    #[opensourced::opensourced]
    pub fn {selected}(handler: Handler{index:04}) -> u32 {{
        handler(7)
    }}

    {root_marker}
}}
"#,
                module = names.module,
                index = index,
                dead_fn = names.dead_fn,
                selected = names.selected,
                root_marker = root_marker,
            ),
            vec!["function_pointer_surfaces"],
            vec![format!("DeadHandler{index:04}")],
        )
    } else {
        (
            format!(
                r#"
pub mod {module} {{
    pub trait {live_trait} {{
        fn call(&self) -> u32;
    }}

    pub trait {dead_trait} {{
        fn call(&self) -> u32;
    }}

    fn {dead_fn}() -> u32 {{
        99
    }}

    #[opensourced::opensourced]
    pub fn {selected}(handler: &dyn {live_trait}) -> u32 {{
        handler.call()
    }}

    {root_marker}
}}
"#,
                module = names.module,
                live_trait = names.live_trait,
                dead_trait = names.dead_trait,
                dead_fn = names.dead_fn,
                selected = names.selected,
                root_marker = root_marker,
            ),
            vec!["dynamic_callback_boundaries"],
            vec![],
        )
    };
    let keep = if entry.edge_shape == "fn_pointer_surface" {
        vec![format!("Handler{index:04}")]
    } else {
        vec![names.live_trait.clone()]
    };
    catalog_case(
        package,
        entry,
        names,
        source,
        keep,
        drop_extra,
        hazards,
        vec![],
        vec![],
    )
}

fn render_include_case(
    workspace: &Path,
    package: &str,
    entry: &RuleCatalogEntry,
    index: usize,
) -> RenderedCatalogCase {
    let names = case_names(index);
    let root_marker = root_kind_marker(entry, &names);
    if entry.edge_shape == "source_include" || entry.failure_mode == "unmodeled_generated_source" {
        let include_file = format!("generated_{index:04}.rs");
        write(
            workspace.join(format!("{package}/src/{include_file}")),
            "41u32\n",
        );
        let source = format!(
            r#"
pub mod {module} {{
    #[opensourced::opensourced]
    pub fn {selected}() -> u32 {{
        include!("{include_file}")
    }}

    pub fn {dead_fn}() -> u32 {{
        99
    }}

    {root_marker}
}}
"#,
            module = names.module,
            selected = names.selected,
            include_file = include_file,
            dead_fn = names.dead_fn,
            root_marker = root_marker,
        );
        return catalog_case(
            package,
            entry,
            names,
            source,
            vec![format!("include!(\"{include_file}\")")],
            vec![],
            vec!["source_include_macros"],
            vec![],
            vec![],
        );
    }

    let live_asset = format!("{package}/src/assets/case_{index:04}/live.txt");
    let dead_asset = format!("{package}/src/assets/case_{index:04}/dead.txt");
    write(workspace.join(&live_asset), "live\n");
    write(workspace.join(&dead_asset), "dead\n");
    let source = format!(
        r#"
pub mod {module} {{
    const LIVE_TEXT_{index:04}: &str = include_str!("assets/case_{index:04}/live.txt");
    const DEAD_TEXT_{index:04}: &str = include_str!("assets/case_{index:04}/dead.txt");

    #[opensourced::opensourced]
    pub fn {selected}() -> usize {{
        LIVE_TEXT_{index:04}.len()
    }}

    pub fn {dead_fn}() -> usize {{
        DEAD_TEXT_{index:04}.len()
    }}

    {root_marker}
}}
"#,
        module = names.module,
        index = index,
        selected = names.selected,
        dead_fn = names.dead_fn,
        root_marker = root_marker,
    );
    catalog_case(
        package,
        entry,
        names,
        source,
        vec![format!("LIVE_TEXT_{index:04}")],
        vec![format!("DEAD_TEXT_{index:04}")],
        vec![],
        vec![PathBuf::from(format!(
            "{package}/src/assets/case_{index:04}/live.txt"
        ))],
        vec![PathBuf::from(format!(
            "{package}/src/assets/case_{index:04}/dead.txt"
        ))],
    )
}

fn render_build_case(package: &str, entry: &RuleCatalogEntry, index: usize) -> RenderedCatalogCase {
    let names = case_names(index);
    let root_marker = root_kind_marker(entry, &names);
    let source = format!(
        r#"
pub mod {module} {{
    #[opensourced::opensourced]
    pub fn {selected}() -> &'static str {{
        env!("{env_var}")
    }}

    pub fn {dead_fn}() -> &'static str {{
        "dead"
    }}

    {root_marker}
}}
"#,
        module = names.module,
        selected = names.selected,
        env_var = names.env_var,
        dead_fn = names.dead_fn,
        root_marker = root_marker,
    );
    let keep = vec![names.env_var.clone()];
    let mut rendered = catalog_case(
        package,
        entry,
        names,
        source,
        keep,
        vec![],
        vec!["compile_env_macros"],
        vec![],
        vec![],
    );
    rendered.build_env = Some((
        format!("CATALOG_TOKEN_{index:04}"),
        format!("live_{index:04}"),
    ));
    rendered
}

fn render_uniffi_case(
    package: &str,
    entry: &RuleCatalogEntry,
    index: usize,
) -> RenderedCatalogCase {
    let names = case_names(index);
    let root_marker = root_kind_marker(entry, &names);
    let source = format!(
        r#"
pub mod {module} {{
    #[cfg_attr(feature = "ffi", uniffi::Record)]
    pub struct {live_type} {{
        pub value: u32,
    }}

    #[cfg_attr(feature = "ffi", uniffi::Record)]
    pub struct {dead_type} {{
        pub value: u32,
    }}

    #[cfg_attr(feature = "ffi", uniffi::export)]
    impl {live_type} {{
        pub fn {live_fn}(&self) -> u32 {{
            self.value
        }}

        pub fn retained_macro_surface_{index:04}(&self) -> u32 {{
            self.value + 1
        }}
    }}

    #[cfg_attr(feature = "ffi", uniffi::export)]
    impl {dead_type} {{
        pub fn {dead_fn}(&self) -> u32 {{
            self.value + 99
        }}
    }}

    #[opensourced::opensourced]
    pub fn {selected}(value: u32) -> {live_type} {{
        {live_type} {{ value }}
    }}

    {root_marker}
}}
"#,
        module = names.module,
        live_type = names.live_type,
        dead_type = names.dead_type,
        live_fn = names.live_fn,
        dead_fn = names.dead_fn,
        selected = names.selected,
        index = index,
        root_marker = root_marker,
    );
    let keep = vec![names.live_type.clone(), names.live_fn.clone()];
    catalog_case(
        package,
        entry,
        names,
        source,
        keep,
        vec![],
        vec!["conditional_compilation_attrs"],
        vec![],
        vec![],
    )
}

fn render_manifest_case(
    workspace: &Path,
    package: &str,
    entry: &RuleCatalogEntry,
    index: usize,
) -> RenderedCatalogCase {
    let names = case_names(index);
    let root_marker = root_kind_marker(entry, &names);
    write(
        workspace.join(format!("{}/Cargo.toml", names.support_pkg)),
        &format!(
            r#"[package]
name = {:?}
version = "0.1.0"
edition = "2021"
"#,
            names.support_pkg
        ),
    );
    write(
        workspace.join(format!("{}/src/lib.rs", names.support_pkg)),
        &format!(
            r#"
pub fn {live_fn}() -> u32 {{
    7
}}

pub fn {dead_fn}() -> u32 {{
    99
}}
"#,
            live_fn = names.live_fn,
            dead_fn = names.dead_fn
        ),
    );
    let source = format!(
        r#"
pub mod {module} {{
    #[opensourced::opensourced]
    pub fn {selected}() -> u32 {{
        {support_pkg}::{live_fn}()
    }}

    pub fn {dead_fn}() -> u32 {{
        {support_pkg}::{dead_fn}()
    }}

    {root_marker}
}}
"#,
        module = names.module,
        selected = names.selected,
        support_pkg = names.support_pkg,
        live_fn = names.live_fn,
        dead_fn = names.dead_fn,
        root_marker = root_marker,
    );
    let keep = vec![names.live_fn.clone()];
    catalog_case(
        package,
        entry,
        names,
        source,
        keep,
        vec![],
        vec![],
        vec![],
        vec![],
    )
}

fn render_repair_case(
    package: &str,
    entry: &RuleCatalogEntry,
    index: usize,
) -> RenderedCatalogCase {
    let names = case_names(index);
    let root_marker = root_kind_marker(entry, &names);
    let source = format!(
        r#"
pub mod {module} {{
    use self::dead_{index}::{{dead_name_{index:04} as DeadAlias{index:04}, {dead_type}}};

    mod live_{index} {{
        pub fn {live_fn}() -> u32 {{
            7
        }}
    }}

    mod dead_{index} {{
        pub struct {dead_type};

        pub fn dead_name_{index:04}() -> u32 {{
            99
        }}
    }}

    #[opensourced::opensourced]
    pub fn {selected}() -> u32 {{
        live_{index}::{live_fn}()
    }}

    pub fn {dead_fn}() -> u32 {{
        let _ = DeadAlias{index:04}();
        let _ = {dead_type};
        99
    }}

    {root_marker}
}}
"#,
        module = names.module,
        index = index,
        live_fn = names.live_fn,
        dead_type = names.dead_type,
        selected = names.selected,
        dead_fn = names.dead_fn,
        root_marker = root_marker,
    );
    let keep = vec![names.live_fn.clone()];
    catalog_case(
        package,
        entry,
        names,
        source,
        keep,
        vec![format!("DeadAlias{index:04}")],
        vec![],
        vec![],
        vec![],
    )
}

fn render_cfg_case(package: &str, entry: &RuleCatalogEntry, index: usize) -> RenderedCatalogCase {
    let names = case_names(index);
    let root_marker = root_kind_marker(entry, &names);
    let source = format!(
        r#"
pub mod {module} {{
    #[cfg(any(unix, windows))]
    pub fn {live_fn}() -> u32 {{
        7
    }}

    #[cfg(target_os = "none")]
    pub fn {dead_fn}() -> u32 {{
        99
    }}

    #[opensourced::opensourced]
    pub fn {selected}() -> u32 {{
        {live_fn}()
    }}

    {root_marker}
}}
"#,
        module = names.module,
        live_fn = names.live_fn,
        dead_fn = names.dead_fn,
        selected = names.selected,
        root_marker = root_marker,
    );
    let keep = vec![names.live_fn.clone(), "cfg(any(unix, windows))".to_string()];
    catalog_case(
        package,
        entry,
        names,
        source,
        keep,
        vec!["target_os = \"none\"".to_string()],
        vec!["conditional_compilation_attrs"],
        vec![],
        vec![],
    )
}

#[allow(clippy::too_many_arguments)]
fn catalog_case(
    package: &str,
    entry: &RuleCatalogEntry,
    names: CaseNames,
    source: String,
    mut keep: Vec<String>,
    mut drop: Vec<String>,
    hazard_codes: Vec<&'static str>,
    copied_files: Vec<PathBuf>,
    omitted_files: Vec<PathBuf>,
) -> RenderedCatalogCase {
    keep.push(names.selected.clone());
    drop.push(names.dead_fn.clone());
    drop.push(names.dead_type.clone());
    drop.push(names.dead_trait.clone());
    drop.push(names.dead_macro.clone());

    let mut rendered = RenderedCatalogCase {
        module: names.module.clone(),
        source,
        expectation: GeneratedCatalogExpectation {
            row_id: entry.id.clone(),
            family: entry.coverage.family,
            package: package.to_string(),
            keep,
            drop,
            hazard_codes,
            copied_files,
            omitted_files,
        },
        dependency: None,
        support_member: None,
        build_env: None,
    };

    if entry.group == "manifest" {
        rendered.support_member = Some(names.support_pkg.clone());
        rendered.dependency = Some(format!(
            "{} = {{ path = \"../{}\" }}\n",
            names.support_pkg, names.support_pkg
        ));
        rendered
            .expectation
            .copied_files
            .push(PathBuf::from(format!("{}/src/lib.rs", names.support_pkg)));
    }

    rendered
}

fn case_names(index: usize) -> CaseNames {
    CaseNames {
        module: format!("case_{index:04}"),
        selected: format!("selected_{index:04}"),
        live_fn: format!("live_value_{index:04}"),
        dead_fn: format!("dead_marker_{index:04}"),
        helper_fn: format!("live_helper_{index:04}"),
        live_type: format!("LiveType{index:04}"),
        dead_type: format!("DeadType{index:04}"),
        live_trait: format!("LiveTrait{index:04}"),
        dead_trait: format!("DeadTrait{index:04}"),
        live_macro: format!("live_macro_{index:04}"),
        dead_macro: format!("dead_macro_{index:04}"),
        root_owner: format!("RootOwner{index:04}"),
        root_struct: format!("RootStruct{index:04}"),
        root_enum: format!("RootEnum{index:04}"),
        root_trait: format!("RootTrait{index:04}"),
        root_mod: format!("root_module_{index:04}"),
        support_pkg: format!("support_case_{index:04}"),
        env_var: format!("CATALOG_TOKEN_{index:04}"),
    }
}

fn root_kind_marker(entry: &RuleCatalogEntry, names: &CaseNames) -> String {
    match entry.root_kind {
        "method" => format!(
            r#"
    pub struct {root_owner};

    impl {root_owner} {{
        #[opensourced::opensourced]
        pub fn marked_method_{suffix}() -> u32 {{
            1
        }}
    }}
"#,
            root_owner = names.root_owner,
            suffix = names.module.trim_start_matches("case_"),
        ),
        "struct" => format!(
            r#"
    #[opensourced::opensourced]
    pub struct {root_struct} {{
        pub value: u32,
    }}
"#,
            root_struct = names.root_struct,
        ),
        "enum" => format!(
            r#"
    #[opensourced::opensourced]
    pub enum {root_enum} {{
        Live,
    }}
"#,
            root_enum = names.root_enum,
        ),
        "trait" => format!(
            r#"
    #[opensourced::opensourced]
    pub trait {root_trait} {{
        fn marker(&self) -> u32;
    }}
"#,
            root_trait = names.root_trait,
        ),
        "module" => format!(
            r#"
    #[opensourced::opensourced]
    pub mod {root_mod} {{
        pub fn marker() -> u32 {{
            1
        }}
    }}
"#,
            root_mod = names.root_mod,
        ),
        _ => String::new(),
    }
}

fn assert_catalog_batch(
    output: &Path,
    report: &opensource_core::GenerateReport,
    expectations: &[GeneratedCatalogExpectation],
    failures: &mut Vec<String>,
) {
    let generated_text = read_all_rs(output);
    for malformed in ["::::", ":::", "use ;", "use ::{", "{};", "#[opensourced"] {
        if generated_text.contains(malformed) {
            failures.push(format!(
                "batch output contains malformed source token {malformed:?}"
            ));
        }
    }

    for (file, contents) in read_parseable_rust_files(output) {
        if let Err(error) = syn::parse_file(&contents) {
            failures.push(format!(
                "{} failed syn parse: {error}\n{}",
                file.display(),
                contents
            ));
        }
    }

    for expectation in expectations {
        for needle in &expectation.keep {
            if !generated_text.contains(needle) {
                failures.push(format!(
                    "{}:{} ({}) missing live token {needle:?}",
                    expectation.package, expectation.row_id, expectation.family
                ));
            }
        }
        for needle in &expectation.drop {
            if generated_text.contains(needle) {
                failures.push(format!(
                    "{}:{} ({}) retained dead token {needle:?}",
                    expectation.package, expectation.row_id, expectation.family
                ));
            }
        }
        for code in &expectation.hazard_codes {
            if !report
                .production
                .hazards
                .iter()
                .any(|hazard| hazard.code == *code)
            {
                failures.push(format!(
                    "{}:{} ({}) missing production hazard code {code:?}; hazard codes={:?}",
                    expectation.package,
                    expectation.row_id,
                    expectation.family,
                    report
                        .production
                        .hazards
                        .iter()
                        .map(|hazard| hazard.code.as_str())
                        .collect::<Vec<_>>()
                ));
            }
        }
        for relative in &expectation.copied_files {
            if !output.join(relative).exists() {
                failures.push(format!(
                    "{}:{} ({}) did not copy expected file {}",
                    expectation.package,
                    expectation.row_id,
                    expectation.family,
                    output.join(relative).display()
                ));
            }
        }
        for relative in &expectation.omitted_files {
            if output.join(relative).exists() {
                failures.push(format!(
                    "{}:{} ({}) copied dead file {}",
                    expectation.package,
                    expectation.row_id,
                    expectation.family,
                    output.join(relative).display()
                ));
            }
        }
    }
}

fn read_parseable_rust_files(root: &Path) -> Vec<(PathBuf, String)> {
    let mut files = Vec::new();
    collect_files(root, &mut |path| {
        if path
            .file_name()
            .is_some_and(|name| name == "lib.rs" || name == "build.rs")
        {
            files.push((path.to_path_buf(), read(path)));
        }
    });
    files
}

fn read_all_rs(root: &Path) -> String {
    let mut combined = String::new();
    collect_files(root, &mut |path| {
        if path.extension().is_some_and(|extension| extension == "rs") {
            combined.push_str(&read(path));
            combined.push('\n');
        }
    });
    combined
}

fn collect_files(root: &Path, visit: &mut impl FnMut(&Path)) {
    if !root.exists() {
        return;
    }
    let entries = fs::read_dir(root).expect("directory should be readable");
    for entry in entries {
        let entry = entry.expect("directory entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, visit);
        } else {
            visit(&path);
        }
    }
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
