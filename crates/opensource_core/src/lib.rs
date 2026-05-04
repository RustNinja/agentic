mod analyzer;
mod feedback;
mod include_path;
mod manifest;
mod model;
mod parse;
mod preflight;
mod reduce;
mod render;
mod repair;

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

use include_path::{static_include_path, StaticIncludePath};
use model::{Project, ReducedProject};
use proc_macro2::TokenStream;
use quote::ToTokens;
use serde::Serialize;
use syn::{
    parse::Parser, punctuated::Punctuated, spanned::Spanned, visit::Visit, Attribute, Item, Macro,
    Meta,
};

pub use analyzer::{AnalyzerMode, AnalyzerReport, SemanticReport};
pub use feedback::{
    check_workspace, write_report, CheckDiagnostic, CheckOptions, CheckReport, CheckSpan,
    CheckSuggestion, CheckTarget, FeedbackHazard, FeedbackWideningCandidate,
    FeedbackWideningReport,
};
pub use model::{
    CallableId, ItemId, RootId, SemanticDependencies, SemanticOwnerId, SemanticReductionHints,
    SourceSpan,
};
pub use preflight::{
    preflight_workspace, write_preflight_report, PreflightDiagnostic, PreflightOptions,
    PreflightReport,
};
pub use repair::{repair_workspace, write_repair_report, RepairOptions, RepairReport};

#[derive(Debug, Clone)]
pub struct GenerateOptions {
    pub workspace_root: PathBuf,
    pub output_root: PathBuf,
}

#[derive(Debug, Clone)]
pub struct GenerateReport {
    pub analyzer: AnalyzerReport,
    pub production: ProductionReadinessReport,
    pub root: RootId,
    pub roots: Vec<RootId>,
    pub feedback_widened_roots: Vec<RootId>,
    pub packages: Vec<String>,
    pub targets: Vec<GeneratedTargetReport>,
    pub reachable: Vec<CallableId>,
    pub reachable_items: Vec<ItemId>,
    pub source_map: SourceMapReport,
    pub files_written: usize,
    pub timings: GenerateTimingReport,
}

#[derive(Debug, Clone)]
pub struct GeneratedTargetReport {
    pub package: String,
    pub name: String,
    pub kind: Vec<String>,
    pub src_path: PathBuf,
    pub required_features: Vec<String>,
    pub default_features: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct GenerateTimingReport {
    pub total_ms: u64,
    pub analyzer_ms: u64,
    pub manifest_ms: u64,
    pub parse_ms: u64,
    pub reduce_ms: u64,
    pub render_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProductionReadinessReport {
    pub status: String,
    pub hazards: Vec<ProductionHazardReport>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProductionHazardReport {
    pub code: String,
    pub severity: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub details: Vec<ProductionHazardDetail>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProductionHazardDetail {
    pub subject: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub module_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_line: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cfg: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suggested_cargo_args: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SourceMapReport {
    pub callables: Vec<CallableLocation>,
    pub items: Vec<ItemLocation>,
}

#[derive(Debug, Clone)]
pub struct CallableLocation {
    pub id: CallableId,
    pub reachable: bool,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct ItemLocation {
    pub id: ItemId,
    pub reachable: bool,
    pub span: SourceSpan,
}

pub fn generate(options: GenerateOptions) -> Result<GenerateReport, Box<dyn std::error::Error>> {
    generate_with_analyzer(options, AnalyzerMode::Syn)
}

pub fn generate_with_analyzer(
    options: GenerateOptions,
    analyzer_mode: AnalyzerMode,
) -> Result<GenerateReport, Box<dyn std::error::Error>> {
    generate_with_analyzer_feedback(options, analyzer_mode, &[])
}

pub fn generate_with_analyzer_feedback(
    options: GenerateOptions,
    analyzer_mode: AnalyzerMode,
    feedback_diagnostics: &[feedback::CheckDiagnostic],
) -> Result<GenerateReport, Box<dyn std::error::Error>> {
    let total_started = Instant::now();
    let phase_started = Instant::now();
    let workspace = manifest::load_workspace(&options.workspace_root)?;
    let manifest_ms = elapsed_ms(phase_started);

    let phase_started = Instant::now();
    let project = parse::parse_workspace(workspace)?;
    let parse_ms = elapsed_ms(phase_started);

    let phase_started = Instant::now();
    let analyzer =
        analyzer::load_report_for_project(&options.workspace_root, analyzer_mode, &project)?;
    let analyzer_ms = elapsed_ms(phase_started);

    let feedback_widened_roots = feedback_extra_roots(&project, feedback_diagnostics);

    let phase_started = Instant::now();
    let reduced = if analyzer.semantic_hints.is_empty() {
        reduce::reduce_with_extra_roots(&project, &feedback_widened_roots)?
    } else {
        reduce::reduce_with_extra_roots_and_semantics(
            &project,
            &feedback_widened_roots,
            &analyzer.semantic_hints,
        )?
    };
    let reduce_ms = elapsed_ms(phase_started);

    let phase_started = Instant::now();
    let files_written = render::write_reduced_workspace(&project, &reduced, &options.output_root)?;
    let render_ms = elapsed_ms(phase_started);
    let timings = GenerateTimingReport {
        total_ms: elapsed_ms(total_started),
        analyzer_ms,
        manifest_ms,
        parse_ms,
        reduce_ms,
        render_ms,
    };

    let mut packages = reduced.packages.iter().cloned().collect::<Vec<_>>();
    packages.sort();

    let mut reachable = reduced.reachable.iter().cloned().collect::<Vec<_>>();
    reachable.sort();

    let mut reachable_items = reduced.reachable_items.iter().cloned().collect::<Vec<_>>();
    reachable_items.sort();
    let targets = target_report(&project, &packages);
    let source_map = source_map_report(&project, &reduced);
    let production = production_readiness_report(&analyzer, &project, &reduced);

    Ok(GenerateReport {
        analyzer,
        production,
        root: reduced.root,
        roots: reduced.roots,
        feedback_widened_roots,
        packages,
        targets,
        reachable,
        reachable_items,
        source_map,
        files_written,
        timings,
    })
}

fn target_report(project: &Project, packages: &[String]) -> Vec<GeneratedTargetReport> {
    packages
        .iter()
        .filter_map(|package_name| {
            let package = project.workspace.packages.get(package_name)?;
            Some(GeneratedTargetReport {
                package: package_name.clone(),
                name: package.entry_target.name.clone(),
                kind: package.entry_target.kind.clone(),
                src_path: package.entry_target.src_path.clone(),
                required_features: package.entry_target.required_features.clone(),
                default_features: default_feature_closure(&package.manifest),
            })
        })
        .collect()
}

fn default_feature_closure(manifest: &toml::Value) -> Vec<String> {
    let Some(features) = manifest.get("features").and_then(toml::Value::as_table) else {
        return Vec::new();
    };
    let mut retained = BTreeSet::new();
    let mut visited = BTreeSet::new();
    let mut pending = vec!["default".to_string()];

    while let Some(feature) = pending.pop() {
        if !visited.insert(feature.clone()) {
            continue;
        }
        let Some(values) = features.get(&feature).and_then(toml::Value::as_array) else {
            continue;
        };
        for value in values {
            let Some(item) = value.as_str() else {
                continue;
            };
            let Some(feature_name) = package_feature_reference(item) else {
                continue;
            };
            if retained.insert(feature_name.to_string()) {
                pending.push(feature_name.to_string());
            }
        }
    }

    retained.into_iter().collect()
}

fn package_feature_reference(item: &str) -> Option<&str> {
    let feature = item.strip_prefix("dep:").unwrap_or(item);
    let feature = feature
        .split_once('/')
        .map(|(dependency, _)| dependency)
        .or_else(|| feature.split_once("?/").map(|(dependency, _)| dependency))
        .unwrap_or(feature);
    let feature = feature.strip_suffix('?').unwrap_or(feature);
    (!feature.is_empty()).then_some(feature)
}

const FEEDBACK_WIDENING_ROOT_MATCH_LIMIT: usize = 24;

fn feedback_extra_roots(
    project: &Project,
    diagnostics: &[feedback::CheckDiagnostic],
) -> Vec<RootId> {
    let mut roots = Vec::new();
    let mut seen = BTreeSet::new();
    for diagnostic in diagnostics {
        if diagnostic.level != "error" {
            continue;
        }
        let Some(code) = diagnostic.code.as_deref() else {
            continue;
        };
        let symbols = diagnostic_symbols(diagnostic);
        if symbols.is_empty() {
            continue;
        }
        let package_hint = diagnostic_package_hint(diagnostic);
        for symbol in symbols {
            let Some(name) = symbol_leaf_name(&symbol) else {
                continue;
            };
            let candidates = feedback_root_candidates(project, code, package_hint.as_deref(), name);
            if candidates.is_empty() || candidates.len() > FEEDBACK_WIDENING_ROOT_MATCH_LIMIT {
                continue;
            }
            for root in candidates {
                if !root_is_marked(project, &root) && seen.insert(root.clone()) {
                    roots.push(root);
                }
            }
        }
    }
    roots.sort();
    roots
}

fn feedback_root_candidates(
    project: &Project,
    code: &str,
    package_hint: Option<&str>,
    name: &str,
) -> Vec<RootId> {
    let mut roots = Vec::new();
    match code {
        "E0405" => {
            roots.extend(item_name_candidates(project, package_hint, name, |item| {
                item.kind == model::ItemKind::Trait
            }));
        }
        "E0412" | "E0422" => {
            roots.extend(item_name_candidates(project, package_hint, name, |item| {
                matches!(
                    item.kind,
                    model::ItemKind::Struct
                        | model::ItemKind::Enum
                        | model::ItemKind::Union
                        | model::ItemKind::Type
                        | model::ItemKind::Trait
                        | model::ItemKind::Mod
                )
            }));
        }
        "E0425" => {
            roots.extend(free_function_name_candidates(project, package_hint, name));
            roots.extend(item_name_candidates(project, package_hint, name, |item| {
                matches!(item.kind, model::ItemKind::Const | model::ItemKind::Static)
            }));
        }
        "E0432" | "E0433" => {
            roots.extend(free_function_name_candidates(project, package_hint, name));
            roots.extend(item_name_candidates(project, package_hint, name, |_| true));
        }
        "E0599" => {
            roots.extend(method_name_candidates(project, package_hint, name));
        }
        _ => {}
    }
    roots.sort();
    roots.dedup();
    roots
}

fn free_function_name_candidates(
    project: &Project,
    package_hint: Option<&str>,
    name: &str,
) -> Vec<RootId> {
    project
        .functions
        .keys()
        .filter(|callable| callable.package_matches(package_hint))
        .filter(|callable| match callable {
            CallableId::Free {
                name: candidate, ..
            } => candidate == name,
            CallableId::Method { .. } => false,
        })
        .cloned()
        .map(RootId::Callable)
        .collect()
}

fn method_name_candidates(
    project: &Project,
    package_hint: Option<&str>,
    name: &str,
) -> Vec<RootId> {
    project
        .methods
        .keys()
        .filter(|callable| callable.package_matches(package_hint))
        .filter(|callable| match callable {
            CallableId::Method { method, .. } => method == name,
            CallableId::Free { .. } => false,
        })
        .cloned()
        .map(RootId::Callable)
        .collect()
}

fn item_name_candidates(
    project: &Project,
    package_hint: Option<&str>,
    name: &str,
    kind_matches: impl Fn(&ItemId) -> bool,
) -> Vec<RootId> {
    project
        .items
        .keys()
        .filter(|item| item.package_matches(package_hint))
        .filter(|item| item.name == name && kind_matches(item))
        .cloned()
        .map(RootId::Item)
        .collect()
}

trait PackageMatch {
    fn package_matches(&self, package_hint: Option<&str>) -> bool;
}

impl PackageMatch for CallableId {
    fn package_matches(&self, package_hint: Option<&str>) -> bool {
        package_hint.is_none_or(|package| self.package() == package)
    }
}

impl PackageMatch for ItemId {
    fn package_matches(&self, package_hint: Option<&str>) -> bool {
        package_hint.is_none_or(|package| self.package() == package)
    }
}

fn diagnostic_symbols(diagnostic: &feedback::CheckDiagnostic) -> Vec<String> {
    let mut symbols = backticked_symbols(&diagnostic.message);
    if let Some(rendered) = &diagnostic.rendered {
        symbols.extend(backticked_symbols(rendered));
    }
    symbols.sort();
    symbols.dedup();
    symbols
}

fn backticked_symbols(text: &str) -> Vec<String> {
    let mut symbols = Vec::new();
    let mut remaining = text;
    while let Some((_, tail)) = remaining.split_once('`') {
        let Some((symbol, after)) = tail.split_once('`') else {
            break;
        };
        if !symbol.is_empty() {
            symbols.push(symbol.to_string());
        }
        remaining = after;
    }
    symbols
}

fn symbol_leaf_name(symbol: &str) -> Option<&str> {
    symbol
        .split('<')
        .next()
        .unwrap_or(symbol)
        .rsplit("::")
        .find(|segment| !matches!(*segment, "" | "crate" | "self" | "super"))
        .filter(|segment| {
            segment
                .chars()
                .next()
                .is_some_and(|character| character.is_ascii_alphabetic() || character == '_')
        })
}

fn diagnostic_package_hint(diagnostic: &feedback::CheckDiagnostic) -> Option<String> {
    diagnostic
        .package_id
        .as_deref()
        .and_then(package_name_from_diagnostic_package_id)
        .or_else(|| diagnostic.target.as_ref().map(|target| target.name.clone()))
}

fn package_name_from_diagnostic_package_id(package_id: &str) -> Option<String> {
    if let Some(fragment) = package_id.split('#').next_back() {
        if let Some((name, _)) = fragment.split_once('@') {
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }
    package_id
        .split_whitespace()
        .next()
        .map(str::to_string)
        .filter(|name| !name.is_empty())
}

fn root_is_marked(project: &Project, root: &RootId) -> bool {
    root_direct_attrs(project, root).is_some_and(|attrs| {
        attrs
            .iter()
            .any(|attribute| reduce::is_opensourced_attr(attribute.path()))
    })
}

fn elapsed_ms(started: Instant) -> u64 {
    started.elapsed().as_millis().try_into().unwrap_or(u64::MAX)
}

fn source_map_report(project: &model::Project, reduced: &model::ReducedProject) -> SourceMapReport {
    let mut callables = project
        .functions
        .values()
        .map(|record| CallableLocation {
            id: record.id.clone(),
            reachable: reduced.reachable.contains(&record.id),
            span: record.span.clone(),
        })
        .chain(project.methods.iter().map(|(id, record)| CallableLocation {
            id: id.clone(),
            reachable: reduced.reachable.contains(id),
            span: record.span.clone(),
        }))
        .collect::<Vec<_>>();
    callables.sort_by_key(|location| location.id.to_string());

    let mut items = project
        .items
        .iter()
        .map(|(id, record)| ItemLocation {
            id: id.clone(),
            reachable: reduced.reachable_items.contains(id),
            span: record.span.clone(),
        })
        .collect::<Vec<_>>();
    items.sort_by_key(|location| location.id.to_string());

    SourceMapReport { callables, items }
}

fn production_readiness_report(
    analyzer: &AnalyzerReport,
    project: &Project,
    reduced: &ReducedProject,
) -> ProductionReadinessReport {
    let mut hazards = Vec::new();
    add_workspace_production_hazards(project, reduced, &mut hazards);
    add_cfg_gated_root_production_hazards(project, reduced, &mut hazards);
    add_syntactic_production_hazards(project, reduced, &mut hazards);
    add_reduction_evidence_production_hazards(reduced, &mut hazards);

    if !analyzer.loaded {
        hazards.push(production_hazard(
            "analyzer_unavailable",
            "error",
            "configured analyzer did not load; generated reachability used fallback evidence",
        ));
    }
    add_semantic_inventory_hazard(
        analyzer,
        reduced.evidence.semantic_edges_applied,
        &mut hazards,
    );

    let Some(semantic) = &analyzer.semantic else {
        hazards.push(production_hazard(
            "semantic_analyzer_unavailable",
            "warning",
            "no semantic analyzer inventory was available; compiler feedback is required before trusting the slice",
        ));
        return production_readiness_status(hazards);
    };

    if semantic.failed_files > 0 {
        hazards.push(production_hazard(
            "semantic_file_failures",
            "warning",
            format!(
                "{} source file(s) failed semantic analysis",
                semantic.failed_files
            ),
        ));
    }
    if semantic.skipped_files > 0 {
        hazards.push(production_hazard(
            "semantic_file_budget_exhausted",
            "warning",
            format!(
                "{} source file(s) were skipped by semantic analysis budget",
                semantic.skipped_files
            ),
        ));
    }
    if semantic.unresolved_method_calls > 0 {
        hazards.push(production_hazard(
            "semantic_unresolved_method_calls",
            "warning",
            format!(
                "{} queried method call(s) did not resolve semantically",
                semantic.unresolved_method_calls
            ),
        ));
    }
    if semantic.unqueried_method_calls > 0 {
        hazards.push(production_hazard(
            "semantic_method_call_budget_exhausted",
            "warning",
            format!(
                "{} method call(s) were not queried because the semantic budget was exhausted",
                semantic.unqueried_method_calls
            ),
        ));
    }
    if semantic.unresolved_paths > 0 {
        hazards.push(production_hazard(
            "semantic_unresolved_paths",
            "warning",
            format!(
                "{} queried path(s) did not resolve semantically",
                semantic.unresolved_paths
            ),
        ));
    }
    if semantic.unqueried_paths > 0 {
        hazards.push(production_hazard(
            "semantic_path_budget_exhausted",
            "warning",
            format!(
                "{} path(s) were not queried because the semantic budget was exhausted",
                semantic.unqueried_paths
            ),
        ));
    }

    production_readiness_status(hazards)
}

fn add_semantic_inventory_hazard(
    analyzer: &AnalyzerReport,
    semantic_edges_applied: usize,
    hazards: &mut Vec<ProductionHazardReport>,
) {
    if analyzer.semantic.is_some() && analyzer.semantic_hints.is_empty() {
        hazards.push(production_hazard(
            "semantic_inventory_not_applied",
            "warning",
            "semantic analyzer inventory is report-only in this build; compiler feedback is still required before trusting the slice",
        ));
    } else if analyzer.semantic.is_some() && semantic_edges_applied > 0 {
        hazards.push(production_hazard(
            "semantic_inventory_partially_applied",
            "warning",
            format!(
                "semantic analyzer applied {} reachable project-local reduction edge(s) from {} available edge(s), but compiler feedback is still required for unresolved, unqueried, unmapped, macro-expanded, and generated-code cases",
                semantic_edges_applied,
                analyzer.semantic_hints.total_edges()
            ),
        ));
    } else if analyzer.semantic.is_some() {
        hazards.push(production_hazard(
            "semantic_inventory_available",
            "warning",
            format!(
                "semantic analyzer produced {} project-local reduction edge(s), but none were reached by the selected roots; compiler feedback is still required for unresolved, unqueried, unmapped, macro-expanded, and generated-code cases",
                analyzer.semantic_hints.total_edges()
            ),
        ));
    }
}

fn add_workspace_production_hazards(
    project: &Project,
    reduced: &ReducedProject,
    hazards: &mut Vec<ProductionHazardReport>,
) {
    let retained_build_script_details = reduced
        .packages
        .iter()
        .filter_map(|package| project.workspace.packages.get(package))
        .filter_map(retained_build_script_detail)
        .collect::<Vec<_>>();
    if !retained_build_script_details.is_empty() {
        hazards.push(production_hazard_with_details(
            "retained_build_scripts",
            "error",
            format!(
                "{} retained package build script(s) may generate source, link metadata, env values, or asset dependencies outside the static parse tree",
                retained_build_script_details.len()
            ),
            retained_build_script_details,
        ));
    }
    let retained_path_dependency_details =
        retained_non_workspace_path_dependency_details(project, reduced);
    if !retained_path_dependency_details.is_empty() {
        hazards.push(production_hazard_with_details(
            "retained_non_workspace_path_dependencies",
            "error",
            format!(
                "{} retained non-workspace path dependency reference(s) would keep the slice tied to the original checkout: {}",
                retained_path_dependency_details.len(),
                retained_path_dependency_details
                    .iter()
                    .map(|detail| detail.subject.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            retained_path_dependency_details,
        ));
    }
    let retained_workspace_patch_path_details =
        retained_workspace_patch_replace_path_dependency_details(project);
    if !retained_workspace_patch_path_details.is_empty() {
        hazards.push(production_hazard_with_details(
            "retained_workspace_patch_replace_path_dependencies",
            "error",
            format!(
                "{} retained workspace patch/replace path entry/entries would keep the slice tied to the original checkout: {}",
                retained_workspace_patch_path_details.len(),
                retained_workspace_patch_path_details
                    .iter()
                    .map(|detail| detail.subject.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            retained_workspace_patch_path_details,
        ));
    }
}

fn retained_build_script_detail(
    package: &crate::manifest::Package,
) -> Option<ProductionHazardDetail> {
    let path = package_build_script_path(package)?;
    Some(ProductionHazardDetail {
        subject: package.name.clone(),
        package: Some(package.name.clone()),
        module_path: None,
        file: Some(path),
        start_line: Some(1),
        cfg: None,
        suggested_cargo_args: Vec::new(),
    })
}

fn retained_workspace_patch_replace_path_dependency_details(
    project: &Project,
) -> Vec<ProductionHazardDetail> {
    let mut dependencies = BTreeMap::new();
    let manifest_path = project.workspace.root.join("Cargo.toml");
    if let Some(patches) = project
        .workspace
        .manifest
        .get("patch")
        .and_then(toml::Value::as_table)
    {
        for (source, value) in patches {
            collect_uncopyable_manifest_path_dependencies(
                &mut dependencies,
                &format!("patch.{source}"),
                value,
                &project.workspace.root,
                &manifest_path,
                None,
                None,
            );
        }
    }
    if let Some(replacements) = project
        .workspace
        .manifest
        .get("replace")
        .and_then(toml::Value::as_table)
    {
        for (name, value) in replacements {
            collect_uncopyable_manifest_path_dependencies(
                &mut dependencies,
                &format!("replace.{name}"),
                value,
                &project.workspace.root,
                &manifest_path,
                None,
                Some(name),
            );
        }
    }
    dependencies.into_values().collect()
}

fn collect_uncopyable_manifest_path_dependencies(
    dependencies: &mut BTreeMap<String, ProductionHazardDetail>,
    prefix: &str,
    value: &toml::Value,
    manifest_dir: &Path,
    manifest_path: &Path,
    package: Option<&str>,
    line_key: Option<&str>,
) {
    let Some(table) = value.as_table() else {
        return;
    };
    if table.get("path").and_then(toml::Value::as_str).is_some() {
        if !path_dependency_is_copyable(value, manifest_dir) {
            dependencies.entry(prefix.to_string()).or_insert_with(|| {
                manifest_path_dependency_detail(prefix, package, manifest_path, line_key)
            });
        }
        return;
    }
    for (name, value) in table {
        collect_uncopyable_manifest_path_dependencies(
            dependencies,
            &format!("{prefix}.{name}"),
            value,
            manifest_dir,
            manifest_path,
            package,
            Some(name),
        );
    }
}

fn retained_non_workspace_path_dependency_details(
    project: &Project,
    reduced: &ReducedProject,
) -> Vec<ProductionHazardDetail> {
    let mut dependencies = BTreeMap::new();
    for package_name in &reduced.packages {
        let Some(package) = project.workspace.packages.get(package_name) else {
            continue;
        };
        for (table_name, table) in package_dependency_tables(package) {
            for (alias, value) in table {
                let (source, manifest_dir, manifest_path) =
                    workspace_resolved_dependency_value(project, alias, value, &package.root);
                if !dependency_has_path(source) {
                    continue;
                }
                let dependency_package = dependency_package_name(alias, source);
                if is_marker_dependency(alias, &dependency_package)
                    || project.workspace.packages.contains_key(&dependency_package)
                {
                    continue;
                }
                if retained_source_mentions_dependency(
                    project,
                    reduced,
                    package_name,
                    alias,
                    &dependency_package,
                ) && !path_dependency_is_copyable(source, manifest_dir)
                {
                    let subject = format!("{package_name}.{table_name}.{alias}");
                    dependencies.entry(subject.clone()).or_insert_with(|| {
                        manifest_path_dependency_detail(
                            &subject,
                            Some(package_name),
                            &manifest_path,
                            Some(alias),
                        )
                    });
                }
            }
        }
    }
    dependencies.into_values().collect()
}

fn manifest_path_dependency_detail(
    subject: &str,
    package: Option<&str>,
    manifest_path: &Path,
    line_key: Option<&str>,
) -> ProductionHazardDetail {
    ProductionHazardDetail {
        subject: subject.to_string(),
        package: package.map(str::to_string),
        module_path: None,
        file: Some(manifest_path.to_path_buf()),
        start_line: line_key.and_then(|key| manifest_key_line(manifest_path, key)),
        cfg: None,
        suggested_cargo_args: Vec::new(),
    }
}

fn manifest_key_line(manifest_path: &Path, key: &str) -> Option<usize> {
    let text = fs::read_to_string(manifest_path).ok()?;
    text.lines()
        .position(|line| manifest_line_starts_with_key(line, key))
        .map(|line| line + 1)
}

fn manifest_line_starts_with_key(line: &str, key: &str) -> bool {
    let trimmed = line.trim_start();
    let rest = trimmed
        .strip_prefix(key)
        .or_else(|| trimmed.strip_prefix(&format!("{key:?}")));
    rest.is_some_and(|rest| {
        rest.trim_start()
            .chars()
            .next()
            .is_some_and(|ch| matches!(ch, '=' | '.' | '{' | '['))
    })
}

fn package_dependency_tables(
    package: &crate::manifest::Package,
) -> Vec<(String, &toml::value::Table)> {
    let mut tables = Vec::new();
    for table_name in ["dependencies", "build-dependencies", "dev-dependencies"] {
        if let Some(table) = package
            .manifest
            .get(table_name)
            .and_then(toml::Value::as_table)
        {
            tables.push((table_name.to_string(), table));
        }
    }
    if let Some(targets) = package
        .manifest
        .get("target")
        .and_then(toml::Value::as_table)
    {
        for (target_name, target) in targets {
            let Some(target_table) = target.as_table() else {
                continue;
            };
            for dependency_table_name in ["dependencies", "build-dependencies", "dev-dependencies"]
            {
                if let Some(table) = target_table
                    .get(dependency_table_name)
                    .and_then(toml::Value::as_table)
                {
                    tables.push((
                        format!("target.{target_name}.{dependency_table_name}"),
                        table,
                    ));
                }
            }
        }
    }
    tables
}

fn workspace_resolved_dependency_value<'a>(
    project: &'a Project,
    alias: &str,
    value: &'a toml::Value,
    package_root: &'a Path,
) -> (&'a toml::Value, &'a Path, PathBuf) {
    let package_manifest = package_root.join("Cargo.toml");
    if !dependency_uses_workspace(value) {
        return (value, package_root, package_manifest);
    }
    project
        .workspace
        .manifest
        .get("workspace")
        .and_then(|workspace| workspace.get("dependencies"))
        .and_then(toml::Value::as_table)
        .and_then(|dependencies| dependencies.get(alias))
        .map(|value| {
            (
                value,
                project.workspace.root.as_path(),
                project.workspace.root.join("Cargo.toml"),
            )
        })
        .unwrap_or((value, package_root, package_manifest))
}

fn path_dependency_is_copyable(value: &toml::Value, manifest_dir: &Path) -> bool {
    let Some(path) = value
        .as_table()
        .and_then(|table| table.get("path"))
        .and_then(toml::Value::as_str)
    else {
        return false;
    };
    let path = PathBuf::from(path);
    let path = if path.is_absolute() {
        path
    } else {
        manifest_dir.join(path)
    };
    path.canonicalize()
        .is_ok_and(|root| root.join("Cargo.toml").is_file())
}

fn dependency_uses_workspace(value: &toml::Value) -> bool {
    value
        .as_table()
        .and_then(|table| table.get("workspace"))
        .and_then(toml::Value::as_bool)
        .unwrap_or(false)
}

fn dependency_has_path(value: &toml::Value) -> bool {
    value
        .as_table()
        .and_then(|table| table.get("path"))
        .and_then(toml::Value::as_str)
        .is_some()
}

fn dependency_package_name(alias: &str, value: &toml::Value) -> String {
    value
        .as_table()
        .and_then(|table| table.get("package"))
        .and_then(toml::Value::as_str)
        .unwrap_or(alias)
        .to_string()
}

fn is_marker_dependency(alias: &str, package: &str) -> bool {
    alias == "opensourced" || package == "opensourced"
}

fn retained_source_mentions_dependency(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    alias: &str,
    dependency_package: &str,
) -> bool {
    let names = dependency_mention_names(alias, dependency_package);
    reduced.reachable.iter().any(|callable| {
        callable.package() == package
            && callable_tokens(project, callable).is_some_and(|tokens| {
                names
                    .iter()
                    .any(|name| token_stream_mentions_ident(&tokens, name))
            })
    }) || reduced.reachable_items.iter().any(|item| {
        item.package == package
            && project.items.get(item).is_some_and(|record| {
                let tokens = record.item.to_token_stream();
                names
                    .iter()
                    .any(|name| token_stream_mentions_ident(&tokens, name))
            })
    })
}

fn callable_tokens(project: &Project, callable: &crate::model::CallableId) -> Option<TokenStream> {
    project
        .functions
        .get(callable)
        .map(|record| record.item.to_token_stream())
        .or_else(|| {
            project
                .methods
                .get(callable)
                .map(|record| record.item.to_token_stream())
        })
}

fn dependency_mention_names(alias: &str, package: &str) -> BTreeSet<String> {
    [alias, package]
        .into_iter()
        .flat_map(|name| [name.to_string(), name.replace('-', "_")])
        .collect()
}

fn token_stream_mentions_ident(tokens: &TokenStream, ident: &str) -> bool {
    tokens.clone().into_iter().any(|token| match token {
        proc_macro2::TokenTree::Ident(candidate) => candidate == ident,
        proc_macro2::TokenTree::Group(group) => token_stream_mentions_ident(&group.stream(), ident),
        proc_macro2::TokenTree::Punct(_) | proc_macro2::TokenTree::Literal(_) => false,
    })
}

fn add_cfg_gated_root_production_hazards(
    project: &Project,
    reduced: &ReducedProject,
    hazards: &mut Vec<ProductionHazardReport>,
) {
    let mut cfg_gated_roots = 0;
    let mut details = Vec::new();
    for root in &reduced.roots {
        let root_details = cfg_gated_root_details(project, root);
        if root_details.is_empty() {
            continue;
        }
        cfg_gated_roots += 1;
        details.extend(root_details);
    }
    if !details.is_empty() {
        hazards.push(production_hazard_with_details(
            "cfg_gated_roots",
            "error",
            format!(
                "{} selected root(s) are behind cfg/cfg_attr gates; production validation must prove the exact feature and target matrix before accepting the slice",
                cfg_gated_roots
            ),
            details,
        ));
    }
}

fn cfg_gated_root_details(project: &Project, root: &RootId) -> Vec<ProductionHazardDetail> {
    let Some((package, module_path)) = root_module_location(project, root) else {
        return Vec::new();
    };
    let mut details = Vec::new();
    for attribute in root_direct_attrs(project, root)
        .unwrap_or_default()
        .iter()
        .filter(|attribute| attr_is_non_test_cfg_gate(attribute))
    {
        details.push(cfg_gate_detail(
            project,
            root,
            package,
            module_path,
            attribute,
        ));
    }
    for (gated_module_path, attribute) in module_path_cfg_gates(project, package, module_path) {
        details.push(cfg_gate_detail(
            project,
            root,
            package,
            &gated_module_path,
            attribute,
        ));
    }
    details
}

fn root_module_location<'a>(
    project: &'a Project,
    root: &'a RootId,
) -> Option<(&'a str, &'a [String])> {
    match root {
        RootId::Callable(callable) => project
            .functions
            .get(callable)
            .map(|record| (record.package.as_str(), record.module_path.as_slice()))
            .or_else(|| {
                project
                    .methods
                    .get(callable)
                    .map(|record| (callable.package(), record.module_path.as_slice()))
            }),
        RootId::Item(item) => project
            .items
            .get(item)
            .map(|record| (record.package.as_str(), record.module_path.as_slice())),
    }
}

fn root_direct_attrs<'a>(project: &'a Project, root: &RootId) -> Option<&'a [Attribute]> {
    match root {
        RootId::Callable(callable) => project
            .functions
            .get(callable)
            .map(|record| record.item.attrs.as_slice())
            .or_else(|| {
                project
                    .methods
                    .get(callable)
                    .map(|record| record.item.attrs.as_slice())
            }),
        RootId::Item(item) => project
            .items
            .get(item)
            .map(|record| item_attrs(&record.item)),
    }
}

fn item_attrs(item: &Item) -> &[Attribute] {
    match item {
        Item::Const(item) => &item.attrs,
        Item::Enum(item) => &item.attrs,
        Item::Macro(item) => &item.attrs,
        Item::Mod(item) => &item.attrs,
        Item::Static(item) => &item.attrs,
        Item::Struct(item) => &item.attrs,
        Item::Trait(item) => &item.attrs,
        Item::Type(item) => &item.attrs,
        Item::Union(item) => &item.attrs,
        _ => &[],
    }
}

fn module_path_cfg_gates<'a>(
    project: &'a Project,
    package: &str,
    module_path: &[String],
) -> Vec<(Vec<String>, &'a Attribute)> {
    let mut gates = Vec::new();
    for depth in 1..=module_path.len() {
        let Some(item_mod) = module_item_for_path(project, package, &module_path[..depth]) else {
            continue;
        };
        gates.extend(
            item_mod
                .attrs
                .iter()
                .filter(|attribute| attr_is_non_test_cfg_gate(attribute))
                .map(|attribute| (module_path[..depth].to_vec(), attribute)),
        );
    }
    gates
}

fn module_item_for_path<'a>(
    project: &'a Project,
    package: &str,
    module_path: &[String],
) -> Option<&'a syn::ItemMod> {
    let (module_name, parent_path) = module_path.split_last()?;
    let source = project
        .files
        .values()
        .find(|source| source.package == package && source.module_path == parent_path)?;
    source.syntax.items.iter().find_map(|item| {
        let Item::Mod(item_mod) = item else {
            return None;
        };
        (item_mod.ident == module_name.as_str()).then_some(item_mod)
    })
}

fn attr_is_non_test_cfg_gate(attribute: &Attribute) -> bool {
    (attribute.path().is_ident("cfg") && !reduce::is_cfg_test_attr(attribute))
        || attribute.path().is_ident("cfg_attr")
}

fn cfg_gate_detail(
    project: &Project,
    root: &RootId,
    package: &str,
    module_path: &[String],
    attribute: &Attribute,
) -> ProductionHazardDetail {
    let span = root_span(project, root);
    ProductionHazardDetail {
        subject: root.to_string(),
        package: Some(package.to_string()),
        module_path: (!module_path.is_empty()).then(|| module_path.join("::")),
        file: span.as_ref().map(|span| span.file.clone()),
        start_line: span.as_ref().map(|span| span.start_line),
        cfg: Some(attribute.to_token_stream().to_string()),
        suggested_cargo_args: cfg_gate_suggested_cargo_args(attribute),
    }
}

fn root_span(project: &Project, root: &RootId) -> Option<SourceSpan> {
    match root {
        RootId::Callable(callable) => project
            .functions
            .get(callable)
            .map(|record| record.span.clone())
            .or_else(|| {
                project
                    .methods
                    .get(callable)
                    .map(|record| record.span.clone())
            }),
        RootId::Item(item) => project.items.get(item).map(|record| record.span.clone()),
    }
}

fn cfg_gate_suggested_cargo_args(attribute: &Attribute) -> Vec<String> {
    let mut features = cfg_gate_feature_names(attribute)
        .into_iter()
        .collect::<Vec<_>>();
    features.sort();
    if features.is_empty() {
        Vec::new()
    } else {
        vec!["--features".to_string(), features.join(",")]
    }
}

fn cfg_gate_feature_names(attribute: &Attribute) -> BTreeSet<String> {
    let mut features = BTreeSet::new();
    if attribute.path().is_ident("cfg") {
        if let Ok(meta) = attribute.parse_args::<Meta>() {
            collect_cfg_feature_names(&meta, &mut features);
        }
    } else if attribute.path().is_ident("cfg_attr") {
        if let Ok(arguments) =
            attribute.parse_args_with(Punctuated::<Meta, syn::Token![,]>::parse_terminated)
        {
            if let Some(predicate) = arguments.first() {
                collect_cfg_feature_names(predicate, &mut features);
            }
        }
    }
    features
}

fn collect_cfg_feature_names(meta: &Meta, features: &mut BTreeSet<String>) {
    match meta {
        Meta::NameValue(name_value) if name_value.path.is_ident("feature") => {
            if let syn::Expr::Lit(expr) = &name_value.value {
                if let syn::Lit::Str(feature) = &expr.lit {
                    features.insert(feature.value());
                }
            }
        }
        Meta::List(list) => {
            if let Ok(arguments) =
                Punctuated::<Meta, syn::Token![,]>::parse_terminated.parse2(list.tokens.clone())
            {
                for nested in arguments {
                    collect_cfg_feature_names(&nested, features);
                }
            }
        }
        Meta::Path(_) | Meta::NameValue(_) => {}
    }
}

fn package_build_script_path(package: &crate::manifest::Package) -> Option<PathBuf> {
    let package_table = package
        .manifest
        .get("package")
        .and_then(toml::Value::as_table);
    match package_table.and_then(|table| table.get("build")) {
        Some(toml::Value::Boolean(false)) => None,
        Some(toml::Value::String(path)) => {
            let path = package.root.join(path);
            path.exists().then_some(path)
        }
        _ => {
            let path = package.root.join("build.rs");
            path.exists().then_some(path)
        }
    }
}

fn add_syntactic_production_hazards(
    project: &Project,
    reduced: &ReducedProject,
    hazards: &mut Vec<ProductionHazardReport>,
) {
    let counts = syntactic_hazard_counts(project, reduced);
    if counts.source_include_macros > 0 {
        hazards.push(production_hazard_with_details(
            "source_include_macros",
            "error",
            format!(
                "{} retained include! macro(s) inject Rust source outside the static reachability graph",
                counts.source_include_macros
            ),
            counts.source_include_details,
        ));
    }
    if counts.out_dir_source_include_macros > 0 {
        hazards.push(production_hazard_with_details(
            "out_dir_source_include_macros",
            "error",
            format!(
                "{} retained include! macro(s) read generated Rust from OUT_DIR; production slicing cannot semantically model build-generated source",
                counts.out_dir_source_include_macros
            ),
            counts.out_dir_source_include_details,
        ));
    }
    if counts.nonliteral_file_include_macros > 0 {
        hazards.push(production_hazard_with_details(
            "nonliteral_file_include_macros",
            "error",
            format!(
                "{} retained include_str!/include_bytes! macro(s) use paths the slicer cannot statically resolve",
                counts.nonliteral_file_include_macros
            ),
            counts.nonliteral_file_include_details,
        ));
    }
    if counts.out_dir_file_include_macros > 0 {
        hazards.push(production_hazard_with_details(
            "out_dir_file_include_macros",
            "error",
            format!(
                "{} retained include_str!/include_bytes! macro(s) read generated files from OUT_DIR; production slicing cannot semantically model build-generated assets",
                counts.out_dir_file_include_macros
            ),
            counts.out_dir_file_include_details,
        ));
    }
    if counts.absolute_file_include_macros > 0 {
        hazards.push(production_hazard_with_details(
            "absolute_file_include_macros",
            "error",
            format!(
                "{} retained include_str!/include_bytes! macro(s) use absolute paths that would read outside the generated slice",
                counts.absolute_file_include_macros
            ),
            counts.absolute_file_include_details,
        ));
    }
    if counts.external_file_include_macros > 0 {
        hazards.push(production_hazard_with_details(
            "external_file_include_macros",
            "error",
            format!(
                "{} retained include_str!/include_bytes! macro(s) resolve outside their package root",
                counts.external_file_include_macros
            ),
            counts.external_file_include_details,
        ));
    }
    if counts.compile_env_macros > 0 {
        hazards.push(production_hazard_with_details(
            "compile_env_macros",
            "error",
            format!(
                "{} retained env!/option_env! macro(s) read compile-time environment outside the manifest model",
                counts.compile_env_macros
            ),
            counts.compile_env_details,
        ));
    }
    if counts.custom_attribute_macros > 0 {
        hazards.push(production_hazard_with_details(
            "custom_attribute_macros",
            "warning",
            format!(
                "{} retained custom attribute macro/helper attribute(s) require compiler feedback because they may rewrite source outside the static parse tree",
                counts.custom_attribute_macros
            ),
            counts.custom_attribute_details,
        ));
    }
    if counts.custom_derive_macros > 0 {
        hazards.push(production_hazard_with_details(
            "custom_derive_macros",
            "warning",
            format!(
                "{} retained custom derive macro(s) require compiler feedback because they may generate impls or bounds outside the static parse tree",
                counts.custom_derive_macros
            ),
            counts.custom_derive_details,
        ));
    }
    if counts.custom_macro_invocations > 0 {
        hazards.push(production_hazard_with_details(
            "custom_macro_invocations",
            "warning",
            format!(
                "{} retained non-builtin macro invocation(s) require compiler feedback because they may expand code outside the static parse tree",
                counts.custom_macro_invocations
            ),
            counts.custom_macro_invocation_details,
        ));
    }
    if counts.function_pointer_surfaces > 0 {
        hazards.push(production_hazard_with_details(
            "function_pointer_surfaces",
            "error",
            format!(
                "{} retained function pointer type surface(s) may hide callback edges outside the static call graph",
                counts.function_pointer_surfaces
            ),
            counts.function_pointer_details,
        ));
    }
    if counts.trait_object_surfaces > 0 {
        hazards.push(production_hazard_with_details(
            "trait_object_surfaces",
            "error",
            format!(
                "{} retained trait object surface(s) may hide dynamic dispatch edges outside the static call graph",
                counts.trait_object_surfaces
            ),
            counts.trait_object_details,
        ));
    }
    if counts.conditional_compilation_attrs > 0 {
        hazards.push(production_hazard_with_details(
            "conditional_compilation_attrs",
            "warning",
            format!(
                "{} retained cfg/cfg_attr attribute(s) require compiler feedback for the selected feature and target matrix before production acceptance",
                counts.conditional_compilation_attrs
            ),
            counts.conditional_compilation_details,
        ));
    }
}

fn add_reduction_evidence_production_hazards(
    reduced: &ReducedProject,
    hazards: &mut Vec<ProductionHazardReport>,
) {
    let evidence = &reduced.evidence;
    if evidence.unresolved_method_fallbacks > 0 {
        hazards.push(production_hazard(
            "syntactic_method_fallbacks",
            "warning",
            format!(
                "{} unresolved method call(s) used syntactic fallback analysis; {} candidate method(s) were retained by name and require compiler feedback validation",
                evidence.unresolved_method_fallbacks, evidence.unresolved_method_candidate_matches
            ),
        ));
    }
    if evidence.capped_unresolved_method_fallbacks > 0 {
        hazards.push(production_hazard(
            "syntactic_method_fallback_cap",
            "warning",
            format!(
                "{} unresolved method fallback(s) exceeded the name-only candidate cap; compiler feedback is required to detect any omitted method dependencies",
                evidence.capped_unresolved_method_fallbacks
            ),
        ));
    }
    if evidence.semantic_edges_applied > 0 {
        hazards.push(production_hazard(
            "semantic_reduction_hints_applied",
            "warning",
            format!(
                "{} semantic analyzer edge(s) were applied to the retained graph; compiler feedback is still required for unresolved semantic surfaces",
                evidence.semantic_edges_applied
            ),
        ));
    }
}

#[derive(Default)]
struct SyntacticHazardCounts {
    source_include_macros: usize,
    source_include_details: Vec<ProductionHazardDetail>,
    out_dir_source_include_macros: usize,
    out_dir_source_include_details: Vec<ProductionHazardDetail>,
    nonliteral_file_include_macros: usize,
    nonliteral_file_include_details: Vec<ProductionHazardDetail>,
    out_dir_file_include_macros: usize,
    out_dir_file_include_details: Vec<ProductionHazardDetail>,
    absolute_file_include_macros: usize,
    absolute_file_include_details: Vec<ProductionHazardDetail>,
    external_file_include_macros: usize,
    external_file_include_details: Vec<ProductionHazardDetail>,
    custom_attribute_macros: usize,
    custom_attribute_details: Vec<ProductionHazardDetail>,
    custom_derive_macros: usize,
    custom_derive_details: Vec<ProductionHazardDetail>,
    custom_macro_invocations: usize,
    custom_macro_invocation_details: Vec<ProductionHazardDetail>,
    compile_env_macros: usize,
    compile_env_details: Vec<ProductionHazardDetail>,
    function_pointer_surfaces: usize,
    function_pointer_details: Vec<ProductionHazardDetail>,
    trait_object_surfaces: usize,
    trait_object_details: Vec<ProductionHazardDetail>,
    conditional_compilation_attrs: usize,
    conditional_compilation_details: Vec<ProductionHazardDetail>,
}

impl SyntacticHazardCounts {
    fn add(&mut self, other: Self) {
        self.source_include_macros += other.source_include_macros;
        self.source_include_details
            .extend(other.source_include_details);
        self.out_dir_source_include_macros += other.out_dir_source_include_macros;
        self.out_dir_source_include_details
            .extend(other.out_dir_source_include_details);
        self.nonliteral_file_include_macros += other.nonliteral_file_include_macros;
        self.nonliteral_file_include_details
            .extend(other.nonliteral_file_include_details);
        self.out_dir_file_include_macros += other.out_dir_file_include_macros;
        self.out_dir_file_include_details
            .extend(other.out_dir_file_include_details);
        self.absolute_file_include_macros += other.absolute_file_include_macros;
        self.absolute_file_include_details
            .extend(other.absolute_file_include_details);
        self.external_file_include_macros += other.external_file_include_macros;
        self.external_file_include_details
            .extend(other.external_file_include_details);
        self.custom_attribute_macros += other.custom_attribute_macros;
        self.custom_attribute_details
            .extend(other.custom_attribute_details);
        self.custom_derive_macros += other.custom_derive_macros;
        self.custom_derive_details
            .extend(other.custom_derive_details);
        self.custom_macro_invocations += other.custom_macro_invocations;
        self.custom_macro_invocation_details
            .extend(other.custom_macro_invocation_details);
        self.compile_env_macros += other.compile_env_macros;
        self.compile_env_details.extend(other.compile_env_details);
        self.function_pointer_surfaces += other.function_pointer_surfaces;
        self.function_pointer_details
            .extend(other.function_pointer_details);
        self.trait_object_surfaces += other.trait_object_surfaces;
        self.trait_object_details.extend(other.trait_object_details);
        self.conditional_compilation_attrs += other.conditional_compilation_attrs;
        self.conditional_compilation_details
            .extend(other.conditional_compilation_details);
    }
}

fn syntactic_hazard_counts(project: &Project, reduced: &ReducedProject) -> SyntacticHazardCounts {
    let mut counts = SyntacticHazardCounts::default();
    counts.add(retained_module_boundary_hazard_counts(project, reduced));

    for callable in &reduced.reachable {
        if let Some(record) = project.functions.get(callable) {
            let mut visitor = syntactic_hazard_visitor_for_location(
                project,
                &record.package,
                &record.module_path,
            );
            visitor.visit_item_fn(&record.item);
            counts.add(visitor.counts);
        } else if let Some(record) = project.methods.get(callable) {
            let mut visitor = syntactic_hazard_visitor_for_location(
                project,
                callable.package(),
                &record.module_path,
            );
            visitor.visit_impl_item_fn(&record.item);
            counts.add(visitor.counts);
        }
    }

    for item in &reduced.reachable_items {
        if let Some(record) = project.items.get(item) {
            let mut visitor = syntactic_hazard_visitor_for_location(
                project,
                &record.package,
                &record.module_path,
            );
            visitor.visit_item(&record.item);
            counts.add(visitor.counts);
        }
    }

    counts
}

fn retained_module_boundary_hazard_counts(
    project: &Project,
    reduced: &ReducedProject,
) -> SyntacticHazardCounts {
    let mut module_boundaries = BTreeSet::<(String, Vec<String>)>::new();

    for callable in &reduced.reachable {
        if let Some(record) = project.functions.get(callable) {
            insert_module_boundary_paths(
                &mut module_boundaries,
                &record.package,
                &record.module_path,
            );
        } else if let Some(record) = project.methods.get(callable) {
            insert_module_boundary_paths(
                &mut module_boundaries,
                callable.package(),
                &record.module_path,
            );
        }
    }

    for item in &reduced.reachable_items {
        if let Some(record) = project.items.get(item) {
            insert_module_boundary_paths(
                &mut module_boundaries,
                &record.package,
                &record.module_path,
            );
        }
    }

    let mut counts = SyntacticHazardCounts::default();
    for (package, module_path) in module_boundaries {
        let Some(item_mod) = module_item_for_path(project, &package, &module_path) else {
            continue;
        };
        let file = module_path
            .split_last()
            .and_then(|(_, parent_module_path)| {
                source_for_module(project, &package, parent_module_path)
            })
            .map(|source| source.path.clone());
        let mut visitor = SyntacticHazardVisitor {
            counts: SyntacticHazardCounts::default(),
            include_context: None,
            location: HazardLocation {
                package,
                module_path,
                file,
            },
        };
        for attribute in &item_mod.attrs {
            visitor.visit_attribute(attribute);
        }
        counts.add(visitor.counts);
    }

    counts
}

fn insert_module_boundary_paths(
    module_boundaries: &mut BTreeSet<(String, Vec<String>)>,
    package: &str,
    module_path: &[String],
) {
    let mut prefix = Vec::new();
    for segment in module_path {
        prefix.push(segment.clone());
        module_boundaries.insert((package.to_string(), prefix.clone()));
    }
}

fn syntactic_hazard_visitor_for_location(
    project: &Project,
    package: &str,
    module_path: &[String],
) -> SyntacticHazardVisitor {
    let file = source_for_module(project, package, module_path).map(|source| source.path.clone());
    let include_context = project.workspace.packages.get(package).and_then(|package| {
        file.as_ref().and_then(|source_path| {
            source_path.parent().map(|source_dir| IncludeContext {
                package_root: package.root.clone(),
                source_dir: source_dir.to_path_buf(),
            })
        })
    });
    SyntacticHazardVisitor {
        counts: SyntacticHazardCounts::default(),
        include_context,
        location: HazardLocation {
            package: package.to_string(),
            module_path: module_path.to_vec(),
            file,
        },
    }
}

fn source_for_module<'a>(
    project: &'a Project,
    package: &str,
    module_path: &[String],
) -> Option<&'a model::SourceFile> {
    project
        .files
        .values()
        .find(|source| source.package == package && source.module_path == module_path)
}

struct SyntacticHazardVisitor {
    counts: SyntacticHazardCounts,
    include_context: Option<IncludeContext>,
    location: HazardLocation,
}

struct IncludeContext {
    package_root: PathBuf,
    source_dir: PathBuf,
}

struct HazardLocation {
    package: String,
    module_path: Vec<String>,
    file: Option<PathBuf>,
}

impl HazardLocation {
    fn subject(&self) -> String {
        if self.module_path.is_empty() {
            self.package.clone()
        } else {
            format!("{}::{}", self.package, self.module_path.join("::"))
        }
    }
}

impl<'ast> Visit<'ast> for SyntacticHazardVisitor {
    fn visit_attribute(&mut self, attribute: &'ast Attribute) {
        if attribute.path().is_ident("cfg") || attribute.path().is_ident("cfg_attr") {
            self.counts.conditional_compilation_attrs += 1;
            self.counts
                .conditional_compilation_details
                .push(self.cfg_attr_detail(attribute));
        }
        let custom_derives = custom_derive_macro_count(attribute);
        self.counts.custom_derive_macros += custom_derives;
        if custom_derives > 0 {
            self.counts
                .custom_derive_details
                .push(self.span_detail(attribute));
        }
        if attribute_requires_macro_expansion(attribute) {
            self.counts.custom_attribute_macros += 1;
            self.counts
                .custom_attribute_details
                .push(self.span_detail(attribute));
        }
        let nested_macro_counts = cfg_attr_nested_macro_counts(attribute);
        if nested_macro_counts.custom_attribute_macros > 0 {
            self.counts
                .custom_attribute_details
                .push(self.span_detail(attribute));
        }
        if nested_macro_counts.custom_derive_macros > 0 {
            self.counts
                .custom_derive_details
                .push(self.span_detail(attribute));
        }
        self.counts.add(nested_macro_counts);

        syn::visit::visit_attribute(self, attribute);
    }

    fn visit_macro(&mut self, mac: &'ast Macro) {
        if macro_path_ends_with(mac, "include") {
            if macro_tokens_reference_out_dir(&mac.tokens) {
                self.counts.out_dir_source_include_macros += 1;
                self.counts
                    .out_dir_source_include_details
                    .push(self.span_detail(mac));
            } else {
                self.counts.source_include_macros += 1;
                self.counts
                    .source_include_details
                    .push(self.span_detail(mac));
            }
        } else if macro_path_ends_with(mac, "include_str")
            || macro_path_ends_with(mac, "include_bytes")
        {
            self.visit_file_include_macro(mac);
        } else if macro_path_ends_with(mac, "env") || macro_path_ends_with(mac, "option_env") {
            self.visit_compile_env_macro(mac);
        }
        if macro_invocation_requires_expansion_boundary(mac) {
            self.counts.custom_macro_invocations += 1;
            self.counts
                .custom_macro_invocation_details
                .push(self.span_detail(mac));
        }

        syn::visit::visit_macro(self, mac);
    }

    fn visit_type_bare_fn(&mut self, bare_fn: &'ast syn::TypeBareFn) {
        self.counts.function_pointer_surfaces += 1;
        self.counts
            .function_pointer_details
            .push(self.type_surface_detail(bare_fn));
        syn::visit::visit_type_bare_fn(self, bare_fn);
    }

    fn visit_type_trait_object(&mut self, trait_object: &'ast syn::TypeTraitObject) {
        self.counts.trait_object_surfaces += 1;
        self.counts
            .trait_object_details
            .push(self.type_surface_detail(trait_object));
        syn::visit::visit_type_trait_object(self, trait_object);
    }
}

impl SyntacticHazardVisitor {
    fn cfg_attr_detail(&self, attribute: &Attribute) -> ProductionHazardDetail {
        ProductionHazardDetail {
            subject: self.location.subject(),
            package: Some(self.location.package.clone()),
            module_path: (!self.location.module_path.is_empty())
                .then(|| self.location.module_path.join("::")),
            file: self.location.file.clone(),
            start_line: Some(attribute.span().start().line),
            cfg: Some(attribute.to_token_stream().to_string()),
            suggested_cargo_args: cfg_gate_suggested_cargo_args(attribute),
        }
    }

    fn type_surface_detail<T: Spanned + ToTokens>(&self, node: &T) -> ProductionHazardDetail {
        let mut detail = self.span_detail(node);
        detail.subject = format!("{}: {}", detail.subject, node.to_token_stream());
        detail
    }

    fn span_detail<T: Spanned>(&self, node: &T) -> ProductionHazardDetail {
        ProductionHazardDetail {
            subject: self.location.subject(),
            package: Some(self.location.package.clone()),
            module_path: (!self.location.module_path.is_empty())
                .then(|| self.location.module_path.join("::")),
            file: self.location.file.clone(),
            start_line: Some(node.span().start().line),
            cfg: None,
            suggested_cargo_args: Vec::new(),
        }
    }
}

impl SyntacticHazardVisitor {
    fn visit_compile_env_macro(&mut self, mac: &Macro) {
        if macro_first_string_literal(&mac.tokens)
            .as_deref()
            .is_some_and(cargo_manifest_modeled_env_var)
        {
            return;
        }
        self.counts.compile_env_macros += 1;
        self.counts.compile_env_details.push(self.span_detail(mac));
    }

    fn visit_file_include_macro(&mut self, mac: &Macro) {
        if macro_tokens_reference_out_dir(&mac.tokens) {
            self.counts.out_dir_file_include_macros += 1;
            self.counts
                .out_dir_file_include_details
                .push(self.span_detail(mac));
            return;
        }

        let Some(path) = static_include_path(&mac.tokens) else {
            self.counts.nonliteral_file_include_macros += 1;
            self.counts
                .nonliteral_file_include_details
                .push(self.span_detail(mac));
            return;
        };
        self.count_file_include_path(mac, &path);
    }

    fn count_file_include_path(&mut self, mac: &Macro, path: &StaticIncludePath) {
        let Some(context) = &self.include_context else {
            return;
        };
        let candidate = match path {
            StaticIncludePath::Absolute(_) => {
                self.counts.absolute_file_include_macros += 1;
                self.counts
                    .absolute_file_include_details
                    .push(self.span_detail(mac));
                return;
            }
            StaticIncludePath::SourceRelative(path) => context.source_dir.join(path),
            StaticIncludePath::PackageRelative(path) => context.package_root.join(path),
        };

        if candidate
            .canonicalize()
            .is_ok_and(|path| !path.starts_with(&context.package_root))
        {
            self.counts.external_file_include_macros += 1;
            self.counts
                .external_file_include_details
                .push(self.span_detail(mac));
        }
    }
}

fn macro_tokens_reference_out_dir(tokens: &TokenStream) -> bool {
    token_stream_mentions_string_literal(tokens, "OUT_DIR")
}

fn macro_first_string_literal(tokens: &TokenStream) -> Option<String> {
    tokens.clone().into_iter().find_map(|token| match token {
        proc_macro2::TokenTree::Literal(literal) => {
            syn::parse2::<syn::LitStr>(literal.to_token_stream())
                .ok()
                .map(|literal| literal.value())
        }
        proc_macro2::TokenTree::Group(group) => macro_first_string_literal(&group.stream()),
        proc_macro2::TokenTree::Ident(_) | proc_macro2::TokenTree::Punct(_) => None,
    })
}

fn cargo_manifest_modeled_env_var(name: &str) -> bool {
    name.starts_with("CARGO_PKG_") || matches!(name, "CARGO_CRATE_NAME" | "CARGO_BIN_NAME")
}

fn token_stream_mentions_string_literal(tokens: &TokenStream, value: &str) -> bool {
    tokens.clone().into_iter().any(|token| match token {
        proc_macro2::TokenTree::Literal(literal) => {
            syn::parse2::<syn::LitStr>(literal.to_token_stream())
                .is_ok_and(|literal| literal.value() == value)
        }
        proc_macro2::TokenTree::Group(group) => {
            token_stream_mentions_string_literal(&group.stream(), value)
        }
        proc_macro2::TokenTree::Ident(_) | proc_macro2::TokenTree::Punct(_) => false,
    })
}

fn macro_path_ends_with(mac: &Macro, name: &str) -> bool {
    mac.path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == name)
}

fn macro_invocation_requires_expansion_boundary(mac: &Macro) -> bool {
    let Some(last) = mac.path.segments.last() else {
        return false;
    };
    if !builtin_macro_name(&last.ident.to_string()) {
        return true;
    }
    if mac.path.leading_colon.is_some() {
        return false;
    }
    if mac.path.segments.len() == 1 {
        return false;
    }
    mac.path.segments.first().is_none_or(|segment| {
        !matches!(segment.ident.to_string().as_str(), "std" | "core" | "alloc")
    })
}

fn builtin_macro_name(name: &str) -> bool {
    matches!(
        name,
        "assert"
            | "assert_eq"
            | "assert_ne"
            | "cfg"
            | "column"
            | "compile_error"
            | "concat"
            | "dbg"
            | "debug_assert"
            | "debug_assert_eq"
            | "debug_assert_ne"
            | "env"
            | "eprint"
            | "eprintln"
            | "file"
            | "format"
            | "format_args"
            | "include"
            | "include_bytes"
            | "include_str"
            | "line"
            | "macro_rules"
            | "matches"
            | "module_path"
            | "option_env"
            | "panic"
            | "print"
            | "println"
            | "stringify"
            | "thread_local"
            | "todo"
            | "try"
            | "unimplemented"
            | "unreachable"
            | "vec"
            | "write"
            | "writeln"
    )
}

fn cfg_attr_nested_macro_counts(attribute: &Attribute) -> SyntacticHazardCounts {
    if !attribute.path().is_ident("cfg_attr") {
        return SyntacticHazardCounts::default();
    }

    let Ok(arguments) =
        attribute.parse_args_with(Punctuated::<Meta, syn::Token![,]>::parse_terminated)
    else {
        return SyntacticHazardCounts::default();
    };

    cfg_attr_argument_macro_counts(&arguments)
}

fn cfg_attr_meta_nested_macro_counts(meta: &Meta) -> SyntacticHazardCounts {
    let Meta::List(list) = meta else {
        return SyntacticHazardCounts::default();
    };
    let Ok(arguments) =
        Punctuated::<Meta, syn::Token![,]>::parse_terminated.parse2(list.tokens.clone())
    else {
        return SyntacticHazardCounts::default();
    };

    cfg_attr_argument_macro_counts(&arguments)
}

fn cfg_attr_argument_macro_counts(
    arguments: &Punctuated<Meta, syn::Token![,]>,
) -> SyntacticHazardCounts {
    let mut counts = SyntacticHazardCounts::default();
    for nested_attr in arguments.iter().skip(1) {
        add_meta_macro_counts(nested_attr, &mut counts);
    }
    counts
}

fn add_meta_macro_counts(meta: &Meta, counts: &mut SyntacticHazardCounts) {
    if meta.path().is_ident("cfg_attr") {
        counts.add(cfg_attr_meta_nested_macro_counts(meta));
        return;
    }
    if meta.path().is_ident("derive") {
        counts.custom_derive_macros += custom_derive_meta_count(meta);
        return;
    }

    let Some(first) = meta.path().segments.first() else {
        return;
    };
    if !attribute_path_is_builtin_or_inert(&first.ident.to_string()) {
        counts.custom_attribute_macros += 1;
    }
}

fn custom_derive_macro_count(attribute: &Attribute) -> usize {
    if !attribute.path().is_ident("derive") {
        return 0;
    }

    attribute
        .parse_args_with(Punctuated::<syn::Path, syn::Token![,]>::parse_terminated)
        .map(|paths| {
            paths
                .iter()
                .filter(|path| !derive_path_is_builtin(path))
                .count()
        })
        .unwrap_or_default()
}

fn custom_derive_meta_count(meta: &Meta) -> usize {
    let Meta::List(list) = meta else {
        return 0;
    };

    Punctuated::<syn::Path, syn::Token![,]>::parse_terminated
        .parse2(list.tokens.clone())
        .map(|paths| {
            paths
                .iter()
                .filter(|path| !derive_path_is_builtin(path))
                .count()
        })
        .unwrap_or_default()
}

fn derive_path_is_builtin(path: &syn::Path) -> bool {
    if path.leading_colon.is_some() || path.segments.len() != 1 {
        return false;
    }
    path.segments.last().is_some_and(|segment| {
        matches!(
            segment.ident.to_string().as_str(),
            "Clone"
                | "Copy"
                | "Debug"
                | "Default"
                | "Eq"
                | "Hash"
                | "Ord"
                | "PartialEq"
                | "PartialOrd"
        )
    })
}

fn attribute_requires_macro_expansion(attribute: &Attribute) -> bool {
    if attribute.path().is_ident("derive") {
        return false;
    }

    let Some(first) = attribute.path().segments.first() else {
        return false;
    };
    !attribute_path_is_builtin_or_inert(&first.ident.to_string())
}

fn attribute_path_is_builtin_or_inert(first_segment: &str) -> bool {
    matches!(
        first_segment,
        "allow"
            | "automatically_derived"
            | "bench"
            | "cfg"
            | "cfg_attr"
            | "cold"
            | "deny"
            | "deprecated"
            | "doc"
            | "export_name"
            | "forbid"
            | "global_allocator"
            | "ignore"
            | "inline"
            | "link"
            | "link_name"
            | "link_section"
            | "macro_export"
            | "macro_use"
            | "must_use"
            | "no_mangle"
            | "non_exhaustive"
            | "opensourced"
            | "panic_handler"
            | "path"
            | "proc_macro"
            | "proc_macro_attribute"
            | "proc_macro_derive"
            | "repr"
            | "should_panic"
            | "test"
            | "track_caller"
            | "used"
            | "warn"
    )
}

fn production_hazard(
    code: &str,
    severity: &str,
    message: impl Into<String>,
) -> ProductionHazardReport {
    production_hazard_with_details(code, severity, message, Vec::new())
}

fn production_hazard_with_details(
    code: &str,
    severity: &str,
    message: impl Into<String>,
    details: Vec<ProductionHazardDetail>,
) -> ProductionHazardReport {
    ProductionHazardReport {
        code: code.to_string(),
        severity: severity.to_string(),
        message: message.into(),
        details,
    }
}

fn production_readiness_status(hazards: Vec<ProductionHazardReport>) -> ProductionReadinessReport {
    let status = if hazards.iter().any(|hazard| hazard.severity == "error") {
        "hazards_detected"
    } else if hazards.is_empty() {
        "ready_for_feedback"
    } else {
        "requires_feedback"
    };
    ProductionReadinessReport {
        status: status.to_string(),
        hazards,
    }
}

pub fn write_generate_report(
    report: &GenerateReport,
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(&GenerateReportJson::from_report(report))?;
    fs::write(path, json)?;
    Ok(())
}

#[derive(Serialize)]
struct GenerateReportJson {
    analyzer: AnalyzerReportJson,
    production: ProductionReadinessReport,
    timings: GenerateTimingReportJson,
    root: String,
    roots: Vec<String>,
    feedback_widened_roots: Vec<String>,
    packages: Vec<String>,
    targets: Vec<GeneratedTargetReportJson>,
    reachable: Vec<String>,
    reachable_items: Vec<String>,
    source_map: SourceMapReportJson,
    files_written: usize,
}

impl GenerateReportJson {
    fn from_report(report: &GenerateReport) -> Self {
        Self {
            analyzer: AnalyzerReportJson::from_report(&report.analyzer),
            production: report.production.clone(),
            timings: GenerateTimingReportJson::from_report(&report.timings),
            root: report.root.to_string(),
            roots: report.roots.iter().map(ToString::to_string).collect(),
            feedback_widened_roots: report
                .feedback_widened_roots
                .iter()
                .map(ToString::to_string)
                .collect(),
            packages: report.packages.clone(),
            targets: report
                .targets
                .iter()
                .map(GeneratedTargetReportJson::from_report)
                .collect(),
            reachable: report.reachable.iter().map(ToString::to_string).collect(),
            reachable_items: report
                .reachable_items
                .iter()
                .map(ToString::to_string)
                .collect(),
            source_map: SourceMapReportJson::from_report(&report.source_map),
            files_written: report.files_written,
        }
    }
}

#[derive(Serialize)]
struct GeneratedTargetReportJson {
    package: String,
    name: String,
    kind: Vec<String>,
    src_path: PathBuf,
    required_features: Vec<String>,
    default_features: Vec<String>,
}

impl GeneratedTargetReportJson {
    fn from_report(report: &GeneratedTargetReport) -> Self {
        Self {
            package: report.package.clone(),
            name: report.name.clone(),
            kind: report.kind.clone(),
            src_path: report.src_path.clone(),
            required_features: report.required_features.clone(),
            default_features: report.default_features.clone(),
        }
    }
}

#[derive(Serialize)]
struct GenerateTimingReportJson {
    total_ms: u64,
    analyzer_ms: u64,
    manifest_ms: u64,
    parse_ms: u64,
    reduce_ms: u64,
    render_ms: u64,
}

impl GenerateTimingReportJson {
    fn from_report(report: &GenerateTimingReport) -> Self {
        Self {
            total_ms: report.total_ms,
            analyzer_ms: report.analyzer_ms,
            manifest_ms: report.manifest_ms,
            parse_ms: report.parse_ms,
            reduce_ms: report.reduce_ms,
            render_ms: report.render_ms,
        }
    }
}

#[derive(Serialize)]
struct SourceMapReportJson {
    callables: Vec<CallableLocationJson>,
    items: Vec<ItemLocationJson>,
}

impl SourceMapReportJson {
    fn from_report(report: &SourceMapReport) -> Self {
        Self {
            callables: report
                .callables
                .iter()
                .map(CallableLocationJson::from_report)
                .collect(),
            items: report
                .items
                .iter()
                .map(ItemLocationJson::from_report)
                .collect(),
        }
    }
}

#[derive(Serialize)]
struct CallableLocationJson {
    id: String,
    reachable: bool,
    span: SourceSpan,
}

impl CallableLocationJson {
    fn from_report(location: &CallableLocation) -> Self {
        Self {
            id: location.id.to_string(),
            reachable: location.reachable,
            span: location.span.clone(),
        }
    }
}

#[derive(Serialize)]
struct ItemLocationJson {
    id: String,
    reachable: bool,
    span: SourceSpan,
}

impl ItemLocationJson {
    fn from_report(location: &ItemLocation) -> Self {
        Self {
            id: location.id.to_string(),
            reachable: location.reachable,
            span: location.span.clone(),
        }
    }
}

#[derive(Serialize)]
struct AnalyzerReportJson {
    mode: String,
    loaded: bool,
    engine: String,
    notes: Vec<String>,
    semantic: Option<SemanticReportJson>,
    semantic_reduction_hints: SemanticReductionHintsJson,
}

impl AnalyzerReportJson {
    fn from_report(report: &AnalyzerReport) -> Self {
        Self {
            mode: report.mode.as_str().to_string(),
            loaded: report.loaded,
            engine: report.engine.clone(),
            notes: report.notes.clone(),
            semantic: report
                .semantic
                .as_ref()
                .map(SemanticReportJson::from_report),
            semantic_reduction_hints: SemanticReductionHintsJson::from_report(
                &report.semantic_hints,
            ),
        }
    }
}

#[derive(Serialize)]
struct SemanticReductionHintsJson {
    callable_owners: usize,
    item_owners: usize,
    total_edges: usize,
    unresolved_queries: usize,
    unqueried_queries: usize,
    unmapped_targets: usize,
}

impl SemanticReductionHintsJson {
    fn from_report(report: &SemanticReductionHints) -> Self {
        Self {
            callable_owners: report.callable_edges.len(),
            item_owners: report.item_edges.len(),
            total_edges: report.total_edges(),
            unresolved_queries: report.unresolved_queries,
            unqueried_queries: report.unqueried_queries,
            unmapped_targets: report.unmapped_targets,
        }
    }
}

#[derive(Serialize)]
struct SemanticReportJson {
    source_files: usize,
    analyzed_files: usize,
    failed_files: usize,
    skipped_files: usize,
    file_budget: usize,
    method_call_budget: usize,
    path_budget: usize,
    method_calls: usize,
    queried_method_calls: usize,
    resolved_method_calls: usize,
    callable_method_calls: usize,
    fallback_method_calls: usize,
    unresolved_method_calls: usize,
    unqueried_method_calls: usize,
    paths: usize,
    queried_paths: usize,
    resolved_paths: usize,
    unresolved_paths: usize,
    unqueried_paths: usize,
}

impl SemanticReportJson {
    fn from_report(report: &SemanticReport) -> Self {
        Self {
            source_files: report.source_files,
            analyzed_files: report.analyzed_files,
            failed_files: report.failed_files,
            skipped_files: report.skipped_files,
            file_budget: report.file_budget,
            method_call_budget: report.method_call_budget,
            path_budget: report.path_budget,
            method_calls: report.method_calls,
            queried_method_calls: report.queried_method_calls,
            resolved_method_calls: report.resolved_method_calls,
            callable_method_calls: report.callable_method_calls,
            fallback_method_calls: report.fallback_method_calls,
            unresolved_method_calls: report.unresolved_method_calls,
            unqueried_method_calls: report.unqueried_method_calls,
            paths: report.paths,
            queried_paths: report.queried_paths,
            resolved_paths: report.resolved_paths,
            unresolved_paths: report.unresolved_paths,
            unqueried_paths: report.unqueried_paths,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[cfg(feature = "ra-hir")]
    use super::generate_with_analyzer;
    use super::{
        add_semantic_inventory_hazard, default_feature_closure, generate,
        generate_with_analyzer_feedback, write_generate_report, AnalyzerMode, AnalyzerReport,
        CallableId, CheckDiagnostic, GenerateOptions, SemanticReductionHints, SemanticReport,
    };

    #[test]
    fn reduces_fixture_to_reachable_callables() {
        let output = temp_output("callables");
        let report = generate(GenerateOptions {
            workspace_root: workspace_root(),
            output_root: output.clone(),
        })
        .expect("reduction should succeed");

        let reachable = report
            .reachable
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        for expected in [
            "a::open_source_entry",
            "b::compute",
            "b::helper",
            "c::adjust",
            "d::Worker::new",
            "d::Worker::run",
            "d::hash",
            "d::normalize",
            "d::shared",
            "e::finish",
            "e::mix",
            "e::seed",
            "d::<Worker as Transform>::transform",
        ] {
            assert!(
                reachable.iter().any(|actual| actual == expected),
                "missing reachable callable {expected}; got {reachable:?}",
            );
        }

        for not_expected in [
            "a::internal_entry",
            "b::unused_public",
            "c::unused",
            "d::Worker::unused_method",
            "d::unused_private",
            "d::unused_public",
            "e::unused_leaf",
            "e::tempting_but_unused",
        ] {
            assert!(
                !reachable.iter().any(|actual| actual == not_expected),
                "unreachable callable {not_expected} was retained in graph",
            );
        }

        let reachable_items = report
            .reachable_items
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        for expected in [
            "d::Mode(Enum)",
            "d::Score(Type)",
            "d::Transform(Trait)",
            "d::Worker(Struct)",
            "e::DEFAULT_SEED(Const)",
            "e::Score(Type)",
        ] {
            assert!(
                reachable_items.iter().any(|actual| actual == expected),
                "missing reachable item {expected}; got {reachable_items:?}",
            );
        }
        assert!(
            !reachable_items
                .iter()
                .any(|actual| actual == "d::UnusedEnum(Enum)"),
            "unreachable enum was retained in item graph",
        );

        let d_source = fs::read_to_string(output.join("d/src/lib.rs")).unwrap();
        assert!(d_source.contains("pub enum Mode"));
        assert!(d_source.contains("pub trait Transform"));
        assert!(d_source.contains("impl Transform for Worker"));
        assert!(d_source.contains("fn transform"));
        assert!(d_source.contains("pub fn hash"));
        assert!(d_source.contains("pub fn run"));
        assert!(!d_source.contains("UnusedEnum"));
        assert!(!d_source.contains("cfg(test)"));
        assert!(!d_source.contains("unit_test_that_must_not_be_exported"));
        assert!(!d_source.contains("unused_public"));
        assert!(!d_source.contains("unused_private"));
        assert!(!d_source.contains("unused_method"));

        let b_source = fs::read_to_string(output.join("b/src/lib.rs")).unwrap();
        assert!(b_source.contains("use d::Transform"));
        assert!(b_source.contains("worker.transform(value)"));
        assert!(!b_source.contains("cfg(test)"));
        assert!(!b_source.contains("unit_test_that_must_not_be_exported"));

        let a_source = fs::read_to_string(output.join("a/src/lib.rs")).unwrap();
        assert!(a_source.contains("pub fn open_source_entry"));
        assert!(!a_source.contains("#[opensourced]"));
        assert!(!a_source.contains("internal_entry"));
    }

    #[test]
    fn default_feature_closure_follows_nested_package_features() {
        let manifest = r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[features]
default = ["demo-ui", "storage"]
demo-ui = ["theme"]
storage = ["dep:rusqlite", "serde?/derive"]
theme = []
"#
        .parse::<toml::Value>()
        .expect("manifest should parse");

        assert_eq!(
            default_feature_closure(&manifest),
            ["demo-ui", "rusqlite", "serde", "storage", "theme"]
        );
    }

    #[test]
    fn generated_fixture_workspace_compiles() {
        let output = temp_output("compile");
        generate(GenerateOptions {
            workspace_root: workspace_root(),
            output_root: output.clone(),
        })
        .expect("reduction should succeed");

        let target_dir = temp_output("target");
        let status = Command::new("cargo")
            .arg("check")
            .current_dir(&output)
            .env("CARGO_TARGET_DIR", target_dir)
            .status()
            .expect("cargo check should start");

        assert!(status.success(), "generated workspace did not compile");
    }

    #[test]
    fn writes_generation_report_json() {
        let output = temp_output("generation-report-output");
        let report = generate(GenerateOptions {
            workspace_root: workspace_root(),
            output_root: output,
        })
        .expect("reduction should succeed");
        let report_path = temp_output("generation-report-json").join("slice-report.json");

        write_generate_report(&report, &report_path).expect("generation report should be written");

        let value: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(report_path).unwrap()).unwrap();
        assert_eq!(value["analyzer"]["mode"], "syn");
        assert_eq!(value["production"]["status"], "requires_feedback");
        assert!(value["production"]["hazards"]
            .as_array()
            .unwrap()
            .iter()
            .any(|hazard| hazard["code"] == "semantic_analyzer_unavailable"));
        assert_eq!(value["root"], report.root.to_string());
        assert_eq!(value["files_written"], report.files_written);
        assert_eq!(value["timings"]["total_ms"], report.timings.total_ms);
        assert!(value["timings"]["render_ms"].as_u64().is_some());
        assert_eq!(
            value["packages"].as_array().unwrap().len(),
            report.packages.len()
        );
        let callables = value["source_map"]["callables"].as_array().unwrap();
        let root_location = callables
            .iter()
            .find(|location| location["id"] == "a::open_source_entry")
            .expect("root callable should have a source-map entry");
        assert_eq!(root_location["reachable"], true);
        assert!(root_location["span"]["file"]
            .as_str()
            .unwrap()
            .ends_with("fixtures/a/src/lib.rs"));
        assert!(root_location["span"]["start_line"].as_u64().unwrap() > 0);

        let internal_location = callables
            .iter()
            .find(|location| location["id"] == "a::internal_entry")
            .expect("unreachable callable should still have a source-map entry");
        assert_eq!(internal_location["reachable"], false);
    }

    #[test]
    fn reports_semantic_inventory_as_report_only_production_hazard() {
        let analyzer = AnalyzerReport {
            mode: AnalyzerMode::RustAnalyzerHir,
            loaded: true,
            engine: "rust-analyzer HIR".to_string(),
            notes: Vec::new(),
            semantic: Some(SemanticReport::default()),
            semantic_hints: SemanticReductionHints::default(),
        };
        let mut hazards = Vec::new();

        add_semantic_inventory_hazard(&analyzer, 0, &mut hazards);

        assert!(hazards
            .iter()
            .any(|hazard| hazard.code == "semantic_inventory_not_applied"));
    }

    #[test]
    #[cfg(feature = "ra-hir")]
    fn applies_ra_semantic_hints_to_reduction() {
        let root = temp_output("ra-semantic-source");
        let output = temp_output("ra-semantic-reduction");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\n",
                opensourced_path
            ),
        );
        write(
            root.join("app/src/lib.rs"),
            r#"
use opensourced::opensourced;

pub struct Service;

impl Service {
    pub fn selected(&self) -> u32 {
        1
    }

    pub fn unused(&self) -> u32 {
        2
    }
}

#[opensourced]
pub fn entry(service: Service) -> u32 {
    service.selected()
}
"#,
        );
        let report = generate_with_analyzer(
            GenerateOptions {
                workspace_root: root,
                output_root: output,
            },
            AnalyzerMode::RustAnalyzerHir,
        )
        .expect("RA-backed generation should succeed");

        assert!(report.analyzer.semantic_hints.total_edges() > 0);
        assert!(report.production.hazards.iter().any(|hazard| {
            hazard.code == "semantic_reduction_hints_applied" && hazard.severity == "warning"
        }));
        assert!(report.production.hazards.iter().any(|hazard| {
            hazard.code == "semantic_inventory_partially_applied" && hazard.severity == "warning"
        }));
        assert!(!report
            .production
            .hazards
            .iter()
            .any(|hazard| hazard.code == "semantic_inventory_not_applied"));
    }

    #[test]
    fn reports_reachable_include_macro_production_hazards() {
        let root = temp_output("include-hazard-source");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\n",
                opensourced_path
            ),
        );
        let absolute_asset = root.join("app/assets/absolute.bin");
        write(absolute_asset.clone(), "absolute asset");
        write(root.join("workspace-secret.bin"), "workspace secret");
        let source = r#"use opensourced::opensourced;

#[opensourced]
pub fn entry() -> &'static str {
    let _generated = include!("generated_expr.rs");
    let _out_dir_generated = include!(concat!(env!("OUT_DIR"), "/generated.rs"));
    let _unknown_asset = include_bytes!(env!("DATA_PATH"));
    let _absolute_asset = include_bytes!(ABSOLUTE_ASSET);
    let _external_asset = include_bytes!("../../workspace-secret.bin");
    include_str!(concat!("data", ".txt"))
}
"#
        .replace("ABSOLUTE_ASSET", &format!("{absolute_asset:?}"));
        write(root.join("app/src/lib.rs"), &source);

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: temp_output("include-hazard-output"),
        })
        .expect("reduction should succeed");

        let source_include = report
            .production
            .hazards
            .iter()
            .find(|hazard| hazard.code == "source_include_macros" && hazard.severity == "error")
            .expect("source include hazard should be reported");
        assert!(source_include.details.iter().any(|detail| {
            detail.subject == "app"
                && detail
                    .file
                    .as_ref()
                    .is_some_and(|file| file.ends_with("app/src/lib.rs"))
                && detail.start_line == Some(5)
        }));
        assert!(report.production.hazards.iter().any(|hazard| {
            hazard.code == "out_dir_source_include_macros" && hazard.severity == "error"
        }));
        assert!(report
            .production
            .hazards
            .iter()
            .any(|hazard| hazard.code == "nonliteral_file_include_macros"
                && hazard.severity == "error"));
        assert!(report
            .production
            .hazards
            .iter()
            .any(|hazard| hazard.code == "absolute_file_include_macros"
                && hazard.severity == "error"));
        let external_include = report
            .production
            .hazards
            .iter()
            .find(|hazard| {
                hazard.code == "external_file_include_macros" && hazard.severity == "error"
            })
            .expect("external include hazard should be reported");
        assert!(external_include
            .details
            .iter()
            .any(|detail| detail.start_line == Some(9)));
        assert_eq!(report.production.status, "hazards_detected");
    }

    #[test]
    fn reports_compile_env_macro_production_hazards() {
        let root = temp_output("compile-env-hazard-source");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\n",
                opensourced_path
            ),
        );
        write(
            root.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;

#[opensourced]
pub fn entry() -> (&'static str, Option<&'static str>) {
    (env!("CARGO_PKG_VERSION"), option_env!("APP_MODE"))
}
"#,
        );

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: temp_output("compile-env-hazard-output"),
        })
        .expect("reduction should succeed");

        let hazard = report
            .production
            .hazards
            .iter()
            .find(|hazard| hazard.code == "compile_env_macros" && hazard.severity == "error")
            .expect("compile env hazard should be reported");
        assert!(hazard.details.iter().any(|detail| {
            detail.subject == "app"
                && detail
                    .file
                    .as_ref()
                    .is_some_and(|file| file.ends_with("app/src/lib.rs"))
                && detail.start_line == Some(5)
        }));
        assert_eq!(report.production.status, "hazards_detected");
    }

    #[test]
    fn reports_reachable_custom_macro_invocation_production_hazards() {
        let root = temp_output("macro-invocation-hazard-source");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\n",
                opensourced_path
            ),
        );
        write(
            root.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;

#[opensourced]
pub fn entry() -> Vec<i32> {
    let value = project_macro!(1);
    println!("{value}");
    vec![value]
}
"#,
        );

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: temp_output("macro-invocation-hazard-output"),
        })
        .expect("reduction should succeed");

        let hazard = report
            .production
            .hazards
            .iter()
            .find(|hazard| {
                hazard.code == "custom_macro_invocations" && hazard.severity == "warning"
            })
            .expect("custom macro invocation hazard should be reported");
        assert!(hazard.details.iter().any(|detail| {
            detail.subject == "app"
                && detail
                    .file
                    .as_ref()
                    .is_some_and(|file| file.ends_with("app/src/lib.rs"))
                && detail.start_line == Some(5)
        }));
        assert_eq!(report.production.status, "requires_feedback");
    }

    #[test]
    fn reports_syntactic_method_fallback_production_hazards() {
        let root = temp_output("method-fallback-hazard-source");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\n",
                opensourced_path
            ),
        );
        write(
            root.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;

macro_rules! make_worker {
    () => {
        Worker
    };
}

#[opensourced]
pub fn entry() -> i32 {
    make_worker!().run()
}

pub struct Worker;

impl Worker {
    pub fn run(&self) -> i32 {
        1
    }
}
"#,
        );

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: temp_output("method-fallback-hazard-output"),
        })
        .expect("reduction should succeed");

        assert!(report
            .production
            .hazards
            .iter()
            .any(|hazard| hazard.code == "syntactic_method_fallbacks"));
    }

    #[test]
    fn caps_ambiguous_syntactic_method_fallbacks() {
        let root = temp_output("method-fallback-cap-source");
        let output = temp_output("method-fallback-cap-output");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\n",
                opensourced_path
            ),
        );
        write(
            root.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;

macro_rules! make_worker {
    () => {
        Worker
    };
}

#[opensourced]
pub fn entry() -> i32 {
    make_worker!().run()
}

pub struct Worker;
pub struct Other;

impl Worker {
    pub fn run(&self) -> i32 {
        1
    }
}

impl Other {
    pub fn run(&self) -> i32 {
        2
    }
}
"#,
        );

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: output.clone(),
        })
        .expect("reduction should succeed");

        assert!(report
            .production
            .hazards
            .iter()
            .any(|hazard| hazard.code == "syntactic_method_fallback_cap"));
        let rendered = fs::read_to_string(output.join("app/src/lib.rs")).unwrap();
        assert!(
            !rendered.contains("impl Other"),
            "ambiguous name-only fallback should not retain unrelated same-name impls:\n{rendered}"
        );
    }

    #[test]
    fn reports_function_pointer_and_trait_object_production_hazards() {
        let root = temp_output("dynamic-dispatch-hazard-source");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\n",
                opensourced_path
            ),
        );
        write(
            root.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;

pub type Callback = fn() -> usize;

pub trait Worker {
    fn run(&self) -> usize;
}

pub struct Real;

impl Worker for Real {
    fn run(&self) -> usize {
        7
    }
}

#[opensourced]
pub fn entry(callback: Callback) -> (Callback, Box<dyn Worker>) {
    (callback, Box::new(Real))
}
"#,
        );

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: temp_output("dynamic-dispatch-hazard-output"),
        })
        .expect("reduction should succeed");

        let function_pointer = report
            .production
            .hazards
            .iter()
            .find(|hazard| hazard.code == "function_pointer_surfaces" && hazard.severity == "error")
            .expect("function pointer hazard should be reported");
        assert!(function_pointer.details.iter().any(|detail| {
            detail.subject == "app: fn () -> usize"
                && detail.package.as_deref() == Some("app")
                && detail
                    .file
                    .as_ref()
                    .is_some_and(|file| file.ends_with("app/src/lib.rs"))
                && detail.start_line == Some(3)
        }));
        let trait_object = report
            .production
            .hazards
            .iter()
            .find(|hazard| hazard.code == "trait_object_surfaces" && hazard.severity == "error")
            .expect("trait object hazard should be reported");
        assert!(trait_object.details.iter().any(|detail| {
            detail.subject == "app: dyn Worker"
                && detail.package.as_deref() == Some("app")
                && detail
                    .file
                    .as_ref()
                    .is_some_and(|file| file.ends_with("app/src/lib.rs"))
                && detail.start_line == Some(18)
        }));
        assert_eq!(report.production.status, "hazards_detected");
    }

    #[test]
    fn reports_retained_build_script_production_hazards() {
        let root = temp_output("build-script-hazard-source");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\nbuild = \"build.rs\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\n",
                opensourced_path
            ),
        );
        write(
            root.join("app/build.rs"),
            r#"fn main() {
    println!("cargo:rerun-if-changed=build.rs");
}
"#,
        );
        write(
            root.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;

#[opensourced]
pub fn entry() -> i32 {
    1
}
"#,
        );

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: temp_output("build-script-hazard-output"),
        })
        .expect("reduction should succeed");

        let hazard = report
            .production
            .hazards
            .iter()
            .find(|hazard| hazard.code == "retained_build_scripts" && hazard.severity == "error")
            .expect("retained build script hazard should be reported");
        assert!(hazard.details.iter().any(|detail| {
            detail.subject == "app"
                && detail
                    .file
                    .as_ref()
                    .is_some_and(|file| file.ends_with("app/build.rs"))
                && detail.start_line == Some(1)
        }));
        assert_eq!(report.production.status, "hazards_detected");
    }

    #[test]
    fn reports_external_state_build_script_production_hazards() {
        let root = temp_output("build-script-external-state-source");
        let output = temp_output("build-script-external-state-output");
        let policy_path = temp_output("build-script-external-policy").join("policy.txt");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(policy_path.clone(), "machine-local-policy");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\nbuild = \"build.rs\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\n",
                opensourced_path
            ),
        );
        write(
            root.join("app/build.rs"),
            &format!(
                r#"fn main() {{
    let policy = std::fs::read_to_string({policy_path:?}).unwrap();
    println!("cargo:rustc-env=POLICY={{}}", policy.trim());
}}
"#
            ),
        );
        write(
            root.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;

#[opensourced]
pub fn entry() -> &'static str {
    env!("POLICY")
}
"#,
        );

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: output,
        })
        .expect("reduction should succeed");

        assert!(report.production.hazards.iter().any(|hazard| {
            hazard.code == "retained_build_scripts" && hazard.severity == "error"
        }));
        assert_eq!(report.production.status, "hazards_detected");
    }

    #[test]
    fn accepts_copyable_retained_non_workspace_path_dependencies() {
        let root = temp_output("path-dependency-hazard-source");
        let external = temp_output("path-dependency-hazard-external");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\nexternal-helper = {{ path = {:?} }}\n",
                opensourced_path, external
            ),
        );
        write(
            root.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;

#[opensourced]
pub fn entry() -> usize {
    external_helper::value()
}
"#,
        );
        write(
            external.join("Cargo.toml"),
            "[package]\nname = \"external-helper\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        write(
            external.join("src/lib.rs"),
            "pub fn value() -> usize { 1 }\n",
        );

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: temp_output("path-dependency-hazard-output"),
        })
        .expect("reduction should succeed");

        assert!(!report
            .production
            .hazards
            .iter()
            .any(|hazard| hazard.code == "retained_non_workspace_path_dependencies"));
    }

    #[test]
    fn accepts_copyable_retained_target_non_workspace_path_dependencies() {
        let root = temp_output("target-path-dependency-hazard-source");
        let external = temp_output("target-path-dependency-hazard-external");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\n\n[target.'cfg(unix)'.dependencies]\nexternal-helper = {{ path = {:?} }}\n",
                opensourced_path, external
            ),
        );
        write(
            root.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;

#[opensourced]
pub fn entry() -> usize {
    external_helper::value()
}
"#,
        );
        write(
            external.join("Cargo.toml"),
            "[package]\nname = \"external-helper\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        write(
            external.join("src/lib.rs"),
            "pub fn value() -> usize { 1 }\n",
        );

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: temp_output("target-path-dependency-hazard-output"),
        })
        .expect("reduction should succeed");

        assert!(!report
            .production
            .hazards
            .iter()
            .any(|hazard| hazard.code == "retained_non_workspace_path_dependencies"));
    }

    #[test]
    fn accepts_copyable_retained_workspace_patch_path_dependencies() {
        let root = temp_output("patch-path-dependency-hazard-source");
        let external = temp_output("patch-path-dependency-hazard-external");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            &format!(
                "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n\n[patch.crates-io]\nexternal-helper = {{ path = {:?} }}\n",
                external
            ),
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\n",
                opensourced_path
            ),
        );
        write(
            root.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;

#[opensourced]
pub fn entry() -> usize {
    1
}
"#,
        );
        write(
            external.join("Cargo.toml"),
            "[package]\nname = \"external-helper\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        write(
            external.join("src/lib.rs"),
            "pub fn value() -> usize { 1 }\n",
        );

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: temp_output("patch-path-dependency-hazard-output"),
        })
        .expect("reduction should succeed");

        assert!(!report
            .production
            .hazards
            .iter()
            .any(|hazard| { hazard.code == "retained_workspace_patch_replace_path_dependencies" }));
    }

    #[test]
    fn reports_uncopyable_workspace_patch_replace_path_dependency_details() {
        let root = temp_output("uncopyable-patch-path-hazard-source");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            r#"[workspace]
members = ["app"]
resolver = "2"

[patch.crates-io]
missing-helper = { path = "missing-helper" }
"#,
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\n",
                opensourced_path
            ),
        );
        write(
            root.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;

#[opensourced]
pub fn entry() -> usize {
    1
}
"#,
        );

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: temp_output("uncopyable-patch-path-hazard-output"),
        })
        .expect("reduction should succeed");

        let hazard = report
            .production
            .hazards
            .iter()
            .find(|hazard| {
                hazard.code == "retained_workspace_patch_replace_path_dependencies"
                    && hazard.severity == "error"
            })
            .expect("uncopyable workspace patch/replace path hazard should be reported");
        assert!(hazard.details.iter().any(|detail| {
            detail.subject == "patch.crates-io.missing-helper"
                && detail.package.is_none()
                && detail
                    .file
                    .as_ref()
                    .is_some_and(|file| file.ends_with("Cargo.toml"))
                && detail.start_line.is_some()
        }));
        assert_eq!(report.production.status, "hazards_detected");
    }

    #[test]
    fn reports_uncopyable_workspace_replace_path_dependency_details() {
        let root = temp_output("uncopyable-replace-path-hazard-source");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            r#"[workspace]
members = ["app"]
resolver = "2"

[replace]
"missing-replace:0.1.0" = { path = "missing-replace" }
"#,
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\n",
                opensourced_path
            ),
        );
        write(
            root.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;

#[opensourced]
pub fn entry() -> usize {
    1
}
"#,
        );

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: temp_output("uncopyable-replace-path-hazard-output"),
        })
        .expect("reduction should succeed");

        let hazard = report
            .production
            .hazards
            .iter()
            .find(|hazard| {
                hazard.code == "retained_workspace_patch_replace_path_dependencies"
                    && hazard.severity == "error"
            })
            .expect("uncopyable workspace replace path hazard should be reported");
        assert!(hazard.details.iter().any(|detail| {
            detail.subject == "replace.missing-replace:0.1.0"
                && detail.package.is_none()
                && detail
                    .file
                    .as_ref()
                    .is_some_and(|file| file.ends_with("Cargo.toml"))
                && detail.start_line.is_some()
        }));
        assert_eq!(report.production.status, "hazards_detected");
    }

    #[test]
    fn reports_reachable_attribute_macro_production_hazards() {
        let root = temp_output("attribute-hazard-source");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\n",
                opensourced_path
            ),
        );
        write(
            root.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;

#[opensourced]
pub fn entry() -> Payload {
    Payload { value: 1 }
}

#[custom_attr::decorate]
#[derive(Debug, serde::Serialize)]
pub struct Payload {
    value: i32,
}
"#,
        );

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: temp_output("attribute-hazard-output"),
        })
        .expect("reduction should succeed");

        let attribute_hazard = report
            .production
            .hazards
            .iter()
            .find(|hazard| hazard.code == "custom_attribute_macros" && hazard.severity == "warning")
            .expect("custom attribute hazard should be reported");
        assert!(attribute_hazard.details.iter().any(|detail| {
            detail.subject == "app"
                && detail
                    .file
                    .as_ref()
                    .is_some_and(|file| file.ends_with("app/src/lib.rs"))
                && detail.start_line == Some(8)
        }));
        let derive_hazard = report
            .production
            .hazards
            .iter()
            .find(|hazard| hazard.code == "custom_derive_macros" && hazard.severity == "warning")
            .expect("custom derive hazard should be reported");
        assert!(derive_hazard
            .details
            .iter()
            .any(|detail| detail.start_line == Some(9)));
        assert_eq!(report.production.status, "requires_feedback");
    }

    #[test]
    fn reports_retained_module_boundary_attribute_macro_hazards() {
        let root = temp_output("module-boundary-attribute-hazard-source");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[features]\nffi = []\n\n[dependencies]\nopensourced = {{ path = {:?} }}\n",
                opensourced_path
            ),
        );
        write(
            root.join("app/src/lib.rs"),
            r#"#[custom_attr::decorate]
#[cfg_attr(feature = "ffi", custom_attr::ffi_module)]
mod api;
"#,
        );
        write(
            root.join("app/src/api.rs"),
            r#"use opensourced::opensourced;

#[opensourced]
pub fn entry() -> i32 {
    1
}
"#,
        );

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: temp_output("module-boundary-attribute-hazard-output"),
        })
        .expect("reduction should succeed");

        assert!(
            report
                .production
                .hazards
                .iter()
                .any(|hazard| hazard.code == "custom_attribute_macros"
                    && hazard.severity == "warning")
        );
        let cfg_hazard = report
            .production
            .hazards
            .iter()
            .find(|hazard| {
                hazard.code == "conditional_compilation_attrs" && hazard.severity == "warning"
            })
            .expect("module boundary cfg_attr should be reported");
        assert!(cfg_hazard.details.iter().any(|detail| {
            detail.subject == "app::api"
                && detail.module_path.as_deref() == Some("api")
                && detail
                    .file
                    .as_ref()
                    .is_some_and(|file| file.ends_with("app/src/lib.rs"))
                && detail.cfg.as_ref().is_some_and(|cfg| cfg.contains("ffi"))
                && detail.suggested_cargo_args == vec!["--features".to_string(), "ffi".to_string()]
        }));
        assert!(report
            .production
            .hazards
            .iter()
            .any(|hazard| hazard.code == "cfg_gated_roots" && hazard.severity == "error"));
        assert_eq!(report.production.status, "hazards_detected");
    }

    #[test]
    fn reports_retained_non_root_cfg_surfaces_as_feedback_hazards() {
        let root = temp_output("cfg-surface-hazard-source");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[features]\nextra = []\n\n[dependencies]\nopensourced = {{ path = {:?} }}\n",
                opensourced_path
            ),
        );
        write(
            root.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;

#[opensourced]
pub fn entry() -> Payload {
    Payload { value: 1 }
}

pub struct Payload {
    value: usize,
    #[cfg(feature = "extra")]
    extra: Extra,
}

#[cfg(feature = "extra")]
pub struct Extra;
"#,
        );

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: temp_output("cfg-surface-hazard-output"),
        })
        .expect("reduction should succeed");

        let hazard = report
            .production
            .hazards
            .iter()
            .find(|hazard| {
                hazard.code == "conditional_compilation_attrs" && hazard.severity == "warning"
            })
            .expect("retained cfg surfaces should be reported");
        assert!(hazard.details.iter().any(|detail| {
            detail.subject == "app"
                && detail.package.as_deref() == Some("app")
                && detail
                    .file
                    .as_ref()
                    .is_some_and(|file| file.ends_with("app/src/lib.rs"))
                && detail.cfg.as_ref().is_some_and(|cfg| cfg.contains("extra"))
                && detail.suggested_cargo_args
                    == vec!["--features".to_string(), "extra".to_string()]
        }));
        assert_eq!(report.production.status, "requires_feedback");
    }

    #[test]
    fn reports_reachable_cfg_attr_macro_production_hazards() {
        let root = temp_output("cfg-attribute-hazard-source");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\n",
                opensourced_path
            ),
        );
        write(
            root.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;

#[opensourced]
pub fn entry() -> Payload {
    Payload { value: 1 }
}

#[cfg_attr(feature = "ffi", cfg_attr(feature = "bindings", uniffi::export))]
#[cfg_attr(feature = "ffi", cfg_attr(feature = "bindings", derive(Debug, uniffi::Record)))]
pub struct Payload {
    value: i32,
}
"#,
        );

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: temp_output("cfg-attribute-hazard-output"),
        })
        .expect("reduction should succeed");

        assert!(
            report
                .production
                .hazards
                .iter()
                .any(|hazard| hazard.code == "custom_attribute_macros"
                    && hazard.severity == "warning")
        );
        assert!(report
            .production
            .hazards
            .iter()
            .any(|hazard| hazard.code == "custom_derive_macros" && hazard.severity == "warning"));
        assert!(report
            .production
            .hazards
            .iter()
            .any(|hazard| hazard.code == "conditional_compilation_attrs"
                && hazard.severity == "warning"));
        assert_eq!(report.production.status, "requires_feedback");
    }

    #[test]
    fn reports_error_hazard_for_cfg_gated_root_function() {
        let root = temp_output("cfg-root-source");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[features]\nselected = []\n\n[dependencies]\nopensourced = {{ path = {:?} }}\n",
                opensourced_path
            ),
        );
        write(
            root.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;

#[cfg(feature = "selected")]
#[opensourced]
pub fn entry() -> usize {
    1
}
"#,
        );

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: temp_output("cfg-root-output"),
        })
        .expect("reduction should succeed");

        assert_eq!(report.production.status, "hazards_detected");
        let hazard = report
            .production
            .hazards
            .iter()
            .find(|hazard| hazard.code == "cfg_gated_roots" && hazard.severity == "error")
            .expect("cfg-gated root hazard should be reported");
        assert_eq!(hazard.details.len(), 1);
        let detail = &hazard.details[0];
        assert_eq!(detail.subject, "app::entry");
        assert_eq!(
            detail.suggested_cargo_args,
            vec!["--features".to_string(), "selected".to_string()]
        );
        assert!(detail
            .cfg
            .as_ref()
            .is_some_and(|cfg| cfg.contains("selected")));
    }

    #[test]
    fn runtime_benchmark_feature_cfg_root_is_reported_not_pruned() {
        let root = temp_output("cfg-runtime-benchmark-root-source");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[features]\nruntime-benchmarks = []\n\n[dependencies]\nopensourced = {{ path = {:?} }}\n",
                opensourced_path
            ),
        );
        write(
            root.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;

#[cfg(feature = "runtime-benchmarks")]
#[opensourced]
pub fn entry() -> usize {
    1
}
"#,
        );

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: temp_output("cfg-runtime-benchmark-root-output"),
        })
        .expect("reduction should succeed");

        assert!(report.reachable.iter().any(|callable| matches!(
            callable,
            CallableId::Free { package, name, .. } if package == "app" && name == "entry"
        )));
        assert_eq!(report.production.status, "hazards_detected");
        assert!(report
            .production
            .hazards
            .iter()
            .any(|hazard| hazard.code == "cfg_gated_roots" && hazard.severity == "error"));
    }

    #[test]
    fn reports_error_hazard_for_root_inside_cfg_gated_module() {
        let root = temp_output("cfg-module-root-source");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[features]\nselected = []\n\n[dependencies]\nopensourced = {{ path = {:?} }}\n",
                opensourced_path
            ),
        );
        write(
            root.join("app/src/lib.rs"),
            r#"#[cfg(feature = "selected")]
pub mod gated;
"#,
        );
        write(
            root.join("app/src/gated.rs"),
            r#"use opensourced::opensourced;

#[opensourced]
pub fn entry() -> usize {
    1
}
"#,
        );

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: temp_output("cfg-module-root-output"),
        })
        .expect("reduction should succeed");

        assert_eq!(report.production.status, "hazards_detected");
        let hazard = report
            .production
            .hazards
            .iter()
            .find(|hazard| hazard.code == "cfg_gated_roots" && hazard.severity == "error")
            .expect("cfg-gated module root hazard should be reported");
        assert_eq!(hazard.details.len(), 1);
        let detail = &hazard.details[0];
        assert_eq!(detail.subject, "app::gated::entry");
        assert_eq!(detail.module_path.as_deref(), Some("gated"));
        assert_eq!(
            detail.suggested_cargo_args,
            vec!["--features".to_string(), "selected".to_string()]
        );
    }

    #[test]
    fn feedback_diagnostics_widen_matching_unmarked_roots() {
        let root = temp_output("feedback-widen-source");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\n",
                opensourced_path
            ),
        );
        write(
            root.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;

#[opensourced]
pub fn entry() -> usize {
    1
}

pub fn helper() -> usize {
    2
}
"#,
        );

        let report = generate_with_analyzer_feedback(
            GenerateOptions {
                workspace_root: root,
                output_root: temp_output("feedback-widen-output"),
            },
            AnalyzerMode::Syn,
            &[CheckDiagnostic {
                level: "error".to_string(),
                message: "cannot find value `helper` in this scope".to_string(),
                code: Some("E0425".to_string()),
                package_id: Some("app 0.1.0 (path+file:///tmp/app)".to_string()),
                target: None,
                rendered: None,
                spans: Vec::new(),
                suggestions: Vec::new(),
            }],
        )
        .expect("feedback widening should generate");

        assert!(report
            .feedback_widened_roots
            .iter()
            .any(|root| root.to_string() == "app::helper"));
        assert!(report
            .reachable
            .iter()
            .any(|callable| callable.to_string() == "app::helper"));
    }

    #[test]
    fn refuses_to_overwrite_unmarked_nonempty_output() {
        let output = temp_output("unmarked-output");
        fs::create_dir_all(&output).unwrap();
        fs::write(output.join("keep.txt"), "do not delete").unwrap();

        let error = generate(GenerateOptions {
            workspace_root: workspace_root(),
            output_root: output.clone(),
        })
        .expect_err("unmarked non-empty outputs should fail closed");

        assert!(
            error.to_string().contains("not created by slicers"),
            "unexpected error: {error}"
        );
        assert!(
            output.join("keep.txt").exists(),
            "unmarked output contents must not be deleted"
        );
    }

    #[test]
    fn reruns_can_replace_marked_outputs() {
        let output = temp_output("marked-output");
        generate(GenerateOptions {
            workspace_root: workspace_root(),
            output_root: output.clone(),
        })
        .expect("initial reduction should succeed");
        assert!(output.join(".slicers-output").exists());
        fs::write(output.join("stale.txt"), "stale").unwrap();

        generate(GenerateOptions {
            workspace_root: workspace_root(),
            output_root: output.clone(),
        })
        .expect("marked output should be replaceable");

        assert!(output.join(".slicers-output").exists());
        assert!(
            !output.join("stale.txt").exists(),
            "rerender should replace stale generated output contents"
        );
    }

    #[test]
    fn refuses_output_inside_input_workspace() {
        let output = workspace_root().join("target/slicers-inside-workspace-output");
        if output.exists() {
            fs::remove_dir_all(&output).unwrap();
        }

        let error = generate(GenerateOptions {
            workspace_root: workspace_root(),
            output_root: output.clone(),
        })
        .expect_err("output inside source workspace should be rejected");

        assert!(
            error.to_string().contains("inside input workspace"),
            "unexpected error: {error}"
        );
        assert!(
            !output.exists(),
            "rejected inside-workspace output should not be created"
        );
    }

    #[test]
    fn rejects_path_attributed_modules_that_escape_package_output() {
        let root = temp_output("path-attr-escape-source");
        let output = temp_output("path-attr-escape-output");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\n",
                opensourced_path
            ),
        );
        write(
            root.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;

#[path = "../../outside.rs"]
mod escaped;

#[opensourced]
pub fn entry() -> usize {
    escaped::value()
}
"#,
        );
        write(root.join("outside.rs"), "pub fn value() -> usize { 1 }\n");

        let error = generate(GenerateOptions {
            workspace_root: root,
            output_root: output.clone(),
        })
        .expect_err("escaped path-attributed module should be rejected");

        assert!(
            error.to_string().contains("not relative to package root"),
            "unexpected error: {error}"
        );
        assert!(
            !output.join("outside.rs").exists(),
            "escaped source must not be written outside the package output"
        );
    }

    fn workspace_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .expect("core crate should live under crates/opensource_core")
            .to_path_buf()
    }

    fn temp_output(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("opensourced-{label}-{unique}"));
        if path.exists() {
            fs::remove_dir_all(&path).unwrap();
        }
        path
    }

    fn write(path: PathBuf, contents: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, contents).unwrap();
    }
}
