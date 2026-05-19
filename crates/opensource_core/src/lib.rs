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
    ffi::OsStr,
    fs,
    path::{Component, Path, PathBuf},
    time::Instant,
};

use include_path::{static_include_path, StaticIncludePath};
use model::{Project, ReducedProject};
use proc_macro2::TokenStream;
use quote::ToTokens;
use serde::Serialize;
use syn::{
    parse::Parser,
    punctuated::Punctuated,
    spanned::Spanned,
    visit::{self, Visit},
    Attribute, Expr, Item, Lit, Macro, Meta, UseTree,
};

pub use analyzer::{
    AnalyzerMode, AnalyzerReport, SemanticFileReport, SemanticReport, SemanticUnresolvedCategory,
    SemanticUnresolvedDiagnostic, SemanticUnresolvedKind, SemanticUsageReport,
};
pub use feedback::{
    check_workspace, write_report, CheckDiagnostic, CheckOptions, CheckReport, CheckSpan,
    CheckSuggestion, CheckTarget, FeedbackHazard, FeedbackWideningCandidate,
    FeedbackWideningReport,
};
pub use manifest::marked_workspace_packages;
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

pub struct GenerateSession {
    workspace_root: PathBuf,
    project: Project,
    analyzer: AnalyzerReport,
    manifest_ms: u64,
    parse_ms: u64,
    analyzer_ms: u64,
}

#[derive(Debug, Clone)]
pub struct GenerateSessionLoadProgress {
    pub event: &'static str,
    pub status: &'static str,
    pub decision: &'static str,
    pub reason: String,
    pub fields: BTreeMap<String, String>,
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
    pub macro_surfaces: MacroSurfaceReport,
    pub usage: UsageClassificationReport,
    pub source_map: SourceMapReport,
    pub files_written: usize,
    pub timings: GenerateTimingReport,
}

#[derive(Debug, Clone, Serialize)]
pub struct FeedbackRootResolutionReport {
    pub manifest_ms: u64,
    pub parse_ms: u64,
    pub diagnostics: usize,
    pub error_diagnostics: usize,
    pub skipped_non_error_diagnostics: usize,
    pub skipped_missing_code_diagnostics: usize,
    pub candidate_symbols: usize,
    pub skipped_missing_symbol: usize,
    pub skipped_no_match: usize,
    pub skipped_too_many_matches: usize,
    pub skipped_marked_roots: usize,
    pub matched_roots: Vec<String>,
    pub entries: Vec<FeedbackRootResolutionEntry>,
    pub entries_truncated: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct FeedbackRootResolutionEntry {
    pub code: String,
    pub symbol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_hint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostic_file: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub glob_imports: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub glob_import_module_roots: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub glob_import_provider_internal_globs: Vec<String>,
    pub matches: usize,
    pub retained_roots: usize,
    pub skipped_marked_roots: usize,
    pub action: String,
    pub roots: Vec<String>,
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
    pub blocked_idents: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suggested_cargo_args: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct MacroSurfaceReport {
    pub summary: MacroSurfaceSummary,
    pub surfaces: Vec<MacroSurface>,
}

#[derive(Debug, Clone, Default)]
pub struct MacroSurfaceSummary {
    pub total: usize,
    pub derive_macros: usize,
    pub attribute_macros: usize,
    pub helper_attributes: usize,
    pub macro_invocations: usize,
    pub macro_blocked: usize,
}

#[derive(Debug, Clone)]
pub struct MacroSurface {
    pub kind: String,
    pub category: String,
    pub path: String,
    pub subject: String,
    pub package: String,
    pub module_path: Option<String>,
    pub owner: Option<String>,
    pub file: Option<PathBuf>,
    pub start_line: Option<usize>,
    pub blocked_idents: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct UsageClassificationReport {
    pub status: String,
    pub summary: UsageClassificationSummary,
    pub semantic_proof: SemanticUsageProofReport,
    pub rendered_symbols: RenderedSymbolProofReport,
    pub rendered_decision_map: RenderedUsageDecisionMap,
    pub public_reexports: PublicReexportProofReport,
    pub used: UsageClassifiedItems,
    pub unused_candidate: UsageClassifiedItems,
    pub blocked_by_unknown: UsageClassifiedItems,
    pub prunable: UsageClassifiedItems,
    pub unused: UsageClassifiedItems,
    pub unknown: Vec<UsageUnknownSurface>,
    pub evidence: UsageClassificationEvidence,
}

#[derive(Debug, Clone)]
pub struct UsageClassificationSummary {
    pub indexed_callables: usize,
    pub indexed_items: usize,
    pub used_callables: usize,
    pub used_items: usize,
    pub unused_candidate_callables: usize,
    pub unused_candidate_items: usize,
    pub blocked_by_unknown_callables: usize,
    pub blocked_by_unknown_items: usize,
    pub prunable_callables: usize,
    pub prunable_items: usize,
    pub unused_callables: usize,
    pub unused_items: usize,
    pub unknown_surfaces: usize,
    pub benign_unknown_surfaces: usize,
    pub macro_blocked_unknown_surfaces: usize,
    pub dependency_risk_unknown_surfaces: usize,
}

#[derive(Debug, Clone)]
pub struct PublicReexportProofReport {
    pub status: String,
    pub summary: PublicReexportProofSummary,
    pub entries: Vec<PublicReexportProofEntry>,
}

impl Default for PublicReexportProofReport {
    fn default() -> Self {
        Self {
            status: "not_checked".to_string(),
            summary: PublicReexportProofSummary::default(),
            entries: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PublicReexportProofSummary {
    pub public_reexports: usize,
    pub local_targets: usize,
    pub retained_targets: usize,
    pub prunable_targets: usize,
    pub unclassified_targets: usize,
    pub external_or_unresolved_targets: usize,
    pub facade_chain_targets: usize,
    pub source_parse_failures: usize,
}

#[derive(Debug, Clone)]
pub struct PublicReexportProofEntry {
    pub package: String,
    pub module_path: Option<String>,
    pub visible: String,
    pub target: String,
    pub resolved_targets: Vec<String>,
    pub classification: String,
}

#[derive(Debug, Clone)]
pub struct RenderedSymbolProofReport {
    pub status: String,
    pub summary: RenderedSymbolProofSummary,
    pub entries: Vec<RenderedSymbolProofEntry>,
}

impl Default for RenderedSymbolProofReport {
    fn default() -> Self {
        Self {
            status: "not_checked".to_string(),
            summary: RenderedSymbolProofSummary::default(),
            entries: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RenderedSymbolProofSummary {
    pub rendered_callables: usize,
    pub rendered_items: usize,
    pub rendered_members: usize,
    pub rendered_assoc_items: usize,
    pub rendered_trait_default_methods: usize,
    pub retained_callables: usize,
    pub retained_items: usize,
    pub retained_members: usize,
    pub retained_assoc_items: usize,
    pub retained_trait_default_methods: usize,
    pub macro_blocked_callables: usize,
    pub surface_blocked_callables: usize,
    pub blocked_members: usize,
    pub blocked_assoc_items: usize,
    pub blocked_trait_default_methods: usize,
    pub prunable_callables: usize,
    pub prunable_items: usize,
    pub prunable_members: usize,
    pub prunable_assoc_items: usize,
    pub unclassified_callables: usize,
    pub unclassified_items: usize,
    pub unclassified_members: usize,
    pub unclassified_assoc_items: usize,
    pub unproven_trait_default_methods: usize,
    pub source_parse_failures: usize,
}

#[derive(Debug, Clone)]
pub struct RenderedSymbolProofEntry {
    pub kind: String,
    pub id: String,
    pub classification: String,
}

#[derive(Debug, Clone)]
pub struct SemanticUsageProofReport {
    pub status: String,
    pub summary: SemanticUsageProofSummary,
    pub unproven: UsageClassifiedItems,
}

#[derive(Debug, Clone, Default)]
pub struct SemanticUsageProofSummary {
    pub analyzer_available: bool,
    pub retained_packages: usize,
    pub prunable_callables: usize,
    pub prunable_items: usize,
    pub package_pruned_callables: usize,
    pub package_pruned_items: usize,
    pub proof_required_callables: usize,
    pub proof_required_items: usize,
    pub proven_callables: usize,
    pub proven_items: usize,
    pub unproven_callables: usize,
    pub unproven_items: usize,
    pub cfg_inactive_callables: usize,
    pub cfg_inactive_items: usize,
    pub source_file_pruned_callables: usize,
    pub source_file_pruned_items: usize,
    pub rendered_absent_callables: usize,
    pub rendered_absent_items: usize,
    pub structural_pruned_items: usize,
    pub unmapped_callables: usize,
    pub unmapped_items: usize,
    pub failed_reference_query_callables: usize,
    pub failed_reference_query_items: usize,
    pub skipped_reference_query_callables: usize,
    pub skipped_reference_query_items: usize,
    pub retained_reference_callables: usize,
    pub retained_reference_items: usize,
}

#[derive(Debug, Clone)]
pub struct UsageClassifiedItems {
    pub callables: Vec<CallableId>,
    pub items: Vec<ItemId>,
}

#[derive(Debug, Clone, Default)]
pub struct RenderedUsageDecisionMap {
    pub callables: BTreeMap<String, String>,
    pub items: BTreeMap<String, String>,
    pub members: BTreeMap<String, String>,
    pub assoc_items: BTreeMap<String, String>,
    pub trait_default_methods: BTreeMap<String, String>,
}

impl RenderedUsageDecisionMap {
    pub fn from_rendered_symbols(report: &RenderedSymbolProofReport) -> Self {
        Self::from_rendered_symbols_with_decisions(report, None)
    }

    fn from_rendered_symbols_and_decisions(
        report: &RenderedSymbolProofReport,
        decisions: &UsageDecisionIndex,
    ) -> Self {
        Self::from_rendered_symbols_with_decisions(report, Some(decisions))
    }

    fn from_rendered_symbols_with_decisions(
        report: &RenderedSymbolProofReport,
        decisions: Option<&UsageDecisionIndex>,
    ) -> Self {
        let callable_decisions = decisions
            .map(rendered_callable_decision_strings)
            .unwrap_or_default();
        let item_decisions = decisions
            .map(rendered_item_decision_strings)
            .unwrap_or_default();
        let mut map = Self::default();
        for entry in &report.entries {
            let classification = match entry.kind.as_str() {
                "callable" => final_rendered_decision(
                    &entry.classification,
                    callable_decisions.get(&entry.id),
                ),
                "item" => {
                    final_rendered_decision(&entry.classification, item_decisions.get(&entry.id))
                }
                _ => normalize_rendered_decision(&entry.classification),
            };
            let target = match entry.kind.as_str() {
                "callable" => &mut map.callables,
                "item" => &mut map.items,
                "member" => &mut map.members,
                "assoc_item" => &mut map.assoc_items,
                "trait_default_method" => &mut map.trait_default_methods,
                _ => continue,
            };
            target.insert(entry.id.clone(), classification);
        }
        map
    }
}

fn rendered_callable_decision_strings(decisions: &UsageDecisionIndex) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    insert_rendered_decision_strings(&mut map, &decisions.used_callables, UsageDecision::Used);
    insert_rendered_decision_strings(
        &mut map,
        &decisions.blocked_by_unknown_callables,
        UsageDecision::BlockedByUnknown,
    );
    insert_rendered_decision_strings(
        &mut map,
        &decisions.prunable_callables,
        UsageDecision::Prunable,
    );
    map
}

fn rendered_item_decision_strings(decisions: &UsageDecisionIndex) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    insert_rendered_decision_strings(&mut map, &decisions.used_items, UsageDecision::Used);
    insert_rendered_decision_strings(
        &mut map,
        &decisions.blocked_by_unknown_items,
        UsageDecision::BlockedByUnknown,
    );
    insert_rendered_decision_strings(&mut map, &decisions.prunable_items, UsageDecision::Prunable);
    map
}

fn insert_rendered_decision_strings<T: ToString>(
    map: &mut BTreeMap<String, String>,
    ids: &BTreeSet<T>,
    decision: UsageDecision,
) {
    for id in ids {
        map.insert(id.to_string(), decision.as_str().to_string());
    }
}

fn normalize_rendered_decision(classification: &str) -> String {
    if classification == "retained" {
        "used".to_string()
    } else {
        classification.to_string()
    }
}

fn final_rendered_decision(classification: &str, graph_decision: Option<&String>) -> String {
    match classification {
        "retained" => {
            if graph_decision.is_some_and(|decision| decision == "blocked_by_unknown") {
                "blocked_by_unknown".to_string()
            } else {
                "used".to_string()
            }
        }
        "blocked_by_unknown" => "blocked_by_unknown".to_string(),
        _ => classification.to_string(),
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum UsageDecision {
    Used,
    BlockedByUnknown,
    Prunable,
}

impl UsageDecision {
    fn as_str(self) -> &'static str {
        match self {
            Self::Used => "used",
            Self::BlockedByUnknown => "blocked_by_unknown",
            Self::Prunable => "prunable",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct UsageDecisionIndex {
    retained_packages: BTreeSet<String>,
    used_callables: BTreeSet<CallableId>,
    used_items: BTreeSet<ItemId>,
    blocked_by_unknown_callables: BTreeSet<CallableId>,
    blocked_by_unknown_items: BTreeSet<ItemId>,
    prunable_callables: BTreeSet<CallableId>,
    prunable_items: BTreeSet<ItemId>,
    callable_decisions: BTreeMap<CallableId, UsageDecision>,
    item_decisions: BTreeMap<ItemId, UsageDecision>,
}

impl UsageDecisionIndex {
    fn from_slice_plan(
        project: &Project,
        root_reduced: &ReducedProject,
        render_reduced: &ReducedProject,
    ) -> Self {
        let used_callables = root_reduced.reachable.clone();
        let used_items = root_reduced.reachable_items.clone();
        let retained_packages = render_reduced.packages.clone();
        let blocked_by_unknown_callables = render_reduced
            .reachable
            .difference(&used_callables)
            .cloned()
            .collect::<BTreeSet<_>>();
        let blocked_by_unknown_items = render_reduced
            .reachable_items
            .difference(&used_items)
            .cloned()
            .collect::<BTreeSet<_>>();
        let retained_callables = render_reduced.reachable.clone();
        let retained_items = render_reduced.reachable_items.clone();
        let prunable_callables = project
            .functions
            .keys()
            .chain(project.methods.keys())
            .filter(|id| !retained_callables.contains(*id))
            .cloned()
            .collect::<BTreeSet<_>>();
        let prunable_items = project
            .items
            .keys()
            .filter(|id| !retained_items.contains(*id))
            .cloned()
            .collect::<BTreeSet<_>>();
        let callable_decisions = usage_decision_map(
            &used_callables,
            &blocked_by_unknown_callables,
            &prunable_callables,
        );
        let item_decisions =
            usage_decision_map(&used_items, &blocked_by_unknown_items, &prunable_items);

        Self {
            retained_packages,
            used_callables,
            used_items,
            blocked_by_unknown_callables,
            blocked_by_unknown_items,
            prunable_callables,
            prunable_items,
            callable_decisions,
            item_decisions,
        }
    }

    pub fn should_render_callable(&self, callable: &CallableId) -> bool {
        !self.can_remove_callable(callable)
    }

    pub fn should_render_item(&self, item: &ItemId) -> bool {
        !self.can_remove_item(item)
    }

    pub fn can_remove_callable(&self, callable: &CallableId) -> bool {
        self.callable_decision(callable) == Some(UsageDecision::Prunable)
    }

    pub fn can_remove_item(&self, item: &ItemId) -> bool {
        self.item_decision(item) == Some(UsageDecision::Prunable)
    }

    pub fn is_blocked_by_unknown_item(&self, item: &ItemId) -> bool {
        self.item_decision(item) == Some(UsageDecision::BlockedByUnknown)
    }

    pub fn callable_decision(&self, callable: &CallableId) -> Option<UsageDecision> {
        self.callable_decisions.get(callable).copied()
    }

    pub fn item_decision(&self, item: &ItemId) -> Option<UsageDecision> {
        self.item_decisions.get(item).copied()
    }

    fn used_callables(&self) -> Vec<CallableId> {
        self.used_callables.iter().cloned().collect()
    }

    fn used_items(&self) -> Vec<ItemId> {
        self.used_items.iter().cloned().collect()
    }

    fn blocked_by_unknown_callables(&self) -> Vec<CallableId> {
        self.blocked_by_unknown_callables.iter().cloned().collect()
    }

    pub fn blocked_by_unknown_items(&self) -> Vec<ItemId> {
        self.blocked_by_unknown_items.iter().cloned().collect()
    }

    fn prunable_callables(&self) -> Vec<CallableId> {
        self.prunable_callables.iter().cloned().collect()
    }

    fn prunable_items(&self) -> Vec<ItemId> {
        self.prunable_items.iter().cloned().collect()
    }
}

fn usage_decision_map<T: Ord + Clone>(
    used: &BTreeSet<T>,
    blocked_by_unknown: &BTreeSet<T>,
    prunable: &BTreeSet<T>,
) -> BTreeMap<T, UsageDecision> {
    let mut decisions = BTreeMap::new();
    for id in used {
        debug_assert!(decisions.insert(id.clone(), UsageDecision::Used).is_none());
    }
    for id in blocked_by_unknown {
        debug_assert!(decisions
            .insert(id.clone(), UsageDecision::BlockedByUnknown)
            .is_none());
    }
    for id in prunable {
        debug_assert!(decisions
            .insert(id.clone(), UsageDecision::Prunable)
            .is_none());
    }
    decisions
}

#[derive(Debug, Clone)]
struct SlicePlan {
    render_reduced: ReducedProject,
    usage_decisions: UsageDecisionIndex,
}

impl SlicePlan {
    fn build(
        project: &Project,
        root_reduced: &ReducedProject,
        analyzer: &AnalyzerReport,
        pre_render_production: &ProductionReadinessReport,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let unknown_retention =
            UnknownRetentionPlan::build(project, root_reduced, analyzer, pre_render_production)?;
        let mut render_extra_roots = root_reduced.roots.clone();
        render_extra_roots.extend(unknown_retention.roots.iter().cloned());
        render_extra_roots.sort();
        render_extra_roots.dedup();
        let mut render_reduced = if unknown_retention.roots.is_empty() {
            root_reduced.clone()
        } else if analyzer.semantic_hints.is_empty() {
            reduce::reduce_with_extra_roots(project, &render_extra_roots)?
        } else {
            reduce::reduce_with_extra_roots_and_semantics(
                project,
                &render_extra_roots,
                &analyzer.semantic_hints,
            )?
        };
        render_reduced.root = root_reduced.root.clone();
        render_reduced.roots = root_reduced.roots.clone();

        let usage_decisions =
            UsageDecisionIndex::from_slice_plan(project, root_reduced, &render_reduced);
        Ok(Self {
            render_reduced,
            usage_decisions,
        })
    }
}

#[derive(Debug, Clone, Default)]
struct UnknownRetentionPlan {
    roots: Vec<RootId>,
}

impl UnknownRetentionPlan {
    fn build(
        project: &Project,
        reduced: &ReducedProject,
        analyzer: &AnalyzerReport,
        pre_render_production: &ProductionReadinessReport,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut roots = deletion_blocked_extra_roots(project, reduced, pre_render_production);

        for _ in 0..8 {
            roots.sort();
            roots.dedup();
            let current = if roots.is_empty() {
                reduced.clone()
            } else if analyzer.semantic_hints.is_empty() {
                reduce::reduce_with_extra_roots(project, &roots)?
            } else {
                reduce::reduce_with_extra_roots_and_semantics(
                    project,
                    &roots,
                    &analyzer.semantic_hints,
                )?
            };
            let before = roots.len();
            roots.extend(semantic_usage_unknown_extra_roots(
                project, &current, analyzer,
            ));
            roots.extend(generated_source_unknown_method_extra_roots(
                project,
                &current,
                pre_render_production,
            ));
            roots.sort();
            roots.dedup();
            if roots.len() == before {
                break;
            }
        }

        Ok(Self { roots })
    }
}

fn generated_source_unknown_method_extra_roots(
    project: &Project,
    reduced: &ReducedProject,
    production: &ProductionReadinessReport,
) -> Vec<RootId> {
    let mut packages = generated_source_unknown_packages(production);
    packages.extend(
        reduced
            .packages
            .iter()
            .filter(|package| package_has_generated_source_unknown_surface(project, package))
            .cloned(),
    );
    if packages.is_empty() {
        return Vec::new();
    }

    let mut roots = Vec::new();
    for package in packages {
        if !reduced.packages.contains(&package) {
            continue;
        }
        let method_roots = generated_source_reachable_method_names(project, reduced, &package);
        if method_roots.is_empty() {
            continue;
        }

        let mut methods_by_type =
            BTreeMap::<Vec<String>, BTreeMap<String, &syn::ImplItemFn>>::new();
        for (id, record) in &project.methods {
            let CallableId::Method {
                package: method_package,
                type_path,
                trait_path,
                method,
                ..
            } = id
            else {
                continue;
            };
            if method_package != &package
                || trait_path.is_some()
                || !rendered_type_path_is_reachable(reduced, &package, type_path)
            {
                continue;
            }
            methods_by_type
                .entry(type_path.clone())
                .or_default()
                .insert(method.clone(), &record.item);
        }

        for (type_path, methods) in methods_by_type {
            let mut pending = method_roots
                .iter()
                .filter(|method| methods.contains_key(*method))
                .cloned()
                .collect::<Vec<_>>();
            let mut seen = BTreeSet::new();
            while let Some(method) = pending.pop() {
                if !seen.insert(method.clone()) {
                    continue;
                }
                let id = CallableId::Method {
                    package: package.clone(),
                    type_path: type_path.clone(),
                    trait_path: None,
                    trait_input_type_paths: Vec::new(),
                    method: method.clone(),
                };
                if !reduced.reachable.contains(&id) {
                    roots.push(RootId::Callable(id));
                }
                let Some(item_fn) = methods.get(&method) else {
                    continue;
                };
                for called in generated_method_call_names_from_tokens(&item_fn.to_token_stream()) {
                    if !seen.contains(&called) && methods.contains_key(&called) {
                        pending.push(called);
                    }
                }
            }
        }
    }
    roots
}

fn generated_source_unknown_packages(production: &ProductionReadinessReport) -> BTreeSet<String> {
    production
        .hazards
        .iter()
        .filter(|hazard| {
            matches!(
                hazard.code.as_str(),
                "source_include_macros"
                    | "out_dir_source_include_macros"
                    | "retained_build_scripts"
            )
        })
        .flat_map(|hazard| {
            hazard
                .details
                .iter()
                .filter_map(|detail| detail.package.clone())
        })
        .collect()
}

fn rendered_type_path_is_reachable(
    reduced: &ReducedProject,
    package: &str,
    type_path: &[String],
) -> bool {
    let Some((name, module_path)) = type_path.split_last() else {
        return false;
    };
    reduced.reachable_items.iter().any(|item| {
        item.package == package && item.module_path == module_path && item.name == *name
    })
}

fn generated_source_reachable_method_names(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
) -> BTreeSet<String> {
    let mut methods = BTreeSet::new();
    let source_include_modules = generated_source_include_module_names(project, package);
    let package_has_build_script = project
        .workspace
        .packages
        .get(package)
        .is_some_and(generated_package_has_build_script);
    for callable in &reduced.reachable {
        if callable.package() != package {
            continue;
        }
        if let Some(record) = project.functions.get(callable) {
            methods.extend(generated_source_method_names_from_item_fn(
                &record.item,
                &source_include_modules,
                package_has_build_script,
            ));
        } else if let Some(record) = project.methods.get(callable) {
            methods.extend(generated_source_method_names_from_impl_item_fn(
                &record.item,
                &source_include_modules,
                package_has_build_script,
            ));
        }
    }
    for item in &reduced.reachable_items {
        if item.package != package {
            continue;
        }
        if let Some(record) = project.items.get(item) {
            methods.extend(generated_source_method_names_from_item(
                &record.item,
                &source_include_modules,
                package_has_build_script,
            ));
        }
    }
    methods
}

fn generated_source_method_names_from_item_fn(
    item: &syn::ItemFn,
    source_include_modules: &BTreeSet<String>,
    package_has_build_script: bool,
) -> BTreeSet<String> {
    if source_include_modules.is_empty() {
        return package_has_build_script
            .then(|| generated_method_call_names_from_tokens(&item.to_token_stream()))
            .unwrap_or_default();
    }
    let mut visitor = GeneratedSourceReceiverMethodVisitor {
        source_include_modules,
        method_names: BTreeSet::new(),
    };
    visitor.visit_item_fn(item);
    visitor.method_names
}

fn generated_source_method_names_from_impl_item_fn(
    item: &syn::ImplItemFn,
    source_include_modules: &BTreeSet<String>,
    package_has_build_script: bool,
) -> BTreeSet<String> {
    if source_include_modules.is_empty() {
        return package_has_build_script
            .then(|| generated_method_call_names_from_tokens(&item.to_token_stream()))
            .unwrap_or_default();
    }
    let mut visitor = GeneratedSourceReceiverMethodVisitor {
        source_include_modules,
        method_names: BTreeSet::new(),
    };
    visitor.visit_impl_item_fn(item);
    visitor.method_names
}

fn generated_source_method_names_from_item(
    item: &Item,
    source_include_modules: &BTreeSet<String>,
    package_has_build_script: bool,
) -> BTreeSet<String> {
    if source_include_modules.is_empty() {
        return package_has_build_script
            .then(|| generated_method_call_names_from_tokens(&item.to_token_stream()))
            .unwrap_or_default();
    }
    let mut visitor = GeneratedSourceReceiverMethodVisitor {
        source_include_modules,
        method_names: BTreeSet::new(),
    };
    visitor.visit_item(item);
    visitor.method_names
}

struct GeneratedSourceReceiverMethodVisitor<'a> {
    source_include_modules: &'a BTreeSet<String>,
    method_names: BTreeSet<String>,
}

impl Visit<'_> for GeneratedSourceReceiverMethodVisitor<'_> {
    fn visit_expr_method_call(&mut self, node: &syn::ExprMethodCall) {
        if generated_expr_mentions_any_path_root(&node.receiver, self.source_include_modules) {
            self.method_names.insert(node.method.to_string());
        }
        syn::visit::visit_expr_method_call(self, node);
    }
}

fn generated_expr_mentions_any_path_root(expr: &Expr, roots: &BTreeSet<String>) -> bool {
    let mut visitor = GeneratedPathRootVisitor {
        roots,
        found: false,
    };
    visitor.visit_expr(expr);
    visitor.found
}

struct GeneratedPathRootVisitor<'a> {
    roots: &'a BTreeSet<String>,
    found: bool,
}

impl Visit<'_> for GeneratedPathRootVisitor<'_> {
    fn visit_path(&mut self, path: &syn::Path) {
        if path
            .segments
            .first()
            .is_some_and(|segment| self.roots.contains(&segment.ident.to_string()))
        {
            self.found = true;
            return;
        }
        syn::visit::visit_path(self, path);
    }
}

fn generated_source_include_module_names(project: &Project, package: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for source in project
        .files
        .values()
        .filter(|source| source.package == package)
    {
        for item in &source.syntax.items {
            let Item::Mod(item_mod) = item else {
                continue;
            };
            let module_name = item_mod.ident.to_string();
            let mut child_path = source.module_path.clone();
            child_path.push(module_name.clone());
            let inline_contains = item_mod
                .content
                .as_ref()
                .is_some_and(|(_, items)| items.iter().any(generated_item_contains_source_include));
            let child_contains = project
                .source_files_by_module
                .get(&(package.to_string(), child_path))
                .and_then(|path| project.files.get(path))
                .is_some_and(|child| {
                    child
                        .syntax
                        .items
                        .iter()
                        .any(generated_item_contains_source_include)
                });
            if inline_contains || child_contains {
                names.insert(module_name);
            }
        }
    }
    names
}

fn generated_package_has_build_script(package: &crate::manifest::Package) -> bool {
    package.root.join("build.rs").exists()
        || package
            .manifest
            .get("package")
            .and_then(toml::Value::as_table)
            .is_some_and(|package| package.contains_key("build"))
}

fn package_has_generated_source_unknown_surface(project: &Project, package: &str) -> bool {
    !generated_source_include_module_names(project, package).is_empty()
        || project
            .workspace
            .packages
            .get(package)
            .is_some_and(generated_package_has_build_script)
}

fn generated_item_contains_source_include(item: &Item) -> bool {
    match item {
        Item::Macro(item_macro) => {
            item_macro.ident.is_none() && item_macro.mac.path.is_ident("include")
        }
        Item::Mod(item_mod) => item_mod
            .content
            .as_ref()
            .is_some_and(|(_, items)| items.iter().any(generated_item_contains_source_include)),
        _ => false,
    }
}

fn generated_method_call_names_from_tokens(tokens: &TokenStream) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    collect_generated_method_call_names_from_tokens(tokens, &mut names);
    names
}

fn collect_generated_method_call_names_from_tokens(
    tokens: &TokenStream,
    names: &mut BTreeSet<String>,
) {
    let tokens = tokens.clone().into_iter().collect::<Vec<_>>();
    for token in &tokens {
        if let proc_macro2::TokenTree::Group(group) = token {
            collect_generated_method_call_names_from_tokens(&group.stream(), names);
        }
    }
    for pair in tokens.windows(2) {
        if matches!(&pair[0], proc_macro2::TokenTree::Punct(punct) if punct.as_char() == '.') {
            if let proc_macro2::TokenTree::Ident(ident) = &pair[1] {
                names.insert(ident.to_string());
            }
        }
    }
    for window in tokens.windows(4) {
        let [proc_macro2::TokenTree::Ident(_), proc_macro2::TokenTree::Punct(left), proc_macro2::TokenTree::Punct(right), proc_macro2::TokenTree::Ident(method)] =
            window
        else {
            continue;
        };
        if left.as_char() == ':' && right.as_char() == ':' {
            names.insert(method.to_string());
        }
    }
}

#[derive(Debug, Clone)]
pub struct UsageClassificationEvidence {
    pub callables: Vec<UsageCallableEvidence>,
    pub items: Vec<UsageItemEvidence>,
}

#[derive(Debug, Clone)]
pub struct UsageCallableEvidence {
    pub id: CallableId,
    pub classification: String,
    pub selected_root: bool,
    pub reason: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct UsageItemEvidence {
    pub id: ItemId,
    pub classification: String,
    pub selected_root: bool,
    pub reason: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct UsageUnknownSurface {
    pub category: String,
    pub code: String,
    pub severity: String,
    pub message: String,
    pub details: Vec<ProductionHazardDetail>,
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

pub fn generate_with_analyzer_roots(
    options: GenerateOptions,
    analyzer_mode: AnalyzerMode,
    root_selectors: &[String],
) -> Result<GenerateReport, Box<dyn std::error::Error>> {
    generate_with_analyzer_feedback_and_roots(options, analyzer_mode, &[], root_selectors)
}

pub fn generate_with_analyzer_feedback(
    options: GenerateOptions,
    analyzer_mode: AnalyzerMode,
    feedback_diagnostics: &[feedback::CheckDiagnostic],
) -> Result<GenerateReport, Box<dyn std::error::Error>> {
    generate_with_analyzer_feedback_and_roots(options, analyzer_mode, feedback_diagnostics, &[])
}

pub fn generate_with_analyzer_feedback_and_roots(
    options: GenerateOptions,
    analyzer_mode: AnalyzerMode,
    feedback_diagnostics: &[feedback::CheckDiagnostic],
    root_selectors: &[String],
) -> Result<GenerateReport, Box<dyn std::error::Error>> {
    let (session, roots) = GenerateSession::load_with_root_selectors(
        &options.workspace_root,
        analyzer_mode,
        root_selectors,
    )?;
    session.generate(options.output_root, &roots, feedback_diagnostics)
}

pub fn resolve_feedback_widening_roots(
    workspace_root: &Path,
    feedback_diagnostics: &[feedback::CheckDiagnostic],
    root_selectors: &[String],
) -> Result<FeedbackRootResolutionReport, Box<dyn std::error::Error>> {
    let phase_started = Instant::now();
    let workspace = manifest::load_workspace_without_marker_targets(workspace_root)?;
    let manifest_ms = elapsed_ms(phase_started);

    let phase_started = Instant::now();
    let project = if let Some(packages) = root_selector_package_filter(root_selectors) {
        parse::parse_workspace_package_closure(workspace, &packages)?
    } else {
        parse::parse_workspace(workspace)?
    };
    let parse_ms = elapsed_ms(phase_started);

    let (_roots, mut report) = feedback_extra_roots_with_report(&project, feedback_diagnostics);
    report.manifest_ms = manifest_ms;
    report.parse_ms = parse_ms;
    Ok(report)
}

pub fn direct_free_function_root_selectors(selectors: &[String]) -> Option<Vec<RootId>> {
    let mut roots = Vec::new();
    let mut seen = BTreeSet::new();
    for selector in selectors {
        let root = direct_free_function_root_selector(selector)?;
        if seen.insert(root.clone()) {
            roots.push(root);
        }
    }
    Some(roots)
}

fn direct_free_function_root_selector(selector: &str) -> Option<RootId> {
    let selector = selector.trim();
    if selector.contains('(')
        || selector.contains('<')
        || selector.contains('>')
        || selector.contains(" as ")
        || selector.contains(char::is_whitespace)
    {
        return None;
    }
    let segments = selector.split("::").map(str::trim).collect::<Vec<_>>();
    if segments.len() < 3 || segments.iter().any(|segment| segment.is_empty()) {
        return None;
    }
    let (package, rest) = segments.split_first()?;
    let (name, module_path) = rest.split_last()?;
    if !module_path
        .iter()
        .all(|segment| looks_like_module_segment(segment))
    {
        return None;
    }
    Some(RootId::Callable(CallableId::Free {
        package: (*package).to_string(),
        module_path: module_path
            .iter()
            .map(|segment| (*segment).to_string())
            .collect(),
        name: (*name).to_string(),
    }))
}

fn looks_like_module_segment(segment: &str) -> bool {
    let segment = segment.strip_prefix("r#").unwrap_or(segment);
    segment
        .chars()
        .next()
        .is_some_and(|ch| ch == '_' || ch.is_ascii_lowercase())
}

impl GenerateSession {
    pub fn load(
        workspace_root: &Path,
        analyzer_mode: AnalyzerMode,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Self::load_with_selected_roots(workspace_root, analyzer_mode, &[])
    }

    pub fn load_without_marker_targets(
        workspace_root: &Path,
        analyzer_mode: AnalyzerMode,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let (project, manifest_ms, parse_ms) = load_project_without_marker_targets(workspace_root)?;
        let phase_started = Instant::now();
        let analyzer = analyzer::load_report_for_project_and_roots(
            workspace_root,
            analyzer_mode,
            &project,
            &[],
        )?;
        let analyzer_ms = elapsed_ms(phase_started);

        Ok(Self {
            workspace_root: workspace_root.to_path_buf(),
            project,
            analyzer,
            manifest_ms,
            parse_ms,
            analyzer_ms,
        })
    }

    pub fn load_without_marker_targets_for_root_selectors(
        workspace_root: &Path,
        analyzer_mode: AnalyzerMode,
        root_selectors: &[String],
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let (project, manifest_ms, parse_ms) =
            load_project_without_marker_targets_for_root_selectors(workspace_root, root_selectors)?;
        let phase_started = Instant::now();
        let analyzer = analyzer::load_report_for_project_and_roots(
            workspace_root,
            analyzer_mode,
            &project,
            &[],
        )?;
        let analyzer_ms = elapsed_ms(phase_started);

        Ok(Self {
            workspace_root: workspace_root.to_path_buf(),
            project,
            analyzer,
            manifest_ms,
            parse_ms,
            analyzer_ms,
        })
    }

    pub fn load_root_selector_resolver_without_marker_targets(
        workspace_root: &Path,
        analyzer_mode: AnalyzerMode,
        root_selectors: &[String],
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let (project, manifest_ms, parse_ms) =
            load_project_without_marker_targets_for_root_selector_packages(
                workspace_root,
                root_selectors,
            )?;
        let phase_started = Instant::now();
        let analyzer = analyzer::load_report_for_project_and_roots(
            workspace_root,
            analyzer_mode,
            &project,
            &[],
        )?;
        let analyzer_ms = elapsed_ms(phase_started);

        Ok(Self {
            workspace_root: workspace_root.to_path_buf(),
            project,
            analyzer,
            manifest_ms,
            parse_ms,
            analyzer_ms,
        })
    }

    pub fn load_with_selected_roots(
        workspace_root: &Path,
        analyzer_mode: AnalyzerMode,
        selected_roots: &[RootId],
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Self::load_with_selected_roots_with_progress(
            workspace_root,
            analyzer_mode,
            selected_roots,
            None,
        )
    }

    pub fn load_with_selected_roots_with_progress(
        workspace_root: &Path,
        analyzer_mode: AnalyzerMode,
        selected_roots: &[RootId],
        progress: Option<&dyn Fn(GenerateSessionLoadProgress)>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let selected_packages = root_packages(selected_roots);
        let mut base_fields = load_progress_base_fields(analyzer_mode, selected_roots);
        base_fields.insert(
            "workspace_root".to_string(),
            workspace_root.display().to_string(),
        );
        base_fields.insert(
            "workspace_manifest".to_string(),
            workspace_manifest_path(workspace_root)
                .display()
                .to_string(),
        );
        if !selected_packages.is_empty() {
            base_fields.insert(
                "root_packages".to_string(),
                selected_packages
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(","),
            );
        }

        emit_load_progress(
            progress,
            "session_manifest",
            "started",
            "load source workspace manifests for selected roots",
            "manifest loading decides which package graph will be parsed before semantic analysis",
            base_fields.clone(),
        );
        let manifest_started = Instant::now();
        let workspace_result = if selected_roots.is_empty() {
            manifest::load_workspace(workspace_root)
        } else {
            manifest::load_workspace_without_marker_targets(workspace_root)
        };
        let (workspace, manifest_ms) = match workspace_result {
            Ok(workspace) => {
                let manifest_ms = elapsed_ms(manifest_started);
                let mut fields = base_fields.clone();
                fields.insert("manifest_ms".to_string(), manifest_ms.to_string());
                emit_load_progress(
                    progress,
                    "session_manifest",
                    "completed",
                    "load source workspace manifests for selected roots",
                    "manifest graph is ready for package-scoped parsing",
                    fields,
                );
                (workspace, manifest_ms)
            }
            Err(error) => {
                let mut fields = base_fields.clone();
                fields.insert(
                    "manifest_ms".to_string(),
                    elapsed_ms(manifest_started).to_string(),
                );
                emit_load_progress(
                    progress,
                    "session_manifest",
                    "failed",
                    "load source workspace manifests for selected roots",
                    error.to_string(),
                    fields,
                );
                return Err(error);
            }
        };

        let parse_scope = if selected_roots.is_empty() {
            "workspace"
        } else {
            "selected_root_package_closure"
        };
        let mut parse_start_fields = base_fields.clone();
        parse_start_fields.insert("parse_scope".to_string(), parse_scope.to_string());
        emit_load_progress(
            progress,
            "session_parse",
            "started",
            "parse source files for selected root dependency closure",
            "top-down slicing starts from the selected root packages and parses only their downstream package closure",
            parse_start_fields,
        );
        let parse_started = Instant::now();
        let project_result = if selected_roots.is_empty() {
            parse::parse_workspace(workspace)
        } else {
            parse::parse_workspace_package_closure(workspace, &selected_packages)
        };
        let project = match project_result {
            Ok(project) => {
                let parse_ms = elapsed_ms(parse_started);
                let mut fields = base_fields.clone();
                fields.insert("parse_scope".to_string(), parse_scope.to_string());
                fields.insert("parse_ms".to_string(), parse_ms.to_string());
                fields.insert("source_files".to_string(), project.files.len().to_string());
                fields.insert(
                    "callables".to_string(),
                    (project.functions.len() + project.methods.len()).to_string(),
                );
                fields.insert("items".to_string(), project.items.len().to_string());
                emit_load_progress(
                    progress,
                    "session_parse",
                    "completed",
                    "parse source files for selected root dependency closure",
                    "project syntax index is ready for semantic analysis",
                    fields,
                );
                (project, parse_ms)
            }
            Err(error) => {
                let mut fields = base_fields.clone();
                fields.insert("parse_scope".to_string(), parse_scope.to_string());
                fields.insert(
                    "parse_ms".to_string(),
                    elapsed_ms(parse_started).to_string(),
                );
                emit_load_progress(
                    progress,
                    "session_parse",
                    "failed",
                    "parse source files for selected root dependency closure",
                    error.to_string(),
                    fields,
                );
                return Err(error);
            }
        };
        let (project, parse_ms) = project;

        emit_load_progress(
            progress,
            "session_analyzer",
            "started",
            "load semantic analyzer for selected root dependency closure",
            "rust-analyzer semantics are loaded after the top-down syntax package closure is known",
            base_fields.clone(),
        );
        let phase_started = Instant::now();
        let analyzer_result = analyzer::load_report_for_project_and_roots(
            workspace_root,
            analyzer_mode,
            &project,
            selected_roots,
        );
        let analyzer_ms = elapsed_ms(phase_started);
        let analyzer = match analyzer_result {
            Ok(analyzer) => {
                let mut fields = base_fields;
                fields.insert("analyzer_ms".to_string(), analyzer_ms.to_string());
                fields.insert("engine".to_string(), analyzer.engine.clone());
                fields.insert("notes".to_string(), analyzer.notes.len().to_string());
                emit_load_progress(
                    progress,
                    "session_analyzer",
                    "completed",
                    "load semantic analyzer for selected root dependency closure",
                    "semantic analyzer report is ready for top-down reduction",
                    fields,
                );
                analyzer
            }
            Err(error) => {
                let mut fields = base_fields;
                fields.insert("analyzer_ms".to_string(), analyzer_ms.to_string());
                emit_load_progress(
                    progress,
                    "session_analyzer",
                    "failed",
                    "load semantic analyzer for selected root dependency closure",
                    error.to_string(),
                    fields,
                );
                return Err(error);
            }
        };

        Ok(Self {
            workspace_root: workspace_root.to_path_buf(),
            project,
            analyzer,
            manifest_ms,
            parse_ms,
            analyzer_ms,
        })
    }

    pub fn load_with_root_selectors(
        workspace_root: &Path,
        analyzer_mode: AnalyzerMode,
        root_selectors: &[String],
    ) -> Result<(Self, Vec<RootId>), Box<dyn std::error::Error>> {
        let (project, manifest_ms, parse_ms) = if root_selectors.is_empty() {
            load_project(workspace_root)?
        } else {
            load_project_without_marker_targets_for_root_selectors(workspace_root, root_selectors)?
        };
        let roots = resolve_root_selectors_in_project(&project, root_selectors)?;
        let phase_started = Instant::now();
        let analyzer = analyzer::load_report_for_project_and_roots(
            workspace_root,
            analyzer_mode,
            &project,
            &roots,
        )?;
        let analyzer_ms = elapsed_ms(phase_started);
        let session = Self {
            workspace_root: workspace_root.to_path_buf(),
            project,
            analyzer,
            manifest_ms,
            parse_ms,
            analyzer_ms,
        };
        Ok((session, roots))
    }

    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    pub fn analyzer(&self) -> &AnalyzerReport {
        &self.analyzer
    }

    pub fn manifest_ms(&self) -> u64 {
        self.manifest_ms
    }

    pub fn parse_ms(&self) -> u64 {
        self.parse_ms
    }

    pub fn analyzer_ms(&self) -> u64 {
        self.analyzer_ms
    }

    pub fn indexed_package_names(&self) -> Vec<String> {
        let mut packages = self
            .project
            .files
            .values()
            .map(|source| source.package.clone())
            .collect::<Vec<_>>();
        packages.sort();
        packages.dedup();
        packages
    }

    pub fn indexed_source_files(&self) -> usize {
        self.project.files.len()
    }

    pub fn indexed_callables(&self) -> usize {
        self.project.functions.len() + self.project.methods.len()
    }

    pub fn indexed_items(&self) -> usize {
        self.project.items.len()
    }

    pub fn selectable_roots(&self) -> Vec<RootId> {
        selectable_roots_for_project(&self.project)
    }

    pub fn resolve_root_selectors(
        &self,
        selectors: &[String],
    ) -> Result<Vec<RootId>, Box<dyn std::error::Error>> {
        resolve_root_selectors_in_project(&self.project, selectors)
    }

    pub fn resolve_root_selector(
        &self,
        selector: &str,
    ) -> Result<RootId, Box<dyn std::error::Error>> {
        resolve_root_selector_in_project(&self.project, selector)
    }

    pub fn generate(
        &self,
        output_root: PathBuf,
        selected_roots: &[RootId],
        feedback_diagnostics: &[feedback::CheckDiagnostic],
    ) -> Result<GenerateReport, Box<dyn std::error::Error>> {
        generate_loaded(
            &self.project,
            self.analyzer.clone(),
            GenerateLoadedOptions {
                output_root,
                manifest_ms: self.manifest_ms,
                parse_ms: self.parse_ms,
                analyzer_ms: self.analyzer_ms,
            },
            selected_roots,
            feedback_diagnostics,
        )
    }
}

fn load_project(workspace_root: &Path) -> Result<(Project, u64, u64), Box<dyn std::error::Error>> {
    let phase_started = Instant::now();
    let workspace = manifest::load_workspace(workspace_root)?;
    let manifest_ms = elapsed_ms(phase_started);

    let phase_started = Instant::now();
    let project = parse::parse_workspace(workspace)?;
    let parse_ms = elapsed_ms(phase_started);

    Ok((project, manifest_ms, parse_ms))
}

fn load_project_without_marker_targets(
    workspace_root: &Path,
) -> Result<(Project, u64, u64), Box<dyn std::error::Error>> {
    let phase_started = Instant::now();
    let workspace = manifest::load_workspace_without_marker_targets(workspace_root)?;
    let manifest_ms = elapsed_ms(phase_started);

    let phase_started = Instant::now();
    let project = parse::parse_workspace(workspace)?;
    let parse_ms = elapsed_ms(phase_started);

    Ok((project, manifest_ms, parse_ms))
}

fn load_project_without_marker_targets_for_root_selectors(
    workspace_root: &Path,
    root_selectors: &[String],
) -> Result<(Project, u64, u64), Box<dyn std::error::Error>> {
    let phase_started = Instant::now();
    let workspace = manifest::load_workspace_without_marker_targets(workspace_root)?;
    let manifest_ms = elapsed_ms(phase_started);

    let phase_started = Instant::now();
    let project = if let Some(packages) = root_selector_package_filter(root_selectors) {
        parse::parse_workspace_package_closure(workspace, &packages)?
    } else {
        parse::parse_workspace(workspace)?
    };
    let parse_ms = elapsed_ms(phase_started);

    Ok((project, manifest_ms, parse_ms))
}

fn load_project_without_marker_targets_for_root_selector_packages(
    workspace_root: &Path,
    root_selectors: &[String],
) -> Result<(Project, u64, u64), Box<dyn std::error::Error>> {
    let phase_started = Instant::now();
    let workspace = manifest::load_workspace_without_marker_targets(workspace_root)?;
    let manifest_ms = elapsed_ms(phase_started);

    let phase_started = Instant::now();
    let project = if let Some(packages) = root_selector_package_filter(root_selectors) {
        parse::parse_workspace_packages(workspace, &packages)?
    } else {
        parse::parse_workspace(workspace)?
    };
    let parse_ms = elapsed_ms(phase_started);

    Ok((project, manifest_ms, parse_ms))
}

fn root_selector_package_filter(root_selectors: &[String]) -> Option<BTreeSet<String>> {
    let mut packages = BTreeSet::new();
    for selector in root_selectors {
        let Some((package, _rest)) = selector.split_once("::") else {
            return None;
        };
        if package.is_empty() {
            return None;
        }
        packages.insert(package.to_string());
    }
    (!packages.is_empty()).then_some(packages)
}

fn root_packages(roots: &[RootId]) -> BTreeSet<String> {
    roots
        .iter()
        .map(|root| root.package().to_string())
        .collect()
}

fn load_progress_base_fields(
    analyzer_mode: AnalyzerMode,
    selected_roots: &[RootId],
) -> BTreeMap<String, String> {
    let mut fields = BTreeMap::new();
    fields.insert("analyzer".to_string(), analyzer_mode.as_str().to_string());
    fields.insert(
        "selected_roots".to_string(),
        selected_roots.len().to_string(),
    );
    if !selected_roots.is_empty() {
        fields.insert(
            "roots".to_string(),
            selected_roots
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(","),
        );
    }
    fields
}

fn workspace_manifest_path(workspace_root: &Path) -> PathBuf {
    if workspace_root.file_name() == Some(OsStr::new("Cargo.toml")) {
        workspace_root.to_path_buf()
    } else {
        workspace_root.join("Cargo.toml")
    }
}

fn emit_load_progress(
    progress: Option<&dyn Fn(GenerateSessionLoadProgress)>,
    event: &'static str,
    status: &'static str,
    decision: &'static str,
    reason: impl Into<String>,
    fields: BTreeMap<String, String>,
) {
    if let Some(progress) = progress {
        progress(GenerateSessionLoadProgress {
            event,
            status,
            decision,
            reason: reason.into(),
            fields,
        });
    }
}

fn resolve_root_selectors_in_project(
    project: &Project,
    selectors: &[String],
) -> Result<Vec<RootId>, Box<dyn std::error::Error>> {
    let mut roots = Vec::new();
    let mut seen = BTreeSet::new();
    for selector in selectors {
        let root = resolve_root_selector_in_project(project, selector)?;
        if seen.insert(root.clone()) {
            roots.push(root);
        }
    }
    Ok(roots)
}

fn resolve_root_selector_in_project(
    project: &Project,
    selector: &str,
) -> Result<RootId, Box<dyn std::error::Error>> {
    let selector = selector.trim();
    if selector.is_empty() {
        return Err("root selector must not be empty".into());
    }
    let mut matches = selectable_roots_for_project(project)
        .into_iter()
        .filter(|root| root_matches_selector(root, selector))
        .collect::<Vec<_>>();
    matches.sort();
    matches.dedup();
    match matches.as_slice() {
        [root] => Ok(root.clone()),
        [] => Err(format!(
            "root selector {selector:?} did not match any function, method, or item"
        )
        .into()),
        _ => {
            let preview = matches
                .iter()
                .take(12)
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            Err(format!(
                "root selector {selector:?} is ambiguous ({} matches): {preview}",
                matches.len()
            )
            .into())
        }
    }
}

fn selectable_roots_for_project(project: &Project) -> Vec<RootId> {
    let mut roots = project
        .functions
        .keys()
        .cloned()
        .map(RootId::Callable)
        .chain(project.methods.keys().cloned().map(RootId::Callable))
        .chain(project.items.keys().cloned().map(RootId::Item))
        .collect::<Vec<_>>();
    roots.sort();
    roots.dedup();
    roots
}

struct GenerateLoadedOptions {
    output_root: PathBuf,
    manifest_ms: u64,
    parse_ms: u64,
    analyzer_ms: u64,
}

fn generate_loaded(
    project: &Project,
    analyzer: AnalyzerReport,
    options: GenerateLoadedOptions,
    selected_roots: &[RootId],
    feedback_diagnostics: &[feedback::CheckDiagnostic],
) -> Result<GenerateReport, Box<dyn std::error::Error>> {
    let feedback_widened_roots = feedback_extra_roots(project, feedback_diagnostics);
    let mut extra_roots = selected_roots.to_vec();
    extra_roots.extend(feedback_widened_roots.iter().cloned());
    extra_roots.sort();
    extra_roots.dedup();

    let phase_started = Instant::now();
    let reduced = if analyzer.semantic_hints.is_empty() {
        reduce::reduce_with_extra_roots(project, &extra_roots)?
    } else {
        reduce::reduce_with_extra_roots_and_semantics(
            project,
            &extra_roots,
            &analyzer.semantic_hints,
        )?
    };
    let reduce_ms = elapsed_ms(phase_started);

    let pre_render_production =
        pre_render_production_readiness_report(&analyzer, project, &reduced);
    let slice_plan = SlicePlan::build(project, &reduced, &analyzer, &pre_render_production)?;
    let render_reduced = slice_plan.render_reduced;
    let usage_decisions = slice_plan.usage_decisions;

    let phase_started = Instant::now();
    let files_written = render::write_reduced_workspace(
        project,
        &render_reduced,
        &usage_decisions,
        &options.output_root,
    )?;
    let render_ms = elapsed_ms(phase_started);
    let timings = GenerateTimingReport {
        total_ms: options
            .manifest_ms
            .saturating_add(options.parse_ms)
            .saturating_add(options.analyzer_ms)
            .saturating_add(reduce_ms)
            .saturating_add(render_ms),
        analyzer_ms: options.analyzer_ms,
        manifest_ms: options.manifest_ms,
        parse_ms: options.parse_ms,
        reduce_ms,
        render_ms,
    };

    let mut packages = render_reduced.packages.iter().cloned().collect::<Vec<_>>();
    packages.sort();

    let mut reachable = render_reduced.reachable.iter().cloned().collect::<Vec<_>>();
    reachable.sort();

    let mut reachable_items = render_reduced
        .reachable_items
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    reachable_items.sort();
    let targets = target_report(project, &packages);
    let source_map = source_map_report(project, &render_reduced);
    let macro_surfaces = macro_surface_report(project, &render_reduced);
    let member_decisions =
        render::rendered_member_decision_index(project, &render_reduced, &usage_decisions);
    let public_reexport_proof = public_reexport_proof_report_with_retained_surface_items(
        &options.output_root,
        &usage_decisions,
        &member_decisions.retained_items,
    );
    let rendered_symbol_proof = rendered_symbol_proof_report_with_members(
        &options.output_root,
        &usage_decisions,
        &member_decisions,
    );
    let semantic_proof = semantic_usage_proof_report(
        project,
        &analyzer,
        &usage_decisions,
        Some(&rendered_symbol_proof),
    );
    let production = production_readiness_report(
        &analyzer,
        project,
        &render_reduced,
        &options.output_root,
        Some(&semantic_proof),
        Some(&rendered_symbol_proof),
        Some(&public_reexport_proof),
    );
    let usage = usage_classification_report(
        project,
        &reduced,
        &analyzer,
        &usage_decisions,
        &production,
        rendered_symbol_proof,
        public_reexport_proof,
    );

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
        macro_surfaces,
        usage,
        source_map,
        files_written,
        timings,
    })
}

#[cfg(test)]
fn usage_guarded_render_reduction(
    project: &Project,
    reduced: &ReducedProject,
    analyzer: &AnalyzerReport,
    pre_render_production: &ProductionReadinessReport,
) -> Result<(ReducedProject, UsageDecisionIndex), Box<dyn std::error::Error>> {
    let slice_plan = SlicePlan::build(project, reduced, analyzer, pre_render_production)?;
    Ok((slice_plan.render_reduced, slice_plan.usage_decisions))
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

fn root_matches_selector(root: &RootId, selector: &str) -> bool {
    root_selector_keys(root).iter().any(|key| {
        key == selector
            || key
                .strip_prefix("crate::")
                .is_some_and(|without_crate| without_crate == selector)
            || key.ends_with(&format!("::{selector}"))
    })
}

fn root_selector_keys(root: &RootId) -> Vec<String> {
    let mut keys = vec![root.to_string()];
    match root {
        RootId::Callable(CallableId::Free {
            package,
            module_path,
            name,
        }) => {
            keys.push(path_key(package, module_path, name));
            keys.push(
                path_key("", module_path, name)
                    .trim_start_matches("::")
                    .to_string(),
            );
            keys.push(name.clone());
        }
        RootId::Callable(CallableId::Method {
            package,
            type_path,
            method,
            ..
        }) => {
            let type_method = format!("{}::{method}", type_path.join("::"));
            keys.push(format!("{package}::{type_method}"));
            keys.push(type_method);
            keys.push(method.clone());
        }
        RootId::Item(item) => {
            let path = path_key(&item.package, &item.module_path, &item.name);
            keys.push(path.clone());
            keys.push(format!("{path}({})", item_kind_selector_name(item.kind)));
            keys.push(format!("{path}({:?})", item.kind));
            keys.push(
                path_key("", &item.module_path, &item.name)
                    .trim_start_matches("::")
                    .to_string(),
            );
            keys.push(item.name.clone());
        }
    }
    keys.sort();
    keys.dedup();
    keys
}

fn path_key(package: &str, module_path: &[String], name: &str) -> String {
    let mut segments = Vec::new();
    if !package.is_empty() {
        segments.push(package.to_string());
    }
    segments.extend(module_path.iter().cloned());
    segments.push(name.to_string());
    segments.join("::")
}

fn item_kind_selector_name(kind: model::ItemKind) -> &'static str {
    match kind {
        model::ItemKind::Struct => "struct",
        model::ItemKind::Enum => "enum",
        model::ItemKind::Union => "union",
        model::ItemKind::Type => "type",
        model::ItemKind::Trait => "trait",
        model::ItemKind::Mod => "mod",
        model::ItemKind::Const => "const",
        model::ItemKind::Static => "static",
        model::ItemKind::Macro => "macro",
    }
}

const FEEDBACK_WIDENING_ROOT_MATCH_LIMIT: usize = 24;
const FEEDBACK_WIDENING_RESOLUTION_ENTRY_LIMIT: usize = 128;

fn feedback_extra_roots(
    project: &Project,
    diagnostics: &[feedback::CheckDiagnostic],
) -> Vec<RootId> {
    feedback_extra_roots_with_report(project, diagnostics).0
}

fn feedback_extra_roots_with_report(
    project: &Project,
    diagnostics: &[feedback::CheckDiagnostic],
) -> (Vec<RootId>, FeedbackRootResolutionReport) {
    let mut roots = Vec::new();
    let mut seen = BTreeSet::new();
    let mut report = FeedbackRootResolutionReport {
        manifest_ms: 0,
        parse_ms: 0,
        diagnostics: diagnostics.len(),
        error_diagnostics: 0,
        skipped_non_error_diagnostics: 0,
        skipped_missing_code_diagnostics: 0,
        candidate_symbols: 0,
        skipped_missing_symbol: 0,
        skipped_no_match: 0,
        skipped_too_many_matches: 0,
        skipped_marked_roots: 0,
        matched_roots: Vec::new(),
        entries: Vec::new(),
        entries_truncated: false,
    };
    for diagnostic in diagnostics {
        if diagnostic.level != "error" {
            report.skipped_non_error_diagnostics += 1;
            continue;
        }
        report.error_diagnostics += 1;
        let Some(code) = diagnostic.code.as_deref() else {
            report.skipped_missing_code_diagnostics += 1;
            continue;
        };
        let symbols = diagnostic_symbols(diagnostic);
        let package_hint = diagnostic_scoped_package_hint(project, diagnostic, &symbols);
        let glob_import_context =
            diagnostic_glob_import_context(project, diagnostic, package_hint.as_deref());
        let diagnostic_candidates =
            feedback_diagnostic_root_candidates(project, diagnostic, package_hint.as_deref());
        if !diagnostic_candidates.is_empty() {
            let (retained_roots, skipped_marked_roots) =
                retain_feedback_roots(project, diagnostic_candidates, &mut seen, &mut roots);
            report.skipped_marked_roots += skipped_marked_roots;
            push_feedback_resolution_entry(
                &mut report,
                FeedbackRootResolutionEntry {
                    code: code.to_string(),
                    symbol: "<diagnostic>".to_string(),
                    name: None,
                    package_hint: package_hint.clone(),
                    diagnostic_file: None,
                    glob_imports: Vec::new(),
                    glob_import_module_roots: Vec::new(),
                    glob_import_provider_internal_globs: Vec::new(),
                    matches: retained_roots.len() + skipped_marked_roots,
                    retained_roots: retained_roots.len(),
                    skipped_marked_roots,
                    action: if retained_roots.is_empty() {
                        "matched only already-selected roots".to_string()
                    } else {
                        "widen to diagnostic-specific root".to_string()
                    },
                    roots: retained_roots,
                },
            );
        }
        if symbols.is_empty() {
            report.skipped_missing_symbol += 1;
            continue;
        }
        for symbol in symbols {
            let Some(name) = symbol_leaf_name(&symbol) else {
                report.skipped_missing_symbol += 1;
                continue;
            };
            let name = name.to_string();
            report.candidate_symbols += 1;
            let candidates =
                feedback_root_candidates(project, code, package_hint.as_deref(), &name);
            if candidates.is_empty() {
                report.skipped_no_match += 1;
                let action = if glob_import_context.glob_import_module_roots.is_empty() {
                    "no matching project-local root"
                } else {
                    "no matching project-local root; inspect glob import provider module roots"
                };
                push_feedback_resolution_entry(
                    &mut report,
                    FeedbackRootResolutionEntry {
                        code: code.to_string(),
                        symbol,
                        name: Some(name),
                        package_hint: package_hint.clone(),
                        diagnostic_file: glob_import_context.diagnostic_file.clone(),
                        glob_imports: glob_import_context.glob_imports.clone(),
                        glob_import_module_roots: glob_import_context
                            .glob_import_module_roots
                            .clone(),
                        glob_import_provider_internal_globs: glob_import_context
                            .glob_import_provider_internal_globs
                            .clone(),
                        matches: 0,
                        retained_roots: 0,
                        skipped_marked_roots: 0,
                        action: action.to_string(),
                        roots: Vec::new(),
                    },
                );
                continue;
            }
            if candidates.len() > FEEDBACK_WIDENING_ROOT_MATCH_LIMIT {
                report.skipped_too_many_matches += 1;
                push_feedback_resolution_entry(
                    &mut report,
                    FeedbackRootResolutionEntry {
                        code: code.to_string(),
                        symbol,
                        name: Some(name),
                        package_hint: package_hint.clone(),
                        diagnostic_file: None,
                        glob_imports: Vec::new(),
                        glob_import_module_roots: Vec::new(),
                        glob_import_provider_internal_globs: Vec::new(),
                        matches: candidates.len(),
                        retained_roots: 0,
                        skipped_marked_roots: 0,
                        action: "too many matches; fail closed instead of over-retaining"
                            .to_string(),
                        roots: candidates.iter().take(8).map(ToString::to_string).collect(),
                    },
                );
                continue;
            }
            let matches = candidates.len();
            let (retained_roots, skipped_marked_roots) =
                retain_feedback_roots(project, candidates, &mut seen, &mut roots);
            report.skipped_marked_roots += skipped_marked_roots;
            push_feedback_resolution_entry(
                &mut report,
                FeedbackRootResolutionEntry {
                    code: code.to_string(),
                    symbol,
                    name: Some(name),
                    package_hint: package_hint.clone(),
                    diagnostic_file: None,
                    glob_imports: Vec::new(),
                    glob_import_module_roots: Vec::new(),
                    glob_import_provider_internal_globs: Vec::new(),
                    matches,
                    retained_roots: retained_roots.len(),
                    skipped_marked_roots,
                    action: if retained_roots.is_empty() {
                        "matched only already-selected roots".to_string()
                    } else {
                        "widen to matched project-local roots".to_string()
                    },
                    roots: retained_roots,
                },
            );
        }
    }
    roots.sort();
    report.matched_roots = roots.iter().map(ToString::to_string).collect();
    (roots, report)
}

fn retain_feedback_roots(
    project: &Project,
    candidates: Vec<RootId>,
    seen: &mut BTreeSet<RootId>,
    roots: &mut Vec<RootId>,
) -> (Vec<String>, usize) {
    let mut retained = Vec::new();
    let mut skipped_marked_roots = 0;
    for root in candidates {
        if root_is_marked(project, &root) {
            skipped_marked_roots += 1;
            continue;
        }
        if seen.insert(root.clone()) {
            retained.push(root.to_string());
            roots.push(root);
        }
    }
    (retained, skipped_marked_roots)
}

fn push_feedback_resolution_entry(
    report: &mut FeedbackRootResolutionReport,
    entry: FeedbackRootResolutionEntry,
) {
    if report.entries.len() < FEEDBACK_WIDENING_RESOLUTION_ENTRY_LIMIT {
        report.entries.push(entry);
    } else {
        report.entries_truncated = true;
    }
}

#[derive(Clone, Default)]
struct FeedbackGlobImportContext {
    diagnostic_file: Option<String>,
    glob_imports: Vec<String>,
    glob_import_module_roots: Vec<String>,
    glob_import_provider_internal_globs: Vec<String>,
}

fn diagnostic_glob_import_context(
    project: &Project,
    diagnostic: &feedback::CheckDiagnostic,
    package_hint: Option<&str>,
) -> FeedbackGlobImportContext {
    let diagnostic_file = diagnostic_primary_file_name(diagnostic).map(ToString::to_string);
    let Some(source) = source_file_for_diagnostic(project, diagnostic, package_hint) else {
        return FeedbackGlobImportContext {
            diagnostic_file,
            ..FeedbackGlobImportContext::default()
        };
    };
    let mut glob_paths = Vec::new();
    for item in &source.syntax.items {
        let Item::Use(item_use) = item else {
            continue;
        };
        collect_glob_import_paths(&item_use.tree, Vec::new(), &mut glob_paths);
    }
    glob_paths.sort();
    glob_paths.dedup();

    let mut glob_imports = Vec::new();
    let mut glob_import_module_roots = Vec::new();
    let mut glob_import_provider_internal_globs = Vec::new();
    for segments in glob_paths {
        glob_imports.push(format!("{}::*", segments.join("::")));
        glob_import_module_roots.extend(
            glob_import_module_root(project, source, &segments).map(|root| root.to_string()),
        );
        glob_import_provider_internal_globs.extend(provider_internal_globs_for_glob_import(
            project, source, &segments,
        ));
    }
    glob_imports.sort();
    glob_imports.dedup();
    glob_import_module_roots.sort();
    glob_import_module_roots.dedup();
    glob_import_provider_internal_globs.sort();
    glob_import_provider_internal_globs.dedup();

    FeedbackGlobImportContext {
        diagnostic_file,
        glob_imports,
        glob_import_module_roots,
        glob_import_provider_internal_globs,
    }
}

fn diagnostic_primary_file_name(diagnostic: &feedback::CheckDiagnostic) -> Option<&str> {
    diagnostic
        .spans
        .iter()
        .find(|span| span.is_primary)
        .or_else(|| diagnostic.spans.first())
        .map(|span| span.file_name.as_str())
}

fn source_file_for_diagnostic<'project>(
    project: &'project Project,
    diagnostic: &feedback::CheckDiagnostic,
    package_hint: Option<&str>,
) -> Option<&'project model::SourceFile> {
    let file_name = diagnostic_primary_file_name(diagnostic)?;
    let diagnostic_path = Path::new(file_name);
    project.files.values().find(|source| {
        package_hint.is_none_or(|package| source.package == package)
            && (source.path == diagnostic_path || source.path.ends_with(diagnostic_path))
    })
}

fn collect_glob_import_paths(
    tree: &UseTree,
    mut prefix: Vec<String>,
    paths: &mut Vec<Vec<String>>,
) {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            collect_glob_import_paths(&path.tree, prefix, paths);
        }
        UseTree::Group(group) => {
            for nested in &group.items {
                collect_glob_import_paths(nested, prefix.clone(), paths);
            }
        }
        UseTree::Glob(_) => {
            if !prefix.is_empty() {
                paths.push(prefix);
            }
        }
        UseTree::Name(_) | UseTree::Rename(_) => {}
    }
}

fn glob_import_module_root(
    project: &Project,
    source: &model::SourceFile,
    segments: &[String],
) -> Option<RootId> {
    let (package, module_segments) =
        resolve_glob_import_module_segments(project, source, segments)?;
    let (name, module_path) = module_segments.split_last()?;
    let item = ItemId {
        package,
        module_path: module_path.to_vec(),
        name: name.clone(),
        kind: model::ItemKind::Mod,
    };
    project
        .items
        .contains_key(&item)
        .then_some(RootId::Item(item))
}

fn provider_internal_globs_for_glob_import(
    project: &Project,
    source: &model::SourceFile,
    segments: &[String],
) -> Vec<String> {
    let Some((package, module_segments)) =
        resolve_glob_import_module_segments(project, source, segments)
    else {
        return Vec::new();
    };
    let Some(provider_source) = project
        .files
        .values()
        .find(|candidate| candidate.package == package && candidate.module_path == module_segments)
    else {
        return Vec::new();
    };
    let mut provider_globs = Vec::new();
    for item in &provider_source.syntax.items {
        let Item::Use(item_use) = item else {
            continue;
        };
        if !matches!(item_use.vis, syn::Visibility::Inherited) {
            continue;
        }
        let mut glob_paths = Vec::new();
        collect_glob_import_paths(&item_use.tree, Vec::new(), &mut glob_paths);
        for glob_path in glob_paths {
            if let Some((glob_package, glob_module_segments)) =
                resolve_glob_import_module_segments(project, provider_source, &glob_path)
            {
                provider_globs.push(format_package_module_glob(
                    &glob_package,
                    &glob_module_segments,
                ));
            } else {
                provider_globs.push(format!("{}::*", glob_path.join("::")));
            }
        }
    }
    provider_globs
}

fn format_package_module_glob(package: &str, module_segments: &[String]) -> String {
    if module_segments.is_empty() {
        format!("{package}::*")
    } else {
        format!("{package}::{}::*", module_segments.join("::"))
    }
}

fn resolve_glob_import_module_segments(
    project: &Project,
    source: &model::SourceFile,
    segments: &[String],
) -> Option<(String, Vec<String>)> {
    let (first, rest) = segments.split_first()?;
    match first.as_str() {
        "crate" => Some((source.package.clone(), rest.to_vec())),
        "self" => {
            let mut module_path = source.module_path.clone();
            module_path.extend(rest.iter().cloned());
            Some((source.package.clone(), module_path))
        }
        "super" => {
            let mut module_path = source.module_path.clone();
            module_path.pop();
            module_path.extend(rest.iter().cloned());
            Some((source.package.clone(), module_path))
        }
        _ => {
            if let Some(package) = feedback_symbol_package_hint(project, first) {
                Some((package, rest.to_vec()))
            } else {
                let mut module_path = source.module_path.clone();
                module_path.extend(segments.iter().cloned());
                Some((source.package.clone(), module_path))
            }
        }
    }
}

fn feedback_diagnostic_root_candidates(
    project: &Project,
    diagnostic: &feedback::CheckDiagnostic,
    package_hint: Option<&str>,
) -> Vec<RootId> {
    match diagnostic.code.as_deref() {
        Some("E0277") => {
            conversion_impl_candidates_from_diagnostic(project, package_hint, diagnostic)
        }
        _ => Vec::new(),
    }
}

fn feedback_root_candidates(
    project: &Project,
    code: &str,
    package_hint: Option<&str>,
    name: &str,
) -> Vec<RootId> {
    let scoped = feedback_root_candidates_in_scope(project, code, package_hint, name);
    if package_hint.is_none() || !scoped.is_empty() {
        return scoped;
    }
    feedback_root_candidates_in_scope(project, code, None, name)
}

fn diagnostic_scoped_package_hint(
    project: &Project,
    diagnostic: &feedback::CheckDiagnostic,
    symbols: &[String],
) -> Option<String> {
    symbols
        .iter()
        .find_map(|symbol| feedback_symbol_package_hint(project, symbol))
        .or_else(|| diagnostic_package_hint(diagnostic))
}

fn feedback_symbol_package_hint(project: &Project, symbol: &str) -> Option<String> {
    let crate_segment = symbol.split("::").next()?.trim();
    if matches!(crate_segment, "" | "crate" | "self" | "super")
        || !crate_segment
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_alphabetic() || character == '_')
    {
        return None;
    }
    let crate_package = crate_segment.replace('_', "-");
    project_has_package(project, &crate_package)
        .then_some(crate_package)
        .or_else(|| project_has_package(project, crate_segment).then(|| crate_segment.to_string()))
}

fn project_has_package(project: &Project, package: &str) -> bool {
    project
        .files
        .values()
        .any(|source| source.package == package)
        || project
            .functions
            .keys()
            .any(|callable| callable.package() == package)
        || project.items.keys().any(|item| item.package() == package)
}

fn feedback_root_candidates_in_scope(
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
                matches!(
                    item.kind,
                    model::ItemKind::Struct
                        | model::ItemKind::Enum
                        | model::ItemKind::Union
                        | model::ItemKind::Type
                        | model::ItemKind::Trait
                        | model::ItemKind::Mod
                        | model::ItemKind::Const
                        | model::ItemKind::Static
                )
            }));
        }
        "E0432" | "E0433" => {
            roots.extend(free_function_name_candidates(project, package_hint, name));
            roots.extend(item_name_candidates(project, package_hint, name, |_| true));
        }
        "E0599" => {
            roots.extend(method_name_candidates(project, package_hint, name));
        }
        "E0560" | "E0609" => {
            roots.extend(item_name_candidates(project, package_hint, name, |item| {
                item.kind == model::ItemKind::Struct
            }));
        }
        _ => {}
    }
    roots.sort();
    roots.dedup();
    roots
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct ConversionTraitBound {
    target_type: String,
    trait_name: String,
    input_type: Option<String>,
}

fn conversion_impl_candidates_from_diagnostic(
    project: &Project,
    package_hint: Option<&str>,
    diagnostic: &feedback::CheckDiagnostic,
) -> Vec<RootId> {
    let mut bounds = conversion_trait_bounds_from_text(&diagnostic.message);
    if let Some(rendered) = &diagnostic.rendered {
        bounds.extend(conversion_trait_bounds_from_text(rendered));
    }
    bounds.sort_by(|left, right| {
        (&left.target_type, &left.trait_name, &left.input_type).cmp(&(
            &right.target_type,
            &right.trait_name,
            &right.input_type,
        ))
    });
    bounds.dedup();

    let mut roots = Vec::new();
    for bound in bounds {
        let Some(method_name) = conversion_trait_method_name(&bound.trait_name) else {
            continue;
        };
        roots.extend(project.methods.keys().filter_map(|callable| {
            let CallableId::Method {
                type_path,
                trait_path: Some(trait_path),
                trait_input_type_paths,
                method,
                ..
            } = callable
            else {
                return None;
            };
            if !callable.package_matches(package_hint)
                || method != method_name
                || trait_path
                    .last()
                    .is_none_or(|candidate| candidate != &bound.trait_name)
                || !path_leaf_matches(type_path, &bound.target_type)
            {
                return None;
            }
            if let Some(input_type) = &bound.input_type {
                if !trait_input_type_paths
                    .iter()
                    .any(|path| path_leaf_matches(path, input_type))
                {
                    return None;
                }
            }
            Some(RootId::Callable(callable.clone()))
        }));
    }
    roots.sort();
    roots.dedup();
    roots
}

fn conversion_trait_bounds_from_text(text: &str) -> Vec<ConversionTraitBound> {
    backticked_symbols(text)
        .into_iter()
        .filter_map(|symbol| {
            let (target_type, trait_bound) = symbol.split_once(": ")?;
            let trait_name = trait_bound.split_once('<')?.0.trim();
            if !matches!(trait_name, "From" | "Into" | "TryFrom" | "TryInto") {
                return None;
            }
            Some(ConversionTraitBound {
                target_type: symbol_leaf_name(target_type)?.to_string(),
                trait_name: trait_name.to_string(),
                input_type: generic_argument_leaf(trait_bound),
            })
        })
        .collect()
}

fn generic_argument_leaf(trait_bound: &str) -> Option<String> {
    let (_, tail) = trait_bound.split_once('<')?;
    let argument = tail.rsplit_once('>').map_or(tail, |(argument, _)| argument);
    symbol_leaf_name(argument).map(ToString::to_string)
}

fn conversion_trait_method_name(trait_name: &str) -> Option<&'static str> {
    match trait_name {
        "From" => Some("from"),
        "Into" => Some("into"),
        "TryFrom" => Some("try_from"),
        "TryInto" => Some("try_into"),
        _ => None,
    }
}

fn path_leaf_matches(path: &[String], expected_leaf: &str) -> bool {
    path.last().is_some_and(|leaf| leaf == expected_leaf)
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
    if matches!(diagnostic.code.as_deref(), Some("E0560" | "E0609")) {
        if let Some(symbol) = diagnostic_field_surface_owner_symbol(diagnostic) {
            return vec![symbol];
        }
    }
    let mut symbols = backticked_symbols(&diagnostic.message);
    if let Some(rendered) = &diagnostic.rendered {
        symbols.extend(backticked_symbols(rendered));
    }
    symbols.sort();
    symbols.dedup();
    symbols
}

fn diagnostic_field_surface_owner_symbol(diagnostic: &feedback::CheckDiagnostic) -> Option<String> {
    field_surface_owner_symbol_from_text(&diagnostic.message).or_else(|| {
        diagnostic
            .rendered
            .as_deref()
            .and_then(field_surface_owner_symbol_from_text)
    })
}

fn field_surface_owner_symbol_from_text(text: &str) -> Option<String> {
    text.split_once(" on type ")
        .and_then(|(_, tail)| backticked_symbols(tail).into_iter().next())
        .or_else(|| {
            text.split_once("struct ")
                .and_then(|(_, tail)| backticked_symbols(tail).into_iter().next())
        })
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
        .or_else(|| {
            diagnostic
                .target
                .as_ref()
                .map(|target| target.name.replace('_', "-"))
        })
}

fn package_name_from_diagnostic_package_id(package_id: &str) -> Option<String> {
    if let Some(path_package) = path_package_name_from_diagnostic_package_id(package_id) {
        return Some(path_package);
    }
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

fn path_package_name_from_diagnostic_package_id(package_id: &str) -> Option<String> {
    let without_fragment = package_id
        .split_once('#')
        .map_or(package_id, |(path, _)| path);
    let path_marker = "path+file://";
    let path_start = without_fragment.find(path_marker)?;
    without_fragment[path_start + path_marker.len()..]
        .trim_end_matches(['/', ')', ' ', '\t'])
        .rsplit('/')
        .find(|segment| !segment.is_empty())
        .map(str::to_string)
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

fn macro_surface_report(project: &Project, reduced: &ReducedProject) -> MacroSurfaceReport {
    let mut surfaces = syntactic_hazard_counts(project, reduced).macro_surfaces;
    surfaces.sort_by(|left, right| {
        (
            &left.package,
            &left.module_path,
            &left.owner,
            &left.start_line,
            &left.kind,
            &left.path,
        )
            .cmp(&(
                &right.package,
                &right.module_path,
                &right.owner,
                &right.start_line,
                &right.kind,
                &right.path,
            ))
    });
    let mut summary = MacroSurfaceSummary {
        total: surfaces.len(),
        macro_blocked: surfaces
            .iter()
            .filter(|surface| surface.category == "macro_blocked")
            .count(),
        ..MacroSurfaceSummary::default()
    };
    for surface in &surfaces {
        match surface.kind.as_str() {
            "derive_macro" => summary.derive_macros += 1,
            "attribute_macro" => summary.attribute_macros += 1,
            "helper_attribute" => summary.helper_attributes += 1,
            "macro_invocation" => summary.macro_invocations += 1,
            _ => {}
        }
    }
    MacroSurfaceReport { summary, surfaces }
}

#[derive(Default)]
struct DeletionBlockerScope {
    global: bool,
    idents: BTreeSet<String>,
    scoped: Vec<ScopedDeletionBlocker>,
}

#[derive(Default)]
struct ScopedDeletionBlocker {
    package: Option<String>,
    module_path: Option<Vec<String>>,
    idents: BTreeSet<String>,
}

impl ScopedDeletionBlocker {
    fn covers_callable(&self, project: &Project, callable: &CallableId) -> bool {
        if !self.covers_package(callable.package()) {
            return false;
        }
        if let Some(module_path) = &self.module_path {
            let callable_module = callable_module_path(project, callable);
            if callable_module.as_deref() != Some(module_path.as_slice()) {
                return false;
            }
        }
        callable_idents(callable)
            .iter()
            .any(|ident| self.idents.contains(ident))
    }

    fn covers_item(&self, item: &ItemId) -> bool {
        if !self.covers_package(&item.package) {
            return false;
        }
        if let Some(module_path) = &self.module_path {
            if &item.module_path != module_path {
                return false;
            }
        }
        item_idents(item)
            .iter()
            .any(|ident| self.idents.contains(ident))
    }

    fn covers_package(&self, package: &str) -> bool {
        self.package
            .as_ref()
            .is_none_or(|blocked_package| blocked_package == package)
    }
}

impl DeletionBlockerScope {
    fn from_report(report: &ProductionReadinessReport) -> Self {
        let mut scope = Self::default();
        for hazard in &report.hazards {
            if !hazard_blocks_unused_pruning(&hazard.code) {
                continue;
            }
            if hazard.details.is_empty() {
                scope.global = true;
                continue;
            }
            for detail in &hazard.details {
                if !detail.blocked_idents.is_empty() {
                    if hazard_uses_scoped_detail_blockers(&hazard.code) {
                        let module_path = detail
                            .module_path
                            .as_deref()
                            .map(module_path_from_report_string)
                            .or_else(|| detail.package.as_ref().map(|_| Vec::new()));
                        scope.scoped.push(ScopedDeletionBlocker {
                            package: detail.package.clone(),
                            module_path,
                            idents: detail.blocked_idents.iter().cloned().collect(),
                        });
                    } else {
                        scope.idents.extend(detail.blocked_idents.iter().cloned());
                    }
                    continue;
                }
                if hazard_uses_precise_detail_blockers(&hazard.code) {
                    continue;
                }
                collect_text_idents(&detail.subject, &mut scope.idents);
                if let Some(cfg) = &detail.cfg {
                    collect_text_idents(cfg, &mut scope.idents);
                }
            }
        }
        scope
    }

    fn covers_callable(&self, project: &Project, callable: &CallableId) -> bool {
        if self.global {
            return true;
        }
        callable_idents(callable)
            .iter()
            .any(|ident| self.idents.contains(ident))
            || self
                .scoped
                .iter()
                .any(|scope| scope.covers_callable(project, callable))
    }

    fn covers_item(&self, item: &ItemId) -> bool {
        if self.global {
            return true;
        }
        item_idents(item)
            .iter()
            .any(|ident| self.idents.contains(ident))
            || self.scoped.iter().any(|scope| scope.covers_item(item))
    }
}

fn callable_module_path(project: &Project, callable: &CallableId) -> Option<Vec<String>> {
    match callable {
        CallableId::Free { module_path, .. } => Some(module_path.clone()),
        CallableId::Method { .. } => project
            .methods
            .get(callable)
            .map(|record| record.module_path.clone()),
    }
}

fn module_path_from_report_string(value: &str) -> Vec<String> {
    value
        .split("::")
        .filter(|segment| !segment.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn callable_idents(callable: &CallableId) -> BTreeSet<String> {
    match callable {
        CallableId::Free { name, .. } => std::iter::once(name.clone()).collect(),
        CallableId::Method { method, .. } => std::iter::once(method.clone()).collect(),
    }
}

fn item_idents(item: &ItemId) -> BTreeSet<String> {
    std::iter::once(item.name.clone()).collect()
}

fn collect_text_idents(text: &str, idents: &mut BTreeSet<String>) {
    let mut current = String::new();
    for ch in text.chars() {
        if ch == '_' || ch.is_ascii_alphanumeric() {
            current.push(ch);
            continue;
        }
        push_text_ident(&mut current, idents);
    }
    push_text_ident(&mut current, idents);
}

fn push_text_ident(current: &mut String, idents: &mut BTreeSet<String>) {
    if current
        .chars()
        .next()
        .is_some_and(|ch| ch == '_' || ch.is_ascii_alphabetic())
        && !rust_keyword_or_common_macro_word(current)
    {
        idents.insert(std::mem::take(current));
    } else {
        current.clear();
    }
}

fn rust_keyword_or_common_macro_word(ident: &str) -> bool {
    matches!(
        ident,
        "as" | "async"
            | "await"
            | "break"
            | "const"
            | "continue"
            | "crate"
            | "dyn"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "macro"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
    )
}

fn hazard_blocks_unused_pruning(code: &str) -> bool {
    matches!(
        code,
        "source_include_macros"
            | "scoped_source_include_macros"
            | "out_dir_source_include_macros"
            | "custom_attribute_macros"
            | "custom_derive_macros"
            | "custom_macro_invocations"
            | "function_pointer_surfaces"
            | "trait_object_surfaces"
            | "dynamic_callback_boundaries"
    )
}

fn hazard_uses_precise_detail_blockers(code: &str) -> bool {
    matches!(
        code,
        "scoped_source_include_macros"
            | "custom_attribute_macros"
            | "custom_derive_macros"
            | "custom_macro_invocations"
            | "function_pointer_surfaces"
            | "trait_object_surfaces"
            | "dynamic_callback_boundaries"
    )
}

fn hazard_uses_scoped_detail_blockers(code: &str) -> bool {
    matches!(
        code,
        "custom_macro_invocations" | "syntactic_method_fallback_cap"
    )
}

fn deletion_blocked_extra_roots(
    project: &Project,
    reduced: &ReducedProject,
    production: &ProductionReadinessReport,
) -> Vec<RootId> {
    let scope = DeletionBlockerScope::from_report(production);
    if !scope.global && scope.idents.is_empty() && scope.scoped.is_empty() {
        return Vec::new();
    }

    let mut roots = Vec::new();
    roots.extend(project.functions.iter().filter_map(|(id, record)| {
        if reduced.reachable.contains(id)
            || !reduced.packages.contains(id.package())
            || callable_record_is_test(&record.item.attrs)
            || !scope.covers_callable(project, id)
        {
            return None;
        }
        Some(RootId::Callable(id.clone()))
    }));
    roots.extend(project.methods.iter().filter_map(|(id, record)| {
        if reduced.reachable.contains(id)
            || !reduced.packages.contains(id.package())
            || callable_record_is_test(&record.item.attrs)
            || !scope.covers_callable(project, id)
        {
            return None;
        }
        Some(RootId::Callable(id.clone()))
    }));
    roots.extend(project.items.iter().filter_map(|(id, record)| {
        if reduced.reachable_items.contains(id)
            || !reduced.packages.contains(&id.package)
            || id.kind == model::ItemKind::Mod
            || item_record_is_test(&record.item)
            || !scope.covers_item(id)
        {
            return None;
        }
        Some(RootId::Item(id.clone()))
    }));
    roots
}

fn semantic_usage_unknown_extra_roots(
    project: &Project,
    reduced: &ReducedProject,
    analyzer: &AnalyzerReport,
) -> Vec<RootId> {
    let Some(usage) = &analyzer.semantic_usage else {
        return Vec::new();
    };

    let mut roots = Vec::new();
    roots.extend(project.functions.iter().filter_map(|(id, record)| {
        if reduced.reachable.contains(id)
            || !reduced.packages.contains(id.package())
            || callable_record_is_test(&record.item.attrs)
        {
            return None;
        }
        semantic_usage_blocks_callable_pruning(id, usage, reduced)
            .then(|| RootId::Callable(id.clone()))
    }));
    roots.extend(project.methods.iter().filter_map(|(id, record)| {
        if reduced.reachable.contains(id)
            || !reduced.packages.contains(id.package())
            || callable_record_is_test(&record.item.attrs)
        {
            return None;
        }
        semantic_usage_blocks_callable_pruning(id, usage, reduced)
            .then(|| RootId::Callable(id.clone()))
    }));
    roots.extend(project.items.iter().filter_map(|(id, record)| {
        if reduced.reachable_items.contains(id)
            || !reduced.packages.contains(&id.package)
            || id.kind == model::ItemKind::Mod
            || item_record_is_test(&record.item)
        {
            return None;
        }
        semantic_usage_blocks_item_pruning(id, usage, reduced).then(|| RootId::Item(id.clone()))
    }));
    roots
}

fn semantic_usage_blocks_callable_pruning(
    callable: &CallableId,
    usage: &SemanticUsageReport,
    retained: &ReducedProject,
) -> bool {
    usage.callable_reference_query_failed(callable)
        || usage.callable_has_retained_reference(
            callable,
            &retained.reachable,
            &retained.reachable_items,
        )
}

fn semantic_usage_blocks_item_pruning(
    item: &ItemId,
    usage: &SemanticUsageReport,
    retained: &ReducedProject,
) -> bool {
    usage.item_reference_query_failed(item)
        || usage.item_has_retained_reference(item, &retained.reachable, &retained.reachable_items)
}

fn callable_record_is_test(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attribute| {
        reduce::is_cfg_test_attr(attribute) || reduce::is_test_attr(attribute.path())
    })
}

fn item_record_is_test(item: &Item) -> bool {
    callable_record_is_test(item_attrs(item))
}

fn usage_classification_report(
    project: &model::Project,
    reduced: &model::ReducedProject,
    analyzer: &AnalyzerReport,
    decisions: &UsageDecisionIndex,
    production: &ProductionReadinessReport,
    rendered_symbols: RenderedSymbolProofReport,
    public_reexports: PublicReexportProofReport,
) -> UsageClassificationReport {
    let mut used_callables = decisions.used_callables();
    used_callables.sort();

    let mut blocked_by_unknown_callables = decisions.blocked_by_unknown_callables();
    blocked_by_unknown_callables.sort();

    let mut prunable_callables = decisions.prunable_callables();
    prunable_callables.sort();

    let mut unused_candidate_callables = blocked_by_unknown_callables
        .iter()
        .cloned()
        .chain(prunable_callables.iter().cloned())
        .collect::<Vec<_>>();
    unused_candidate_callables.sort();
    let unused_callables = prunable_callables.clone();

    let mut used_items = decisions.used_items();
    used_items.sort();

    let mut blocked_by_unknown_items = decisions.blocked_by_unknown_items();
    blocked_by_unknown_items.sort();

    let mut prunable_items = decisions.prunable_items();
    prunable_items.sort();

    let mut unused_candidate_items = blocked_by_unknown_items
        .iter()
        .cloned()
        .chain(prunable_items.iter().cloned())
        .collect::<Vec<_>>();
    unused_candidate_items.sort();
    let unused_items = prunable_items.clone();

    let unknown = production
        .hazards
        .iter()
        .map(|hazard| UsageUnknownSurface {
            category: unknown_surface_category(hazard).to_string(),
            code: hazard.code.clone(),
            severity: hazard.severity.clone(),
            message: hazard.message.clone(),
            details: hazard.details.clone(),
        })
        .collect::<Vec<_>>();
    let benign_unknown_surfaces = unknown
        .iter()
        .filter(|surface| surface.category == "benign")
        .count();
    let macro_blocked_unknown_surfaces = unknown
        .iter()
        .filter(|surface| surface.category == "macro_blocked")
        .count();
    let dependency_risk_unknown_surfaces = unknown
        .iter()
        .filter(|surface| surface.category == "dependency_risk")
        .count();

    let status = if unknown.is_empty() {
        "classified".to_string()
    } else {
        "classified_with_unknowns".to_string()
    };
    let evidence_input = UsageEvidenceInput {
        used_callables: &used_callables,
        blocked_by_unknown_callables: &blocked_by_unknown_callables,
        prunable_callables: &prunable_callables,
        used_items: &used_items,
        blocked_by_unknown_items: &blocked_by_unknown_items,
        prunable_items: &prunable_items,
    };
    let evidence = usage_classification_evidence(project, reduced, &evidence_input);
    let semantic_proof =
        semantic_usage_proof_report(project, analyzer, decisions, Some(&rendered_symbols));
    let rendered_decision_map =
        RenderedUsageDecisionMap::from_rendered_symbols_and_decisions(&rendered_symbols, decisions);

    UsageClassificationReport {
        status,
        summary: UsageClassificationSummary {
            indexed_callables: used_callables.len() + unused_candidate_callables.len(),
            indexed_items: used_items.len() + unused_candidate_items.len(),
            used_callables: used_callables.len(),
            used_items: used_items.len(),
            unused_candidate_callables: unused_candidate_callables.len(),
            unused_candidate_items: unused_candidate_items.len(),
            blocked_by_unknown_callables: blocked_by_unknown_callables.len(),
            blocked_by_unknown_items: blocked_by_unknown_items.len(),
            prunable_callables: prunable_callables.len(),
            prunable_items: prunable_items.len(),
            unused_callables: unused_callables.len(),
            unused_items: unused_items.len(),
            unknown_surfaces: unknown.len(),
            benign_unknown_surfaces,
            macro_blocked_unknown_surfaces,
            dependency_risk_unknown_surfaces,
        },
        semantic_proof,
        rendered_symbols,
        rendered_decision_map,
        public_reexports,
        used: UsageClassifiedItems {
            callables: used_callables,
            items: used_items,
        },
        unused_candidate: UsageClassifiedItems {
            callables: unused_candidate_callables,
            items: unused_candidate_items,
        },
        blocked_by_unknown: UsageClassifiedItems {
            callables: blocked_by_unknown_callables,
            items: blocked_by_unknown_items,
        },
        prunable: UsageClassifiedItems {
            callables: prunable_callables,
            items: prunable_items,
        },
        unused: UsageClassifiedItems {
            callables: unused_callables,
            items: unused_items,
        },
        unknown,
        evidence,
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct GeneratedRenderedTraitDefaultMethod {
    trait_item: String,
    method: String,
}

#[derive(Default)]
struct GeneratedRenderedSymbols {
    callables: BTreeSet<String>,
    items: BTreeSet<String>,
    members: BTreeSet<String>,
    assoc_items: BTreeSet<String>,
    module_items: BTreeSet<String>,
    structural_module_items: BTreeSet<String>,
    structural_callables: BTreeSet<String>,
    macro_blocked_callables: BTreeSet<String>,
    surface_blocked_callables: BTreeSet<String>,
    trait_required_methods: BTreeMap<String, BTreeSet<String>>,
    trait_default_methods: BTreeSet<GeneratedRenderedTraitDefaultMethod>,
    direct_call_references: BTreeSet<String>,
    trait_default_method_references:
        BTreeMap<GeneratedRenderedTraitDefaultMethod, BTreeSet<String>>,
    source_parse_failures: usize,
}

#[cfg(test)]
fn rendered_symbol_proof_report(
    output_root: &Path,
    decisions: &UsageDecisionIndex,
) -> RenderedSymbolProofReport {
    rendered_symbol_proof_report_with_members(
        output_root,
        decisions,
        &render::RenderedMemberDecisionIndex::default(),
    )
}

fn rendered_symbol_proof_report_with_members(
    output_root: &Path,
    decisions: &UsageDecisionIndex,
    member_decisions: &render::RenderedMemberDecisionIndex,
) -> RenderedSymbolProofReport {
    let rendered = collect_generated_rendered_symbols(output_root, &decisions.retained_packages);
    let retained_callables = decisions
        .used_callables
        .union(&decisions.blocked_by_unknown_callables)
        .map(ToString::to_string)
        .collect::<BTreeSet<_>>();
    let retained_items = decisions
        .used_items
        .union(&decisions.blocked_by_unknown_items)
        .map(ToString::to_string)
        .collect::<BTreeSet<_>>();
    let blocked_items = decisions
        .blocked_by_unknown_items
        .iter()
        .map(ToString::to_string)
        .collect::<BTreeSet<_>>();
    let prunable_callables = decisions
        .prunable_callables
        .iter()
        .map(ToString::to_string)
        .collect::<BTreeSet<_>>();
    let prunable_items = decisions
        .prunable_items
        .iter()
        .map(ToString::to_string)
        .collect::<BTreeSet<_>>();
    let mut summary = RenderedSymbolProofSummary {
        rendered_callables: rendered.callables.len(),
        rendered_items: rendered.items.len(),
        rendered_members: rendered.members.len(),
        rendered_assoc_items: rendered.assoc_items.len(),
        rendered_trait_default_methods: rendered.trait_default_methods.len(),
        source_parse_failures: rendered.source_parse_failures,
        ..RenderedSymbolProofSummary::default()
    };
    let mut entries = Vec::new();

    for callable in &rendered.callables {
        let classification = if retained_callables.contains(callable) {
            summary.retained_callables += 1;
            "retained"
        } else if member_decisions.retained_callables.contains(callable) {
            summary.retained_callables += 1;
            "retained"
        } else if rendered.structural_callables.contains(callable) {
            summary.retained_callables += 1;
            "retained"
        } else if rendered.macro_blocked_callables.contains(callable) {
            summary.macro_blocked_callables += 1;
            "blocked_by_unknown"
        } else if rendered.surface_blocked_callables.contains(callable) {
            summary.surface_blocked_callables += 1;
            "blocked_by_unknown"
        } else if prunable_callables.contains(callable) {
            summary.prunable_callables += 1;
            "prunable"
        } else {
            summary.unclassified_callables += 1;
            "unclassified"
        };
        entries.push(RenderedSymbolProofEntry {
            kind: "callable".to_string(),
            id: callable.clone(),
            classification: classification.to_string(),
        });
    }

    for item in &rendered.items {
        let classification = if retained_items.contains(item) {
            summary.retained_items += 1;
            "retained"
        } else if member_decisions.retained_items.contains(item) {
            summary.retained_items += 1;
            "retained"
        } else if rendered.module_items.contains(item)
            && rendered.structural_module_items.contains(item)
        {
            summary.retained_items += 1;
            "retained"
        } else if prunable_items.contains(item) {
            summary.prunable_items += 1;
            "prunable"
        } else {
            summary.unclassified_items += 1;
            "unclassified"
        };
        entries.push(RenderedSymbolProofEntry {
            kind: "item".to_string(),
            id: item.clone(),
            classification: classification.to_string(),
        });
    }

    for member in &rendered.members {
        let classification = if member_decisions.retained.contains(member) {
            summary.retained_members += 1;
            "retained"
        } else if member_decisions.blocked_by_unknown.contains(member) {
            summary.blocked_members += 1;
            "blocked_by_unknown"
        } else if member_decisions.prunable.contains(member) {
            summary.prunable_members += 1;
            "prunable"
        } else {
            summary.unclassified_members += 1;
            "unclassified"
        };
        entries.push(RenderedSymbolProofEntry {
            kind: "member".to_string(),
            id: member.clone(),
            classification: classification.to_string(),
        });
    }

    for assoc_item in &rendered.assoc_items {
        let classification = if member_decisions.retained_assoc_items.contains(assoc_item) {
            summary.retained_assoc_items += 1;
            "retained"
        } else if member_decisions
            .blocked_by_unknown_assoc_items
            .contains(assoc_item)
        {
            summary.blocked_assoc_items += 1;
            "blocked_by_unknown"
        } else if member_decisions.prunable_assoc_items.contains(assoc_item) {
            summary.prunable_assoc_items += 1;
            "prunable"
        } else {
            summary.blocked_assoc_items += 1;
            "blocked_by_unknown"
        };
        entries.push(RenderedSymbolProofEntry {
            kind: "assoc_item".to_string(),
            id: assoc_item.clone(),
            classification: classification.to_string(),
        });
    }

    let retained_trait_default_methods =
        generated_retained_trait_default_methods(&rendered, &blocked_items);
    for method in &rendered.trait_default_methods {
        let classification = if !retained_trait_default_methods.contains(method) {
            summary.unproven_trait_default_methods += 1;
            "unproven"
        } else if blocked_items.contains(&method.trait_item) {
            summary.blocked_trait_default_methods += 1;
            "blocked_by_unknown"
        } else {
            summary.retained_trait_default_methods += 1;
            "retained"
        };
        entries.push(RenderedSymbolProofEntry {
            kind: "trait_default_method".to_string(),
            id: generated_rendered_trait_default_method_id(method),
            classification: classification.to_string(),
        });
    }

    let status = if summary.source_parse_failures > 0 {
        "incomplete".to_string()
    } else if summary.prunable_callables > 0
        || summary.prunable_items > 0
        || summary.prunable_members > 0
        || summary.prunable_assoc_items > 0
        || summary.unclassified_callables > 0
        || summary.unclassified_items > 0
        || summary.unclassified_members > 0
        || summary.unclassified_assoc_items > 0
        || summary.unproven_trait_default_methods > 0
    {
        "failed".to_string()
    } else {
        "proven".to_string()
    };

    RenderedSymbolProofReport {
        status,
        summary,
        entries,
    }
}

fn collect_generated_rendered_symbols(
    output_root: &Path,
    packages: &BTreeSet<String>,
) -> GeneratedRenderedSymbols {
    let mut symbols = GeneratedRenderedSymbols::default();
    for (package, source_root) in generated_package_source_roots(output_root, packages) {
        if !source_root.exists() {
            continue;
        }
        for file in generated_rust_files_under(&source_root) {
            let module_path = generated_module_path_from_source_file(&source_root, &file);
            let Ok(source) = fs::read_to_string(&file) else {
                symbols.source_parse_failures += 1;
                continue;
            };
            let Ok(syntax) = syn::parse_file(&source) else {
                symbols.source_parse_failures += 1;
                continue;
            };
            let aliases = generated_rendered_aliases_from_items(&syntax.items, None);
            collect_generated_rendered_trait_requirements(
                &package,
                &module_path,
                &syntax.items,
                &aliases,
                &mut symbols,
            );
            collect_generated_rendered_items(
                &package,
                &module_path,
                &syntax.items,
                &aliases,
                &mut symbols,
            );
        }
    }
    symbols
}

fn collect_generated_rendered_items(
    package: &str,
    module_path: &[String],
    items: &[syn::Item],
    aliases: &BTreeMap<String, Vec<String>>,
    symbols: &mut GeneratedRenderedSymbols,
) {
    let glob_roots = generated_rendered_glob_use_roots(module_path, items);
    for item in items {
        match item {
            syn::Item::Fn(function) => {
                record_generated_rendered_structural_modules(package, module_path, symbols);
                let callable = generated_rendered_symbol_path(
                    package,
                    module_path,
                    &function.sig.ident.to_string(),
                );
                if generated_rendered_fn_is_proc_macro_export(function) {
                    symbols.macro_blocked_callables.insert(callable.clone());
                }
                if generated_rendered_fn_is_structural_main(function, module_path) {
                    symbols.structural_callables.insert(callable.clone());
                }
                symbols.callables.insert(callable);
                symbols
                    .direct_call_references
                    .extend(generated_rendered_call_references_in_block(&function.block));
            }
            syn::Item::Struct(item) => {
                let item_id = generated_rendered_item_symbol_path(
                    package,
                    module_path,
                    &item.ident.to_string(),
                    "Struct",
                );
                symbols.items.insert(item_id.clone());
                symbols
                    .members
                    .extend(generated_rendered_struct_members(&item_id, &item.fields));
                record_generated_rendered_structural_modules(package, module_path, symbols);
            }
            syn::Item::Enum(item) => {
                let item_id = generated_rendered_item_symbol_path(
                    package,
                    module_path,
                    &item.ident.to_string(),
                    "Enum",
                );
                symbols.items.insert(item_id.clone());
                symbols
                    .members
                    .extend(generated_rendered_enum_members(&item_id, item));
                record_generated_rendered_structural_modules(package, module_path, symbols);
            }
            syn::Item::Union(item) => {
                symbols.items.insert(generated_rendered_item_symbol_path(
                    package,
                    module_path,
                    &item.ident.to_string(),
                    "Union",
                ));
                record_generated_rendered_structural_modules(package, module_path, symbols);
            }
            syn::Item::Type(item) => {
                symbols.items.insert(generated_rendered_item_symbol_path(
                    package,
                    module_path,
                    &item.ident.to_string(),
                    "Type",
                ));
                record_generated_rendered_structural_modules(package, module_path, symbols);
            }
            syn::Item::Trait(item) => {
                record_generated_rendered_structural_modules(package, module_path, symbols);
                let trait_item_id = generated_rendered_item_symbol_path(
                    package,
                    module_path,
                    &item.ident.to_string(),
                    "Trait",
                );
                symbols.items.insert(trait_item_id.clone());
                for trait_member in &item.items {
                    match trait_member {
                        syn::TraitItem::Fn(method) => {
                            let Some(block) = &method.default else {
                                continue;
                            };
                            let default_method = GeneratedRenderedTraitDefaultMethod {
                                trait_item: trait_item_id.clone(),
                                method: method.sig.ident.to_string(),
                            };
                            symbols.trait_default_methods.insert(default_method.clone());
                            symbols.trait_default_method_references.insert(
                                default_method,
                                generated_rendered_call_references_in_block(block),
                            );
                        }
                        syn::TraitItem::Const(item) => {
                            symbols
                                .assoc_items
                                .insert(generated_rendered_assoc_item_path(
                                    &trait_item_id,
                                    &item.ident.to_string(),
                                    "Const",
                                ));
                        }
                        syn::TraitItem::Type(item) => {
                            symbols
                                .assoc_items
                                .insert(generated_rendered_assoc_item_path(
                                    &trait_item_id,
                                    &item.ident.to_string(),
                                    "Type",
                                ));
                        }
                        syn::TraitItem::Macro(_) | syn::TraitItem::Verbatim(_) => {}
                        _ => {}
                    }
                }
            }
            syn::Item::Const(item) => {
                symbols.items.insert(generated_rendered_item_symbol_path(
                    package,
                    module_path,
                    &item.ident.to_string(),
                    "Const",
                ));
                record_generated_rendered_structural_modules(package, module_path, symbols);
            }
            syn::Item::Static(item) => {
                symbols.items.insert(generated_rendered_item_symbol_path(
                    package,
                    module_path,
                    &item.ident.to_string(),
                    "Static",
                ));
                record_generated_rendered_structural_modules(package, module_path, symbols);
            }
            syn::Item::Macro(item) => {
                if let Some(ident) = &item.ident {
                    symbols.items.insert(generated_rendered_item_symbol_path(
                        package,
                        module_path,
                        &ident.to_string(),
                        "Macro",
                    ));
                    record_generated_rendered_structural_modules(package, module_path, symbols);
                }
                if generated_rendered_item_macro_is_source_include(item) {
                    record_generated_rendered_structural_modules(package, module_path, symbols);
                }
            }
            syn::Item::Mod(item) => {
                let module_item = generated_rendered_item_symbol_path(
                    package,
                    module_path,
                    &item.ident.to_string(),
                    "Mod",
                );
                symbols.items.insert(module_item.clone());
                symbols.module_items.insert(module_item);
                if let Some((_, nested)) = &item.content {
                    let mut nested_module_path = module_path.to_vec();
                    nested_module_path.push(item.ident.to_string());
                    let nested_aliases = generated_rendered_aliases_from_items(nested, None);
                    collect_generated_rendered_items(
                        package,
                        &nested_module_path,
                        nested,
                        &nested_aliases,
                        symbols,
                    );
                }
            }
            syn::Item::Use(item) => {
                if generated_use_is_public_api_reexport(&item.vis) {
                    record_generated_rendered_structural_modules(package, module_path, symbols);
                }
            }
            syn::Item::Impl(item) => {
                if let Some(type_path) = generated_rendered_impl_type_path(
                    module_path,
                    &item.self_ty,
                    aliases,
                    &glob_roots,
                ) {
                    let trait_path = item.trait_.as_ref().map(|(_, path, _)| {
                        generated_rendered_normalized_path(module_path, path, aliases)
                    });
                    let required_trait_methods = trait_path
                        .as_ref()
                        .and_then(|trait_path| {
                            let trait_symbol =
                                generated_rendered_segments_path(package, trait_path);
                            symbols.trait_required_methods.get(&trait_symbol)
                        })
                        .cloned();
                    let trait_input_type_paths = item
                        .trait_
                        .as_ref()
                        .map(|(_, path, _)| {
                            generated_rendered_trait_input_type_paths(module_path, path, aliases)
                        })
                        .unwrap_or_default();
                    for impl_item in &item.items {
                        match impl_item {
                            syn::ImplItem::Fn(method) => {
                                record_generated_rendered_structural_modules(
                                    package,
                                    module_path,
                                    symbols,
                                );
                                let method_name = method.sig.ident.to_string();
                                let callable = generated_rendered_method_symbol_path(
                                    package,
                                    &type_path,
                                    trait_path.as_deref(),
                                    &trait_input_type_paths,
                                    &method_name,
                                );
                                if required_trait_methods
                                    .as_ref()
                                    .is_some_and(|methods| methods.contains(&method_name))
                                {
                                    symbols.surface_blocked_callables.insert(callable.clone());
                                }
                                symbols.callables.insert(callable);
                                symbols.direct_call_references.extend(
                                    generated_rendered_call_references_in_block(&method.block),
                                );
                            }
                            syn::ImplItem::Const(item) => {
                                record_generated_rendered_structural_modules(
                                    package,
                                    module_path,
                                    symbols,
                                );
                                symbols.assoc_items.insert(
                                    generated_rendered_impl_assoc_item_symbol_path(
                                        package,
                                        &type_path,
                                        trait_path.as_deref(),
                                        &trait_input_type_paths,
                                        &item.ident.to_string(),
                                        "Const",
                                    ),
                                );
                            }
                            syn::ImplItem::Type(item) => {
                                record_generated_rendered_structural_modules(
                                    package,
                                    module_path,
                                    symbols,
                                );
                                symbols.assoc_items.insert(
                                    generated_rendered_impl_assoc_item_symbol_path(
                                        package,
                                        &type_path,
                                        trait_path.as_deref(),
                                        &trait_input_type_paths,
                                        &item.ident.to_string(),
                                        "Type",
                                    ),
                                );
                            }
                            syn::ImplItem::Macro(_) | syn::ImplItem::Verbatim(_) => {}
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

fn record_generated_rendered_structural_modules(
    package: &str,
    module_path: &[String],
    symbols: &mut GeneratedRenderedSymbols,
) {
    symbols
        .structural_module_items
        .extend(generated_rendered_module_item_parents(package, module_path));
}

fn generated_rendered_module_item_parents(
    package: &str,
    module_path: &[String],
) -> BTreeSet<String> {
    let mut modules = BTreeSet::new();
    for index in 1..=module_path.len() {
        let parent = &module_path[..index - 1];
        let name = &module_path[index - 1];
        modules.insert(generated_rendered_item_symbol_path(
            package, parent, name, "Mod",
        ));
    }
    modules
}

fn collect_generated_rendered_trait_requirements(
    package: &str,
    module_path: &[String],
    items: &[syn::Item],
    _aliases: &BTreeMap<String, Vec<String>>,
    symbols: &mut GeneratedRenderedSymbols,
) {
    for item in items {
        match item {
            syn::Item::Trait(item) => {
                let trait_symbol =
                    generated_rendered_symbol_path(package, module_path, &item.ident.to_string());
                let required_methods = item
                    .items
                    .iter()
                    .filter_map(|trait_item| match trait_item {
                        syn::TraitItem::Fn(method) if method.default.is_none() => {
                            Some(method.sig.ident.to_string())
                        }
                        _ => None,
                    })
                    .collect::<BTreeSet<_>>();
                if !required_methods.is_empty() {
                    symbols
                        .trait_required_methods
                        .insert(trait_symbol, required_methods);
                }
            }
            syn::Item::Mod(item) => {
                if let Some((_, nested)) = &item.content {
                    let mut nested_module_path = module_path.to_vec();
                    nested_module_path.push(item.ident.to_string());
                    let nested_aliases = generated_rendered_aliases_from_items(nested, None);
                    collect_generated_rendered_trait_requirements(
                        package,
                        &nested_module_path,
                        nested,
                        &nested_aliases,
                        symbols,
                    );
                }
            }
            _ => {}
        }
    }
}

fn generated_rendered_fn_is_proc_macro_export(function: &syn::ItemFn) -> bool {
    function.attrs.iter().any(|attr| {
        attr.path().is_ident("proc_macro")
            || attr.path().is_ident("proc_macro_attribute")
            || attr.path().is_ident("proc_macro_derive")
    })
}

fn generated_rendered_fn_is_structural_main(
    function: &syn::ItemFn,
    module_path: &[String],
) -> bool {
    module_path.is_empty()
        && function.sig.ident == "main"
        && function.block.stmts.is_empty()
        && function.sig.inputs.is_empty()
}

fn generated_rendered_item_macro_is_source_include(item: &syn::ItemMacro) -> bool {
    item.mac.path.is_ident("include")
}

fn generated_retained_trait_default_methods(
    rendered: &GeneratedRenderedSymbols,
    blocked_items: &BTreeSet<String>,
) -> BTreeSet<GeneratedRenderedTraitDefaultMethod> {
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

fn generated_rendered_call_references_in_block(block: &syn::Block) -> BTreeSet<String> {
    let mut collector = GeneratedRenderedCallReferenceCollector::default();
    Visit::visit_block(&mut collector, block);
    collector.references
}

#[derive(Default)]
struct GeneratedRenderedCallReferenceCollector {
    references: BTreeSet<String>,
}

impl<'ast> Visit<'ast> for GeneratedRenderedCallReferenceCollector {
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

fn generated_rendered_aliases_from_items(
    items: &[syn::Item],
    parent: Option<&BTreeMap<String, Vec<String>>>,
) -> BTreeMap<String, Vec<String>> {
    let mut aliases = parent.cloned().unwrap_or_default();
    for item in items {
        if let syn::Item::Use(item_use) = item {
            collect_generated_rendered_use_tree(&item_use.tree, Vec::new(), &mut aliases);
        }
    }
    aliases
}

fn generated_rendered_glob_use_roots(
    module_path: &[String],
    items: &[syn::Item],
) -> Vec<Vec<String>> {
    let mut roots = Vec::new();
    for item in items {
        if let syn::Item::Use(item_use) = item {
            collect_generated_rendered_glob_use_roots(
                module_path,
                &item_use.tree,
                Vec::new(),
                &mut roots,
            );
        }
    }
    roots.sort();
    roots.dedup();
    roots
}

fn collect_generated_rendered_glob_use_roots(
    module_path: &[String],
    tree: &UseTree,
    mut prefix: Vec<String>,
    roots: &mut Vec<Vec<String>>,
) {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            collect_generated_rendered_glob_use_roots(module_path, &path.tree, prefix, roots);
        }
        UseTree::Group(group) => {
            for nested in &group.items {
                collect_generated_rendered_glob_use_roots(
                    module_path,
                    nested,
                    prefix.clone(),
                    roots,
                );
            }
        }
        UseTree::Glob(_) => {
            if let Some(root) = generated_rendered_normalize_segments(module_path, prefix) {
                roots.push(root);
            }
        }
        UseTree::Name(_) | UseTree::Rename(_) => {}
    }
}

fn collect_generated_rendered_use_tree(
    tree: &UseTree,
    mut prefix: Vec<String>,
    aliases: &mut BTreeMap<String, Vec<String>>,
) {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            collect_generated_rendered_use_tree(&path.tree, prefix, aliases);
        }
        UseTree::Name(name) => {
            let ident = name.ident.to_string();
            let mut target = prefix;
            target.push(ident.clone());
            aliases.insert(ident, target);
        }
        UseTree::Rename(rename) => {
            let mut target = prefix;
            target.push(rename.ident.to_string());
            aliases.insert(rename.rename.to_string(), target);
        }
        UseTree::Group(group) => {
            for nested in &group.items {
                collect_generated_rendered_use_tree(nested, prefix.clone(), aliases);
            }
        }
        UseTree::Glob(_) => {}
    }
}

fn generated_rendered_impl_type_path(
    module_path: &[String],
    ty: &syn::Type,
    aliases: &BTreeMap<String, Vec<String>>,
    glob_roots: &[Vec<String>],
) -> Option<Vec<String>> {
    let syn::Type::Path(path) = ty else {
        return None;
    };
    if path.qself.is_some() {
        return None;
    }
    generated_rendered_normalized_impl_type_path(module_path, ty, aliases, glob_roots)
}

fn generated_rendered_normalized_impl_type_path(
    module_path: &[String],
    ty: &syn::Type,
    aliases: &BTreeMap<String, Vec<String>>,
    glob_roots: &[Vec<String>],
) -> Option<Vec<String>> {
    let syn::Type::Path(type_path) = ty else {
        return None;
    };
    let segments = type_path
        .path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>();
    let aliased = generated_rendered_apply_alias(segments.clone(), aliases);
    if aliased == segments && segments.len() == 1 && glob_roots.len() == 1 {
        let mut path = glob_roots[0].clone();
        path.extend(segments);
        return Some(path);
    }
    generated_rendered_normalize_segments(module_path, aliased)
}

fn generated_rendered_normalized_path(
    module_path: &[String],
    path: &syn::Path,
    aliases: &BTreeMap<String, Vec<String>>,
) -> Vec<String> {
    let segments = generated_rendered_apply_alias(
        path.segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect(),
        aliases,
    );
    generated_rendered_normalize_segments(module_path, segments).unwrap_or_default()
}

fn generated_rendered_normalized_type_path(
    module_path: &[String],
    ty: &syn::Type,
    aliases: &BTreeMap<String, Vec<String>>,
) -> Option<Vec<String>> {
    let syn::Type::Path(type_path) = ty else {
        return None;
    };
    let segments = generated_rendered_apply_alias(
        type_path
            .path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect(),
        aliases,
    );
    generated_rendered_normalize_segments(module_path, segments)
}

fn generated_rendered_normalize_segments(
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

fn generated_rendered_apply_alias(
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

fn generated_rendered_trait_input_type_paths(
    module_path: &[String],
    path: &syn::Path,
    aliases: &BTreeMap<String, Vec<String>>,
) -> Vec<Vec<String>> {
    let mut type_paths = Vec::new();
    for segment in &path.segments {
        if let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments {
            for argument in &arguments.args {
                if let syn::GenericArgument::Type(ty) = argument {
                    collect_generated_rendered_type_paths(
                        module_path,
                        ty,
                        aliases,
                        &mut type_paths,
                    );
                }
            }
        }
    }
    type_paths.sort();
    type_paths.dedup();
    type_paths
}

fn collect_generated_rendered_type_paths(
    module_path: &[String],
    ty: &syn::Type,
    aliases: &BTreeMap<String, Vec<String>>,
    type_paths: &mut Vec<Vec<String>>,
) {
    match ty {
        syn::Type::Path(type_path) => {
            if let Some(path) = generated_rendered_normalized_type_path(module_path, ty, aliases) {
                type_paths.push(path);
            }
            for segment in &type_path.path.segments {
                if let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments {
                    for argument in &arguments.args {
                        if let syn::GenericArgument::Type(ty) = argument {
                            collect_generated_rendered_type_paths(
                                module_path,
                                ty,
                                aliases,
                                type_paths,
                            );
                        }
                    }
                }
            }
        }
        syn::Type::Reference(reference) => {
            collect_generated_rendered_type_paths(
                module_path,
                &reference.elem,
                aliases,
                type_paths,
            );
        }
        _ => {}
    }
}

fn generated_rendered_symbol_path(package: &str, module_path: &[String], name: &str) -> String {
    let mut path = package.to_string();
    for segment in module_path {
        path.push_str("::");
        path.push_str(segment);
    }
    path.push_str("::");
    path.push_str(name);
    path
}

fn generated_rendered_segments_path(package: &str, segments: &[String]) -> String {
    let mut path = package.to_string();
    for segment in segments {
        path.push_str("::");
        path.push_str(segment);
    }
    path
}

fn generated_rendered_trait_default_method_id(
    method: &GeneratedRenderedTraitDefaultMethod,
) -> String {
    format!("{}::{}", method.trait_item, method.method)
}

fn generated_rendered_method_symbol_path(
    package: &str,
    type_path: &[String],
    trait_path: Option<&[String]>,
    trait_input_type_paths: &[Vec<String>],
    method: &str,
) -> String {
    if let Some(trait_path) = trait_path {
        let mut path = format!("{package}::<");
        push_generated_rendered_segments(&mut path, type_path);
        path.push_str(" as ");
        push_generated_rendered_segments(&mut path, trait_path);
        if !trait_input_type_paths.is_empty() {
            path.push('<');
            for (index, input_path) in trait_input_type_paths.iter().enumerate() {
                if index > 0 {
                    path.push_str(", ");
                }
                push_generated_rendered_segments(&mut path, input_path);
            }
            path.push('>');
        }
        path.push_str(">::");
        path.push_str(method);
        path
    } else {
        generated_rendered_method_path(package, type_path, method)
    }
}

fn generated_rendered_impl_assoc_item_symbol_path(
    package: &str,
    type_path: &[String],
    trait_path: Option<&[String]>,
    trait_input_type_paths: &[Vec<String>],
    name: &str,
    kind: &str,
) -> String {
    let parent = if let Some(trait_path) = trait_path {
        let mut path = format!("{package}::<");
        push_generated_rendered_segments(&mut path, type_path);
        path.push_str(" as ");
        push_generated_rendered_segments(&mut path, trait_path);
        if !trait_input_type_paths.is_empty() {
            path.push('<');
            for (index, input_path) in trait_input_type_paths.iter().enumerate() {
                if index > 0 {
                    path.push_str(", ");
                }
                push_generated_rendered_segments(&mut path, input_path);
            }
            path.push('>');
        }
        path.push('>');
        path
    } else {
        generated_rendered_segments_path(package, type_path)
    };
    generated_rendered_assoc_item_path(&parent, name, kind)
}

fn push_generated_rendered_segments(output: &mut String, segments: &[String]) {
    for (index, segment) in segments.iter().enumerate() {
        if index > 0 {
            output.push_str("::");
        }
        output.push_str(segment);
    }
}

fn generated_rendered_method_path(package: &str, type_path: &[String], method: &str) -> String {
    let mut path = package.to_string();
    for segment in type_path {
        path.push_str("::");
        path.push_str(segment);
    }
    path.push_str("::");
    path.push_str(method);
    path
}

fn generated_rendered_item_symbol_path(
    package: &str,
    module_path: &[String],
    name: &str,
    kind: &str,
) -> String {
    format!(
        "{}({kind})",
        generated_rendered_symbol_path(package, module_path, name)
    )
}

fn generated_rendered_struct_members(parent: &str, fields: &syn::Fields) -> BTreeSet<String> {
    let mut members = BTreeSet::new();
    match fields {
        syn::Fields::Named(fields) => {
            for field in &fields.named {
                let Some(name) = field.ident.as_ref().map(ToString::to_string) else {
                    continue;
                };
                members.insert(generated_rendered_member_symbol_path(parent, &name));
            }
        }
        syn::Fields::Unnamed(fields) => {
            for index in 0..fields.unnamed.len() {
                members.insert(generated_rendered_member_symbol_path(
                    parent,
                    &index.to_string(),
                ));
            }
        }
        syn::Fields::Unit => {}
    }
    members
}

fn generated_rendered_enum_members(parent: &str, item: &syn::ItemEnum) -> BTreeSet<String> {
    item.variants
        .iter()
        .map(|variant| generated_rendered_member_symbol_path(parent, &variant.ident.to_string()))
        .collect()
}

fn generated_rendered_member_symbol_path(parent: &str, member: &str) -> String {
    format!("{parent}::{member}")
}

fn generated_rendered_assoc_item_path(parent: &str, name: &str, kind: &str) -> String {
    format!("{parent}::{name}({kind})")
}

#[derive(Clone, Debug)]
struct GeneratedPublicReexport {
    package: String,
    module_path: Vec<String>,
    visible: String,
    target: Vec<String>,
}

#[derive(Default)]
struct GeneratedPublicReexportSymbols {
    module_paths: BTreeSet<Vec<String>>,
    public_reexports: Vec<GeneratedPublicReexport>,
    source_parse_failures: usize,
}

#[cfg(test)]
fn public_reexport_proof_report(
    output_root: &Path,
    decisions: &UsageDecisionIndex,
) -> PublicReexportProofReport {
    public_reexport_proof_report_with_retained_surface_items(
        output_root,
        decisions,
        &BTreeSet::new(),
    )
}

fn public_reexport_proof_report_with_retained_surface_items(
    output_root: &Path,
    decisions: &UsageDecisionIndex,
    retained_surface_items: &BTreeSet<String>,
) -> PublicReexportProofReport {
    let symbols = collect_generated_public_reexports(output_root, &decisions.retained_packages);
    let mut retained_symbols = usage_symbol_base_paths(
        &decisions.used_callables,
        &decisions.blocked_by_unknown_callables,
        &decisions.used_items,
        &decisions.blocked_by_unknown_items,
    );
    retained_symbols.extend(
        retained_surface_items
            .iter()
            .map(|item| usage_item_base_path(item)),
    );
    let prunable_symbols = usage_symbol_base_paths(
        &decisions.prunable_callables,
        &BTreeSet::new(),
        &decisions.prunable_items,
        &BTreeSet::new(),
    );
    let known_symbols = retained_symbols
        .union(&prunable_symbols)
        .cloned()
        .collect::<BTreeSet<_>>();
    let exposed_reexports = generated_exposed_reexports(&symbols);
    let mut summary = PublicReexportProofSummary {
        public_reexports: symbols.public_reexports.len(),
        source_parse_failures: symbols.source_parse_failures,
        ..PublicReexportProofSummary::default()
    };
    let mut entries = Vec::new();

    for reexport in &symbols.public_reexports {
        let direct = generated_public_reexport_local_candidates(
            reexport,
            &symbols.module_paths,
            &known_symbols,
        );
        let resolved = generated_public_reexport_target_closure(
            reexport,
            &symbols.module_paths,
            &known_symbols,
            &exposed_reexports,
        );
        if !resolved.is_empty() {
            summary.local_targets += 1;
        }
        if resolved.len() > direct.len() {
            summary.facade_chain_targets += 1;
        }

        let classification = if resolved.is_empty() {
            summary.external_or_unresolved_targets += 1;
            "external_or_unresolved"
        } else if resolved
            .iter()
            .any(|candidate| retained_symbols.contains(candidate))
        {
            summary.retained_targets += 1;
            "retained"
        } else if resolved
            .iter()
            .any(|candidate| prunable_symbols.contains(candidate))
        {
            summary.prunable_targets += 1;
            "prunable"
        } else {
            summary.unclassified_targets += 1;
            "unclassified"
        };

        entries.push(PublicReexportProofEntry {
            package: reexport.package.clone(),
            module_path: (!reexport.module_path.is_empty())
                .then(|| reexport.module_path.join("::")),
            visible: reexport.visible.clone(),
            target: reexport.target.join("::"),
            resolved_targets: resolved.into_iter().collect(),
            classification: classification.to_string(),
        });
    }

    let status = if summary.source_parse_failures > 0 {
        "incomplete".to_string()
    } else if summary.prunable_targets > 0 || summary.unclassified_targets > 0 {
        "failed".to_string()
    } else {
        "proven".to_string()
    };

    PublicReexportProofReport {
        status,
        summary,
        entries,
    }
}

fn usage_symbol_base_paths(
    left_callables: &BTreeSet<CallableId>,
    right_callables: &BTreeSet<CallableId>,
    left_items: &BTreeSet<ItemId>,
    right_items: &BTreeSet<ItemId>,
) -> BTreeSet<String> {
    let mut paths = left_callables
        .union(right_callables)
        .map(ToString::to_string)
        .collect::<BTreeSet<_>>();
    paths.extend(
        left_items
            .union(right_items)
            .map(ToString::to_string)
            .map(|item| usage_item_base_path(&item)),
    );
    paths
}

fn usage_item_base_path(item: &str) -> String {
    item.rfind('(')
        .map(|index| item[..index].to_string())
        .unwrap_or_else(|| item.to_string())
}

fn collect_generated_public_reexports(
    output_root: &Path,
    packages: &BTreeSet<String>,
) -> GeneratedPublicReexportSymbols {
    let mut symbols = GeneratedPublicReexportSymbols::default();
    for (package, source_root) in generated_package_source_roots(output_root, packages) {
        if !source_root.exists() {
            continue;
        }
        for file in generated_rust_files_under(&source_root) {
            let module_path = generated_module_path_from_source_file(&source_root, &file);
            insert_generated_module_path_with_parents(&mut symbols.module_paths, &module_path);
            let Ok(source) = fs::read_to_string(&file) else {
                symbols.source_parse_failures += 1;
                continue;
            };
            let Ok(syntax) = syn::parse_file(&source) else {
                symbols.source_parse_failures += 1;
                continue;
            };
            collect_generated_public_reexport_items(
                &package,
                &module_path,
                &syntax.items,
                &mut symbols,
            );
        }
    }
    symbols
}

fn generated_package_source_roots(
    output_root: &Path,
    packages: &BTreeSet<String>,
) -> BTreeMap<String, PathBuf> {
    let mut manifests = Vec::new();
    collect_generated_package_manifests(output_root, &mut manifests);
    let mut roots = BTreeMap::new();
    for manifest in manifests {
        let Ok(source) = fs::read_to_string(&manifest) else {
            continue;
        };
        let Ok(value) = source.parse::<toml::Value>() else {
            continue;
        };
        let Some(name) = value
            .get("package")
            .and_then(|package| package.get("name"))
            .and_then(toml::Value::as_str)
        else {
            continue;
        };
        if !packages.contains(name) {
            continue;
        }
        let Some(package_root) = manifest.parent() else {
            continue;
        };
        roots
            .entry(name.to_string())
            .or_insert_with(|| package_root.join("src"));
    }
    for package in packages {
        roots
            .entry(package.clone())
            .or_insert_with(|| output_root.join(package).join("src"));
    }
    roots
}

fn collect_generated_package_manifests(root: &Path, manifests: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| matches!(name, ".git" | ".hg" | ".svn" | "target"))
        {
            continue;
        }
        if path.is_dir() {
            collect_generated_package_manifests(&path, manifests);
        } else if path.file_name().and_then(|name| name.to_str()) == Some("Cargo.toml") {
            manifests.push(path);
        }
    }
}

fn collect_generated_public_reexport_items(
    package: &str,
    module_path: &[String],
    items: &[syn::Item],
    symbols: &mut GeneratedPublicReexportSymbols,
) {
    for item in items {
        match item {
            syn::Item::Use(item_use) if generated_use_is_public_api_reexport(&item_use.vis) => {
                collect_generated_public_reexport_tree(
                    package,
                    module_path,
                    &item_use.tree,
                    Vec::new(),
                    symbols,
                );
            }
            syn::Item::Mod(item) => {
                if let Some((_, nested)) = &item.content {
                    let mut nested_module_path = module_path.to_vec();
                    nested_module_path.push(item.ident.to_string());
                    insert_generated_module_path_with_parents(
                        &mut symbols.module_paths,
                        &nested_module_path,
                    );
                    collect_generated_public_reexport_items(
                        package,
                        &nested_module_path,
                        nested,
                        symbols,
                    );
                }
            }
            _ => {}
        }
    }
}

fn collect_generated_public_reexport_tree(
    package: &str,
    module_path: &[String],
    tree: &UseTree,
    mut prefix: Vec<String>,
    symbols: &mut GeneratedPublicReexportSymbols,
) {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            collect_generated_public_reexport_tree(
                package,
                module_path,
                &path.tree,
                prefix,
                symbols,
            );
        }
        UseTree::Name(name) => {
            if let Some((visible, target)) = generated_reexport_leaf(prefix, name.ident.to_string())
            {
                symbols.public_reexports.push(GeneratedPublicReexport {
                    package: package.to_string(),
                    module_path: module_path.to_vec(),
                    visible,
                    target,
                });
            }
        }
        UseTree::Rename(rename) => {
            if let Some((_, target)) = generated_reexport_leaf(prefix, rename.ident.to_string()) {
                symbols.public_reexports.push(GeneratedPublicReexport {
                    package: package.to_string(),
                    module_path: module_path.to_vec(),
                    visible: rename.rename.to_string(),
                    target,
                });
            }
        }
        UseTree::Group(group) => {
            for nested in &group.items {
                collect_generated_public_reexport_tree(
                    package,
                    module_path,
                    nested,
                    prefix.clone(),
                    symbols,
                );
            }
        }
        UseTree::Glob(_) => {}
    }
}

fn generated_reexport_leaf(
    mut prefix: Vec<String>,
    ident: String,
) -> Option<(String, Vec<String>)> {
    if ident == "self" {
        let visible = prefix.last()?.clone();
        Some((visible, prefix))
    } else {
        prefix.push(ident.clone());
        Some((ident, prefix))
    }
}

fn generated_exposed_reexports(
    symbols: &GeneratedPublicReexportSymbols,
) -> BTreeMap<String, Vec<GeneratedPublicReexport>> {
    let mut reexports = BTreeMap::<String, Vec<GeneratedPublicReexport>>::new();
    for reexport in &symbols.public_reexports {
        let mut exposed_path = reexport.module_path.clone();
        exposed_path.push(reexport.visible.clone());
        reexports
            .entry(generated_qualified_symbol_path(
                &reexport.package,
                &exposed_path,
            ))
            .or_default()
            .push(reexport.clone());
    }
    reexports
}

fn generated_public_reexport_target_closure(
    reexport: &GeneratedPublicReexport,
    module_paths: &BTreeSet<Vec<String>>,
    known_symbols: &BTreeSet<String>,
    exposed_reexports: &BTreeMap<String, Vec<GeneratedPublicReexport>>,
) -> BTreeSet<String> {
    let mut candidates = BTreeSet::new();
    let mut pending =
        generated_public_reexport_local_candidates(reexport, module_paths, known_symbols)
            .into_iter()
            .collect::<Vec<_>>();
    if let Some(target) = generated_exposed_raw_reexport_target(&reexport.target, exposed_reexports)
    {
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
            for next in
                generated_public_reexport_local_candidates(exposed, module_paths, known_symbols)
            {
                if !candidates.contains(&next) {
                    pending.push(next);
                }
            }
            if let Some(next) =
                generated_exposed_raw_reexport_target(&exposed.target, exposed_reexports)
            {
                if !candidates.contains(&next) {
                    pending.push(next);
                }
            }
        }
    }

    candidates
}

fn generated_public_reexport_local_candidates(
    reexport: &GeneratedPublicReexport,
    module_paths: &BTreeSet<Vec<String>>,
    known_symbols: &BTreeSet<String>,
) -> BTreeSet<String> {
    let mut candidates = BTreeSet::new();
    if let Some(target) = generated_known_raw_reexport_target(&reexport.target, known_symbols) {
        candidates.insert(target);
    }
    if let Some(local_segments) =
        generated_normalize_public_reexport_target(&reexport.module_path, &reexport.target)
    {
        let explicit_local = reexport
            .target
            .first()
            .is_some_and(|segment| segment == "crate" || segment == "self" || segment == "super");
        if generated_local_reexport_candidate_is_local(
            &reexport.package,
            &local_segments,
            module_paths,
            known_symbols,
            explicit_local,
        ) {
            candidates.insert(generated_qualified_symbol_path(
                &reexport.package,
                &local_segments,
            ));
        }
    }

    if reexport
        .target
        .first()
        .is_some_and(|segment| segment != "crate" && segment != "self" && segment != "super")
        && generated_local_reexport_candidate_is_local(
            &reexport.package,
            &reexport.target,
            module_paths,
            known_symbols,
            false,
        )
    {
        candidates.insert(generated_qualified_symbol_path(
            &reexport.package,
            &reexport.target,
        ));
    }

    candidates
}

fn generated_known_raw_reexport_target(
    target: &[String],
    known_symbols: &BTreeSet<String>,
) -> Option<String> {
    let path = generated_raw_reexport_target(target)?;
    known_symbols.contains(&path).then_some(path)
}

fn generated_exposed_raw_reexport_target(
    target: &[String],
    exposed_reexports: &BTreeMap<String, Vec<GeneratedPublicReexport>>,
) -> Option<String> {
    let path = generated_raw_reexport_target(target)?;
    exposed_reexports.contains_key(&path).then_some(path)
}

fn generated_raw_reexport_target(target: &[String]) -> Option<String> {
    if target.is_empty() {
        return None;
    }
    Some(target.join("::"))
}

fn generated_local_reexport_candidate_is_local(
    package: &str,
    segments: &[String],
    module_paths: &BTreeSet<Vec<String>>,
    known_symbols: &BTreeSet<String>,
    explicit_local: bool,
) -> bool {
    if segments.is_empty() {
        return false;
    }
    let symbol = generated_qualified_symbol_path(package, segments);
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

fn generated_normalize_public_reexport_target(
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

fn generated_qualified_symbol_path(package: &str, segments: &[String]) -> String {
    let mut path = package.to_string();
    for segment in segments {
        path.push_str("::");
        path.push_str(segment);
    }
    path
}

fn generated_rust_files_under(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_generated_rust_files(root, &mut files);
    files.sort();
    files
}

fn collect_generated_rust_files(root: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_generated_rust_files(&path, files);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}

fn generated_module_path_from_source_file(source_root: &Path, file: &Path) -> Vec<String> {
    let Ok(relative) = file.strip_prefix(source_root) else {
        return Vec::new();
    };
    let mut components = relative
        .components()
        .filter_map(|component| match component {
            Component::Normal(value) => Some(value.to_string_lossy().to_string()),
            _ => None,
        })
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
                .map(|stem| stem.to_string_lossy().to_string())
                .unwrap_or_default();
            components.push(stem);
            components
        }
    }
}

fn insert_generated_module_path_with_parents(
    modules: &mut BTreeSet<Vec<String>>,
    module_path: &[String],
) {
    modules.insert(Vec::new());
    for index in 1..=module_path.len() {
        modules.insert(module_path[..index].to_vec());
    }
}

fn generated_use_is_public_api_reexport(visibility: &syn::Visibility) -> bool {
    matches!(visibility, syn::Visibility::Public(_))
}

fn add_rendered_symbol_proof_hazards(
    proof: &RenderedSymbolProofReport,
    hazards: &mut Vec<ProductionHazardReport>,
) {
    if proof.summary.prunable_callables > 0
        || proof.summary.prunable_items > 0
        || proof.summary.prunable_members > 0
        || proof.summary.prunable_assoc_items > 0
    {
        hazards.push(production_hazard_with_details(
            "rendered_prunable_symbols",
            "error",
            "generated source still declares callables, items, members, or associated items classified as prunable",
            rendered_symbol_proof_details(proof, "prunable"),
        ));
    }
    if proof.summary.unclassified_callables > 0
        || proof.summary.unclassified_items > 0
        || proof.summary.unclassified_members > 0
        || proof.summary.unclassified_assoc_items > 0
    {
        hazards.push(production_hazard_with_details(
            "rendered_unclassified_symbols",
            "error",
            "generated source declares callables, items, members, or associated items missing used/unknown classification",
            rendered_symbol_proof_details(proof, "unclassified"),
        ));
    }
    if proof.summary.unproven_trait_default_methods > 0 {
        hazards.push(production_hazard_with_details(
            "rendered_unproven_trait_default_methods",
            "error",
            "generated source retained trait default methods without used/unknown proof",
            rendered_symbol_proof_details(proof, "unproven"),
        ));
    }
    if proof.summary.source_parse_failures > 0 {
        hazards.push(production_hazard(
            "rendered_symbol_proof_incomplete",
            "warning",
            "generated source-symbol proof skipped source files that did not parse",
        ));
    }
}

fn rendered_symbol_proof_details(
    proof: &RenderedSymbolProofReport,
    classification: &str,
) -> Vec<ProductionHazardDetail> {
    proof
        .entries
        .iter()
        .filter(|entry| entry.classification == classification)
        .map(|entry| ProductionHazardDetail {
            subject: format!("{}={}", entry.kind, entry.id),
            package: rendered_symbol_package(&entry.id),
            module_path: None,
            file: None,
            start_line: None,
            cfg: None,
            blocked_idents: vec![entry.id.clone()],
            suggested_cargo_args: Vec::new(),
        })
        .collect()
}

fn rendered_symbol_package(id: &str) -> Option<String> {
    id.split("::").next().map(str::to_string)
}

fn add_public_reexport_proof_hazards(
    proof: &PublicReexportProofReport,
    hazards: &mut Vec<ProductionHazardReport>,
) {
    if proof.summary.prunable_targets > 0 {
        hazards.push(production_hazard_with_details(
            "public_reexport_prunable_targets",
            "error",
            "generated public reexports point at local targets classified as prunable",
            public_reexport_proof_details(proof, "prunable"),
        ));
    }
    if proof.summary.unclassified_targets > 0 {
        hazards.push(production_hazard_with_details(
            "public_reexport_unclassified_targets",
            "error",
            "generated public reexports point at local targets missing usage classification",
            public_reexport_proof_details(proof, "unclassified"),
        ));
    }
    if proof.summary.source_parse_failures > 0 {
        hazards.push(production_hazard(
            "public_reexport_proof_incomplete",
            "warning",
            "generated public reexport proof skipped source files that did not parse",
        ));
    }
}

fn public_reexport_proof_details(
    proof: &PublicReexportProofReport,
    classification: &str,
) -> Vec<ProductionHazardDetail> {
    proof
        .entries
        .iter()
        .filter(|entry| entry.classification == classification)
        .map(|entry| ProductionHazardDetail {
            subject: format!(
                "visible={}; target={}; resolved={}",
                entry.visible,
                entry.target,
                entry.resolved_targets.join(",")
            ),
            package: Some(entry.package.clone()),
            module_path: entry.module_path.clone(),
            file: None,
            start_line: None,
            cfg: None,
            blocked_idents: vec![entry.visible.clone()],
            suggested_cargo_args: Vec::new(),
        })
        .collect()
}

fn unknown_surface_category(hazard: &ProductionHazardReport) -> &'static str {
    match hazard.code.as_str() {
        "custom_attribute_macros" | "custom_derive_macros" | "custom_macro_invocations" => {
            "macro_blocked"
        }
        "semantic_unresolved_method_calls" | "semantic_unresolved_paths" => {
            semantic_unresolved_hazard_category(hazard)
        }
        "semantic_reduction_hints_applied" | "semantic_inventory_partially_applied" => "benign",
        _ => "dependency_risk",
    }
}

fn semantic_unresolved_hazard_category(hazard: &ProductionHazardReport) -> &'static str {
    if hazard.details.is_empty() {
        return "dependency_risk";
    }
    if hazard
        .details
        .iter()
        .any(|detail| detail.subject.starts_with("category=dependency_risk;"))
    {
        return "dependency_risk";
    }
    if hazard
        .details
        .iter()
        .any(|detail| detail.subject.starts_with("category=macro_blocked;"))
    {
        return "macro_blocked";
    }
    "benign"
}

fn semantic_usage_proof_report(
    project: &Project,
    analyzer: &AnalyzerReport,
    decisions: &UsageDecisionIndex,
    rendered_symbols: Option<&RenderedSymbolProofReport>,
) -> SemanticUsageProofReport {
    let mut summary = SemanticUsageProofSummary {
        analyzer_available: analyzer.semantic_usage.is_some(),
        retained_packages: decisions.retained_packages.len(),
        prunable_callables: decisions.prunable_callables.len(),
        prunable_items: decisions.prunable_items.len(),
        ..SemanticUsageProofSummary::default()
    };
    let retained_callables = decisions
        .used_callables
        .union(&decisions.blocked_by_unknown_callables)
        .cloned()
        .collect::<BTreeSet<_>>();
    let retained_items = decisions
        .used_items
        .union(&decisions.blocked_by_unknown_items)
        .cloned()
        .collect::<BTreeSet<_>>();
    let retained_source_files =
        retained_source_files(project, &retained_callables, &retained_items);
    let rendered_absence = RenderedAbsenceProof::from_report(rendered_symbols);

    let mut unproven_callables = Vec::new();
    let mut unproven_items = Vec::new();
    for callable in &decisions.prunable_callables {
        if !decisions.retained_packages.contains(callable.package()) {
            summary.package_pruned_callables += 1;
            continue;
        }
        summary.proof_required_callables += 1;
        if let Some(usage) = &analyzer.semantic_usage {
            let mut unproven = false;
            let discharge = callable_semantic_proof_discharge(
                project,
                callable,
                &retained_source_files,
                &rendered_absence,
            );
            let mut cfg_discharged = false;
            let mut source_file_discharged = false;
            let mut rendered_absent_discharged = false;
            if !usage.is_callable_mapped(callable) {
                match discharge {
                    SemanticProofDischarge::CfgInactive => cfg_discharged = true,
                    SemanticProofDischarge::SourceFilePruned => source_file_discharged = true,
                    SemanticProofDischarge::RenderedAbsent => rendered_absent_discharged = true,
                    SemanticProofDischarge::None | SemanticProofDischarge::StructuralPruned => {
                        summary.unmapped_callables += 1;
                        unproven = true;
                    }
                }
            }
            if usage.callable_reference_query_failed(callable) {
                match discharge {
                    SemanticProofDischarge::CfgInactive => cfg_discharged = true,
                    SemanticProofDischarge::SourceFilePruned => source_file_discharged = true,
                    SemanticProofDischarge::RenderedAbsent => rendered_absent_discharged = true,
                    SemanticProofDischarge::None | SemanticProofDischarge::StructuralPruned => {
                        summary.failed_reference_query_callables += 1;
                        unproven = true;
                    }
                }
            }
            if usage.callable_reference_query_skipped(callable) {
                match discharge {
                    SemanticProofDischarge::CfgInactive => cfg_discharged = true,
                    SemanticProofDischarge::SourceFilePruned => source_file_discharged = true,
                    SemanticProofDischarge::RenderedAbsent => rendered_absent_discharged = true,
                    SemanticProofDischarge::None | SemanticProofDischarge::StructuralPruned => {
                        summary.skipped_reference_query_callables += 1;
                        unproven = true;
                    }
                }
            }
            if usage.callable_has_retained_reference(callable, &retained_callables, &retained_items)
            {
                summary.retained_reference_callables += 1;
                unproven = true;
            }
            if unproven {
                unproven_callables.push(callable.clone());
            } else {
                if cfg_discharged {
                    summary.cfg_inactive_callables += 1;
                } else if source_file_discharged {
                    summary.source_file_pruned_callables += 1;
                } else if rendered_absent_discharged {
                    summary.rendered_absent_callables += 1;
                }
                summary.proven_callables += 1;
            }
        } else {
            unproven_callables.push(callable.clone());
        }
    }
    for item in &decisions.prunable_items {
        if !decisions.retained_packages.contains(&item.package) {
            summary.package_pruned_items += 1;
            continue;
        }
        summary.proof_required_items += 1;
        if let Some(usage) = &analyzer.semantic_usage {
            let mut unproven = false;
            let discharge = item_semantic_proof_discharge(
                project,
                item,
                &retained_source_files,
                &retained_callables,
                &retained_items,
                &rendered_absence,
            );
            let mut cfg_discharged = false;
            let mut source_file_discharged = false;
            let mut rendered_absent_discharged = false;
            let mut structural_discharged = false;
            if !usage.is_item_mapped(item) {
                match discharge {
                    SemanticProofDischarge::CfgInactive => cfg_discharged = true,
                    SemanticProofDischarge::SourceFilePruned => source_file_discharged = true,
                    SemanticProofDischarge::RenderedAbsent => rendered_absent_discharged = true,
                    SemanticProofDischarge::StructuralPruned => structural_discharged = true,
                    SemanticProofDischarge::None => {
                        summary.unmapped_items += 1;
                        unproven = true;
                    }
                }
            }
            if usage.item_reference_query_failed(item) {
                match discharge {
                    SemanticProofDischarge::CfgInactive => cfg_discharged = true,
                    SemanticProofDischarge::SourceFilePruned => source_file_discharged = true,
                    SemanticProofDischarge::RenderedAbsent => rendered_absent_discharged = true,
                    SemanticProofDischarge::StructuralPruned => structural_discharged = true,
                    SemanticProofDischarge::None => {
                        summary.failed_reference_query_items += 1;
                        unproven = true;
                    }
                }
            }
            if usage.item_reference_query_skipped(item) {
                match discharge {
                    SemanticProofDischarge::CfgInactive => cfg_discharged = true,
                    SemanticProofDischarge::SourceFilePruned => source_file_discharged = true,
                    SemanticProofDischarge::RenderedAbsent => rendered_absent_discharged = true,
                    SemanticProofDischarge::StructuralPruned => structural_discharged = true,
                    SemanticProofDischarge::None => {
                        summary.skipped_reference_query_items += 1;
                        unproven = true;
                    }
                }
            }
            if usage.item_has_retained_reference(item, &retained_callables, &retained_items) {
                summary.retained_reference_items += 1;
                unproven = true;
            }
            if unproven {
                unproven_items.push(item.clone());
            } else {
                if cfg_discharged {
                    summary.cfg_inactive_items += 1;
                } else if source_file_discharged {
                    summary.source_file_pruned_items += 1;
                } else if rendered_absent_discharged {
                    summary.rendered_absent_items += 1;
                } else if structural_discharged {
                    summary.structural_pruned_items += 1;
                }
                summary.proven_items += 1;
            }
        } else {
            unproven_items.push(item.clone());
        }
    }

    unproven_callables.sort();
    unproven_items.sort();
    summary.unproven_callables = unproven_callables.len();
    summary.unproven_items = unproven_items.len();
    let status = if analyzer.semantic_usage.is_none()
        && (summary.proof_required_callables + summary.proof_required_items > 0)
    {
        "semantic_usage_unavailable"
    } else if summary.unproven_callables + summary.unproven_items == 0 {
        "complete_for_retained_packages"
    } else {
        "partial_for_retained_packages"
    }
    .to_string();

    SemanticUsageProofReport {
        status,
        summary,
        unproven: UsageClassifiedItems {
            callables: unproven_callables,
            items: unproven_items,
        },
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SemanticProofDischarge {
    None,
    CfgInactive,
    SourceFilePruned,
    RenderedAbsent,
    StructuralPruned,
}

#[derive(Default)]
struct RenderedAbsenceProof {
    enabled: bool,
    callables: BTreeSet<String>,
    items: BTreeSet<String>,
}

impl RenderedAbsenceProof {
    fn from_report(report: Option<&RenderedSymbolProofReport>) -> Self {
        let Some(report) = report else {
            return Self::default();
        };
        if report.status != "proven" {
            return Self::default();
        }
        let mut proof = Self {
            enabled: true,
            ..Self::default()
        };
        for entry in &report.entries {
            match entry.kind.as_str() {
                "callable" => {
                    proof.callables.insert(entry.id.clone());
                }
                "item" => {
                    proof.items.insert(entry.id.clone());
                }
                _ => {}
            }
        }
        proof
    }

    fn callable_is_absent(&self, callable: &CallableId) -> bool {
        self.enabled && !self.callables.contains(&callable.to_string())
    }

    fn item_is_absent(&self, item: &ItemId) -> bool {
        self.enabled && !self.items.contains(&item.to_string())
    }
}

fn callable_semantic_proof_discharge(
    project: &Project,
    callable: &CallableId,
    retained_source_files: &BTreeSet<PathBuf>,
    rendered_absence: &RenderedAbsenceProof,
) -> SemanticProofDischarge {
    if callable_is_current_target_inactive(project, callable) {
        return SemanticProofDischarge::CfgInactive;
    }
    if callable_source_file_is_pruned(project, callable, retained_source_files) {
        return SemanticProofDischarge::SourceFilePruned;
    }
    if rendered_absence.callable_is_absent(callable) {
        return SemanticProofDischarge::RenderedAbsent;
    }
    SemanticProofDischarge::None
}

fn item_semantic_proof_discharge(
    project: &Project,
    item: &ItemId,
    retained_source_files: &BTreeSet<PathBuf>,
    retained_callables: &BTreeSet<CallableId>,
    retained_items: &BTreeSet<ItemId>,
    rendered_absence: &RenderedAbsenceProof,
) -> SemanticProofDischarge {
    if item_is_current_target_inactive(project, item) {
        return SemanticProofDischarge::CfgInactive;
    }
    if item_source_file_is_pruned(project, item, retained_source_files) {
        return SemanticProofDischarge::SourceFilePruned;
    }
    if item.kind == model::ItemKind::Macro {
        return SemanticProofDischarge::StructuralPruned;
    }
    if rendered_absence.item_is_absent(item) {
        return SemanticProofDischarge::RenderedAbsent;
    }
    if item_module_has_no_retained_subtree(project, item, retained_callables, retained_items) {
        return SemanticProofDischarge::StructuralPruned;
    }
    if pruned_module_has_no_retained_subtree(project, item, retained_callables, retained_items) {
        return SemanticProofDischarge::StructuralPruned;
    }
    SemanticProofDischarge::None
}

fn retained_source_files(
    project: &Project,
    retained_callables: &BTreeSet<CallableId>,
    retained_items: &BTreeSet<ItemId>,
) -> BTreeSet<PathBuf> {
    retained_callables
        .iter()
        .filter_map(|callable| callable_source_file(project, callable).cloned())
        .chain(
            retained_items
                .iter()
                .filter_map(|item| item_source_file(project, item).cloned()),
        )
        .collect()
}

fn callable_source_file<'a>(project: &'a Project, callable: &CallableId) -> Option<&'a PathBuf> {
    project
        .functions
        .get(callable)
        .map(|record| &record.span.file)
        .or_else(|| {
            project
                .methods
                .get(callable)
                .map(|record| &record.span.file)
        })
}

fn item_source_file<'a>(project: &'a Project, item: &ItemId) -> Option<&'a PathBuf> {
    project.items.get(item).map(|record| &record.span.file)
}

fn callable_source_file_is_pruned(
    project: &Project,
    callable: &CallableId,
    retained_source_files: &BTreeSet<PathBuf>,
) -> bool {
    callable_source_file(project, callable)
        .is_some_and(|file| !retained_source_files.contains(file))
}

fn item_source_file_is_pruned(
    project: &Project,
    item: &ItemId,
    retained_source_files: &BTreeSet<PathBuf>,
) -> bool {
    item_source_file(project, item).is_some_and(|file| !retained_source_files.contains(file))
}

fn item_module_has_no_retained_subtree(
    project: &Project,
    item: &ItemId,
    retained_callables: &BTreeSet<CallableId>,
    retained_items: &BTreeSet<ItemId>,
) -> bool {
    if item.module_path.is_empty() {
        return false;
    }
    let retained_callables_in_module = retained_callables.iter().any(|callable| {
        callable.package() == item.package
            && callable_module_path(project, callable)
                .as_deref()
                .is_some_and(|module_path| path_has_prefix(module_path, &item.module_path))
    });
    let retained_items_in_module = retained_items.iter().any(|retained| {
        retained.package == item.package
            && path_has_prefix(&retained_item_module_path(retained), &item.module_path)
    });
    !retained_callables_in_module && !retained_items_in_module
}

fn retained_item_module_path(item: &ItemId) -> Vec<String> {
    if item.kind == model::ItemKind::Mod {
        let mut module_path = item.module_path.clone();
        module_path.push(item.name.clone());
        module_path
    } else {
        item.module_path.clone()
    }
}

fn pruned_module_has_no_retained_subtree(
    project: &Project,
    item: &ItemId,
    retained_callables: &BTreeSet<CallableId>,
    retained_items: &BTreeSet<ItemId>,
) -> bool {
    if item.kind != model::ItemKind::Mod {
        return false;
    }
    let mut child_module_path = item.module_path.clone();
    child_module_path.push(item.name.clone());
    let retained_callables_in_module = retained_callables.iter().any(|callable| {
        callable.package() == item.package
            && callable_module_path(project, callable)
                .as_deref()
                .is_some_and(|module_path| path_has_prefix(module_path, &child_module_path))
    });
    let retained_items_in_module = retained_items.iter().any(|retained| {
        retained.package == item.package
            && path_has_prefix(&retained.module_path, &child_module_path)
    });
    !retained_callables_in_module && !retained_items_in_module
}

fn callable_is_current_target_inactive(project: &Project, callable: &CallableId) -> bool {
    if let Some(record) = project.functions.get(callable) {
        return attrs_disable_current_target(&record.item.attrs)
            || module_path_disables_current_target(
                project,
                callable.package(),
                &record.module_path,
            );
    }
    if let Some(record) = project.methods.get(callable) {
        return attrs_disable_current_target(&record.item.attrs)
            || attrs_disable_current_target(&record.impl_attrs)
            || module_path_disables_current_target(
                project,
                callable.package(),
                &record.module_path,
            );
    }
    false
}

fn item_is_current_target_inactive(project: &Project, item: &ItemId) -> bool {
    let Some(record) = project.items.get(item) else {
        return false;
    };
    attrs_disable_current_target(item_attrs(&record.item))
        || module_path_disables_current_target(project, &item.package, &item.module_path)
}

fn module_path_disables_current_target(
    project: &Project,
    package: &str,
    module_path: &[String],
) -> bool {
    module_path_cfg_gates(project, package, module_path)
        .iter()
        .any(|(_, attribute)| cfg_attribute_is_definitely_false_for_current_target(attribute))
}

fn attrs_disable_current_target(attrs: &[Attribute]) -> bool {
    attrs
        .iter()
        .any(cfg_attribute_is_definitely_false_for_current_target)
}

fn cfg_attribute_is_definitely_false_for_current_target(attribute: &Attribute) -> bool {
    if !attribute.path().is_ident("cfg") {
        return false;
    }
    attribute
        .parse_args::<Meta>()
        .ok()
        .and_then(|meta| cfg_meta_eval_current_target(&meta))
        .is_some_and(|active| !active)
}

fn cfg_meta_eval_current_target(meta: &Meta) -> Option<bool> {
    match meta {
        Meta::Path(path) => cfg_path_eval_current_target(path),
        Meta::NameValue(name_value) => {
            let key = name_value.path.segments.last()?.ident.to_string();
            let value = expr_string_literal(&name_value.value)?;
            cfg_key_value_eval_current_target(&key, &value)
        }
        Meta::List(list) => {
            let key = list.path.segments.last()?.ident.to_string();
            let args = Punctuated::<Meta, syn::Token![,]>::parse_terminated
                .parse2(list.tokens.clone())
                .ok()?
                .into_iter()
                .collect::<Vec<_>>();
            match key.as_str() {
                "all" => {
                    let mut has_unknown = false;
                    for arg in &args {
                        match cfg_meta_eval_current_target(arg) {
                            Some(true) => {}
                            Some(false) => return Some(false),
                            None => has_unknown = true,
                        }
                    }
                    (!has_unknown).then_some(true)
                }
                "any" => {
                    let mut has_unknown = false;
                    for arg in &args {
                        match cfg_meta_eval_current_target(arg) {
                            Some(true) => return Some(true),
                            Some(false) => {}
                            None => has_unknown = true,
                        }
                    }
                    (!has_unknown).then_some(false)
                }
                "not" if args.len() == 1 => cfg_meta_eval_current_target(&args[0]).map(|v| !v),
                _ => None,
            }
        }
    }
}

fn cfg_path_eval_current_target(path: &syn::Path) -> Option<bool> {
    let key = path.segments.last()?.ident.to_string();
    match key.as_str() {
        "test" => Some(false),
        "unix" => Some(cfg!(unix)),
        "windows" => Some(cfg!(windows)),
        "debug_assertions" => Some(cfg!(debug_assertions)),
        _ => None,
    }
}

fn cfg_key_value_eval_current_target(key: &str, value: &str) -> Option<bool> {
    match key {
        "target_arch" => Some(value == std::env::consts::ARCH),
        "target_family" => Some(value == std::env::consts::FAMILY),
        "target_os" => Some(value == std::env::consts::OS),
        "target_pointer_width" => Some(value == (std::mem::size_of::<usize>() * 8).to_string()),
        _ => None,
    }
}

fn expr_string_literal(expr: &Expr) -> Option<String> {
    let Expr::Lit(expr_lit) = expr else {
        return None;
    };
    let Lit::Str(lit) = &expr_lit.lit else {
        return None;
    };
    Some(lit.value())
}

struct UsageEvidenceInput<'a> {
    used_callables: &'a [CallableId],
    blocked_by_unknown_callables: &'a [CallableId],
    prunable_callables: &'a [CallableId],
    used_items: &'a [ItemId],
    blocked_by_unknown_items: &'a [ItemId],
    prunable_items: &'a [ItemId],
}

fn usage_classification_evidence(
    project: &model::Project,
    reduced: &model::ReducedProject,
    input: &UsageEvidenceInput<'_>,
) -> UsageClassificationEvidence {
    let blocked_surface_count =
        input.blocked_by_unknown_callables.len() + input.blocked_by_unknown_items.len();
    let selected_callable_roots = reduced
        .roots
        .iter()
        .filter_map(|root| match root {
            RootId::Callable(callable) => Some(callable),
            RootId::Item(_) => None,
        })
        .collect::<BTreeSet<_>>();
    let selected_item_roots = reduced
        .roots
        .iter()
        .filter_map(|root| match root {
            RootId::Callable(_) => None,
            RootId::Item(item) => Some(item),
        })
        .collect::<BTreeSet<_>>();

    let mut callables = input
        .used_callables
        .iter()
        .map(|id| {
            usage_callable_evidence(
                id,
                "used",
                selected_callable_roots.contains(id),
                &reduced.evidence,
                0,
            )
        })
        .chain(input.blocked_by_unknown_callables.iter().map(|id| {
            usage_callable_evidence(
                id,
                "blocked_by_unknown",
                false,
                &reduced.evidence,
                blocked_surface_count,
            )
        }))
        .chain(
            input
                .prunable_callables
                .iter()
                .map(|id| usage_callable_evidence(id, "prunable", false, &reduced.evidence, 0)),
        )
        .collect::<Vec<_>>();
    callables.sort_by_key(|entry| entry.id.to_string());

    let mut items = input
        .used_items
        .iter()
        .map(|id| {
            usage_item_evidence(
                id,
                "used",
                selected_item_roots.contains(id),
                &reduced.evidence,
                0,
            )
        })
        .chain(input.blocked_by_unknown_items.iter().map(|id| {
            usage_item_evidence(
                id,
                "blocked_by_unknown",
                false,
                &reduced.evidence,
                blocked_surface_count,
            )
        }))
        .chain(
            input
                .prunable_items
                .iter()
                .map(|id| usage_item_evidence(id, "prunable", false, &reduced.evidence, 0)),
        )
        .collect::<Vec<_>>();
    items.sort_by_key(|entry| entry.id.to_string());

    debug_assert_eq!(
        callables.len(),
        project.functions.len() + project.methods.len()
    );
    debug_assert_eq!(items.len(), project.items.len());

    UsageClassificationEvidence { callables, items }
}

fn usage_callable_evidence(
    id: &CallableId,
    classification: &str,
    selected_root: bool,
    evidence: &model::ReductionEvidence,
    unknown_surfaces: usize,
) -> UsageCallableEvidence {
    let (reason, details) =
        usage_evidence_reason(classification, selected_root, evidence, unknown_surfaces);
    UsageCallableEvidence {
        id: id.clone(),
        classification: classification.to_string(),
        selected_root,
        reason,
        evidence: details,
    }
}

fn usage_item_evidence(
    id: &ItemId,
    classification: &str,
    selected_root: bool,
    evidence: &model::ReductionEvidence,
    unknown_surfaces: usize,
) -> UsageItemEvidence {
    let (reason, details) =
        usage_evidence_reason(classification, selected_root, evidence, unknown_surfaces);
    UsageItemEvidence {
        id: id.clone(),
        classification: classification.to_string(),
        selected_root,
        reason,
        evidence: details,
    }
}

fn usage_evidence_reason(
    classification: &str,
    selected_root: bool,
    evidence: &model::ReductionEvidence,
    unknown_surfaces: usize,
) -> (String, Vec<String>) {
    let mut details = vec![
        "source=syn_inventory".to_string(),
        format!("classification={classification}"),
    ];

    if selected_root {
        details.push("selected_root=true".to_string());
        return (
            "selected opensourced root retained as a slice entrypoint".to_string(),
            details,
        );
    }

    match classification {
        "used" => {
            details.push("reachable=true".to_string());
            if evidence.semantic_edges_applied > 0 {
                details.push(format!(
                    "semantic_edges_applied={}",
                    evidence.semantic_edges_applied
                ));
            }
            if evidence.unresolved_method_fallbacks > 0 {
                details.push(format!(
                    "syntactic_method_fallbacks={}",
                    evidence.unresolved_method_fallbacks
                ));
            }
            (
                "reachable from selected roots through the current syntactic and semantic reduction graph"
                    .to_string(),
                details,
            )
        }
        "blocked_by_unknown" => {
            details.push("reachable=false".to_string());
            details.push("unused_candidate=true".to_string());
            details.push("prunable=false".to_string());
            details.push(format!("unknown_surfaces={unknown_surfaces}"));
            (
                "indexed by syn inventory and absent from the reachable graph, but retained unknown surfaces may still reference it"
                    .to_string(),
                details,
            )
        }
        "prunable" => {
            details.push("reachable=false".to_string());
            details.push("unused_candidate=true".to_string());
            details.push("prunable=true".to_string());
            details.push("unknown_surfaces=0".to_string());
            (
                "indexed by syn inventory, absent from the reachable graph, and not blocked by retained unknown surfaces"
                    .to_string(),
                details,
            )
        }
        _ => (
            "classification produced by the current reduction graph".to_string(),
            details,
        ),
    }
}

fn production_readiness_report(
    analyzer: &AnalyzerReport,
    project: &Project,
    reduced: &ReducedProject,
    output_root: &Path,
    semantic_proof: Option<&SemanticUsageProofReport>,
    rendered_symbol_proof: Option<&RenderedSymbolProofReport>,
    public_reexport_proof: Option<&PublicReexportProofReport>,
) -> ProductionReadinessReport {
    production_readiness_report_inner(
        analyzer,
        project,
        reduced,
        Some(output_root),
        semantic_proof,
        rendered_symbol_proof,
        public_reexport_proof,
    )
}

fn pre_render_production_readiness_report(
    analyzer: &AnalyzerReport,
    project: &Project,
    reduced: &ReducedProject,
) -> ProductionReadinessReport {
    production_readiness_report_inner(analyzer, project, reduced, None, None, None, None)
}

fn production_readiness_report_inner(
    analyzer: &AnalyzerReport,
    project: &Project,
    reduced: &ReducedProject,
    output_root: Option<&Path>,
    semantic_proof: Option<&SemanticUsageProofReport>,
    rendered_symbol_proof: Option<&RenderedSymbolProofReport>,
    public_reexport_proof: Option<&PublicReexportProofReport>,
) -> ProductionReadinessReport {
    let mut hazards = Vec::new();
    let semantic_pruning_proven = semantic_pruning_proof_complete(semantic_proof);
    add_workspace_production_hazards(project, reduced, &mut hazards);
    if let Some(output_root) = output_root {
        add_generated_support_package_production_hazards(output_root, &mut hazards);
    }
    if let Some(rendered_symbol_proof) = rendered_symbol_proof {
        add_rendered_symbol_proof_hazards(rendered_symbol_proof, &mut hazards);
    }
    if let Some(public_reexport_proof) = public_reexport_proof {
        add_public_reexport_proof_hazards(public_reexport_proof, &mut hazards);
    }
    add_cfg_gated_root_production_hazards(project, reduced, &mut hazards);
    add_syntactic_production_hazards(project, reduced, &mut hazards);
    add_reduction_evidence_production_hazards(
        project,
        reduced,
        semantic_pruning_proven,
        &mut hazards,
    );

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
        semantic_pruning_proven,
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
    add_semantic_query_hazards(
        semantic,
        project,
        reduced,
        rendered_symbol_proof,
        &mut hazards,
    );
    add_semantic_usage_mapping_hazard(analyzer, project, reduced, &mut hazards);
    add_semantic_usage_reference_hazard(analyzer, project, semantic_proof, &mut hazards);

    production_readiness_status(hazards)
}

fn semantic_pruning_proof_complete(proof: Option<&SemanticUsageProofReport>) -> bool {
    proof.is_some_and(|proof| {
        proof.status == "complete_for_retained_packages"
            && proof.summary.analyzer_available
            && proof.summary.unproven_callables == 0
            && proof.summary.unproven_items == 0
    })
}

fn add_semantic_usage_mapping_hazard(
    analyzer: &AnalyzerReport,
    project: &Project,
    reduced: &ReducedProject,
    hazards: &mut Vec<ProductionHazardReport>,
) {
    let Some(usage) = &analyzer.semantic_usage else {
        return;
    };

    let mut details = Vec::new();
    for callable in &reduced.reachable {
        if usage.is_callable_mapped(callable) {
            continue;
        }
        if let Some(record) = project.functions.get(callable) {
            details.push(ProductionHazardDetail {
                subject: callable.to_string(),
                package: Some(callable.package().to_string()),
                module_path: callable_detail_module_path(callable),
                file: Some(record.span.file.clone()),
                start_line: Some(record.span.start_line),
                cfg: None,
                blocked_idents: Vec::new(),
                suggested_cargo_args: Vec::new(),
            });
        } else if let Some(record) = project.methods.get(callable) {
            details.push(ProductionHazardDetail {
                subject: callable.to_string(),
                package: Some(callable.package().to_string()),
                module_path: callable_detail_module_path(callable),
                file: Some(record.span.file.clone()),
                start_line: Some(record.span.start_line),
                cfg: None,
                blocked_idents: Vec::new(),
                suggested_cargo_args: Vec::new(),
            });
        }
    }
    for item in &reduced.reachable_items {
        if usage.is_item_mapped(item)
            || matches!(item.kind, model::ItemKind::Mod | model::ItemKind::Macro)
        {
            continue;
        }
        if let Some(record) = project.items.get(item) {
            details.push(ProductionHazardDetail {
                subject: item.to_string(),
                package: Some(item.package.clone()),
                module_path: Some(item.module_path.join("::")),
                file: Some(record.span.file.clone()),
                start_line: Some(record.span.start_line),
                cfg: None,
                blocked_idents: Vec::new(),
                suggested_cargo_args: Vec::new(),
            });
        }
    }

    if details.is_empty() {
        return;
    }
    hazards.push(production_hazard_with_details(
        "semantic_usage_mapping_incomplete",
        "warning",
        "retained source contains items or callables that were not mapped to rust-analyzer definitions; compiler feedback is required before treating removal proof as complete",
        details,
    ));
}

fn add_semantic_usage_reference_hazard(
    analyzer: &AnalyzerReport,
    project: &Project,
    semantic_proof: Option<&SemanticUsageProofReport>,
    hazards: &mut Vec<ProductionHazardReport>,
) {
    let Some(usage) = &analyzer.semantic_usage else {
        return;
    };
    if usage.reference_queries_skipped > 0 && !semantic_pruning_proof_complete(semantic_proof) {
        let (debt, details) = semantic_usage_skipped_reference_debt(project, usage, semantic_proof);
        if debt > 0 {
            hazards.push(production_hazard_with_details(
                "semantic_usage_reference_skipped",
                "warning",
                semantic_usage_skipped_reference_message(debt, usage.reference_queries_skipped),
                details,
            ));
        }
    }
    if usage.reference_query_failures == 0 {
        return;
    }

    let mut details = Vec::new();
    for callable in &usage.failed_callable_reference_ids {
        if let Some(record) = project.functions.get(callable) {
            details.push(ProductionHazardDetail {
                subject: callable.to_string(),
                package: Some(callable.package().to_string()),
                module_path: callable_detail_module_path(callable),
                file: Some(record.span.file.clone()),
                start_line: Some(record.span.start_line),
                cfg: None,
                blocked_idents: Vec::new(),
                suggested_cargo_args: Vec::new(),
            });
        } else if let Some(record) = project.methods.get(callable) {
            details.push(ProductionHazardDetail {
                subject: callable.to_string(),
                package: Some(callable.package().to_string()),
                module_path: callable_detail_module_path(callable),
                file: Some(record.span.file.clone()),
                start_line: Some(record.span.start_line),
                cfg: None,
                blocked_idents: Vec::new(),
                suggested_cargo_args: Vec::new(),
            });
        }
    }
    for item in &usage.failed_item_reference_ids {
        if let Some(record) = project.items.get(item) {
            details.push(ProductionHazardDetail {
                subject: item.to_string(),
                package: Some(item.package.clone()),
                module_path: Some(item.module_path.join("::")),
                file: Some(record.span.file.clone()),
                start_line: Some(record.span.start_line),
                cfg: None,
                blocked_idents: Vec::new(),
                suggested_cargo_args: Vec::new(),
            });
        }
    }

    hazards.push(production_hazard_with_details(
        "semantic_usage_reference_incomplete",
        "warning",
        "rust-analyzer reference search failed for one or more mapped items; those candidates are retained as unknown until compiler feedback or a later semantic pass proves they are removable",
        details,
    ));
}

fn semantic_usage_skipped_reference_debt(
    project: &Project,
    usage: &SemanticUsageReport,
    semantic_proof: Option<&SemanticUsageProofReport>,
) -> (usize, Vec<ProductionHazardDetail>) {
    let Some(proof) = semantic_proof else {
        return (usage.reference_queries_skipped, Vec::new());
    };
    let debt = proof.summary.skipped_reference_query_callables
        + proof.summary.skipped_reference_query_items;
    let mut details = Vec::new();
    for callable in &proof.unproven.callables {
        if !usage.callable_reference_query_skipped(callable) {
            continue;
        }
        if let Some(detail) = callable_production_hazard_detail(project, callable) {
            details.push(detail);
        }
    }
    for item in &proof.unproven.items {
        if !usage.item_reference_query_skipped(item) {
            continue;
        }
        if let Some(detail) = item_production_hazard_detail(project, item) {
            details.push(detail);
        }
    }
    (debt, details)
}

fn semantic_usage_skipped_reference_message(debt: usize, skipped_total: usize) -> String {
    let total_context = if skipped_total == debt {
        String::new()
    } else {
        format!(" ({skipped_total} total indexed query/queries skipped)")
    };
    format!(
        "rust-analyzer reference search skipped {debt} retained-package callable/item pruning proof query/queries{total_context}; increase OPENSOURCE_RA_REFERENCE_QUERY_BUDGET or pass --ra-reference-budget to run a deeper retained-package pruning proof"
    )
}

fn callable_production_hazard_detail(
    project: &Project,
    callable: &CallableId,
) -> Option<ProductionHazardDetail> {
    if let Some(record) = project.functions.get(callable) {
        Some(ProductionHazardDetail {
            subject: callable.to_string(),
            package: Some(callable.package().to_string()),
            module_path: callable_detail_module_path(callable),
            file: Some(record.span.file.clone()),
            start_line: Some(record.span.start_line),
            cfg: None,
            blocked_idents: Vec::new(),
            suggested_cargo_args: Vec::new(),
        })
    } else if let Some(record) = project.methods.get(callable) {
        Some(ProductionHazardDetail {
            subject: callable.to_string(),
            package: Some(callable.package().to_string()),
            module_path: callable_detail_module_path(callable),
            file: Some(record.span.file.clone()),
            start_line: Some(record.span.start_line),
            cfg: None,
            blocked_idents: Vec::new(),
            suggested_cargo_args: Vec::new(),
        })
    } else {
        None
    }
}

fn item_production_hazard_detail(
    project: &Project,
    item: &ItemId,
) -> Option<ProductionHazardDetail> {
    project
        .items
        .get(item)
        .map(|record| ProductionHazardDetail {
            subject: item.to_string(),
            package: Some(item.package.clone()),
            module_path: Some(item.module_path.join("::")),
            file: Some(record.span.file.clone()),
            start_line: Some(record.span.start_line),
            cfg: None,
            blocked_idents: Vec::new(),
            suggested_cargo_args: Vec::new(),
        })
}

fn callable_detail_module_path(callable: &CallableId) -> Option<String> {
    match callable {
        CallableId::Free { module_path, .. } => Some(module_path.join("::")),
        CallableId::Method { type_path, .. } => {
            let module_segments = type_path
                .iter()
                .take(type_path.len().saturating_sub(1))
                .cloned()
                .collect::<Vec<_>>();
            Some(module_segments.join("::"))
        }
    }
    .filter(|module_path| !module_path.is_empty())
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum SemanticHazardScope {
    RetainedSlice,
    SelectedRoots,
    Workspace,
}

impl SemanticHazardScope {
    fn source_prefix(self) -> &'static str {
        match self {
            Self::RetainedSlice => "retained slice ",
            Self::SelectedRoots => "selected-root ",
            Self::Workspace => "",
        }
    }

    fn query_prefix(self) -> &'static str {
        match self {
            Self::RetainedSlice => "retained-slice ",
            Self::SelectedRoots => "selected-root ",
            Self::Workspace => "",
        }
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
struct SemanticHazardMetrics {
    source_files: usize,
    failed_files: usize,
    skipped_files: usize,
    unresolved_method_calls: usize,
    unqueried_method_calls: usize,
    unresolved_paths: usize,
    unqueried_paths: usize,
    unresolved_diagnostics: Vec<SemanticUnresolvedDiagnostic>,
}

fn add_semantic_query_hazards(
    semantic: &SemanticReport,
    project: &Project,
    reduced: &ReducedProject,
    rendered_symbol_proof: Option<&RenderedSymbolProofReport>,
    hazards: &mut Vec<ProductionHazardReport>,
) {
    let retained_paths = retained_semantic_file_paths(project, reduced);
    let (scope, metrics) = semantic_hazard_metrics(semantic, &retained_paths);

    if metrics.failed_files > 0 {
        hazards.push(production_hazard(
            "semantic_file_failures",
            "warning",
            format!(
                "{} {}source file(s) failed semantic analysis",
                metrics.failed_files,
                scope.source_prefix()
            ),
        ));
    }
    if metrics.skipped_files > 0 {
        hazards.push(production_hazard(
            "semantic_file_budget_exhausted",
            "warning",
            format!(
                "{} {}source file(s) were skipped by semantic analysis budget",
                metrics.skipped_files,
                scope.source_prefix()
            ),
        ));
    }
    let raw_unresolved_method_diagnostics =
        unresolved_diagnostics_for_kind(&metrics, SemanticUnresolvedKind::MethodCall);
    let unresolved_method_diagnostics = raw_unresolved_method_diagnostics
        .iter()
        .copied()
        .filter(|diagnostic| semantic_unresolved_owner_is_retained(reduced, diagnostic))
        .filter(|diagnostic| {
            !covered_project_method_unresolved_diagnostic(
                project,
                reduced,
                &retained_paths,
                diagnostic,
            )
        })
        .collect::<Vec<_>>();
    let unresolved_method_calls = if raw_unresolved_method_diagnostics.is_empty() {
        metrics.unresolved_method_calls
    } else {
        unresolved_method_diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.category == SemanticUnresolvedCategory::DependencyRisk)
            .count()
    };
    if unresolved_method_calls > 0 {
        hazards.push(production_hazard(
            "semantic_unresolved_method_calls",
            "warning",
            format!(
                "{} queried {}method call(s) did not resolve semantically",
                unresolved_method_calls,
                scope.query_prefix()
            ),
        ));
        let last = hazards
            .last_mut()
            .expect("semantic unresolved method hazard was just pushed");
        last.details = semantic_unresolved_details(unresolved_method_diagnostics);
    }
    let raw_unresolved_path_diagnostics =
        unresolved_diagnostics_for_kind(&metrics, SemanticUnresolvedKind::Path);
    let unresolved_path_diagnostics = raw_unresolved_path_diagnostics
        .iter()
        .copied()
        .filter(|diagnostic| semantic_unresolved_owner_is_retained(reduced, diagnostic))
        .filter(|diagnostic| {
            !covered_project_path_unresolved_diagnostic(project, rendered_symbol_proof, diagnostic)
        })
        .collect::<Vec<_>>();
    let unresolved_paths = if raw_unresolved_path_diagnostics.is_empty() {
        metrics.unresolved_paths
    } else {
        unresolved_path_diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.category == SemanticUnresolvedCategory::DependencyRisk)
            .count()
    };
    if unresolved_paths > 0 {
        hazards.push(production_hazard(
            "semantic_unresolved_paths",
            "warning",
            format!(
                "{} queried {}path(s) did not resolve semantically",
                unresolved_paths,
                scope.query_prefix()
            ),
        ));
        let last = hazards
            .last_mut()
            .expect("semantic unresolved path hazard was just pushed");
        last.details = semantic_unresolved_details(unresolved_path_diagnostics);
    }
    if metrics.unqueried_method_calls > 0 {
        hazards.push(production_hazard(
            "semantic_method_call_budget_exhausted",
            "warning",
            format!(
                "{} {}method call(s) were not queried because the semantic budget was exhausted",
                metrics.unqueried_method_calls,
                scope.query_prefix()
            ),
        ));
    }
    if metrics.unqueried_paths > 0 {
        hazards.push(production_hazard(
            "semantic_path_budget_exhausted",
            "warning",
            format!(
                "{} {}path(s) were not queried because the semantic budget was exhausted",
                metrics.unqueried_paths,
                scope.query_prefix()
            ),
        ));
    }
}

fn unresolved_diagnostics_for_kind(
    metrics: &SemanticHazardMetrics,
    kind: SemanticUnresolvedKind,
) -> Vec<&SemanticUnresolvedDiagnostic> {
    metrics
        .unresolved_diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.kind == kind)
        .collect()
}

fn semantic_unresolved_owner_is_retained(
    reduced: &ReducedProject,
    diagnostic: &SemanticUnresolvedDiagnostic,
) -> bool {
    match &diagnostic.owner {
        None => true,
        Some(SemanticOwnerId::Callable(callable)) => {
            reduced.reachable.contains(callable)
                || reduced
                    .roots
                    .iter()
                    .any(|root| matches!(root, RootId::Callable(root) if root == callable))
        }
        Some(SemanticOwnerId::Item(item)) => {
            reduced.reachable_items.contains(item)
                || reduced
                    .roots
                    .iter()
                    .any(|root| matches!(root, RootId::Item(root) if root == item))
        }
    }
}

#[cfg(test)]
fn non_benign_unresolved_count(
    fallback_count: usize,
    diagnostics: &[&SemanticUnresolvedDiagnostic],
) -> usize {
    if diagnostics.is_empty() {
        return fallback_count;
    }
    diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.category == SemanticUnresolvedCategory::DependencyRisk)
        .count()
}

fn semantic_unresolved_details(
    diagnostics: Vec<&SemanticUnresolvedDiagnostic>,
) -> Vec<ProductionHazardDetail> {
    diagnostics
        .into_iter()
        .filter(|diagnostic| diagnostic.category == SemanticUnresolvedCategory::DependencyRisk)
        .map(|diagnostic| ProductionHazardDetail {
            subject: format!(
                "category={}; kind={}; reason={}; owner={}; symbol={}; snippet={}",
                diagnostic.category.as_str(),
                diagnostic.kind.as_str(),
                diagnostic.reason,
                diagnostic
                    .owner
                    .as_ref()
                    .map(semantic_owner_to_string)
                    .unwrap_or_else(|| "none".to_string()),
                diagnostic.symbol.as_deref().unwrap_or("none"),
                diagnostic.snippet
            ),
            package: None,
            module_path: None,
            file: Some(diagnostic.file.clone()),
            start_line: Some(diagnostic.start_line),
            cfg: None,
            blocked_idents: diagnostic.symbol.iter().cloned().collect(),
            suggested_cargo_args: Vec::new(),
        })
        .collect()
}

fn covered_project_method_unresolved_diagnostic(
    project: &Project,
    reduced: &ReducedProject,
    retained_paths: &BTreeSet<PathBuf>,
    diagnostic: &SemanticUnresolvedDiagnostic,
) -> bool {
    diagnostic.kind == SemanticUnresolvedKind::MethodCall
        && diagnostic.category == SemanticUnresolvedCategory::DependencyRisk
        && diagnostic.reason == "unresolved_method_name_matches_project_method"
        && diagnostic.symbol.as_deref().is_some_and(|method| {
            project_method_name_fully_reachable_in_retained_paths(
                project,
                reduced,
                retained_paths,
                method,
            )
        })
}

fn covered_project_path_unresolved_diagnostic(
    project: &Project,
    rendered_symbol_proof: Option<&RenderedSymbolProofReport>,
    diagnostic: &SemanticUnresolvedDiagnostic,
) -> bool {
    diagnostic.kind == SemanticUnresolvedKind::Path
        && diagnostic.category == SemanticUnresolvedCategory::DependencyRisk
        && diagnostic.reason == "unresolved_path_anchor_matches_project_local_identifier"
        && rendered_symbol_proof.is_some_and(|proof| {
            proof.status == "proven"
                && (covered_project_associated_callable_path(project, proof, diagnostic)
                    || covered_project_enum_variant_path(project, proof, diagnostic)
                    || covered_project_item_path(project, proof, diagnostic)
                    || covered_build_generated_path(project, proof, diagnostic))
        })
}

fn covered_project_associated_callable_path(
    project: &Project,
    proof: &RenderedSymbolProofReport,
    diagnostic: &SemanticUnresolvedDiagnostic,
) -> bool {
    let Some(symbol) = diagnostic.symbol.as_deref() else {
        return false;
    };
    let Some((qualifier, member)) = unresolved_associated_path_segments(diagnostic) else {
        return false;
    };
    if member != symbol {
        return false;
    }

    project.methods.keys().any(|callable| {
        let CallableId::Method {
            type_path, method, ..
        } = callable
        else {
            return false;
        };
        method == symbol
            && path_qualifier_matches_type_path(&qualifier, type_path)
            && rendered_symbol_is_used_or_unknown(proof, "callable", &callable.to_string())
    })
}

fn covered_project_enum_variant_path(
    project: &Project,
    proof: &RenderedSymbolProofReport,
    diagnostic: &SemanticUnresolvedDiagnostic,
) -> bool {
    let Some(symbol) = diagnostic.symbol.as_deref() else {
        return false;
    };
    let Some((qualifier, member)) = unresolved_associated_path_segments(diagnostic) else {
        return false;
    };
    if member != symbol {
        return false;
    }

    project.items.keys().any(|item| {
        item.kind == model::ItemKind::Enum
            && path_qualifier_matches_item_path(&qualifier, item)
            && rendered_symbol_is_used_or_unknown(proof, "member", &format!("{item}::{symbol}"))
    })
}

fn covered_project_item_path(
    project: &Project,
    proof: &RenderedSymbolProofReport,
    diagnostic: &SemanticUnresolvedDiagnostic,
) -> bool {
    let Some(symbol) = diagnostic.symbol.as_deref() else {
        return false;
    };
    let segments = syn_path_segments(&diagnostic.snippet);
    if segments.last().is_none_or(|segment| segment != symbol) {
        return false;
    }
    let owner_package = diagnostic_owner_package(diagnostic);

    let mut matches = project.items.keys().filter(|item| {
        item.name == symbol
            && rendered_symbol_is_used_or_unknown(proof, "item", &item.to_string())
            && unresolved_path_segments_can_target_item(project, owner_package, &segments, item)
    });
    let Some(_first) = matches.next() else {
        return false;
    };
    matches.next().is_none()
}

fn diagnostic_owner_package(diagnostic: &SemanticUnresolvedDiagnostic) -> Option<&str> {
    match diagnostic.owner.as_ref()? {
        SemanticOwnerId::Callable(callable) => Some(callable.package()),
        SemanticOwnerId::Item(item) => Some(item.package()),
    }
}

fn unresolved_path_segments_can_target_item(
    project: &Project,
    owner_package: Option<&str>,
    segments: &[String],
    item: &ItemId,
) -> bool {
    if segments.len() == 1 {
        return true;
    }

    let Some((target_package, module_path)) =
        unresolved_project_path_module(project, owner_package, &segments[..segments.len() - 1])
    else {
        return false;
    };
    if target_package != item.package {
        return false;
    }

    let mut item_path = item.module_path.clone();
    item_path.push(item.name.clone());
    let mut direct_path = module_path.clone();
    direct_path.push(item.name.clone());
    if direct_path == item_path {
        return true;
    }

    project
        .module_aliases
        .get(&(target_package, module_path.clone()))
        .and_then(|aliases| aliases.get(&item.name))
        .is_some_and(|target| local_alias_target_path(&module_path, target) == item_path)
}

fn unresolved_project_path_module(
    project: &Project,
    owner_package: Option<&str>,
    prefix: &[String],
) -> Option<(String, Vec<String>)> {
    let first = prefix.first()?;
    match first.as_str() {
        "crate" | "self" => {
            let owner_package = owner_package?;
            Some((owner_package.to_string(), prefix[1..].to_vec()))
        }
        _ => {
            if let Some(owner_package) = owner_package {
                if let Some(package) = project.workspace.packages.get(owner_package) {
                    if let Some(dependency) = package
                        .dependencies
                        .iter()
                        .find(|dependency| dependency_name_matches(dependency, first))
                    {
                        return Some((dependency.package.clone(), prefix[1..].to_vec()));
                    }
                }
            }

            project
                .workspace
                .packages
                .keys()
                .find(|package| package_name_matches_path_root(package, first))
                .map(|package| (package.clone(), prefix[1..].to_vec()))
        }
    }
}

fn local_alias_target_path(module_path: &[String], target: &[String]) -> Vec<String> {
    let Some((first, rest)) = target.split_first() else {
        return module_path.to_vec();
    };
    match first.as_str() {
        "crate" => rest.to_vec(),
        "self" => module_path
            .iter()
            .cloned()
            .chain(rest.iter().cloned())
            .collect(),
        "super" => {
            let mut path = module_path.to_vec();
            path.pop();
            path.extend(rest.iter().cloned());
            path
        }
        _ => module_path
            .iter()
            .cloned()
            .chain(target.iter().cloned())
            .collect(),
    }
}

fn dependency_name_matches(dependency: &manifest::Dependency, name: &str) -> bool {
    dependency.alias == name
        || dependency.package == name
        || dependency_code_name(&dependency.alias) == name
        || dependency_code_name(&dependency.package) == name
}

fn package_name_matches_path_root(package: &str, root: &str) -> bool {
    package == root || dependency_code_name(package) == root
}

fn dependency_code_name(alias: &str) -> String {
    alias.replace('-', "_")
}

fn covered_build_generated_path(
    project: &Project,
    proof: &RenderedSymbolProofReport,
    diagnostic: &SemanticUnresolvedDiagnostic,
) -> bool {
    let Some((qualifier, _member)) = unresolved_associated_path_segments(diagnostic) else {
        return false;
    };

    project.items.iter().any(|(item, record)| {
        item.kind == model::ItemKind::Mod
            && rendered_symbol_is_used_or_unknown(proof, "item", &item.to_string())
            && path_qualifier_matches_item_path(&qualifier, item)
            && module_item_contains_out_dir_source_include(project, item, record)
    })
}

fn module_item_contains_out_dir_source_include(
    project: &Project,
    item: &ItemId,
    record: &model::ItemRecord,
) -> bool {
    item_contains_out_dir_source_include(&record.item)
        || project
            .source_files_by_module
            .get(&(
                item.package.clone(),
                item.module_path
                    .iter()
                    .cloned()
                    .chain(std::iter::once(item.name.clone()))
                    .collect::<Vec<_>>(),
            ))
            .and_then(|path| project.files.get(path))
            .is_some_and(|source| {
                source
                    .syntax
                    .items
                    .iter()
                    .any(item_contains_out_dir_source_include)
            })
}

fn item_contains_out_dir_source_include(item: &Item) -> bool {
    match item {
        Item::Macro(item_macro) => {
            item_macro.ident.is_none()
                && item_macro.mac.path.is_ident("include")
                && macro_tokens_reference_out_dir(&item_macro.mac.tokens)
        }
        Item::Mod(item_mod) => item_mod
            .content
            .as_ref()
            .is_some_and(|(_, items)| items.iter().any(item_contains_out_dir_source_include)),
        _ => false,
    }
}

fn unresolved_associated_path_segments(
    diagnostic: &SemanticUnresolvedDiagnostic,
) -> Option<(Vec<String>, String)> {
    let mut segments = syn_path_segments(&diagnostic.snippet);
    if segments.len() < 2 {
        return None;
    }
    let member = segments.pop()?;
    Some((segments, member))
}

fn syn_path_segments(snippet: &str) -> Vec<String> {
    syn::parse_str::<syn::Path>(snippet)
        .ok()
        .map(|path| {
            path.segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn path_qualifier_matches_type_path(qualifier: &[String], type_path: &[String]) -> bool {
    qualifier != ["Self"] && path_segments_have_matching_suffix(qualifier, type_path)
}

fn path_qualifier_matches_item_path(qualifier: &[String], item: &ItemId) -> bool {
    qualifier != ["Self"]
        && path_segments_have_matching_suffix(
            qualifier,
            &item
                .module_path
                .iter()
                .cloned()
                .chain(std::iter::once(item.name.clone()))
                .collect::<Vec<_>>(),
        )
}

fn path_segments_have_matching_suffix(left: &[String], right: &[String]) -> bool {
    !left.is_empty() && !right.is_empty() && (left.ends_with(right) || right.ends_with(left))
}

fn rendered_symbol_is_used_or_unknown(
    proof: &RenderedSymbolProofReport,
    kind: &str,
    id: &str,
) -> bool {
    proof.entries.iter().any(|entry| {
        entry.kind == kind
            && entry.id == id
            && matches!(
                entry.classification.as_str(),
                "retained" | "blocked_by_unknown"
            )
    })
}

fn project_method_name_fully_reachable_in_retained_paths(
    project: &Project,
    reduced: &ReducedProject,
    retained_paths: &BTreeSet<PathBuf>,
    method_name: &str,
) -> bool {
    let mut found = false;
    for callable in project.methods.keys() {
        let CallableId::Method { method, .. } = callable else {
            continue;
        };
        if method != method_name {
            continue;
        }
        if !method_source_path_is_retained(project, callable, retained_paths) {
            continue;
        }
        found = true;
        if !reduced.reachable.contains(callable)
            && !reduced
                .roots
                .iter()
                .any(|root| matches!(root, RootId::Callable(root) if root == callable))
        {
            return false;
        }
    }
    found
}

fn method_source_path_is_retained(
    project: &Project,
    callable: &CallableId,
    retained_paths: &BTreeSet<PathBuf>,
) -> bool {
    project
        .methods
        .get(callable)
        .is_some_and(|record| retained_paths.contains(&normalize_report_path(&record.span.file)))
}

fn semantic_hazard_metrics(
    semantic: &SemanticReport,
    retained_paths: &BTreeSet<PathBuf>,
) -> (SemanticHazardScope, SemanticHazardMetrics) {
    if !semantic.file_reports.is_empty() && !retained_paths.is_empty() {
        let metrics = semantic_file_metrics_for_paths(semantic, retained_paths);
        if metrics.source_files > 0 {
            return (SemanticHazardScope::RetainedSlice, metrics);
        }
    }

    if semantic.selected_root_source_files > 0 {
        let unresolved_diagnostics = semantic
            .file_reports
            .iter()
            .filter(|file_report| file_report.selected_root_file)
            .flat_map(|file_report| file_report.unresolved_diagnostics.iter().cloned())
            .collect();
        return (
            SemanticHazardScope::SelectedRoots,
            SemanticHazardMetrics {
                source_files: semantic.selected_root_source_files,
                failed_files: semantic.selected_root_failed_files,
                skipped_files: semantic.selected_root_skipped_files,
                unresolved_method_calls: semantic.selected_root_unresolved_method_calls,
                unqueried_method_calls: semantic.selected_root_unqueried_method_calls,
                unresolved_paths: semantic.selected_root_unresolved_paths,
                unqueried_paths: semantic.selected_root_unqueried_paths,
                unresolved_diagnostics,
            },
        );
    }

    (
        SemanticHazardScope::Workspace,
        SemanticHazardMetrics {
            source_files: semantic.source_files,
            failed_files: semantic.failed_files,
            skipped_files: semantic.skipped_files,
            unresolved_method_calls: semantic.unresolved_method_calls,
            unqueried_method_calls: semantic.unqueried_method_calls,
            unresolved_paths: semantic.unresolved_paths,
            unqueried_paths: semantic.unqueried_paths,
            unresolved_diagnostics: semantic.unresolved_diagnostics.clone(),
        },
    )
}

fn semantic_file_metrics_for_paths(
    semantic: &SemanticReport,
    retained_paths: &BTreeSet<PathBuf>,
) -> SemanticHazardMetrics {
    let mut metrics = SemanticHazardMetrics::default();
    for file_report in &semantic.file_reports {
        let path = normalize_report_path(&file_report.path);
        if !retained_paths.contains(&path) {
            continue;
        }
        metrics.source_files += 1;
        metrics.failed_files += usize::from(file_report.failed);
        metrics.skipped_files += usize::from(file_report.skipped_by_file_budget);
        metrics.unresolved_method_calls += file_report.unresolved_method_calls;
        metrics.unqueried_method_calls += file_report.unqueried_method_calls;
        metrics.unresolved_paths += file_report.unresolved_paths;
        metrics.unqueried_paths += file_report.unqueried_paths;
        metrics
            .unresolved_diagnostics
            .extend(file_report.unresolved_diagnostics.iter().cloned());
    }
    metrics
}

fn retained_semantic_file_paths(project: &Project, reduced: &ReducedProject) -> BTreeSet<PathBuf> {
    let mut paths = BTreeSet::new();
    for root in &reduced.roots {
        match root {
            RootId::Callable(callable) => add_callable_source_path(project, callable, &mut paths),
            RootId::Item(item) => add_item_source_path(project, item, &mut paths),
        }
    }
    for callable in &reduced.reachable {
        add_callable_source_path(project, callable, &mut paths);
    }
    for item in &reduced.reachable_items {
        add_item_source_path(project, item, &mut paths);
    }
    paths
}

fn add_callable_source_path(
    project: &Project,
    callable: &CallableId,
    paths: &mut BTreeSet<PathBuf>,
) {
    if let Some(record) = project.functions.get(callable) {
        paths.insert(normalize_report_path(&record.span.file));
    }
    if let Some(record) = project.methods.get(callable) {
        paths.insert(normalize_report_path(&record.span.file));
    }
}

fn add_item_source_path(project: &Project, item: &ItemId, paths: &mut BTreeSet<PathBuf>) {
    if let Some(record) = project.items.get(item) {
        paths.insert(normalize_report_path(&record.span.file));
    }
}

fn normalize_report_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn add_semantic_inventory_hazard(
    analyzer: &AnalyzerReport,
    semantic_edges_applied: usize,
    semantic_pruning_proven: bool,
    hazards: &mut Vec<ProductionHazardReport>,
) {
    if semantic_pruning_proven {
        return;
    }
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

fn add_generated_support_package_production_hazards(
    output_root: &Path,
    hazards: &mut Vec<ProductionHazardReport>,
) {
    let support_root = output_root.join("support");
    let Ok(support_packages) = generated_support_packages(&support_root) else {
        return;
    };
    if support_packages.is_empty() {
        return;
    }

    let retained_build_script_details = support_packages
        .iter()
        .filter_map(generated_support_build_script_detail)
        .collect::<Vec<_>>();
    if !retained_build_script_details.is_empty() {
        hazards.push(production_hazard_with_details(
            "retained_build_scripts",
            "error",
            format!(
                "{} copied support package build script(s) may generate source, link metadata, env values, or asset dependencies outside the static parse tree",
                retained_build_script_details.len()
            ),
            retained_build_script_details,
        ));
    }

    let mut counts = SyntacticHazardCounts::default();
    for package in &support_packages {
        counts.add(generated_support_package_syntactic_hazard_counts(package));
    }
    add_generated_support_syntactic_hazards(counts, hazards);
}

struct GeneratedSupportPackage {
    name: String,
    root: PathBuf,
    manifest: toml::Value,
}

fn generated_support_packages(
    support_root: &Path,
) -> Result<Vec<GeneratedSupportPackage>, Box<dyn std::error::Error>> {
    if !support_root.is_dir() {
        return Ok(Vec::new());
    }
    let mut packages = Vec::new();
    let mut entries = fs::read_dir(support_root)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let root = entry.path();
        let manifest_path = root.join("Cargo.toml");
        if !manifest_path.is_file() {
            continue;
        }
        let text = match fs::read_to_string(&manifest_path) {
            Ok(text) => text,
            Err(_) => continue,
        };
        let Ok(manifest) = text.parse::<toml::Value>() else {
            continue;
        };
        let name = manifest
            .get("package")
            .and_then(|package| package.get("name"))
            .and_then(toml::Value::as_str)
            .map(str::to_string)
            .or_else(|| {
                root.file_name()
                    .and_then(|name| name.to_str())
                    .map(str::to_string)
            })
            .unwrap_or_else(|| "support".to_string());
        packages.push(GeneratedSupportPackage {
            name,
            root,
            manifest,
        });
    }
    Ok(packages)
}

fn generated_support_build_script_detail(
    package: &GeneratedSupportPackage,
) -> Option<ProductionHazardDetail> {
    let path = generated_support_build_script_path(package)?;
    Some(ProductionHazardDetail {
        subject: package.name.clone(),
        package: Some(package.name.clone()),
        module_path: None,
        file: Some(path),
        start_line: Some(1),
        cfg: None,
        blocked_idents: Vec::new(),
        suggested_cargo_args: Vec::new(),
    })
}

fn generated_support_build_script_path(package: &GeneratedSupportPackage) -> Option<PathBuf> {
    match package
        .manifest
        .get("package")
        .and_then(toml::Value::as_table)
        .and_then(|table| table.get("build"))
    {
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

fn generated_support_package_syntactic_hazard_counts(
    package: &GeneratedSupportPackage,
) -> SyntacticHazardCounts {
    let mut counts = SyntacticHazardCounts::default();
    let Ok(rust_files) = generated_support_rust_files(&package.root) else {
        return counts;
    };
    for path in rust_files {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(syntax) = syn::parse_file(&text) else {
            continue;
        };
        let mut visitor = SyntacticHazardVisitor {
            project: None,
            counts: SyntacticHazardCounts::default(),
            include_context: path.parent().map(|source_dir| IncludeContext {
                package_root: package.root.clone(),
                source_dir: source_dir.to_path_buf(),
                build_script_path: generated_support_build_script_path(package),
            }),
            location: HazardLocation {
                package: package.name.clone(),
                module_path: Vec::new(),
                file: Some(path),
                owner: None,
            },
            macro_context: MacroInvocationContext::for_syntax(
                &syntax,
                &BTreeSet::new(),
                &BTreeSet::new(),
                &BTreeSet::new(),
                &BTreeSet::new(),
                &BTreeSet::new(),
            ),
            proven_trait_object_surfaces: BTreeSet::new(),
            allow_rust_default_variant_attribute: 0,
        };
        visitor.visit_file(&syntax);
        counts.add(visitor.counts.support_package_blocking_subset());
    }
    counts
}

fn generated_support_rust_files(root: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let root = root.canonicalize()?;
    let mut files = Vec::new();
    collect_generated_support_rust_files(&root, &root, &mut BTreeSet::new(), &mut files)?;
    Ok(files)
}

fn collect_generated_support_rust_files(
    root: &Path,
    path: &Path,
    visited: &mut BTreeSet<PathBuf>,
    files: &mut Vec<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Ok(());
    }
    let resolved = path.canonicalize()?;
    if !resolved.starts_with(root) {
        return Ok(());
    }
    if metadata.is_dir() {
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| matches!(name, ".git" | ".hg" | ".svn" | "target"))
        {
            return Ok(());
        }
        if !visited.insert(resolved) {
            return Ok(());
        }
        let mut entries = fs::read_dir(path)?.collect::<Result<Vec<_>, _>>()?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            collect_generated_support_rust_files(root, &entry.path(), visited, files)?;
        }
        return Ok(());
    }
    if metadata.is_file() && path.extension().is_some_and(|extension| extension == "rs") {
        files.push(path.to_path_buf());
    }
    Ok(())
}

fn add_generated_support_syntactic_hazards(
    counts: SyntacticHazardCounts,
    hazards: &mut Vec<ProductionHazardReport>,
) {
    if counts.source_include_macros > 0 {
        hazards.push(production_hazard_with_details(
            "source_include_macros",
            "error",
            format!(
                "{} copied support include! macro(s) inject Rust source outside the static reachability graph",
                counts.source_include_macros
            ),
            counts.source_include_details,
        ));
    }
    if counts.scoped_source_include_macros > 0 {
        hazards.push(production_hazard_with_details(
            "scoped_source_include_macros",
            "warning",
            format!(
                "{} copied support expression/type include! macro(s) inject package-local Rust tokens already scoped by retained identifier blockers",
                counts.scoped_source_include_macros
            ),
            counts.scoped_source_include_details,
        ));
    }
    if counts.out_dir_source_include_macros > 0 {
        hazards.push(production_hazard_with_details(
            "out_dir_source_include_macros",
            "error",
            format!(
                "{} copied support include! macro(s) read generated Rust from OUT_DIR; production slicing cannot semantically model build-generated source",
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
                "{} copied support include_str!/include_bytes! macro(s) use paths the slicer cannot statically resolve",
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
                "{} copied support include_str!/include_bytes! macro(s) read generated files from OUT_DIR; production slicing cannot semantically model build-generated assets",
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
                "{} copied support include_str!/include_bytes! macro(s) use absolute paths that would read outside the generated slice",
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
                "{} copied support include_str!/include_bytes! macro(s) resolve outside their package root",
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
                "{} copied support env!/option_env! macro(s) read compile-time environment outside the manifest model",
                counts.compile_env_macros
            ),
            counts.compile_env_details,
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
        blocked_idents: Vec::new(),
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
        blocked_idents: Vec::new(),
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
        blocked_idents: Vec::new(),
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
    if counts.scoped_source_include_macros > 0 {
        hazards.push(production_hazard_with_details(
            "scoped_source_include_macros",
            "warning",
            format!(
                "{} retained expression/type include! macro(s) inject package-local Rust tokens already scoped by retained identifier blockers",
                counts.scoped_source_include_macros
            ),
            counts.scoped_source_include_details,
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
            "warning",
            format!(
                "{} retained function pointer type surface(s) may hide callback edges outside the static call graph; compiler feedback validates the generated signatures and remaining warning requires semantic review",
                counts.function_pointer_surfaces
            ),
            counts.function_pointer_details,
        ));
    }
    if counts.trait_object_surfaces > 0 {
        hazards.push(production_hazard_with_details(
            "trait_object_surfaces",
            "warning",
            format!(
                "{} retained trait object surface(s) may hide dynamic dispatch edges outside the static call graph; compiler feedback validates the generated signatures and remaining warning requires semantic review",
                counts.trait_object_surfaces
            ),
            counts.trait_object_details,
        ));
    }
    if counts.dynamic_callback_boundary_surfaces > 0 {
        hazards.push(production_hazard_with_details(
            "dynamic_callback_boundaries",
            "warning",
            format!(
                "{} retained direct callback/dynamic-dispatch input boundary surface(s) were preserved as selected API inputs; compiler feedback validates the generated signature",
                counts.dynamic_callback_boundary_surfaces
            ),
            counts.dynamic_callback_boundary_details,
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
    project: &Project,
    reduced: &ReducedProject,
    semantic_pruning_proven: bool,
    hazards: &mut Vec<ProductionHazardReport>,
) {
    let evidence = &reduced.evidence;
    if evidence.unresolved_method_candidate_matches > 0 && !semantic_pruning_proven {
        hazards.push(production_hazard(
            "syntactic_method_fallbacks",
            "warning",
            format!(
                "{} unresolved method call(s) used syntactic fallback analysis; {} candidate method(s) were retained by name and require compiler feedback validation",
                evidence.unresolved_method_fallbacks, evidence.unresolved_method_candidate_matches
            ),
        ));
    }
    if evidence.generic_unresolved_method_candidate_matches > 0 && !semantic_pruning_proven {
        hazards.push(production_hazard_with_details(
            "generic_method_name_fallbacks",
            "warning",
            format!(
                "{} receiverless generic method fallback(s) skipped {} local same-name candidate method(s); compiler feedback is required to detect any omitted dependencies behind external or macro-generated receivers",
                evidence.generic_unresolved_method_fallbacks,
                evidence.generic_unresolved_method_candidate_matches
            ),
            method_fallback_details(&evidence.generic_unresolved_method_details),
        ));
    }
    let capped_details = capped_method_fallback_details(project, reduced, evidence);
    let unaccounted_capped_fallbacks = evidence
        .capped_unresolved_method_fallbacks
        .saturating_sub(evidence.capped_unresolved_method_details.len());
    let capped_fallback_hazard_count = unaccounted_capped_fallbacks + capped_details.len();
    if capped_fallback_hazard_count > 0 {
        hazards.push(production_hazard_with_details(
            "syntactic_method_fallback_cap",
            "warning",
            format!(
                "{} unresolved method fallback(s) exceeded the name-only candidate cap; compiler feedback is required to detect any omitted method dependencies",
                capped_fallback_hazard_count
            ),
            capped_details,
        ));
    }
    if evidence.semantic_edges_applied > 0 && !semantic_pruning_proven {
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

fn capped_method_fallback_details(
    project: &Project,
    reduced: &ReducedProject,
    evidence: &model::ReductionEvidence,
) -> Vec<ProductionHazardDetail> {
    let retained_paths = retained_semantic_file_paths(project, reduced);
    evidence
        .capped_unresolved_method_details
        .iter()
        .filter(|detail| {
            !project_method_name_fully_reachable_in_retained_paths(
                project,
                reduced,
                &retained_paths,
                &detail.method_name,
            )
        })
        .map(|detail| ProductionHazardDetail {
            subject: format!(
                "{}method={}; local_candidate_methods={}; receiver_candidates={}",
                detail
                    .owner
                    .as_ref()
                    .map(|owner| format!("{owner}: "))
                    .unwrap_or_default(),
                detail.method_name,
                detail.candidate_count,
                detail.receiver_candidate_count
            ),
            package: detail.package.clone(),
            module_path: detail.module_path.as_ref().map(|path| path.join("::")),
            file: detail.file.clone(),
            start_line: detail.start_line,
            cfg: None,
            blocked_idents: vec![detail.method_name.clone()],
            suggested_cargo_args: Vec::new(),
        })
        .collect()
}

fn method_fallback_details(
    details: &[model::CappedMethodFallbackEvidence],
) -> Vec<ProductionHazardDetail> {
    details
        .iter()
        .map(|detail| ProductionHazardDetail {
            subject: format!(
                "{}method={}; local_candidate_methods={}; receiver_candidates={}",
                detail
                    .owner
                    .as_ref()
                    .map(|owner| format!("{owner}: "))
                    .unwrap_or_default(),
                detail.method_name,
                detail.candidate_count,
                detail.receiver_candidate_count
            ),
            package: detail.package.clone(),
            module_path: detail.module_path.as_ref().map(|path| path.join("::")),
            file: detail.file.clone(),
            start_line: detail.start_line,
            cfg: None,
            blocked_idents: vec![detail.method_name.clone()],
            suggested_cargo_args: Vec::new(),
        })
        .collect()
}

#[derive(Default)]
struct SyntacticHazardCounts {
    source_include_macros: usize,
    source_include_details: Vec<ProductionHazardDetail>,
    scoped_source_include_macros: usize,
    scoped_source_include_details: Vec<ProductionHazardDetail>,
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
    macro_surfaces: Vec<MacroSurface>,
    compile_env_macros: usize,
    compile_env_details: Vec<ProductionHazardDetail>,
    function_pointer_surfaces: usize,
    function_pointer_details: Vec<ProductionHazardDetail>,
    trait_object_surfaces: usize,
    trait_object_details: Vec<ProductionHazardDetail>,
    dynamic_callback_boundary_surfaces: usize,
    dynamic_callback_boundary_details: Vec<ProductionHazardDetail>,
    conditional_compilation_attrs: usize,
    conditional_compilation_details: Vec<ProductionHazardDetail>,
}

impl SyntacticHazardCounts {
    fn add(&mut self, other: Self) {
        self.source_include_macros += other.source_include_macros;
        self.source_include_details
            .extend(other.source_include_details);
        self.scoped_source_include_macros += other.scoped_source_include_macros;
        self.scoped_source_include_details
            .extend(other.scoped_source_include_details);
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
        self.macro_surfaces.extend(other.macro_surfaces);
        self.compile_env_macros += other.compile_env_macros;
        self.compile_env_details.extend(other.compile_env_details);
        self.function_pointer_surfaces += other.function_pointer_surfaces;
        self.function_pointer_details
            .extend(other.function_pointer_details);
        self.trait_object_surfaces += other.trait_object_surfaces;
        self.trait_object_details.extend(other.trait_object_details);
        self.dynamic_callback_boundary_surfaces += other.dynamic_callback_boundary_surfaces;
        self.dynamic_callback_boundary_details
            .extend(other.dynamic_callback_boundary_details);
        self.conditional_compilation_attrs += other.conditional_compilation_attrs;
        self.conditional_compilation_details
            .extend(other.conditional_compilation_details);
    }

    fn support_package_blocking_subset(self) -> Self {
        Self {
            source_include_macros: self.source_include_macros,
            source_include_details: self.source_include_details,
            scoped_source_include_macros: self.scoped_source_include_macros,
            scoped_source_include_details: self.scoped_source_include_details,
            out_dir_source_include_macros: self.out_dir_source_include_macros,
            out_dir_source_include_details: self.out_dir_source_include_details,
            nonliteral_file_include_macros: self.nonliteral_file_include_macros,
            nonliteral_file_include_details: self.nonliteral_file_include_details,
            out_dir_file_include_macros: self.out_dir_file_include_macros,
            out_dir_file_include_details: self.out_dir_file_include_details,
            absolute_file_include_macros: self.absolute_file_include_macros,
            absolute_file_include_details: self.absolute_file_include_details,
            external_file_include_macros: self.external_file_include_macros,
            external_file_include_details: self.external_file_include_details,
            compile_env_macros: self.compile_env_macros,
            compile_env_details: self.compile_env_details,
            ..Self::default()
        }
    }
}

fn syntactic_hazard_counts(project: &Project, reduced: &ReducedProject) -> SyntacticHazardCounts {
    let mut counts = SyntacticHazardCounts::default();
    let mut scanned_impl_attrs =
        BTreeSet::<(String, Vec<String>, PathBuf, usize, usize, usize, usize)>::new();
    counts.add(retained_module_boundary_hazard_counts(project, reduced));
    counts.add(retained_top_level_out_dir_macro_hazard_counts(
        project, reduced,
    ));
    counts.add(retained_inline_out_dir_macro_hazard_counts(
        project, reduced,
    ));
    counts.add(retained_top_level_macro_invocation_hazard_counts(
        project, reduced,
    ));

    for callable in &reduced.reachable {
        if let Some(record) = project.functions.get(callable) {
            let mut visitor = syntactic_hazard_visitor_for_location(
                project,
                &record.package,
                &record.module_path,
                Some(record.item.sig.ident.to_string()),
            );
            visitor.visit_item_fn(&record.item);
            counts.add(visitor.counts);
        } else if let Some(record) = project.methods.get(callable) {
            let mut visitor = syntactic_hazard_visitor_for_location(
                project,
                callable.package(),
                &record.module_path,
                Some(callable_name(callable)),
            );
            let impl_attr_key = (
                callable.package().to_string(),
                record.module_path.clone(),
                record.impl_span.file.clone(),
                record.impl_span.start_line,
                record.impl_span.start_column,
                record.impl_span.end_line,
                record.impl_span.end_column,
            );
            if scanned_impl_attrs.insert(impl_attr_key) {
                for attribute in &record.impl_attrs {
                    visitor.visit_attribute(attribute);
                }
            }
            visitor.visit_impl_item_fn(&record.item);
            counts.add(visitor.counts);
        }
    }

    for item in &reduced.reachable_items {
        if let Some(record) = project.items.get(item) {
            let scan_item = item_for_syntactic_hazard_scan(project, reduced, item, &record.item);
            let mut visitor = syntactic_hazard_visitor_for_location(
                project,
                &record.package,
                &record.module_path,
                Some(item.name.clone()),
            );
            visitor.visit_item(&scan_item);
            counts.add(visitor.counts);
        }
    }

    counts
}

fn retained_top_level_macro_invocation_hazard_counts(
    project: &Project,
    reduced: &ReducedProject,
) -> SyntacticHazardCounts {
    let mut counts = SyntacticHazardCounts::default();
    let retained_paths = retained_semantic_file_paths(project, reduced);
    for source in project
        .files
        .values()
        .filter(|source| reduced.packages.contains(&source.package))
        .filter(|source| retained_paths.contains(&normalize_report_path(&source.path)))
    {
        let mut visitor = syntactic_hazard_visitor_for_location(
            project,
            &source.package,
            &source.module_path,
            None,
        );
        for item in &source.syntax.items {
            let Item::Macro(item_macro) = item else {
                continue;
            };
            if item_macro.ident.is_some()
                || (macro_path_starts_with(&item_macro.mac, "uniffi")
                    && !reduced_package_preserves_uniffi_surface(project, reduced, &source.package))
                || !macro_invocation_requires_expansion_boundary(
                    &item_macro.mac,
                    &visitor.macro_context,
                )
                || !top_level_macro_invocation_mentions_reduced_code(
                    project, reduced, source, item_macro,
                )
            {
                continue;
            }
            visitor.visit_item(item);
        }
        counts.add(visitor.counts);
    }
    counts
}

fn top_level_macro_invocation_mentions_reduced_code(
    project: &Project,
    reduced: &ReducedProject,
    source: &model::SourceFile,
    item_macro: &syn::ItemMacro,
) -> bool {
    token_stream_idents(&item_macro.mac.tokens)
        .iter()
        .any(|ident| reduced_package_mentions_ident(project, reduced, &source.package, ident))
}

fn macro_path_starts_with(mac: &Macro, name: &str) -> bool {
    mac.path
        .segments
        .first()
        .is_some_and(|segment| segment.ident == name)
}

fn reduced_package_preserves_uniffi_surface(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
) -> bool {
    reduced
        .reachable
        .iter()
        .filter(|callable| callable.package() == package)
        .any(|callable| {
            project
                .functions
                .get(callable)
                .is_some_and(|record| attrs_include_uniffi_export(&record.item.attrs))
                || project.methods.get(callable).is_some_and(|record| {
                    attrs_include_uniffi_export(&record.item.attrs)
                        || attrs_include_uniffi_export(&record.impl_attrs)
                })
        })
        || reduced
            .reachable_items
            .iter()
            .filter(|item| item.package == package)
            .any(|item| {
                project
                    .items
                    .get(item)
                    .is_some_and(|record| attrs_include_uniffi_export(item_attrs(&record.item)))
            })
}

fn attrs_include_uniffi_export(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        let path = attr.path();
        (path.segments.len() == 2
            && path.segments[0].ident == "uniffi"
            && path.segments[1].ident == "export")
            || (path.is_ident("cfg_attr")
                && token_stream_mentions_ident(&attr.to_token_stream(), "uniffi")
                && token_stream_mentions_ident(&attr.to_token_stream(), "export"))
    })
}

fn retained_top_level_out_dir_macro_hazard_counts(
    project: &Project,
    reduced: &ReducedProject,
) -> SyntacticHazardCounts {
    let mut counts = SyntacticHazardCounts::default();
    for source in project
        .files
        .values()
        .filter(|source| reduced.packages.contains(&source.package))
    {
        let mut visitor = syntactic_hazard_visitor_for_location(
            project,
            &source.package,
            &source.module_path,
            None,
        );
        let generated_source_idents = visitor.build_script_generated_source_candidate_idents();
        if generated_source_idents.is_empty() {
            continue;
        }
        for item in &source.syntax.items {
            let Item::Macro(item_macro) = item else {
                continue;
            };
            if item_macro.ident.is_some()
                || !macro_path_ends_with(&item_macro.mac, "include")
                || !macro_tokens_reference_out_dir(&item_macro.mac.tokens)
                || !generated_source_idents.iter().any(|ident| {
                    reduced_package_mentions_ident(project, reduced, &source.package, ident)
                })
            {
                continue;
            }
            visitor.visit_item(item);
        }
        counts.add(visitor.counts);
    }
    counts
}

fn item_for_syntactic_hazard_scan(
    project: &Project,
    reduced: &ReducedProject,
    item_id: &ItemId,
    item: &Item,
) -> Item {
    match item {
        Item::Struct(item_struct) => {
            struct_item_for_syntactic_hazard_scan(project, reduced, item_id, item_struct)
        }
        Item::Trait(item_trait) => {
            trait_item_for_syntactic_hazard_scan(project, reduced, item_id, item_trait)
        }
        _ => item.clone(),
    }
}

fn struct_item_for_syntactic_hazard_scan(
    project: &Project,
    reduced: &ReducedProject,
    item_id: &ItemId,
    item_struct: &syn::ItemStruct,
) -> Item {
    if reduced
        .roots
        .iter()
        .any(|root| matches!(root, RootId::Item(root_item) if root_item == item_id))
    {
        return Item::Struct(item_struct.clone());
    }

    if !matches!(item_struct.fields, syn::Fields::Named(_)) {
        return Item::Struct(item_struct.clone());
    }

    let mut item_struct = item_struct.clone();
    let original_struct = item_struct.clone();
    if let syn::Fields::Named(fields) = &mut item_struct.fields {
        fields.named = fields
            .named
            .iter()
            .filter(|field| {
                struct_field_should_scan_hazards(
                    project,
                    reduced,
                    &item_id.package,
                    &original_struct,
                    field,
                )
            })
            .cloned()
            .collect();
    }

    Item::Struct(item_struct)
}

fn trait_item_for_syntactic_hazard_scan(
    project: &Project,
    reduced: &ReducedProject,
    item_id: &ItemId,
    item_trait: &syn::ItemTrait,
) -> Item {
    if reduced
        .roots
        .iter()
        .any(|root| matches!(root, RootId::Item(root_item) if root_item == item_id))
        || trait_has_reachable_impl_methods_for_hazard_scan(reduced, item_id)
        || trait_items_are_referenced_by_reachable_callables(project, reduced, item_id, item_trait)
    {
        return Item::Trait(item_trait.clone());
    }

    let mut item_trait = item_trait.clone();
    item_trait.items.clear();
    item_trait.attrs.retain(attr_is_inert_type_surface);
    Item::Trait(item_trait)
}

fn trait_has_reachable_impl_methods_for_hazard_scan(
    reduced: &ReducedProject,
    item_id: &ItemId,
) -> bool {
    let mut trait_path = item_id.module_path.clone();
    trait_path.push(item_id.name.clone());
    reduced.reachable.iter().any(|callable| {
        matches!(
            callable,
            CallableId::Method {
                package,
                trait_path: Some(callable_trait_path),
                ..
            } if package == &item_id.package && callable_trait_path == &trait_path
        )
    })
}

fn trait_items_are_referenced_by_reachable_callables(
    project: &Project,
    reduced: &ReducedProject,
    item_id: &ItemId,
    item_trait: &syn::ItemTrait,
) -> bool {
    item_trait.items.iter().any(|item| {
        let syn::TraitItem::Fn(method) = item else {
            return false;
        };
        reduced_callables_mention_ident(
            project,
            reduced,
            &item_id.package,
            &method.sig.ident.to_string(),
        )
    })
}

fn struct_field_should_scan_hazards(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    item_struct: &syn::ItemStruct,
    field: &syn::Field,
) -> bool {
    if matches!(item_struct.vis, syn::Visibility::Public(_))
        && matches!(field.vis, syn::Visibility::Public(_))
    {
        return true;
    }
    if item_struct.generics.type_params().any(|param| {
        token_stream_mentions_ident(&field.to_token_stream(), &param.ident.to_string())
    }) {
        return true;
    }
    if field
        .attrs
        .iter()
        .any(|attr| !attr_is_inert_type_surface(attr))
    {
        return true;
    }
    let Some(name) = field.ident.as_ref() else {
        return true;
    };
    reduced_callables_mention_ident(project, reduced, package, &name.to_string())
}

fn attr_is_inert_type_surface(attr: &Attribute) -> bool {
    let path = attr.path();
    path.is_ident("cfg")
        || path.is_ident("allow")
        || path.is_ident("deny")
        || path.is_ident("doc")
        || path.is_ident("deprecated")
}

fn retained_inline_out_dir_macro_hazard_counts(
    project: &Project,
    reduced: &ReducedProject,
) -> SyntacticHazardCounts {
    let mut counts = SyntacticHazardCounts::default();
    for source in project
        .files
        .values()
        .filter(|source| reduced.packages.contains(&source.package))
    {
        collect_inline_out_dir_macro_hazards(
            project,
            reduced,
            &source.package,
            &source.module_path,
            &source.module_path,
            &source.syntax.items,
            &mut counts,
        );
    }
    counts
}

fn collect_inline_out_dir_macro_hazards(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    source_module_path: &[String],
    module_path: &[String],
    items: &[Item],
    counts: &mut SyntacticHazardCounts,
) {
    for item in items {
        let Item::Mod(item_mod) = item else {
            continue;
        };
        let mut child_path = module_path.to_vec();
        child_path.push(item_mod.ident.to_string());
        if !retained_inline_module_should_scan(
            project,
            reduced,
            package,
            module_path,
            &item_mod.ident.to_string(),
            &child_path,
        ) {
            continue;
        }
        let Some((_, child_items)) = &item_mod.content else {
            continue;
        };

        let mut visitor = syntactic_hazard_visitor_for_inline_location(
            project,
            package,
            source_module_path,
            &child_path,
        );
        for child in child_items {
            if let Item::Mod(item_mod) = child {
                for attribute in &item_mod.attrs {
                    visitor.visit_attribute(attribute);
                }
            } else {
                visitor.visit_item(child);
            }
        }
        counts.add(visitor.counts);

        collect_inline_out_dir_macro_hazards(
            project,
            reduced,
            package,
            source_module_path,
            &child_path,
            child_items,
            counts,
        );
    }
}

fn retained_inline_module_should_scan(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    parent_module_path: &[String],
    module_name: &str,
    child_module_path: &[String],
) -> bool {
    reduced.reachable_items.contains(&ItemId {
        package: package.to_string(),
        module_path: parent_module_path.to_vec(),
        name: module_name.to_string(),
        kind: model::ItemKind::Mod,
    }) || reduced_contains_module_path(project, reduced, package, child_module_path)
        || reduced_package_mentions_ident(project, reduced, package, module_name)
}

fn reduced_contains_module_path(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
) -> bool {
    reduced.reachable.iter().any(|callable| match callable {
        CallableId::Free {
            package: callable_package,
            module_path: callable_module_path,
            ..
        } => callable_package == package && path_has_prefix(callable_module_path, module_path),
        CallableId::Method {
            package: callable_package,
            type_path,
            ..
        } => {
            callable_package == package
                && project
                    .methods
                    .get(callable)
                    .map(|record| path_has_prefix(&record.module_path, module_path))
                    .unwrap_or_else(|| path_has_prefix(type_path, module_path))
        }
    }) || reduced
        .reachable_items
        .iter()
        .any(|item| item.package == package && path_has_prefix(&item.module_path, module_path))
}

fn reduced_package_mentions_ident(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    ident: &str,
) -> bool {
    reduced_callables_mention_ident(project, reduced, package, ident)
        || reduced.reachable_items.iter().any(|item| {
            item.package == package
                && project.items.get(item).is_some_and(|record| {
                    token_stream_mentions_ident(&record.item.to_token_stream(), ident)
                })
        })
}

fn reduced_callables_mention_ident(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    ident: &str,
) -> bool {
    reduced.reachable.iter().any(|callable| match callable {
        CallableId::Free {
            package: callable_package,
            ..
        } if callable_package == package => project.functions.get(callable).is_some_and(|record| {
            token_stream_mentions_ident(&record.item.to_token_stream(), ident)
        }),
        CallableId::Method {
            package: callable_package,
            ..
        } if callable_package == package => project.methods.get(callable).is_some_and(|record| {
            token_stream_mentions_ident(&record.item.to_token_stream(), ident)
        }),
        _ => false,
    })
}

fn path_has_prefix(path: &[String], prefix: &[String]) -> bool {
    path.len() >= prefix.len() && path.iter().zip(prefix).all(|(left, right)| left == right)
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
        let source = module_path
            .split_last()
            .and_then(|(_, parent_module_path)| {
                source_for_module(project, &package, parent_module_path)
            });
        let file = source.map(|source| source.path.clone());
        let mut visitor = SyntacticHazardVisitor {
            project: Some(project),
            counts: SyntacticHazardCounts::default(),
            include_context: None,
            macro_context: MacroInvocationContext::for_project_source(project, &package, source),
            location: HazardLocation {
                package,
                module_path,
                file,
                owner: None,
            },
            proven_trait_object_surfaces: BTreeSet::new(),
            allow_rust_default_variant_attribute: 0,
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

fn syntactic_hazard_visitor_for_location<'project>(
    project: &'project Project,
    package: &str,
    module_path: &[String],
    owner: Option<String>,
) -> SyntacticHazardVisitor<'project> {
    let source = source_for_module(project, package, module_path);
    let file = source.map(|source| source.path.clone());
    let include_context = project.workspace.packages.get(package).and_then(|package| {
        file.as_ref().and_then(|source_path| {
            source_path.parent().map(|source_dir| IncludeContext {
                package_root: package.root.clone(),
                source_dir: source_dir.to_path_buf(),
                build_script_path: package_build_script_path(package),
            })
        })
    });
    SyntacticHazardVisitor {
        project: Some(project),
        counts: SyntacticHazardCounts::default(),
        include_context,
        macro_context: MacroInvocationContext::for_project_source(project, package, source),
        location: HazardLocation {
            package: package.to_string(),
            module_path: module_path.to_vec(),
            file,
            owner,
        },
        proven_trait_object_surfaces: BTreeSet::new(),
        allow_rust_default_variant_attribute: 0,
    }
}

fn syntactic_hazard_visitor_for_inline_location<'project>(
    project: &'project Project,
    package: &str,
    source_module_path: &[String],
    rendered_module_path: &[String],
) -> SyntacticHazardVisitor<'project> {
    let source = source_for_module(project, package, source_module_path);
    let file = source.map(|source| source.path.clone());
    let include_context = project.workspace.packages.get(package).and_then(|package| {
        file.as_ref().and_then(|source_path| {
            source_path.parent().map(|source_dir| IncludeContext {
                package_root: package.root.clone(),
                source_dir: source_dir.to_path_buf(),
                build_script_path: package_build_script_path(package),
            })
        })
    });
    SyntacticHazardVisitor {
        project: Some(project),
        counts: SyntacticHazardCounts::default(),
        include_context,
        macro_context: MacroInvocationContext::for_project_source(project, package, source),
        location: HazardLocation {
            package: package.to_string(),
            module_path: rendered_module_path.to_vec(),
            file,
            owner: None,
        },
        proven_trait_object_surfaces: BTreeSet::new(),
        allow_rust_default_variant_attribute: 0,
    }
}

fn callable_name(callable: &CallableId) -> String {
    match callable {
        CallableId::Free { name, .. } => name.clone(),
        CallableId::Method { method, .. } => method.clone(),
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

struct SyntacticHazardVisitor<'project> {
    project: Option<&'project Project>,
    counts: SyntacticHazardCounts,
    include_context: Option<IncludeContext>,
    macro_context: MacroInvocationContext,
    location: HazardLocation,
    proven_trait_object_surfaces: BTreeSet<String>,
    allow_rust_default_variant_attribute: usize,
}

#[derive(Clone, Default)]
struct MacroInvocationContext {
    async_trait_macro_roots: BTreeSet<String>,
    async_trait_attribute_imports: BTreeSet<String>,
    serde_derive_macro_roots: BTreeSet<String>,
    serde_derive_imports: BTreeSet<String>,
    thiserror_derive_macro_roots: BTreeSet<String>,
    thiserror_derive_imports: BTreeSet<String>,
    uniffi_macro_roots: BTreeSet<String>,
    uniffi_derive_imports: BTreeSet<String>,
    uniffi_attribute_imports: BTreeSet<String>,
    format_like_macro_roots: BTreeSet<String>,
    format_like_macro_imports: BTreeSet<String>,
    local_macro_definitions: BTreeSet<String>,
}

impl MacroInvocationContext {
    fn for_project_source(
        project: &Project,
        package: &str,
        source: Option<&model::SourceFile>,
    ) -> Self {
        let roots = format_like_macro_roots_for_package(project, package);
        let async_trait_roots = async_trait_macro_roots_for_package(project, package);
        let serde_derive_roots = serde_derive_macro_roots_for_package(project, package);
        let thiserror_derive_roots = thiserror_derive_macro_roots_for_package(project, package);
        let uniffi_roots = uniffi_macro_roots_for_package(project, package);
        match source {
            Some(source) => Self::for_syntax(
                &source.syntax,
                &roots,
                &async_trait_roots,
                &serde_derive_roots,
                &thiserror_derive_roots,
                &uniffi_roots,
            ),
            None => Self {
                format_like_macro_roots: roots,
                async_trait_macro_roots: async_trait_roots,
                serde_derive_macro_roots: serde_derive_roots,
                thiserror_derive_macro_roots: thiserror_derive_roots,
                uniffi_macro_roots: uniffi_roots,
                ..Self::default()
            },
        }
    }

    fn for_syntax(
        syntax: &syn::File,
        roots: &BTreeSet<String>,
        async_trait_roots: &BTreeSet<String>,
        serde_derive_roots: &BTreeSet<String>,
        thiserror_derive_roots: &BTreeSet<String>,
        uniffi_roots: &BTreeSet<String>,
    ) -> Self {
        Self {
            async_trait_macro_roots: async_trait_roots.clone(),
            async_trait_attribute_imports: async_trait_attribute_imports_from_file(
                syntax,
                async_trait_roots,
            ),
            serde_derive_macro_roots: serde_derive_roots.clone(),
            serde_derive_imports: known_derive_imports_from_file(
                syntax,
                serde_derive_roots,
                &["Serialize", "Deserialize"],
            ),
            thiserror_derive_macro_roots: thiserror_derive_roots.clone(),
            thiserror_derive_imports: known_derive_imports_from_file(
                syntax,
                thiserror_derive_roots,
                &["Error"],
            ),
            uniffi_macro_roots: uniffi_roots.clone(),
            uniffi_derive_imports: known_derive_imports_from_file(
                syntax,
                uniffi_roots,
                &["Enum", "Error", "Object", "Record"],
            ),
            uniffi_attribute_imports: known_attribute_imports_from_file(
                syntax,
                uniffi_roots,
                &["constructor", "export"],
            ),
            format_like_macro_roots: roots.clone(),
            format_like_macro_imports: format_like_macro_imports_from_file(syntax, roots),
            local_macro_definitions: local_macro_definitions_from_file(syntax),
        }
    }
}

struct IncludeContext {
    package_root: PathBuf,
    source_dir: PathBuf,
    build_script_path: Option<PathBuf>,
}

struct HazardLocation {
    package: String,
    module_path: Vec<String>,
    file: Option<PathBuf>,
    owner: Option<String>,
}

impl HazardLocation {
    fn subject(&self) -> String {
        if self.module_path.is_empty() {
            self.package.clone()
        } else {
            format!("{}::{}", self.package, self.module_path.join("::"))
        }
    }

    fn owner_subject(&self) -> String {
        let module = self.subject();
        self.owner
            .as_ref()
            .map(|owner| format!("{module}::{owner}"))
            .unwrap_or(module)
    }
}

impl<'ast> Visit<'ast> for SyntacticHazardVisitor<'_> {
    fn visit_item_enum(&mut self, item_enum: &'ast syn::ItemEnum) {
        for attribute in &item_enum.attrs {
            self.visit_attribute(attribute);
        }
        visit::visit_generics(self, &item_enum.generics);

        let previous = self.allow_rust_default_variant_attribute;
        if item_enum.attrs.iter().any(attribute_derives_default) {
            self.allow_rust_default_variant_attribute =
                self.allow_rust_default_variant_attribute.saturating_add(1);
        }
        for variant in &item_enum.variants {
            self.visit_variant(variant);
        }
        self.allow_rust_default_variant_attribute = previous;
    }

    fn visit_item_fn(&mut self, item_fn: &'ast syn::ItemFn) {
        self.with_function_dynamic_surface_proofs(&item_fn.sig.output, &item_fn.block, |visitor| {
            syn::visit::visit_item_fn(visitor, item_fn);
        });
    }

    fn visit_impl_item_fn(&mut self, item_fn: &'ast syn::ImplItemFn) {
        self.with_function_dynamic_surface_proofs(&item_fn.sig.output, &item_fn.block, |visitor| {
            syn::visit::visit_impl_item_fn(visitor, item_fn);
        });
    }

    fn visit_item_macro(&mut self, item_macro: &'ast syn::ItemMacro) {
        for attribute in &item_macro.attrs {
            self.visit_attribute(attribute);
        }
        self.visit_macro(&item_macro.mac);
    }

    fn visit_fn_arg(&mut self, argument: &'ast syn::FnArg) {
        if let syn::FnArg::Typed(argument) = argument {
            if dynamic_callback_boundary_kind(&argument.ty).is_some() {
                self.counts.dynamic_callback_boundary_surfaces += 1;
                self.counts
                    .dynamic_callback_boundary_details
                    .push(self.type_surface_detail(&argument.ty));
                for attribute in &argument.attrs {
                    self.visit_attribute(attribute);
                }
                self.visit_pat(&argument.pat);
                return;
            }
        }

        syn::visit::visit_fn_arg(self, argument);
    }

    fn visit_attribute(&mut self, attribute: &'ast Attribute) {
        if attribute.path().is_ident("cfg") || attribute.path().is_ident("cfg_attr") {
            self.counts.conditional_compilation_attrs += 1;
            self.counts
                .conditional_compilation_details
                .push(self.cfg_attr_detail(attribute));
        }
        let custom_derives = custom_derive_macro_paths(attribute, &self.macro_context);
        self.counts.custom_derive_macros += custom_derives.len();
        for derive_path in custom_derives {
            self.record_macro_surface("derive_macro", &derive_path, attribute, Vec::new());
            self.counts
                .custom_derive_details
                .push(self.macro_surface_detail(attribute, format!("derive {derive_path}")));
        }
        if !(self.allow_rust_default_variant_attribute > 0
            && modeled_rust_default_variant_attribute(attribute))
            && attribute_requires_macro_expansion(attribute, &self.macro_context)
        {
            let path = format_path(attribute.path());
            let blocked_idents = macro_attribute_blocked_idents(attribute);
            self.record_macro_surface(
                attribute_surface_kind(attribute),
                &path,
                attribute,
                blocked_idents.clone(),
            );
            self.counts.custom_attribute_macros += 1;
            self.counts
                .custom_attribute_details
                .push(self.attribute_macro_detail(attribute, blocked_idents));
        }
        let nested_macro_paths = cfg_attr_nested_macro_paths(attribute);
        self.counts.custom_attribute_macros += nested_macro_paths.custom_attributes.len();
        self.counts.custom_derive_macros += nested_macro_paths.custom_derives.len();
        for attribute_path in nested_macro_paths.custom_attributes {
            self.record_macro_surface(
                "attribute_macro",
                &attribute_path,
                attribute,
                macro_surface_blocked_idents(attribute.path(), &attribute.meta.to_token_stream()),
            );
            self.counts.custom_attribute_details.push(
                self.macro_surface_detail(attribute, format!("nested attr {attribute_path}")),
            );
        }
        for derive_path in nested_macro_paths.custom_derives {
            self.record_macro_surface("derive_macro", &derive_path, attribute, Vec::new());
            self.counts
                .custom_derive_details
                .push(self.macro_surface_detail(attribute, format!("nested derive {derive_path}")));
        }

        syn::visit::visit_attribute(self, attribute);
    }

    fn visit_macro(&mut self, mac: &'ast Macro) {
        if macro_path_ends_with(mac, "include") {
            if macro_tokens_reference_out_dir(&mac.tokens) {
                self.counts.out_dir_source_include_macros += 1;
                self.counts
                    .out_dir_source_include_details
                    .push(self.out_dir_source_include_detail(mac));
            } else if self.static_source_include_is_expression_or_type(mac) {
                self.counts.scoped_source_include_macros += 1;
                self.counts
                    .scoped_source_include_details
                    .push(self.source_include_detail(mac));
            } else {
                self.counts.source_include_macros += 1;
                self.counts
                    .source_include_details
                    .push(self.source_include_detail(mac));
            }
        } else if macro_path_ends_with(mac, "include_str")
            || macro_path_ends_with(mac, "include_bytes")
        {
            self.visit_file_include_macro(mac);
        } else if macro_path_ends_with(mac, "env") || macro_path_ends_with(mac, "option_env") {
            self.visit_compile_env_macro(mac);
        } else if macro_path_ends_with(mac, "macro_rules") {
            let count = compile_env_macro_count_in_tokens(&mac.tokens);
            if count > 0 {
                self.counts.compile_env_macros += count;
                self.counts.compile_env_details.push(self.span_detail(mac));
            }
        }
        if macro_invocation_requires_expansion_boundary(mac, &self.macro_context) {
            let path = format_path(&mac.path);
            let blocked_idents = macro_surface_blocked_idents(&mac.path, &mac.tokens);
            self.record_macro_surface("macro_invocation", &path, mac, blocked_idents.clone());
            self.counts.custom_macro_invocations += 1;
            self.counts
                .custom_macro_invocation_details
                .push(self.macro_invocation_detail(mac, blocked_idents));
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
        if trait_object_is_auto_trait_only(trait_object)
            || self
                .proven_trait_object_surfaces
                .contains(&trait_object_surface_key(trait_object))
        {
            syn::visit::visit_type_trait_object(self, trait_object);
            return;
        }
        self.counts.trait_object_surfaces += 1;
        self.counts
            .trait_object_details
            .push(self.type_surface_detail(trait_object));
        syn::visit::visit_type_trait_object(self, trait_object);
    }
}

impl SyntacticHazardVisitor<'_> {
    fn with_function_dynamic_surface_proofs(
        &mut self,
        output: &syn::ReturnType,
        block: &syn::Block,
        visit: impl FnOnce(&mut Self),
    ) {
        let previous = self.proven_trait_object_surfaces.clone();
        self.proven_trait_object_surfaces
            .extend(self.proven_return_trait_object_surfaces(output, block));
        visit(self);
        self.proven_trait_object_surfaces = previous;
    }

    fn proven_return_trait_object_surfaces(
        &self,
        output: &syn::ReturnType,
        block: &syn::Block,
    ) -> BTreeSet<String> {
        let Some(project) = self.project else {
            return BTreeSet::new();
        };
        let syn::ReturnType::Type(_, output_ty) = output else {
            return BTreeSet::new();
        };
        let surfaces = collect_trait_object_surfaces(output_ty);
        if surfaces.is_empty() {
            return BTreeSet::new();
        }

        let constructed_local_types = returned_constructed_local_type_names(
            project,
            &self.location.package,
            &self.location.module_path,
            block,
        );
        let constructed_project_types = returned_constructed_project_type_names(
            project,
            &self.location.package,
            &self.location.module_path,
            block,
        );
        if constructed_local_types.is_empty() && constructed_project_types.is_empty() {
            return BTreeSet::new();
        }

        surfaces
            .into_iter()
            .filter(|surface| {
                let traits = dispatchable_trait_object_names(surface);
                proven_local_trait_object_return(
                    project,
                    &self.location.package,
                    &constructed_local_types,
                    &traits,
                ) || proven_iterator_trait_object_return(
                    surface,
                    &traits,
                    &constructed_project_types,
                )
            })
            .map(|surface| trait_object_surface_key(surface))
            .collect()
    }

    fn cfg_attr_detail(&self, attribute: &Attribute) -> ProductionHazardDetail {
        ProductionHazardDetail {
            subject: self.location.subject(),
            package: Some(self.location.package.clone()),
            module_path: (!self.location.module_path.is_empty())
                .then(|| self.location.module_path.join("::")),
            file: self.location.file.clone(),
            start_line: Some(attribute.span().start().line),
            cfg: Some(attribute.to_token_stream().to_string()),
            blocked_idents: Vec::new(),
            suggested_cargo_args: cfg_gate_suggested_cargo_args(attribute),
        }
    }

    fn type_surface_detail<T: Spanned + ToTokens>(&self, node: &T) -> ProductionHazardDetail {
        let mut detail = self.span_detail(node);
        detail.subject = format!("{}: {}", detail.subject, node.to_token_stream());
        detail.blocked_idents = type_surface_blocked_idents(&node.to_token_stream());
        detail
    }

    fn source_include_detail(&self, mac: &Macro) -> ProductionHazardDetail {
        let mut detail = self.span_detail(mac);
        detail.blocked_idents = self.static_source_include_blocked_idents(mac);
        detail
    }

    fn out_dir_source_include_detail(&self, mac: &Macro) -> ProductionHazardDetail {
        let mut detail = self.span_detail(mac);
        detail.blocked_idents = self.build_script_generated_source_blocked_idents();
        detail
    }

    fn static_source_include_blocked_idents(&self, mac: &Macro) -> Vec<String> {
        let Some(path) = static_include_path(&mac.tokens) else {
            return Vec::new();
        };
        let Some(path) = self.resolved_package_include_path(&path) else {
            return Vec::new();
        };
        let Ok(text) = fs::read_to_string(path) else {
            return Vec::new();
        };
        source_text_blocked_idents(&text)
    }

    fn static_source_include_is_expression_or_type(&self, mac: &Macro) -> bool {
        let Some(path) = static_include_path(&mac.tokens) else {
            return false;
        };
        let Some(path) = self.resolved_package_include_path(&path) else {
            return false;
        };
        let Ok(text) = fs::read_to_string(path) else {
            return false;
        };
        source_text_is_expression_or_type_only(&text)
    }

    fn build_script_generated_source_blocked_idents(&self) -> Vec<String> {
        let Some(context) = &self.include_context else {
            return Vec::new();
        };
        let Some(build_script) = &context.build_script_path else {
            return Vec::new();
        };
        let Ok(text) = fs::read_to_string(build_script) else {
            return Vec::new();
        };
        let Ok(tokens) = text.parse::<TokenStream>() else {
            return Vec::new();
        };
        let mut idents = BTreeSet::new();
        collect_rust_source_string_literal_idents(&tokens, &mut idents);
        filtered_source_blocker_idents(idents)
    }

    fn build_script_generated_source_candidate_idents(&self) -> BTreeSet<String> {
        let Some(context) = &self.include_context else {
            return BTreeSet::new();
        };
        let Some(build_script) = &context.build_script_path else {
            return BTreeSet::new();
        };
        let Ok(text) = fs::read_to_string(build_script) else {
            return BTreeSet::new();
        };
        let Ok(tokens) = text.parse::<TokenStream>() else {
            return BTreeSet::new();
        };
        let mut idents = BTreeSet::new();
        collect_rust_source_candidate_idents(&tokens, &mut idents);
        idents
    }

    fn resolved_package_include_path(&self, path: &StaticIncludePath) -> Option<PathBuf> {
        let context = self.include_context.as_ref()?;
        let candidate = match path {
            StaticIncludePath::Absolute(path) => path.clone(),
            StaticIncludePath::SourceRelative(path) => context.source_dir.join(path),
            StaticIncludePath::PackageRelative(path) => context.package_root.join(path),
        };
        include_candidate_inside_package(context, &candidate).then_some(candidate)
    }

    fn attribute_macro_detail(
        &self,
        attribute: &Attribute,
        blocked_idents: Vec<String>,
    ) -> ProductionHazardDetail {
        self.macro_surface_detail_with_blockers(
            attribute,
            format!(
                "#[{}]",
                format_token_stream(&attribute.meta.to_token_stream())
            ),
            blocked_idents,
        )
    }

    fn macro_invocation_detail(
        &self,
        mac: &Macro,
        blocked_idents: Vec<String>,
    ) -> ProductionHazardDetail {
        self.macro_surface_detail_with_blockers(
            mac,
            format!("{}!", format_path(&mac.path)),
            blocked_idents,
        )
    }

    fn macro_surface_detail<T: Spanned>(
        &self,
        node: &T,
        surface: impl AsRef<str>,
    ) -> ProductionHazardDetail {
        self.macro_surface_detail_with_blockers(node, surface, Vec::new())
    }

    fn macro_surface_detail_with_blockers<T: Spanned>(
        &self,
        node: &T,
        surface: impl AsRef<str>,
        blocked_idents: Vec<String>,
    ) -> ProductionHazardDetail {
        let mut detail = self.span_detail(node);
        detail.subject = format!("{}: {}", self.location.owner_subject(), surface.as_ref());
        detail.blocked_idents = blocked_idents;
        detail
    }

    fn record_macro_surface<T: Spanned>(
        &mut self,
        kind: &str,
        path: &str,
        node: &T,
        blocked_idents: Vec<String>,
    ) {
        self.counts.macro_surfaces.push(MacroSurface {
            kind: kind.to_string(),
            category: "macro_blocked".to_string(),
            path: path.to_string(),
            subject: self.location.owner_subject(),
            package: self.location.package.clone(),
            module_path: (!self.location.module_path.is_empty())
                .then(|| self.location.module_path.join("::")),
            owner: self.location.owner.clone(),
            file: self.location.file.clone(),
            start_line: Some(node.span().start().line),
            blocked_idents,
        });
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
            blocked_idents: Vec::new(),
            suggested_cargo_args: Vec::new(),
        }
    }
}

impl SyntacticHazardVisitor<'_> {
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

        let package_root_lexical = normalize_path_lexically(&context.package_root);
        let candidate_lexical = normalize_path_lexically(&candidate);
        let lexical_outside = !candidate_lexical.starts_with(&package_root_lexical);
        let canonical_outside = match (
            candidate.canonicalize(),
            context.package_root.canonicalize(),
        ) {
            (Ok(candidate), Ok(package_root)) => !candidate.starts_with(package_root),
            _ => lexical_outside,
        };
        if canonical_outside {
            self.counts.external_file_include_macros += 1;
            self.counts
                .external_file_include_details
                .push(self.span_detail(mac));
        }
    }
}

fn include_candidate_inside_package(context: &IncludeContext, candidate: &Path) -> bool {
    let package_root_lexical = normalize_path_lexically(&context.package_root);
    let candidate_lexical = normalize_path_lexically(candidate);
    let lexical_inside = candidate_lexical.starts_with(&package_root_lexical);
    match (
        candidate.canonicalize(),
        context.package_root.canonicalize(),
    ) {
        (Ok(candidate), Ok(package_root)) => candidate.starts_with(package_root),
        _ => lexical_inside,
    }
}

fn source_text_blocked_idents(text: &str) -> Vec<String> {
    let mut visitor = SourceIncludeReferenceVisitor::default();
    if let Ok(file) = syn::parse_file(text) {
        visitor.visit_file(&file);
        return filtered_source_blocker_idents(visitor.idents);
    }
    if let Ok(expr) = syn::parse_str::<syn::Expr>(text) {
        visitor.visit_expr(&expr);
        return filtered_source_blocker_idents(visitor.idents);
    }
    if let Ok(ty) = syn::parse_str::<syn::Type>(text) {
        visitor.visit_type(&ty);
        return filtered_source_blocker_idents(visitor.idents);
    }
    Vec::new()
}

pub(crate) fn source_text_candidate_idents(text: &str) -> BTreeSet<String> {
    let mut idents = BTreeSet::new();
    collect_text_idents(text, &mut idents);
    idents.retain(|ident| {
        !type_surface_wrapper_or_builtin_ident(ident) && !source_include_noise_ident(ident)
    });
    idents
}

fn source_text_is_expression_or_type_only(text: &str) -> bool {
    if let Ok(file) = syn::parse_file(text) {
        return file.items.is_empty();
    }
    syn::parse_str::<syn::Expr>(text).is_ok() || syn::parse_str::<syn::Type>(text).is_ok()
}

fn collect_rust_source_string_literal_idents(tokens: &TokenStream, idents: &mut BTreeSet<String>) {
    let tokens: Vec<_> = tokens.clone().into_iter().collect();
    let mut index = 0;
    while index < tokens.len() {
        if let (
            Some(proc_macro2::TokenTree::Ident(ident)),
            Some(proc_macro2::TokenTree::Punct(punct)),
            Some(proc_macro2::TokenTree::Group(group)),
        ) = (
            tokens.get(index),
            tokens.get(index + 1),
            tokens.get(index + 2),
        ) {
            if punct.as_char() == '!' {
                let macro_name = ident.to_string();
                if let Some(value) =
                    generated_source_macro_string_value(&macro_name, &group.stream())
                {
                    if string_literal_may_contain_rust_source(&value) {
                        idents.extend(source_text_blocked_idents(&value));
                    }
                }
                collect_rust_source_string_literal_idents(&group.stream(), idents);
                index += 3;
                continue;
            }
        }

        match &tokens[index] {
            proc_macro2::TokenTree::Literal(literal) => {
                let Ok(literal) = syn::parse2::<syn::LitStr>(literal.to_token_stream()) else {
                    index += 1;
                    continue;
                };
                let value = literal.value();
                if string_literal_may_contain_rust_source(&value) {
                    idents.extend(source_text_blocked_idents(&value));
                }
            }
            proc_macro2::TokenTree::Group(group) => {
                collect_rust_source_string_literal_idents(&group.stream(), idents);
            }
            proc_macro2::TokenTree::Ident(_) | proc_macro2::TokenTree::Punct(_) => {}
        }
        index += 1;
    }
}

fn collect_rust_source_candidate_idents(tokens: &TokenStream, idents: &mut BTreeSet<String>) {
    let tokens: Vec<_> = tokens.clone().into_iter().collect();
    let mut index = 0;
    while index < tokens.len() {
        if let (
            Some(proc_macro2::TokenTree::Ident(ident)),
            Some(proc_macro2::TokenTree::Punct(punct)),
            Some(proc_macro2::TokenTree::Group(group)),
        ) = (
            tokens.get(index),
            tokens.get(index + 1),
            tokens.get(index + 2),
        ) {
            if punct.as_char() == '!' {
                let macro_name = ident.to_string();
                if let Some(value) =
                    generated_source_macro_string_value(&macro_name, &group.stream())
                {
                    if string_literal_may_contain_rust_source(&value) {
                        idents.extend(source_text_candidate_idents(&value));
                    }
                }
                collect_rust_source_candidate_idents(&group.stream(), idents);
                index += 3;
                continue;
            }
        }

        match &tokens[index] {
            proc_macro2::TokenTree::Literal(literal) => {
                let Ok(literal) = syn::parse2::<syn::LitStr>(literal.to_token_stream()) else {
                    index += 1;
                    continue;
                };
                let value = literal.value();
                if string_literal_may_contain_rust_source(&value) {
                    idents.extend(source_text_candidate_idents(&value));
                }
            }
            proc_macro2::TokenTree::Group(group) => {
                collect_rust_source_candidate_idents(&group.stream(), idents);
            }
            proc_macro2::TokenTree::Ident(_) | proc_macro2::TokenTree::Punct(_) => {}
        }
        index += 1;
    }
}

pub(crate) fn generated_source_macro_string_value(
    name: &str,
    tokens: &TokenStream,
) -> Option<String> {
    match name {
        "concat" => concat_macro_string_literal_value(tokens),
        "format" | "format_args" => format_macro_string_literal_value(tokens, 0, false),
        "write" => format_macro_string_literal_value(tokens, 1, false),
        "writeln" => format_macro_string_literal_value(tokens, 1, true),
        "quote" => quote_macro_token_source_value(tokens),
        "quote_spanned" => quote_spanned_macro_token_source_value(tokens),
        _ => None,
    }
}

fn concat_macro_string_literal_value(tokens: &TokenStream) -> Option<String> {
    let mut value = String::new();
    let mut saw_literal = false;
    for token in tokens.clone() {
        match token {
            proc_macro2::TokenTree::Literal(literal) => {
                let literal = syn::parse2::<syn::LitStr>(literal.to_token_stream()).ok()?;
                value.push_str(&literal.value());
                saw_literal = true;
            }
            proc_macro2::TokenTree::Punct(punct) if punct.as_char() == ',' => {}
            proc_macro2::TokenTree::Group(_)
            | proc_macro2::TokenTree::Ident(_)
            | proc_macro2::TokenTree::Punct(_) => return None,
        }
    }
    saw_literal.then_some(value)
}

fn quote_macro_token_source_value(tokens: &TokenStream) -> Option<String> {
    if token_stream_contains_quote_interpolation(tokens) {
        return None;
    }
    let value = tokens.to_string();
    (!value.trim().is_empty()).then_some(value)
}

fn quote_spanned_macro_token_source_value(tokens: &TokenStream) -> Option<String> {
    let tokens: Vec<_> = tokens.clone().into_iter().collect();
    for index in 0..tokens.len().saturating_sub(1) {
        if matches!(&tokens[index], proc_macro2::TokenTree::Punct(punct) if punct.as_char() == '=')
            && matches!(&tokens[index + 1], proc_macro2::TokenTree::Punct(punct) if punct.as_char() == '>')
        {
            let body = tokens[index + 2..].iter().cloned().collect::<TokenStream>();
            return quote_macro_token_source_value(&body);
        }
    }
    None
}

fn token_stream_contains_quote_interpolation(tokens: &TokenStream) -> bool {
    let tokens: Vec<_> = tokens.clone().into_iter().collect();
    let mut index = 0;
    while index < tokens.len() {
        match &tokens[index] {
            proc_macro2::TokenTree::Punct(punct) if punct.as_char() == '#' => {
                if let Some(group) = rust_attribute_group_after_hash(&tokens, index) {
                    if token_stream_contains_quote_interpolation(&group.stream()) {
                        return true;
                    }
                    index += rust_attribute_token_len_after_hash(&tokens, index);
                    continue;
                }
                return true;
            }
            proc_macro2::TokenTree::Group(group) => {
                if token_stream_contains_quote_interpolation(&group.stream()) {
                    return true;
                }
            }
            proc_macro2::TokenTree::Ident(_)
            | proc_macro2::TokenTree::Literal(_)
            | proc_macro2::TokenTree::Punct(_) => {}
        }
        index += 1;
    }
    false
}

fn rust_attribute_group_after_hash(
    tokens: &[proc_macro2::TokenTree],
    hash_index: usize,
) -> Option<&proc_macro2::Group> {
    match tokens.get(hash_index + 1) {
        Some(proc_macro2::TokenTree::Group(group))
            if group.delimiter() == proc_macro2::Delimiter::Bracket =>
        {
            Some(group)
        }
        Some(proc_macro2::TokenTree::Punct(punct)) if punct.as_char() == '!' => {
            match tokens.get(hash_index + 2) {
                Some(proc_macro2::TokenTree::Group(group))
                    if group.delimiter() == proc_macro2::Delimiter::Bracket =>
                {
                    Some(group)
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn rust_attribute_token_len_after_hash(
    tokens: &[proc_macro2::TokenTree],
    hash_index: usize,
) -> usize {
    if matches!(tokens.get(hash_index + 1), Some(proc_macro2::TokenTree::Punct(punct)) if punct.as_char() == '!')
    {
        3
    } else {
        2
    }
}

fn format_macro_string_literal_value(
    tokens: &TokenStream,
    skipped_args: usize,
    append_newline: bool,
) -> Option<String> {
    let args = split_top_level_comma_args(tokens);
    let format_arg = args.get(skipped_args)?;
    let format_string = literal_string_arg_value(format_arg)?;
    let mut positional = Vec::new();
    let mut named = BTreeMap::new();
    for arg in args.iter().skip(skipped_args + 1) {
        if let Some((name, value)) = named_literal_string_arg_value(arg) {
            named.insert(name, value);
        } else if let Some(value) = literal_string_arg_value(arg) {
            positional.push(value);
        } else {
            return None;
        }
    }
    evaluate_literal_format_string(&format_string, &positional, &named, append_newline)
}

fn split_top_level_comma_args(tokens: &TokenStream) -> Vec<TokenStream> {
    let mut args = Vec::new();
    let mut current = TokenStream::new();
    for token in tokens.clone() {
        if matches!(&token, proc_macro2::TokenTree::Punct(punct) if punct.as_char() == ',') {
            if !token_stream_is_empty(&current) {
                args.push(current);
                current = TokenStream::new();
            }
            continue;
        }
        current.extend([token]);
    }
    if !token_stream_is_empty(&current) {
        args.push(current);
    }
    args
}

fn token_stream_is_empty(tokens: &TokenStream) -> bool {
    tokens.clone().into_iter().next().is_none()
}

fn literal_string_arg_value(tokens: &TokenStream) -> Option<String> {
    if let Ok(literal) = syn::parse2::<syn::LitStr>(tokens.clone()) {
        return Some(literal.value());
    }
    stringify_macro_value(tokens)
}

fn named_literal_string_arg_value(tokens: &TokenStream) -> Option<(String, String)> {
    let mut iter = tokens.clone().into_iter();
    let proc_macro2::TokenTree::Ident(name) = iter.next()? else {
        return None;
    };
    let proc_macro2::TokenTree::Punct(eq) = iter.next()? else {
        return None;
    };
    if eq.as_char() != '=' {
        return None;
    }
    let value = iter.collect::<TokenStream>();
    Some((name.to_string(), literal_string_arg_value(&value)?))
}

fn stringify_macro_value(tokens: &TokenStream) -> Option<String> {
    let tokens: Vec<_> = tokens.clone().into_iter().collect();
    let [proc_macro2::TokenTree::Ident(ident), proc_macro2::TokenTree::Punct(punct), proc_macro2::TokenTree::Group(group)] =
        tokens.as_slice()
    else {
        return None;
    };
    (ident == "stringify" && punct.as_char() == '!').then(|| group.stream().to_string())
}

fn evaluate_literal_format_string(
    format_string: &str,
    positional: &[String],
    named: &BTreeMap<String, String>,
    append_newline: bool,
) -> Option<String> {
    let mut output = String::new();
    let mut chars = format_string.chars().peekable();
    let mut next_positional = 0;
    while let Some(ch) = chars.next() {
        match ch {
            '{' if chars.peek() == Some(&'{') => {
                chars.next();
                output.push('{');
            }
            '}' if chars.peek() == Some(&'}') => {
                chars.next();
                output.push('}');
            }
            '{' => {
                let mut placeholder = String::new();
                loop {
                    let next = chars.next()?;
                    if next == '}' {
                        break;
                    }
                    placeholder.push(next);
                }
                let name = placeholder
                    .split_once(':')
                    .map(|(name, _)| name)
                    .unwrap_or(&placeholder)
                    .trim();
                if name.is_empty() {
                    let value = positional.get(next_positional)?;
                    output.push_str(value);
                    next_positional += 1;
                } else if let Ok(index) = name.parse::<usize>() {
                    output.push_str(positional.get(index)?);
                } else {
                    output.push_str(named.get(name)?);
                }
            }
            '}' => return None,
            _ => output.push(ch),
        }
    }
    if append_newline {
        output.push('\n');
    }
    Some(output)
}

pub(crate) fn string_literal_may_contain_rust_source(value: &str) -> bool {
    value.contains("::")
        || value.contains("fn ")
        || value.contains("pub ")
        || value.contains("struct ")
        || value.contains("enum ")
        || value.contains("impl ")
        || value.contains("trait ")
        || value.contains("const ")
        || value.contains("static ")
        || value.contains("let ")
        || value.contains("->")
        || value.contains('{')
        || string_literal_parses_as_rust_call_expression(value)
        || string_literal_parses_as_rust_generic_type(value)
}

fn string_literal_parses_as_rust_call_expression(value: &str) -> bool {
    let Ok(expr) = syn::parse_str::<syn::Expr>(value) else {
        return false;
    };
    let mut visitor = RustSourceExpressionSignalVisitor { found: false };
    visitor.visit_expr(&expr);
    visitor.found
}

struct RustSourceExpressionSignalVisitor {
    found: bool,
}

impl<'ast> Visit<'ast> for RustSourceExpressionSignalVisitor {
    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        self.found = true;
        syn::visit::visit_expr_call(self, call);
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        self.found = true;
        syn::visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_macro(&mut self, mac: &'ast syn::ExprMacro) {
        self.found = true;
        syn::visit::visit_expr_macro(self, mac);
    }
}

fn string_literal_parses_as_rust_generic_type(value: &str) -> bool {
    value.contains('<') && value.contains('>') && syn::parse_str::<syn::Type>(value).is_ok()
}

fn filtered_source_blocker_idents(mut idents: BTreeSet<String>) -> Vec<String> {
    idents.retain(|ident| {
        !type_surface_wrapper_or_builtin_ident(ident) && !source_include_noise_ident(ident)
    });
    idents.into_iter().collect()
}

fn source_include_noise_ident(ident: &str) -> bool {
    matches!(
        ident,
        "CARGO_MANIFEST_DIR" | "OUT_DIR" | "include" | "include_str" | "include_bytes"
    )
}

#[derive(Default)]
struct SourceIncludeReferenceVisitor {
    idents: BTreeSet<String>,
}

impl<'ast> Visit<'ast> for SourceIncludeReferenceVisitor {
    fn visit_item_impl(&mut self, item_impl: &'ast syn::ItemImpl) {
        for attribute in &item_impl.attrs {
            self.visit_attribute(attribute);
        }
        self.visit_generics(&item_impl.generics);
        if let Some((_, trait_path, _)) = &item_impl.trait_ {
            self.visit_path(trait_path);
        }
        for item in &item_impl.items {
            self.visit_impl_item(item);
        }
    }

    fn visit_path(&mut self, path: &'ast syn::Path) {
        if let Some(segment) = path.segments.last() {
            let ident = segment.ident.to_string();
            if !rust_keyword_or_common_macro_word(&ident)
                && !macro_surface_meta_word(&ident)
                && !type_surface_wrapper_or_builtin_ident(&ident)
            {
                self.idents.insert(ident);
            }
        }
        syn::visit::visit_path(self, path);
    }
}

fn collect_trait_object_surfaces(ty: &syn::Type) -> Vec<&syn::TypeTraitObject> {
    let mut visitor = TraitObjectSurfaceCollector {
        surfaces: Vec::new(),
    };
    visitor.visit_type(ty);
    visitor.surfaces
}

struct TraitObjectSurfaceCollector<'ast> {
    surfaces: Vec<&'ast syn::TypeTraitObject>,
}

impl<'ast> Visit<'ast> for TraitObjectSurfaceCollector<'ast> {
    fn visit_type_trait_object(&mut self, trait_object: &'ast syn::TypeTraitObject) {
        self.surfaces.push(trait_object);
        syn::visit::visit_type_trait_object(self, trait_object);
    }
}

fn trait_object_is_auto_trait_only(trait_object: &syn::TypeTraitObject) -> bool {
    dispatchable_trait_object_names(trait_object).is_empty()
}

fn dispatchable_trait_object_names(trait_object: &syn::TypeTraitObject) -> BTreeSet<String> {
    trait_object
        .bounds
        .iter()
        .filter_map(|bound| {
            let syn::TypeParamBound::Trait(trait_bound) = bound else {
                return None;
            };
            let name = trait_bound.path.segments.last()?.ident.to_string();
            (!auto_trait_object_bound_name(&name)).then_some(name)
        })
        .collect()
}

fn auto_trait_object_bound_name(name: &str) -> bool {
    matches!(
        name,
        "Send" | "Sync" | "Unpin" | "UnwindSafe" | "RefUnwindSafe" | "Sized"
    )
}

fn trait_object_surface_key(trait_object: &syn::TypeTraitObject) -> String {
    format_token_stream(&trait_object.to_token_stream())
}

fn returned_constructed_local_type_names(
    project: &Project,
    package: &str,
    module_path: &[String],
    block: &syn::Block,
) -> BTreeSet<String> {
    let local_types = unique_data_type_names(project.items.keys().filter(|item| {
        item.package == package
            && matches!(
                item.kind,
                model::ItemKind::Struct | model::ItemKind::Enum | model::ItemKind::Union
            )
    }));
    returned_constructed_type_names(project, package, module_path, block, &local_types)
}

fn returned_constructed_project_type_names(
    project: &Project,
    package: &str,
    module_path: &[String],
    block: &syn::Block,
) -> BTreeSet<String> {
    let project_types = unique_data_type_names(project.items.keys().filter(|item| {
        matches!(
            item.kind,
            model::ItemKind::Struct | model::ItemKind::Enum | model::ItemKind::Union
        )
    }));
    returned_constructed_type_names_with_closures(
        project,
        package,
        module_path,
        block,
        &project_types,
    )
}

fn unique_data_type_names<'a>(items: impl Iterator<Item = &'a ItemId>) -> BTreeSet<String> {
    let mut counts = BTreeMap::<String, usize>::new();
    for item in items {
        *counts.entry(item.name.clone()).or_default() += 1;
    }
    counts
        .into_iter()
        .filter_map(|(name, count)| (count == 1).then_some(name))
        .collect()
}

fn returned_constructed_type_names(
    project: &Project,
    package: &str,
    module_path: &[String],
    block: &syn::Block,
    candidate_types: &BTreeSet<String>,
) -> BTreeSet<String> {
    returned_constructed_type_names_inner_entry(
        project,
        package,
        module_path,
        block,
        candidate_types,
        false,
    )
}

fn returned_constructed_type_names_with_closures(
    project: &Project,
    package: &str,
    module_path: &[String],
    block: &syn::Block,
    candidate_types: &BTreeSet<String>,
) -> BTreeSet<String> {
    returned_constructed_type_names_inner_entry(
        project,
        package,
        module_path,
        block,
        candidate_types,
        true,
    )
}

fn returned_constructed_type_names_inner_entry(
    project: &Project,
    package: &str,
    module_path: &[String],
    block: &syn::Block,
    candidate_types: &BTreeSet<String>,
    visit_closures: bool,
) -> BTreeSet<String> {
    let mut visited_callables = BTreeSet::new();
    returned_constructed_type_names_inner(
        project,
        package,
        module_path,
        block,
        candidate_types,
        visit_closures,
        &mut visited_callables,
    )
}

fn returned_constructed_type_names_inner(
    project: &Project,
    package: &str,
    module_path: &[String],
    block: &syn::Block,
    candidate_types: &BTreeSet<String>,
    visit_closures: bool,
    visited_callables: &mut BTreeSet<CallableId>,
) -> BTreeSet<String> {
    if candidate_types.is_empty() {
        return BTreeSet::new();
    }

    let mut return_visitor = ReturnConstructedLocalTypeVisitor {
        constructor: ConstructedLocalTypeVisitor {
            local_types: candidate_types,
            constructed: BTreeSet::new(),
            visit_closures,
        },
    };
    return_visitor.visit_block(block);

    if let Some(tail_expr) = block.stmts.iter().rev().find_map(|stmt| match stmt {
        syn::Stmt::Expr(expr, None) => Some(expr),
        syn::Stmt::Local(_)
        | syn::Stmt::Item(_)
        | syn::Stmt::Expr(_, Some(_))
        | syn::Stmt::Macro(_) => None,
    }) {
        return_visitor.constructor.visit_expr(tail_expr);
    }

    let mut constructed = return_visitor.constructor.constructed;
    for callable in returned_local_function_calls(project, package, module_path, block) {
        if !visited_callables.insert(callable.clone()) {
            continue;
        }
        let Some(record) = project.functions.get(&callable) else {
            continue;
        };
        constructed.extend(returned_constructed_type_names_inner(
            project,
            package,
            &record.module_path,
            &record.item.block,
            candidate_types,
            visit_closures,
            visited_callables,
        ));
    }
    constructed
}

fn proven_local_trait_object_return(
    project: &Project,
    package: &str,
    constructed_local_types: &BTreeSet<String>,
    traits: &BTreeSet<String>,
) -> bool {
    !traits.is_empty()
        && constructed_local_types.iter().any(|type_name| {
            traits.iter().all(|trait_name| {
                local_type_implements_trait(project, package, type_name, trait_name)
            })
        })
}

fn proven_iterator_trait_object_return(
    surface: &syn::TypeTraitObject,
    traits: &BTreeSet<String>,
    constructed_project_types: &BTreeSet<String>,
) -> bool {
    traits.len() == 1 && traits.contains("Iterator") && {
        let item_types = iterator_trait_object_item_type_names(surface);
        !item_types.is_empty()
            && item_types
                .iter()
                .all(|item_type| constructed_project_types.contains(item_type))
    }
}

fn iterator_trait_object_item_type_names(surface: &syn::TypeTraitObject) -> BTreeSet<String> {
    let mut item_types = BTreeSet::new();
    for bound in &surface.bounds {
        let syn::TypeParamBound::Trait(trait_bound) = bound else {
            continue;
        };
        let Some(segment) = trait_bound.path.segments.last() else {
            continue;
        };
        if segment.ident != "Iterator" {
            continue;
        }
        let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
            continue;
        };
        for argument in &arguments.args {
            if let syn::GenericArgument::AssocType(assoc) = argument {
                if assoc.ident == "Item" {
                    collect_type_leaf_idents(&assoc.ty, &mut item_types);
                }
            }
        }
    }
    item_types
}

fn collect_type_leaf_idents(ty: &syn::Type, idents: &mut BTreeSet<String>) {
    match ty {
        syn::Type::Array(array) => collect_type_leaf_idents(&array.elem, idents),
        syn::Type::Group(group) => collect_type_leaf_idents(&group.elem, idents),
        syn::Type::Paren(paren) => collect_type_leaf_idents(&paren.elem, idents),
        syn::Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                let ident = segment.ident.to_string();
                if !type_surface_wrapper_or_builtin_ident(&ident) {
                    idents.insert(ident);
                }
            }
            for segment in &type_path.path.segments {
                collect_path_argument_type_leaf_idents(&segment.arguments, idents);
            }
        }
        syn::Type::Ptr(ptr) => collect_type_leaf_idents(&ptr.elem, idents),
        syn::Type::Reference(reference) => collect_type_leaf_idents(&reference.elem, idents),
        syn::Type::Slice(slice) => collect_type_leaf_idents(&slice.elem, idents),
        syn::Type::Tuple(tuple) => {
            for elem in &tuple.elems {
                collect_type_leaf_idents(elem, idents);
            }
        }
        _ => {}
    }
}

fn collect_path_argument_type_leaf_idents(
    arguments: &syn::PathArguments,
    idents: &mut BTreeSet<String>,
) {
    let syn::PathArguments::AngleBracketed(arguments) = arguments else {
        return;
    };
    for argument in &arguments.args {
        match argument {
            syn::GenericArgument::Type(ty) => collect_type_leaf_idents(ty, idents),
            syn::GenericArgument::AssocType(assoc) => collect_type_leaf_idents(&assoc.ty, idents),
            _ => {}
        }
    }
}

struct ReturnConstructedLocalTypeVisitor<'types> {
    constructor: ConstructedLocalTypeVisitor<'types>,
}

impl<'ast> Visit<'ast> for ReturnConstructedLocalTypeVisitor<'_> {
    fn visit_expr_return(&mut self, expr_return: &'ast syn::ExprReturn) {
        if let Some(expr) = &expr_return.expr {
            self.constructor.visit_expr(expr);
        }
    }

    fn visit_expr_closure(&mut self, _closure: &'ast syn::ExprClosure) {}

    fn visit_item_fn(&mut self, _item: &'ast syn::ItemFn) {}

    fn visit_impl_item_fn(&mut self, _item: &'ast syn::ImplItemFn) {}
}

fn returned_local_function_calls(
    project: &Project,
    package: &str,
    module_path: &[String],
    block: &syn::Block,
) -> BTreeSet<CallableId> {
    let mut visitor = ReturnLocalFunctionCallVisitor {
        project,
        package,
        module_path,
        calls: BTreeSet::new(),
    };
    visitor.visit_block(block);
    if let Some(tail_expr) = block.stmts.iter().rev().find_map(|stmt| match stmt {
        syn::Stmt::Expr(expr, None) => Some(expr),
        syn::Stmt::Local(_)
        | syn::Stmt::Item(_)
        | syn::Stmt::Expr(_, Some(_))
        | syn::Stmt::Macro(_) => None,
    }) {
        visitor.visit_return_value_expr(tail_expr);
    }
    visitor.calls
}

struct ReturnLocalFunctionCallVisitor<'a> {
    project: &'a Project,
    package: &'a str,
    module_path: &'a [String],
    calls: BTreeSet<CallableId>,
}

impl ReturnLocalFunctionCallVisitor<'_> {
    fn visit_return_value_expr(&mut self, expr: &syn::Expr) {
        match expr {
            Expr::Block(expr) => self.visit_return_value_block(&expr.block),
            Expr::Unsafe(expr) => self.visit_return_value_block(&expr.block),
            Expr::If(expr) => {
                self.visit_return_value_block(&expr.then_branch);
                if let Some((_, else_branch)) = &expr.else_branch {
                    self.visit_return_value_expr(else_branch);
                }
            }
            Expr::Match(expr) => {
                for arm in &expr.arms {
                    self.visit_return_value_expr(&arm.body);
                }
            }
            Expr::Call(expr) => {
                if let syn::Expr::Path(func) = expr.func.as_ref() {
                    if let Some(callable) = resolve_returned_local_function_call(
                        self.project,
                        self.package,
                        self.module_path,
                        &func.path,
                    ) {
                        self.calls.insert(callable);
                        return;
                    }
                    if transparent_return_wrapper_path(&func.path) {
                        for arg in &expr.args {
                            self.visit_return_value_expr(arg);
                        }
                    }
                }
            }
            Expr::Paren(expr) => self.visit_return_value_expr(&expr.expr),
            Expr::Group(expr) => self.visit_return_value_expr(&expr.expr),
            Expr::Try(expr) => self.visit_return_value_expr(&expr.expr),
            _ => {}
        }
    }

    fn visit_return_value_block(&mut self, block: &syn::Block) {
        self.visit_block(block);
        if let Some(tail_expr) = block.stmts.iter().rev().find_map(|stmt| match stmt {
            syn::Stmt::Expr(expr, None) => Some(expr),
            syn::Stmt::Local(_)
            | syn::Stmt::Item(_)
            | syn::Stmt::Expr(_, Some(_))
            | syn::Stmt::Macro(_) => None,
        }) {
            self.visit_return_value_expr(tail_expr);
        }
    }
}

impl<'ast> Visit<'ast> for ReturnLocalFunctionCallVisitor<'_> {
    fn visit_expr_return(&mut self, expr_return: &'ast syn::ExprReturn) {
        if let Some(expr) = &expr_return.expr {
            self.visit_return_value_expr(expr);
        }
    }

    fn visit_expr_closure(&mut self, _closure: &'ast syn::ExprClosure) {}

    fn visit_item_fn(&mut self, _item: &'ast syn::ItemFn) {}

    fn visit_impl_item_fn(&mut self, _item: &'ast syn::ImplItemFn) {}
}

fn resolve_returned_local_function_call(
    project: &Project,
    package: &str,
    module_path: &[String],
    path: &syn::Path,
) -> Option<CallableId> {
    if path.segments.is_empty() {
        return None;
    }
    let mut segments = path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>();
    let name = segments.pop()?;
    let resolved_module = match segments.split_first() {
        None => module_path.to_vec(),
        Some((first, rest)) if first == "self" => {
            let mut path = module_path.to_vec();
            path.extend(rest.iter().cloned());
            path
        }
        Some((first, rest)) if first == "crate" => rest.to_vec(),
        Some((first, rest)) if first == "super" => {
            let mut path = module_path.to_vec();
            path.pop();
            path.extend(rest.iter().cloned());
            path
        }
        Some(_) => {
            let mut path = module_path.to_vec();
            path.extend(segments);
            path
        }
    };
    let callable = CallableId::Free {
        package: package.to_string(),
        module_path: resolved_module,
        name,
    };
    project
        .functions
        .contains_key(&callable)
        .then_some(callable)
}

struct ConstructedLocalTypeVisitor<'types> {
    local_types: &'types BTreeSet<String>,
    constructed: BTreeSet<String>,
    visit_closures: bool,
}

impl<'ast> Visit<'ast> for ConstructedLocalTypeVisitor<'_> {
    fn visit_expr_block(&mut self, expr: &'ast syn::ExprBlock) {
        self.visit_return_value_block(&expr.block);
    }

    fn visit_expr_unsafe(&mut self, expr: &'ast syn::ExprUnsafe) {
        self.visit_return_value_block(&expr.block);
    }

    fn visit_expr_if(&mut self, expr: &'ast syn::ExprIf) {
        self.visit_return_value_block(&expr.then_branch);
        if let Some((_, else_branch)) = &expr.else_branch {
            self.visit_expr(else_branch);
        }
    }

    fn visit_expr_match(&mut self, expr: &'ast syn::ExprMatch) {
        for arm in &expr.arms {
            self.visit_expr(&arm.body);
        }
    }

    fn visit_expr_closure(&mut self, closure: &'ast syn::ExprClosure) {
        if self.visit_closures {
            self.visit_expr(&closure.body);
        }
    }

    fn visit_expr_async(&mut self, _async_expr: &'ast syn::ExprAsync) {}

    fn visit_expr_struct(&mut self, expr: &'ast syn::ExprStruct) {
        self.record_path_leaf(&expr.path);
    }

    fn visit_expr_call(&mut self, expr: &'ast syn::ExprCall) {
        if let syn::Expr::Path(path) = expr.func.as_ref() {
            if self.record_constructor_path(&path.path) {
                return;
            }
            if transparent_return_wrapper_path(&path.path) {
                for arg in &expr.args {
                    self.visit_expr(arg);
                }
            }
        }
    }

    fn visit_expr_path(&mut self, expr: &'ast syn::ExprPath) {
        if expr.qself.is_none() {
            self.record_path_leaf(&expr.path);
        }
    }
}

impl ConstructedLocalTypeVisitor<'_> {
    fn visit_return_value_block(&mut self, block: &syn::Block) {
        let mut return_visitor = ReturnConstructedLocalTypeVisitor {
            constructor: ConstructedLocalTypeVisitor {
                local_types: self.local_types,
                constructed: BTreeSet::new(),
                visit_closures: self.visit_closures,
            },
        };
        return_visitor.visit_block(block);
        self.constructed
            .extend(return_visitor.constructor.constructed);
        if let Some(tail_expr) = block.stmts.iter().rev().find_map(|stmt| match stmt {
            syn::Stmt::Expr(expr, None) => Some(expr),
            syn::Stmt::Local(_)
            | syn::Stmt::Item(_)
            | syn::Stmt::Expr(_, Some(_))
            | syn::Stmt::Macro(_) => None,
        }) {
            self.visit_expr(tail_expr);
        }
    }

    fn record_constructor_path(&mut self, path: &syn::Path) -> bool {
        if path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "new")
            && path.segments.len() >= 2
        {
            if let Some(segment) = path.segments.iter().rev().nth(1) {
                return self.record_ident(&segment.ident.to_string());
            }
            return false;
        }
        self.record_path_leaf(path)
    }

    fn record_path_leaf(&mut self, path: &syn::Path) -> bool {
        if let Some(segment) = path.segments.last() {
            return self.record_ident(&segment.ident.to_string());
        }
        false
    }

    fn record_ident(&mut self, ident: &str) -> bool {
        if self.local_types.contains(ident) {
            self.constructed.insert(ident.to_string());
            true
        } else {
            false
        }
    }
}

fn transparent_return_wrapper_path(path: &syn::Path) -> bool {
    if let Some(last) = path
        .segments
        .last()
        .map(|segment| segment.ident.to_string())
    {
        if matches!(last.as_str(), "Some" | "Ok" | "Err") {
            return true;
        }
    }
    let segments = path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>();
    match segments.as_slice() {
        [wrapper, method]
            if matches!(wrapper.as_str(), "Box" | "Arc" | "Rc" | "Pin")
                && matches!(method.as_str(), "new" | "pin" | "new_unchecked") =>
        {
            true
        }
        segments
            if segments.len() >= 2
                && matches!(
                    segments[segments.len() - 2].as_str(),
                    "Box" | "Arc" | "Rc" | "Pin"
                )
                && matches!(
                    segments[segments.len() - 1].as_str(),
                    "new" | "pin" | "new_unchecked"
                ) =>
        {
            true
        }
        _ => false,
    }
}

fn local_type_implements_trait(
    project: &Project,
    package: &str,
    type_name: &str,
    trait_name: &str,
) -> bool {
    project.methods.keys().any(|callable| {
        matches!(
            callable,
            CallableId::Method {
                package: method_package,
                type_path,
                trait_path: Some(trait_path),
                ..
            } if method_package == package
                && type_path.last().is_some_and(|name| name == type_name)
                && trait_path.last().is_some_and(|name| name == trait_name)
        )
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DynamicCallbackBoundaryKind {
    FunctionPointer,
    TraitObject,
}

fn dynamic_callback_boundary_kind(ty: &syn::Type) -> Option<DynamicCallbackBoundaryKind> {
    dynamic_callback_boundary_kind_inner(ty, 0)
}

fn dynamic_callback_boundary_kind_inner(
    ty: &syn::Type,
    depth: usize,
) -> Option<DynamicCallbackBoundaryKind> {
    if depth > 8 {
        return None;
    }
    match peel_grouped_type(ty) {
        syn::Type::BareFn(_) => Some(DynamicCallbackBoundaryKind::FunctionPointer),
        syn::Type::Reference(reference) => match peel_grouped_type(&reference.elem) {
            syn::Type::TraitObject(_) => Some(DynamicCallbackBoundaryKind::TraitObject),
            inner => dynamic_callback_boundary_kind_inner(inner, depth + 1),
        },
        syn::Type::Slice(slice) => dynamic_callback_boundary_kind_inner(&slice.elem, depth + 1),
        syn::Type::Array(array) => dynamic_callback_boundary_kind_inner(&array.elem, depth + 1),
        syn::Type::Tuple(tuple) => tuple
            .elems
            .iter()
            .find_map(|elem| dynamic_callback_boundary_kind_inner(elem, depth + 1)),
        syn::Type::Path(type_path)
            if type_path_is_transparent_callback_boundary_wrapper(type_path) =>
        {
            type_path
                .path
                .segments
                .last()
                .and_then(|segment| match &segment.arguments {
                    syn::PathArguments::AngleBracketed(arguments) => {
                        arguments.args.iter().find_map(|argument| match argument {
                            syn::GenericArgument::Type(argument_ty) => {
                                dynamic_callback_boundary_kind_inner(argument_ty, depth + 1)
                            }
                            _ => None,
                        })
                    }
                    syn::PathArguments::None | syn::PathArguments::Parenthesized(_) => None,
                })
        }
        _ => None,
    }
}

fn type_path_is_transparent_callback_boundary_wrapper(type_path: &syn::TypePath) -> bool {
    if type_path.qself.is_some() {
        return false;
    }
    let Some(last) = type_path.path.segments.last() else {
        return false;
    };
    if !matches!(
        last.ident.to_string().as_str(),
        "Option" | "Result" | "Vec" | "Pin"
    ) {
        return false;
    }
    type_path
        .path
        .segments
        .iter()
        .take(type_path.path.segments.len().saturating_sub(1))
        .all(|segment| {
            matches!(
                segment.ident.to_string().as_str(),
                "std" | "core" | "alloc" | "option" | "result" | "vec" | "pin"
            )
        })
}

fn peel_grouped_type(mut ty: &syn::Type) -> &syn::Type {
    loop {
        match ty {
            syn::Type::Group(group) => ty = &group.elem,
            syn::Type::Paren(paren) => ty = &paren.elem,
            _ => return ty,
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

fn compile_env_macro_count_in_tokens(tokens: &TokenStream) -> usize {
    let tokens = tokens.clone().into_iter().collect::<Vec<_>>();
    let mut count = 0;
    let mut index = 0;
    while index < tokens.len() {
        match &tokens[index] {
            proc_macro2::TokenTree::Ident(ident)
                if matches!(ident.to_string().as_str(), "env" | "option_env") =>
            {
                if matches!(tokens.get(index + 1), Some(proc_macro2::TokenTree::Punct(punct)) if punct.as_char() == '!')
                    && tokens
                        .get(index + 2)
                        .and_then(|token| match token {
                            proc_macro2::TokenTree::Group(group) => {
                                macro_first_string_literal(&group.stream())
                            }
                            _ => None,
                        })
                        .as_deref()
                        .is_none_or(|name| !cargo_manifest_modeled_env_var(name))
                {
                    count += 1;
                }
                index += 3;
                continue;
            }
            proc_macro2::TokenTree::Group(group) => {
                count += compile_env_macro_count_in_tokens(&group.stream());
            }
            proc_macro2::TokenTree::Ident(_)
            | proc_macro2::TokenTree::Punct(_)
            | proc_macro2::TokenTree::Literal(_) => {}
        }
        index += 1;
    }
    count
}

fn cargo_manifest_modeled_env_var(name: &str) -> bool {
    name.starts_with("CARGO_PKG_") || matches!(name, "CARGO_CRATE_NAME" | "CARGO_BIN_NAME")
}

pub(crate) fn token_stream_mentions_string_literal(tokens: &TokenStream, value: &str) -> bool {
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

fn normalize_path_lexically(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(segment) => normalized.push(segment),
        }
    }
    normalized
}

fn macro_path_ends_with(mac: &Macro, name: &str) -> bool {
    mac.path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == name)
}

fn macro_invocation_requires_expansion_boundary(
    mac: &Macro,
    context: &MacroInvocationContext,
) -> bool {
    let Some(last) = mac.path.segments.last() else {
        return false;
    };
    if format_like_logging_macro_invocation(mac, context) {
        return false;
    }
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

fn format_like_logging_macro_invocation(mac: &Macro, context: &MacroInvocationContext) -> bool {
    let Some(last) = mac.path.segments.last() else {
        return false;
    };
    let name = last.ident.to_string();
    if !format_like_logging_macro_name(&name) {
        return false;
    }
    if mac.path.segments.len() == 1 {
        return context.format_like_macro_imports.contains(&name)
            && !context.local_macro_definitions.contains(&name);
    }
    mac.path.segments.first().is_some_and(|root| {
        context
            .format_like_macro_roots
            .contains(&root.ident.to_string())
    })
}

fn format_like_macro_roots_for_package(project: &Project, package: &str) -> BTreeSet<String> {
    let mut roots = BTreeSet::new();
    let Some(package) = project.workspace.packages.get(package) else {
        return roots;
    };
    for dependency in &package.dependencies {
        let package_name = rust_path_ident_for_package(&dependency.package);
        let alias = rust_path_ident_for_package(&dependency.alias);
        if format_like_logging_macro_root(&package_name) || format_like_logging_macro_root(&alias) {
            roots.insert(alias);
        }
    }
    roots
}

fn async_trait_macro_roots_for_package(project: &Project, package: &str) -> BTreeSet<String> {
    let mut roots = BTreeSet::new();
    let Some(package) = project.workspace.packages.get(package) else {
        return roots;
    };
    for dependency in &package.dependencies {
        let package_name = rust_path_ident_for_package(&dependency.package);
        let alias = rust_path_ident_for_package(&dependency.alias);
        if package_name == "async_trait" || alias == "async_trait" {
            roots.insert(alias);
        }
    }
    roots
}

fn serde_derive_macro_roots_for_package(project: &Project, package: &str) -> BTreeSet<String> {
    let mut roots = BTreeSet::new();
    let Some(package) = project.workspace.packages.get(package) else {
        return roots;
    };
    for dependency in &package.dependencies {
        let package_name = rust_path_ident_for_package(&dependency.package);
        let alias = rust_path_ident_for_package(&dependency.alias);
        if matches!(package_name.as_str(), "serde" | "serde_derive")
            || matches!(alias.as_str(), "serde" | "serde_derive")
        {
            roots.insert(alias);
        }
    }
    roots
}

fn thiserror_derive_macro_roots_for_package(project: &Project, package: &str) -> BTreeSet<String> {
    let mut roots = BTreeSet::new();
    let Some(package) = project.workspace.packages.get(package) else {
        return roots;
    };
    for dependency in &package.dependencies {
        let package_name = rust_path_ident_for_package(&dependency.package);
        let alias = rust_path_ident_for_package(&dependency.alias);
        if package_name == "thiserror" || alias == "thiserror" {
            roots.insert(alias);
        }
    }
    roots
}

fn uniffi_macro_roots_for_package(project: &Project, package: &str) -> BTreeSet<String> {
    let mut roots = BTreeSet::new();
    let Some(package) = project.workspace.packages.get(package) else {
        return roots;
    };
    for dependency in &package.dependencies {
        let package_name = rust_path_ident_for_package(&dependency.package);
        let alias = rust_path_ident_for_package(&dependency.alias);
        if package_name == "uniffi" || alias == "uniffi" {
            roots.insert(alias);
        }
    }
    roots
}

fn rust_path_ident_for_package(package: &str) -> String {
    package.replace('-', "_")
}

fn format_like_logging_macro_root(root: &str) -> bool {
    matches!(root, "tracing" | "log")
}

fn format_like_logging_macro_name(name: &str) -> bool {
    matches!(
        name,
        "trace" | "debug" | "info" | "warn" | "error" | "event" | "span"
    )
}

fn format_like_macro_imports_from_file(
    syntax: &syn::File,
    roots: &BTreeSet<String>,
) -> BTreeSet<String> {
    let mut imports = BTreeSet::new();
    for item in &syntax.items {
        collect_format_like_macro_imports_from_item(item, roots, &mut imports);
    }
    imports
}

fn async_trait_attribute_imports_from_file(
    syntax: &syn::File,
    roots: &BTreeSet<String>,
) -> BTreeSet<String> {
    let mut imports = BTreeSet::new();
    for item in &syntax.items {
        collect_async_trait_attribute_imports_from_item(item, roots, &mut imports);
    }
    imports
}

fn known_derive_imports_from_file(
    syntax: &syn::File,
    roots: &BTreeSet<String>,
    derive_names: &[&str],
) -> BTreeSet<String> {
    let derive_names = derive_names.iter().copied().collect::<BTreeSet<_>>();
    let mut imports = BTreeSet::new();
    for item in &syntax.items {
        collect_known_derive_imports_from_item(item, roots, &derive_names, &mut imports);
    }
    imports
}

fn known_attribute_imports_from_file(
    syntax: &syn::File,
    roots: &BTreeSet<String>,
    attribute_names: &[&str],
) -> BTreeSet<String> {
    let attribute_names = attribute_names.iter().copied().collect::<BTreeSet<_>>();
    let mut imports = BTreeSet::new();
    for item in &syntax.items {
        collect_known_attribute_imports_from_item(item, roots, &attribute_names, &mut imports);
    }
    imports
}

fn collect_known_derive_imports_from_item(
    item: &Item,
    roots: &BTreeSet<String>,
    derive_names: &BTreeSet<&str>,
    imports: &mut BTreeSet<String>,
) {
    match item {
        Item::Use(item_use) => {
            collect_known_derive_imports(&item_use.tree, Vec::new(), roots, derive_names, imports);
        }
        Item::Mod(item_mod) => {
            if let Some((_, items)) = &item_mod.content {
                for item in items {
                    collect_known_derive_imports_from_item(item, roots, derive_names, imports);
                }
            }
        }
        _ => {}
    }
}

fn collect_known_attribute_imports_from_item(
    item: &Item,
    roots: &BTreeSet<String>,
    attribute_names: &BTreeSet<&str>,
    imports: &mut BTreeSet<String>,
) {
    match item {
        Item::Use(item_use) => {
            collect_known_attribute_imports(
                &item_use.tree,
                Vec::new(),
                roots,
                attribute_names,
                imports,
            );
        }
        Item::Mod(item_mod) => {
            if let Some((_, items)) = &item_mod.content {
                for item in items {
                    collect_known_attribute_imports_from_item(
                        item,
                        roots,
                        attribute_names,
                        imports,
                    );
                }
            }
        }
        _ => {}
    }
}

fn collect_known_derive_imports(
    tree: &UseTree,
    mut prefix: Vec<String>,
    roots: &BTreeSet<String>,
    derive_names: &BTreeSet<&str>,
    imports: &mut BTreeSet<String>,
) {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            collect_known_derive_imports(&path.tree, prefix, roots, derive_names, imports);
        }
        UseTree::Name(name) => {
            let local = name.ident.to_string();
            if use_prefix_is_known_macro_root(&prefix, roots)
                && derive_names.contains(local.as_str())
            {
                imports.insert(local);
            }
        }
        UseTree::Rename(rename) => {
            let original = rename.ident.to_string();
            if use_prefix_is_known_macro_root(&prefix, roots)
                && derive_names.contains(original.as_str())
            {
                imports.insert(rename.rename.to_string());
            }
        }
        UseTree::Group(group) => {
            for item in &group.items {
                collect_known_derive_imports(item, prefix.clone(), roots, derive_names, imports);
            }
        }
        UseTree::Glob(_) => {
            if use_prefix_is_known_macro_root(&prefix, roots) {
                imports.extend(derive_names.iter().map(|name| (*name).to_string()));
            }
        }
    }
}

fn collect_known_attribute_imports(
    tree: &UseTree,
    mut prefix: Vec<String>,
    roots: &BTreeSet<String>,
    attribute_names: &BTreeSet<&str>,
    imports: &mut BTreeSet<String>,
) {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            collect_known_attribute_imports(
                path.tree.as_ref(),
                prefix,
                roots,
                attribute_names,
                imports,
            );
        }
        UseTree::Name(name) => {
            let local = name.ident.to_string();
            if use_prefix_is_known_macro_root(&prefix, roots)
                && attribute_names.contains(local.as_str())
            {
                imports.insert(local);
            }
        }
        UseTree::Rename(rename) => {
            let original = rename.ident.to_string();
            if use_prefix_is_known_macro_root(&prefix, roots)
                && attribute_names.contains(original.as_str())
            {
                imports.insert(rename.rename.to_string());
            }
        }
        UseTree::Group(group) => {
            for item in &group.items {
                collect_known_attribute_imports(
                    item,
                    prefix.clone(),
                    roots,
                    attribute_names,
                    imports,
                );
            }
        }
        UseTree::Glob(_) => {
            if use_prefix_is_known_macro_root(&prefix, roots) {
                imports.extend(attribute_names.iter().map(|name| (*name).to_string()));
            }
        }
    }
}

fn collect_async_trait_attribute_imports_from_item(
    item: &Item,
    roots: &BTreeSet<String>,
    imports: &mut BTreeSet<String>,
) {
    match item {
        Item::Use(item_use) => {
            collect_async_trait_attribute_imports(&item_use.tree, Vec::new(), roots, imports);
        }
        Item::Mod(item_mod) => {
            if let Some((_, items)) = &item_mod.content {
                for item in items {
                    collect_async_trait_attribute_imports_from_item(item, roots, imports);
                }
            }
        }
        _ => {}
    }
}

fn collect_async_trait_attribute_imports(
    tree: &UseTree,
    mut prefix: Vec<String>,
    roots: &BTreeSet<String>,
    imports: &mut BTreeSet<String>,
) {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            collect_async_trait_attribute_imports(&path.tree, prefix, roots, imports);
        }
        UseTree::Name(name) => {
            if use_prefix_is_known_macro_root(&prefix, roots) && name.ident == "async_trait" {
                imports.insert(name.ident.to_string());
            }
        }
        UseTree::Rename(rename) => {
            if use_prefix_is_known_macro_root(&prefix, roots) && rename.ident == "async_trait" {
                imports.insert(rename.rename.to_string());
            }
        }
        UseTree::Group(group) => {
            for item in &group.items {
                collect_async_trait_attribute_imports(item, prefix.clone(), roots, imports);
            }
        }
        UseTree::Glob(_) => {
            if use_prefix_is_known_macro_root(&prefix, roots) {
                imports.insert("async_trait".to_string());
            }
        }
    }
}

fn collect_format_like_macro_imports_from_item(
    item: &Item,
    roots: &BTreeSet<String>,
    imports: &mut BTreeSet<String>,
) {
    match item {
        Item::Use(item_use) => {
            collect_format_like_macro_imports(&item_use.tree, Vec::new(), roots, imports);
        }
        Item::Mod(item_mod) => {
            if let Some((_, items)) = &item_mod.content {
                for item in items {
                    collect_format_like_macro_imports_from_item(item, roots, imports);
                }
            }
        }
        _ => {}
    }
}

fn collect_format_like_macro_imports(
    tree: &UseTree,
    mut prefix: Vec<String>,
    roots: &BTreeSet<String>,
    imports: &mut BTreeSet<String>,
) {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            collect_format_like_macro_imports(&path.tree, prefix, roots, imports);
        }
        UseTree::Name(name) => {
            let local = name.ident.to_string();
            if use_prefix_is_format_like_macro_root(&prefix, roots)
                && format_like_logging_macro_name(&local)
            {
                imports.insert(local);
            }
        }
        UseTree::Rename(rename) => {
            if use_prefix_is_format_like_macro_root(&prefix, roots)
                && format_like_logging_macro_name(&rename.ident.to_string())
            {
                imports.insert(rename.rename.to_string());
            }
        }
        UseTree::Group(group) => {
            for item in &group.items {
                collect_format_like_macro_imports(item, prefix.clone(), roots, imports);
            }
        }
        UseTree::Glob(_) => {
            if use_prefix_is_format_like_macro_root(&prefix, roots) {
                imports.extend(
                    ["trace", "debug", "info", "warn", "error", "event", "span"]
                        .into_iter()
                        .map(str::to_string),
                );
            }
        }
    }
}

fn use_prefix_is_format_like_macro_root(prefix: &[String], roots: &BTreeSet<String>) -> bool {
    use_prefix_is_known_macro_root(prefix, roots)
}

fn use_prefix_is_known_macro_root(prefix: &[String], roots: &BTreeSet<String>) -> bool {
    prefix
        .first()
        .is_some_and(|root| roots.contains(&rust_path_ident_for_package(root)))
}

fn local_macro_definitions_from_file(syntax: &syn::File) -> BTreeSet<String> {
    let mut definitions = BTreeSet::new();
    for item in &syntax.items {
        collect_local_macro_definitions_from_item(item, &mut definitions);
    }
    definitions
}

fn collect_local_macro_definitions_from_item(item: &Item, definitions: &mut BTreeSet<String>) {
    match item {
        Item::Macro(item_macro) => {
            if let Some(ident) = &item_macro.ident {
                definitions.insert(ident.to_string());
            }
        }
        Item::Mod(item_mod) => {
            if let Some((_, items)) = &item_mod.content {
                for item in items {
                    collect_local_macro_definitions_from_item(item, definitions);
                }
            }
        }
        _ => {}
    }
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

#[derive(Default)]
struct CfgAttrNestedMacroPaths {
    custom_attributes: Vec<String>,
    custom_derives: Vec<String>,
}

fn cfg_attr_nested_macro_paths(attribute: &Attribute) -> CfgAttrNestedMacroPaths {
    if !attribute.path().is_ident("cfg_attr") {
        return CfgAttrNestedMacroPaths::default();
    }

    let Ok(arguments) =
        attribute.parse_args_with(Punctuated::<Meta, syn::Token![,]>::parse_terminated)
    else {
        return CfgAttrNestedMacroPaths::default();
    };

    cfg_attr_argument_macro_paths(&arguments)
}

fn cfg_attr_meta_nested_macro_paths(meta: &Meta) -> CfgAttrNestedMacroPaths {
    let Meta::List(list) = meta else {
        return CfgAttrNestedMacroPaths::default();
    };
    let Ok(arguments) =
        Punctuated::<Meta, syn::Token![,]>::parse_terminated.parse2(list.tokens.clone())
    else {
        return CfgAttrNestedMacroPaths::default();
    };

    cfg_attr_argument_macro_paths(&arguments)
}

fn cfg_attr_argument_macro_paths(
    arguments: &Punctuated<Meta, syn::Token![,]>,
) -> CfgAttrNestedMacroPaths {
    let mut paths = CfgAttrNestedMacroPaths::default();
    for nested_attr in arguments.iter().skip(1) {
        add_meta_macro_paths(nested_attr, &mut paths);
    }
    paths
}

fn add_meta_macro_paths(meta: &Meta, paths: &mut CfgAttrNestedMacroPaths) {
    if meta.path().is_ident("cfg_attr") {
        let nested = cfg_attr_meta_nested_macro_paths(meta);
        paths.custom_attributes.extend(nested.custom_attributes);
        paths.custom_derives.extend(nested.custom_derives);
        return;
    }
    if meta.path().is_ident("derive") {
        paths.custom_derives.extend(custom_derive_meta_paths(meta));
        return;
    }

    let Some(first) = meta.path().segments.first() else {
        return;
    };
    if !attribute_path_is_builtin_or_inert(&first.ident.to_string()) {
        paths.custom_attributes.push(format_path(meta.path()));
    }
}

fn custom_derive_macro_paths(
    attribute: &Attribute,
    context: &MacroInvocationContext,
) -> Vec<String> {
    if !attribute.path().is_ident("derive") {
        return Vec::new();
    }

    attribute
        .parse_args_with(Punctuated::<syn::Path, syn::Token![,]>::parse_terminated)
        .map(|paths| {
            paths
                .iter()
                .filter(|path| !derive_path_is_builtin(path))
                .filter(|path| !modeled_known_derive_macro_path(path, context))
                .map(format_path)
                .collect()
        })
        .unwrap_or_default()
}

fn custom_derive_meta_paths(meta: &Meta) -> Vec<String> {
    let Meta::List(list) = meta else {
        return Vec::new();
    };

    Punctuated::<syn::Path, syn::Token![,]>::parse_terminated
        .parse2(list.tokens.clone())
        .map(|paths| {
            paths
                .iter()
                .filter(|path| !derive_path_is_builtin(path))
                .map(format_path)
                .collect()
        })
        .unwrap_or_default()
}

fn format_path(path: &syn::Path) -> String {
    format_token_stream(&path.to_token_stream())
}

fn format_token_stream(tokens: &TokenStream) -> String {
    tokens.to_string().replace(" :: ", "::")
}

fn attribute_surface_kind(attribute: &Attribute) -> &'static str {
    let Some(first) = attribute.path().segments.first() else {
        return "attribute_macro";
    };
    if helper_attribute_name(&first.ident.to_string()) {
        "helper_attribute"
    } else {
        "attribute_macro"
    }
}

fn helper_attribute_name(name: &str) -> bool {
    matches!(
        name,
        "serde"
            | "serde_with"
            | "error"
            | "from"
            | "source"
            | "backtrace"
            | "strum"
            | "schemars"
            | "clap"
            | "arg"
            | "command"
            | "builder"
    )
}

fn macro_surface_blocked_idents(path: &syn::Path, tokens: &TokenStream) -> Vec<String> {
    let mut idents = token_stream_idents(tokens);
    collect_string_literal_path_idents(tokens, &mut idents);
    filter_macro_surface_blocked_idents(path, idents)
}

fn macro_attribute_blocked_idents(attribute: &Attribute) -> Vec<String> {
    let mut idents = if attribute
        .path()
        .segments
        .first()
        .is_some_and(|segment| helper_attribute_name(&segment.ident.to_string()))
    {
        BTreeSet::new()
    } else {
        token_stream_idents(&attribute.meta.to_token_stream())
    };
    collect_attribute_helper_path_idents(&attribute.meta, &mut idents);
    filter_macro_surface_blocked_idents(attribute.path(), idents)
}

fn filter_macro_surface_blocked_idents(
    path: &syn::Path,
    mut idents: BTreeSet<String>,
) -> Vec<String> {
    for segment in &path.segments {
        idents.remove(&segment.ident.to_string());
    }
    idents.retain(|ident| !macro_surface_meta_word(ident));
    idents.into_iter().collect()
}

fn collect_attribute_helper_path_idents(meta: &Meta, idents: &mut BTreeSet<String>) {
    match meta {
        Meta::Path(_) => {}
        Meta::NameValue(name_value) => {
            if helper_path_meta_key(&format_path(&name_value.path)) {
                if let syn::Expr::Lit(expr_lit) = &name_value.value {
                    if let syn::Lit::Str(literal) = &expr_lit.lit {
                        collect_path_like_string_idents(&literal.value(), idents);
                    }
                }
            }
        }
        Meta::List(list) => {
            if helper_path_meta_key(&format_path(&list.path)) {
                if let Ok(literal) = syn::parse2::<syn::LitStr>(list.tokens.clone()) {
                    collect_path_like_string_idents(&literal.value(), idents);
                    return;
                }
            }
            let Ok(arguments) =
                Punctuated::<Meta, syn::Token![,]>::parse_terminated.parse2(list.tokens.clone())
            else {
                return;
            };
            for nested in arguments {
                collect_attribute_helper_path_idents(&nested, idents);
            }
        }
    }
}

fn helper_path_meta_key(key: &str) -> bool {
    matches!(
        key,
        "default"
            | "deserialize_with"
            | "serialize_with"
            | "skip_serializing_if"
            | "with"
            | "serde_as"
            | "value_parser"
    )
}

fn type_surface_blocked_idents(tokens: &TokenStream) -> Vec<String> {
    let mut idents = token_stream_idents(tokens);
    idents.retain(|ident| !type_surface_wrapper_or_builtin_ident(ident));
    idents.into_iter().collect()
}

fn type_surface_wrapper_or_builtin_ident(ident: &str) -> bool {
    matches!(
        ident,
        "Arc"
            | "Box"
            | "Cell"
            | "Cow"
            | "Fn"
            | "FnMut"
            | "FnOnce"
            | "HashMap"
            | "HashSet"
            | "Mutex"
            | "Option"
            | "Pin"
            | "Rc"
            | "RefCell"
            | "Result"
            | "RwLock"
            | "Send"
            | "Sync"
            | "UnsafeCell"
            | "Vec"
            | "bool"
            | "char"
            | "f32"
            | "f64"
            | "i8"
            | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "isize"
            | "str"
            | "String"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "usize"
    )
}

fn token_stream_idents(tokens: &TokenStream) -> BTreeSet<String> {
    let mut idents = BTreeSet::new();
    collect_token_stream_idents(tokens, &mut idents);
    idents
}

fn collect_token_stream_idents(tokens: &TokenStream, idents: &mut BTreeSet<String>) {
    for token in tokens.clone() {
        match token {
            proc_macro2::TokenTree::Ident(ident) => {
                let ident = ident.to_string();
                if !rust_keyword_or_common_macro_word(&ident) && !macro_surface_meta_word(&ident) {
                    idents.insert(ident);
                }
            }
            proc_macro2::TokenTree::Group(group) => {
                collect_token_stream_idents(&group.stream(), idents)
            }
            proc_macro2::TokenTree::Punct(_) | proc_macro2::TokenTree::Literal(_) => {}
        }
    }
}

fn collect_string_literal_path_idents(tokens: &TokenStream, idents: &mut BTreeSet<String>) {
    for token in tokens.clone() {
        match token {
            proc_macro2::TokenTree::Literal(literal) => {
                if let Ok(literal) = syn::parse2::<syn::LitStr>(literal.to_token_stream()) {
                    collect_path_like_string_idents(&literal.value(), idents);
                }
            }
            proc_macro2::TokenTree::Group(group) => {
                collect_string_literal_path_idents(&group.stream(), idents)
            }
            proc_macro2::TokenTree::Ident(_) | proc_macro2::TokenTree::Punct(_) => {}
        }
    }
}

fn collect_path_like_string_idents(value: &str, idents: &mut BTreeSet<String>) {
    if value.is_empty()
        || value
            .chars()
            .any(|ch| !(ch == ':' || ch == '_' || ch.is_ascii_alphanumeric()))
    {
        return;
    }
    let segments = value.split("::").collect::<Vec<_>>();
    if segments
        .iter()
        .any(|segment| type_surface_wrapper_or_builtin_ident(segment))
    {
        return;
    }
    for segment in segments {
        if segment
            .chars()
            .next()
            .is_some_and(|ch| ch == '_' || ch.is_ascii_alphabetic())
            && !rust_keyword_or_common_macro_word(segment)
            && !macro_surface_meta_word(segment)
        {
            idents.insert(segment.to_string());
        }
    }
}

fn macro_surface_meta_word(ident: &str) -> bool {
    matches!(
        ident,
        "as" | "bound"
            | "content"
            | "crate"
            | "default"
            | "deny_unknown_fields"
            | "deserialize_with"
            | "flatten"
            | "from"
            | "getter"
            | "into"
            | "camelCase"
            | "kebab-case"
            | "lowercase"
            | "PascalCase"
            | "rename"
            | "rename_all"
            | "SCREAMING_SNAKE_CASE"
            | "snake_case"
            | "serialize_with"
            | "skip"
            | "skip_deserializing"
            | "skip_serializing"
            | "skip_serializing_if"
            | "tag"
            | "transparent"
            | "try_from"
            | "untagged"
            | "with"
    )
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

fn modeled_known_derive_macro_path(path: &syn::Path, context: &MacroInvocationContext) -> bool {
    modeled_serde_derive_macro_path(path, context)
        || modeled_thiserror_derive_macro_path(path, context)
        || modeled_uniffi_derive_macro_path(path, context)
}

fn modeled_serde_derive_macro_path(path: &syn::Path, context: &MacroInvocationContext) -> bool {
    let Some(last) = path.segments.last() else {
        return false;
    };
    let name = last.ident.to_string();
    if !matches!(name.as_str(), "Serialize" | "Deserialize") {
        return false;
    }
    if path.segments.len() == 1 {
        return context.serde_derive_imports.contains(&name);
    }
    path.segments.first().is_some_and(|root| {
        context
            .serde_derive_macro_roots
            .contains(&root.ident.to_string())
    })
}

fn modeled_thiserror_derive_macro_path(path: &syn::Path, context: &MacroInvocationContext) -> bool {
    let Some(last) = path.segments.last() else {
        return false;
    };
    let name = last.ident.to_string();
    if name != "Error" {
        return false;
    }
    if path.segments.len() == 1 {
        return context.thiserror_derive_imports.contains(&name);
    }
    path.segments.first().is_some_and(|root| {
        context
            .thiserror_derive_macro_roots
            .contains(&root.ident.to_string())
    })
}

fn modeled_uniffi_derive_macro_path(path: &syn::Path, context: &MacroInvocationContext) -> bool {
    let Some(last) = path.segments.last() else {
        return false;
    };
    let name = last.ident.to_string();
    if !matches!(name.as_str(), "Enum" | "Error" | "Object" | "Record") {
        return false;
    }
    if path.segments.len() == 1 {
        return context.uniffi_derive_imports.contains(&name);
    }
    path.segments
        .first()
        .is_some_and(|root| context.uniffi_macro_roots.contains(&root.ident.to_string()))
}

fn attribute_requires_macro_expansion(
    attribute: &Attribute,
    context: &MacroInvocationContext,
) -> bool {
    if attribute.path().is_ident("derive") {
        return false;
    }
    if modeled_async_trait_attribute(attribute, context) {
        return false;
    }
    if modeled_uniffi_attribute(attribute, context) {
        return false;
    }
    if modeled_uniffi_helper_attribute(attribute, context) {
        return false;
    }
    if modeled_direct_serde_helper_attribute(attribute) {
        return false;
    }

    let Some(first) = attribute.path().segments.first() else {
        return false;
    };
    if helper_attribute_name(&first.ident.to_string()) {
        return !macro_attribute_blocked_idents(attribute).is_empty();
    }
    !attribute_path_is_builtin_or_inert(&first.ident.to_string())
}

fn modeled_direct_serde_helper_attribute(attribute: &Attribute) -> bool {
    if !attribute.path().segments.first().is_some_and(|segment| {
        let ident = segment.ident.to_string();
        ident == "serde" || ident.ends_with("_serde")
    }) {
        return false;
    }
    let mut has_direct_helper = false;
    let mut has_module_helper = false;
    collect_serde_helper_attribute_shape(
        &attribute.meta,
        &mut has_direct_helper,
        &mut has_module_helper,
    );
    has_direct_helper && !has_module_helper
}

fn collect_serde_helper_attribute_shape(
    meta: &Meta,
    has_direct_helper: &mut bool,
    has_module_helper: &mut bool,
) {
    match meta {
        Meta::Path(_) => {}
        Meta::NameValue(name_value) => {
            classify_serde_helper_meta_key(
                &format_path(&name_value.path),
                has_direct_helper,
                has_module_helper,
            );
        }
        Meta::List(list) => {
            classify_serde_helper_meta_key(
                &format_path(&list.path),
                has_direct_helper,
                has_module_helper,
            );
            let Ok(arguments) =
                Punctuated::<Meta, syn::Token![,]>::parse_terminated.parse2(list.tokens.clone())
            else {
                return;
            };
            for nested in arguments {
                collect_serde_helper_attribute_shape(&nested, has_direct_helper, has_module_helper);
            }
        }
    }
}

fn classify_serde_helper_meta_key(
    key: &str,
    has_direct_helper: &mut bool,
    has_module_helper: &mut bool,
) {
    match key {
        "default" | "deserialize_with" | "serialize_with" | "skip_serializing_if" => {
            *has_direct_helper = true;
        }
        "with" | "serde_as" => {
            *has_module_helper = true;
        }
        _ => {}
    }
}

fn attribute_derives_default(attribute: &Attribute) -> bool {
    if !attribute.path().is_ident("derive") {
        return false;
    }
    let Ok(list) = attribute.meta.require_list() else {
        return false;
    };
    list.tokens
        .to_string()
        .split(|character: char| {
            !(character.is_ascii_alphanumeric() || matches!(character, '_' | ':'))
        })
        .any(|token| token.rsplit("::").next() == Some("Default"))
}

fn modeled_rust_default_variant_attribute(attribute: &Attribute) -> bool {
    attribute.path().is_ident("default") && matches!(attribute.meta, Meta::Path(_))
}

fn modeled_uniffi_helper_attribute(
    attribute: &Attribute,
    context: &MacroInvocationContext,
) -> bool {
    if !attribute.path().segments.first().is_some_and(|segment| {
        context
            .uniffi_macro_roots
            .contains(&segment.ident.to_string())
    }) {
        return false;
    }

    let mut has_direct_helper = false;
    let mut has_module_helper = false;
    collect_uniffi_helper_attribute_shape(
        &attribute.meta,
        &mut has_direct_helper,
        &mut has_module_helper,
    );
    has_direct_helper && !has_module_helper
}

fn collect_uniffi_helper_attribute_shape(
    meta: &Meta,
    has_direct_helper: &mut bool,
    has_module_helper: &mut bool,
) {
    match meta {
        Meta::Path(_) => {}
        Meta::NameValue(name_value) => {
            classify_uniffi_helper_meta_key(
                &format_path(&name_value.path),
                has_direct_helper,
                has_module_helper,
            );
        }
        Meta::List(list) => {
            classify_uniffi_helper_meta_key(
                &format_path(&list.path),
                has_direct_helper,
                has_module_helper,
            );
            let Ok(arguments) =
                Punctuated::<Meta, syn::Token![,]>::parse_terminated.parse2(list.tokens.clone())
            else {
                return;
            };
            for nested in arguments {
                collect_uniffi_helper_attribute_shape(
                    &nested,
                    has_direct_helper,
                    has_module_helper,
                );
            }
        }
    }
}

fn classify_uniffi_helper_meta_key(
    key: &str,
    has_direct_helper: &mut bool,
    has_module_helper: &mut bool,
) {
    match key {
        "default" => {
            *has_direct_helper = true;
        }
        "custom" | "with" => {
            *has_module_helper = true;
        }
        _ => {}
    }
}

fn modeled_uniffi_attribute(attribute: &Attribute, context: &MacroInvocationContext) -> bool {
    let path = attribute.path();
    let Some(last) = path.segments.last() else {
        return false;
    };
    let name = last.ident.to_string();
    if !matches!(name.as_str(), "constructor" | "export") {
        return false;
    }
    if path.segments.len() == 1 {
        return context.uniffi_attribute_imports.contains(&name)
            && !context.local_macro_definitions.contains(&name);
    }
    path.segments
        .first()
        .is_some_and(|root| context.uniffi_macro_roots.contains(&root.ident.to_string()))
}

fn modeled_async_trait_attribute(attribute: &Attribute, context: &MacroInvocationContext) -> bool {
    let path = attribute.path();
    let Some(last) = path.segments.last() else {
        return false;
    };
    if last.ident != "async_trait" {
        return false;
    }
    if path.segments.len() == 1 {
        let local = last.ident.to_string();
        return context.async_trait_attribute_imports.contains(&local)
            && !context.local_macro_definitions.contains(&local);
    }
    path.segments.first().is_some_and(|root| {
        context
            .async_trait_macro_roots
            .contains(&root.ident.to_string())
    })
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
            | "unsafe"
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
    rendered_callables: Vec<String>,
    rendered_items: Vec<String>,
    macro_surfaces: MacroSurfaceReportJson,
    usage: UsageClassificationReportJson,
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
            rendered_callables: rendered_symbol_ids(&report.usage.rendered_symbols, "callable"),
            rendered_items: rendered_symbol_ids(&report.usage.rendered_symbols, "item"),
            macro_surfaces: MacroSurfaceReportJson::from_report(&report.macro_surfaces),
            usage: UsageClassificationReportJson::from_report(&report.usage),
            source_map: SourceMapReportJson::from_report(&report.source_map),
            files_written: report.files_written,
        }
    }
}

fn rendered_symbol_ids(report: &RenderedSymbolProofReport, kind: &str) -> Vec<String> {
    report
        .entries
        .iter()
        .filter(|entry| entry.kind == kind)
        .map(|entry| entry.id.clone())
        .collect()
}

#[derive(Serialize)]
struct MacroSurfaceReportJson {
    summary: MacroSurfaceSummaryJson,
    surfaces: Vec<MacroSurfaceJson>,
}

impl MacroSurfaceReportJson {
    fn from_report(report: &MacroSurfaceReport) -> Self {
        Self {
            summary: MacroSurfaceSummaryJson::from_report(&report.summary),
            surfaces: report
                .surfaces
                .iter()
                .map(MacroSurfaceJson::from_report)
                .collect(),
        }
    }
}

#[derive(Serialize)]
struct MacroSurfaceSummaryJson {
    total: usize,
    derive_macros: usize,
    attribute_macros: usize,
    helper_attributes: usize,
    macro_invocations: usize,
    macro_blocked: usize,
}

impl MacroSurfaceSummaryJson {
    fn from_report(summary: &MacroSurfaceSummary) -> Self {
        Self {
            total: summary.total,
            derive_macros: summary.derive_macros,
            attribute_macros: summary.attribute_macros,
            helper_attributes: summary.helper_attributes,
            macro_invocations: summary.macro_invocations,
            macro_blocked: summary.macro_blocked,
        }
    }
}

#[derive(Serialize)]
struct MacroSurfaceJson {
    kind: String,
    category: String,
    path: String,
    subject: String,
    package: String,
    module_path: Option<String>,
    owner: Option<String>,
    file: Option<PathBuf>,
    start_line: Option<usize>,
    blocked_idents: Vec<String>,
}

impl MacroSurfaceJson {
    fn from_report(surface: &MacroSurface) -> Self {
        Self {
            kind: surface.kind.clone(),
            category: surface.category.clone(),
            path: surface.path.clone(),
            subject: surface.subject.clone(),
            package: surface.package.clone(),
            module_path: surface.module_path.clone(),
            owner: surface.owner.clone(),
            file: surface.file.clone(),
            start_line: surface.start_line,
            blocked_idents: surface.blocked_idents.clone(),
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
struct UsageClassificationReportJson {
    status: String,
    summary: UsageClassificationSummaryJson,
    semantic_proof: SemanticUsageProofReportJson,
    rendered_symbols: RenderedSymbolProofReportJson,
    rendered_decision_map: RenderedUsageDecisionMapJson,
    public_reexports: PublicReexportProofReportJson,
    decision_map: UsageDecisionMapJson,
    used: UsageClassifiedItemsJson,
    unused_candidate: UsageClassifiedItemsJson,
    blocked_by_unknown: UsageClassifiedItemsJson,
    prunable: UsageClassifiedItemsJson,
    unused: UsageClassifiedItemsJson,
    unknown: Vec<UsageUnknownSurfaceJson>,
    evidence: UsageClassificationEvidenceJson,
}

impl UsageClassificationReportJson {
    fn from_report(report: &UsageClassificationReport) -> Self {
        Self {
            status: report.status.clone(),
            summary: UsageClassificationSummaryJson::from_report(&report.summary),
            semantic_proof: SemanticUsageProofReportJson::from_report(&report.semantic_proof),
            rendered_symbols: RenderedSymbolProofReportJson::from_report(&report.rendered_symbols),
            rendered_decision_map: RenderedUsageDecisionMapJson::from_report(
                &report.rendered_decision_map,
            ),
            public_reexports: PublicReexportProofReportJson::from_report(&report.public_reexports),
            decision_map: UsageDecisionMapJson::from_report(report),
            used: UsageClassifiedItemsJson::from_report(&report.used),
            unused_candidate: UsageClassifiedItemsJson::from_report(&report.unused_candidate),
            blocked_by_unknown: UsageClassifiedItemsJson::from_report(&report.blocked_by_unknown),
            prunable: UsageClassifiedItemsJson::from_report(&report.prunable),
            unused: UsageClassifiedItemsJson::from_report(&report.unused),
            unknown: report
                .unknown
                .iter()
                .map(UsageUnknownSurfaceJson::from_report)
                .collect(),
            evidence: UsageClassificationEvidenceJson::from_report(&report.evidence),
        }
    }
}

#[derive(Serialize)]
struct RenderedSymbolProofReportJson {
    status: String,
    summary: RenderedSymbolProofSummaryJson,
    entries: Vec<RenderedSymbolProofEntryJson>,
}

impl RenderedSymbolProofReportJson {
    fn from_report(report: &RenderedSymbolProofReport) -> Self {
        Self {
            status: report.status.clone(),
            summary: RenderedSymbolProofSummaryJson::from_report(&report.summary),
            entries: report
                .entries
                .iter()
                .map(RenderedSymbolProofEntryJson::from_report)
                .collect(),
        }
    }
}

#[derive(Serialize)]
struct RenderedSymbolProofSummaryJson {
    rendered_callables: usize,
    rendered_items: usize,
    rendered_members: usize,
    rendered_assoc_items: usize,
    rendered_trait_default_methods: usize,
    retained_callables: usize,
    retained_items: usize,
    retained_members: usize,
    retained_assoc_items: usize,
    retained_trait_default_methods: usize,
    macro_blocked_callables: usize,
    surface_blocked_callables: usize,
    blocked_members: usize,
    blocked_assoc_items: usize,
    blocked_trait_default_methods: usize,
    prunable_callables: usize,
    prunable_items: usize,
    prunable_members: usize,
    prunable_assoc_items: usize,
    unclassified_callables: usize,
    unclassified_items: usize,
    unclassified_members: usize,
    unclassified_assoc_items: usize,
    unproven_trait_default_methods: usize,
    source_parse_failures: usize,
}

impl RenderedSymbolProofSummaryJson {
    fn from_report(summary: &RenderedSymbolProofSummary) -> Self {
        Self {
            rendered_callables: summary.rendered_callables,
            rendered_items: summary.rendered_items,
            rendered_members: summary.rendered_members,
            rendered_assoc_items: summary.rendered_assoc_items,
            rendered_trait_default_methods: summary.rendered_trait_default_methods,
            retained_callables: summary.retained_callables,
            retained_items: summary.retained_items,
            retained_members: summary.retained_members,
            retained_assoc_items: summary.retained_assoc_items,
            retained_trait_default_methods: summary.retained_trait_default_methods,
            macro_blocked_callables: summary.macro_blocked_callables,
            surface_blocked_callables: summary.surface_blocked_callables,
            blocked_members: summary.blocked_members,
            blocked_assoc_items: summary.blocked_assoc_items,
            blocked_trait_default_methods: summary.blocked_trait_default_methods,
            prunable_callables: summary.prunable_callables,
            prunable_items: summary.prunable_items,
            prunable_members: summary.prunable_members,
            prunable_assoc_items: summary.prunable_assoc_items,
            unclassified_callables: summary.unclassified_callables,
            unclassified_items: summary.unclassified_items,
            unclassified_members: summary.unclassified_members,
            unclassified_assoc_items: summary.unclassified_assoc_items,
            unproven_trait_default_methods: summary.unproven_trait_default_methods,
            source_parse_failures: summary.source_parse_failures,
        }
    }
}

#[derive(Serialize)]
struct RenderedSymbolProofEntryJson {
    kind: String,
    id: String,
    classification: String,
}

impl RenderedSymbolProofEntryJson {
    fn from_report(entry: &RenderedSymbolProofEntry) -> Self {
        Self {
            kind: entry.kind.clone(),
            id: entry.id.clone(),
            classification: entry.classification.clone(),
        }
    }
}

#[derive(Serialize)]
struct RenderedUsageDecisionMapJson {
    callables: BTreeMap<String, String>,
    items: BTreeMap<String, String>,
    members: BTreeMap<String, String>,
    assoc_items: BTreeMap<String, String>,
    trait_default_methods: BTreeMap<String, String>,
}

impl RenderedUsageDecisionMapJson {
    fn from_report(report: &RenderedUsageDecisionMap) -> Self {
        Self {
            callables: report.callables.clone(),
            items: report.items.clone(),
            members: report.members.clone(),
            assoc_items: report.assoc_items.clone(),
            trait_default_methods: report.trait_default_methods.clone(),
        }
    }
}

#[derive(Serialize)]
struct PublicReexportProofReportJson {
    status: String,
    summary: PublicReexportProofSummaryJson,
    entries: Vec<PublicReexportProofEntryJson>,
}

impl PublicReexportProofReportJson {
    fn from_report(report: &PublicReexportProofReport) -> Self {
        Self {
            status: report.status.clone(),
            summary: PublicReexportProofSummaryJson::from_report(&report.summary),
            entries: report
                .entries
                .iter()
                .map(PublicReexportProofEntryJson::from_report)
                .collect(),
        }
    }
}

#[derive(Serialize)]
struct PublicReexportProofSummaryJson {
    public_reexports: usize,
    local_targets: usize,
    retained_targets: usize,
    prunable_targets: usize,
    unclassified_targets: usize,
    external_or_unresolved_targets: usize,
    facade_chain_targets: usize,
    source_parse_failures: usize,
}

impl PublicReexportProofSummaryJson {
    fn from_report(summary: &PublicReexportProofSummary) -> Self {
        Self {
            public_reexports: summary.public_reexports,
            local_targets: summary.local_targets,
            retained_targets: summary.retained_targets,
            prunable_targets: summary.prunable_targets,
            unclassified_targets: summary.unclassified_targets,
            external_or_unresolved_targets: summary.external_or_unresolved_targets,
            facade_chain_targets: summary.facade_chain_targets,
            source_parse_failures: summary.source_parse_failures,
        }
    }
}

#[derive(Serialize)]
struct PublicReexportProofEntryJson {
    package: String,
    module_path: Option<String>,
    visible: String,
    target: String,
    resolved_targets: Vec<String>,
    classification: String,
}

impl PublicReexportProofEntryJson {
    fn from_report(entry: &PublicReexportProofEntry) -> Self {
        Self {
            package: entry.package.clone(),
            module_path: entry.module_path.clone(),
            visible: entry.visible.clone(),
            target: entry.target.clone(),
            resolved_targets: entry.resolved_targets.clone(),
            classification: entry.classification.clone(),
        }
    }
}

#[derive(Serialize)]
struct SemanticUsageProofReportJson {
    status: String,
    summary: SemanticUsageProofSummaryJson,
    unproven: UsageClassifiedItemsJson,
}

impl SemanticUsageProofReportJson {
    fn from_report(report: &SemanticUsageProofReport) -> Self {
        Self {
            status: report.status.clone(),
            summary: SemanticUsageProofSummaryJson::from_report(&report.summary),
            unproven: UsageClassifiedItemsJson::from_report(&report.unproven),
        }
    }
}

#[derive(Serialize)]
struct SemanticUsageProofSummaryJson {
    analyzer_available: bool,
    retained_packages: usize,
    prunable_callables: usize,
    prunable_items: usize,
    package_pruned_callables: usize,
    package_pruned_items: usize,
    proof_required_callables: usize,
    proof_required_items: usize,
    proven_callables: usize,
    proven_items: usize,
    unproven_callables: usize,
    unproven_items: usize,
    cfg_inactive_callables: usize,
    cfg_inactive_items: usize,
    source_file_pruned_callables: usize,
    source_file_pruned_items: usize,
    rendered_absent_callables: usize,
    rendered_absent_items: usize,
    structural_pruned_items: usize,
    unmapped_callables: usize,
    unmapped_items: usize,
    failed_reference_query_callables: usize,
    failed_reference_query_items: usize,
    skipped_reference_query_callables: usize,
    skipped_reference_query_items: usize,
    retained_reference_callables: usize,
    retained_reference_items: usize,
}

impl SemanticUsageProofSummaryJson {
    fn from_report(summary: &SemanticUsageProofSummary) -> Self {
        Self {
            analyzer_available: summary.analyzer_available,
            retained_packages: summary.retained_packages,
            prunable_callables: summary.prunable_callables,
            prunable_items: summary.prunable_items,
            package_pruned_callables: summary.package_pruned_callables,
            package_pruned_items: summary.package_pruned_items,
            proof_required_callables: summary.proof_required_callables,
            proof_required_items: summary.proof_required_items,
            proven_callables: summary.proven_callables,
            proven_items: summary.proven_items,
            unproven_callables: summary.unproven_callables,
            unproven_items: summary.unproven_items,
            cfg_inactive_callables: summary.cfg_inactive_callables,
            cfg_inactive_items: summary.cfg_inactive_items,
            source_file_pruned_callables: summary.source_file_pruned_callables,
            source_file_pruned_items: summary.source_file_pruned_items,
            rendered_absent_callables: summary.rendered_absent_callables,
            rendered_absent_items: summary.rendered_absent_items,
            structural_pruned_items: summary.structural_pruned_items,
            unmapped_callables: summary.unmapped_callables,
            unmapped_items: summary.unmapped_items,
            failed_reference_query_callables: summary.failed_reference_query_callables,
            failed_reference_query_items: summary.failed_reference_query_items,
            skipped_reference_query_callables: summary.skipped_reference_query_callables,
            skipped_reference_query_items: summary.skipped_reference_query_items,
            retained_reference_callables: summary.retained_reference_callables,
            retained_reference_items: summary.retained_reference_items,
        }
    }
}

#[derive(Serialize)]
struct UsageDecisionMapJson {
    callables: BTreeMap<String, String>,
    items: BTreeMap<String, String>,
}

impl UsageDecisionMapJson {
    fn from_report(report: &UsageClassificationReport) -> Self {
        let mut callables = BTreeMap::new();
        let mut items = BTreeMap::new();
        insert_usage_decisions(&mut callables, &report.used.callables, UsageDecision::Used);
        insert_usage_decisions(
            &mut callables,
            &report.blocked_by_unknown.callables,
            UsageDecision::BlockedByUnknown,
        );
        insert_usage_decisions(
            &mut callables,
            &report.prunable.callables,
            UsageDecision::Prunable,
        );
        insert_usage_decisions(&mut items, &report.used.items, UsageDecision::Used);
        insert_usage_decisions(
            &mut items,
            &report.blocked_by_unknown.items,
            UsageDecision::BlockedByUnknown,
        );
        insert_usage_decisions(&mut items, &report.prunable.items, UsageDecision::Prunable);
        Self { callables, items }
    }
}

fn insert_usage_decisions<T: ToString>(
    decisions: &mut BTreeMap<String, String>,
    ids: &[T],
    decision: UsageDecision,
) {
    for id in ids {
        let previous = decisions.insert(id.to_string(), decision.as_str().to_string());
        debug_assert!(previous.is_none());
    }
}

#[derive(Serialize)]
struct UsageClassificationSummaryJson {
    indexed_callables: usize,
    indexed_items: usize,
    used_callables: usize,
    used_items: usize,
    unused_candidate_callables: usize,
    unused_candidate_items: usize,
    blocked_by_unknown_callables: usize,
    blocked_by_unknown_items: usize,
    prunable_callables: usize,
    prunable_items: usize,
    unused_callables: usize,
    unused_items: usize,
    unknown_surfaces: usize,
    benign_unknown_surfaces: usize,
    macro_blocked_unknown_surfaces: usize,
    dependency_risk_unknown_surfaces: usize,
}

impl UsageClassificationSummaryJson {
    fn from_report(summary: &UsageClassificationSummary) -> Self {
        Self {
            indexed_callables: summary.indexed_callables,
            indexed_items: summary.indexed_items,
            used_callables: summary.used_callables,
            used_items: summary.used_items,
            unused_candidate_callables: summary.unused_candidate_callables,
            unused_candidate_items: summary.unused_candidate_items,
            blocked_by_unknown_callables: summary.blocked_by_unknown_callables,
            blocked_by_unknown_items: summary.blocked_by_unknown_items,
            prunable_callables: summary.prunable_callables,
            prunable_items: summary.prunable_items,
            unused_callables: summary.unused_callables,
            unused_items: summary.unused_items,
            unknown_surfaces: summary.unknown_surfaces,
            benign_unknown_surfaces: summary.benign_unknown_surfaces,
            macro_blocked_unknown_surfaces: summary.macro_blocked_unknown_surfaces,
            dependency_risk_unknown_surfaces: summary.dependency_risk_unknown_surfaces,
        }
    }
}

#[derive(Serialize)]
struct UsageClassifiedItemsJson {
    callables: Vec<String>,
    items: Vec<String>,
}

impl UsageClassifiedItemsJson {
    fn from_report(items: &UsageClassifiedItems) -> Self {
        Self {
            callables: items.callables.iter().map(ToString::to_string).collect(),
            items: items.items.iter().map(ToString::to_string).collect(),
        }
    }
}

#[derive(Serialize)]
struct UsageClassificationEvidenceJson {
    callables: Vec<UsageCallableEvidenceJson>,
    items: Vec<UsageItemEvidenceJson>,
}

impl UsageClassificationEvidenceJson {
    fn from_report(evidence: &UsageClassificationEvidence) -> Self {
        Self {
            callables: evidence
                .callables
                .iter()
                .map(UsageCallableEvidenceJson::from_report)
                .collect(),
            items: evidence
                .items
                .iter()
                .map(UsageItemEvidenceJson::from_report)
                .collect(),
        }
    }
}

#[derive(Serialize)]
struct UsageCallableEvidenceJson {
    id: String,
    classification: String,
    selected_root: bool,
    reason: String,
    evidence: Vec<String>,
}

impl UsageCallableEvidenceJson {
    fn from_report(entry: &UsageCallableEvidence) -> Self {
        Self {
            id: entry.id.to_string(),
            classification: entry.classification.clone(),
            selected_root: entry.selected_root,
            reason: entry.reason.clone(),
            evidence: entry.evidence.clone(),
        }
    }
}

#[derive(Serialize)]
struct UsageItemEvidenceJson {
    id: String,
    classification: String,
    selected_root: bool,
    reason: String,
    evidence: Vec<String>,
}

impl UsageItemEvidenceJson {
    fn from_report(entry: &UsageItemEvidence) -> Self {
        Self {
            id: entry.id.to_string(),
            classification: entry.classification.clone(),
            selected_root: entry.selected_root,
            reason: entry.reason.clone(),
            evidence: entry.evidence.clone(),
        }
    }
}

#[derive(Serialize)]
struct UsageUnknownSurfaceJson {
    category: String,
    code: String,
    severity: String,
    message: String,
    details: Vec<ProductionHazardDetail>,
}

impl UsageUnknownSurfaceJson {
    fn from_report(surface: &UsageUnknownSurface) -> Self {
        Self {
            category: surface.category.clone(),
            code: surface.code.clone(),
            severity: surface.severity.clone(),
            message: surface.message.clone(),
            details: surface.details.clone(),
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
    semantic_usage: Option<SemanticUsageReportJson>,
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
            semantic_usage: report
                .semantic_usage
                .as_ref()
                .map(SemanticUsageReportJson::from_report),
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
struct SemanticUsageReportJson {
    indexed_callables: usize,
    indexed_items: usize,
    mapped_callables: usize,
    mapped_items: usize,
    unmapped_callables: usize,
    unmapped_items: usize,
    reference_queries: usize,
    reference_queries_skipped: usize,
    reference_query_failures: usize,
    callable_reference_edges: usize,
    item_reference_edges: usize,
    referenced_callables: usize,
    referenced_items: usize,
    mapped_callable_ids: Vec<String>,
    mapped_item_ids: Vec<String>,
    queried_callable_reference_ids: Vec<String>,
    queried_item_reference_ids: Vec<String>,
    skipped_callable_reference_ids: Vec<String>,
    skipped_item_reference_ids: Vec<String>,
    failed_callable_reference_ids: Vec<String>,
    failed_item_reference_ids: Vec<String>,
    referenced_callable_ids: Vec<String>,
    referenced_item_ids: Vec<String>,
    callable_reference_owners: Vec<SemanticCallableReferenceOwnersJson>,
    item_reference_owners: Vec<SemanticItemReferenceOwnersJson>,
    callable_unowned_reference_files: Vec<SemanticCallableReferenceFilesJson>,
    item_unowned_reference_files: Vec<SemanticItemReferenceFilesJson>,
}

#[derive(Serialize)]
struct SemanticCallableReferenceOwnersJson {
    target: String,
    owners: Vec<String>,
}

#[derive(Serialize)]
struct SemanticItemReferenceOwnersJson {
    target: String,
    owners: Vec<String>,
}

#[derive(Serialize)]
struct SemanticCallableReferenceFilesJson {
    target: String,
    files: Vec<PathBuf>,
}

#[derive(Serialize)]
struct SemanticItemReferenceFilesJson {
    target: String,
    files: Vec<PathBuf>,
}

impl SemanticUsageReportJson {
    fn from_report(report: &SemanticUsageReport) -> Self {
        Self {
            indexed_callables: report.indexed_callables,
            indexed_items: report.indexed_items,
            mapped_callables: report.mapped_callables,
            mapped_items: report.mapped_items,
            unmapped_callables: report.unmapped_callables,
            unmapped_items: report.unmapped_items,
            reference_queries: report.reference_queries,
            reference_queries_skipped: report.reference_queries_skipped,
            reference_query_failures: report.reference_query_failures,
            callable_reference_edges: report.callable_reference_edges,
            item_reference_edges: report.item_reference_edges,
            referenced_callables: report.referenced_callables,
            referenced_items: report.referenced_items,
            mapped_callable_ids: report
                .mapped_callable_ids
                .iter()
                .map(ToString::to_string)
                .collect(),
            mapped_item_ids: report
                .mapped_item_ids
                .iter()
                .map(ToString::to_string)
                .collect(),
            queried_callable_reference_ids: report
                .queried_callable_reference_ids
                .iter()
                .map(ToString::to_string)
                .collect(),
            queried_item_reference_ids: report
                .queried_item_reference_ids
                .iter()
                .map(ToString::to_string)
                .collect(),
            skipped_callable_reference_ids: report
                .skipped_callable_reference_ids
                .iter()
                .map(ToString::to_string)
                .collect(),
            skipped_item_reference_ids: report
                .skipped_item_reference_ids
                .iter()
                .map(ToString::to_string)
                .collect(),
            failed_callable_reference_ids: report
                .failed_callable_reference_ids
                .iter()
                .map(ToString::to_string)
                .collect(),
            failed_item_reference_ids: report
                .failed_item_reference_ids
                .iter()
                .map(ToString::to_string)
                .collect(),
            referenced_callable_ids: report
                .referenced_callable_ids
                .iter()
                .map(ToString::to_string)
                .collect(),
            referenced_item_ids: report
                .referenced_item_ids
                .iter()
                .map(ToString::to_string)
                .collect(),
            callable_reference_owners: report
                .callable_reference_owners
                .iter()
                .map(|(target, owners)| SemanticCallableReferenceOwnersJson {
                    target: target.to_string(),
                    owners: semantic_owner_ids_to_strings(owners),
                })
                .collect(),
            item_reference_owners: report
                .item_reference_owners
                .iter()
                .map(|(target, owners)| SemanticItemReferenceOwnersJson {
                    target: target.to_string(),
                    owners: semantic_owner_ids_to_strings(owners),
                })
                .collect(),
            callable_unowned_reference_files: report
                .callable_unowned_reference_files
                .iter()
                .map(|(target, files)| SemanticCallableReferenceFilesJson {
                    target: target.to_string(),
                    files: files.iter().cloned().collect(),
                })
                .collect(),
            item_unowned_reference_files: report
                .item_unowned_reference_files
                .iter()
                .map(|(target, files)| SemanticItemReferenceFilesJson {
                    target: target.to_string(),
                    files: files.iter().cloned().collect(),
                })
                .collect(),
        }
    }
}

fn semantic_owner_ids_to_strings(owners: &BTreeSet<SemanticOwnerId>) -> Vec<String> {
    owners
        .iter()
        .map(|owner| match owner {
            SemanticOwnerId::Callable(callable) => callable.to_string(),
            SemanticOwnerId::Item(item) => item.to_string(),
        })
        .collect()
}

#[derive(Serialize)]
struct SemanticReportJson {
    source_files: usize,
    analyzed_files: usize,
    failed_files: usize,
    skipped_files: usize,
    top_down_skipped_files: usize,
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
    selected_root_source_files: usize,
    selected_root_analyzed_files: usize,
    selected_root_failed_files: usize,
    selected_root_skipped_files: usize,
    selected_root_method_calls: usize,
    selected_root_queried_method_calls: usize,
    selected_root_resolved_method_calls: usize,
    selected_root_callable_method_calls: usize,
    selected_root_fallback_method_calls: usize,
    selected_root_unresolved_method_calls: usize,
    selected_root_unqueried_method_calls: usize,
    selected_root_paths: usize,
    selected_root_queried_paths: usize,
    selected_root_resolved_paths: usize,
    selected_root_unresolved_paths: usize,
    selected_root_unqueried_paths: usize,
    unresolved_diagnostics: Vec<SemanticUnresolvedDiagnosticJson>,
    file_reports: Vec<SemanticFileReportJson>,
}

#[derive(Serialize)]
struct SemanticFileReportJson {
    path: PathBuf,
    selected_root_file: bool,
    analyzed: bool,
    failed: bool,
    skipped_by_file_budget: bool,
    skipped_by_top_down_scope: bool,
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
    unresolved_diagnostics: Vec<SemanticUnresolvedDiagnosticJson>,
}

#[derive(Serialize)]
struct SemanticUnresolvedDiagnosticJson {
    kind: String,
    category: String,
    reason: String,
    file: PathBuf,
    start_line: usize,
    start_column: usize,
    end_line: usize,
    end_column: usize,
    snippet: String,
    ast_kind: String,
    symbol: Option<String>,
    owner: Option<String>,
}

impl SemanticReportJson {
    fn from_report(report: &SemanticReport) -> Self {
        Self {
            source_files: report.source_files,
            analyzed_files: report.analyzed_files,
            failed_files: report.failed_files,
            skipped_files: report.skipped_files,
            top_down_skipped_files: report.top_down_skipped_files,
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
            selected_root_source_files: report.selected_root_source_files,
            selected_root_analyzed_files: report.selected_root_analyzed_files,
            selected_root_failed_files: report.selected_root_failed_files,
            selected_root_skipped_files: report.selected_root_skipped_files,
            selected_root_method_calls: report.selected_root_method_calls,
            selected_root_queried_method_calls: report.selected_root_queried_method_calls,
            selected_root_resolved_method_calls: report.selected_root_resolved_method_calls,
            selected_root_callable_method_calls: report.selected_root_callable_method_calls,
            selected_root_fallback_method_calls: report.selected_root_fallback_method_calls,
            selected_root_unresolved_method_calls: report.selected_root_unresolved_method_calls,
            selected_root_unqueried_method_calls: report.selected_root_unqueried_method_calls,
            selected_root_paths: report.selected_root_paths,
            selected_root_queried_paths: report.selected_root_queried_paths,
            selected_root_resolved_paths: report.selected_root_resolved_paths,
            selected_root_unresolved_paths: report.selected_root_unresolved_paths,
            selected_root_unqueried_paths: report.selected_root_unqueried_paths,
            unresolved_diagnostics: report
                .unresolved_diagnostics
                .iter()
                .map(SemanticUnresolvedDiagnosticJson::from_report)
                .collect(),
            file_reports: report
                .file_reports
                .iter()
                .map(SemanticFileReportJson::from_report)
                .collect(),
        }
    }
}

impl SemanticFileReportJson {
    fn from_report(report: &SemanticFileReport) -> Self {
        Self {
            path: report.path.clone(),
            selected_root_file: report.selected_root_file,
            analyzed: report.analyzed,
            failed: report.failed,
            skipped_by_file_budget: report.skipped_by_file_budget,
            skipped_by_top_down_scope: report.skipped_by_top_down_scope,
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
            unresolved_diagnostics: report
                .unresolved_diagnostics
                .iter()
                .map(SemanticUnresolvedDiagnosticJson::from_report)
                .collect(),
        }
    }
}

impl SemanticUnresolvedDiagnosticJson {
    fn from_report(report: &SemanticUnresolvedDiagnostic) -> Self {
        Self {
            kind: report.kind.as_str().to_string(),
            category: report.category.as_str().to_string(),
            reason: report.reason.clone(),
            file: report.file.clone(),
            start_line: report.start_line,
            start_column: report.start_column,
            end_line: report.end_line,
            end_column: report.end_column,
            snippet: report.snippet.clone(),
            ast_kind: report.ast_kind.clone(),
            symbol: report.symbol.clone(),
            owner: report.owner.as_ref().map(semantic_owner_to_string),
        }
    }
}

fn semantic_owner_to_string(owner: &SemanticOwnerId) -> String {
    match owner {
        SemanticOwnerId::Callable(callable) => callable.to_string(),
        SemanticOwnerId::Item(item) => item.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{BTreeMap, BTreeSet, HashMap, HashSet},
        fs,
        path::{Path, PathBuf},
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[cfg(feature = "ra-hir")]
    use super::generate_with_analyzer;
    use super::model::{ItemKind, ReductionEvidence};
    use super::non_benign_unresolved_count;
    use super::{
        add_public_reexport_proof_hazards, add_rendered_symbol_proof_hazards,
        add_semantic_inventory_hazard, covered_project_path_unresolved_diagnostic,
        default_feature_closure, direct_free_function_root_selectors, generate,
        generate_with_analyzer_feedback, generate_with_analyzer_roots,
        generated_package_source_roots, production_hazard_with_details,
        production_readiness_report, production_readiness_status, public_reexport_proof_report,
        public_reexport_proof_report_with_retained_surface_items, rendered_symbol_proof_report,
        rendered_symbol_proof_report_with_members, resolve_feedback_widening_roots,
        semantic_hazard_metrics, semantic_unresolved_details,
        semantic_unresolved_owner_is_retained, semantic_usage_proof_report,
        unknown_surface_category, usage_classification_report, usage_evidence_reason,
        usage_guarded_render_reduction, write_generate_report, AnalyzerMode, AnalyzerReport,
        CallableId, CheckDiagnostic, CheckSpan, GenerateOptions, ItemId, ProductionHazardDetail,
        PublicReexportProofEntry, PublicReexportProofReport, PublicReexportProofSummary,
        ReducedProject, RenderedSymbolProofEntry, RenderedSymbolProofReport,
        RenderedSymbolProofSummary, RootId, SemanticFileReport, SemanticHazardScope,
        SemanticOwnerId, SemanticReductionHints, SemanticReport, SemanticUnresolvedCategory,
        SemanticUnresolvedDiagnostic, SemanticUnresolvedKind, SemanticUsageReport, SourceSpan,
        UsageDecision, UsageDecisionIndex,
    };
    use super::{manifest, parse, reduce, render};

    #[test]
    fn direct_free_function_roots_decode_without_workspace_scan() {
        let roots = direct_free_function_root_selectors(&[
            "codex-tui::theme::health_color".to_string(),
            "codex-tui::theme::health_symbol".to_string(),
            "codex-tui::theme::health_color".to_string(),
        ])
        .expect("fully qualified free functions should decode directly");

        assert_eq!(
            roots.iter().map(ToString::to_string).collect::<Vec<_>>(),
            [
                "codex-tui::theme::health_color",
                "codex-tui::theme::health_symbol"
            ]
        );
        assert!(
            direct_free_function_root_selectors(&["theme::health_color".to_string()]).is_none()
        );
        assert!(direct_free_function_root_selectors(&["app::Type::method".to_string()]).is_none());
        assert!(
            direct_free_function_root_selectors(&["app::<Type as Trait>::method".to_string()])
                .is_none()
        );
    }

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
        let usage_used_callables = report
            .usage
            .used
            .callables
            .iter()
            .map(ToString::to_string)
            .collect::<BTreeSet<_>>();
        let usage_unused_callables = report
            .usage
            .unused
            .callables
            .iter()
            .map(ToString::to_string)
            .collect::<BTreeSet<_>>();
        assert!(usage_used_callables.contains("a::open_source_entry"));
        assert!(usage_used_callables.contains("b::helper"));
        assert!(usage_unused_callables.contains("a::internal_entry"));
        assert!(usage_unused_callables.contains("b::unused_public"));
        let usage_unused_candidate_callables = report
            .usage
            .unused_candidate
            .callables
            .iter()
            .map(ToString::to_string)
            .collect::<BTreeSet<_>>();
        let usage_blocked_callables = report
            .usage
            .blocked_by_unknown
            .callables
            .iter()
            .map(ToString::to_string)
            .collect::<BTreeSet<_>>();
        let usage_prunable_callables = report
            .usage
            .prunable
            .callables
            .iter()
            .map(ToString::to_string)
            .collect::<BTreeSet<_>>();
        assert_eq!(usage_unused_callables, usage_unused_candidate_callables);
        assert!(usage_blocked_callables.is_empty());
        assert!(
            usage_prunable_callables.contains("b::unused_public"),
            "semantic inventory warnings alone should not block a graph-unreachable callable from being reported as prunable",
        );
        assert!(
            usage_used_callables.is_disjoint(&usage_unused_callables),
            "usage classifier must not classify a callable as both used and unused",
        );
        assert_eq!(
            report.usage.summary.indexed_callables,
            report.usage.summary.used_callables + report.usage.summary.unused_callables,
        );
        assert_eq!(
            report.usage.summary.unused_candidate_callables,
            report.usage.summary.unused_callables,
        );
        assert_eq!(report.usage.summary.blocked_by_unknown_callables, 0);
        assert_eq!(
            report.usage.summary.prunable_callables,
            report.usage.summary.unused_callables,
        );
        let root_evidence = report
            .usage
            .evidence
            .callables
            .iter()
            .find(|entry| entry.id.to_string() == "a::open_source_entry")
            .expect("root callable should have usage evidence");
        assert_eq!(root_evidence.classification, "used");
        assert!(root_evidence.selected_root);
        assert!(root_evidence.reason.contains("selected opensourced root"));
        assert!(root_evidence
            .evidence
            .iter()
            .any(|detail| detail == "selected_root=true"));
        let helper_evidence = report
            .usage
            .evidence
            .callables
            .iter()
            .find(|entry| entry.id.to_string() == "b::helper")
            .expect("reachable helper should have usage evidence");
        assert_eq!(helper_evidence.classification, "used");
        assert!(!helper_evidence.selected_root);
        assert!(helper_evidence
            .reason
            .contains("reachable from selected roots"));
        let unused_evidence = report
            .usage
            .evidence
            .callables
            .iter()
            .find(|entry| entry.id.to_string() == "b::unused_public")
            .expect("unused callable should have usage evidence");
        assert_eq!(unused_evidence.classification, "prunable");
        assert!(unused_evidence
            .reason
            .contains("not blocked by retained unknown surfaces"));
        assert!(unused_evidence
            .evidence
            .iter()
            .any(|detail| detail == "reachable=false"));
        assert!(unused_evidence
            .evidence
            .iter()
            .any(|detail| detail == "prunable=true"));
        assert_eq!(
            report.usage.evidence.callables.len(),
            report.usage.summary.indexed_callables,
        );

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
        let usage_used_items = report
            .usage
            .used
            .items
            .iter()
            .map(ToString::to_string)
            .collect::<BTreeSet<_>>();
        let usage_unused_items = report
            .usage
            .unused
            .items
            .iter()
            .map(ToString::to_string)
            .collect::<BTreeSet<_>>();
        assert!(usage_used_items.contains("d::Worker(Struct)"));
        assert!(usage_unused_items.contains("d::UnusedEnum(Enum)"));
        let usage_blocked_items = report
            .usage
            .blocked_by_unknown
            .items
            .iter()
            .map(ToString::to_string)
            .collect::<BTreeSet<_>>();
        assert!(usage_blocked_items.is_empty());
        assert!(
            report
                .usage
                .prunable
                .items
                .iter()
                .any(|item| item.to_string() == "d::UnusedEnum(Enum)"),
            "semantic inventory warnings alone should not block a graph-unreachable item from being reported as prunable",
        );
        assert!(
            usage_used_items.is_disjoint(&usage_unused_items),
            "usage classifier must not classify an item as both used and unused",
        );
        assert_eq!(
            report.usage.summary.indexed_items,
            report.usage.summary.used_items + report.usage.summary.unused_items,
        );
        assert_eq!(report.usage.summary.blocked_by_unknown_items, 0);
        assert_eq!(
            report.usage.summary.prunable_items,
            report.usage.summary.unused_items,
        );
        let item_evidence = report
            .usage
            .evidence
            .items
            .iter()
            .find(|entry| entry.id.to_string() == "d::Worker(Struct)")
            .expect("used item should have usage evidence");
        assert_eq!(item_evidence.classification, "used");
        assert!(item_evidence
            .reason
            .contains("reachable from selected roots"));
        let unused_item_evidence = report
            .usage
            .evidence
            .items
            .iter()
            .find(|entry| entry.id.to_string() == "d::UnusedEnum(Enum)")
            .expect("unused item should have usage evidence");
        assert_eq!(unused_item_evidence.classification, "prunable");
        assert_eq!(
            report.usage.evidence.items.len(),
            report.usage.summary.indexed_items,
        );
        assert_eq!(
            report.usage.status, "classified_with_unknowns",
            "syn generation should expose unknown semantic surfaces instead of pretending classification is complete",
        );
        assert!(report
            .usage
            .unknown
            .iter()
            .any(|surface| surface.code == "semantic_analyzer_unavailable"));

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
    fn usage_evidence_marks_prunable_only_without_unknowns() {
        let evidence = super::model::ReductionEvidence::default();
        let (reason, details) = usage_evidence_reason("prunable", false, &evidence, 0);

        assert!(reason.contains("not blocked by retained unknown surfaces"));
        assert!(details.iter().any(|detail| detail == "reachable=false"));
        assert!(details
            .iter()
            .any(|detail| detail == "unused_candidate=true"));
        assert!(details.iter().any(|detail| detail == "prunable=true"));
        assert!(details.iter().any(|detail| detail == "unknown_surfaces=0"));
    }

    #[test]
    fn usage_decision_index_blocks_scoped_unused_candidates_from_pruning() {
        let root = temp_output("usage-decision-blocker-source");
        let output = temp_output("usage-decision-blocker-output");
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

pub mod risky;
pub mod safe;

#[opensourced]
pub fn entry() -> i32 {
    risky::selected()
}
"#,
        );
        write(
            root.join("app/src/risky.rs"),
            r#"pub fn selected() -> i32 {
    1
}

pub fn maybe_macro_helper() -> i32 {
    private_leaf()
}

fn private_leaf() -> i32 {
    2
}
"#,
        );
        write(
            root.join("app/src/safe.rs"),
            r#"pub fn unrelated_dead_code() -> i32 {
    99
}
"#,
        );

        let workspace = manifest::load_workspace(&root).expect("workspace should load");
        let project = parse::parse_workspace(workspace).expect("workspace should parse");
        let reduced =
            reduce::reduce_with_extra_roots(&project, &[]).expect("initial reduction should work");
        let analyzer = AnalyzerReport {
            mode: AnalyzerMode::Syn,
            loaded: true,
            engine: "syn".to_string(),
            notes: Vec::new(),
            semantic: None,
            semantic_hints: SemanticReductionHints::default(),
            semantic_usage: None,
        };
        let blocker = production_readiness_status(vec![production_hazard_with_details(
            "custom_attribute_macros",
            "warning",
            "synthetic retained custom attribute may reference helper",
            vec![ProductionHazardDetail {
                subject: "app::risky: #[custom_attr::decorate(maybe_macro_helper)]".to_string(),
                package: Some("app".to_string()),
                module_path: Some("risky".to_string()),
                file: None,
                start_line: None,
                cfg: None,
                blocked_idents: vec!["maybe_macro_helper".to_string()],
                suggested_cargo_args: Vec::new(),
            }],
        )]);
        let (render_reduced, usage_decisions) =
            usage_guarded_render_reduction(&project, &reduced, &analyzer, &blocker)
                .expect("usage-guarded render reduction should work");
        render::write_reduced_workspace(&project, &render_reduced, &usage_decisions, &output)
            .expect("render should succeed");
        let usage = usage_classification_report(
            &project,
            &reduced,
            &analyzer,
            &usage_decisions,
            &blocker,
            RenderedSymbolProofReport::default(),
            PublicReexportProofReport::default(),
        );

        let used_callables = usage
            .used
            .callables
            .iter()
            .map(ToString::to_string)
            .collect::<BTreeSet<_>>();
        assert!(
            !used_callables.contains("app::risky::maybe_macro_helper"),
            "synthetic blocker must not relabel the candidate as used: {used_callables:?}",
        );
        let blocked_callables = usage
            .blocked_by_unknown
            .callables
            .iter()
            .map(ToString::to_string)
            .collect::<BTreeSet<_>>();
        assert!(
            blocked_callables.contains("app::risky::maybe_macro_helper"),
            "unused sibling named by a retained unknown macro surface should be retained as blocked_by_unknown: {blocked_callables:?}",
        );
        assert!(
            blocked_callables.contains("app::risky::private_leaf"),
            "dependencies of a blocked callable should also be retained as blocked_by_unknown: {blocked_callables:?}",
        );
        let prunable_callables = usage
            .prunable
            .callables
            .iter()
            .map(ToString::to_string)
            .collect::<BTreeSet<_>>();
        assert!(
            prunable_callables.contains("app::safe::unrelated_dead_code"),
            "unused code outside the unknown surface scope should remain prunable: {prunable_callables:?}",
        );
        let removable_callables = usage
            .unused
            .callables
            .iter()
            .map(ToString::to_string)
            .collect::<BTreeSet<_>>();
        assert_eq!(
            removable_callables, prunable_callables,
            "usage.unused is the public removable set and must match prunable only",
        );
        assert!(
            !removable_callables.contains("app::risky::maybe_macro_helper"),
            "blocked_by_unknown candidates must not leak into the removable unused set",
        );
        assert_eq!(
            usage.summary.unused_callables, usage.summary.prunable_callables,
            "unused summary count must describe removable/prunable code only",
        );
        assert_eq!(
            usage.summary.unused_candidate_callables,
            usage.summary.blocked_by_unknown_callables + usage.summary.prunable_callables,
            "unused_candidate remains the wider graph-unreachable set",
        );

        let risky_source = fs::read_to_string(output.join("app/src/risky.rs")).unwrap();
        assert!(risky_source.contains("maybe_macro_helper"));
        assert!(risky_source.contains("private_leaf"));
        assert!(
            !output.join("app/src/safe.rs").exists(),
            "prunable module outside the unknown surface scope should not be rendered",
        );
    }

    #[test]
    fn custom_macro_invocation_blockers_retain_token_named_unknowns() {
        let root = temp_output("custom-macro-invocation-blocker-source");
        let output = temp_output("custom-macro-invocation-blocker-output");
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

pub mod risky;
pub mod safe;

#[opensourced]
pub fn entry() -> i32 {
    risky::selected()
}
"#,
        );
        write(
            root.join("app/src/risky.rs"),
            r#"pub fn selected() -> i32 {
    1
}

pub fn macro_token_helper() -> i32 {
    private_leaf()
}

fn private_leaf() -> i32 {
    2
}
"#,
        );
        write(
            root.join("app/src/safe.rs"),
            r#"pub fn macro_token_helper() -> i32 {
    99
}

pub fn unrelated_dead_code() -> i32 {
    100
}
"#,
        );

        let workspace = manifest::load_workspace(&root).expect("workspace should load");
        let project = parse::parse_workspace(workspace).expect("workspace should parse");
        let reduced =
            reduce::reduce_with_extra_roots(&project, &[]).expect("initial reduction should work");
        let analyzer = AnalyzerReport {
            mode: AnalyzerMode::Syn,
            loaded: true,
            engine: "syn".to_string(),
            notes: Vec::new(),
            semantic: None,
            semantic_hints: SemanticReductionHints::default(),
            semantic_usage: None,
        };
        let blocker = production_readiness_status(vec![production_hazard_with_details(
            "custom_macro_invocations",
            "warning",
            "synthetic retained macro invocation may expand through token-named helper",
            vec![ProductionHazardDetail {
                subject: "app::risky: custom_macro!(macro_token_helper)".to_string(),
                package: Some("app".to_string()),
                module_path: Some("risky".to_string()),
                file: None,
                start_line: None,
                cfg: None,
                blocked_idents: vec!["macro_token_helper".to_string()],
                suggested_cargo_args: Vec::new(),
            }],
        )]);
        let (render_reduced, usage_decisions) =
            usage_guarded_render_reduction(&project, &reduced, &analyzer, &blocker)
                .expect("usage-guarded render reduction should work");
        render::write_reduced_workspace(&project, &render_reduced, &usage_decisions, &output)
            .expect("render should succeed");
        let usage = usage_classification_report(
            &project,
            &reduced,
            &analyzer,
            &usage_decisions,
            &blocker,
            RenderedSymbolProofReport::default(),
            PublicReexportProofReport::default(),
        );

        let blocked_callables = usage
            .blocked_by_unknown
            .callables
            .iter()
            .map(ToString::to_string)
            .collect::<BTreeSet<_>>();
        assert!(
            blocked_callables.contains("app::risky::macro_token_helper"),
            "token-named helper in the retained macro surface must stay blocked_by_unknown: {blocked_callables:?}",
        );
        assert!(
            !blocked_callables.contains("app::safe::macro_token_helper"),
            "same-name helpers outside the macro invocation module should stay prunable: {blocked_callables:?}",
        );

        let prunable_callables = usage
            .prunable
            .callables
            .iter()
            .map(ToString::to_string)
            .collect::<BTreeSet<_>>();
        assert!(
            prunable_callables.contains("app::safe::unrelated_dead_code"),
            "unrelated dead code must remain prunable: {prunable_callables:?}",
        );
        assert!(
            prunable_callables.contains("app::safe::macro_token_helper"),
            "macro blockers should be scoped to the macro invocation module: {prunable_callables:?}",
        );
        assert!(
            !usage
                .unused
                .callables
                .iter()
                .any(|callable| callable.to_string() == "app::risky::macro_token_helper"),
            "blocked_by_unknown macro helpers must not be exposed as removable unused code",
        );

        let risky_source = fs::read_to_string(output.join("app/src/risky.rs")).unwrap();
        assert!(risky_source.contains("macro_token_helper"));
        assert!(risky_source.contains("private_leaf"));
        assert!(!risky_source.contains("custom_macro"));
        assert!(
            !output.join("app/src/safe.rs").exists(),
            "same-name helper in an unrelated module should not be rendered",
        );
    }

    #[test]
    fn unsafe_block_helper_return_receiver_keeps_resolved_method() {
        let root = temp_output("unsafe-helper-return-source");
        let output = temp_output("unsafe-helper-return-output");
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
use std::ffi::c_void;
use std::sync::Arc;

pub struct Manager;

impl Manager {
    pub fn reset(&self) -> usize {
        helper()
    }

    pub fn unrelated(&self) -> usize {
        99
    }
}

fn helper() -> usize {
    1
}

unsafe fn arc_from_raw(handle: *mut c_void) -> Arc<Manager> {
    unsafe { Arc::from_raw(handle as *const Manager) }
}

#[opensourced]
pub fn entry(handle: *mut c_void) -> usize {
    let manager = unsafe { arc_from_raw(handle) };
    manager.reset()
}
"#,
        );

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: output.clone(),
        })
        .expect("reduction should succeed");
        let reachable = report
            .reachable
            .iter()
            .map(ToString::to_string)
            .collect::<BTreeSet<_>>();
        assert!(
            reachable.contains("app::Manager::reset"),
            "method receiver inferred through unsafe helper return should keep reset: {reachable:?}",
        );
        assert!(
            !reachable.contains("app::Manager::unrelated"),
            "unrelated methods on the same receiver type must remain prunable: {reachable:?}",
        );

        let rendered = fs::read_to_string(output.join("app/src/lib.rs")).unwrap();
        assert!(rendered.contains("pub fn reset"));
        assert!(rendered.contains("fn helper"));
        assert!(rendered.contains("fn arc_from_raw"));
        assert!(rendered.contains("unsafe { arc_from_raw(handle) }"));
        assert!(
            !rendered.contains("pub fn unrelated"),
            "receiver type inference must not retain unrelated methods: {rendered}",
        );
    }

    #[test]
    fn macro_surface_blockers_do_not_retain_macro_names_as_unknowns() {
        let root = temp_output("macro-surface-blocker-source");
        let output = temp_output("macro-surface-blocker-output");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\nserde = {{ version = \"1\", features = [\"derive\"] }}\n",
                opensourced_path
            ),
        );
        write(
            root.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;

#[opensourced]
pub fn entry() -> WireDto {
    WireDto { value: 1, optional: None, kind: 2 }
}

#[derive(serde::Serialize)]
pub struct WireDto {
    #[serde(with = "wire_helper")]
    pub value: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optional: Option<u32>,
    #[serde(rename = "resultType")]
    pub kind: u32,
}

pub mod wire_helper {
    pub fn serialize<S>(value: &u32, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u32(*value)
    }
}

pub fn Serialize() -> u32 {
    9
}

pub fn with() -> u32 {
    9
}

pub fn Option() -> u32 {
    9
}

pub fn is_none() -> u32 {
    9
}

pub fn resultType() -> u32 {
    9
}

pub fn unrelated_dead() -> u32 {
    10
}
"#,
        );

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: output.clone(),
        })
        .expect("generation should succeed");

        let helper_surface = report
            .macro_surfaces
            .surfaces
            .iter()
            .find(|surface| surface.kind == "helper_attribute" && surface.path == "serde")
            .expect("serde helper attribute should be indexed as a macro surface");
        assert_eq!(
            helper_surface.blocked_idents,
            vec!["wire_helper".to_string()]
        );
        assert!(
            report.macro_surfaces.surfaces.iter().all(|surface| {
                !surface.blocked_idents.contains(&"Option".to_string())
                    && !surface.blocked_idents.contains(&"is_none".to_string())
            }),
            "std/prelude helper paths must not become macro blockers: {:?}",
            report.macro_surfaces.surfaces
        );

        let prunable_callables = report
            .usage
            .prunable
            .callables
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
        assert!(
            !blocked_callables.contains("app::Serialize"),
            "derive macro name `Serialize` must not be retained as blocked_by_unknown: {blocked_callables:?}",
        );
        assert!(
            !blocked_callables.contains("app::with"),
            "helper meta key `with` must not be retained as blocked_by_unknown: {blocked_callables:?}",
        );
        assert!(
            !blocked_callables.contains("app::Option"),
            "std wrapper path `Option::is_none` must not retain local Option-named code: {blocked_callables:?}",
        );
        assert!(
            !blocked_callables.contains("app::is_none"),
            "std helper method `Option::is_none` must not retain local is_none code: {blocked_callables:?}",
        );
        assert!(
            !blocked_callables.contains("app::resultType"),
            "serde data strings such as rename/tag values must not retain same-named local code: {blocked_callables:?}",
        );
        assert!(
            prunable_callables.contains("app::unrelated_dead"),
            "unrelated dead code must remain prunable: {prunable_callables:?}",
        );
        let attribute_hazard = report
            .production
            .hazards
            .iter()
            .find(|hazard| hazard.code == "custom_attribute_macros")
            .expect("serde helper path should still report a scoped helper blocker");
        assert!(attribute_hazard.details.iter().all(|detail| {
            !detail.subject.contains("Option::is_none")
                && !detail.subject.contains("resultType")
                && !detail.subject.contains("rename")
        }));

        let generated = fs::read_to_string(output.join("app/src/lib.rs")).unwrap();
        assert!(generated.contains("pub mod wire_helper"));
        assert!(!generated.contains("pub fn unrelated_dead"));
    }

    #[test]
    fn direct_serde_helper_attributes_are_modeled_without_macro_hazards() {
        let root = temp_output("direct-serde-helper-attribute-source");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\nserde = {{ version = \"1\", features = [\"derive\"] }}\n",
                opensourced_path
            ),
        );
        write(
            root.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;

#[opensourced]
pub fn entry() -> WireDto {
    WireDto { duration: Some(5), optional: None }
}

#[derive(serde::Serialize)]
pub struct WireDto {
    #[serde(serialize_with = "serialize_optional_u64")]
    pub duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optional: Option<u32>,
}

pub fn serialize_optional_u64<S>(value: &Option<u64>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match value {
        Some(value) => serializer.serialize_some(value),
        None => serializer.serialize_none(),
    }
}
"#,
        );

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: temp_output("direct-serde-helper-attribute-output"),
        })
        .expect("generation should succeed");

        assert!(
            report
                .production
                .hazards
                .iter()
                .all(|hazard| hazard.code != "custom_attribute_macros"),
            "direct serde helper functions should be statically modeled without macro feedback hazards: {:?}",
            report.production.hazards
        );
        assert!(report
            .macro_surfaces
            .surfaces
            .iter()
            .all(|surface| { !(surface.kind == "helper_attribute" && surface.path == "serde") }));
    }

    #[test]
    fn default_helper_attributes_are_modeled_without_macro_hazards() {
        let root = temp_output("default-helper-attribute-source");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\nuniffi = \"0.28\"\n",
                opensourced_path
            ),
        );
        write(
            root.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;

#[opensourced]
pub fn entry() -> WireDto {
    WireDto { value: None, mode: Mode::Auto }
}

#[derive(Debug, Clone, Default, uniffi::Enum)]
pub enum Mode {
    #[default]
    Auto,
    Manual,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct WireDto {
    #[uniffi(default = None)]
    pub value: Option<String>,
    pub mode: Mode,
}
"#,
        );

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: temp_output("default-helper-attribute-output"),
        })
        .expect("generation should succeed");

        assert!(
            report
                .production
                .hazards
                .iter()
                .all(|hazard| hazard.code != "custom_attribute_macros"),
            "rust #[default] and UniFFI default helpers should be statically modeled: {:?}",
            report.production.hazards
        );
        assert!(report.macro_surfaces.surfaces.iter().all(|surface| {
            !(surface.kind == "attribute_macro" && surface.path == "default")
                && !(surface.kind == "helper_attribute" && surface.path == "uniffi")
        }));
    }

    #[test]
    fn semantic_usage_mapping_reports_unmapped_unused_candidates_without_retaining() {
        let root = temp_output("semantic-usage-mapping-source");
        let output = temp_output("semantic-usage-mapping-output");
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
pub fn entry() -> i32 {
    1
}

pub fn unmapped_dead_code() -> i32 {
    2
}

pub fn mapped_dead_code() -> i32 {
    3
}
"#,
        );

        let workspace = manifest::load_workspace(&root).expect("workspace should load");
        let project = parse::parse_workspace(workspace).expect("workspace should parse");
        let reduced =
            reduce::reduce_with_extra_roots(&project, &[]).expect("initial reduction should work");
        let entry = project_callable_named(&project, "entry");
        let mapped_dead = project_callable_named(&project, "mapped_dead_code");
        let unmapped_dead = project_callable_named(&project, "unmapped_dead_code");
        let mapped_callable_ids = BTreeSet::from([entry.clone(), mapped_dead.clone()]);
        let semantic_usage = SemanticUsageReport {
            indexed_callables: project.functions.len() + project.methods.len(),
            indexed_items: project.items.len(),
            mapped_callables: mapped_callable_ids.len(),
            unmapped_callables: project
                .functions
                .len()
                .saturating_sub(mapped_callable_ids.len()),
            mapped_callable_ids,
            ..SemanticUsageReport::default()
        };
        let analyzer = AnalyzerReport {
            mode: AnalyzerMode::RustAnalyzerHir,
            loaded: true,
            engine: "rust-analyzer HIR".to_string(),
            notes: Vec::new(),
            semantic: Some(SemanticReport::default()),
            semantic_hints: SemanticReductionHints::default(),
            semantic_usage: Some(semantic_usage),
        };
        let production = production_readiness_status(Vec::new());

        let (render_reduced, usage_decisions) =
            usage_guarded_render_reduction(&project, &reduced, &analyzer, &production)
                .expect("usage-guarded render reduction should work");
        assert_eq!(
            usage_decisions.callable_decision(&entry),
            Some(UsageDecision::Used),
            "selected roots should be classified in the decision map as used",
        );
        assert_eq!(
            usage_decisions.callable_decision(&unmapped_dead),
            Some(UsageDecision::Prunable),
            "RA-unmapped candidates without retained references should not force unknown retention",
        );
        assert_eq!(
            usage_decisions.callable_decision(&mapped_dead),
            Some(UsageDecision::Prunable),
            "RA-mapped unreferenced candidates should be classified in the decision map as prunable",
        );
        render::write_reduced_workspace(&project, &render_reduced, &usage_decisions, &output)
            .expect("render should succeed");
        let usage = usage_classification_report(
            &project,
            &reduced,
            &analyzer,
            &usage_decisions,
            &production,
            RenderedSymbolProofReport::default(),
            PublicReexportProofReport::default(),
        );

        assert!(!usage.blocked_by_unknown.callables.contains(&unmapped_dead));
        assert!(
            usage.prunable.callables.contains(&mapped_dead),
            "graph-unused callables with RA mapping evidence should remain prunable",
        );
        assert_eq!(
            usage.semantic_proof.status, "partial_for_retained_packages",
            "unmapped pruned code should keep semantic proof partial: {:?}",
            usage.semantic_proof.unproven.callables,
        );
        assert!(
            usage.semantic_proof.summary.proven_callables >= 1,
            "mapped clean candidates should be counted as semantically proven"
        );
        assert!(
            usage
                .semantic_proof
                .unproven
                .callables
                .contains(&unmapped_dead),
            "unmapped clean candidates should be reported as unproven semantic proof"
        );
        assert!(
            usage.unused.callables.contains(&unmapped_dead),
            "RA-unmapped candidates without retained references are removable after compiler feedback",
        );
        assert!(
            usage.unused.callables.contains(&mapped_dead),
            "RA-mapped clean candidates should be reported in removable unused code",
        );
        let generated = fs::read_to_string(output.join("app/src/lib.rs")).unwrap();
        assert!(!generated.contains("pub fn unmapped_dead_code"));
        assert!(!generated.contains("pub fn mapped_dead_code"));
    }

    #[test]
    fn semantic_usage_proof_discharges_current_target_inactive_prunable_code() {
        let root = temp_output("semantic-usage-inactive-cfg-source");
        let output = temp_output("semantic-usage-inactive-cfg-output");
        let opensourced_path = workspace_root().join("crates/opensourced");
        let inactive_os = inactive_target_os();
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
            &format!(
                r#"use opensourced::opensourced;

#[cfg(target_os = "{inactive_os}")]
pub mod inactive {{
    pub struct InactiveDeadType;

    pub fn inactive_dead_code() -> i32 {{
        2
    }}
}}

#[opensourced]
pub fn entry() -> i32 {{
    1
}}
"#,
            ),
        );

        let workspace = manifest::load_workspace(&root).expect("workspace should load");
        let project = parse::parse_workspace(workspace).expect("workspace should parse");
        let reduced =
            reduce::reduce_with_extra_roots(&project, &[]).expect("initial reduction should work");
        let entry = project_callable_named(&project, "entry");
        let inactive_function = project_callable_named(&project, "inactive_dead_code");
        let inactive_module = project_item_named(&project, "inactive", ItemKind::Mod);
        let inactive_type = project_item_named(&project, "InactiveDeadType", ItemKind::Struct);
        let semantic_usage = SemanticUsageReport {
            indexed_callables: project.functions.len() + project.methods.len(),
            indexed_items: project.items.len(),
            mapped_callables: 1,
            unmapped_callables: project.functions.len().saturating_sub(1),
            mapped_callable_ids: BTreeSet::from([entry.clone()]),
            unmapped_items: project.items.len(),
            ..SemanticUsageReport::default()
        };
        let analyzer = AnalyzerReport {
            mode: AnalyzerMode::RustAnalyzerHir,
            loaded: true,
            engine: "rust-analyzer HIR".to_string(),
            notes: Vec::new(),
            semantic: Some(SemanticReport::default()),
            semantic_hints: SemanticReductionHints::default(),
            semantic_usage: Some(semantic_usage),
        };
        let production = production_readiness_status(Vec::new());

        let (render_reduced, usage_decisions) =
            usage_guarded_render_reduction(&project, &reduced, &analyzer, &production)
                .expect("usage-guarded render reduction should work");
        render::write_reduced_workspace(&project, &render_reduced, &usage_decisions, &output)
            .expect("render should succeed");
        let usage = usage_classification_report(
            &project,
            &reduced,
            &analyzer,
            &usage_decisions,
            &production,
            RenderedSymbolProofReport::default(),
            PublicReexportProofReport::default(),
        );

        assert_eq!(
            usage.semantic_proof.status, "complete_for_retained_packages",
            "{:#?}",
            usage.semantic_proof
        );
        assert_eq!(usage.semantic_proof.summary.cfg_inactive_callables, 1);
        assert_eq!(usage.semantic_proof.summary.cfg_inactive_items, 2);
        assert_eq!(usage.semantic_proof.summary.unmapped_callables, 0);
        assert_eq!(usage.semantic_proof.summary.unmapped_items, 0);
        assert!(usage.unused.callables.contains(&inactive_function));
        assert!(usage.unused.items.contains(&inactive_module));
        assert!(usage.unused.items.contains(&inactive_type));

        let semantic_proof =
            semantic_usage_proof_report(&project, &analyzer, &usage_decisions, None);
        let production = production_readiness_report(
            &analyzer,
            &project,
            &render_reduced,
            &output,
            Some(&semantic_proof),
            Some(&RenderedSymbolProofReport::default()),
            Some(&PublicReexportProofReport::default()),
        );
        assert!(
            !production
                .hazards
                .iter()
                .any(|hazard| hazard.code == "semantic_inventory_available"),
            "current-target inactive prunable code should not leave generic semantic proof debt: {production:#?}"
        );
        let generated = fs::read_to_string(output.join("app/src/lib.rs")).unwrap();
        assert!(!generated.contains("inactive_dead_code"));
        assert!(!generated.contains("InactiveDeadType"));
    }

    #[test]
    fn semantic_usage_proof_discharges_pruned_source_file_contents() {
        let root = temp_output("semantic-usage-source-file-pruned-source");
        let output = temp_output("semantic-usage-source-file-pruned-output");
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

pub mod dead_support;

#[opensourced]
pub fn entry() -> i32 {
    1
}
"#,
        );
        write(
            root.join("app/src/dead_support.rs"),
            r#"pub const DEAD_TIMEOUT: u64 = 30;

pub fn dead_helper() -> u64 {
    DEAD_TIMEOUT
}
"#,
        );

        let workspace = manifest::load_workspace(&root).expect("workspace should load");
        let project = parse::parse_workspace(workspace).expect("workspace should parse");
        let reduced =
            reduce::reduce_with_extra_roots(&project, &[]).expect("initial reduction should work");
        let entry = project_callable_named(&project, "entry");
        let dead_helper = project_callable_named(&project, "dead_helper");
        let dead_module = project_item_named(&project, "dead_support", ItemKind::Mod);
        let dead_timeout = project_item_named(&project, "DEAD_TIMEOUT", ItemKind::Const);
        let semantic_usage = SemanticUsageReport {
            indexed_callables: project.functions.len() + project.methods.len(),
            indexed_items: project.items.len(),
            mapped_callables: 1,
            unmapped_callables: project.functions.len().saturating_sub(1),
            mapped_callable_ids: BTreeSet::from([entry]),
            unmapped_items: project.items.len(),
            ..SemanticUsageReport::default()
        };
        let analyzer = AnalyzerReport {
            mode: AnalyzerMode::RustAnalyzerHir,
            loaded: true,
            engine: "rust-analyzer HIR".to_string(),
            notes: Vec::new(),
            semantic: Some(SemanticReport::default()),
            semantic_hints: SemanticReductionHints::default(),
            semantic_usage: Some(semantic_usage),
        };
        let production = production_readiness_status(Vec::new());

        let (render_reduced, usage_decisions) =
            usage_guarded_render_reduction(&project, &reduced, &analyzer, &production)
                .expect("usage-guarded render reduction should work");
        render::write_reduced_workspace(&project, &render_reduced, &usage_decisions, &output)
            .expect("render should succeed");
        let usage = usage_classification_report(
            &project,
            &reduced,
            &analyzer,
            &usage_decisions,
            &production,
            RenderedSymbolProofReport::default(),
            PublicReexportProofReport::default(),
        );

        assert_eq!(
            usage.semantic_proof.status, "complete_for_retained_packages",
            "{:#?}",
            usage.semantic_proof
        );
        assert_eq!(usage.semantic_proof.summary.source_file_pruned_callables, 1);
        assert_eq!(usage.semantic_proof.summary.source_file_pruned_items, 1);
        assert_eq!(usage.semantic_proof.summary.structural_pruned_items, 1);
        assert_eq!(usage.semantic_proof.summary.unmapped_callables, 0);
        assert_eq!(usage.semantic_proof.summary.unmapped_items, 0);
        assert!(usage.unused.callables.contains(&dead_helper));
        assert!(usage.unused.items.contains(&dead_module));
        assert!(usage.unused.items.contains(&dead_timeout));
        assert!(!output.join("app/src/dead_support.rs").exists());
        let generated = fs::read_to_string(output.join("app/src/lib.rs")).unwrap();
        assert!(!generated.contains("dead_support"));
    }

    #[test]
    fn semantic_usage_proof_discharges_pruned_inline_module_items() {
        let root = temp_output("semantic-usage-inline-module-pruned-source");
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

pub mod live {
    pub struct LiveType;

    #[opensourced]
    pub fn entry() -> LiveType {
        LiveType
    }
}

pub mod dead {
    pub struct DeadInlineType;
}
"#,
        );

        let workspace = manifest::load_workspace(&root).expect("workspace should load");
        let project = parse::parse_workspace(workspace).expect("workspace should parse");
        let reduced =
            reduce::reduce_with_extra_roots(&project, &[]).expect("initial reduction should work");
        let entry = project_callable_named(&project, "entry");
        let live_type = project_item_named(&project, "LiveType", ItemKind::Struct);
        let dead_module = project_item_named(&project, "dead", ItemKind::Mod);
        let dead_type = project_item_named(&project, "DeadInlineType", ItemKind::Struct);
        let mapped_item_ids = BTreeSet::from([live_type, dead_type.clone()]);
        let semantic_usage = SemanticUsageReport {
            indexed_callables: project.functions.len() + project.methods.len(),
            indexed_items: project.items.len(),
            mapped_callables: 1,
            mapped_items: mapped_item_ids.len(),
            unmapped_callables: project.functions.len().saturating_sub(1),
            unmapped_items: project.items.len().saturating_sub(mapped_item_ids.len()),
            mapped_callable_ids: BTreeSet::from([entry]),
            mapped_item_ids,
            skipped_item_reference_ids: BTreeSet::from([dead_type.clone()]),
            reference_queries_skipped: 1,
            ..SemanticUsageReport::default()
        };
        let analyzer = AnalyzerReport {
            mode: AnalyzerMode::RustAnalyzerFeedback,
            loaded: true,
            engine: "rust-analyzer HIR".to_string(),
            notes: Vec::new(),
            semantic: Some(SemanticReport::default()),
            semantic_hints: SemanticReductionHints::default(),
            semantic_usage: Some(semantic_usage),
        };
        let production = production_readiness_status(Vec::new());

        let (_render_reduced, usage_decisions) =
            usage_guarded_render_reduction(&project, &reduced, &analyzer, &production)
                .expect("usage-guarded render reduction should work");
        let usage = usage_classification_report(
            &project,
            &reduced,
            &analyzer,
            &usage_decisions,
            &production,
            RenderedSymbolProofReport::default(),
            PublicReexportProofReport::default(),
        );

        assert_eq!(
            usage.semantic_proof.status, "complete_for_retained_packages",
            "{:#?}",
            usage.semantic_proof
        );
        assert_eq!(usage.semantic_proof.summary.unproven_items, 0);
        assert_eq!(
            usage.semantic_proof.summary.skipped_reference_query_items,
            0
        );
        assert_eq!(usage.semantic_proof.summary.structural_pruned_items, 2);
        assert!(usage.unused.items.contains(&dead_module));
        assert!(usage.unused.items.contains(&dead_type));
        let production = production_readiness_report(
            &analyzer,
            &project,
            &reduced,
            Path::new("/tmp"),
            Some(&usage.semantic_proof),
            Some(&RenderedSymbolProofReport::default()),
            Some(&PublicReexportProofReport::default()),
        );
        assert!(
            !production
                .hazards
                .iter()
                .any(|hazard| hazard.code == "semantic_usage_reference_skipped"),
            "module-pruned skipped references should not leave semantic proof debt: {production:#?}"
        );
    }

    #[test]
    fn semantic_usage_proof_discharges_rendered_absent_items() {
        let root = temp_output("semantic-usage-rendered-absent-source");
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

pub struct LiveType;

pub struct DeadRootType;

#[opensourced]
pub fn entry() -> LiveType {
    LiveType
}
"#,
        );

        let workspace = manifest::load_workspace(&root).expect("workspace should load");
        let project = parse::parse_workspace(workspace).expect("workspace should parse");
        let reduced =
            reduce::reduce_with_extra_roots(&project, &[]).expect("initial reduction should work");
        let entry = project_callable_named(&project, "entry");
        let live_type = project_item_named(&project, "LiveType", ItemKind::Struct);
        let dead_type = project_item_named(&project, "DeadRootType", ItemKind::Struct);
        let mapped_item_ids = BTreeSet::from([live_type.clone(), dead_type.clone()]);
        let semantic_usage = SemanticUsageReport {
            indexed_callables: project.functions.len() + project.methods.len(),
            indexed_items: project.items.len(),
            mapped_callables: 1,
            mapped_items: mapped_item_ids.len(),
            mapped_callable_ids: BTreeSet::from([entry]),
            mapped_item_ids,
            skipped_item_reference_ids: BTreeSet::from([dead_type.clone()]),
            reference_queries_skipped: 1,
            ..SemanticUsageReport::default()
        };
        let analyzer = AnalyzerReport {
            mode: AnalyzerMode::RustAnalyzerFeedback,
            loaded: true,
            engine: "rust-analyzer HIR".to_string(),
            notes: Vec::new(),
            semantic: Some(SemanticReport::default()),
            semantic_hints: SemanticReductionHints::default(),
            semantic_usage: Some(semantic_usage),
        };
        let production = production_readiness_status(Vec::new());

        let (_render_reduced, usage_decisions) =
            usage_guarded_render_reduction(&project, &reduced, &analyzer, &production)
                .expect("usage-guarded render reduction should work");
        let rendered_symbols = RenderedSymbolProofReport {
            status: "proven".to_string(),
            summary: RenderedSymbolProofSummary::default(),
            entries: vec![RenderedSymbolProofEntry {
                kind: "item".to_string(),
                id: live_type.to_string(),
                classification: "retained".to_string(),
            }],
        };
        let usage = usage_classification_report(
            &project,
            &reduced,
            &analyzer,
            &usage_decisions,
            &production,
            rendered_symbols,
            PublicReexportProofReport::default(),
        );

        assert_eq!(
            usage.semantic_proof.status, "complete_for_retained_packages",
            "{:#?}",
            usage.semantic_proof
        );
        assert_eq!(usage.semantic_proof.summary.unproven_items, 0);
        assert_eq!(
            usage.semantic_proof.summary.skipped_reference_query_items,
            0
        );
        assert_eq!(usage.semantic_proof.summary.rendered_absent_items, 1);
        assert!(usage.unused.items.contains(&dead_type));
        let production = production_readiness_report(
            &analyzer,
            &project,
            &reduced,
            Path::new("/tmp"),
            Some(&usage.semantic_proof),
            Some(&usage.rendered_symbols),
            Some(&PublicReexportProofReport::default()),
        );
        assert!(
            !production
                .hazards
                .iter()
                .any(|hazard| hazard.code == "semantic_usage_reference_skipped"),
            "rendered-absent skipped references should not leave semantic proof debt: {production:#?}"
        );
    }

    #[test]
    fn semantic_usage_proof_tracks_reference_skips_per_target() {
        let root = temp_output("semantic-usage-per-target-reference-skip-source");
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

pub mod dead_support;

#[opensourced]
pub fn entry() -> i32 {
    1
}

pub fn clean_dead_code() -> i32 {
    2
}
"#,
        );
        write(
            root.join("app/src/dead_support.rs"),
            r#"pub fn skipped_dead_helper() -> i32 {
    3
}
"#,
        );

        let workspace = manifest::load_workspace(&root).expect("workspace should load");
        let project = parse::parse_workspace(workspace).expect("workspace should parse");
        let reduced =
            reduce::reduce_with_extra_roots(&project, &[]).expect("initial reduction should work");
        let entry = project_callable_named(&project, "entry");
        let clean_dead = project_callable_named(&project, "clean_dead_code");
        let skipped_dead = project_callable_named(&project, "skipped_dead_helper");
        let mapped_callable_ids = BTreeSet::from([entry.clone(), clean_dead.clone()]);
        let semantic_usage = SemanticUsageReport {
            indexed_callables: project.functions.len() + project.methods.len(),
            indexed_items: project.items.len(),
            mapped_callables: mapped_callable_ids.len(),
            unmapped_callables: project
                .functions
                .len()
                .saturating_sub(mapped_callable_ids.len()),
            mapped_callable_ids: mapped_callable_ids.clone(),
            queried_callable_reference_ids: mapped_callable_ids,
            reference_queries: 2,
            reference_queries_skipped: 1,
            skipped_callable_reference_ids: BTreeSet::from([skipped_dead]),
            unmapped_items: project.items.len(),
            ..SemanticUsageReport::default()
        };
        let analyzer = AnalyzerReport {
            mode: AnalyzerMode::RustAnalyzerFeedback,
            loaded: true,
            engine: "rust-analyzer HIR".to_string(),
            notes: Vec::new(),
            semantic: Some(SemanticReport::default()),
            semantic_hints: SemanticReductionHints::default(),
            semantic_usage: Some(semantic_usage),
        };
        let production = production_readiness_status(Vec::new());

        let (_render_reduced, usage_decisions) =
            usage_guarded_render_reduction(&project, &reduced, &analyzer, &production)
                .expect("usage-guarded render reduction should work");
        let usage = usage_classification_report(
            &project,
            &reduced,
            &analyzer,
            &usage_decisions,
            &production,
            RenderedSymbolProofReport::default(),
            PublicReexportProofReport::default(),
        );

        assert_eq!(
            usage.semantic_proof.status, "complete_for_retained_packages",
            "{:#?}",
            usage.semantic_proof
        );
        assert_eq!(
            usage
                .semantic_proof
                .summary
                .skipped_reference_query_callables,
            0
        );
        assert!(usage.prunable.callables.contains(&clean_dead));
        let production = production_readiness_report(
            &analyzer,
            &project,
            &reduced,
            Path::new("/tmp"),
            Some(&usage.semantic_proof),
            Some(&RenderedSymbolProofReport::default()),
            Some(&PublicReexportProofReport::default()),
        );
        assert!(
            !production
                .hazards
                .iter()
                .any(|hazard| hazard.code == "semantic_usage_reference_skipped"),
            "per-target reference proof completion should discharge unrelated skipped queries: {production:#?}"
        );
    }

    #[test]
    fn semantic_usage_reference_skip_hazard_reports_unproven_debt_only() {
        let root = temp_output("semantic-usage-reference-skip-debt-source");
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
pub fn entry() -> i32 {
    1
}

pub fn same_file_dead_code() -> i32 {
    2
}

pub mod dead_support;
"#,
        );
        write(
            root.join("app/src/dead_support.rs"),
            r#"pub fn pruned_file_dead_code() -> i32 {
    3
}
"#,
        );

        let workspace = manifest::load_workspace(&root).expect("workspace should load");
        let project = parse::parse_workspace(workspace).expect("workspace should parse");
        let reduced =
            reduce::reduce_with_extra_roots(&project, &[]).expect("initial reduction should work");
        let entry = project_callable_named(&project, "entry");
        let same_file_dead = project_callable_named(&project, "same_file_dead_code");
        let pruned_file_dead = project_callable_named(&project, "pruned_file_dead_code");
        let mapped_callable_ids = BTreeSet::from([
            entry.clone(),
            same_file_dead.clone(),
            pruned_file_dead.clone(),
        ]);
        let semantic_usage = SemanticUsageReport {
            indexed_callables: project.functions.len() + project.methods.len(),
            indexed_items: project.items.len(),
            mapped_callables: mapped_callable_ids.len(),
            unmapped_callables: project
                .functions
                .len()
                .saturating_sub(mapped_callable_ids.len()),
            mapped_callable_ids: mapped_callable_ids.clone(),
            skipped_callable_reference_ids: BTreeSet::from([
                same_file_dead.clone(),
                pruned_file_dead.clone(),
            ]),
            reference_queries_skipped: 10,
            unmapped_items: project.items.len(),
            ..SemanticUsageReport::default()
        };
        let analyzer = AnalyzerReport {
            mode: AnalyzerMode::RustAnalyzerFeedback,
            loaded: true,
            engine: "rust-analyzer HIR".to_string(),
            notes: Vec::new(),
            semantic: Some(SemanticReport::default()),
            semantic_hints: SemanticReductionHints::default(),
            semantic_usage: Some(semantic_usage),
        };
        let production = production_readiness_status(Vec::new());

        let (_render_reduced, usage_decisions) =
            usage_guarded_render_reduction(&project, &reduced, &analyzer, &production)
                .expect("usage-guarded render reduction should work");
        let usage = usage_classification_report(
            &project,
            &reduced,
            &analyzer,
            &usage_decisions,
            &production,
            RenderedSymbolProofReport::default(),
            PublicReexportProofReport::default(),
        );

        assert_eq!(
            usage
                .semantic_proof
                .summary
                .skipped_reference_query_callables,
            1,
            "{:#?}",
            usage.semantic_proof
        );
        let production = production_readiness_report(
            &analyzer,
            &project,
            &reduced,
            Path::new("/tmp"),
            Some(&usage.semantic_proof),
            Some(&RenderedSymbolProofReport::default()),
            Some(&PublicReexportProofReport::default()),
        );
        let hazard = production
            .hazards
            .iter()
            .find(|hazard| hazard.code == "semantic_usage_reference_skipped")
            .expect("same-file skipped callable should leave focused reference proof debt");
        assert!(
            hazard.message.contains("skipped 1 retained-package"),
            "{hazard:#?}"
        );
        assert!(
            hazard
                .message
                .contains("(10 total indexed query/queries skipped)"),
            "{hazard:#?}"
        );
        assert_eq!(hazard.details.len(), 1);
        assert_eq!(hazard.details[0].subject, same_file_dead.to_string());
    }

    #[test]
    fn semantic_usage_prunes_only_proven_unused_public_use_aliases() {
        let root = temp_output("semantic-usage-public-use-source");
        let output = temp_output("semantic-usage-public-use-output");
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

pub use crate::{
    clean_dead_code as CleanDeadCode,
    unknown_helper as UnknownHelper,
};

#[opensourced]
pub fn entry() -> i32 {
    1
}

pub fn unknown_helper() -> i32 {
    private_leaf()
}

fn private_leaf() -> i32 {
    2
}

pub fn clean_dead_code() -> i32 {
    3
}
"#,
        );

        let workspace = manifest::load_workspace(&root).expect("workspace should load");
        let project = parse::parse_workspace(workspace).expect("workspace should parse");
        let reduced =
            reduce::reduce_with_extra_roots(&project, &[]).expect("initial reduction should work");
        let entry = project_callable_named(&project, "entry");
        let unknown_helper = project_callable_named(&project, "unknown_helper");
        let private_leaf = project_callable_named(&project, "private_leaf");
        let clean_dead = project_callable_named(&project, "clean_dead_code");
        let mapped_callable_ids =
            BTreeSet::from([entry.clone(), private_leaf.clone(), clean_dead.clone()]);
        let semantic_usage = SemanticUsageReport {
            indexed_callables: project.functions.len() + project.methods.len(),
            indexed_items: project.items.len(),
            mapped_callables: mapped_callable_ids.len(),
            unmapped_callables: project
                .functions
                .len()
                .saturating_sub(mapped_callable_ids.len()),
            mapped_callable_ids,
            ..SemanticUsageReport::default()
        };
        let analyzer = AnalyzerReport {
            mode: AnalyzerMode::RustAnalyzerHir,
            loaded: true,
            engine: "rust-analyzer HIR".to_string(),
            notes: Vec::new(),
            semantic: Some(SemanticReport::default()),
            semantic_hints: SemanticReductionHints::default(),
            semantic_usage: Some(semantic_usage),
        };
        let production = production_readiness_status(Vec::new());

        let (render_reduced, usage_decisions) =
            usage_guarded_render_reduction(&project, &reduced, &analyzer, &production)
                .expect("usage-guarded render reduction should work");
        assert_eq!(
            usage_decisions.callable_decision(&unknown_helper),
            Some(UsageDecision::Prunable),
            "RA-unmapped public reexport target without retained references should be removable",
        );
        assert_eq!(
            usage_decisions.callable_decision(&clean_dead),
            Some(UsageDecision::Prunable),
            "RA-mapped unreferenced public reexport target should be removable",
        );
        render::write_reduced_workspace(&project, &render_reduced, &usage_decisions, &output)
            .expect("render should succeed");
        let usage = usage_classification_report(
            &project,
            &reduced,
            &analyzer,
            &usage_decisions,
            &production,
            RenderedSymbolProofReport::default(),
            PublicReexportProofReport::default(),
        );
        assert!(!usage.blocked_by_unknown.callables.contains(&unknown_helper));
        assert!(usage.unused.callables.contains(&clean_dead));
        assert!(usage.unused.callables.contains(&unknown_helper));

        let generated = fs::read_to_string(output.join("app/src/lib.rs")).unwrap();
        assert!(
            !generated.contains("UnknownHelper"),
            "unreferenced reexport aliases should be removed even when RA mapping is incomplete:\n{generated}",
        );
        assert!(
            !generated.contains("CleanDeadCode"),
            "proven-unused public use aliases should be removed:\n{generated}",
        );
        assert!(
            !generated.contains("pub fn clean_dead_code"),
            "proven-unused target function should be stripped:\n{generated}",
        );
        assert!(
            !generated.contains("pub fn unknown_helper"),
            "unreferenced target function should be stripped:\n{generated}",
        );
    }

    #[test]
    fn blocked_unknown_items_keep_their_import_mentions() {
        let root = temp_output("semantic-usage-blocked-item-import-source");
        let output = temp_output("semantic-usage-blocked-item-import-output");
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

pub mod dep;
pub mod surface;

#[opensourced]
pub fn entry() -> i32 {
    1
}
"#,
        );
        write(
            root.join("app/src/dep.rs"),
            r#"pub struct BlockedType {
    pub value: i32,
}

pub struct CleanType {
    pub value: i32,
}
"#,
        );
        write(
            root.join("app/src/surface.rs"),
            r#"use crate::dep::{BlockedType, CleanType};

pub struct UnknownSurface {
    pub field: BlockedType,
}

pub struct CleanSurface {
    pub field: CleanType,
}
"#,
        );

        let workspace = manifest::load_workspace(&root).expect("workspace should load");
        let project = parse::parse_workspace(workspace).expect("workspace should parse");
        let reduced =
            reduce::reduce_with_extra_roots(&project, &[]).expect("initial reduction should work");
        let entry = project_callable_named(&project, "entry");
        let blocked_type = project_item_named(&project, "BlockedType", ItemKind::Struct);
        let clean_type = project_item_named(&project, "CleanType", ItemKind::Struct);
        let unknown_surface = project_item_named(&project, "UnknownSurface", ItemKind::Struct);
        let clean_surface = project_item_named(&project, "CleanSurface", ItemKind::Struct);
        let mapped_item_ids = BTreeSet::from([
            blocked_type.clone(),
            clean_type.clone(),
            unknown_surface.clone(),
            clean_surface.clone(),
        ]);
        let semantic_usage = SemanticUsageReport {
            indexed_callables: project.functions.len() + project.methods.len(),
            indexed_items: project.items.len(),
            mapped_callables: 1,
            mapped_items: mapped_item_ids.len(),
            unmapped_items: project.items.len().saturating_sub(mapped_item_ids.len()),
            mapped_callable_ids: BTreeSet::from([entry]),
            mapped_item_ids,
            referenced_items: 1,
            referenced_item_ids: BTreeSet::from([unknown_surface.clone()]),
            item_reference_owners: BTreeMap::from([(
                unknown_surface,
                BTreeSet::from([SemanticOwnerId::Callable(project_callable_named(
                    &project, "entry",
                ))]),
            )]),
            ..SemanticUsageReport::default()
        };
        let analyzer = AnalyzerReport {
            mode: AnalyzerMode::RustAnalyzerHir,
            loaded: true,
            engine: "rust-analyzer HIR".to_string(),
            notes: Vec::new(),
            semantic: Some(SemanticReport::default()),
            semantic_hints: SemanticReductionHints::default(),
            semantic_usage: Some(semantic_usage),
        };
        let production = production_readiness_status(Vec::new());

        let (render_reduced, usage_decisions) =
            usage_guarded_render_reduction(&project, &reduced, &analyzer, &production)
                .expect("usage-guarded render reduction should work");
        render::write_reduced_workspace(&project, &render_reduced, &usage_decisions, &output)
            .expect("render should succeed");
        let generated = fs::read_to_string(output.join("app/src/surface.rs")).unwrap();

        assert!(
            generated.contains("BlockedType"),
            "blocked unknown item field type and import should remain:\n{generated}",
        );
        assert!(
            !generated.contains("CleanType"),
            "proven-unused sibling import should be pruned:\n{generated}",
        );
        assert!(
            !generated.contains("CleanSurface"),
            "proven-unused sibling item should be pruned:\n{generated}",
        );
    }

    #[test]
    fn semantic_usage_references_from_retained_code_block_pruning() {
        let root = temp_output("semantic-usage-reference-source");
        let output = temp_output("semantic-usage-reference-output");
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
pub fn entry() -> i32 {
    1
}

pub fn semantically_referenced_dead_code() -> i32 {
    private_leaf()
}

fn private_leaf() -> i32 {
    2
}

pub fn clean_dead_code() -> i32 {
    3
}
"#,
        );

        let workspace = manifest::load_workspace(&root).expect("workspace should load");
        let project = parse::parse_workspace(workspace).expect("workspace should parse");
        let reduced =
            reduce::reduce_with_extra_roots(&project, &[]).expect("initial reduction should work");
        let entry = project_callable_named(&project, "entry");
        let referenced_dead = project_callable_named(&project, "semantically_referenced_dead_code");
        let private_leaf = project_callable_named(&project, "private_leaf");
        let clean_dead = project_callable_named(&project, "clean_dead_code");
        let mapped_callable_ids = BTreeSet::from([
            entry.clone(),
            referenced_dead.clone(),
            private_leaf.clone(),
            clean_dead.clone(),
        ]);
        let semantic_usage = SemanticUsageReport {
            indexed_callables: project.functions.len() + project.methods.len(),
            indexed_items: project.items.len(),
            mapped_callables: mapped_callable_ids.len(),
            unmapped_callables: project
                .functions
                .len()
                .saturating_sub(mapped_callable_ids.len()),
            referenced_callables: 1,
            referenced_callable_ids: BTreeSet::from([referenced_dead.clone()]),
            callable_reference_owners: BTreeMap::from([(
                referenced_dead.clone(),
                BTreeSet::from([SemanticOwnerId::Callable(entry.clone())]),
            )]),
            mapped_callable_ids,
            ..SemanticUsageReport::default()
        };
        let analyzer = AnalyzerReport {
            mode: AnalyzerMode::RustAnalyzerHir,
            loaded: true,
            engine: "rust-analyzer HIR".to_string(),
            notes: Vec::new(),
            semantic: Some(SemanticReport::default()),
            semantic_hints: SemanticReductionHints::default(),
            semantic_usage: Some(semantic_usage),
        };
        let production = production_readiness_status(Vec::new());

        let (render_reduced, usage_decisions) =
            usage_guarded_render_reduction(&project, &reduced, &analyzer, &production)
                .expect("usage-guarded render reduction should work");
        render::write_reduced_workspace(&project, &render_reduced, &usage_decisions, &output)
            .expect("render should succeed");
        let usage = usage_classification_report(
            &project,
            &reduced,
            &analyzer,
            &usage_decisions,
            &production,
            RenderedSymbolProofReport::default(),
            PublicReexportProofReport::default(),
        );

        assert!(
            usage
                .blocked_by_unknown
                .callables
                .contains(&referenced_dead),
            "graph-unused callables referenced by retained RA evidence must stay blocked_by_unknown",
        );
        assert!(
            usage.blocked_by_unknown.callables.contains(&private_leaf),
            "dependencies of a semantically blocked callable should also be retained",
        );
        assert!(
            usage.prunable.callables.contains(&clean_dead),
            "mapped graph-unused callables with no retained references should remain prunable",
        );
        assert!(
            !usage.unused.callables.contains(&referenced_dead),
            "RA-referenced unknown candidates must not be reported as removable unused code",
        );
        assert!(
            usage.unused.callables.contains(&clean_dead),
            "RA-clean candidates should be reported as removable unused code",
        );
        let generated = fs::read_to_string(output.join("app/src/lib.rs")).unwrap();
        assert!(generated.contains("pub fn semantically_referenced_dead_code"));
        assert!(generated.contains("fn private_leaf"));
        assert!(!generated.contains("pub fn clean_dead_code"));
    }

    #[test]
    fn semantic_reference_edges_classify_referenced_candidates_as_used() {
        let root = temp_output("semantic-reference-edge-source");
        let output = temp_output("semantic-reference-edge-output");
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
pub fn entry() -> i32 {
    1
}

pub fn semantically_referenced_code() -> i32 {
    private_leaf()
}

fn private_leaf() -> i32 {
    2
}

pub fn clean_dead_code() -> i32 {
    3
}
"#,
        );

        let workspace = manifest::load_workspace(&root).expect("workspace should load");
        let project = parse::parse_workspace(workspace).expect("workspace should parse");
        let entry = project_callable_named(&project, "entry");
        let referenced = project_callable_named(&project, "semantically_referenced_code");
        let private_leaf = project_callable_named(&project, "private_leaf");
        let clean_dead = project_callable_named(&project, "clean_dead_code");
        let mapped_callable_ids = BTreeSet::from([
            entry.clone(),
            referenced.clone(),
            private_leaf.clone(),
            clean_dead.clone(),
        ]);
        let mut semantic_hints = SemanticReductionHints::default();
        semantic_hints
            .add_callable_edge(SemanticOwnerId::Callable(entry.clone()), referenced.clone());
        let reduced = reduce::reduce_with_extra_roots_and_semantics(&project, &[], &semantic_hints)
            .expect("semantic edge reduction should work");
        let semantic_usage = SemanticUsageReport {
            indexed_callables: project.functions.len() + project.methods.len(),
            indexed_items: project.items.len(),
            mapped_callables: mapped_callable_ids.len(),
            unmapped_callables: project
                .functions
                .len()
                .saturating_sub(mapped_callable_ids.len()),
            callable_reference_edges: 1,
            referenced_callables: 1,
            referenced_callable_ids: BTreeSet::from([referenced.clone()]),
            callable_reference_owners: BTreeMap::from([(
                referenced.clone(),
                BTreeSet::from([SemanticOwnerId::Callable(entry.clone())]),
            )]),
            mapped_callable_ids,
            ..SemanticUsageReport::default()
        };
        let analyzer = AnalyzerReport {
            mode: AnalyzerMode::RustAnalyzerHir,
            loaded: true,
            engine: "rust-analyzer HIR".to_string(),
            notes: Vec::new(),
            semantic: Some(SemanticReport::default()),
            semantic_hints,
            semantic_usage: Some(semantic_usage),
        };
        let production = production_readiness_status(Vec::new());

        let (render_reduced, usage_decisions) =
            usage_guarded_render_reduction(&project, &reduced, &analyzer, &production)
                .expect("usage-guarded render reduction should work");
        render::write_reduced_workspace(&project, &render_reduced, &usage_decisions, &output)
            .expect("render should succeed");
        let usage = usage_classification_report(
            &project,
            &reduced,
            &analyzer,
            &usage_decisions,
            &production,
            RenderedSymbolProofReport::default(),
            PublicReexportProofReport::default(),
        );

        assert!(
            usage.used.callables.contains(&referenced),
            "RA-promoted reference edges should classify reachable targets as used",
        );
        assert!(
            usage.used.callables.contains(&private_leaf),
            "syntactic dependencies of an RA-promoted target should also be used",
        );
        assert!(
            !usage.blocked_by_unknown.callables.contains(&referenced),
            "RA-promoted reference edges should not remain unknown-retained",
        );
        assert!(
            usage.prunable.callables.contains(&clean_dead),
            "unreferenced mapped siblings should remain prunable",
        );
        let generated = fs::read_to_string(output.join("app/src/lib.rs")).unwrap();
        assert!(generated.contains("pub fn semantically_referenced_code"));
        assert!(generated.contains("fn private_leaf"));
        assert!(!generated.contains("pub fn clean_dead_code"));
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
        let mut report = generate(GenerateOptions {
            workspace_root: workspace_root(),
            output_root: output,
        })
        .expect("reduction should succeed");
        let semantic_callable = report
            .reachable
            .first()
            .cloned()
            .expect("fixture should have reachable callables");
        let semantic_owner_callable = report
            .reachable
            .get(1)
            .cloned()
            .unwrap_or_else(|| semantic_callable.clone());
        let semantic_item = report
            .reachable_items
            .first()
            .cloned()
            .expect("fixture should have reachable items");
        let semantic_owner_item = report
            .reachable_items
            .get(1)
            .cloned()
            .unwrap_or_else(|| semantic_item.clone());
        report.analyzer.semantic_usage = Some(SemanticUsageReport {
            indexed_callables: 3,
            indexed_items: 2,
            mapped_callables: 1,
            mapped_items: 1,
            unmapped_callables: 2,
            unmapped_items: 1,
            reference_queries: 4,
            reference_queries_skipped: 0,
            reference_query_failures: 2,
            callable_reference_edges: 1,
            item_reference_edges: 1,
            referenced_callables: 1,
            referenced_items: 1,
            mapped_callable_ids: BTreeSet::from([semantic_callable.clone()]),
            mapped_item_ids: BTreeSet::from([semantic_item.clone()]),
            queried_callable_reference_ids: BTreeSet::from([semantic_callable.clone()]),
            queried_item_reference_ids: BTreeSet::from([semantic_item.clone()]),
            skipped_callable_reference_ids: BTreeSet::new(),
            skipped_item_reference_ids: BTreeSet::new(),
            failed_callable_reference_ids: BTreeSet::from([semantic_callable.clone()]),
            failed_item_reference_ids: BTreeSet::from([semantic_item.clone()]),
            referenced_callable_ids: BTreeSet::from([semantic_callable.clone()]),
            referenced_item_ids: BTreeSet::from([semantic_item.clone()]),
            callable_reference_owners: BTreeMap::from([(
                semantic_callable.clone(),
                BTreeSet::from([SemanticOwnerId::Callable(semantic_owner_callable.clone())]),
            )]),
            item_reference_owners: BTreeMap::from([(
                semantic_item.clone(),
                BTreeSet::from([SemanticOwnerId::Item(semantic_owner_item.clone())]),
            )]),
            callable_unowned_reference_files: BTreeMap::from([(
                semantic_callable.clone(),
                BTreeSet::from([PathBuf::from("/tmp/unowned-callable.rs")]),
            )]),
            item_unowned_reference_files: BTreeMap::from([(
                semantic_item.clone(),
                BTreeSet::from([PathBuf::from("/tmp/unowned-item.rs")]),
            )]),
        });
        let report_path = temp_output("generation-report-json").join("slice-report.json");

        write_generate_report(&report, &report_path).expect("generation report should be written");

        let value: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(report_path).unwrap()).unwrap();
        assert_eq!(value["analyzer"]["mode"], "syn");
        assert_eq!(
            value["analyzer"]["semantic_usage"]["reference_query_failures"],
            2
        );
        assert!(value["analyzer"]["semantic_usage"]["mapped_callable_ids"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry == &semantic_callable.to_string()));
        assert!(
            value["analyzer"]["semantic_usage"]["failed_item_reference_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry == &semantic_item.to_string())
        );
        assert_eq!(
            value["analyzer"]["semantic_usage"]["callable_reference_owners"][0]["target"],
            semantic_callable.to_string(),
        );
        assert!(
            value["analyzer"]["semantic_usage"]["callable_reference_owners"][0]["owners"]
                .as_array()
                .unwrap()
                .iter()
                .any(|owner| owner == &semantic_owner_callable.to_string())
        );
        assert!(
            value["analyzer"]["semantic_usage"]["item_unowned_reference_files"][0]["files"]
                .as_array()
                .unwrap()
                .iter()
                .any(|file| file == "/tmp/unowned-item.rs")
        );
        assert_eq!(
            value["usage"]["semantic_proof"]["status"],
            report.usage.semantic_proof.status
        );
        assert_eq!(
            value["usage"]["rendered_symbols"]["status"],
            report.usage.rendered_symbols.status
        );
        assert_eq!(
            value["usage"]["rendered_symbols"]["summary"]["prunable_callables"],
            report.usage.rendered_symbols.summary.prunable_callables
        );
        assert_eq!(
            value["usage"]["rendered_symbols"]["summary"]["macro_blocked_callables"],
            report
                .usage
                .rendered_symbols
                .summary
                .macro_blocked_callables
        );
        assert_eq!(
            value["usage"]["rendered_symbols"]["summary"]["unproven_trait_default_methods"],
            report
                .usage
                .rendered_symbols
                .summary
                .unproven_trait_default_methods
        );
        assert!(value["usage"]["rendered_symbols"]["entries"].is_array());
        assert_eq!(
            value["rendered_callables"].as_array().unwrap().len(),
            report.usage.rendered_symbols.summary.rendered_callables
        );
        assert_eq!(
            value["rendered_items"].as_array().unwrap().len(),
            report.usage.rendered_symbols.summary.rendered_items
        );
        assert_eq!(
            value["usage"]["rendered_decision_map"]["callables"]
                .as_object()
                .unwrap()
                .len(),
            report.usage.rendered_symbols.summary.rendered_callables
        );
        assert_eq!(
            value["usage"]["rendered_decision_map"]["items"]
                .as_object()
                .unwrap()
                .len(),
            report.usage.rendered_symbols.summary.rendered_items
        );
        assert_eq!(
            value["usage"]["public_reexports"]["status"],
            report.usage.public_reexports.status
        );
        assert_eq!(
            value["usage"]["public_reexports"]["summary"]["prunable_targets"],
            report.usage.public_reexports.summary.prunable_targets
        );
        assert!(value["usage"]["public_reexports"]["entries"].is_array());
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
        assert_eq!(value["usage"]["status"], "classified_with_unknowns");
        assert_eq!(
            value["usage"]["summary"]["indexed_callables"],
            report.usage.summary.indexed_callables,
        );
        assert!(value["usage"]["used"]["callables"]
            .as_array()
            .unwrap()
            .iter()
            .any(|callable| callable == "a::open_source_entry"));
        assert_eq!(
            value["usage"]["decision_map"]["callables"]["a::open_source_entry"],
            "used"
        );
        assert_eq!(
            value["usage"]["rendered_decision_map"]["callables"]["a::open_source_entry"],
            "used"
        );
        assert!(value["usage"]["unused"]["callables"]
            .as_array()
            .unwrap()
            .iter()
            .any(|callable| callable == "a::internal_entry"));
        assert_eq!(
            value["usage"]["decision_map"]["callables"]["a::internal_entry"],
            "prunable"
        );
        assert!(value["usage"]["unused_candidate"]["callables"]
            .as_array()
            .unwrap()
            .iter()
            .any(|callable| callable == "a::internal_entry"));
        assert!(value["usage"]["blocked_by_unknown"]["callables"]
            .as_array()
            .unwrap()
            .is_empty());
        assert!(value["usage"]["prunable"]["callables"]
            .as_array()
            .unwrap()
            .iter()
            .any(|callable| callable == "a::internal_entry"));
        assert_eq!(
            value["usage"]["summary"]["prunable_callables"].as_u64(),
            Some(report.usage.summary.unused_callables as u64),
            "semantic analyzer availability warnings alone should not block graph-unreachable callables from prunable",
        );
        assert!(value["usage"]["unknown"]
            .as_array()
            .unwrap()
            .iter()
            .any(|surface| surface["code"] == "semantic_analyzer_unavailable"));
        let evidence_callables = value["usage"]["evidence"]["callables"].as_array().unwrap();
        let root_evidence = evidence_callables
            .iter()
            .find(|entry| entry["id"] == "a::open_source_entry")
            .expect("root callable usage evidence should be serialized");
        assert_eq!(root_evidence["classification"], "used");
        assert_eq!(root_evidence["selected_root"], true);
        assert!(root_evidence["reason"]
            .as_str()
            .unwrap()
            .contains("selected opensourced root"));
        let unused_evidence = evidence_callables
            .iter()
            .find(|entry| entry["id"] == "a::internal_entry")
            .expect("unused callable usage evidence should be serialized");
        assert_eq!(unused_evidence["classification"], "prunable");
        assert!(unused_evidence["evidence"]
            .as_array()
            .unwrap()
            .iter()
            .any(|detail| detail == "reachable=false"));
        assert!(unused_evidence["evidence"]
            .as_array()
            .unwrap()
            .iter()
            .any(|detail| detail == "prunable=true"));
    }

    #[test]
    fn rendered_symbol_proof_hazards_block_prunable_and_unclassified_symbols() {
        let report = RenderedSymbolProofReport {
            status: "failed".to_string(),
            summary: RenderedSymbolProofSummary {
                prunable_callables: 1,
                prunable_items: 1,
                prunable_members: 1,
                prunable_assoc_items: 1,
                unclassified_callables: 1,
                unclassified_items: 1,
                unclassified_members: 1,
                unclassified_assoc_items: 1,
                unproven_trait_default_methods: 1,
                source_parse_failures: 1,
                ..RenderedSymbolProofSummary::default()
            },
            entries: vec![
                RenderedSymbolProofEntry {
                    kind: "callable".to_string(),
                    id: "facade::dead_fn".to_string(),
                    classification: "prunable".to_string(),
                },
                RenderedSymbolProofEntry {
                    kind: "item".to_string(),
                    id: "facade::DeadType(Struct)".to_string(),
                    classification: "prunable".to_string(),
                },
                RenderedSymbolProofEntry {
                    kind: "member".to_string(),
                    id: "facade::DeadType(Struct)::dead_field".to_string(),
                    classification: "prunable".to_string(),
                },
                RenderedSymbolProofEntry {
                    kind: "assoc_item".to_string(),
                    id: "facade::DeadTrait(Trait)::DeadAssoc(Type)".to_string(),
                    classification: "prunable".to_string(),
                },
                RenderedSymbolProofEntry {
                    kind: "callable".to_string(),
                    id: "facade::escaped_fn".to_string(),
                    classification: "unclassified".to_string(),
                },
                RenderedSymbolProofEntry {
                    kind: "item".to_string(),
                    id: "facade::EscapedType(Struct)".to_string(),
                    classification: "unclassified".to_string(),
                },
                RenderedSymbolProofEntry {
                    kind: "member".to_string(),
                    id: "facade::EscapedType(Struct)::escaped_field".to_string(),
                    classification: "unclassified".to_string(),
                },
                RenderedSymbolProofEntry {
                    kind: "assoc_item".to_string(),
                    id: "facade::EscapedTrait(Trait)::EscapedAssoc(Const)".to_string(),
                    classification: "unclassified".to_string(),
                },
                RenderedSymbolProofEntry {
                    kind: "trait_default_method".to_string(),
                    id: "facade::EscapedTrait(Trait)::dead_default".to_string(),
                    classification: "unproven".to_string(),
                },
            ],
        };
        let mut hazards = Vec::new();

        add_rendered_symbol_proof_hazards(&report, &mut hazards);

        assert!(hazards.iter().any(|hazard| {
            hazard.code == "rendered_prunable_symbols"
                && hazard.severity == "error"
                && hazard.details.iter().any(|detail| {
                    detail.package.as_deref() == Some("facade")
                        && detail
                            .blocked_idents
                            .iter()
                            .any(|ident| ident == "facade::dead_fn")
                })
        }));
        assert!(hazards.iter().any(|hazard| {
            hazard.code == "rendered_unclassified_symbols"
                && hazard.severity == "error"
                && hazard.details.iter().any(|detail| {
                    detail
                        .blocked_idents
                        .iter()
                        .any(|ident| ident == "facade::EscapedType(Struct)::escaped_field")
                })
        }));
        assert!(hazards.iter().any(|hazard| {
            hazard.code == "rendered_unproven_trait_default_methods"
                && hazard.severity == "error"
                && hazard.details.iter().any(|detail| {
                    detail
                        .blocked_idents
                        .iter()
                        .any(|ident| ident == "facade::EscapedTrait(Trait)::dead_default")
                })
        }));
        assert!(hazards
            .iter()
            .any(|hazard| hazard.code == "rendered_symbol_proof_incomplete"
                && hazard.severity == "warning"));
    }

    #[test]
    fn rendered_symbol_proof_treats_proc_macro_exports_as_scoped_unknowns() {
        let output = temp_output("rendered-symbol-proc-macro");
        write(
            output.join("support/proc_helpers/Cargo.toml"),
            r#"[package]
name = "proc_helpers"
version = "0.1.0"
edition = "2021"

[lib]
proc-macro = true
"#,
        );
        write(
            output.join("support/proc_helpers/src/lib.rs"),
            r#"use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn live_attr(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
"#,
        );
        let decisions = UsageDecisionIndex {
            retained_packages: BTreeSet::from(["proc_helpers".to_string()]),
            ..UsageDecisionIndex::default()
        };

        let proof = rendered_symbol_proof_report(&output, &decisions);

        assert_eq!(proof.status, "proven", "{proof:#?}");
        assert_eq!(proof.summary.rendered_callables, 1, "{proof:#?}");
        assert_eq!(proof.summary.macro_blocked_callables, 1, "{proof:#?}");
        assert_eq!(proof.summary.unclassified_callables, 0, "{proof:#?}");
        assert!(proof.entries.iter().any(|entry| {
            entry.kind == "callable"
                && entry.id == "proc_helpers::live_attr"
                && entry.classification == "blocked_by_unknown"
        }));
    }

    #[test]
    fn rendered_symbol_proof_treats_empty_binary_main_as_structural() {
        let output = temp_output("rendered-symbol-empty-main");
        write(
            output.join("app/Cargo.toml"),
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "app"
path = "src/main.rs"
"#,
        );
        write(output.join("app/src/main.rs"), "fn main() {}\n");
        let main = CallableId::Free {
            package: "app".to_string(),
            module_path: Vec::new(),
            name: "main".to_string(),
        };
        let decisions = UsageDecisionIndex {
            retained_packages: BTreeSet::from(["app".to_string()]),
            prunable_callables: BTreeSet::from([main]),
            ..UsageDecisionIndex::default()
        };

        let proof = rendered_symbol_proof_report(&output, &decisions);

        assert_eq!(proof.status, "proven", "{proof:#?}");
        assert_eq!(proof.summary.rendered_callables, 1, "{proof:#?}");
        assert_eq!(proof.summary.retained_callables, 1, "{proof:#?}");
        assert_eq!(proof.summary.prunable_callables, 0, "{proof:#?}");
        assert!(proof.entries.iter().any(|entry| {
            entry.kind == "callable"
                && entry.id == "app::main"
                && entry.classification == "retained"
        }));
    }

    #[test]
    fn rendered_symbol_proof_resolves_impl_self_type_from_super_glob() {
        let output = temp_output("rendered-symbol-super-glob-impl");
        write(
            output.join("app/Cargo.toml"),
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"
"#,
        );
        write(
            output.join("app/src/lib.rs"),
            r#"pub mod client;
"#,
        );
        write(
            output.join("app/src/client/mod.rs"),
            r#"pub struct MobileClient;
pub mod event_loop;
"#,
        );
        write(
            output.join("app/src/client/event_loop.rs"),
            r#"use super::*;

impl MobileClient {
    pub(crate) fn spawn_detached() {}
}
"#,
        );
        let callable = CallableId::Method {
            package: "app".to_string(),
            type_path: vec!["client".to_string(), "MobileClient".to_string()],
            trait_path: None,
            trait_input_type_paths: Vec::new(),
            method: "spawn_detached".to_string(),
        };
        let decisions = UsageDecisionIndex {
            retained_packages: BTreeSet::from(["app".to_string()]),
            used_callables: BTreeSet::from([callable]),
            used_items: BTreeSet::from([ItemId {
                package: "app".to_string(),
                module_path: vec!["client".to_string()],
                name: "MobileClient".to_string(),
                kind: ItemKind::Struct,
            }]),
            ..UsageDecisionIndex::default()
        };

        let proof = rendered_symbol_proof_report(&output, &decisions);

        assert_eq!(proof.status, "proven", "{proof:#?}");
        assert_eq!(proof.summary.unclassified_callables, 0, "{proof:#?}");
        assert!(proof.entries.iter().any(|entry| {
            entry.kind == "callable"
                && entry.id == "app::client::MobileClient::spawn_detached"
                && entry.classification == "retained"
        }));
    }

    #[test]
    fn rendered_symbol_proof_blocks_unproven_trait_default_methods() {
        let output = temp_output("rendered-symbol-trait-default");
        write(
            output.join("trait_defaults/Cargo.toml"),
            r#"[package]
name = "trait_defaults"
version = "0.1.0"
edition = "2021"
"#,
        );
        write(
            output.join("trait_defaults/src/lib.rs"),
            r#"pub trait Reader {
    fn read(&self) -> u32;

    fn dead_default(&self) -> u32 {
        0
    }
}
"#,
        );
        let trait_item = ItemId {
            package: "trait_defaults".to_string(),
            module_path: Vec::new(),
            name: "Reader".to_string(),
            kind: ItemKind::Trait,
        };
        let decisions = UsageDecisionIndex {
            retained_packages: BTreeSet::from(["trait_defaults".to_string()]),
            used_items: BTreeSet::from([trait_item]),
            ..UsageDecisionIndex::default()
        };

        let proof = rendered_symbol_proof_report(&output, &decisions);

        assert_eq!(proof.status, "failed", "{proof:#?}");
        assert_eq!(
            proof.summary.rendered_trait_default_methods, 1,
            "{proof:#?}"
        );
        assert_eq!(
            proof.summary.unproven_trait_default_methods, 1,
            "{proof:#?}"
        );
        assert!(proof.entries.iter().any(|entry| {
            entry.kind == "trait_default_method"
                && entry.id == "trait_defaults::Reader(Trait)::dead_default"
                && entry.classification == "unproven"
        }));
    }

    #[test]
    fn rendered_symbol_proof_allows_unknown_blocked_trait_default_methods() {
        let output = temp_output("rendered-symbol-blocked-trait-default");
        write(
            output.join("trait_defaults/Cargo.toml"),
            r#"[package]
name = "trait_defaults"
version = "0.1.0"
edition = "2021"
"#,
        );
        write(
            output.join("trait_defaults/src/lib.rs"),
            r#"pub trait Reader {
    fn read(&self) -> u32;

    fn dead_default(&self) -> u32 {
        0
    }
}
"#,
        );
        let trait_item = ItemId {
            package: "trait_defaults".to_string(),
            module_path: Vec::new(),
            name: "Reader".to_string(),
            kind: ItemKind::Trait,
        };
        let decisions = UsageDecisionIndex {
            retained_packages: BTreeSet::from(["trait_defaults".to_string()]),
            blocked_by_unknown_items: BTreeSet::from([trait_item]),
            ..UsageDecisionIndex::default()
        };

        let proof = rendered_symbol_proof_report(&output, &decisions);

        assert_eq!(proof.status, "proven", "{proof:#?}");
        assert_eq!(
            proof.summary.rendered_trait_default_methods, 1,
            "{proof:#?}"
        );
        assert_eq!(proof.summary.blocked_trait_default_methods, 1, "{proof:#?}");
        assert_eq!(
            proof.summary.unproven_trait_default_methods, 0,
            "{proof:#?}"
        );
        assert!(proof.entries.iter().any(|entry| {
            entry.kind == "trait_default_method"
                && entry.id == "trait_defaults::Reader(Trait)::dead_default"
                && entry.classification == "blocked_by_unknown"
        }));
    }

    #[test]
    fn rendered_symbol_proof_accounts_for_trait_and_impl_associated_items() {
        let output = temp_output("rendered-symbol-associated-items");
        write(
            output.join("assoc_case/Cargo.toml"),
            r#"[package]
name = "assoc_case"
version = "0.1.0"
edition = "2021"
"#,
        );
        write(
            output.join("assoc_case/src/lib.rs"),
            r#"pub struct Live;

pub trait Spec {
    type Item;
    const VERSION: u32;
    fn run(&self) -> u32;
}

impl Spec for Live {
    type Item = u32;
    const VERSION: u32 = 7;

    fn run(&self) -> u32 {
        Self::VERSION
    }
}
"#,
        );
        let live = ItemId {
            package: "assoc_case".to_string(),
            module_path: Vec::new(),
            name: "Live".to_string(),
            kind: ItemKind::Struct,
        };
        let spec = ItemId {
            package: "assoc_case".to_string(),
            module_path: Vec::new(),
            name: "Spec".to_string(),
            kind: ItemKind::Trait,
        };
        let run = CallableId::Method {
            package: "assoc_case".to_string(),
            type_path: vec!["Live".to_string()],
            trait_path: Some(vec!["Spec".to_string()]),
            trait_input_type_paths: Vec::new(),
            method: "run".to_string(),
        };
        let decisions = UsageDecisionIndex {
            retained_packages: BTreeSet::from(["assoc_case".to_string()]),
            used_callables: BTreeSet::from([run]),
            used_items: BTreeSet::from([live, spec]),
            ..UsageDecisionIndex::default()
        };

        let proof = rendered_symbol_proof_report(&output, &decisions);

        assert_eq!(proof.status, "proven", "{proof:#?}");
        assert_eq!(proof.summary.rendered_assoc_items, 4, "{proof:#?}");
        assert_eq!(proof.summary.blocked_assoc_items, 4, "{proof:#?}");
        assert_eq!(proof.summary.prunable_assoc_items, 0, "{proof:#?}");
        assert_eq!(proof.summary.unclassified_assoc_items, 0, "{proof:#?}");
        assert!(proof.entries.iter().any(|entry| {
            entry.kind == "assoc_item"
                && entry.id == "assoc_case::Spec(Trait)::Item(Type)"
                && entry.classification == "blocked_by_unknown"
        }));
        assert!(proof.entries.iter().any(|entry| {
            entry.kind == "assoc_item"
                && entry.id == "assoc_case::<Live as Spec>::VERSION(Const)"
                && entry.classification == "blocked_by_unknown"
        }));
    }

    #[test]
    fn rendered_symbol_proof_uses_assoc_item_decision_index() {
        let output = temp_output("rendered-symbol-associated-item-decisions");
        write(
            output.join("assoc_case/Cargo.toml"),
            r#"[package]
name = "assoc_case"
version = "0.1.0"
edition = "2021"
"#,
        );
        write(
            output.join("assoc_case/src/lib.rs"),
            r#"pub struct Live;

impl Live {
    pub const LIVE: u32 = 1;
    pub const DEAD: u32 = 2;
}
"#,
        );
        let live = ItemId {
            package: "assoc_case".to_string(),
            module_path: Vec::new(),
            name: "Live".to_string(),
            kind: ItemKind::Struct,
        };
        let decisions = UsageDecisionIndex {
            retained_packages: BTreeSet::from(["assoc_case".to_string()]),
            used_items: BTreeSet::from([live]),
            ..UsageDecisionIndex::default()
        };
        let member_decisions = render::RenderedMemberDecisionIndex {
            retained_assoc_items: BTreeSet::from(["assoc_case::Live::LIVE(Const)".to_string()]),
            prunable_assoc_items: BTreeSet::from(["assoc_case::Live::DEAD(Const)".to_string()]),
            ..render::RenderedMemberDecisionIndex::default()
        };

        let proof =
            rendered_symbol_proof_report_with_members(&output, &decisions, &member_decisions);

        assert_eq!(proof.status, "failed", "{proof:#?}");
        assert_eq!(proof.summary.rendered_assoc_items, 2, "{proof:#?}");
        assert_eq!(proof.summary.retained_assoc_items, 1, "{proof:#?}");
        assert_eq!(proof.summary.prunable_assoc_items, 1, "{proof:#?}");
        assert!(proof.entries.iter().any(|entry| {
            entry.kind == "assoc_item"
                && entry.id == "assoc_case::Live::LIVE(Const)"
                && entry.classification == "retained"
        }));
        assert!(proof.entries.iter().any(|entry| {
            entry.kind == "assoc_item"
                && entry.id == "assoc_case::Live::DEAD(Const)"
                && entry.classification == "prunable"
        }));
    }

    #[test]
    fn rendered_symbol_proof_blocks_prunable_module_declarations() {
        let output = temp_output("rendered-symbol-prunable-module");
        write(
            output.join("module_case/Cargo.toml"),
            r#"[package]
name = "module_case"
version = "0.1.0"
edition = "2021"
"#,
        );
        write(output.join("module_case/src/lib.rs"), "mod dead;\n");
        write(output.join("module_case/src/dead.rs"), "");
        let dead_module = ItemId {
            package: "module_case".to_string(),
            module_path: Vec::new(),
            name: "dead".to_string(),
            kind: ItemKind::Mod,
        };
        let decisions = UsageDecisionIndex {
            retained_packages: BTreeSet::from(["module_case".to_string()]),
            prunable_items: BTreeSet::from([dead_module]),
            ..UsageDecisionIndex::default()
        };

        let proof = rendered_symbol_proof_report(&output, &decisions);

        assert_eq!(proof.status, "failed", "{proof:#?}");
        assert_eq!(proof.summary.rendered_items, 1, "{proof:#?}");
        assert_eq!(proof.summary.prunable_items, 1, "{proof:#?}");
        assert!(proof.entries.iter().any(|entry| {
            entry.kind == "item"
                && entry.id == "module_case::dead(Mod)"
                && entry.classification == "prunable"
        }));
    }

    #[test]
    fn rendered_symbol_proof_allows_structural_module_declarations() {
        let output = temp_output("rendered-symbol-structural-module");
        write(
            output.join("module_case/Cargo.toml"),
            r#"[package]
name = "module_case"
version = "0.1.0"
edition = "2021"
"#,
        );
        write(output.join("module_case/src/lib.rs"), "mod live;\n");
        write(output.join("module_case/src/live.rs"), "pub struct Live;\n");
        let live_module = ItemId {
            package: "module_case".to_string(),
            module_path: Vec::new(),
            name: "live".to_string(),
            kind: ItemKind::Mod,
        };
        let live_struct = ItemId {
            package: "module_case".to_string(),
            module_path: vec!["live".to_string()],
            name: "Live".to_string(),
            kind: ItemKind::Struct,
        };
        let decisions = UsageDecisionIndex {
            retained_packages: BTreeSet::from(["module_case".to_string()]),
            used_items: BTreeSet::from([live_struct]),
            prunable_items: BTreeSet::from([live_module]),
            ..UsageDecisionIndex::default()
        };

        let proof = rendered_symbol_proof_report(&output, &decisions);

        assert_eq!(proof.status, "proven", "{proof:#?}");
        assert_eq!(proof.summary.rendered_items, 2, "{proof:#?}");
        assert_eq!(proof.summary.prunable_items, 0, "{proof:#?}");
        assert_eq!(proof.summary.retained_items, 2, "{proof:#?}");
        assert!(proof.entries.iter().any(|entry| {
            entry.kind == "item"
                && entry.id == "module_case::live(Mod)"
                && entry.classification == "retained"
        }));
    }

    #[test]
    fn rendered_symbol_proof_allows_source_include_module_declarations() {
        let output = temp_output("rendered-symbol-source-include-module");
        write(
            output.join("module_case/Cargo.toml"),
            r#"[package]
name = "module_case"
version = "0.1.0"
edition = "2021"
"#,
        );
        write(
            output.join("module_case/src/lib.rs"),
            r#"mod generated {
    include!(concat!(env!("OUT_DIR"), "/generated.rs"));
}
"#,
        );
        let generated_module = ItemId {
            package: "module_case".to_string(),
            module_path: Vec::new(),
            name: "generated".to_string(),
            kind: ItemKind::Mod,
        };
        let decisions = UsageDecisionIndex {
            retained_packages: BTreeSet::from(["module_case".to_string()]),
            prunable_items: BTreeSet::from([generated_module]),
            ..UsageDecisionIndex::default()
        };

        let proof = rendered_symbol_proof_report(&output, &decisions);

        assert_eq!(proof.status, "proven", "{proof:#?}");
        assert_eq!(proof.summary.rendered_items, 1, "{proof:#?}");
        assert_eq!(proof.summary.prunable_items, 0, "{proof:#?}");
        assert!(proof.entries.iter().any(|entry| {
            entry.kind == "item"
                && entry.id == "module_case::generated(Mod)"
                && entry.classification == "retained"
        }));
    }

    #[test]
    fn rendered_symbol_proof_blocks_prunable_struct_members() {
        let output = temp_output("rendered-symbol-prunable-struct-member");
        write(
            output.join("member_case/Cargo.toml"),
            r#"[package]
name = "member_case"
version = "0.1.0"
edition = "2021"
"#,
        );
        write(
            output.join("member_case/src/lib.rs"),
            r#"pub struct Live {
    pub keep: u32,
    pub dead: u32,
}
"#,
        );
        let live = ItemId {
            package: "member_case".to_string(),
            module_path: Vec::new(),
            name: "Live".to_string(),
            kind: ItemKind::Struct,
        };
        let decisions = UsageDecisionIndex {
            retained_packages: BTreeSet::from(["member_case".to_string()]),
            used_items: BTreeSet::from([live]),
            ..UsageDecisionIndex::default()
        };
        let member_decisions = render::RenderedMemberDecisionIndex {
            retained: BTreeSet::from(["member_case::Live(Struct)::keep".to_string()]),
            prunable: BTreeSet::from(["member_case::Live(Struct)::dead".to_string()]),
            ..render::RenderedMemberDecisionIndex::default()
        };

        let proof =
            rendered_symbol_proof_report_with_members(&output, &decisions, &member_decisions);

        assert_eq!(proof.status, "failed", "{proof:#?}");
        assert_eq!(proof.summary.rendered_members, 2, "{proof:#?}");
        assert_eq!(proof.summary.retained_members, 1, "{proof:#?}");
        assert_eq!(proof.summary.prunable_members, 1, "{proof:#?}");
        assert!(proof.entries.iter().any(|entry| {
            entry.kind == "member"
                && entry.id == "member_case::Live(Struct)::dead"
                && entry.classification == "prunable"
        }));
    }

    #[test]
    fn rendered_symbol_proof_allows_unknown_blocked_enum_members() {
        let output = temp_output("rendered-symbol-blocked-enum-member");
        write(
            output.join("member_case/Cargo.toml"),
            r#"[package]
name = "member_case"
version = "0.1.0"
edition = "2021"
"#,
        );
        write(
            output.join("member_case/src/lib.rs"),
            r#"pub enum Live {
    Keep,
    MaybeMacroUsed,
}
"#,
        );
        let live = ItemId {
            package: "member_case".to_string(),
            module_path: Vec::new(),
            name: "Live".to_string(),
            kind: ItemKind::Enum,
        };
        let decisions = UsageDecisionIndex {
            retained_packages: BTreeSet::from(["member_case".to_string()]),
            blocked_by_unknown_items: BTreeSet::from([live]),
            ..UsageDecisionIndex::default()
        };
        let member_decisions = render::RenderedMemberDecisionIndex {
            blocked_by_unknown: BTreeSet::from([
                "member_case::Live(Enum)::Keep".to_string(),
                "member_case::Live(Enum)::MaybeMacroUsed".to_string(),
            ]),
            ..render::RenderedMemberDecisionIndex::default()
        };

        let proof =
            rendered_symbol_proof_report_with_members(&output, &decisions, &member_decisions);

        assert_eq!(proof.status, "proven", "{proof:#?}");
        assert_eq!(proof.summary.rendered_members, 2, "{proof:#?}");
        assert_eq!(proof.summary.blocked_members, 2, "{proof:#?}");
        assert_eq!(proof.summary.prunable_members, 0, "{proof:#?}");
        assert_eq!(proof.summary.unclassified_members, 0, "{proof:#?}");
    }

    #[test]
    fn public_reexport_proof_hazards_block_prunable_and_unclassified_targets() {
        let report = PublicReexportProofReport {
            status: "failed".to_string(),
            summary: PublicReexportProofSummary {
                prunable_targets: 1,
                unclassified_targets: 1,
                source_parse_failures: 1,
                ..PublicReexportProofSummary::default()
            },
            entries: vec![
                PublicReexportProofEntry {
                    package: "facade".to_string(),
                    module_path: Some("api".to_string()),
                    visible: "DeadType".to_string(),
                    target: "model::DeadType".to_string(),
                    resolved_targets: vec!["model::DeadType".to_string()],
                    classification: "prunable".to_string(),
                },
                PublicReexportProofEntry {
                    package: "facade".to_string(),
                    module_path: None,
                    visible: "Escaped".to_string(),
                    target: "api::Escaped".to_string(),
                    resolved_targets: vec!["facade::api::Escaped".to_string()],
                    classification: "unclassified".to_string(),
                },
            ],
        };
        let mut hazards = Vec::new();

        add_public_reexport_proof_hazards(&report, &mut hazards);

        assert!(hazards
            .iter()
            .any(|hazard| hazard.code == "public_reexport_prunable_targets"
                && hazard.severity == "error"
                && hazard.details.iter().any(|detail| {
                    detail.package.as_deref() == Some("facade")
                        && detail.module_path.as_deref() == Some("api")
                        && detail
                            .blocked_idents
                            .iter()
                            .any(|ident| ident == "DeadType")
                })));
        assert!(hazards.iter().any(
            |hazard| hazard.code == "public_reexport_unclassified_targets"
                && hazard.severity == "error"
                && hazard
                    .details
                    .iter()
                    .any(|detail| detail.blocked_idents.iter().any(|ident| ident == "Escaped"))
        ));
        assert!(hazards
            .iter()
            .any(|hazard| hazard.code == "public_reexport_proof_incomplete"
                && hazard.severity == "warning"));
    }

    #[test]
    fn generated_package_source_roots_include_nested_support_packages() {
        let output = temp_output("generated-package-source-roots");
        write(
            output.join("Cargo.toml"),
            r#"[workspace]
members = ["root", "support/external-helper"]
"#,
        );
        write(
            output.join("root/Cargo.toml"),
            r#"[package]
name = "root"
version = "0.1.0"
edition = "2021"
"#,
        );
        write(output.join("root/src/lib.rs"), "pub fn live() {}\n");
        write(
            output.join("support/external-helper/Cargo.toml"),
            r#"[package]
name = "external-helper"
version = "0.1.0"
edition = "2021"
"#,
        );
        write(
            output.join("support/external-helper/src/lib.rs"),
            "pub fn helper() {}\n",
        );
        let packages = BTreeSet::from(["external-helper".to_string(), "root".to_string()]);

        let roots = generated_package_source_roots(&output, &packages);

        assert_eq!(roots.get("root"), Some(&output.join("root/src")));
        assert_eq!(
            roots.get("external-helper"),
            Some(&output.join("support/external-helper/src"))
        );
    }

    #[test]
    fn public_reexport_proof_scans_nested_support_package_sources() {
        let output = temp_output("support-public-reexport-proof");
        write(
            output.join("Cargo.toml"),
            r#"[workspace]
members = ["support/external_helper"]
"#,
        );
        write(
            output.join("support/external_helper/Cargo.toml"),
            r#"[package]
name = "external_helper"
version = "0.1.0"
edition = "2021"
"#,
        );
        write(
            output.join("support/external_helper/src/lib.rs"),
            r#"mod live;
mod dead;

pub use live::Retained;
pub use dead::Dead;
"#,
        );
        write(
            output.join("support/external_helper/src/live.rs"),
            "pub struct Retained;\n",
        );
        write(
            output.join("support/external_helper/src/dead.rs"),
            "pub struct Dead;\n",
        );
        let retained = ItemId {
            package: "external_helper".to_string(),
            module_path: vec!["live".to_string()],
            name: "Retained".to_string(),
            kind: ItemKind::Struct,
        };
        let prunable = ItemId {
            package: "external_helper".to_string(),
            module_path: vec!["dead".to_string()],
            name: "Dead".to_string(),
            kind: ItemKind::Struct,
        };
        let decisions = UsageDecisionIndex {
            retained_packages: BTreeSet::from(["external_helper".to_string()]),
            used_items: BTreeSet::from([retained]),
            prunable_items: BTreeSet::from([prunable]),
            ..UsageDecisionIndex::default()
        };

        let proof = public_reexport_proof_report(&output, &decisions);

        assert_eq!(proof.status, "failed", "{proof:#?}");
        assert_eq!(proof.summary.public_reexports, 2, "{proof:#?}");
        assert_eq!(proof.summary.retained_targets, 1, "{proof:#?}");
        assert_eq!(proof.summary.prunable_targets, 1, "{proof:#?}");
        assert!(proof.entries.iter().any(|entry| {
            entry.package == "external_helper"
                && entry.visible == "Dead"
                && entry.classification == "prunable"
                && entry
                    .resolved_targets
                    .iter()
                    .any(|target| target == "external_helper::dead::Dead")
        }));
    }

    #[test]
    fn public_reexport_proof_accepts_retained_type_surface_items() {
        let output = temp_output("surface-public-reexport-proof");
        write(
            output.join("Cargo.toml"),
            r#"[workspace]
members = ["support"]
"#,
        );
        write(
            output.join("support/Cargo.toml"),
            r#"[package]
name = "support"
version = "0.1.0"
edition = "2021"
"#,
        );
        write(
            output.join("support/src/lib.rs"),
            r#"mod api;

pub use api::Config;
"#,
        );
        write(output.join("support/src/api.rs"), "pub struct Config;\n");
        let config = ItemId {
            package: "support".to_string(),
            module_path: vec!["api".to_string()],
            name: "Config".to_string(),
            kind: ItemKind::Struct,
        };
        let decisions = UsageDecisionIndex {
            retained_packages: BTreeSet::from(["support".to_string()]),
            prunable_items: BTreeSet::from([config.clone()]),
            ..UsageDecisionIndex::default()
        };
        let retained_surface_items = BTreeSet::from([config.to_string()]);

        let proof = public_reexport_proof_report_with_retained_surface_items(
            &output,
            &decisions,
            &retained_surface_items,
        );

        assert_eq!(proof.status, "proven", "{proof:#?}");
        assert_eq!(proof.summary.retained_targets, 1, "{proof:#?}");
        assert_eq!(proof.summary.prunable_targets, 0, "{proof:#?}");
        assert!(proof.entries.iter().any(|entry| {
            entry.package == "support"
                && entry.visible == "Config"
                && entry.classification == "retained"
                && entry
                    .resolved_targets
                    .iter()
                    .any(|target| target == "support::api::Config")
        }));
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
            semantic_usage: None,
        };
        let mut hazards = Vec::new();

        add_semantic_inventory_hazard(&analyzer, 0, false, &mut hazards);

        assert!(hazards
            .iter()
            .any(|hazard| hazard.code == "semantic_inventory_not_applied"));
    }

    #[test]
    fn semantic_hazards_prefer_retained_slice_file_reports() {
        let retained_path = PathBuf::from("/tmp/slicer-root/src/lib.rs");
        let unrelated_path = PathBuf::from("/tmp/slicer-root/src/unrelated.rs");
        let semantic = SemanticReport {
            source_files: 2,
            analyzed_files: 2,
            unresolved_method_calls: 8,
            unqueried_paths: 5,
            selected_root_source_files: 1,
            file_reports: vec![
                SemanticFileReport {
                    path: retained_path.clone(),
                    analyzed: true,
                    queried_method_calls: 3,
                    resolved_method_calls: 3,
                    queried_paths: 2,
                    resolved_paths: 2,
                    ..SemanticFileReport::default()
                },
                SemanticFileReport {
                    path: unrelated_path,
                    analyzed: true,
                    queried_method_calls: 8,
                    unresolved_method_calls: 8,
                    paths: 5,
                    unqueried_paths: 5,
                    ..SemanticFileReport::default()
                },
            ],
            ..SemanticReport::default()
        };
        let retained_paths = BTreeSet::from([retained_path]);

        let (scope, metrics) = semantic_hazard_metrics(&semantic, &retained_paths);

        assert_eq!(scope, SemanticHazardScope::RetainedSlice);
        assert_eq!(metrics.source_files, 1);
        assert_eq!(metrics.unresolved_method_calls, 0);
        assert_eq!(metrics.unqueried_paths, 0);
    }

    #[test]
    fn semantic_hazards_fall_back_to_selected_roots_before_workspace() {
        let semantic = SemanticReport {
            source_files: 10,
            analyzed_files: 10,
            unresolved_method_calls: 6,
            unqueried_paths: 4,
            selected_root_source_files: 1,
            selected_root_analyzed_files: 1,
            selected_root_unresolved_method_calls: 0,
            selected_root_unqueried_paths: 0,
            ..SemanticReport::default()
        };

        let (scope, metrics) = semantic_hazard_metrics(&semantic, &BTreeSet::new());

        assert_eq!(scope, SemanticHazardScope::SelectedRoots);
        assert_eq!(metrics.source_files, 1);
        assert_eq!(metrics.unresolved_method_calls, 0);
        assert_eq!(metrics.unqueried_paths, 0);
    }

    #[test]
    fn semantic_hazards_keep_workspace_fallback_without_focused_inventory() {
        let semantic = SemanticReport {
            source_files: 10,
            analyzed_files: 9,
            failed_files: 1,
            unresolved_method_calls: 6,
            unqueried_paths: 4,
            ..SemanticReport::default()
        };

        let (scope, metrics) = semantic_hazard_metrics(&semantic, &BTreeSet::new());

        assert_eq!(scope, SemanticHazardScope::Workspace);
        assert_eq!(metrics.source_files, 10);
        assert_eq!(metrics.failed_files, 1);
        assert_eq!(metrics.unresolved_method_calls, 6);
        assert_eq!(metrics.unqueried_paths, 4);
    }

    #[test]
    fn benign_unresolved_diagnostics_do_not_count_as_semantic_hazards() {
        let diagnostic = semantic_unresolved_diagnostic(
            SemanticUnresolvedKind::MethodCall,
            SemanticUnresolvedCategory::Benign,
            "external trait method",
        );
        let diagnostics = vec![&diagnostic];

        assert_eq!(non_benign_unresolved_count(1, &diagnostics), 0);
        assert!(semantic_unresolved_details(diagnostics).is_empty());
    }

    #[test]
    fn macro_blocked_unresolved_diagnostics_do_not_create_generic_semantic_hazards() {
        let diagnostic = semantic_unresolved_diagnostic(
            SemanticUnresolvedKind::Path,
            SemanticUnresolvedCategory::MacroBlocked,
            "attribute helper path",
        );
        let diagnostics = vec![&diagnostic];

        assert_eq!(non_benign_unresolved_count(1, &diagnostics), 0);
        assert!(semantic_unresolved_details(diagnostics).is_empty());
    }

    #[test]
    fn dependency_risk_unresolved_diagnostics_remain_scoped_hazards() {
        let diagnostic = semantic_unresolved_diagnostic(
            SemanticUnresolvedKind::Path,
            SemanticUnresolvedCategory::DependencyRisk,
            "local enum variant",
        );
        let diagnostics = vec![&diagnostic];
        let details = semantic_unresolved_details(diagnostics);

        assert_eq!(non_benign_unresolved_count(1, &[&diagnostic]), 1);
        assert_eq!(details.len(), 1);
        assert!(details[0].subject.contains("category=dependency_risk;"));

        let hazard = production_hazard_with_details(
            "semantic_unresolved_paths",
            "warning",
            "unresolved path",
            details,
        );
        assert_eq!(unknown_surface_category(&hazard), "dependency_risk");
    }

    #[test]
    fn semantic_unresolved_diagnostics_ignore_pruned_owners() {
        let retained = CallableId::Free {
            package: "app".to_string(),
            module_path: vec!["ui".to_string()],
            name: "selected".to_string(),
        };
        let pruned = CallableId::Free {
            package: "app".to_string(),
            module_path: vec!["ui".to_string()],
            name: "dead_sibling".to_string(),
        };
        let reduced = ReducedProject {
            root: RootId::Callable(retained.clone()),
            roots: vec![RootId::Callable(retained.clone())],
            packages: BTreeSet::from(["app".to_string()]),
            reachable: BTreeSet::from([retained.clone()]),
            reachable_items: BTreeSet::new(),
            evidence: ReductionEvidence::default(),
        };
        let mut retained_diagnostic = semantic_unresolved_diagnostic(
            SemanticUnresolvedKind::Path,
            SemanticUnresolvedCategory::DependencyRisk,
            "retained owner",
        );
        retained_diagnostic.owner = Some(SemanticOwnerId::Callable(retained));
        let mut pruned_diagnostic = semantic_unresolved_diagnostic(
            SemanticUnresolvedKind::Path,
            SemanticUnresolvedCategory::DependencyRisk,
            "pruned owner",
        );
        pruned_diagnostic.owner = Some(SemanticOwnerId::Callable(pruned));

        assert!(semantic_unresolved_owner_is_retained(
            &reduced,
            &retained_diagnostic
        ));
        assert!(!semantic_unresolved_owner_is_retained(
            &reduced,
            &pruned_diagnostic
        ));
    }

    #[test]
    fn semantic_unresolved_project_reexport_paths_are_covered_by_rendered_items() {
        let owner = CallableId::Free {
            package: "app".to_string(),
            module_path: vec!["theme".to_string()],
            name: "health_color".to_string(),
        };
        let item = ItemId {
            package: "dep-crate".to_string(),
            module_path: vec!["store".to_string(), "snapshot".to_string()],
            name: "ServerHealthSnapshot".to_string(),
            kind: ItemKind::Enum,
        };
        let project = super::model::Project {
            workspace: manifest::Workspace {
                root: PathBuf::from("/tmp/workspace"),
                packages: HashMap::from([
                    (
                        "app".to_string(),
                        manifest::Package {
                            name: "app".to_string(),
                            root: PathBuf::from("/tmp/workspace/app"),
                            lib_path: PathBuf::from("/tmp/workspace/app/src/lib.rs"),
                            entry_target: manifest::PackageTarget {
                                name: "app".to_string(),
                                kind: vec!["lib".to_string()],
                                src_path: PathBuf::from("/tmp/workspace/app/src/lib.rs"),
                                required_features: Vec::new(),
                            },
                            dependencies: vec![manifest::Dependency {
                                alias: "dep_crate".to_string(),
                                package: "dep-crate".to_string(),
                            }],
                            manifest: toml::Value::Table(Default::default()),
                        },
                    ),
                    (
                        "dep-crate".to_string(),
                        manifest::Package {
                            name: "dep-crate".to_string(),
                            root: PathBuf::from("/tmp/workspace/dep-crate"),
                            lib_path: PathBuf::from("/tmp/workspace/dep-crate/src/lib.rs"),
                            entry_target: manifest::PackageTarget {
                                name: "dep-crate".to_string(),
                                kind: vec!["lib".to_string()],
                                src_path: PathBuf::from("/tmp/workspace/dep-crate/src/lib.rs"),
                                required_features: Vec::new(),
                            },
                            dependencies: Vec::new(),
                            manifest: toml::Value::Table(Default::default()),
                        },
                    ),
                ]),
                manifest: toml::Value::Table(Default::default()),
            },
            files: HashMap::new(),
            functions: HashMap::new(),
            methods: HashMap::new(),
            items: HashMap::from([(
                item.clone(),
                super::model::ItemRecord {
                    package: item.package.clone(),
                    module_path: item.module_path.clone(),
                    span: SourceSpan {
                        file: PathBuf::from("/tmp/workspace/dep-crate/src/store/snapshot.rs"),
                        start_line: 1,
                        start_column: 0,
                        end_line: 3,
                        end_column: 1,
                    },
                    item: syn::parse_str("pub enum ServerHealthSnapshot { Connected }").unwrap(),
                    aliases: HashMap::new(),
                },
            )]),
            module_aliases: HashMap::from([(
                ("dep-crate".to_string(), vec!["store".to_string()]),
                HashMap::from([(
                    "ServerHealthSnapshot".to_string(),
                    vec!["snapshot".to_string(), "ServerHealthSnapshot".to_string()],
                )]),
            )]),
            glob_use_paths_by_module: HashMap::new(),
            source_files_by_module: HashMap::new(),
            methods_by_receiver: HashMap::new(),
            receivers_with_methods: HashSet::new(),
        };
        let proof = RenderedSymbolProofReport {
            status: "proven".to_string(),
            summary: RenderedSymbolProofSummary::default(),
            entries: vec![RenderedSymbolProofEntry {
                kind: "item".to_string(),
                id: item.to_string(),
                classification: "retained".to_string(),
            }],
        };
        let mut diagnostic = semantic_unresolved_diagnostic(
            SemanticUnresolvedKind::Path,
            SemanticUnresolvedCategory::DependencyRisk,
            "unresolved_path_anchor_matches_project_local_identifier",
        );
        diagnostic.symbol = Some("ServerHealthSnapshot".to_string());
        diagnostic.snippet = "dep_crate::store::ServerHealthSnapshot".to_string();
        diagnostic.owner = Some(SemanticOwnerId::Callable(owner.clone()));
        assert!(covered_project_path_unresolved_diagnostic(
            &project,
            Some(&proof),
            &diagnostic
        ));

        diagnostic.snippet = "ServerHealthSnapshot".to_string();
        diagnostic.owner = Some(SemanticOwnerId::Callable(owner));
        assert!(covered_project_path_unresolved_diagnostic(
            &project,
            Some(&proof),
            &diagnostic
        ));
    }

    #[test]
    fn semantic_unresolved_hazard_category_prefers_benign_when_all_details_are_benign() {
        let hazard = production_hazard_with_details(
            "semantic_unresolved_method_calls",
            "warning",
            "unresolved method",
            vec![ProductionHazardDetail {
                subject: "category=benign; kind=method_call; snippet=reader.read()".to_string(),
                package: None,
                module_path: None,
                file: Some(PathBuf::from("/tmp/src/lib.rs")),
                start_line: Some(1),
                cfg: None,
                blocked_idents: Vec::new(),
                suggested_cargo_args: Vec::new(),
            }],
        );

        assert_eq!(unknown_surface_category(&hazard), "benign");
    }

    #[test]
    fn semantic_item_edges_do_not_retain_pruned_struct_fields() {
        let root = temp_output("semantic-item-pruned-struct-field-source");
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

pub struct Facade {
    pub inner: Core,
}

pub struct Core {
    pub used: usize,
    pub unused: UnusedSurface,
}

pub struct UnusedSurface {
    pub value: usize,
}

#[opensourced]
pub fn entry(facade: Facade) -> usize {
    facade.inner.used
}
"#,
        );

        let workspace = manifest::load_workspace(&root).expect("workspace should load");
        let project = parse::parse_workspace(workspace).expect("workspace should parse");
        let core = project_item_named(&project, "Core", ItemKind::Struct);
        let unused_surface = project_item_named(&project, "UnusedSurface", ItemKind::Struct);
        let mut semantic_hints = SemanticReductionHints::default();
        semantic_hints.add_item_edge(SemanticOwnerId::Item(core.clone()), unused_surface.clone());

        let reduced = reduce::reduce_with_extra_roots_and_semantics(&project, &[], &semantic_hints)
            .expect("semantic reduction should work");

        assert!(
            reduced.reachable_items.contains(&core),
            "retained struct owner should stay reachable"
        );
        assert!(
            !reduced.reachable_items.contains(&unused_surface),
            "semantic item edges from fields pruned out of a struct surface should not retain the field type"
        );
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
        let semantic_usage = report
            .analyzer
            .semantic_usage
            .as_ref()
            .expect("RA-backed generation should report semantic usage mapping");
        assert_eq!(semantic_usage.indexed_callables, 3);
        assert!(semantic_usage.mapped_callables > 0);
        assert!(
            semantic_usage.reference_queries > 0,
            "RA-backed usage reports should collect reference-search evidence"
        );
        assert!(
            semantic_usage.callable_reference_edges + semantic_usage.item_reference_edges > 0,
            "RA-backed usage reports should promote reference owners into semantic reduction edges"
        );
        assert!(report
            .analyzer
            .notes
            .iter()
            .any(|note| note.contains("HIR usage mapping:")));
        if report.usage.semantic_proof.status != "complete_for_retained_packages" {
            assert!(report.production.hazards.iter().any(|hazard| {
                hazard.code == "semantic_reduction_hints_applied" && hazard.severity == "warning"
            }));
            assert!(report.production.hazards.iter().any(|hazard| {
                hazard.code == "semantic_inventory_partially_applied"
                    && hazard.severity == "warning"
            }));
        }
        assert!(!report
            .production
            .hazards
            .iter()
            .any(|hazard| hazard.code == "semantic_inventory_not_applied"));
    }

    #[test]
    #[cfg(feature = "ra-hir")]
    fn explicit_root_selectors_focus_ra_selected_root_inventory() {
        let root = temp_output("ra-explicit-root-source");
        let output = temp_output("ra-explicit-root-output");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        write(
            root.join("app/src/lib.rs"),
            r#"
pub struct Service;

impl Service {
    pub fn selected(&self) -> u32 {
        1
    }
}

pub fn entry(service: Service) -> u32 {
    service.selected()
}
"#,
        );

        let report = generate_with_analyzer_roots(
            GenerateOptions {
                workspace_root: root,
                output_root: output,
            },
            AnalyzerMode::RustAnalyzerHir,
            &["app::entry".to_string()],
        )
        .expect("RA-backed explicit-root generation should succeed");
        let semantic = report
            .analyzer
            .semantic
            .as_ref()
            .expect("RA-backed generation should report semantic inventory");

        assert_eq!(semantic.selected_root_source_files, 1);
        assert_eq!(semantic.selected_root_analyzed_files, 1);
        assert_eq!(semantic.selected_root_skipped_files, 0);
        assert_eq!(semantic.selected_root_unqueried_method_calls, 0);
        assert_eq!(semantic.selected_root_unqueried_paths, 0);
    }

    #[test]
    #[cfg(feature = "ra-hir")]
    fn ra_semantic_inventory_ignores_dead_same_file_budget_noise() {
        let root = temp_output("ra-semantic-budget-focus-source");
        let output = temp_output("ra-semantic-budget-focus-reduction");
        let opensourced_path = workspace_root().join("crates/opensourced");
        let noise_calls = (0..1_100)
            .map(|_| "    total += dead.noise();")
            .collect::<Vec<_>>()
            .join("\n");
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
            &format!(
                r#"
use opensourced::opensourced;

pub struct Dead;

impl Dead {{
    pub fn noise(&self) -> u32 {{
        1
    }}
}}

pub fn dead_budget_sink(dead: Dead) -> u32 {{
    let mut total = 0;
{noise_calls}
    total
}}

pub struct Live;

impl Live {{
    pub fn selected(&self) -> u32 {{
        7
    }}
}}

#[opensourced]
pub fn entry(live: Live) -> u32 {{
    live.selected()
}}
"#
            ),
        );

        let report = generate_with_analyzer(
            GenerateOptions {
                workspace_root: root,
                output_root: output,
            },
            AnalyzerMode::RustAnalyzerHir,
        )
        .expect("RA-backed generation should succeed");
        let semantic = report
            .analyzer
            .semantic
            .as_ref()
            .expect("semantic inventory should be available");

        assert!(
            semantic.queried_method_calls < semantic.method_call_budget,
            "dead same-file method calls should not spend the retained-owner semantic budget: {:?}",
            semantic
        );
        assert_eq!(semantic.unqueried_method_calls, 0);
        assert!(report
            .reachable
            .iter()
            .any(|callable| callable.to_string() == "app::Live::selected"));
        assert!(!report
            .production
            .hazards
            .iter()
            .any(|hazard| hazard.code == "semantic_method_call_budget_exhausted"));
    }

    #[test]
    #[cfg(feature = "ra-hir")]
    fn ra_usage_reference_search_recovers_from_attribute_name_collisions() {
        let root = temp_output("ra-reference-focus-source");
        let output = temp_output("ra-reference-focus-reduction");
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

#[doc = "Widget appears before the struct declaration name"]
pub struct Widget {
    value: u32,
}

#[doc = "helper appears before the function declaration name"]
pub fn helper(widget: Widget) -> u32 {
    widget.value
}

#[doc = "unused_fn appears before the function declaration name"]
pub fn unused_fn() -> u32 {
    7
}

#[doc = "entry appears before the function declaration name"]
#[opensourced]
pub fn entry(widget: Widget) -> u32 {
    helper(widget)
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
        let semantic_usage = report
            .analyzer
            .semantic_usage
            .as_ref()
            .expect("RA-backed generation should report semantic usage mapping");

        assert!(
            semantic_usage.failed_callable_reference_ids.is_empty(),
            "reference search should retry past doc/attribute name collisions for callables: {:?}",
            semantic_usage.failed_callable_reference_ids,
        );
        assert!(
            semantic_usage.failed_item_reference_ids.is_empty(),
            "reference search should retry past doc/attribute name collisions for items: {:?}",
            semantic_usage.failed_item_reference_ids,
        );
        assert_eq!(semantic_usage.reference_query_failures, 0);
    }

    #[test]
    #[cfg(feature = "ra-hir")]
    fn ra_feedback_records_outgoing_call_closure_edges() {
        let root = temp_output("ra-feedback-source");
        let output = temp_output("ra-feedback-reduction");
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

#[doc = "helper appears before the function declaration name"]
pub fn helper() -> u32 {
    1
}

#[doc = "entry appears before the function declaration name"]
#[opensourced]
pub fn entry() -> u32 {
    helper()
}
"#,
        );
        let report = generate_with_analyzer(
            GenerateOptions {
                workspace_root: root,
                output_root: output,
            },
            AnalyzerMode::RustAnalyzerFeedback,
        )
        .expect("RA feedback generation should succeed");

        assert!(report
            .analyzer
            .notes
            .iter()
            .any(|note| note.contains("RA feedback closure:")));
        assert!(report.analyzer.semantic_hints.total_edges() > 0);
        assert!(report
            .reachable
            .iter()
            .any(|callable| callable.to_string() == "app::helper"));
    }

    #[test]
    #[cfg(feature = "ra-hir")]
    fn production_default_records_outgoing_call_closure_edges() {
        let root = temp_output("production-ra-feedback-source");
        let output = temp_output("production-ra-feedback-reduction");
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

#[doc = "helper appears before the function declaration name"]
pub fn helper() -> u32 {
    1
}

#[doc = "entry appears before the function declaration name"]
#[opensourced]
pub fn entry() -> u32 {
    helper()
}
"#,
        );
        let report = generate_with_analyzer(
            GenerateOptions {
                workspace_root: root,
                output_root: output,
            },
            AnalyzerMode::production_default_for_build(),
        )
        .expect("production RA-backed generation should succeed");

        assert_eq!(
            report.analyzer.mode,
            AnalyzerMode::production_default_for_build()
        );
        assert!(report
            .analyzer
            .notes
            .iter()
            .any(|note| note.contains("RA feedback closure:")));
        assert!(report.analyzer.semantic_hints.total_edges() > 0);
        assert!(report
            .reachable
            .iter()
            .any(|callable| callable.to_string() == "app::helper"));
    }

    #[test]
    #[cfg(feature = "ra-hir")]
    fn ra_feedback_follows_newly_discovered_call_targets_transitively() {
        let root = temp_output("ra-feedback-transitive-source");
        let output = temp_output("ra-feedback-transitive-reduction");
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
mod bridge;
mod leaf;
mod noise;

use opensourced::opensourced;

pub struct Entry;
pub struct Bridge;
pub struct Leaf;

impl Entry {
    pub fn run(&self, bridge: Bridge, leaf: Leaf) -> u32 {
        bridge.dispatch(leaf)
    }
}

#[opensourced]
pub fn entry(entry: Entry, bridge: Bridge, leaf: Leaf) -> u32 {
    entry.run(bridge, leaf)
}
"#,
        );
        write(
            root.join("app/src/bridge.rs"),
            r#"
use crate::{Bridge, Leaf};

impl Bridge {
    pub fn dispatch(&self, leaf: Leaf) -> u32 {
        leaf.finish()
    }
}
"#,
        );
        write(
            root.join("app/src/leaf.rs"),
            r#"
use crate::Leaf;

impl Leaf {
    pub fn finish(&self) -> u32 {
        7
    }
}
"#,
        );
        write(
            root.join("app/src/noise.rs"),
            r#"
pub struct NoiseBridge;
pub struct NoiseLeaf;

impl NoiseBridge {
    pub fn dispatch(&self) -> u32 {
        1
    }
}

impl NoiseLeaf {
    pub fn finish(&self) -> u32 {
        2
    }
}
"#,
        );

        let report = generate_with_analyzer(
            GenerateOptions {
                workspace_root: root,
                output_root: output.clone(),
            },
            AnalyzerMode::RustAnalyzerFeedback,
        )
        .expect("RA feedback generation should succeed");

        assert!(report
            .reachable
            .iter()
            .any(|callable| callable.to_string() == "app::Bridge::dispatch"));
        assert!(report
            .reachable
            .iter()
            .any(|callable| callable.to_string() == "app::Leaf::finish"));

        let lib = fs::read_to_string(output.join("app/src/lib.rs")).unwrap();
        let bridge = fs::read_to_string(output.join("app/src/bridge.rs")).unwrap();
        let leaf = fs::read_to_string(output.join("app/src/leaf.rs")).unwrap();
        assert!(lib.contains("mod bridge;"));
        assert!(lib.contains("mod leaf;"));
        assert!(bridge.contains("pub fn dispatch(&self, leaf: Leaf) -> u32"));
        assert!(leaf.contains("pub fn finish(&self) -> u32"));
        assert!(!lib.contains("mod noise;"));
        assert!(!output.join("app/src/noise.rs").exists());

        let target_dir = temp_output("ra-feedback-transitive-target");
        let status = Command::new("cargo")
            .arg("check")
            .arg("--all-targets")
            .current_dir(&output)
            .env("CARGO_TARGET_DIR", target_dir)
            .status()
            .expect("cargo check should start");
        assert!(status.success(), "generated workspace should compile");
    }

    #[test]
    #[cfg(feature = "ra-hir")]
    fn ra_feedback_repeatedly_prunes_selected_same_module_item_sets() {
        struct SliceCase<'a> {
            name: &'a str,
            marked_roots: &'a [&'a str],
            expected_present: &'a [&'a str],
            expected_absent: &'a [&'a str],
        }

        let cases = [
            SliceCase {
                name: "one-function",
                marked_roots: &["alpha_entry"],
                expected_present: &[
                    "pub fn alpha_entry",
                    "fn alpha_helper",
                    "pub struct AlphaConfig",
                    "pub fn value(&self)",
                ],
                expected_absent: &[
                    "pub fn beta_entry",
                    "fn beta_helper",
                    "pub struct BetaConfig",
                    "pub struct GammaConfig",
                    "pub const GAMMA_LIMIT",
                    "pub fn unrelated_entry",
                    "fn unrelated_leaf",
                    "alpha_unused_method",
                ],
            },
            SliceCase {
                name: "two-functions",
                marked_roots: &["alpha_entry", "beta_entry"],
                expected_present: &[
                    "pub fn alpha_entry",
                    "fn alpha_helper",
                    "pub struct AlphaConfig",
                    "pub fn beta_entry",
                    "fn beta_helper",
                    "pub struct BetaConfig",
                ],
                expected_absent: &[
                    "pub struct GammaConfig",
                    "pub const GAMMA_LIMIT",
                    "pub fn unrelated_entry",
                    "fn unrelated_leaf",
                    "alpha_unused_method",
                ],
            },
            SliceCase {
                name: "three-mixed-items",
                marked_roots: &["alpha_entry", "beta_entry", "GammaConfig"],
                expected_present: &[
                    "pub fn alpha_entry",
                    "pub fn beta_entry",
                    "pub struct AlphaConfig",
                    "pub struct BetaConfig",
                    "pub struct GammaConfig",
                ],
                expected_absent: &[
                    "pub const GAMMA_LIMIT",
                    "pub fn gamma_entry",
                    "fn gamma_helper",
                    "pub fn unrelated_entry",
                    "fn unrelated_leaf",
                    "pub struct DeltaUnused",
                    "alpha_unused_method",
                ],
            },
        ];

        for case in cases {
            for pass in 0..3 {
                let root = temp_output(&format!("ra-feedback-{}-pass-{pass}-source", case.name));
                let output =
                    temp_output(&format!("ra-feedback-{}-pass-{pass}-reduction", case.name));
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
                    &same_module_item_set_fixture(case.marked_roots),
                );

                let report = generate_with_analyzer(
                    GenerateOptions {
                        workspace_root: root,
                        output_root: output.clone(),
                    },
                    AnalyzerMode::RustAnalyzerFeedback,
                )
                .unwrap_or_else(|error| {
                    panic!(
                        "RA feedback generation failed for case {} pass {pass}: {error}",
                        case.name
                    )
                });

                assert!(
                    report
                        .analyzer
                        .notes
                        .iter()
                        .any(|note| note.contains("RA feedback closure:")),
                    "case {} pass {pass} did not record RA feedback closure notes",
                    case.name,
                );

                let generated = fs::read_to_string(output.join("app/src/lib.rs")).unwrap();
                for expected in case.expected_present {
                    assert!(
                        generated.contains(expected),
                        "case {} pass {pass} missing expected snippet {expected:?}\n{generated}",
                        case.name,
                    );
                }
                for unexpected in case.expected_absent {
                    assert!(
                        !generated.contains(unexpected),
                        "case {} pass {pass} retained unrelated snippet {unexpected:?}\n{generated}",
                        case.name,
                    );
                }

                let target_dir =
                    temp_output(&format!("ra-feedback-{}-pass-{pass}-target", case.name));
                let status = Command::new("cargo")
                    .arg("check")
                    .arg("--all-targets")
                    .current_dir(&output)
                    .env("CARGO_TARGET_DIR", target_dir)
                    .status()
                    .expect("cargo check should start");
                assert!(
                    status.success(),
                    "case {} pass {pass} generated workspace did not compile\n{generated}",
                    case.name,
                );
            }
        }
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
            detail.subject == "app::entry: project_macro!"
                && detail
                    .file
                    .as_ref()
                    .is_some_and(|file| file.ends_with("app/src/lib.rs"))
                && detail.start_line == Some(5)
        }));
        assert!(report.macro_surfaces.surfaces.iter().any(|surface| {
            surface.kind == "macro_invocation"
                && surface.path == "project_macro"
                && surface.owner.as_deref() == Some("entry")
        }));
        assert_eq!(report.production.status, "requires_feedback");
    }

    #[test]
    fn imported_logging_macros_are_modeled_as_format_like() {
        let root = temp_output("logging-macro-source");
        let output = temp_output("logging-macro-output");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\ntracing = \"0.1\"\n",
                opensourced_path
            ),
        );
        write(
            root.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;
use tracing::{debug, warn};

#[opensourced]
pub fn entry() -> u32 {
    debug!(answer = macro_only_helper(), "macro dependency");
    warn!("entry finished");
    1
}

fn macro_only_helper() -> u32 {
    7
}

fn dead() -> u32 {
    9
}
"#,
        );

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: output.clone(),
        })
        .expect("reduction should succeed");

        assert!(
            !report
                .production
                .hazards
                .iter()
                .any(|hazard| hazard.code == "custom_macro_invocations"),
            "imported tracing/logging macros should not require expansion feedback: {:?}",
            report.production.hazards
        );
        assert!(report.macro_surfaces.surfaces.iter().all(|surface| {
            !(surface.kind == "macro_invocation"
                && matches!(surface.path.as_str(), "debug" | "warn"))
        }));

        let generated = fs::read_to_string(output.join("app/src/lib.rs")).unwrap();
        assert!(
            generated.contains("fn macro_only_helper"),
            "logging macro token dependencies must still retain called helpers:\n{generated}"
        );
        assert!(!generated.contains("fn dead"));
    }

    #[test]
    fn dependency_proven_async_trait_attribute_is_modeled() {
        let root = temp_output("async-trait-attribute-source");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nasync-trait = \"0.1\"\nopensourced = {{ path = {:?} }}\n",
                opensourced_path
            ),
        );
        write(
            root.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;

#[async_trait::async_trait]
#[opensourced]
pub trait Service {
    async fn run(&self) -> u32;
}
"#,
        );

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: temp_output("async-trait-attribute-output"),
        })
        .expect("reduction should succeed");

        assert!(
            !report
                .production
                .hazards
                .iter()
                .any(|hazard| hazard.code == "custom_attribute_macros"),
            "dependency-proven async_trait should not be reported as an unknown attribute macro: {:?}",
            report.production.hazards
        );
        assert!(report.macro_surfaces.surfaces.iter().all(|surface| {
            !(surface.kind == "attribute_macro" && surface.path.contains("async_trait"))
        }));
    }

    #[test]
    fn dependency_proven_serde_and_thiserror_derives_are_modeled() {
        let root = temp_output("known-derive-source");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\nserde = {{ version = \"1\", features = [\"derive\"] }}\nthiserror = \"2\"\n",
                opensourced_path
            ),
        );
        write(
            root.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;
use serde::{Deserialize, Serialize};

#[opensourced]
pub fn entry() -> Result<Wire, AppError> {
    Ok(Wire { value: 1 })
}

#[derive(Serialize, Deserialize)]
pub struct Wire {
    pub value: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("bad {code}")]
    Bad { code: u32 },
}
"#,
        );

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: temp_output("known-derive-output"),
        })
        .expect("reduction should succeed");

        assert!(
            !report
                .production
                .hazards
                .iter()
                .any(|hazard| hazard.code == "custom_derive_macros"),
            "dependency/import-proven serde and thiserror derives should not be unknown macro blockers: {:?}",
            report.production.hazards
        );
        assert!(report.macro_surfaces.surfaces.iter().all(|surface| {
            !(surface.kind == "derive_macro"
                && matches!(
                    surface.path.as_str(),
                    "Serialize" | "Deserialize" | "thiserror::Error"
                ))
        }));
    }

    #[test]
    fn dependency_proven_uniffi_macros_are_modeled() {
        let root = temp_output("known-uniffi-macros-source");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\nuniffi = \"0.31\"\n",
                opensourced_path
            ),
        );
        write(
            root.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;

#[derive(uniffi::Object)]
pub struct Bridge;

#[derive(Debug, Clone, uniffi::Record)]
pub struct AppRecord {
    pub value: String,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum AppKind {
    One,
}

#[derive(Debug, uniffi::Error)]
pub enum AppError {
    Bad,
}

#[uniffi::export]
impl Bridge {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self
    }

    #[opensourced]
    pub fn entry(&self) -> Result<AppRecord, AppError> {
        Ok(AppRecord {
            value: "ok".to_string(),
        })
    }
}
"#,
        );

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: temp_output("known-uniffi-macros-output"),
        })
        .expect("reduction should succeed");

        assert!(
            report.production.hazards.iter().all(|hazard| {
                !matches!(
                    hazard.code.as_str(),
                    "custom_attribute_macros" | "custom_derive_macros"
                )
            }),
            "dependency-proven UniFFI derives and attributes should not be unknown macro blockers: {:?}",
            report.production.hazards
        );
        assert!(report.macro_surfaces.surfaces.iter().all(|surface| {
            !(matches!(surface.kind.as_str(), "attribute_macro" | "derive_macro")
                && surface.path.starts_with("uniffi"))
        }));
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
    fn suppresses_syntactic_method_fallback_hazard_without_local_candidate_retention() {
        let root = temp_output("method-fallback-no-candidates-source");
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

macro_rules! make_number {
    () => {
        7u32
    };
}

#[opensourced]
pub fn entry() -> u32 {
    make_number!().count_ones()
}
"#,
        );

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: temp_output("method-fallback-no-candidates-output"),
        })
        .expect("reduction should succeed");

        assert!(report
            .production
            .hazards
            .iter()
            .all(|hazard| hazard.code != "syntactic_method_fallbacks"));
    }

    #[test]
    fn external_associated_calls_do_not_trip_method_fallback_cap() {
        let root = temp_output("external-associated-call-no-cap-source");
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
    let values = std::collections::HashMap::<String, String>::new();
    values.len()
}

pub struct Worker;
pub struct Other;

impl Worker {
    pub fn new() -> Self {
        Self
    }
}

impl Other {
    pub fn new() -> Self {
        Self
    }
}
"#,
        );

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: temp_output("external-associated-call-no-cap-output"),
        })
        .expect("reduction should succeed");

        assert!(report
            .production
            .hazards
            .iter()
            .all(|hazard| hazard.code != "syntactic_method_fallback_cap"));
        assert!(
            report
                .production
                .hazards
                .iter()
                .all(|hazard| hazard.code != "syntactic_method_fallbacks"),
            "external associated calls must not retain unrelated same-name local impl methods: {:?}",
            report.production.hazards
        );
        assert!(
            report
                .reachable
                .iter()
                .map(ToString::to_string)
                .all(|callable| !callable.contains("Worker::new")
                    && !callable.contains("Other::new")),
            "external associated call fallback should stay typed and top-down: {:?}",
            report.reachable
        );
    }

    #[test]
    fn external_builder_chains_do_not_retain_unrelated_local_methods() {
        let root = temp_output("external-builder-chain-no-local-method-source");
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
use std::fs;
use std::path::PathBuf;

#[opensourced]
pub fn entry(path: PathBuf) {
    let _ = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path);
}

pub struct LocalFactory;

impl LocalFactory {
    pub fn create() -> Self {
        Self
    }
}
"#,
        );

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: temp_output("external-builder-chain-no-local-method-output"),
        })
        .expect("reduction should succeed");

        assert!(
            report
                .production
                .hazards
                .iter()
                .all(|hazard| hazard.code != "syntactic_method_fallbacks"),
            "external builder chain should not retain unrelated local methods: {:?}",
            report.production.hazards
        );
        assert!(
            report
                .reachable
                .iter()
                .map(ToString::to_string)
                .all(|callable| !callable.contains("LocalFactory::create")),
            "external builder chain fallback should stay typed and top-down: {:?}",
            report.reachable
        );
    }

    #[test]
    fn common_adapter_receiver_chains_do_not_trip_method_fallback_cap() {
        let root = temp_output("common-adapter-chain-no-method-cap-source");
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
pub fn entry(wire: Wire) -> bool {
    wire.node_id.trim().is_empty()
}

pub struct Wire {
    pub node_id: String,
}

pub struct LocalA;
pub struct LocalB;

impl LocalA {
    pub fn is_empty(&self) -> bool {
        false
    }
}

impl LocalB {
    pub fn is_empty(&self) -> bool {
        false
    }
}
"#,
        );

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: temp_output("common-adapter-chain-no-method-cap-output"),
        })
        .expect("reduction should succeed");

        assert!(
            report
                .production
                .hazards
                .iter()
                .all(|hazard| hazard.code != "syntactic_method_fallback_cap"),
            "std-like adapter receiver chains should not report capped same-name local method risk: {:?}",
            report.production.hazards
        );
        assert!(
            report
                .reachable
                .iter()
                .map(ToString::to_string)
                .all(|callable| !callable.contains("LocalA::is_empty")
                    && !callable.contains("LocalB::is_empty")),
            "std-like adapter receiver chains should not retain unrelated same-name local methods: {:?}",
            report.reachable
        );
    }

    #[test]
    fn common_indexed_collection_methods_do_not_trip_method_fallback_cap() {
        let root = temp_output("common-indexed-collection-no-method-cap-source");
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
pub fn entry(lines: Vec<&str>, index: usize, mut current: Vec<&str>, text: String) -> String {
    current.clear();
    let joined = lines[index..].join("\n");
    let _ = text.as_bytes().get(0);
    joined
}

pub struct LocalA;
pub struct LocalB;

impl LocalA {
    pub fn clear(&self) {}
    pub fn join(&self, _separator: &str) -> String {
        String::new()
    }
}

impl LocalB {
    pub fn clear(&self) {}
    pub fn join(&self, _separator: &str) -> String {
        String::new()
    }
}
"#,
        );

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: temp_output("common-indexed-collection-no-method-cap-output"),
        })
        .expect("reduction should succeed");

        assert!(
            report
                .production
                .hazards
                .iter()
                .all(|hazard| hazard.code != "syntactic_method_fallback_cap"),
            "indexed common collection receivers should not report capped same-name local method risk: {:?}",
            report.production.hazards
        );
        assert!(
            report
                .reachable
                .iter()
                .map(ToString::to_string)
                .all(|callable| !callable.contains("LocalA::clear")
                    && !callable.contains("LocalB::clear")
                    && !callable.contains("LocalA::join")
                    && !callable.contains("LocalB::join")),
            "indexed common collection receivers should not retain unrelated same-name local methods: {:?}",
            report.reachable
        );
    }

    #[test]
    fn literal_into_inside_external_enum_field_does_not_trip_conversion_cap() {
        let root = temp_output("literal-into-enum-field-no-cap-source");
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
        let mut source = r#"use opensourced::opensourced;

#[opensourced]
pub fn entry() -> Result<(), AppError> {
    Err(AppError::Message("empty".into()))
}

pub enum AppError {
    Message(String),
}
"#
        .to_string();
        for index in 0..=24 {
            source.push_str(&format!(
                r#"
pub struct Source{index};
pub struct Target{index};

impl From<Source{index}> for Target{index} {{
    fn from(_value: Source{index}) -> Self {{
        Self
    }}
}}
"#
            ));
        }
        write(root.join("app/src/lib.rs"), &source);

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: temp_output("literal-into-enum-field-no-cap-output"),
        })
        .expect("reduction should succeed");

        assert!(
            report
                .production
                .hazards
                .iter()
                .all(|hazard| hazard.code != "syntactic_method_fallback_cap"),
            "literal conversions into known external enum fields should not scan capped local From impls: {:?}",
            report.production.hazards
        );
    }

    #[test]
    fn generic_receiverless_method_fallbacks_do_not_retain_local_name_matches() {
        let root = temp_output("generic-method-fallback-no-local-name-source");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\nregex = \"1\"\n",
                opensourced_path
            ),
        );
        write(
            root.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;
use regex::Regex;

#[opensourced]
pub fn entry(input: &str) -> usize {
    let re = Regex::new("x").expect("valid regex");
    let captures = re.captures(input).expect("captures");
    let whole = captures.get(0).expect("whole match");
    whole.start()
}

pub struct LocalRunner;

impl LocalRunner {
    pub fn start(&self) -> usize {
        1
    }
}
"#,
        );

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: temp_output("generic-method-fallback-no-local-name-output"),
        })
        .expect("reduction should succeed");

        let generic_fallback_hazard = report
            .production
            .hazards
            .iter()
            .find(|hazard| hazard.code == "generic_method_name_fallbacks");
        assert!(
            generic_fallback_hazard.is_some(),
            "generic receiverless fallback debt should be reported: {:?}",
            report.production.hazards
        );
        let generic_fallback_hazard = generic_fallback_hazard.unwrap();
        assert!(
            generic_fallback_hazard.details.iter().any(|detail| {
                detail.subject.contains("method=start")
                    && detail.package.as_deref() == Some("app")
                    && detail.start_line.is_some()
            }),
            "generic receiverless fallback should include structured method evidence: {:?}",
            generic_fallback_hazard.details
        );
        assert!(
            report
                .reachable
                .iter()
                .map(ToString::to_string)
                .all(|callable| !callable.contains("LocalRunner::start")),
            "generic receiverless fallback should not retain unrelated local methods: {:?}",
            report.reachable
        );
    }

    #[test]
    fn prelude_receiver_adapter_methods_do_not_trip_generic_fallback() {
        let root = temp_output("prelude-receiver-adapter-no-generic-fallback-source");
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
pub fn entry(input: String) -> usize {
    if input.trim().is_empty() {
        0
    } else {
        input.len()
    }
}

pub struct LocalProbe;

impl LocalProbe {
    pub fn is_empty(&self) -> bool {
        false
    }
}
"#,
        );

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: temp_output("prelude-receiver-adapter-no-generic-fallback-output"),
        })
        .expect("reduction should succeed");

        assert!(
            report
                .production
                .hazards
                .iter()
                .all(|hazard| hazard.code != "generic_method_name_fallbacks"),
            "prelude receiver adapter calls should not emit generic method fallback debt: {:?}",
            report.production.hazards
        );
        assert!(
            report
                .reachable
                .iter()
                .map(ToString::to_string)
                .all(|callable| !callable.contains("LocalProbe::is_empty")),
            "prelude receiver adapter calls should not retain unrelated local methods: {:?}",
            report.reachable
        );
    }

    #[test]
    fn lock_guard_map_get_does_not_trip_generic_method_fallback() {
        let root = temp_output("lock-guard-map-get-no-generic-fallback-source");
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
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Clone)]
pub struct Metadata {
    pub name: String,
}

pub struct Store {
    inner: RwLock<HashMap<String, Metadata>>,
}

#[opensourced]
pub fn entry(store: &Store, key: &str) -> Option<Metadata> {
    store.get(key)
}

impl Store {
    pub fn get(&self, key: &str) -> Option<Metadata> {
        let guard = self.inner.read().expect("metadata lock");
        guard.get(key).cloned()
    }
}

pub struct Cache;

impl Cache {
    pub fn get(&mut self, _key: &str) -> usize {
        1
    }
}
"#,
        );

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: temp_output("lock-guard-map-get-no-generic-fallback-output"),
        })
        .expect("reduction should succeed");

        assert!(
            report
                .production
                .hazards
                .iter()
                .all(|hazard| hazard.code != "generic_method_name_fallbacks"),
            "lock guard map access should keep type argument context: {:?}",
            report.production.hazards
        );
        assert!(
            report
                .reachable
                .iter()
                .map(ToString::to_string)
                .all(|callable| !callable.contains("Cache::get")),
            "lock guard map fallback should not retain unrelated local get methods: {:?}",
            report.reachable
        );
    }

    #[test]
    fn poisoned_lock_into_inner_get_does_not_trip_generic_method_fallback() {
        let root = temp_output("poisoned-lock-into-inner-get-source");
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
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Clone)]
pub struct Session {
    pub name: String,
}

pub struct Store {
    sessions: RwLock<HashMap<String, Session>>,
}

#[opensourced]
pub fn entry(store: &Store, key: &str) -> Option<Session> {
    match store.sessions.read() {
        Ok(guard) => guard.get(key).cloned(),
        Err(error) => error.into_inner().get(key).cloned(),
    }
}

pub struct LocalCache;

impl LocalCache {
    pub fn get(&self, _key: &str) -> usize {
        1
    }
}
"#,
        );

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: temp_output("poisoned-lock-into-inner-get-output"),
        })
        .expect("reduction should succeed");

        assert!(
            report
                .production
                .hazards
                .iter()
                .all(|hazard| hazard.code != "generic_method_name_fallbacks"),
            "poisoned lock into_inner map access should be modeled as an external receiver chain: {:?}",
            report.production.hazards
        );
        assert!(
            report
                .reachable
                .iter()
                .map(ToString::to_string)
                .all(|callable| !callable.contains("LocalCache::get")),
            "poisoned lock fallback should not retain unrelated local get methods: {:?}",
            report.reachable
        );
    }

    #[test]
    fn pruned_struct_fields_do_not_retain_dependency_public_reexports() {
        let root = temp_output("pruned-field-reexport-source");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\", \"dep\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\ndep = {{ path = \"../dep\" }}\n",
                opensourced_path
            ),
        );
        write(
            root.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;

pub struct RootState {
    used: usize,
    unused: dep::Exported,
}

#[opensourced]
pub fn entry(state: &RootState) -> usize {
    state.used
}
"#,
        );
        write(
            root.join("dep/Cargo.toml"),
            "[package]\nname = \"dep\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        write(
            root.join("dep/src/lib.rs"),
            r#"pub use inner::Exported;

mod inner {
    pub struct Exported;
}
"#,
        );

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: temp_output("pruned-field-reexport-output"),
        })
        .expect("reduction should succeed");

        assert!(
            report
                .reachable_items
                .iter()
                .map(ToString::to_string)
                .all(|item| !item.contains("dep::inner::Exported")),
            "dependency reexports behind pruned fields should not be retained: {:?}",
            report.reachable_items
        );
        assert!(
            report.packages.iter().all(|package| package != "dep"),
            "dependency package should not be rendered solely for a pruned field: {:?}",
            report.packages
        );
    }

    #[test]
    fn public_fields_on_non_root_structs_are_pruned_when_unused() {
        let root = temp_output("non-root-public-field-pruning-source");
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

pub struct Facade {
    inner: Core,
}

pub struct Core {
    pub used: usize,
    pub unused: UnusedSurface,
}

pub struct UnusedSurface {
    pub value: usize,
}

#[opensourced]
pub fn entry(facade: &Facade) -> usize {
    facade.inner.used
}
"#,
        );

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: temp_output("non-root-public-field-pruning-output"),
        })
        .expect("reduction should succeed");

        assert!(
            report
                .reachable_items
                .iter()
                .map(ToString::to_string)
                .all(|item| !item.contains("UnusedSurface")),
            "unused public fields on non-root structs should not retain their types: {:?}",
            report.reachable_items
        );
    }

    #[test]
    fn callable_signature_macro_surface_does_not_retain_whole_export_impl() {
        let root = temp_output("callable-signature-macro-surface-source");
        let output = temp_output("callable-signature-macro-surface-output");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        write(
            root.join("app/src/lib.rs"),
            r#"pub struct Api {
    value: usize,
}

#[uniffi::export]
impl Api {
    pub fn create() -> Self {
        Self { value: helper() }
    }

    pub fn unused(&self) -> usize {
        unused_helper()
    }
}

fn helper() -> usize {
    1
}

fn unused_helper() -> usize {
    2
}
"#,
        );

        let report = generate_with_analyzer_roots(
            GenerateOptions {
                workspace_root: root,
                output_root: output.clone(),
            },
            AnalyzerMode::Syn,
            &["app::Api::create".to_string()],
        )
        .expect("reduction should succeed");

        let rendered = fs::read_to_string(output.join("app/src/lib.rs")).unwrap();
        assert!(rendered.contains("pub fn create"), "{rendered}");
        assert!(rendered.contains("fn helper"), "{rendered}");
        assert!(!rendered.contains("pub fn unused"), "{rendered}");
        assert!(!rendered.contains("fn unused_helper"), "{rendered}");
        assert!(report
            .reachable
            .iter()
            .any(|callable| callable.to_string() == "app::Api::create"));
        assert!(
            report
                .reachable
                .iter()
                .all(|callable| callable.to_string() != "app::Api::unused"),
            "{:?}",
            report.reachable
        );
    }

    #[test]
    fn enum_match_logging_does_not_retain_unconstructed_uniffi_variants() {
        let root = temp_output("enum-match-unconstructed-uniffi-variant-source");
        let output = temp_output("enum-match-unconstructed-uniffi-variant-output");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\nuniffi = \"0.31\"\n",
                opensourced_path
            ),
        );
        write(
            root.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;

#[opensourced]
#[uniffi::export]
pub fn entry() -> usize {
    let reducer = Reducer;
    reducer.emit(AppEvent::Used)
}

pub struct Reducer;

impl Reducer {
    pub fn emit(&self, event: AppEvent) -> usize {
        match event {
            AppEvent::Used => 1,
            AppEvent::Unused { payload } => payload.value,
        }
    }
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum AppEvent {
    Used,
    Unused { payload: UnusedPayload },
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct UnusedPayload {
    pub value: usize,
}
"#,
        );

        let _report = generate(GenerateOptions {
            workspace_root: root,
            output_root: output.clone(),
        })
        .expect("reduction should succeed");

        let rendered = fs::read_to_string(output.join("app/src/lib.rs")).unwrap();
        assert!(rendered.contains("AppEvent::Used => 1"), "{rendered}");
        assert!(
            !rendered.contains("UnusedPayload"),
            "unconstructed enum variant payload should be dropped:\n{rendered}"
        );
        assert!(
            !rendered.contains("AppEvent::Unused"),
            "match arms for pruned enum variants should be dropped:\n{rendered}"
        );
    }

    #[test]
    fn public_reexports_drop_pruned_enum_items() {
        let root = temp_output("public-reexport-pruned-enum-source");
        let output = temp_output("public-reexport-pruned-enum-output");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        write(
            root.join("app/src/lib.rs"),
            r#"pub mod updates;
pub use updates::{AppEvent, ThreadStreamingDeltaKind};

pub fn entry() -> usize {
    let reducer = Reducer;
    reducer.emit(AppEvent::Used)
}

pub struct Reducer;

impl Reducer {
    pub fn emit(&self, event: AppEvent) -> usize {
        match event {
            AppEvent::Used => 1,
            AppEvent::Unused { kind } => match kind {
                ThreadStreamingDeltaKind::AssistantText => 2,
            },
        }
    }
}
"#,
        );
        write(
            root.join("app/src/updates.rs"),
            r#"#[derive(Debug, Clone)]
pub enum AppEvent {
    Used,
    Unused { kind: ThreadStreamingDeltaKind },
}

#[derive(Debug, Clone)]
pub enum ThreadStreamingDeltaKind {
    AssistantText,
}
"#,
        );

        generate_with_analyzer_roots(
            GenerateOptions {
                workspace_root: root,
                output_root: output.clone(),
            },
            AnalyzerMode::Syn,
            &["app::entry".to_string()],
        )
        .expect("reduction should succeed");

        let rendered_lib = fs::read_to_string(output.join("app/src/lib.rs")).unwrap();
        let rendered_updates = fs::read_to_string(output.join("app/src/updates.rs")).unwrap();
        assert!(
            rendered_lib.contains("pub use updates::AppEvent;"),
            "{rendered_lib}"
        );
        assert!(
            !rendered_lib.contains("ThreadStreamingDeltaKind"),
            "public reexports should drop pruned enum targets:\n{rendered_lib}"
        );
        assert!(
            !rendered_updates.contains("ThreadStreamingDeltaKind"),
            "pruned enum payload target should not render:\n{rendered_updates}"
        );
        assert!(
            !rendered_updates.contains("Unused"),
            "unconstructed variant should not render:\n{rendered_updates}"
        );
    }

    #[test]
    fn guard_wrapped_field_access_retains_nested_struct_fields() {
        let root = temp_output("guard-wrapped-field-access-source");
        let output = temp_output("guard-wrapped-field-access-output");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        write(
            root.join("app/src/lib.rs"),
            r#"use std::sync::Mutex;

pub struct Holder {
    inner: Mutex<Inner>,
}

pub struct Inner {
    value: usize,
    unused: usize,
}

impl Holder {
    pub fn value(&self) -> usize {
        let inner = self.inner.lock().unwrap();
        inner.value
    }
}
"#,
        );

        generate_with_analyzer_roots(
            GenerateOptions {
                workspace_root: root,
                output_root: output.clone(),
            },
            AnalyzerMode::Syn,
            &["app::Holder::value".to_string()],
        )
        .expect("reduction should succeed");

        let rendered = fs::read_to_string(output.join("app/src/lib.rs")).unwrap();
        assert!(rendered.contains("value: usize"), "{rendered}");
        assert!(!rendered.contains("unused: usize"), "{rendered}");
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

        let cap_hazard = report
            .production
            .hazards
            .iter()
            .find(|hazard| hazard.code == "syntactic_method_fallback_cap")
            .expect("capped fallback hazard should be reported");
        assert!(cap_hazard.details.iter().any(|detail| {
            detail.subject
                == "app::entry: method=run; local_candidate_methods=2; receiver_candidates=0"
                && detail.package.as_deref() == Some("app")
                && detail
                    .file
                    .as_ref()
                    .is_some_and(|file| file.ends_with("app/src/lib.rs"))
                && detail.blocked_idents == vec!["run".to_string()]
        }));
        let rendered = fs::read_to_string(output.join("app/src/lib.rs")).unwrap();
        assert!(
            !rendered.contains("impl Other"),
            "ambiguous name-only fallback should not retain unrelated same-name impls:\n{rendered}"
        );
    }

    #[test]
    fn reports_function_pointer_callback_boundaries_and_proven_returned_dyn_surfaces() {
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
pub fn entry(
    callback: Callback,
    observer: &dyn Worker,
    direct: fn() -> usize,
) -> (Callback, Box<dyn Worker>) {
    let _ = observer.run() + direct();
    (callback, Box::new(Real))
}

pub fn Box() -> usize {
    99
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
            .find(|hazard| {
                hazard.code == "function_pointer_surfaces" && hazard.severity == "warning"
            })
            .expect("function pointer hazard should be reported");
        assert!(function_pointer.details.iter().any(|detail| {
            detail.subject == "app: fn () -> usize"
                && detail.package.as_deref() == Some("app")
                && detail.blocked_idents.is_empty()
                && detail
                    .file
                    .as_ref()
                    .is_some_and(|file| file.ends_with("app/src/lib.rs"))
                && detail.start_line == Some(3)
        }));
        assert!(!report
            .production
            .hazards
            .iter()
            .any(|hazard| hazard.code == "trait_object_surfaces"));
        let callback_boundary = report
            .production
            .hazards
            .iter()
            .find(|hazard| {
                hazard.code == "dynamic_callback_boundaries" && hazard.severity == "warning"
            })
            .expect("direct callback boundary warning should be reported");
        assert_eq!(callback_boundary.details.len(), 2);
        assert!(callback_boundary.details.iter().any(|detail| {
            detail.subject == "app: & dyn Worker"
                && detail.package.as_deref() == Some("app")
                && detail.blocked_idents == vec!["Worker"]
                && detail
                    .file
                    .as_ref()
                    .is_some_and(|file| file.ends_with("app/src/lib.rs"))
        }));
        assert!(callback_boundary.details.iter().any(|detail| {
            detail.subject == "app: & dyn Worker"
                && detail.package.as_deref() == Some("app")
                && detail.blocked_idents == vec!["Worker"]
                && detail
                    .file
                    .as_ref()
                    .is_some_and(|file| file.ends_with("app/src/lib.rs"))
        }));
        assert!(callback_boundary.details.iter().any(|detail| {
            detail.subject == "app: fn () -> usize"
                && detail.package.as_deref() == Some("app")
                && detail.blocked_idents.is_empty()
                && detail
                    .file
                    .as_ref()
                    .is_some_and(|file| file.ends_with("app/src/lib.rs"))
        }));
        let prunable_callables = report
            .usage
            .prunable
            .callables
            .iter()
            .map(ToString::to_string)
            .collect::<BTreeSet<_>>();
        assert!(
            prunable_callables.contains("app::Box"),
            "dynamic surface wrappers must not become unknown-retention blockers: {prunable_callables:?}",
        );
        assert_eq!(report.production.status, "requires_feedback");
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
    fn reports_copied_support_package_build_and_generated_source_hazards() {
        let root = temp_output("support-build-hazard-source");
        let output = temp_output("support-build-hazard-output");
        let external = temp_output("support-build-hazard-external");
        let helper = external.join("external-helper");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\nexternal-helper = {{ path = {:?} }}\n",
                opensourced_path, helper
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
            helper.join("Cargo.toml"),
            r#"[package]
name = "external-helper"
version = "0.1.0"
edition = "2021"
build = "build.rs"
"#,
        );
        write(
            helper.join("build.rs"),
            r#"use std::{env, fs, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(out_dir.join("generated.rs"), "pub fn generated() -> usize { 41 }\n").unwrap();
    println!("cargo:rustc-env=SUPPORT_TOKEN=token");
}
"#,
        );
        write(
            helper.join("src/lib.rs"),
            r#"include!(concat!(env!("OUT_DIR"), "/generated.rs"));

pub fn value() -> usize {
    generated() + env!("SUPPORT_TOKEN").len()
}
"#,
        );

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: output.clone(),
        })
        .expect("reduction should succeed");

        let build_script = report
            .production
            .hazards
            .iter()
            .find(|hazard| hazard.code == "retained_build_scripts" && hazard.severity == "error")
            .expect("copied support build script should be a retained build script hazard");
        assert!(build_script.details.iter().any(|detail| {
            detail.subject == "external-helper"
                && detail.package.as_deref() == Some("external-helper")
                && detail
                    .file
                    .as_ref()
                    .is_some_and(|file| file.ends_with("support/external-helper/build.rs"))
        }));

        let out_dir_include = report
            .production
            .hazards
            .iter()
            .find(|hazard| {
                hazard.code == "out_dir_source_include_macros" && hazard.severity == "error"
            })
            .expect("copied support OUT_DIR include should be a source include hazard");
        assert!(out_dir_include.details.iter().any(|detail| {
            detail.subject == "external-helper"
                && detail
                    .file
                    .as_ref()
                    .is_some_and(|file| file.ends_with("support/external-helper/src/lib.rs"))
        }));

        let compile_env = report
            .production
            .hazards
            .iter()
            .find(|hazard| hazard.code == "compile_env_macros" && hazard.severity == "error")
            .expect("copied support env! usage should be a compile env hazard");
        assert!(compile_env.details.iter().any(|detail| {
            detail.subject == "external-helper"
                && detail
                    .file
                    .as_ref()
                    .is_some_and(|file| file.ends_with("support/external-helper/src/lib.rs"))
        }));
        assert_eq!(report.production.status, "hazards_detected");
    }

    #[test]
    fn reports_copied_support_package_opaque_file_include_hazards() {
        let root = temp_output("support-include-hazard-source");
        let external = temp_output("support-include-hazard-external");
        let helper = external.join("external-helper");
        let output = temp_output("support-include-hazard-output");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(external.join("outside.txt"), "outside support state");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\nexternal-helper = {{ path = {:?} }}\n",
                opensourced_path, helper
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
            helper.join("Cargo.toml"),
            r#"[package]
name = "external-helper"
version = "0.1.0"
edition = "2021"
"#,
        );
        write(
            helper.join("src/lib.rs"),
            r#"pub fn value() -> usize {
    let dynamic = include_str!(support_path!());
    let absolute = include_str!("/definitely/outside/generated/slice.txt");
    let external = include_str!("../../outside.txt");
    dynamic.len() + absolute.len() + external.len()
}
"#,
        );

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: output,
        })
        .expect("reduction should succeed");

        let nonliteral = report
            .production
            .hazards
            .iter()
            .find(|hazard| hazard.code == "nonliteral_file_include_macros")
            .expect("copied support nonliteral include should be reported");
        assert!(nonliteral.details.iter().any(|detail| {
            detail.package.as_deref() == Some("external-helper")
                && detail
                    .file
                    .as_ref()
                    .is_some_and(|file| file.ends_with("support/external-helper/src/lib.rs"))
        }));

        let absolute = report
            .production
            .hazards
            .iter()
            .find(|hazard| hazard.code == "absolute_file_include_macros")
            .expect("copied support absolute include should be reported");
        assert!(absolute
            .details
            .iter()
            .any(|detail| detail.package.as_deref() == Some("external-helper")));

        let external = report
            .production
            .hazards
            .iter()
            .find(|hazard| hazard.code == "external_file_include_macros")
            .expect("copied support external include should be reported");
        assert!(external
            .details
            .iter()
            .any(|detail| detail.package.as_deref() == Some("external-helper")));
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
    fn narrows_retained_workspace_tokio_features_to_used_public_surface() {
        let root = temp_output("workspace-tokio-feature-narrowing-source");
        let output = temp_output("workspace-tokio-feature-narrowing-output");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            r#"[workspace]
members = ["app"]
resolver = "2"

[workspace.dependencies.tokio]
version = "1"
features = ["rt-multi-thread", "macros", "sync", "time", "net", "io-util"]
"#,
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\ntokio = {{ workspace = true }}\n",
                opensourced_path
            ),
        );
        write(
            root.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;
use tokio::sync::broadcast;

#[opensourced]
pub fn entry(sender: broadcast::Sender<u8>) -> usize {
    let _ = sender.send(1);
    1
}
"#,
        );

        generate(GenerateOptions {
            workspace_root: root,
            output_root: output.clone(),
        })
        .expect("reduction should succeed");

        let features = generated_workspace_dependency_features(&output, "tokio");

        assert_eq!(features, vec!["sync".to_string()]);
    }

    #[test]
    fn narrows_retained_workspace_uuid_features_to_used_constructors() {
        let root = temp_output("workspace-uuid-feature-narrowing-source");
        let output = temp_output("workspace-uuid-feature-narrowing-output");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            r#"[workspace]
members = ["app"]
resolver = "2"

[workspace.dependencies.uuid]
version = "1"
features = ["v4", "serde"]
"#,
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\nuuid = {{ workspace = true }}\n",
                opensourced_path
            ),
        );
        write(
            root.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;
use uuid::Uuid;

#[opensourced]
pub fn entry() -> String {
    let uuid = Uuid::new_v4().to_string();
    uuid.clone()
}
"#,
        );

        generate(GenerateOptions {
            workspace_root: root,
            output_root: output.clone(),
        })
        .expect("reduction should succeed");

        let features = generated_workspace_dependency_features(&output, "uuid");

        assert_eq!(features, vec!["v4".to_string()]);
    }

    #[test]
    fn retains_uuid_serde_feature_for_serde_derived_uuid_fields() {
        let root = temp_output("workspace-uuid-serde-feature-source");
        let output = temp_output("workspace-uuid-serde-feature-output");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            r#"[workspace]
members = ["app"]
resolver = "2"

[workspace.dependencies.serde]
version = "1"
features = ["derive"]

[workspace.dependencies.uuid]
version = "1"
features = ["v4", "serde"]
"#,
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\nserde = {{ workspace = true }}\nuuid = {{ workspace = true }}\n",
                opensourced_path
            ),
        );
        write(
            root.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Payload {
    id: Uuid,
}

#[opensourced]
pub fn entry() -> Payload {
    Payload { id: Uuid::new_v4() }
}
"#,
        );

        generate(GenerateOptions {
            workspace_root: root,
            output_root: output.clone(),
        })
        .expect("reduction should succeed");

        let features = generated_workspace_dependency_features(&output, "uuid")
            .into_iter()
            .collect::<BTreeSet<_>>();

        assert_eq!(
            features,
            BTreeSet::from(["serde".to_string(), "v4".to_string()])
        );
    }

    #[test]
    fn removes_uniffi_tokio_feature_without_async_exports() {
        let root = temp_output("uniffi-no-async-feature-source");
        let output = temp_output("uniffi-no-async-feature-output");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\nuniffi = {{ version = \"0.31\", features = [\"tokio\"] }}\n",
                opensourced_path
            ),
        );
        write(
            root.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;

uniffi::setup_scaffolding!();

#[derive(Debug, Clone, uniffi::Record)]
pub struct Payload {
    pub value: String,
}

#[uniffi::export]
#[opensourced]
pub fn entry() -> Payload {
    Payload {
        value: "ok".to_string(),
    }
}
"#,
        );

        generate(GenerateOptions {
            workspace_root: root,
            output_root: output.clone(),
        })
        .expect("reduction should succeed");

        assert_eq!(
            generated_package_dependency_features(&output, "app", "uniffi"),
            None
        );
    }

    #[test]
    fn retains_uniffi_tokio_feature_for_async_exports() {
        let root = temp_output("uniffi-async-feature-source");
        let output = temp_output("uniffi-async-feature-output");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\nuniffi = {{ version = \"0.31\", features = [\"tokio\"] }}\n",
                opensourced_path
            ),
        );
        write(
            root.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;

uniffi::setup_scaffolding!();

#[uniffi::export]
#[opensourced]
pub async fn entry() -> String {
    "ok".to_string()
}
"#,
        );

        generate(GenerateOptions {
            workspace_root: root,
            output_root: output.clone(),
        })
        .expect("reduction should succeed");

        assert_eq!(
            generated_package_dependency_features(&output, "app", "uniffi"),
            Some(vec!["tokio".to_string()])
        );
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
#[derive(Debug, CustomSerialize)]
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
            detail.subject == "app::Payload: #[custom_attr::decorate]"
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
        assert!(derive_hazard.details.iter().any(|detail| detail.subject
            == "app::Payload: derive CustomSerialize"
            && detail.start_line == Some(9)));
        assert!(report.macro_surfaces.surfaces.iter().any(|surface| {
            surface.kind == "attribute_macro"
                && surface.path == "custom_attr::decorate"
                && surface.owner.as_deref() == Some("Payload")
        }));
        assert!(report.macro_surfaces.surfaces.iter().any(|surface| {
            surface.kind == "derive_macro"
                && surface.path == "CustomSerialize"
                && surface.owner.as_deref() == Some("Payload")
        }));
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

#[custom_attr::decorate(maybe_macro_helper)]
pub fn risky() {}

pub fn maybe_macro_helper() -> usize {
    3
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
        assert!(report
            .usage
            .used
            .callables
            .iter()
            .any(|callable| callable.to_string() == "app::helper"));
        assert!(!report
            .usage
            .prunable
            .callables
            .iter()
            .any(|callable| callable.to_string() == "app::helper"));
    }

    #[test]
    fn feedback_diagnostics_widen_matching_type_roots_from_e0425() {
        let root = temp_output("feedback-widen-type-source");
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

pub struct Helper {
    pub value: usize,
}
"#,
        );

        let diagnostic = CheckDiagnostic {
            level: "error".to_string(),
            message: "cannot find type `Helper` in this scope".to_string(),
            code: Some("E0425".to_string()),
            package_id: Some("app 0.1.0 (path+file:///tmp/app)".to_string()),
            target: None,
            rendered: None,
            spans: Vec::new(),
            suggestions: Vec::new(),
        };
        let resolution =
            resolve_feedback_widening_roots(&root, std::slice::from_ref(&diagnostic), &[])
                .expect("feedback root resolution should load");

        assert!(resolution
            .matched_roots
            .iter()
            .any(|root| root == "app::Helper(Struct)"));
        assert_eq!(resolution.skipped_no_match, 0);

        let report = generate_with_analyzer_feedback(
            GenerateOptions {
                workspace_root: root,
                output_root: temp_output("feedback-widen-type-output"),
            },
            AnalyzerMode::Syn,
            &[diagnostic],
        )
        .expect("feedback widening should generate");

        assert!(report
            .feedback_widened_roots
            .iter()
            .any(|root| root.to_string() == "app::Helper(Struct)"));
        assert!(report
            .reachable_items
            .iter()
            .any(|item| item.to_string() == "app::Helper(Struct)"));
        assert!(report
            .usage
            .used
            .items
            .iter()
            .any(|item| item.to_string() == "app::Helper(Struct)"));
    }

    #[test]
    fn feedback_diagnostics_widen_owner_struct_roots_for_field_surface_errors() {
        let root = temp_output("feedback-widen-field-surface-source");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\", \"support\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\nsupport = {{ path = \"../support\" }}\n",
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
            root.join("support/Cargo.toml"),
            "[package]\nname = \"support\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        write(
            root.join("support/src/lib.rs"),
            r#"pub struct ThreadReadResponse {
    pub thread: usize,
    pub approval_policy: Option<usize>,
}

pub struct ThreadRealtimeStartParams {
    pub thread_id: String,
    pub dynamic_tools: Option<Vec<String>>,
}
"#,
        );
        let read_response_diagnostic = CheckDiagnostic {
            level: "error".to_string(),
            message: "no field `approval_policy` on type `ThreadReadResponse`".to_string(),
            code: Some("E0609".to_string()),
            package_id: Some("app 0.1.0 (path+file:///tmp/app)".to_string()),
            target: None,
            rendered: None,
            spans: Vec::new(),
            suggestions: Vec::new(),
        };
        let realtime_params_diagnostic = CheckDiagnostic {
            level: "error".to_string(),
            message: "struct `ThreadRealtimeStartParams` has no field named `dynamic_tools`"
                .to_string(),
            code: Some("E0560".to_string()),
            package_id: Some("app 0.1.0 (path+file:///tmp/app)".to_string()),
            target: None,
            rendered: None,
            spans: Vec::new(),
            suggestions: Vec::new(),
        };
        let diagnostics = vec![read_response_diagnostic, realtime_params_diagnostic];
        let resolution = resolve_feedback_widening_roots(&root, &diagnostics, &[])
            .expect("feedback root resolution should load");

        assert!(resolution
            .matched_roots
            .iter()
            .any(|root| root == "support::ThreadReadResponse(Struct)"));
        assert!(resolution
            .matched_roots
            .iter()
            .any(|root| root == "support::ThreadRealtimeStartParams(Struct)"));
        assert_eq!(resolution.skipped_no_match, 0);

        let output = temp_output("feedback-widen-field-surface-output");
        let report = generate_with_analyzer_feedback(
            GenerateOptions {
                workspace_root: root,
                output_root: output.clone(),
            },
            AnalyzerMode::Syn,
            &diagnostics,
        )
        .expect("feedback widening should generate");

        assert!(report
            .feedback_widened_roots
            .iter()
            .any(|root| root.to_string() == "support::ThreadReadResponse(Struct)"));
        assert!(report
            .feedback_widened_roots
            .iter()
            .any(|root| root.to_string() == "support::ThreadRealtimeStartParams(Struct)"));
        let rendered_support =
            fs::read_to_string(output.join("support/src/lib.rs")).expect("support should render");
        assert!(rendered_support.contains("pub approval_policy: Option<usize>"));
        assert!(rendered_support.contains("pub dynamic_tools: Option<Vec<String>>"));
    }

    #[test]
    fn feedback_diagnostics_fallback_to_dependency_package_roots() {
        let root = temp_output("feedback-widen-dependency-type-source");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\", \"support\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\nsupport = {{ path = \"../support\" }}\n",
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
            root.join("support/Cargo.toml"),
            "[package]\nname = \"support\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        write(
            root.join("support/src/lib.rs"),
            r#"pub struct SupportType {
    pub value: usize,
}
"#,
        );

        let diagnostic = CheckDiagnostic {
            level: "error".to_string(),
            message: "cannot find type `SupportType` in this scope".to_string(),
            code: Some("E0425".to_string()),
            package_id: Some("app 0.1.0 (path+file:///tmp/app)".to_string()),
            target: None,
            rendered: None,
            spans: Vec::new(),
            suggestions: Vec::new(),
        };
        let resolution = resolve_feedback_widening_roots(&root, &[diagnostic], &[])
            .expect("feedback root resolution should load dependency closure");

        assert!(resolution
            .matched_roots
            .iter()
            .any(|root| root == "support::SupportType(Struct)"));
        assert_eq!(resolution.skipped_no_match, 0);
    }

    #[test]
    fn feedback_resolution_logs_glob_import_context_for_missing_bare_symbol() {
        let root = temp_output("feedback-glob-context-source");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\", \"provider\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\nprovider = {{ path = \"../provider\" }}\n",
                opensourced_path
            ),
        );
        write(
            root.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;
use provider::conversation::*;

#[opensourced]
pub fn entry() -> usize {
    1
}
"#,
        );
        write(
            root.join("provider/Cargo.toml"),
            "[package]\nname = \"provider\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        write(
            root.join("provider/src/lib.rs"),
            r#"pub mod conversation {
    pub struct Existing;
}
"#,
        );

        let diagnostic = CheckDiagnostic {
            level: "error".to_string(),
            message: "cannot find type `ConversationItem` in this scope".to_string(),
            code: Some("E0412".to_string()),
            package_id: Some("app 0.1.0 (path+file:///tmp/app)".to_string()),
            target: None,
            rendered: None,
            spans: vec![CheckSpan {
                file_name: "app/src/lib.rs".to_string(),
                line_start: 5,
                line_end: 5,
                column_start: 22,
                column_end: 38,
                is_primary: true,
                text: Vec::new(),
            }],
            suggestions: Vec::new(),
        };
        let resolution =
            resolve_feedback_widening_roots(&root, std::slice::from_ref(&diagnostic), &[])
                .expect("feedback root resolution should load");

        assert!(resolution.matched_roots.is_empty(), "{resolution:#?}");
        assert_eq!(resolution.skipped_no_match, 1);
        let entry = resolution
            .entries
            .iter()
            .find(|entry| entry.symbol == "ConversationItem")
            .expect("missing symbol should have a resolution entry");
        assert_eq!(
            entry.glob_imports,
            vec!["provider::conversation::*".to_string()]
        );
        assert_eq!(
            entry.glob_import_module_roots,
            vec!["provider::conversation(Mod)".to_string()]
        );
        assert_eq!(
            entry.action,
            "no matching project-local root; inspect glob import provider module roots"
        );
    }

    #[test]
    fn feedback_resolution_logs_private_provider_globs_for_missing_bare_symbol() {
        let root = temp_output("feedback-private-provider-glob-context-source");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\", \"provider\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\nprovider = {{ path = \"../provider\" }}\n",
                opensourced_path
            ),
        );
        write(
            root.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;
use provider::conversation::*;

#[opensourced]
pub fn entry() -> usize {
    1
}
"#,
        );
        write(
            root.join("provider/Cargo.toml"),
            "[package]\nname = \"provider\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        write(
            root.join("provider/src/lib.rs"),
            r#"pub mod conversation;
pub mod conversation_uniffi;
"#,
        );
        write(
            root.join("provider/src/conversation.rs"),
            r#"use crate::conversation_uniffi::*;

pub fn hydrate() -> HydratedConversationItem {
    HydratedConversationItem
}
"#,
        );
        write(
            root.join("provider/src/conversation_uniffi.rs"),
            r#"pub struct HydratedConversationItem;
"#,
        );

        let diagnostic = CheckDiagnostic {
            level: "error".to_string(),
            message: "cannot find type `ConversationItem` in this scope".to_string(),
            code: Some("E0412".to_string()),
            package_id: Some("app 0.1.0 (path+file:///tmp/app)".to_string()),
            target: None,
            rendered: None,
            spans: vec![CheckSpan {
                file_name: "app/src/lib.rs".to_string(),
                line_start: 5,
                line_end: 5,
                column_start: 22,
                column_end: 38,
                is_primary: true,
                text: Vec::new(),
            }],
            suggestions: Vec::new(),
        };
        let resolution =
            resolve_feedback_widening_roots(&root, std::slice::from_ref(&diagnostic), &[])
                .expect("feedback root resolution should load");

        assert!(resolution.matched_roots.is_empty(), "{resolution:#?}");
        assert_eq!(resolution.skipped_no_match, 1);
        let entry = resolution
            .entries
            .iter()
            .find(|entry| entry.symbol == "ConversationItem")
            .expect("missing symbol should have a resolution entry");
        assert_eq!(
            entry.glob_imports,
            vec!["provider::conversation::*".to_string()]
        );
        assert_eq!(
            entry.glob_import_module_roots,
            vec!["provider::conversation(Mod)".to_string()]
        );
        assert_eq!(
            entry.glob_import_provider_internal_globs,
            vec!["provider::conversation_uniffi::*".to_string()]
        );
    }

    #[test]
    fn feedback_unresolved_external_import_prefers_crate_prefix_package_over_target_leaf_module() {
        let root = temp_output("feedback-widen-external-module-source");
        let output = temp_output("feedback-widen-external-module-output");
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\", \"model-crate\"]\nresolver = \"2\"\n",
        );
        write(
			root.join("app/Cargo.toml"),
			&format!(
				"[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\nmodel-crate = {{ path = \"../model-crate\" }}\n",
				opensourced_path
			),
		);
        write(
            root.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;

pub mod screens {
    pub mod conversation {
        pub struct WrongPackage;
    }
}

#[opensourced]
pub fn entry() -> usize {
    1
}
"#,
        );
        write(
            root.join("model-crate/Cargo.toml"),
            "[package]\nname = \"model-crate\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        write(
            root.join("model-crate/src/lib.rs"),
            "pub mod conversation;\n",
        );
        write(
            root.join("model-crate/src/conversation.rs"),
            r#"pub struct Needed {
    pub value: usize,
}
"#,
        );

        let diagnostic = CheckDiagnostic {
			level: "error".to_string(),
			message: "unresolved import `model_crate::conversation`".to_string(),
			code: Some("E0432".to_string()),
			package_id: Some("app 0.1.0 (path+file:///tmp/app)".to_string()),
			target: None,
			rendered: Some(
				"error[E0432]: unresolved import `model_crate::conversation`\n  |\n  | use model_crate::conversation::*;\n  |                  ^^^^^^^^^^^^ could not find `conversation` in `model_crate`\n"
					.to_string(),
			),
			spans: Vec::new(),
			suggestions: Vec::new(),
		};
        let resolution =
            resolve_feedback_widening_roots(&root, std::slice::from_ref(&diagnostic), &[])
                .expect("feedback root resolution should load");

        assert!(
            resolution
                .matched_roots
                .iter()
                .any(|root| root == "model-crate::conversation(Mod)"),
            "{resolution:#?}"
        );
        assert!(
            !resolution
                .matched_roots
                .iter()
                .any(|root| root == "app::screens::conversation(Mod)"),
            "{resolution:#?}"
        );

        let report = generate_with_analyzer_feedback(
            GenerateOptions {
                workspace_root: root,
                output_root: output.clone(),
            },
            AnalyzerMode::Syn,
            &[diagnostic],
        )
        .expect("feedback widening should render the dependency module");

        assert!(report
            .feedback_widened_roots
            .iter()
            .any(|root| root.to_string() == "model-crate::conversation(Mod)"));
        let model_lib = fs::read_to_string(output.join("model-crate/src/lib.rs")).unwrap();
        assert!(model_lib.contains("pub mod conversation;"), "{model_lib}");
        let app_lib = fs::read_to_string(output.join("app/src/lib.rs")).unwrap();
        assert!(!app_lib.contains("WrongPackage"), "{app_lib}");
    }

    #[test]
    fn feedback_diagnostics_widen_path_package_ids_and_conversion_impls() {
        let root = temp_output("feedback-widen-conversion-source");
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
pub fn entry() {}

pub struct Source;
pub struct Target;

impl From<Source> for Target {
    fn from(_: Source) -> Self {
        Target
    }
}
"#,
        );

        let report = generate_with_analyzer_feedback(
            GenerateOptions {
                workspace_root: root.clone(),
                output_root: temp_output("feedback-widen-conversion-output"),
            },
            AnalyzerMode::Syn,
            &[CheckDiagnostic {
                level: "error".to_string(),
                message: "the trait bound `Target: From<Source>` is not satisfied".to_string(),
                code: Some("E0277".to_string()),
                package_id: Some(format!("path+file://{}#0.1.0", root.join("app").display())),
                target: None,
                rendered: None,
                spans: Vec::new(),
                suggestions: Vec::new(),
            }],
        )
        .expect("feedback conversion widening should generate");

        assert_eq!(
            super::package_name_from_diagnostic_package_id(&format!(
                "path+file://{}#0.1.0",
                root.join("app").display()
            ))
            .as_deref(),
            Some("app")
        );
        assert!(report.feedback_widened_roots.iter().any(|root| {
            root.to_string()
                .contains("app::<Target as From<Source>>::from")
        }));
        assert!(report.reachable.iter().any(|callable| {
            callable
                .to_string()
                .contains("app::<Target as From<Source>>::from")
        }));
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

    fn generated_workspace_dependency_features(output: &Path, alias: &str) -> Vec<String> {
        fs::read_to_string(output.join("Cargo.toml"))
            .expect("generated workspace manifest should exist")
            .parse::<toml::Value>()
            .expect("generated workspace manifest should parse")
            .get("workspace")
            .and_then(|workspace| workspace.get("dependencies"))
            .and_then(|dependencies| dependencies.get(alias))
            .and_then(|dependency| dependency.get("features"))
            .and_then(toml::Value::as_array)
            .unwrap_or_else(|| {
                panic!("generated {alias} workspace dependency should keep features")
            })
            .iter()
            .map(|feature| {
                feature
                    .as_str()
                    .expect("features should be strings")
                    .to_string()
            })
            .collect()
    }

    fn generated_package_dependency_features(
        output: &Path,
        package: &str,
        alias: &str,
    ) -> Option<Vec<String>> {
        fs::read_to_string(output.join(package).join("Cargo.toml"))
            .expect("generated package manifest should exist")
            .parse::<toml::Value>()
            .expect("generated package manifest should parse")
            .get("dependencies")
            .and_then(|dependencies| dependencies.get(alias))
            .and_then(|dependency| dependency.get("features"))
            .and_then(toml::Value::as_array)
            .map(|features| {
                features
                    .iter()
                    .map(|feature| {
                        feature
                            .as_str()
                            .expect("features should be strings")
                            .to_string()
                    })
                    .collect()
            })
    }

    fn inactive_target_os() -> &'static str {
        if std::env::consts::OS == "windows" {
            "linux"
        } else {
            "windows"
        }
    }

    fn semantic_unresolved_diagnostic(
        kind: SemanticUnresolvedKind,
        category: SemanticUnresolvedCategory,
        reason: &str,
    ) -> SemanticUnresolvedDiagnostic {
        SemanticUnresolvedDiagnostic {
            kind,
            category,
            reason: reason.to_string(),
            file: PathBuf::from("/tmp/src/lib.rs"),
            start_line: 1,
            start_column: 0,
            end_line: 1,
            end_column: 12,
            snippet: "unresolved()".to_string(),
            ast_kind: kind.as_str().to_string(),
            symbol: Some("unresolved".to_string()),
            owner: None,
        }
    }

    fn project_callable_named(project: &super::model::Project, expected: &str) -> CallableId {
        project
            .functions
            .keys()
            .chain(project.methods.keys())
            .find(|callable| match callable {
                CallableId::Free { name, .. } => name == expected,
                CallableId::Method { method, .. } => method == expected,
            })
            .cloned()
            .unwrap_or_else(|| panic!("callable {expected} should exist"))
    }

    fn project_item_named(
        project: &super::model::Project,
        expected: &str,
        kind: ItemKind,
    ) -> ItemId {
        project
            .items
            .keys()
            .find(|item| item.name == expected && item.kind == kind)
            .cloned()
            .unwrap_or_else(|| panic!("item {expected}({kind:?}) should exist"))
    }

    #[cfg(feature = "ra-hir")]
    fn same_module_item_set_fixture(marked_roots: &[&str]) -> String {
        let marked_roots = marked_roots.iter().copied().collect::<BTreeSet<_>>();
        let marker = |name: &str| {
            if marked_roots.contains(name) {
                "#[opensourced]\n"
            } else {
                ""
            }
        };

        format!(
            r#"use opensourced::opensourced;

pub struct AlphaConfig {{
    pub value: u32,
}}

impl AlphaConfig {{
    pub fn value(&self) -> u32 {{
        self.value + alpha_helper()
    }}

    pub fn alpha_unused_method(&self) -> u32 {{
        unrelated_leaf()
    }}
}}

pub struct BetaConfig {{
    pub value: u32,
}}

impl BetaConfig {{
    pub fn value(&self) -> u32 {{
        self.value + beta_helper()
    }}
}}

{}pub struct GammaConfig {{
    pub value: u32,
}}

pub struct DeltaUnused {{
    pub value: u32,
}}

pub const GAMMA_LIMIT: u32 = 9;

{}pub fn alpha_entry(seed: u32) -> u32 {{
    let config = AlphaConfig {{
        value: seed + alpha_helper(),
    }};
    config.value()
}}

fn alpha_helper() -> u32 {{
    1
}}

{}pub fn beta_entry(seed: u32) -> u32 {{
    let config = BetaConfig {{
        value: seed + beta_helper(),
    }};
    config.value()
}}

fn beta_helper() -> u32 {{
    2
}}

pub fn gamma_entry(seed: u32) -> u32 {{
    seed + gamma_helper() + GAMMA_LIMIT
}}

fn gamma_helper() -> u32 {{
    3
}}

pub fn unrelated_entry() -> u32 {{
    unrelated_leaf()
}}

fn unrelated_leaf() -> u32 {{
    99
}}
"#,
            marker("GammaConfig"),
            marker("alpha_entry"),
            marker("beta_entry")
        )
    }
}
