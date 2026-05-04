use std::collections::{BTreeMap, BTreeSet};

const TARGET_RULE_COUNT: usize = 1_200;

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
    promotion_hint: String,
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
                promotion_hint: format!(
                    "promote:{}:{}:{}:{}",
                    group.id, root_kind.id, edge_shape.id, failure_mode.id
                ),
            }
        })
        .collect()
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
