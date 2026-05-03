mod analyzer;
mod feedback;
mod manifest;
mod model;
mod parse;
mod preflight;
mod reduce;
mod render;
mod repair;

use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

use model::{Project, ReducedProject};
use serde::Serialize;
use syn::{parse::Parser, punctuated::Punctuated, visit::Visit, Attribute, Macro, Meta};

pub use analyzer::{AnalyzerMode, AnalyzerReport, SemanticReport};
pub use feedback::{
    check_workspace, write_report, CheckDiagnostic, CheckOptions, CheckReport, CheckSpan,
    CheckSuggestion, CheckTarget, FeedbackHazard, FeedbackWideningCandidate,
    FeedbackWideningReport,
};
pub use model::{CallableId, ItemId, RootId, SourceSpan};
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
    let total_started = Instant::now();
    let phase_started = Instant::now();
    let analyzer = analyzer::load_report(&options.workspace_root, analyzer_mode)?;
    let analyzer_ms = elapsed_ms(phase_started);

    let phase_started = Instant::now();
    let workspace = manifest::load_workspace(&options.workspace_root)?;
    let manifest_ms = elapsed_ms(phase_started);

    let phase_started = Instant::now();
    let project = parse::parse_workspace(workspace)?;
    let parse_ms = elapsed_ms(phase_started);

    let phase_started = Instant::now();
    let reduced = reduce::reduce(&project)?;
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
            })
        })
        .collect()
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
    add_syntactic_production_hazards(project, reduced, &mut hazards);
    add_reduction_evidence_production_hazards(reduced, &mut hazards);

    if !analyzer.loaded {
        hazards.push(production_hazard(
            "analyzer_unavailable",
            "error",
            "configured analyzer did not load; generated reachability used fallback evidence",
        ));
    }
    add_semantic_inventory_hazard(analyzer, &mut hazards);

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
    hazards: &mut Vec<ProductionHazardReport>,
) {
    if analyzer.semantic.is_some() {
        hazards.push(production_hazard(
            "semantic_inventory_not_applied",
            "warning",
            "semantic analyzer inventory is report-only in this build; compiler feedback is still required before trusting the slice",
        ));
    }
}

fn add_workspace_production_hazards(
    project: &Project,
    reduced: &ReducedProject,
    hazards: &mut Vec<ProductionHazardReport>,
) {
    let retained_build_scripts = reduced
        .packages
        .iter()
        .filter_map(|package| project.workspace.packages.get(package))
        .filter(|package| package_build_script_path(package).is_some())
        .count();
    if retained_build_scripts > 0 {
        hazards.push(production_hazard(
            "retained_build_scripts",
            "warning",
            format!(
                "{} retained package build script(s) may generate source, link metadata, or asset dependencies outside the static parse tree",
                retained_build_scripts
            ),
        ));
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
        hazards.push(production_hazard(
            "source_include_macros",
            "warning",
            format!(
                "{} retained include! macro(s) may inject Rust source outside the static parse tree",
                counts.source_include_macros
            ),
        ));
    }
    if counts.nonliteral_file_include_macros > 0 {
        hazards.push(production_hazard(
            "nonliteral_file_include_macros",
            "warning",
            format!(
                "{} retained include_str!/include_bytes! macro(s) use non-literal paths; asset copying needs compiler feedback validation",
                counts.nonliteral_file_include_macros
            ),
        ));
    }
    if counts.custom_attribute_macros > 0 {
        hazards.push(production_hazard(
            "custom_attribute_macros",
            "warning",
            format!(
                "{} retained custom attribute macro/helper attribute(s) require compiler expansion to fully trust the slice",
                counts.custom_attribute_macros
            ),
        ));
    }
    if counts.custom_derive_macros > 0 {
        hazards.push(production_hazard(
            "custom_derive_macros",
            "warning",
            format!(
                "{} retained custom derive macro(s) may generate impls or bounds outside the static parse tree",
                counts.custom_derive_macros
            ),
        ));
    }
    if counts.custom_macro_invocations > 0 {
        hazards.push(production_hazard(
            "custom_macro_invocations",
            "warning",
            format!(
                "{} retained non-builtin macro invocation(s) may expand code outside the static parse tree",
                counts.custom_macro_invocations
            ),
        ));
    }
    if counts.conditional_compilation_attrs > 0 {
        hazards.push(production_hazard(
            "conditional_compilation_attrs",
            "warning",
            format!(
                "{} retained cfg/cfg_attr attribute(s) require feature or target matrix validation for full production confidence",
                counts.conditional_compilation_attrs
            ),
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
}

#[derive(Default)]
struct SyntacticHazardCounts {
    source_include_macros: usize,
    nonliteral_file_include_macros: usize,
    custom_attribute_macros: usize,
    custom_derive_macros: usize,
    custom_macro_invocations: usize,
    conditional_compilation_attrs: usize,
}

impl SyntacticHazardCounts {
    fn add(&mut self, other: Self) {
        self.source_include_macros += other.source_include_macros;
        self.nonliteral_file_include_macros += other.nonliteral_file_include_macros;
        self.custom_attribute_macros += other.custom_attribute_macros;
        self.custom_derive_macros += other.custom_derive_macros;
        self.custom_macro_invocations += other.custom_macro_invocations;
        self.conditional_compilation_attrs += other.conditional_compilation_attrs;
    }
}

fn syntactic_hazard_counts(project: &Project, reduced: &ReducedProject) -> SyntacticHazardCounts {
    let mut visitor = SyntacticHazardVisitor::default();

    for callable in &reduced.reachable {
        if let Some(record) = project.functions.get(callable) {
            visitor.visit_item_fn(&record.item);
        } else if let Some(record) = project.methods.get(callable) {
            visitor.visit_impl_item_fn(&record.item);
        }
    }

    for item in &reduced.reachable_items {
        if let Some(record) = project.items.get(item) {
            visitor.visit_item(&record.item);
        }
    }

    visitor.counts
}

#[derive(Default)]
struct SyntacticHazardVisitor {
    counts: SyntacticHazardCounts,
}

impl<'ast> Visit<'ast> for SyntacticHazardVisitor {
    fn visit_attribute(&mut self, attribute: &'ast Attribute) {
        if attribute.path().is_ident("cfg") || attribute.path().is_ident("cfg_attr") {
            self.counts.conditional_compilation_attrs += 1;
        }
        self.counts.custom_derive_macros += custom_derive_macro_count(attribute);
        if attribute_requires_macro_expansion(attribute) {
            self.counts.custom_attribute_macros += 1;
        }
        self.counts.add(cfg_attr_nested_macro_counts(attribute));

        syn::visit::visit_attribute(self, attribute);
    }

    fn visit_macro(&mut self, mac: &'ast Macro) {
        if macro_path_ends_with(mac, "include") {
            self.counts.source_include_macros += 1;
        } else if (macro_path_ends_with(mac, "include_str")
            || macro_path_ends_with(mac, "include_bytes"))
            && !macro_has_literal_path(mac)
        {
            self.counts.nonliteral_file_include_macros += 1;
        }
        if macro_invocation_requires_expansion_boundary(mac) {
            self.counts.custom_macro_invocations += 1;
        }

        syn::visit::visit_macro(self, mac);
    }
}

fn macro_path_ends_with(mac: &Macro, name: &str) -> bool {
    mac.path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == name)
}

fn macro_has_literal_path(mac: &Macro) -> bool {
    syn::parse2::<syn::LitStr>(mac.tokens.clone()).is_ok()
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
    ProductionHazardReport {
        code: code.to_string(),
        severity: severity.to_string(),
        message: message.into(),
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
}

impl GeneratedTargetReportJson {
    fn from_report(report: &GeneratedTargetReport) -> Self {
        Self {
            package: report.package.clone(),
            name: report.name.clone(),
            kind: report.kind.clone(),
            src_path: report.src_path.clone(),
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

    use super::{
        add_semantic_inventory_hazard, generate, write_generate_report, AnalyzerMode,
        AnalyzerReport, GenerateOptions, SemanticReport,
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
        };
        let mut hazards = Vec::new();

        add_semantic_inventory_hazard(&analyzer, &mut hazards);

        assert!(hazards
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
        write(
            root.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;

#[opensourced]
pub fn entry() -> &'static str {
    let _generated = include!("generated_expr.rs");
    include_str!(concat!("data", ".txt"))
}
"#,
        );

        let report = generate(GenerateOptions {
            workspace_root: root,
            output_root: temp_output("include-hazard-output"),
        })
        .expect("reduction should succeed");

        assert!(report
            .production
            .hazards
            .iter()
            .any(|hazard| hazard.code == "source_include_macros"));
        assert!(report
            .production
            .hazards
            .iter()
            .any(|hazard| hazard.code == "nonliteral_file_include_macros"));
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

        assert!(report
            .production
            .hazards
            .iter()
            .any(|hazard| hazard.code == "custom_macro_invocations"));
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

        assert!(report
            .production
            .hazards
            .iter()
            .any(|hazard| hazard.code == "retained_build_scripts"));
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

        assert!(report
            .production
            .hazards
            .iter()
            .any(|hazard| hazard.code == "custom_attribute_macros"));
        assert!(report
            .production
            .hazards
            .iter()
            .any(|hazard| hazard.code == "custom_derive_macros"));
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

        assert!(report
            .production
            .hazards
            .iter()
            .any(|hazard| hazard.code == "custom_attribute_macros"));
        assert!(report
            .production
            .hazards
            .iter()
            .any(|hazard| hazard.code == "custom_derive_macros"));
        assert!(report
            .production
            .hazards
            .iter()
            .any(|hazard| hazard.code == "conditional_compilation_attrs"));
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
