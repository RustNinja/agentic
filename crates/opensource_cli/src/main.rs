use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::{OsStr, OsString},
    fs,
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
    time::Duration,
};

use serde::Serialize;

use opensource_core::{
    check_workspace, generate_with_analyzer_feedback_and_roots, generate_with_analyzer_roots,
    marked_workspace_packages, preflight_workspace, repair_workspace, write_generate_report,
    write_preflight_report, write_repair_report, write_report, AnalyzerMode, CheckDiagnostic,
    CheckOptions, CheckReport, GenerateOptions, GenerateReport, GenerateSession,
    GeneratedTargetReport, PreflightDiagnostic, PreflightOptions, PreflightReport, RepairOptions,
    RepairReport, RootId, SemanticReport,
};

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = parse_args()?;

    if same_path(&options.workspace_root, &options.output_root) {
        return Err("output root must be different from workspace root".into());
    }

    apply_default_marked_package_scope(&mut options)?;

    if options.batch_roots {
        return run_batch_roots(&options);
    }

    let mut validation = ValidationReport::new(&options);

    let baseline = if options.run_baseline_check {
        let report = run_baseline_check(&options)?;
        let report_path = baseline_report_path(&options);
        if !report.success && !options.allow_baseline_failures {
            write_baseline_report(&options, &report)?;
            print_baseline(&report, options.feedback_limit, Some(&report_path));
            validation.add_check_gate(
                "baseline",
                "failed",
                "source workspace failed baseline cargo check",
                &report,
                Some(report_path),
                0,
            );
            finish_validation(
                &options,
                &mut validation,
                "rejected",
                Some("source workspace failed baseline cargo check"),
            )?;
            return Err("source workspace failed baseline cargo check".into());
        }
        print_baseline(&report, options.feedback_limit, None);
        validation.add_check_gate(
            "baseline",
            if report.success {
                "passed"
            } else {
                "baseline_allowed"
            },
            if report.success {
                "source workspace cargo check passed"
            } else {
                "source workspace baseline failed but --allow-baseline-failures is enabled"
            },
            &report,
            Some(report_path),
            0,
        );
        Some(report)
    } else {
        None
    };

    let report = generate_with_analyzer_roots(
        GenerateOptions {
            workspace_root: options.workspace_root.clone(),
            output_root: options.output_root.clone(),
        },
        options.analyzer_mode,
        &options.root_selectors,
    )?;

    println!(
        "analyzer: {} ({})",
        report.analyzer.mode.as_str(),
        report.analyzer.engine
    );
    for note in &report.analyzer.notes {
        println!("  analyzer note: {note}");
    }
    if let Some(semantic) = &report.analyzer.semantic {
        println!(
            "  analyzer semantic files: {}/{} analyzed ({} failed, {} skipped by budget)",
            semantic.analyzed_files,
            semantic.source_files,
            semantic.failed_files,
            semantic.skipped_files
        );
        println!(
            "  analyzer budgets: files={}, method_calls={}, paths={}",
            semantic.file_budget, semantic.method_call_budget, semantic.path_budget
        );
        if semantic.selected_root_source_files > 0 {
            println!(
                "  analyzer selected-root files: {}/{} analyzed ({} failed, {} skipped by budget)",
                semantic.selected_root_analyzed_files,
                semantic.selected_root_source_files,
                semantic.selected_root_failed_files,
                semantic.selected_root_skipped_files
            );
        }
        println!(
            "  analyzer method calls: {}/{} queried function-resolved, {}/{} callable, {}/{} fallback ({} unresolved, {} unqueried)",
            semantic.resolved_method_calls,
            semantic.queried_method_calls,
            semantic.callable_method_calls,
            semantic.queried_method_calls,
            semantic.fallback_method_calls,
            semantic.queried_method_calls,
            semantic.unresolved_method_calls,
            semantic.unqueried_method_calls
        );
        println!(
            "  analyzer paths: {}/{} queried resolved ({} unresolved, {} unqueried)",
            semantic.resolved_paths,
            semantic.queried_paths,
            semantic.unresolved_paths,
            semantic.unqueried_paths
        );
    }
    println!(
        "production readiness: {} ({} hazard(s))",
        report.production.status,
        report.production.hazards.len()
    );
    for hazard in report
        .production
        .hazards
        .iter()
        .take(options.feedback_limit)
    {
        println!(
            "  production {} {}: {}",
            hazard.severity, hazard.code, hazard.message
        );
    }
    if report.roots.len() == 1 {
        println!("root: {}", report.root);
    } else {
        println!("roots:");
        for root in &report.roots {
            println!("  {root}");
        }
    }
    println!("files written: {}", report.files_written);
    println!(
        "generation timing: total={}ms analyzer={}ms manifest={}ms parse={}ms reduce={}ms render={}ms",
        report.timings.total_ms,
        report.timings.analyzer_ms,
        report.timings.manifest_ms,
        report.timings.parse_ms,
        report.timings.reduce_ms,
        report.timings.render_ms
    );
    println!("packages: {}", report.packages.join(", "));
    print_rendered_symbol_summary(&report, "callable", "rendered callables");
    print_rendered_symbol_summary(&report, "item", "rendered items");
    let rendered_usage_contract = rendered_usage_contract(&report);
    println!(
        "rendered usage contract: used={} blocked_by_unknown={} invalid={}",
        rendered_usage_contract.used,
        rendered_usage_contract.blocked_by_unknown,
        rendered_usage_contract.invalid.len()
    );
    if let Some(report_path) = slice_report_path(&options) {
        write_generate_report(&report, &report_path)?;
        println!("slice report: {}", report_path.display());
    }
    validation.gates.push(ValidationGateReport {
        name: "generation".to_string(),
        status: "passed".to_string(),
        reason: "slice workspace was generated".to_string(),
        report_path: slice_report_path(&options),
        error_count: None,
        warning_count: None,
        semantic_warning_hazards: None,
        review_warning_hazards: None,
    });
    if rendered_usage_contract.invalid.is_empty() {
        validation.gates.push(ValidationGateReport {
            name: "rendered_usage_contract".to_string(),
            status: "passed".to_string(),
            reason: format!(
                "rendered source contains {} used and {} blocked_by_unknown symbols",
                rendered_usage_contract.used, rendered_usage_contract.blocked_by_unknown
            ),
            report_path: slice_report_path(&options),
            error_count: Some(0),
            warning_count: Some(rendered_usage_contract.blocked_by_unknown),
            semantic_warning_hazards: None,
            review_warning_hazards: None,
        });
    } else {
        let reason = format!(
            "rendered source contains invalid usage decisions: {}",
            rendered_usage_contract.invalid_preview()
        );
        validation.gates.push(ValidationGateReport {
            name: "rendered_usage_contract".to_string(),
            status: "failed".to_string(),
            reason: reason.clone(),
            report_path: slice_report_path(&options),
            error_count: Some(rendered_usage_contract.invalid.len()),
            warning_count: Some(rendered_usage_contract.blocked_by_unknown),
            semantic_warning_hazards: None,
            review_warning_hazards: None,
        });
        finish_validation_with_decision_log(
            &options,
            &mut validation,
            &report,
            "rejected",
            Some(&reason),
        )?;
        return Err(reason.into());
    }
    if let Some(reason) = record_semantic_proof_gate(&options, &mut validation, &report) {
        finish_validation_with_decision_log(
            &options,
            &mut validation,
            &report,
            "rejected",
            Some(&reason),
        )?;
        return Err(reason.into());
    }
    record_production_readiness_gate(
        &options,
        &mut validation,
        &report.production,
        "before compiler feedback",
    );
    if production_readiness_blocks_validation(&options, &report.production) {
        let reason =
            "production readiness reported error hazards before compiler feedback".to_string();
        finish_validation_with_decision_log(
            &options,
            &mut validation,
            &report,
            "rejected",
            Some(&reason),
        )?;
        return Err(reason.into());
    }
    let uncovered_targets =
        uncovered_validation_targets(&report.targets, &options.cargo_check_args);
    if !uncovered_targets.is_empty() {
        let reason = format!(
            "selected target(s) are not covered by cargo check args: {}",
            uncovered_targets.join(", ")
        );
        validation.gates.push(ValidationGateReport {
            name: "target_coverage".to_string(),
            status: "failed".to_string(),
            reason: reason.clone(),
            report_path: slice_report_path(&options),
            error_count: None,
            warning_count: None,
            semantic_warning_hazards: None,
            review_warning_hazards: None,
        });
        finish_validation_with_decision_log(
            &options,
            &mut validation,
            &report,
            "rejected",
            Some(&reason),
        )?;
        return Err(reason.into());
    }
    validation.gates.push(ValidationGateReport {
        name: "target_coverage".to_string(),
        status: "passed".to_string(),
        reason: "selected targets are covered by cargo check arguments".to_string(),
        report_path: slice_report_path(&options),
        error_count: None,
        warning_count: None,
        semantic_warning_hazards: None,
        review_warning_hazards: None,
    });
    if let Err(error) = refresh_generated_lockfile_for_locked_validation(&options, &mut validation)
    {
        let reason = error.to_string();
        finish_validation_with_decision_log(
            &options,
            &mut validation,
            &report,
            "rejected",
            Some(&reason),
        )?;
        return Err(reason.into());
    }
    if let Some(report) = &baseline {
        write_baseline_report(&options, report)?;
        println!(
            "baseline report: {}",
            baseline_report_path(&options).display()
        );
    }

    if options.run_preflight
        || options.feedback_iterations > 0
        || options.feedback_repair_iterations > 0
    {
        let preflight = run_preflight(&options)?;
        validation.gates.push(ValidationGateReport {
            name: "preflight".to_string(),
            status: if preflight.success {
                "passed"
            } else {
                "failed"
            }
            .to_string(),
            reason: if preflight.success {
                "generated workspace passed fast structural validation".to_string()
            } else {
                "generated workspace failed fast structural validation".to_string()
            },
            report_path: Some(preflight_report_path(&options)),
            error_count: Some(preflight.error_count()),
            warning_count: Some(preflight.warning_count()),
            semantic_warning_hazards: None,
            review_warning_hazards: None,
        });
        if !preflight.success {
            finish_validation_with_decision_log(
                &options,
                &mut validation,
                &report,
                "rejected",
                Some("generated workspace failed fast preflight validation"),
            )?;
            return Err("generated workspace failed fast preflight validation".into());
        }
    }

    if options.feedback_repair_iterations > 0 {
        if let Err(error) = run_feedback_repair_loop(&options, baseline.as_ref(), &mut validation) {
            let reason = error.to_string();
            finish_validation_with_decision_log(
                &options,
                &mut validation,
                &report,
                "rejected",
                Some(&reason),
            )?;
            return Err(reason.into());
        }
    } else if options.feedback_iterations > 0 {
        if let Err(error) = run_feedback_loop(&options, baseline.as_ref(), &mut validation) {
            let reason = error.to_string();
            finish_validation_with_decision_log(
                &options,
                &mut validation,
                &report,
                "rejected",
                Some(&reason),
            )?;
            return Err(reason.into());
        }
    } else if options.run_check {
        if let Err(error) = run_plain_check_gate(&options, &mut validation) {
            let reason = error.to_string();
            write_decision_log(
                &options,
                &report,
                Some(&validation),
                "rejected",
                Some(&reason),
            )?;
            return Err(error);
        }
    }

    if let Err(error) =
        run_production_validation_matrix(&options, &report.production, &mut validation)
    {
        let reason = error.to_string();
        finish_validation_with_decision_log(
            &options,
            &mut validation,
            &report,
            "rejected",
            Some(&reason),
        )?;
        return Err(reason.into());
    }

    record_final_production_readiness(&options, &mut validation);
    finish_validation_with_decision_log(&options, &mut validation, &report, "accepted", None)?;
    Ok(())
}

fn print_rendered_symbol_summary(report: &GenerateReport, kind: &str, label: &str) {
    println!("{label}:");
    let decisions = match kind {
        "callable" => Some(&report.usage.rendered_decision_map.callables),
        "item" => Some(&report.usage.rendered_decision_map.items),
        _ => None,
    };
    for entry in report
        .usage
        .rendered_symbols
        .entries
        .iter()
        .filter(|entry| entry.kind == kind)
    {
        let classification = decisions
            .and_then(|decisions| decisions.get(&entry.id))
            .unwrap_or(&entry.classification);
        println!("  {} [{}]", entry.id, classification);
    }
}

#[derive(Debug, Default)]
struct RenderedUsageContract {
    used: usize,
    blocked_by_unknown: usize,
    invalid: Vec<String>,
}

impl RenderedUsageContract {
    fn invalid_preview(&self) -> String {
        let mut preview = self.invalid.iter().take(5).cloned().collect::<Vec<_>>();
        if self.invalid.len() > preview.len() {
            preview.push(format!("... {} more", self.invalid.len() - preview.len()));
        }
        preview.join(", ")
    }
}

fn rendered_usage_contract(report: &GenerateReport) -> RenderedUsageContract {
    let mut contract = RenderedUsageContract::default();
    let decisions = &report.usage.rendered_decision_map;
    accumulate_rendered_usage_decisions("callable", &decisions.callables, &mut contract);
    accumulate_rendered_usage_decisions("item", &decisions.items, &mut contract);
    accumulate_rendered_usage_decisions("member", &decisions.members, &mut contract);
    accumulate_rendered_usage_decisions("assoc_item", &decisions.assoc_items, &mut contract);
    accumulate_rendered_usage_decisions(
        "trait_default_method",
        &decisions.trait_default_methods,
        &mut contract,
    );
    contract
}

fn accumulate_rendered_usage_decisions(
    kind: &str,
    decisions: &BTreeMap<String, String>,
    contract: &mut RenderedUsageContract,
) {
    for (id, decision) in decisions {
        match decision.as_str() {
            "used" => contract.used += 1,
            "blocked_by_unknown" => contract.blocked_by_unknown += 1,
            _ => contract.invalid.push(format!("{kind} {id} [{decision}]")),
        }
    }
}

struct CliOptions {
    analyzer_mode: AnalyzerMode,
    run_check: bool,
    feedback_iterations: usize,
    feedback_repair_iterations: usize,
    feedback_limit: usize,
    feedback_report: Option<PathBuf>,
    feedback_target_dir: Option<PathBuf>,
    feedback_timeout: Option<Duration>,
    cargo_check_args: Vec<String>,
    deny_warnings: bool,
    repair_report: Option<PathBuf>,
    run_baseline_check: bool,
    allow_baseline_failures: bool,
    baseline_report: Option<PathBuf>,
    baseline_target_dir: Option<PathBuf>,
    slice_report: Option<PathBuf>,
    decision_log: Option<PathBuf>,
    validation_report: Option<PathBuf>,
    run_preflight: bool,
    preflight_report: Option<PathBuf>,
    production_preset: bool,
    root_selectors: Vec<String>,
    random_roots: Option<usize>,
    random_root_packages: Vec<String>,
    random_seed: u64,
    batch_roots: bool,
    batch_report: Option<PathBuf>,
    workspace_root: PathBuf,
    output_root: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
struct ValidationReport {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    production_preset: bool,
    workspace_root: PathBuf,
    output_root: PathBuf,
    cargo_check_args: Vec<String>,
    deny_warnings: bool,
    allow_baseline_failures: bool,
    gates: Vec<ValidationGateReport>,
    attempts: Vec<ValidationAttemptReport>,
}

#[derive(Debug, Clone, Serialize)]
struct ValidationGateReport {
    name: String,
    status: String,
    reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    report_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    warning_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    semantic_warning_hazards: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    review_warning_hazards: Option<usize>,
}

#[derive(Debug, Serialize)]
struct BatchRootReport {
    root: String,
    output_root: PathBuf,
    status: String,
    files_written: Option<usize>,
    production_status: Option<String>,
    production_hazards: Option<usize>,
    rendered_usage_used: Option<usize>,
    rendered_usage_blocked_by_unknown: Option<usize>,
    rendered_usage_invalid: Option<usize>,
    semantic_proof_status: Option<String>,
    semantic_file_budget: Option<usize>,
    semantic_method_call_budget: Option<usize>,
    semantic_path_budget: Option<usize>,
    semantic_source_files: Option<usize>,
    semantic_analyzed_files: Option<usize>,
    semantic_failed_files: Option<usize>,
    semantic_skipped_files: Option<usize>,
    semantic_unresolved_method_calls: Option<usize>,
    semantic_unqueried_method_calls: Option<usize>,
    semantic_unresolved_paths: Option<usize>,
    semantic_unqueried_paths: Option<usize>,
    selected_root_semantic_source_files: Option<usize>,
    selected_root_semantic_analyzed_files: Option<usize>,
    selected_root_semantic_failed_files: Option<usize>,
    selected_root_semantic_skipped_files: Option<usize>,
    selected_root_unresolved_method_calls: Option<usize>,
    selected_root_unqueried_method_calls: Option<usize>,
    selected_root_unresolved_paths: Option<usize>,
    selected_root_unqueried_paths: Option<usize>,
    preflight_errors: Option<usize>,
    preflight_warnings: Option<usize>,
    check_success: Option<bool>,
    check_errors: Option<usize>,
    check_warnings: Option<usize>,
    duration_ms: u64,
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ValidationAttemptReport {
    stage: String,
    attempt: usize,
    status: String,
    reason: String,
    report_path: PathBuf,
    cargo_success: bool,
    baseline_limited: bool,
    timed_out: bool,
    error_count: usize,
    warning_count: usize,
    semantic_warning_hazards: usize,
    repairable_warnings: usize,
    widening_candidates: usize,
    widening_hazards: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    feedback_widened_roots: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    repair_report_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    repair_total_changes: Option<usize>,
}

#[derive(Default)]
struct FeedbackWideningState {
    diagnostics: Vec<CheckDiagnostic>,
    seen_root_sets: BTreeSet<String>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct ProductionMatrixEntry {
    name: String,
    reason: String,
    cargo_args: Vec<String>,
}

impl ValidationReport {
    fn new(options: &CliOptions) -> Self {
        Self {
            status: "running".to_string(),
            reason: None,
            production_preset: options.production_preset,
            workspace_root: options.workspace_root.clone(),
            output_root: options.output_root.clone(),
            cargo_check_args: options.cargo_check_args.clone(),
            deny_warnings: options.deny_warnings,
            allow_baseline_failures: options.allow_baseline_failures,
            gates: Vec::new(),
            attempts: Vec::new(),
        }
    }

    fn add_check_gate(
        &mut self,
        name: &str,
        status: &str,
        reason: &str,
        report: &CheckReport,
        report_path: Option<PathBuf>,
        semantic_warning_hazards: usize,
    ) {
        self.gates.push(ValidationGateReport {
            name: name.to_string(),
            status: status.to_string(),
            reason: reason.to_string(),
            report_path,
            error_count: Some(report.error_count()),
            warning_count: Some(report.warning_count()),
            semantic_warning_hazards: Some(semantic_warning_hazards),
            review_warning_hazards: None,
        });
    }
}

fn parse_args() -> Result<CliOptions, Box<dyn std::error::Error>> {
    parse_args_from(std::env::args_os().skip(1))
}

fn parse_args_from<I>(args: I) -> Result<CliOptions, Box<dyn std::error::Error>>
where
    I: IntoIterator<Item = OsString>,
{
    let mut analyzer_mode = AnalyzerMode::default_for_build();
    let mut analyzer_mode_explicit = false;
    let mut run_check = false;
    let mut feedback_iterations = 0;
    let mut feedback_repair_iterations = 0;
    let mut feedback_limit = 12;
    let mut feedback_report = None;
    let mut feedback_target_dir = None;
    let mut feedback_timeout = Some(Duration::from_secs(600));
    let mut cargo_check_args = Vec::new();
    let mut deny_warnings = false;
    let mut repair_report = None;
    let mut run_baseline_check = false;
    let mut allow_baseline_failures = false;
    let mut baseline_report = None;
    let mut baseline_target_dir = None;
    let mut slice_report = None;
    let mut decision_log = None;
    let mut validation_report = None;
    let mut run_preflight = false;
    let mut preflight_report = None;
    let mut production_preset = false;
    let mut root_selectors = Vec::new();
    let mut random_roots = None;
    let mut random_root_packages = Vec::new();
    let mut random_seed = 0;
    let mut batch_roots = false;
    let mut batch_report = None;
    let mut positional = Vec::new();
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        if arg == OsStr::new("--check") {
            run_check = true;
        } else if arg == OsStr::new("--production") {
            production_preset = true;
            run_baseline_check = true;
            run_preflight = true;
            feedback_repair_iterations = feedback_repair_iterations.max(3);
            deny_warnings = true;
        } else if arg == OsStr::new("--analyzer") {
            let value = args
                .next()
                .ok_or("--analyzer requires a following value: syn, ra-hir, ra-feedback, or ra-hir-proc-macros")?;
            let value = value
                .to_str()
                .ok_or("--analyzer value must be valid UTF-8")?;
            analyzer_mode = value.parse::<AnalyzerMode>()?;
            analyzer_mode_explicit = true;
        } else if arg == OsStr::new("--feedback") {
            feedback_iterations = feedback_iterations.max(1);
        } else if arg == OsStr::new("--preflight") {
            run_preflight = true;
        } else if arg == OsStr::new("--baseline-check") {
            run_baseline_check = true;
        } else if arg == OsStr::new("--allow-baseline-failures") {
            allow_baseline_failures = true;
            run_baseline_check = true;
        } else if arg == OsStr::new("--feedback-loop") {
            feedback_iterations = parse_usize_arg("--feedback-loop", args.next())?;
        } else if arg == OsStr::new("--feedback-repair-loop") {
            feedback_repair_iterations = parse_usize_arg("--feedback-repair-loop", args.next())?;
        } else if arg == OsStr::new("--feedback-limit") {
            feedback_limit = parse_usize_arg("--feedback-limit", args.next())?;
        } else if arg == OsStr::new("--feedback-timeout") {
            feedback_timeout = parse_feedback_timeout(args.next())?;
        } else if arg == OsStr::new("--cargo-check-arg") {
            let value = args
                .next()
                .ok_or("--cargo-check-arg requires a following cargo check argument")?;
            cargo_check_args.push(
                value
                    .to_str()
                    .ok_or("--cargo-check-arg value must be valid UTF-8")?
                    .to_string(),
            );
        } else if arg == OsStr::new("--deny-warnings") {
            deny_warnings = true;
        } else if arg == OsStr::new("--feedback-report") {
            feedback_report = Some(PathBuf::from(
                args.next()
                    .ok_or("--feedback-report requires a following path")?,
            ));
        } else if arg == OsStr::new("--feedback-target-dir") {
            feedback_target_dir = Some(PathBuf::from(
                args.next()
                    .ok_or("--feedback-target-dir requires a following path")?,
            ));
        } else if arg == OsStr::new("--repair-report") {
            repair_report = Some(PathBuf::from(
                args.next()
                    .ok_or("--repair-report requires a following path")?,
            ));
        } else if arg == OsStr::new("--baseline-report") {
            baseline_report = Some(PathBuf::from(
                args.next()
                    .ok_or("--baseline-report requires a following path")?,
            ));
            run_baseline_check = true;
        } else if arg == OsStr::new("--baseline-target-dir") {
            baseline_target_dir = Some(PathBuf::from(
                args.next()
                    .ok_or("--baseline-target-dir requires a following path")?,
            ));
            run_baseline_check = true;
        } else if arg == OsStr::new("--slice-report") {
            slice_report = Some(PathBuf::from(
                args.next()
                    .ok_or("--slice-report requires a following path")?,
            ));
        } else if arg == OsStr::new("--decision-log") {
            decision_log = Some(PathBuf::from(
                args.next()
                    .ok_or("--decision-log requires a following path")?,
            ));
        } else if arg == OsStr::new("--validation-report") {
            validation_report = Some(PathBuf::from(
                args.next()
                    .ok_or("--validation-report requires a following path")?,
            ));
        } else if arg == OsStr::new("--preflight-report") {
            preflight_report = Some(PathBuf::from(
                args.next()
                    .ok_or("--preflight-report requires a following path")?,
            ));
        } else if arg == OsStr::new("--root") {
            let value = args
                .next()
                .ok_or("--root requires a following function, method, or item selector")?;
            root_selectors.push(
                value
                    .to_str()
                    .ok_or("--root value must be valid UTF-8")?
                    .to_string(),
            );
        } else if arg == OsStr::new("--roots-file") {
            let path = PathBuf::from(
                args.next()
                    .ok_or("--roots-file requires a following path")?,
            );
            root_selectors.extend(read_root_selectors_file(&path)?);
        } else if arg == OsStr::new("--random-roots") {
            random_roots = Some(parse_usize_arg("--random-roots", args.next())?);
            batch_roots = true;
        } else if arg == OsStr::new("--random-root-package") {
            let value = args
                .next()
                .ok_or("--random-root-package requires a following package name")?;
            random_root_packages.push(
                value
                    .to_str()
                    .ok_or("--random-root-package value must be valid UTF-8")?
                    .to_string(),
            );
        } else if arg == OsStr::new("--random-seed") {
            random_seed = parse_u64_arg("--random-seed", args.next())?;
        } else if arg == OsStr::new("--batch-roots") {
            batch_roots = true;
        } else if arg == OsStr::new("--batch-report") {
            batch_report = Some(PathBuf::from(
                args.next()
                    .ok_or("--batch-report requires a following path")?,
            ));
        } else if arg == OsStr::new("--help") || arg == OsStr::new("-h") {
            println!("{}", usage());
            std::process::exit(0);
        } else {
            positional.push(PathBuf::from(&arg));
        }
    }

    let [workspace_root, output_root] = positional.as_slice() else {
        return Err(usage().into());
    };
    let workspace_root = normalize_workspace_root_arg(workspace_root);

    if production_preset && !analyzer_mode_explicit {
        analyzer_mode = AnalyzerMode::production_default_for_build();
    }

    if production_should_add_locked_arg(production_preset, &workspace_root, &cargo_check_args) {
        cargo_check_args.push("--locked".to_string());
    }

    Ok(CliOptions {
        analyzer_mode,
        run_check,
        feedback_iterations,
        feedback_repair_iterations,
        feedback_limit,
        feedback_report,
        feedback_target_dir,
        feedback_timeout,
        cargo_check_args,
        deny_warnings,
        repair_report,
        run_baseline_check,
        allow_baseline_failures,
        baseline_report,
        baseline_target_dir,
        slice_report,
        decision_log,
        validation_report,
        run_preflight,
        preflight_report,
        production_preset,
        root_selectors,
        random_roots,
        random_root_packages,
        random_seed,
        batch_roots,
        batch_report,
        workspace_root,
        output_root: output_root.clone(),
    })
}

fn normalize_workspace_root_arg(path: &Path) -> PathBuf {
    if path.file_name() != Some(OsStr::new("Cargo.toml")) {
        return path.to_path_buf();
    }
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}

fn apply_default_marked_package_scope(
    options: &mut CliOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    if !validation_runs_cargo_check(options)
        || cargo_args_have_package_scope(&options.cargo_check_args)
    {
        return Ok(());
    }

    let packages = marked_workspace_packages(&options.workspace_root)?;
    if packages.is_empty() {
        return Ok(());
    }

    println!(
        "validation package scope: {}",
        packages
            .iter()
            .map(|package| format!("-p {package}"))
            .collect::<Vec<_>>()
            .join(" ")
    );
    let mut scoped_args = package_scope_cargo_args(&packages);
    scoped_args.extend(options.cargo_check_args.clone());
    options.cargo_check_args = scoped_args;
    Ok(())
}

fn validation_runs_cargo_check(options: &CliOptions) -> bool {
    options.run_baseline_check
        || options.run_check
        || options.feedback_iterations > 0
        || options.feedback_repair_iterations > 0
}

fn package_scope_cargo_args(packages: &[String]) -> Vec<String> {
    packages
        .iter()
        .flat_map(|package| ["-p".to_string(), package.clone()])
        .collect()
}

fn cargo_args_have_package_scope(cargo_args: &[String]) -> bool {
    cargo_args.iter().any(|arg| {
        arg == "--workspace"
            || arg == "--all"
            || arg == "-p"
            || arg == "--package"
            || arg.starts_with("--package=")
            || (arg.starts_with("-p") && arg.len() > 2)
    })
}

fn production_should_add_locked_arg(
    production_preset: bool,
    workspace_root: &Path,
    cargo_check_args: &[String],
) -> bool {
    production_preset
        && workspace_root.join("Cargo.lock").exists()
        && !cargo_check_args
            .iter()
            .any(|arg| matches!(arg.as_str(), "--locked" | "--frozen"))
}

fn run_batch_roots(options: &CliOptions) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(&options.output_root)?;
    let report_path = batch_report_path(options);
    if let Some(parent) = report_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&report_path, "")?;

    let baseline = if options.run_baseline_check {
        let report = run_baseline_check(options)?;
        write_baseline_report(options, &report)?;
        print_baseline(
            &report,
            options.feedback_limit,
            Some(&baseline_report_path(options)),
        );
        if !report.success && !options.allow_baseline_failures {
            return Err("source workspace failed baseline cargo check".into());
        }
        Some(report)
    } else {
        None
    };

    let resolver_session = GenerateSession::load(&options.workspace_root, AnalyzerMode::Syn)?;
    let mut roots = resolver_session.resolve_root_selectors(&options.root_selectors)?;
    if let Some(count) = options.random_roots {
        roots.extend(select_random_roots(
            random_root_candidates(
                resolver_session.selectable_roots(),
                &options.random_root_packages,
            ),
            count,
            options.random_seed,
        ));
    }
    roots.sort();
    roots.dedup();
    if roots.is_empty() {
        return Err(
            "--batch-roots requires at least one --root, --roots-file entry, or --random-roots"
                .into(),
        );
    }

    let session = if options.analyzer_mode == AnalyzerMode::Syn {
        resolver_session
    } else {
        GenerateSession::load_with_selected_roots(
            &options.workspace_root,
            options.analyzer_mode,
            &roots,
        )?
    };
    println!(
        "analyzer: {} ({})",
        session.analyzer().mode.as_str(),
        session.analyzer().engine
    );
    for note in &session.analyzer().notes {
        println!("  analyzer note: {note}");
    }

    println!(
        "batch roots: {} slice(s); report: {}",
        roots.len(),
        report_path.display()
    );
    for root in &roots {
        println!("  batch root: {root}");
    }
    for (index, root) in roots.iter().enumerate() {
        let output_root = options.output_root.join(batch_output_dir_name(index, root));
        println!(
            "batch {}/{} start: {} -> {}",
            index + 1,
            roots.len(),
            root,
            output_root.display()
        );
        let started = std::time::Instant::now();
        let row = match run_batch_root(options, &session, root, &output_root, baseline.as_ref()) {
            Ok(mut row) => {
                row.duration_ms = elapsed_ms(started);
                row
            }
            Err(error) => BatchRootReport {
                root: root.to_string(),
                output_root,
                status: "failed".to_string(),
                files_written: None,
                production_status: None,
                production_hazards: None,
                rendered_usage_used: None,
                rendered_usage_blocked_by_unknown: None,
                rendered_usage_invalid: None,
                semantic_proof_status: None,
                semantic_file_budget: None,
                semantic_method_call_budget: None,
                semantic_path_budget: None,
                semantic_source_files: None,
                semantic_analyzed_files: None,
                semantic_failed_files: None,
                semantic_skipped_files: None,
                semantic_unresolved_method_calls: None,
                semantic_unqueried_method_calls: None,
                semantic_unresolved_paths: None,
                semantic_unqueried_paths: None,
                selected_root_semantic_source_files: None,
                selected_root_semantic_analyzed_files: None,
                selected_root_semantic_failed_files: None,
                selected_root_semantic_skipped_files: None,
                selected_root_unresolved_method_calls: None,
                selected_root_unqueried_method_calls: None,
                selected_root_unresolved_paths: None,
                selected_root_unqueried_paths: None,
                preflight_errors: None,
                preflight_warnings: None,
                check_success: None,
                check_errors: None,
                check_warnings: None,
                duration_ms: elapsed_ms(started),
                error: Some(error.to_string()),
            },
        };
        println!(
            "batch {}/{}: {} {}",
            index + 1,
            roots.len(),
            row.status,
            row.root
        );
        append_batch_report_row(&report_path, &row)?;
    }
    Ok(())
}

fn random_root_candidates(roots: Vec<RootId>, packages: &[String]) -> Vec<RootId> {
    if packages.is_empty() {
        return roots;
    }
    let packages = packages.iter().map(String::as_str).collect::<BTreeSet<_>>();
    roots
        .into_iter()
        .filter(|root| packages.contains(root.package()))
        .collect()
}

fn run_batch_root(
    options: &CliOptions,
    session: &GenerateSession,
    root: &RootId,
    output_root: &Path,
    baseline: Option<&CheckReport>,
) -> Result<BatchRootReport, Box<dyn std::error::Error>> {
    let mut diagnostics = Vec::<CheckDiagnostic>::new();
    let attempts = options
        .feedback_iterations
        .max(options.feedback_repair_iterations)
        .max(usize::from(options.run_check));
    let attempts = attempts.max(1);
    let mut last_report = None;
    let mut last_preflight = None;

    for attempt in 1..=attempts {
        let report = session.generate(output_root.to_path_buf(), &[root.clone()], &diagnostics)?;
        write_generate_report(&report, &output_root.join("slice-report.json"))?;
        let rendered_usage_contract = rendered_usage_contract(&report);
        let semantic_proof_block = options
            .production_preset
            .then(|| semantic_proof_block_reason(report.analyzer.semantic.as_ref()))
            .flatten();
        last_report = Some(report);
        if !rendered_usage_contract.invalid.is_empty() {
            return Ok(batch_row_from_reports(
                root,
                output_root,
                "rendered_usage_failed",
                last_report.as_ref(),
                last_preflight.as_ref(),
                None,
                Some(format!(
                    "rendered source contains invalid usage decisions: {}",
                    rendered_usage_contract.invalid_preview()
                )),
            ));
        }
        if let Some(reason) = semantic_proof_block {
            return Ok(batch_row_from_reports(
                root,
                output_root,
                "semantic_proof_failed",
                last_report.as_ref(),
                last_preflight.as_ref(),
                None,
                Some(reason),
            ));
        }
        if let Err(error) = refresh_generated_lockfile_for_output(options, output_root) {
            return Ok(batch_row_from_reports(
                root,
                output_root,
                "lockfile_failed",
                last_report.as_ref(),
                last_preflight.as_ref(),
                None,
                Some(error.to_string()),
            ));
        }

        if options.run_preflight || attempts > 1 {
            let preflight = preflight_workspace(PreflightOptions {
                manifest_path: output_root.join("Cargo.toml"),
            })?;
            write_preflight_report(&preflight, &output_root.join("slice-preflight.json"))?;
            if !preflight.success {
                let row = batch_row_from_reports(
                    root,
                    output_root,
                    "preflight_failed",
                    last_report.as_ref(),
                    Some(&preflight),
                    None,
                    None,
                );
                return Ok(row);
            }
            last_preflight = Some(preflight);
        }

        if !batch_runs_check(options) {
            return Ok(batch_row_from_reports(
                root,
                output_root,
                "generated",
                last_report.as_ref(),
                last_preflight.as_ref(),
                None,
                None,
            ));
        }

        let check = check_workspace(CheckOptions {
            manifest_path: output_root.join("Cargo.toml"),
            target_dir: Some(batch_feedback_target_dir(options)),
            timeout: options.feedback_timeout,
            cargo_args: batch_cargo_args(options, root),
        })?;
        write_report(&check, &output_root.join("slice-feedback.json"))?;
        let accepted = if options.feedback_repair_iterations > 0 {
            feedback_repair_is_accepted(&check, baseline, options.deny_warnings)
        } else {
            feedback_is_accepted(&check, baseline, options.deny_warnings)
        };
        if accepted {
            return Ok(batch_row_from_reports(
                root,
                output_root,
                "accepted",
                last_report.as_ref(),
                last_preflight.as_ref(),
                Some(&check),
                None,
            ));
        }
        if options.feedback_repair_iterations > 0 {
            let mut repaired_check = check.clone();
            let mut saw_deferred_warning_allows = false;
            for _repair_attempt in 1..=options.feedback_repair_iterations {
                let repair = repair_workspace(RepairOptions {
                    output_root: output_root.to_path_buf(),
                    diagnostics: repaired_check.diagnostics.clone(),
                })?;
                write_repair_report(&repair, &output_root.join("slice-repair.json"))?;
                saw_deferred_warning_allows |= repair_has_deferred_warning_allows(&repair);
                if repair.total_changes() == 0 {
                    break;
                }
                let preflight = preflight_workspace(PreflightOptions {
                    manifest_path: output_root.join("Cargo.toml"),
                })?;
                write_preflight_report(&preflight, &output_root.join("slice-preflight.json"))?;
                if !preflight.success {
                    return Ok(batch_row_from_reports(
                        root,
                        output_root,
                        "preflight_failed",
                        last_report.as_ref(),
                        Some(&preflight),
                        Some(&check),
                        Some("batch repair produced a structurally invalid workspace".to_string()),
                    ));
                }
                last_preflight = Some(preflight);
                repaired_check = check_workspace(CheckOptions {
                    manifest_path: output_root.join("Cargo.toml"),
                    target_dir: Some(batch_feedback_target_dir(options)),
                    timeout: options.feedback_timeout,
                    cargo_args: batch_cargo_args(options, root),
                })?;
                write_report(&repaired_check, &output_root.join("slice-feedback.json"))?;
                if feedback_repair_is_accepted(&repaired_check, baseline, options.deny_warnings) {
                    return Ok(batch_row_from_reports(
                        root,
                        output_root,
                        "accepted",
                        last_report.as_ref(),
                        last_preflight.as_ref(),
                        Some(&repaired_check),
                        None,
                    ));
                }
            }
            if should_run_deferred_warning_repair(
                &repaired_check,
                baseline,
                options.deny_warnings,
                saw_deferred_warning_allows,
            ) {
                let repair = repair_workspace(RepairOptions {
                    output_root: output_root.to_path_buf(),
                    diagnostics: repaired_check.diagnostics.clone(),
                })?;
                write_repair_report(&repair, &output_root.join("slice-repair.json"))?;
                if repair.total_changes() > 0 {
                    let preflight = preflight_workspace(PreflightOptions {
                        manifest_path: output_root.join("Cargo.toml"),
                    })?;
                    write_preflight_report(&preflight, &output_root.join("slice-preflight.json"))?;
                    if !preflight.success {
                        return Ok(batch_row_from_reports(
                            root,
                            output_root,
                            "preflight_failed",
                            last_report.as_ref(),
                            Some(&preflight),
                            Some(&repaired_check),
                            Some(
                                "batch warning repair produced a structurally invalid workspace"
                                    .to_string(),
                            ),
                        ));
                    }
                    last_preflight = Some(preflight);
                    repaired_check = check_workspace(CheckOptions {
                        manifest_path: output_root.join("Cargo.toml"),
                        target_dir: Some(batch_feedback_target_dir(options)),
                        timeout: options.feedback_timeout,
                        cargo_args: batch_cargo_args(options, root),
                    })?;
                    write_report(&repaired_check, &output_root.join("slice-feedback.json"))?;
                    if feedback_repair_is_accepted(&repaired_check, baseline, options.deny_warnings)
                    {
                        return Ok(batch_row_from_reports(
                            root,
                            output_root,
                            "accepted",
                            last_report.as_ref(),
                            last_preflight.as_ref(),
                            Some(&repaired_check),
                            None,
                        ));
                    }
                }
            }
            if attempt == attempts || repaired_check.error_count() == 0 {
                return Ok(batch_row_from_reports(
                    root,
                    output_root,
                    "check_failed",
                    last_report.as_ref(),
                    last_preflight.as_ref(),
                    Some(&repaired_check),
                    Some("repaired workspace did not pass batch feedback gate".to_string()),
                ));
            }
            diagnostics.extend(repaired_check.diagnostics);
            continue;
        }
        if attempt == attempts || check.error_count() == 0 {
            return Ok(batch_row_from_reports(
                root,
                output_root,
                "check_failed",
                last_report.as_ref(),
                last_preflight.as_ref(),
                Some(&check),
                Some("generated workspace did not pass batch feedback gate".to_string()),
            ));
        }
        diagnostics.extend(check.diagnostics);
    }

    Ok(batch_row_from_reports(
        root,
        output_root,
        "failed",
        last_report.as_ref(),
        last_preflight.as_ref(),
        None,
        Some("batch loop ended without a final report".to_string()),
    ))
}

fn batch_row_from_reports(
    root: &RootId,
    output_root: &Path,
    status: &str,
    report: Option<&GenerateReport>,
    preflight: Option<&PreflightReport>,
    check: Option<&CheckReport>,
    error: Option<String>,
) -> BatchRootReport {
    let rendered_usage = report.map(rendered_usage_contract);
    let semantic = report.and_then(|report| report.analyzer.semantic.as_ref());
    BatchRootReport {
        root: root.to_string(),
        output_root: output_root.to_path_buf(),
        status: status.to_string(),
        files_written: report.map(|report| report.files_written),
        production_status: report.map(|report| report.production.status.clone()),
        production_hazards: report.map(|report| report.production.hazards.len()),
        rendered_usage_used: rendered_usage.as_ref().map(|contract| contract.used),
        rendered_usage_blocked_by_unknown: rendered_usage
            .as_ref()
            .map(|contract| contract.blocked_by_unknown),
        rendered_usage_invalid: rendered_usage
            .as_ref()
            .map(|contract| contract.invalid.len()),
        semantic_proof_status: semantic.map(semantic_proof_status),
        semantic_file_budget: semantic.map(|semantic| semantic.file_budget),
        semantic_method_call_budget: semantic.map(|semantic| semantic.method_call_budget),
        semantic_path_budget: semantic.map(|semantic| semantic.path_budget),
        semantic_source_files: semantic.map(|semantic| semantic.source_files),
        semantic_analyzed_files: semantic.map(|semantic| semantic.analyzed_files),
        semantic_failed_files: semantic.map(|semantic| semantic.failed_files),
        semantic_skipped_files: semantic.map(|semantic| semantic.skipped_files),
        semantic_unresolved_method_calls: semantic.map(|semantic| semantic.unresolved_method_calls),
        semantic_unqueried_method_calls: semantic.map(|semantic| semantic.unqueried_method_calls),
        semantic_unresolved_paths: semantic.map(|semantic| semantic.unresolved_paths),
        semantic_unqueried_paths: semantic.map(|semantic| semantic.unqueried_paths),
        selected_root_semantic_source_files: semantic
            .map(|semantic| semantic.selected_root_source_files),
        selected_root_semantic_analyzed_files: semantic
            .map(|semantic| semantic.selected_root_analyzed_files),
        selected_root_semantic_failed_files: semantic
            .map(|semantic| semantic.selected_root_failed_files),
        selected_root_semantic_skipped_files: semantic
            .map(|semantic| semantic.selected_root_skipped_files),
        selected_root_unresolved_method_calls: semantic
            .map(|semantic| semantic.selected_root_unresolved_method_calls),
        selected_root_unqueried_method_calls: semantic
            .map(|semantic| semantic.selected_root_unqueried_method_calls),
        selected_root_unresolved_paths: semantic
            .map(|semantic| semantic.selected_root_unresolved_paths),
        selected_root_unqueried_paths: semantic
            .map(|semantic| semantic.selected_root_unqueried_paths),
        preflight_errors: preflight.map(PreflightReport::error_count),
        preflight_warnings: preflight.map(PreflightReport::warning_count),
        check_success: check.map(|check| check.success),
        check_errors: check.map(CheckReport::error_count),
        check_warnings: check.map(CheckReport::warning_count),
        duration_ms: 0,
        error,
    }
}

fn semantic_proof_status(semantic: &SemanticReport) -> String {
    let workspace_limited = semantic.skipped_files > 0
        || semantic.unqueried_method_calls > 0
        || semantic.unqueried_paths > 0;

    if semantic.selected_root_source_files > 0 {
        if semantic.selected_root_failed_files > 0 {
            return "selected_root_failed".to_string();
        }
        if semantic.selected_root_skipped_files > 0
            || semantic.selected_root_unqueried_method_calls > 0
            || semantic.selected_root_unqueried_paths > 0
        {
            return "selected_root_limited".to_string();
        }
        if workspace_limited {
            return "selected_root_complete_workspace_limited".to_string();
        }
        return "complete".to_string();
    }

    if semantic.failed_files > 0 {
        return "workspace_failed".to_string();
    }
    if workspace_limited {
        return "workspace_limited".to_string();
    }
    if semantic.source_files == 0 {
        return "empty".to_string();
    }
    "complete".to_string()
}

fn record_semantic_proof_gate(
    options: &CliOptions,
    validation: &mut ValidationReport,
    report: &GenerateReport,
) -> Option<String> {
    let semantic = report.analyzer.semantic.as_ref();
    let status = semantic
        .map(semantic_proof_status)
        .unwrap_or_else(|| "not_available".to_string());
    let reason = semantic_proof_reason(semantic);
    let blocks = options.production_preset && semantic_proof_status_blocks(&status);
    validation.gates.push(ValidationGateReport {
        name: "semantic_proof".to_string(),
        status: if blocks {
            "failed".to_string()
        } else {
            status.clone()
        },
        reason: reason.clone(),
        report_path: slice_report_path(options),
        error_count: semantic.map(semantic_proof_error_count),
        warning_count: semantic.map(semantic_proof_warning_count),
        semantic_warning_hazards: None,
        review_warning_hazards: None,
    });
    blocks.then_some(reason)
}

fn semantic_proof_block_reason(semantic: Option<&SemanticReport>) -> Option<String> {
    let status = semantic
        .map(semantic_proof_status)
        .unwrap_or_else(|| "not_available".to_string());
    semantic_proof_status_blocks(&status).then(|| semantic_proof_reason(semantic))
}

fn semantic_proof_status_blocks(status: &str) -> bool {
    matches!(
        status,
        "not_available"
            | "empty"
            | "selected_root_failed"
            | "selected_root_limited"
            | "workspace_failed"
            | "workspace_limited"
    )
}

fn semantic_proof_reason(semantic: Option<&SemanticReport>) -> String {
    let Some(semantic) = semantic else {
        return "rust-analyzer semantic proof is not available".to_string();
    };
    match semantic_proof_status(semantic).as_str() {
        "complete" => "rust-analyzer semantic proof covered the selected slice".to_string(),
        "selected_root_complete_workspace_limited" => format!(
            "selected root semantic proof is complete; wider workspace budget remains limited ({} skipped file(s), {} unqueried method call(s), {} unqueried path(s))",
            semantic.skipped_files, semantic.unqueried_method_calls, semantic.unqueried_paths
        ),
        "selected_root_failed" => format!(
            "selected root semantic proof failed in {} root file(s)",
            semantic.selected_root_failed_files
        ),
        "selected_root_limited" => format!(
            "selected root semantic proof is budget-limited ({} skipped root file(s), {} unqueried root method call(s), {} unqueried root path(s))",
            semantic.selected_root_skipped_files,
            semantic.selected_root_unqueried_method_calls,
            semantic.selected_root_unqueried_paths
        ),
        "workspace_failed" => format!(
            "workspace semantic proof failed in {} file(s)",
            semantic.failed_files
        ),
        "workspace_limited" => format!(
            "workspace semantic proof is budget-limited ({} skipped file(s), {} unqueried method call(s), {} unqueried path(s))",
            semantic.skipped_files, semantic.unqueried_method_calls, semantic.unqueried_paths
        ),
        "empty" => "rust-analyzer semantic proof did not find Rust source files".to_string(),
        _ => "rust-analyzer semantic proof state is unknown".to_string(),
    }
}

fn semantic_proof_error_count(semantic: &SemanticReport) -> usize {
    if semantic.selected_root_source_files > 0 {
        semantic.selected_root_failed_files
            + semantic.selected_root_skipped_files
            + semantic.selected_root_unqueried_method_calls
            + semantic.selected_root_unqueried_paths
    } else {
        semantic.failed_files
            + semantic.skipped_files
            + semantic.unqueried_method_calls
            + semantic.unqueried_paths
    }
}

fn semantic_proof_warning_count(semantic: &SemanticReport) -> usize {
    if semantic.selected_root_source_files > 0 {
        semantic.skipped_files + semantic.unqueried_method_calls + semantic.unqueried_paths
    } else {
        semantic.unresolved_method_calls + semantic.unresolved_paths
    }
}

fn batch_runs_check(options: &CliOptions) -> bool {
    options.run_check || options.feedback_iterations > 0 || options.feedback_repair_iterations > 0
}

fn batch_feedback_target_dir(options: &CliOptions) -> PathBuf {
    options
        .feedback_target_dir
        .clone()
        .unwrap_or_else(|| options.output_root.join("target-feedback"))
}

fn batch_cargo_args(options: &CliOptions, root: &RootId) -> Vec<String> {
    if cargo_args_have_package_scope(&options.cargo_check_args) {
        return options.cargo_check_args.clone();
    }
    let mut args = vec!["-p".to_string(), root.package().to_string()];
    args.extend(options.cargo_check_args.clone());
    args
}

fn batch_report_path(options: &CliOptions) -> PathBuf {
    options
        .batch_report
        .clone()
        .unwrap_or_else(|| options.output_root.join("batch-report.jsonl"))
}

fn append_batch_report_row(
    report_path: &Path,
    row: &BatchRootReport,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(report_path)?;
    serde_json::to_writer(&mut file, row)?;
    file.write_all(b"\n")?;
    Ok(())
}

fn batch_output_dir_name(index: usize, root: &RootId) -> String {
    let mut slug = root
        .to_string()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();
    while slug.contains("__") {
        slug = slug.replace("__", "_");
    }
    let slug = slug.trim_matches('_');
    let slug = if slug.is_empty() { "root" } else { slug };
    let max_len = slug.len().min(96);
    format!("{:04}-{}", index + 1, &slug[..max_len])
}

fn select_random_roots(mut roots: Vec<RootId>, count: usize, seed: u64) -> Vec<RootId> {
    roots.sort_by_key(|root| stable_root_score(root, seed));
    roots.truncate(count.min(roots.len()));
    roots
}

fn stable_root_score(root: &RootId, seed: u64) -> u64 {
    let mut hash = seed ^ 0xcbf2_9ce4_8422_2325;
    for byte in root.to_string().bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x1000_0000_01b3);
    }
    hash
}

fn elapsed_ms(started: std::time::Instant) -> u64 {
    started.elapsed().as_millis().try_into().unwrap_or(u64::MAX)
}

fn parse_usize_arg(
    flag: &str,
    value: Option<std::ffi::OsString>,
) -> Result<usize, Box<dyn std::error::Error>> {
    let value = value.ok_or_else(|| format!("{flag} requires a following number"))?;
    let value = value
        .to_str()
        .ok_or_else(|| format!("{flag} value must be valid UTF-8"))?;
    let parsed = value.parse::<usize>()?;
    if parsed == 0 {
        return Err(format!("{flag} must be greater than zero").into());
    }
    Ok(parsed)
}

fn parse_u64_arg(
    flag: &str,
    value: Option<std::ffi::OsString>,
) -> Result<u64, Box<dyn std::error::Error>> {
    let value = value.ok_or_else(|| format!("{flag} requires a following number"))?;
    let value = value
        .to_str()
        .ok_or_else(|| format!("{flag} value must be valid UTF-8"))?;
    Ok(value.parse::<u64>()?)
}

fn read_root_selectors_file(path: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(path)?;
    Ok(contents
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_string)
        .collect())
}

fn parse_feedback_timeout(
    value: Option<std::ffi::OsString>,
) -> Result<Option<Duration>, Box<dyn std::error::Error>> {
    let value = value.ok_or("--feedback-timeout requires a following number of seconds")?;
    let value = value
        .to_str()
        .ok_or("--feedback-timeout value must be valid UTF-8")?;
    let seconds = value.parse::<u64>()?;
    Ok((seconds > 0).then(|| Duration::from_secs(seconds)))
}

fn run_plain_check(options: &CliOptions) -> Result<(), Box<dyn std::error::Error>> {
    let manifest_path = absolute_path(&options.output_root.join("Cargo.toml"))?;
    let working_dir = manifest_working_dir(&manifest_path);
    let status = Command::new("cargo")
        .arg("check")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .args(&options.cargo_check_args)
        .current_dir(&working_dir)
        .status()?;
    if !status.success() {
        return Err(format!("generated workspace failed cargo check with {status}").into());
    }
    Ok(())
}

fn run_plain_check_gate(
    options: &CliOptions,
    validation: &mut ValidationReport,
) -> Result<(), Box<dyn std::error::Error>> {
    match run_plain_check(options) {
        Ok(()) => {
            validation.gates.push(ValidationGateReport {
                name: "check".to_string(),
                status: "passed".to_string(),
                reason: "generated workspace cargo check passed".to_string(),
                report_path: None,
                error_count: None,
                warning_count: None,
                semantic_warning_hazards: None,
                review_warning_hazards: None,
            });
            Ok(())
        }
        Err(error) => {
            let reason = error.to_string();
            validation.gates.push(ValidationGateReport {
                name: "check".to_string(),
                status: "failed".to_string(),
                reason: reason.clone(),
                report_path: None,
                error_count: None,
                warning_count: None,
                semantic_warning_hazards: None,
                review_warning_hazards: None,
            });
            finish_validation(options, validation, "rejected", Some(&reason))?;
            Err(reason.into())
        }
    }
}

fn absolute_path(path: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    Ok(std::env::current_dir()?.join(path))
}

fn manifest_working_dir(manifest_path: &Path) -> PathBuf {
    manifest_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}

fn run_preflight(options: &CliOptions) -> Result<PreflightReport, Box<dyn std::error::Error>> {
    let report_path = preflight_report_path(options);
    let report = preflight_workspace(PreflightOptions {
        manifest_path: options.output_root.join("Cargo.toml"),
    })?;
    write_preflight_report(&report, &report_path)?;
    print_preflight(&report, options.feedback_limit, &report_path);
    Ok(report)
}

fn run_baseline_check(options: &CliOptions) -> Result<CheckReport, Box<dyn std::error::Error>> {
    let target_dir = baseline_target_dir(options);
    println!(
        "baseline: cargo check --message-format=json (target {})",
        target_dir.display()
    );
    run_baseline_check_with_args(options, options.cargo_check_args.clone(), target_dir)
}

fn run_baseline_check_with_args(
    options: &CliOptions,
    cargo_args: Vec<String>,
    target_dir: PathBuf,
) -> Result<CheckReport, Box<dyn std::error::Error>> {
    check_workspace(CheckOptions {
        manifest_path: options.workspace_root.join("Cargo.toml"),
        target_dir: Some(target_dir),
        timeout: options.feedback_timeout,
        cargo_args,
    })
}

fn write_baseline_report(
    options: &CliOptions,
    report: &CheckReport,
) -> Result<(), Box<dyn std::error::Error>> {
    write_report(report, &baseline_report_path(options))
}

fn baseline_report_path(options: &CliOptions) -> PathBuf {
    options
        .baseline_report
        .clone()
        .unwrap_or_else(|| options.output_root.join("slice-baseline.json"))
}

fn slice_report_path(options: &CliOptions) -> Option<PathBuf> {
    options.slice_report.clone().or_else(|| {
        options
            .production_preset
            .then(|| options.output_root.join("slice-report.json"))
    })
}

fn decision_log_path(options: &CliOptions) -> Option<PathBuf> {
    options.decision_log.clone().or_else(|| {
        options
            .production_preset
            .then(|| options.output_root.join("slice-decision-log.json"))
    })
}

fn preflight_report_path(options: &CliOptions) -> PathBuf {
    options
        .preflight_report
        .clone()
        .unwrap_or_else(|| options.output_root.join("slice-preflight.json"))
}

fn maybe_preflight_report_path(options: &CliOptions) -> Option<PathBuf> {
    (options.run_preflight
        || options.feedback_iterations > 0
        || options.feedback_repair_iterations > 0)
        .then(|| preflight_report_path(options))
}

fn feedback_report_path(options: &CliOptions) -> Option<PathBuf> {
    (options.feedback_iterations > 0 || options.feedback_repair_iterations > 0).then(|| {
        options
            .feedback_report
            .clone()
            .unwrap_or_else(|| options.output_root.join("slice-feedback.json"))
    })
}

fn validation_report_path(options: &CliOptions) -> Option<PathBuf> {
    options.validation_report.clone().or_else(|| {
        (options.production_preset
            || options.run_baseline_check
            || options.run_preflight
            || options.feedback_iterations > 0
            || options.feedback_repair_iterations > 0
            || options.run_check)
            .then(|| options.output_root.join("slice-validation.json"))
    })
}

fn uncovered_validation_targets(
    targets: &[GeneratedTargetReport],
    cargo_args: &[String],
) -> Vec<String> {
    targets
        .iter()
        .filter_map(|target| {
            let coverage_kind = validation_target_kind(target);
            if let Some(target_kind) = coverage_kind {
                if !validation_target_is_covered(target_kind, &target.name, cargo_args) {
                    return Some(format!(
                        "{} {} {}",
                        target.package, target_kind, target.name
                    ));
                }
            }
            let missing_features = uncovered_target_required_features(target, cargo_args);
            let feature_kind = coverage_kind.or_else(|| required_feature_target_kind(target));
            (!missing_features.is_empty()).then(|| {
                let target_kind = feature_kind.unwrap_or("target");
                format!(
                    "{} {} {} requires feature(s): {}",
                    target.package,
                    target_kind,
                    target.name,
                    missing_features.join(", ")
                )
            })
        })
        .collect()
}

fn validation_target_kind(target: &GeneratedTargetReport) -> Option<&'static str> {
    ["example", "test", "bench"]
        .into_iter()
        .find(|kind| target.kind.iter().any(|target_kind| target_kind == *kind))
}

fn required_feature_target_kind(target: &GeneratedTargetReport) -> Option<&'static str> {
    ["bin", "example", "test", "bench"]
        .into_iter()
        .find(|kind| target.kind.iter().any(|target_kind| target_kind == *kind))
}

fn validation_target_is_covered(kind: &str, name: &str, cargo_args: &[String]) -> bool {
    if cargo_args.iter().any(|arg| arg == "--all-targets") {
        return true;
    }
    if cargo_args
        .iter()
        .any(|arg| arg == target_kind_plural_flag(kind))
    {
        return true;
    }
    let singular_flag = format!("--{kind}");
    for index in 0..cargo_args.len() {
        let arg = &cargo_args[index];
        if arg == &singular_flag && cargo_args.get(index + 1).is_some_and(|value| value == name) {
            return true;
        }
        if arg == &format!("{singular_flag}={name}") {
            return true;
        }
    }
    false
}

fn target_kind_plural_flag(kind: &str) -> &'static str {
    match kind {
        "bin" => "--bins",
        "example" => "--examples",
        "test" => "--tests",
        "bench" => "--benches",
        _ => "--all-targets",
    }
}

fn refresh_generated_lockfile_for_locked_validation(
    options: &CliOptions,
    validation: &mut ValidationReport,
) -> Result<(), Box<dyn std::error::Error>> {
    match refresh_generated_lockfile_for_output(options, &options.output_root) {
        Ok(Some(lockfile_path)) => {
            println!("lockfile: generated Cargo.lock reconciled before locked validation");
            validation.gates.push(ValidationGateReport {
                name: "lockfile".to_string(),
                status: "passed".to_string(),
                reason: "generated Cargo.lock was reconciled before locked validation".to_string(),
                report_path: Some(lockfile_path),
                error_count: None,
                warning_count: None,
                semantic_warning_hazards: None,
                review_warning_hazards: None,
            });
            Ok(())
        }
        Ok(None) => Ok(()),
        Err(error) => {
            let reason = error.to_string();
            validation.gates.push(ValidationGateReport {
                name: "lockfile".to_string(),
                status: "failed".to_string(),
                reason: reason.clone(),
                report_path: Some(options.output_root.join("Cargo.lock")),
                error_count: None,
                warning_count: None,
                semantic_warning_hazards: None,
                review_warning_hazards: None,
            });
            Err(reason.into())
        }
    }
}

fn refresh_generated_lockfile_for_output(
    options: &CliOptions,
    output_root: &Path,
) -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
    if !locked_validation_requested(&options.cargo_check_args)
        || !options.workspace_root.join("Cargo.lock").exists()
    {
        return Ok(None);
    }

    let lockfile_path = output_root.join("Cargo.lock");
    if !lockfile_path.exists() {
        return Err("generated Cargo.lock is missing before locked validation".into());
    }

    let manifest_path = absolute_path(&output_root.join("Cargo.toml"))?;
    let working_dir = manifest_working_dir(&manifest_path);
    let offline_requested = options
        .cargo_check_args
        .iter()
        .any(|arg| matches!(arg.as_str(), "--offline" | "--frozen"));
    let mut output = run_generate_lockfile(&manifest_path, &working_dir, true)?;
    if !output.status.success() && !offline_requested {
        output = run_generate_lockfile(&manifest_path, &working_dir, false)?;
    }
    if !output.status.success() {
        return Err(format!(
            "generated Cargo.lock could not be reconciled before locked validation\n{}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    Ok(Some(lockfile_path))
}

fn run_generate_lockfile(
    manifest_path: &Path,
    working_dir: &Path,
    offline: bool,
) -> Result<std::process::Output, Box<dyn std::error::Error>> {
    let mut command = Command::new("cargo");
    command
        .arg("generate-lockfile")
        .arg("--manifest-path")
        .arg(manifest_path)
        .current_dir(working_dir);
    if offline {
        command.arg("--offline");
    }
    Ok(command.output()?)
}

fn locked_validation_requested(cargo_check_args: &[String]) -> bool {
    cargo_check_args
        .iter()
        .any(|arg| matches!(arg.as_str(), "--locked" | "--frozen"))
}

fn uncovered_target_required_features(
    target: &GeneratedTargetReport,
    cargo_args: &[String],
) -> Vec<String> {
    if target.required_features.is_empty() || cargo_args.iter().any(|arg| arg == "--all-features") {
        return Vec::new();
    }

    let explicitly_enabled = enabled_cargo_features(cargo_args);
    let default_enabled = !cargo_args.iter().any(|arg| arg == "--no-default-features");
    target
        .required_features
        .iter()
        .filter(|feature| {
            !feature_is_enabled_for_target(
                feature,
                &target.package,
                &explicitly_enabled,
                default_enabled.then_some(target.default_features.as_slice()),
            )
        })
        .cloned()
        .collect()
}

fn enabled_cargo_features(cargo_args: &[String]) -> BTreeSet<String> {
    let mut features = BTreeSet::new();
    let mut index = 0;
    while index < cargo_args.len() {
        let arg = &cargo_args[index];
        if arg == "--features" {
            if let Some(value) = cargo_args.get(index + 1) {
                collect_feature_arg(value, &mut features);
                index += 2;
                continue;
            }
        } else if let Some(value) = arg.strip_prefix("--features=") {
            collect_feature_arg(value, &mut features);
        }
        index += 1;
    }
    features
}

fn collect_feature_arg(value: &str, features: &mut BTreeSet<String>) {
    for feature in value.split([',', ' ']).map(str::trim) {
        if !feature.is_empty() {
            features.insert(feature.to_string());
        }
    }
}

fn feature_is_enabled_for_target(
    feature: &str,
    package: &str,
    explicitly_enabled: &BTreeSet<String>,
    default_features: Option<&[String]>,
) -> bool {
    explicitly_enabled.contains(feature)
        || explicitly_enabled.contains(&format!("{package}/{feature}"))
        || default_features
            .is_some_and(|features| features.iter().any(|enabled| enabled == feature))
}

fn finish_validation(
    options: &CliOptions,
    report: &mut ValidationReport,
    status: &str,
    reason: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    report.status = status.to_string();
    report.reason = reason.map(str::to_string);
    if let Some(path) = validation_report_path(options) {
        write_validation_report(report, &path)?;
        println!("validation report: {}", path.display());
    }
    Ok(())
}

fn write_validation_report(
    report: &ValidationReport,
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(report)?)?;
    Ok(())
}

fn finish_validation_with_decision_log(
    options: &CliOptions,
    validation: &mut ValidationReport,
    report: &GenerateReport,
    status: &str,
    reason: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    finish_validation(options, validation, status, reason)?;
    write_decision_log(options, report, Some(validation), status, reason)
}

#[derive(Debug, Serialize)]
struct DecisionLogReport {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    workspace_root: PathBuf,
    output_root: PathBuf,
    analyzer_mode: String,
    production_preset: bool,
    run_check: bool,
    run_preflight: bool,
    feedback_iterations: usize,
    feedback_repair_iterations: usize,
    deny_warnings: bool,
    root_selectors: Vec<String>,
    cargo_check_args: Vec<String>,
    reports: DecisionLogReportPaths,
    #[serde(skip_serializing_if = "Option::is_none")]
    validation: Option<DecisionLogValidationSummary>,
    steps: Vec<DecisionLogStep>,
}

#[derive(Debug, Serialize)]
struct DecisionLogReportPaths {
    decision_log: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    slice_report: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    validation_report: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    baseline_report: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    baseline_target_dir: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    preflight_report: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    feedback_report: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    feedback_target_dir: Option<PathBuf>,
}

#[derive(Debug, Serialize)]
struct DecisionLogValidationSummary {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    gates: usize,
    attempts: usize,
    failed_gates: Vec<String>,
}

#[derive(Debug, Serialize)]
struct DecisionLogStep {
    step: String,
    status: String,
    decision: String,
    reason: String,
    metrics: BTreeMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    evidence: Vec<String>,
}

fn write_decision_log(
    options: &CliOptions,
    report: &GenerateReport,
    validation: Option<&ValidationReport>,
    status: &str,
    reason: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(path) = decision_log_path(options) else {
        return Ok(());
    };
    let log = build_decision_log(options, report, validation, &path, status, reason);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, serde_json::to_string_pretty(&log)?)?;
    println!("decision log: {}", path.display());
    Ok(())
}

fn build_decision_log(
    options: &CliOptions,
    report: &GenerateReport,
    validation: Option<&ValidationReport>,
    decision_log_path: &Path,
    status: &str,
    reason: Option<&str>,
) -> DecisionLogReport {
    let validation_summary = validation.map(|validation| DecisionLogValidationSummary {
        status: validation.status.clone(),
        reason: validation.reason.clone(),
        gates: validation.gates.len(),
        attempts: validation.attempts.len(),
        failed_gates: validation
            .gates
            .iter()
            .filter(|gate| gate.status == "failed")
            .map(|gate| gate.name.clone())
            .collect(),
    });
    DecisionLogReport {
        status: status.to_string(),
        reason: reason.map(str::to_string),
        workspace_root: options.workspace_root.clone(),
        output_root: options.output_root.clone(),
        analyzer_mode: options.analyzer_mode.as_str().to_string(),
        production_preset: options.production_preset,
        run_check: options.run_check,
        run_preflight: options.run_preflight,
        feedback_iterations: options.feedback_iterations,
        feedback_repair_iterations: options.feedback_repair_iterations,
        deny_warnings: options.deny_warnings,
        root_selectors: options.root_selectors.clone(),
        cargo_check_args: options.cargo_check_args.clone(),
        reports: DecisionLogReportPaths {
            decision_log: decision_log_path.to_path_buf(),
            slice_report: slice_report_path(options),
            validation_report: validation_report_path(options),
            baseline_report: options
                .run_baseline_check
                .then(|| baseline_report_path(options)),
            baseline_target_dir: options
                .run_baseline_check
                .then(|| baseline_target_dir(options)),
            preflight_report: maybe_preflight_report_path(options),
            feedback_report: feedback_report_path(options),
            feedback_target_dir: (options.run_check
                || options.feedback_iterations > 0
                || options.feedback_repair_iterations > 0)
                .then(|| feedback_target_dir(options)),
        },
        validation: validation_summary,
        steps: decision_log_steps(options, report, validation),
    }
}

fn decision_log_steps(
    options: &CliOptions,
    report: &GenerateReport,
    validation: Option<&ValidationReport>,
) -> Vec<DecisionLogStep> {
    let mut steps = Vec::new();

    let mut input_metrics = BTreeMap::new();
    input_metrics.insert(
        "explicit_root_selectors".to_string(),
        serde_json::json!(options.root_selectors.len()),
    );
    input_metrics.insert(
        "cargo_check_args".to_string(),
        serde_json::json!(options.cargo_check_args.len()),
    );
    steps.push(DecisionLogStep {
        step: "input".to_string(),
        status: "configured".to_string(),
        decision: if options.root_selectors.is_empty() {
            "use source markers".to_string()
        } else {
            "use explicit in-memory roots".to_string()
        },
        reason: if options.root_selectors.is_empty() {
            "no --root or --roots-file was provided, so #[opensourced] markers define the top slice surface".to_string()
        } else {
            "--root/--roots-file selects the top slice surface without modifying the source checkout".to_string()
        },
        metrics: input_metrics,
        evidence: options.root_selectors.clone(),
    });

    let mut analyzer_metrics = BTreeMap::new();
    analyzer_metrics.insert(
        "loaded".to_string(),
        serde_json::json!(report.analyzer.loaded),
    );
    analyzer_metrics.insert(
        "semantic_available".to_string(),
        serde_json::json!(report.analyzer.semantic.is_some()),
    );
    analyzer_metrics.insert(
        "semantic_hint_callable_owners".to_string(),
        serde_json::json!(report.analyzer.semantic_hints.callable_edges.len()),
    );
    analyzer_metrics.insert(
        "semantic_hint_item_owners".to_string(),
        serde_json::json!(report.analyzer.semantic_hints.item_edges.len()),
    );
    analyzer_metrics.insert(
        "semantic_hint_edges".to_string(),
        serde_json::json!(report.analyzer.semantic_hints.total_edges()),
    );
    analyzer_metrics.insert(
        "semantic_unresolved_queries".to_string(),
        serde_json::json!(report.analyzer.semantic_hints.unresolved_queries),
    );
    analyzer_metrics.insert(
        "semantic_unqueried_queries".to_string(),
        serde_json::json!(report.analyzer.semantic_hints.unqueried_queries),
    );
    analyzer_metrics.insert(
        "semantic_unmapped_targets".to_string(),
        serde_json::json!(report.analyzer.semantic_hints.unmapped_targets),
    );
    if let Some(semantic) = &report.analyzer.semantic {
        analyzer_metrics.insert(
            "semantic_source_files".to_string(),
            serde_json::json!(semantic.source_files),
        );
        analyzer_metrics.insert(
            "semantic_analyzed_files".to_string(),
            serde_json::json!(semantic.analyzed_files),
        );
        analyzer_metrics.insert(
            "selected_root_source_files".to_string(),
            serde_json::json!(semantic.selected_root_source_files),
        );
        analyzer_metrics.insert(
            "selected_root_analyzed_files".to_string(),
            serde_json::json!(semantic.selected_root_analyzed_files),
        );
        analyzer_metrics.insert(
            "selected_root_unresolved_paths".to_string(),
            serde_json::json!(semantic.selected_root_unresolved_paths),
        );
        analyzer_metrics.insert(
            "selected_root_unresolved_method_calls".to_string(),
            serde_json::json!(semantic.selected_root_unresolved_method_calls),
        );
    }
    steps.push(DecisionLogStep {
        step: "analyzer".to_string(),
        status: if report.analyzer.semantic.is_some() {
            semantic_proof_status(report.analyzer.semantic.as_ref().unwrap())
        } else if report.analyzer.loaded {
            "loaded_without_semantics".to_string()
        } else {
            "syntactic_only".to_string()
        },
        decision: format!(
            "run {} via {}",
            report.analyzer.mode.as_str(),
            report.analyzer.engine
        ),
        reason: "the slicer uses the syntactic index plus available rust-analyzer semantic hints to expand the downstream dependency closure".to_string(),
        metrics: analyzer_metrics,
        evidence: report.analyzer.notes.clone(),
    });

    let mut root_metrics = BTreeMap::new();
    root_metrics.insert("roots".to_string(), serde_json::json!(report.roots.len()));
    root_metrics.insert(
        "packages".to_string(),
        serde_json::json!(report.packages.len()),
    );
    steps.push(DecisionLogStep {
        step: "root_selection".to_string(),
        status: "selected".to_string(),
        decision: "treat selected roots as the only top-level open-source surface".to_string(),
        reason: "reverse dependents are not retained; the retained workspace is grown only from what selected roots depend on".to_string(),
        metrics: root_metrics,
        evidence: report.roots.iter().map(ToString::to_string).collect(),
    });

    let mut closure_metrics = BTreeMap::new();
    closure_metrics.insert(
        "reachable_callables".to_string(),
        serde_json::json!(report.reachable.len()),
    );
    closure_metrics.insert(
        "reachable_items".to_string(),
        serde_json::json!(report.reachable_items.len()),
    );
    closure_metrics.insert(
        "rendered_callables".to_string(),
        serde_json::json!(rendered_symbol_count(report, "callable")),
    );
    closure_metrics.insert(
        "rendered_items".to_string(),
        serde_json::json!(rendered_symbol_count(report, "item")),
    );
    closure_metrics.insert(
        "files_written".to_string(),
        serde_json::json!(report.files_written),
    );
    closure_metrics.insert(
        "total_ms".to_string(),
        serde_json::json!(report.timings.total_ms),
    );
    closure_metrics.insert(
        "analyzer_ms".to_string(),
        serde_json::json!(report.timings.analyzer_ms),
    );
    closure_metrics.insert(
        "reduce_ms".to_string(),
        serde_json::json!(report.timings.reduce_ms),
    );
    closure_metrics.insert(
        "render_ms".to_string(),
        serde_json::json!(report.timings.render_ms),
    );
    steps.push(DecisionLogStep {
        step: "top_down_closure".to_string(),
        status: "rendered".to_string(),
        decision: "copy and render only the selected roots plus their downstream dependency closure".to_string(),
        reason: "the top-down closure is the production direction: start at the open-source surface, walk dependencies, then render the retained set".to_string(),
        metrics: closure_metrics,
        evidence: report.packages.clone(),
    });

    let macro_summary = &report.macro_surfaces.summary;
    let mut macro_metrics = BTreeMap::new();
    macro_metrics.insert("total".to_string(), serde_json::json!(macro_summary.total));
    macro_metrics.insert(
        "derive_macros".to_string(),
        serde_json::json!(macro_summary.derive_macros),
    );
    macro_metrics.insert(
        "attribute_macros".to_string(),
        serde_json::json!(macro_summary.attribute_macros),
    );
    macro_metrics.insert(
        "helper_attributes".to_string(),
        serde_json::json!(macro_summary.helper_attributes),
    );
    macro_metrics.insert(
        "macro_invocations".to_string(),
        serde_json::json!(macro_summary.macro_invocations),
    );
    macro_metrics.insert(
        "macro_blocked".to_string(),
        serde_json::json!(macro_summary.macro_blocked),
    );
    steps.push(DecisionLogStep {
        step: "macro_surfaces".to_string(),
        status: if macro_summary.macro_blocked > 0 {
            "fail_closed".to_string()
        } else {
            "classified".to_string()
        },
        decision: "scope macro uncertainty to touched macro surfaces".to_string(),
        reason: "unknown macro expansion can require token-named helpers, so affected owners are blocked_by_unknown instead of pruned".to_string(),
        metrics: macro_metrics,
        evidence: report
            .macro_surfaces
            .surfaces
            .iter()
            .take(20)
            .map(|surface| {
                format!(
                    "{} {} on {}",
                    surface.category, surface.path, surface.subject
                )
            })
            .collect(),
    });

    let usage = &report.usage.summary;
    let rendered_usage = rendered_usage_contract(report);
    let mut usage_metrics = BTreeMap::new();
    usage_metrics.insert(
        "used_callables".to_string(),
        serde_json::json!(usage.used_callables),
    );
    usage_metrics.insert(
        "used_items".to_string(),
        serde_json::json!(usage.used_items),
    );
    usage_metrics.insert(
        "prunable_callables".to_string(),
        serde_json::json!(usage.prunable_callables),
    );
    usage_metrics.insert(
        "prunable_items".to_string(),
        serde_json::json!(usage.prunable_items),
    );
    usage_metrics.insert(
        "blocked_by_unknown_callables".to_string(),
        serde_json::json!(usage.blocked_by_unknown_callables),
    );
    usage_metrics.insert(
        "blocked_by_unknown_items".to_string(),
        serde_json::json!(usage.blocked_by_unknown_items),
    );
    usage_metrics.insert(
        "rendered_used".to_string(),
        serde_json::json!(rendered_usage.used),
    );
    usage_metrics.insert(
        "rendered_blocked_by_unknown".to_string(),
        serde_json::json!(rendered_usage.blocked_by_unknown),
    );
    usage_metrics.insert(
        "rendered_invalid".to_string(),
        serde_json::json!(rendered_usage.invalid.len()),
    );
    steps.push(DecisionLogStep {
        step: "usage_pruning".to_string(),
        status: if rendered_usage.invalid.is_empty() {
            "contract_ok".to_string()
        } else {
            "contract_failed".to_string()
        },
        decision: "strip prunable unused symbols and retain used or blocked_by_unknown symbols".to_string(),
        reason: "generated source must not retain known-unused code except where an unknown surface blocks safe deletion".to_string(),
        metrics: usage_metrics,
        evidence: rendered_usage.invalid.into_iter().take(20).collect(),
    });

    let mut readiness_metrics = BTreeMap::new();
    readiness_metrics.insert(
        "hazards".to_string(),
        serde_json::json!(report.production.hazards.len()),
    );
    readiness_metrics.insert(
        "error_hazards".to_string(),
        serde_json::json!(report
            .production
            .hazards
            .iter()
            .filter(|hazard| hazard.severity == "error")
            .count()),
    );
    readiness_metrics.insert(
        "warning_hazards".to_string(),
        serde_json::json!(report
            .production
            .hazards
            .iter()
            .filter(|hazard| hazard.severity == "warning")
            .count()),
    );
    steps.push(DecisionLogStep {
        step: "production_readiness".to_string(),
        status: report.production.status.clone(),
        decision: "accept only if hard validation gates pass".to_string(),
        reason: "hazards describe remaining semantic uncertainty; error hazards fail production validation unless specifically discharged".to_string(),
        metrics: readiness_metrics,
        evidence: report
            .production
            .hazards
            .iter()
            .take(20)
            .map(|hazard| format!("{} {}: {}", hazard.severity, hazard.code, hazard.message))
            .collect(),
    });

    if let Some(validation) = validation {
        let mut validation_metrics = BTreeMap::new();
        validation_metrics.insert(
            "gates".to_string(),
            serde_json::json!(validation.gates.len()),
        );
        validation_metrics.insert(
            "attempts".to_string(),
            serde_json::json!(validation.attempts.len()),
        );
        validation_metrics.insert(
            "failed_gates".to_string(),
            serde_json::json!(validation
                .gates
                .iter()
                .filter(|gate| gate.status == "failed")
                .count()),
        );
        steps.push(DecisionLogStep {
            step: "validation".to_string(),
            status: validation.status.clone(),
            decision: "use validation gates as the acceptance contract".to_string(),
            reason: validation
                .reason
                .clone()
                .unwrap_or_else(|| "all configured validation gates completed".to_string()),
            metrics: validation_metrics,
            evidence: validation
                .gates
                .iter()
                .map(|gate| format!("{}={}: {}", gate.name, gate.status, gate.reason))
                .collect(),
        });
    }

    steps
}

fn rendered_symbol_count(report: &GenerateReport, kind: &str) -> usize {
    report
        .usage
        .rendered_symbols
        .entries
        .iter()
        .filter(|entry| entry.kind == kind)
        .count()
}

#[allow(clippy::too_many_arguments)]
fn record_feedback_attempt(
    validation: &mut ValidationReport,
    stage: &str,
    attempt: usize,
    status: &str,
    reason: &str,
    report: &CheckReport,
    report_path: PathBuf,
    baseline_limited: bool,
    semantic_warning_hazards: usize,
    repairable_warnings: usize,
    repair_report_path: Option<PathBuf>,
    repair_total_changes: Option<usize>,
) {
    validation.attempts.push(ValidationAttemptReport {
        stage: stage.to_string(),
        attempt,
        status: status.to_string(),
        reason: reason.to_string(),
        report_path,
        cargo_success: report.success,
        baseline_limited,
        timed_out: report.timed_out,
        error_count: report.error_count(),
        warning_count: report.warning_count(),
        semantic_warning_hazards,
        repairable_warnings,
        widening_candidates: report.widening.candidates.len(),
        widening_hazards: report.widening.hazards.len(),
        feedback_widened_roots: None,
        repair_report_path,
        repair_total_changes,
    });
}

fn record_feedback_gate(
    validation: &mut ValidationReport,
    name: &str,
    status: &str,
    reason: &str,
    report: &CheckReport,
    report_path: PathBuf,
    semantic_warning_hazards: usize,
) {
    validation.add_check_gate(
        name,
        status,
        reason,
        report,
        Some(report_path),
        semantic_warning_hazards,
    );
}

fn record_production_readiness_gate(
    options: &CliOptions,
    validation: &mut ValidationReport,
    production: &opensource_core::ProductionReadinessReport,
    phase: &str,
) {
    let status = production_readiness_gate_status(options, production);
    let error_count = production
        .hazards
        .iter()
        .filter(|hazard| hazard.severity == "error")
        .count();
    let warning_count = production
        .hazards
        .iter()
        .filter(|hazard| hazard.severity == "warning")
        .count();
    let discharged_error_count = production
        .hazards
        .iter()
        .filter(|hazard| {
            hazard.severity == "error" && production_error_hazard_is_discharged(options, hazard)
        })
        .count();
    let reason = if discharged_error_count > 0 {
        format!(
            "{} production hazard(s) reported {phase}; {} feature cfg error hazard(s) covered by validation cargo arguments",
            production.hazards.len(),
            discharged_error_count
        )
    } else {
        format!(
            "{} production hazard(s) reported {phase}",
            production.hazards.len()
        )
    };
    validation.gates.push(ValidationGateReport {
        name: "production_readiness".to_string(),
        status,
        reason,
        report_path: slice_report_path(options),
        error_count: Some(error_count),
        warning_count: Some(warning_count),
        semantic_warning_hazards: None,
        review_warning_hazards: Some(review_warning_hazard_count(production)),
    });
}

#[allow(clippy::too_many_arguments)]
fn try_widen_from_feedback(
    options: &CliOptions,
    validation: &mut ValidationReport,
    state: &mut FeedbackWideningState,
    stage: &str,
    attempt: usize,
    report: &CheckReport,
    report_path: &Path,
    semantic_warning_hazards: usize,
    repairable_warnings: usize,
) -> Result<bool, Box<dyn std::error::Error>> {
    if report.error_count() == 0 {
        return Ok(false);
    }

    state.diagnostics.extend(report.diagnostics.clone());
    let widened_report = generate_with_analyzer_feedback_and_roots(
        GenerateOptions {
            workspace_root: options.workspace_root.clone(),
            output_root: options.output_root.clone(),
        },
        options.analyzer_mode,
        &state.diagnostics,
        &options.root_selectors,
    )?;
    let widened_roots = widened_report
        .feedback_widened_roots
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let widened_signature = widened_roots.join("\n");
    if widened_signature.is_empty() || !state.seen_root_sets.insert(widened_signature) {
        return Ok(false);
    }

    if let Some(report_path) = slice_report_path(options) {
        write_generate_report(&widened_report, &report_path)?;
    }
    println!(
        "feedback: widened {} root(s) from compiler diagnostics and re-rendered generated workspace",
        widened_roots.len()
    );
    record_feedback_attempt(
        validation,
        stage,
        attempt,
        "widened",
        "compiler feedback widened the generated workspace",
        report,
        report_path.to_path_buf(),
        false,
        semantic_warning_hazards,
        repairable_warnings,
        None,
        None,
    );
    if let Some(recorded) = validation.attempts.last_mut() {
        recorded.feedback_widened_roots = Some(widened_roots.len());
    }
    record_production_readiness_gate(
        options,
        validation,
        &widened_report.production,
        "after compiler feedback widening",
    );
    if production_readiness_blocks_validation(options, &widened_report.production) {
        return Err(
            "production readiness reported error hazards after compiler feedback widening".into(),
        );
    }

    refresh_generated_lockfile_for_locked_validation(options, validation)?;
    let preflight = run_preflight(options)?;
    if !preflight.success {
        return Err("feedback widening produced a structurally invalid generated workspace".into());
    }

    Ok(true)
}

fn run_feedback_loop(
    options: &CliOptions,
    baseline: Option<&CheckReport>,
    validation: &mut ValidationReport,
) -> Result<(), Box<dyn std::error::Error>> {
    let report_path = options
        .feedback_report
        .clone()
        .unwrap_or_else(|| options.output_root.join("slice-feedback.json"));
    let mut seen_diagnostics = std::collections::BTreeSet::new();
    let mut seen_diagnostic_shapes = std::collections::BTreeSet::new();
    let mut widening_state = FeedbackWideningState::default();

    for attempt in 1..=options.feedback_iterations {
        println!(
            "feedback attempt {attempt}/{}: cargo check --message-format=json",
            options.feedback_iterations
        );
        let report = check_workspace(CheckOptions {
            manifest_path: options.output_root.join("Cargo.toml"),
            target_dir: Some(feedback_target_dir(options)),
            timeout: options.feedback_timeout,
            cargo_args: options.cargo_check_args.clone(),
        })?;
        write_report(&report, &report_path)?;
        print_feedback(&report, options.feedback_limit, &report_path);

        let semantic_warnings = semantic_hazard_warning_count(&report.diagnostics, baseline);
        if feedback_is_accepted(&report, baseline, options.deny_warnings) {
            record_feedback_attempt(
                validation,
                "feedback",
                attempt,
                "accepted",
                "generated workspace cargo check passed all feedback gates",
                &report,
                report_path.clone(),
                false,
                semantic_warnings,
                repairable_warning_count(&report.diagnostics),
                None,
                None,
            );
            record_feedback_gate(
                validation,
                "feedback",
                "accepted",
                "generated workspace cargo check passed all feedback gates",
                &report,
                report_path.clone(),
                semantic_warnings,
            );
            return Ok(());
        }
        if report.success && semantic_warnings > 0 {
            println!("feedback: semantic warning gate rejected {semantic_warnings} new warning(s)");
        }
        if report.success && options.deny_warnings {
            println!("feedback: warnings denied by --deny-warnings");
        }
        if options.allow_baseline_failures
            && baseline_limited_feedback_is_accepted(&report, baseline, options.deny_warnings)
        {
            println!(
                "feedback: generated errors match the source baseline; treating as baseline-limited pass"
            );
            record_feedback_attempt(
                validation,
                "feedback",
                attempt,
                "baseline_limited",
                "generated errors match the source baseline and remaining gates passed",
                &report,
                report_path.clone(),
                true,
                semantic_warnings,
                repairable_warning_count(&report.diagnostics),
                None,
                None,
            );
            record_feedback_gate(
                validation,
                "feedback",
                "baseline_limited",
                "generated errors match the source baseline and remaining gates passed",
                &report,
                report_path.clone(),
                semantic_warnings,
            );
            return Ok(());
        }
        if report.timed_out {
            record_feedback_attempt(
                validation,
                "feedback",
                attempt,
                "timed_out",
                "feedback cargo check timed out",
                &report,
                report_path.clone(),
                false,
                semantic_warnings,
                repairable_warning_count(&report.diagnostics),
                None,
                None,
            );
            record_feedback_gate(
                validation,
                "feedback",
                "failed",
                "feedback cargo check timed out",
                &report,
                report_path.clone(),
                semantic_warnings,
            );
            return Err("feedback cargo check timed out".into());
        }

        if try_widen_from_feedback(
            options,
            validation,
            &mut widening_state,
            "feedback",
            attempt,
            &report,
            &report_path,
            semantic_warnings,
            repairable_warning_count(&report.diagnostics),
        )? {
            continue;
        }

        let signature = diagnostics_signature(&report.diagnostics);
        if !seen_diagnostics.insert(signature) {
            record_feedback_attempt(
                validation,
                "feedback",
                attempt,
                "no_progress",
                "feedback made no diagnostic progress",
                &report,
                report_path.clone(),
                false,
                semantic_warnings,
                repairable_warning_count(&report.diagnostics),
                None,
                None,
            );
            record_feedback_gate(
                validation,
                "feedback",
                "failed",
                "feedback made no diagnostic progress",
                &report,
                report_path.clone(),
                semantic_warnings,
            );
            return Err(format!(
                "feedback made no diagnostic progress; report written to {}",
                report_path.display()
            )
            .into());
        }
        let shape_signature = diagnostics_shape_signature(&report.diagnostics);
        if !seen_diagnostic_shapes.insert(shape_signature) {
            record_feedback_attempt(
                validation,
                "feedback",
                attempt,
                "low_progress",
                "feedback repeated the same diagnostic shape",
                &report,
                report_path.clone(),
                false,
                semantic_warnings,
                repairable_warning_count(&report.diagnostics),
                None,
                None,
            );
            record_feedback_gate(
                validation,
                "feedback",
                "failed",
                "feedback repeated the same diagnostic shape",
                &report,
                report_path.clone(),
                semantic_warnings,
            );
            return Err(format!(
                "feedback repeated the same diagnostic shape; report written to {}",
                report_path.display()
            )
            .into());
        }
        record_feedback_attempt(
            validation,
            "feedback",
            attempt,
            "retrying",
            "generated workspace did not pass feedback gates",
            &report,
            report_path.clone(),
            false,
            semantic_warnings,
            repairable_warning_count(&report.diagnostics),
            None,
            None,
        );
    }

    validation.gates.push(ValidationGateReport {
        name: "feedback".to_string(),
        status: "failed".to_string(),
        reason: "generated workspace failed compiler feedback loop".to_string(),
        report_path: Some(report_path.clone()),
        error_count: None,
        warning_count: None,
        semantic_warning_hazards: None,
        review_warning_hazards: None,
    });
    Err(format!(
        "generated workspace failed compiler feedback loop; report written to {}",
        report_path.display()
    )
    .into())
}

fn run_feedback_repair_loop(
    options: &CliOptions,
    baseline: Option<&CheckReport>,
    validation: &mut ValidationReport,
) -> Result<(), Box<dyn std::error::Error>> {
    let feedback_report_path = options
        .feedback_report
        .clone()
        .unwrap_or_else(|| options.output_root.join("slice-feedback.json"));
    let repair_report_path = options
        .repair_report
        .clone()
        .unwrap_or_else(|| options.output_root.join("slice-repair.json"));
    let mut seen_diagnostics = std::collections::BTreeSet::new();
    let mut seen_diagnostic_shapes = std::collections::BTreeSet::new();
    let mut widening_state = FeedbackWideningState::default();
    let mut repaired_previous_attempt = false;

    for attempt in 1..=options.feedback_repair_iterations {
        println!(
            "feedback repair attempt {attempt}/{}: cargo check --message-format=json",
            options.feedback_repair_iterations
        );
        let report = check_workspace(CheckOptions {
            manifest_path: options.output_root.join("Cargo.toml"),
            target_dir: Some(feedback_target_dir(options)),
            timeout: options.feedback_timeout,
            cargo_args: options.cargo_check_args.clone(),
        })?;
        write_report(&report, &feedback_report_path)?;
        print_feedback(&report, options.feedback_limit, &feedback_report_path);

        let warnings = report.warning_count();
        let semantic_warnings = semantic_hazard_warning_count(&report.diagnostics, baseline);
        let repairable_warnings = repairable_warning_count(&report.diagnostics);
        if feedback_repair_is_accepted(&report, baseline, options.deny_warnings) {
            record_feedback_attempt(
                validation,
                "feedback-repair",
                attempt,
                "accepted",
                "generated workspace cargo check passed all feedback gates",
                &report,
                feedback_report_path.clone(),
                false,
                semantic_warnings,
                repairable_warnings,
                None,
                None,
            );
            record_feedback_gate(
                validation,
                "feedback-repair",
                "accepted",
                "generated workspace cargo check passed all feedback gates",
                &report,
                feedback_report_path.clone(),
                semantic_warnings,
            );
            return Ok(());
        }
        if report.success && semantic_warnings > 0 {
            println!("feedback: semantic warning gate rejected {semantic_warnings} new warning(s)");
            record_feedback_attempt(
                validation,
                "feedback-repair",
                attempt,
                "rejected",
                "semantic warning gate rejected generated workspace",
                &report,
                feedback_report_path.clone(),
                false,
                semantic_warnings,
                repairable_warnings,
                None,
                None,
            );
            record_feedback_gate(
                validation,
                "feedback-repair",
                "failed",
                "semantic warning gate rejected generated workspace",
                &report,
                feedback_report_path.clone(),
                semantic_warnings,
            );
            break;
        }
        if report.success && repairable_warnings > 0 {
            println!(
                "feedback: cargo check passed but {repairable_warnings} repairable warning(s) remain; attempting conservative repair"
            );
        }
        if report.success && options.deny_warnings && repairable_warnings == 0 && warnings > 0 {
            println!("feedback: warnings denied by --deny-warnings and no conservative repair is available");
            record_feedback_attempt(
                validation,
                "feedback-repair",
                attempt,
                "rejected",
                "warnings were denied and no conservative repair was available",
                &report,
                feedback_report_path.clone(),
                false,
                semantic_warnings,
                repairable_warnings,
                None,
                None,
            );
            record_feedback_gate(
                validation,
                "feedback-repair",
                "failed",
                "warnings were denied and no conservative repair was available",
                &report,
                feedback_report_path.clone(),
                semantic_warnings,
            );
            break;
        }
        if options.allow_baseline_failures
            && baseline_limited_feedback_repair_is_accepted(
                &report,
                baseline,
                options.deny_warnings,
            )
        {
            println!(
                "feedback: generated errors match the source baseline; treating as baseline-limited pass"
            );
            record_feedback_attempt(
                validation,
                "feedback-repair",
                attempt,
                "baseline_limited",
                "generated errors match the source baseline and remaining gates passed",
                &report,
                feedback_report_path.clone(),
                true,
                semantic_warnings,
                repairable_warnings,
                None,
                None,
            );
            record_feedback_gate(
                validation,
                "feedback-repair",
                "baseline_limited",
                "generated errors match the source baseline and remaining gates passed",
                &report,
                feedback_report_path.clone(),
                semantic_warnings,
            );
            return Ok(());
        }
        if report.timed_out {
            record_feedback_attempt(
                validation,
                "feedback-repair",
                attempt,
                "timed_out",
                "feedback cargo check timed out",
                &report,
                feedback_report_path.clone(),
                false,
                semantic_warnings,
                repairable_warnings,
                None,
                None,
            );
            record_feedback_gate(
                validation,
                "feedback-repair",
                "failed",
                "feedback cargo check timed out",
                &report,
                feedback_report_path.clone(),
                semantic_warnings,
            );
            break;
        }

        if try_widen_from_feedback(
            options,
            validation,
            &mut widening_state,
            "feedback-repair",
            attempt,
            &report,
            &feedback_report_path,
            semantic_warnings,
            repairable_warnings,
        )? {
            repaired_previous_attempt = false;
            continue;
        }

        let signature = diagnostics_signature(&report.diagnostics);
        if !seen_diagnostics.insert(signature) {
            record_feedback_attempt(
                validation,
                "feedback-repair",
                attempt,
                "no_progress",
                "feedback repair made no diagnostic progress",
                &report,
                feedback_report_path.clone(),
                false,
                semantic_warnings,
                repairable_warnings,
                None,
                None,
            );
            record_feedback_gate(
                validation,
                "feedback-repair",
                "failed",
                "feedback repair made no diagnostic progress",
                &report,
                feedback_report_path.clone(),
                semantic_warnings,
            );
            return Err(format!(
                "feedback repair made no diagnostic progress; report written to {}",
                feedback_report_path.display()
            )
            .into());
        }
        let shape_signature = diagnostics_shape_signature(&report.diagnostics);
        let shape_repeated = !seen_diagnostic_shapes.insert(shape_signature);
        if repaired_previous_attempt && report.error_count() > 0 && shape_repeated {
            record_feedback_attempt(
                validation,
                "feedback-repair",
                attempt,
                "low_progress",
                "feedback repair repeated the same diagnostic shape after changing files",
                &report,
                feedback_report_path.clone(),
                false,
                semantic_warnings,
                repairable_warnings,
                None,
                None,
            );
            record_feedback_gate(
                validation,
                "feedback-repair",
                "failed",
                "feedback repair repeated the same diagnostic shape after changing files",
                &report,
                feedback_report_path.clone(),
                semantic_warnings,
            );
            return Err(format!(
                "feedback repair repeated the same diagnostic shape after changing files; report written to {}",
                feedback_report_path.display()
            )
            .into());
        }

        let repair_report = repair_workspace(RepairOptions {
            output_root: options.output_root.clone(),
            diagnostics: report.diagnostics.clone(),
        })?;
        write_repair_report(&repair_report, &repair_report_path)?;
        println!(
            "repair: removed_items={}, removed_imports={}, normalized_paths={}, applied_suggestions={}, added_dead_code_allows={}, deferred_dead_code_allows={}, skipped_diagnostics={}, changed_files={}, total_changes={}; report: {}",
            repair_report.removed_items,
            repair_report.removed_imports,
            repair_report.normalized_paths,
            repair_report.applied_suggestions,
            repair_report.added_dead_code_allows,
            repair_report.deferred_dead_code_allows,
            repair_report.skipped_diagnostics,
            repair_report.changed_files.len(),
            repair_report.total_changes(),
            repair_report_path.display()
        );

        let repair_total_changes = repair_report.total_changes();
        record_feedback_attempt(
            validation,
            "feedback-repair",
            attempt,
            if repair_total_changes > 0 {
                "repaired"
            } else {
                "unrepaired"
            },
            if repair_total_changes > 0 {
                "conservative repair changed the generated workspace"
            } else {
                "no conservative repair was available"
            },
            &report,
            feedback_report_path.clone(),
            false,
            semantic_warnings,
            repairable_warnings,
            Some(repair_report_path.clone()),
            Some(repair_total_changes),
        );

        if repair_total_changes == 0 {
            break;
        }
        repaired_previous_attempt = true;

        let preflight = run_preflight(options)?;
        if !preflight.success {
            return Err("repair produced a structurally invalid generated workspace".into());
        }
        refresh_generated_lockfile_for_locked_validation(options, validation)?;
    }

    if !validation
        .gates
        .iter()
        .any(|gate| gate.name == "feedback-repair")
    {
        validation.gates.push(ValidationGateReport {
            name: "feedback-repair".to_string(),
            status: "failed".to_string(),
            reason: "generated workspace failed compiler repair loop".to_string(),
            report_path: Some(feedback_report_path.clone()),
            error_count: None,
            warning_count: None,
            semantic_warning_hazards: None,
            review_warning_hazards: None,
        });
    }
    Err(format!(
        "generated workspace failed compiler repair loop; feedback report written to {}, repair report written to {}",
        feedback_report_path.display(),
        repair_report_path.display()
    )
    .into())
}

fn run_production_validation_matrix(
    options: &CliOptions,
    production: &opensource_core::ProductionReadinessReport,
    validation: &mut ValidationReport,
) -> Result<(), Box<dyn std::error::Error>> {
    if !options.production_preset {
        return Ok(());
    }

    let entries = production_validation_matrix_entries(options, production);
    if entries.is_empty() {
        validation.gates.push(ValidationGateReport {
            name: "production_matrix".to_string(),
            status: "not_required".to_string(),
            reason: "no uncovered concrete feature cfg matrix entries were discovered".to_string(),
            report_path: slice_report_path(options),
            error_count: None,
            warning_count: None,
            semantic_warning_hazards: None,
            review_warning_hazards: None,
        });
        return Ok(());
    }

    let mut baseline_limited = false;
    for (index, entry) in entries.iter().enumerate() {
        let matrix_index = index + 1;
        let baseline_report_path = production_matrix_baseline_report_path(options, matrix_index);
        let feedback_report_path = production_matrix_feedback_report_path(options, matrix_index);
        println!(
            "production matrix {matrix_index}/{} ({}): cargo check --message-format=json {}",
            entries.len(),
            entry.name,
            entry.cargo_args.join(" ")
        );

        let baseline = run_baseline_check_with_args(
            options,
            entry.cargo_args.clone(),
            production_matrix_baseline_target_dir(options, matrix_index),
        )?;
        write_report(&baseline, &baseline_report_path)?;
        validation.add_check_gate(
            "baseline-matrix",
            if baseline.success {
                "passed"
            } else if options.allow_baseline_failures {
                "baseline_allowed"
            } else {
                "failed"
            },
            if baseline.success {
                "source workspace matrix cargo check passed"
            } else if options.allow_baseline_failures {
                "source workspace matrix baseline failed but --allow-baseline-failures is enabled"
            } else {
                "source workspace matrix baseline failed"
            },
            &baseline,
            Some(baseline_report_path.clone()),
            0,
        );
        if !baseline.success && !options.allow_baseline_failures {
            return Err(format!(
                "source workspace failed production matrix baseline {}; report written to {}",
                entry.name,
                baseline_report_path.display()
            )
            .into());
        }

        let mut seen_diagnostics = BTreeSet::new();
        let mut seen_diagnostic_shapes = BTreeSet::new();
        let mut widening_state = FeedbackWideningState::default();
        let mut accepted = false;
        let mut entry_baseline_limited = false;
        for attempt in 1..=production_matrix_iterations(options) {
            let report = check_workspace(CheckOptions {
                manifest_path: options.output_root.join("Cargo.toml"),
                target_dir: Some(feedback_target_dir(options)),
                timeout: options.feedback_timeout,
                cargo_args: entry.cargo_args.clone(),
            })?;
            write_report(&report, &feedback_report_path)?;
            print_feedback(&report, options.feedback_limit, &feedback_report_path);

            let semantic_warnings =
                semantic_hazard_warning_count(&report.diagnostics, Some(&baseline));
            let repairable_warnings = repairable_warning_count(&report.diagnostics);
            if feedback_is_accepted(&report, Some(&baseline), options.deny_warnings) {
                record_feedback_attempt(
                    validation,
                    "production-matrix",
                    attempt,
                    "accepted",
                    &entry.reason,
                    &report,
                    feedback_report_path.clone(),
                    false,
                    semantic_warnings,
                    repairable_warnings,
                    None,
                    None,
                );
                accepted = true;
                break;
            }
            if options.allow_baseline_failures
                && baseline_limited_feedback_is_accepted(
                    &report,
                    Some(&baseline),
                    options.deny_warnings,
                )
            {
                record_feedback_attempt(
                    validation,
                    "production-matrix",
                    attempt,
                    "baseline_limited",
                    "generated matrix errors match the source matrix baseline",
                    &report,
                    feedback_report_path.clone(),
                    true,
                    semantic_warnings,
                    repairable_warnings,
                    None,
                    None,
                );
                accepted = true;
                entry_baseline_limited = true;
                break;
            }
            if report.timed_out {
                record_feedback_attempt(
                    validation,
                    "production-matrix",
                    attempt,
                    "timed_out",
                    "production matrix cargo check timed out",
                    &report,
                    feedback_report_path.clone(),
                    false,
                    semantic_warnings,
                    repairable_warnings,
                    None,
                    None,
                );
                record_feedback_gate(
                    validation,
                    "production_matrix",
                    "failed",
                    "production matrix cargo check timed out",
                    &report,
                    feedback_report_path.clone(),
                    semantic_warnings,
                );
                return Err(format!(
                    "production matrix {} timed out; report written to {}",
                    entry.name,
                    feedback_report_path.display()
                )
                .into());
            }

            if try_widen_from_feedback(
                options,
                validation,
                &mut widening_state,
                "production-matrix",
                attempt,
                &report,
                &feedback_report_path,
                semantic_warnings,
                repairable_warnings,
            )? {
                continue;
            }

            let signature = diagnostics_signature(&report.diagnostics);
            if !seen_diagnostics.insert(signature) {
                record_feedback_attempt(
                    validation,
                    "production-matrix",
                    attempt,
                    "no_progress",
                    "production matrix made no diagnostic progress",
                    &report,
                    feedback_report_path.clone(),
                    false,
                    semantic_warnings,
                    repairable_warnings,
                    None,
                    None,
                );
                record_feedback_gate(
                    validation,
                    "production_matrix",
                    "failed",
                    "production matrix made no diagnostic progress",
                    &report,
                    feedback_report_path.clone(),
                    semantic_warnings,
                );
                return Err(format!(
                    "production matrix {} made no diagnostic progress; report written to {}",
                    entry.name,
                    feedback_report_path.display()
                )
                .into());
            }
            let shape_signature = diagnostics_shape_signature(&report.diagnostics);
            if !seen_diagnostic_shapes.insert(shape_signature) {
                record_feedback_attempt(
                    validation,
                    "production-matrix",
                    attempt,
                    "low_progress",
                    "production matrix repeated the same diagnostic shape",
                    &report,
                    feedback_report_path.clone(),
                    false,
                    semantic_warnings,
                    repairable_warnings,
                    None,
                    None,
                );
                record_feedback_gate(
                    validation,
                    "production_matrix",
                    "failed",
                    "production matrix repeated the same diagnostic shape",
                    &report,
                    feedback_report_path.clone(),
                    semantic_warnings,
                );
                return Err(format!(
                    "production matrix {} repeated the same diagnostic shape; report written to {}",
                    entry.name,
                    feedback_report_path.display()
                )
                .into());
            }
            record_feedback_attempt(
                validation,
                "production-matrix",
                attempt,
                "retrying",
                "generated workspace did not pass production matrix feedback",
                &report,
                feedback_report_path.clone(),
                false,
                semantic_warnings,
                repairable_warnings,
                None,
                None,
            );
        }

        if !accepted {
            validation.gates.push(ValidationGateReport {
                name: "production_matrix".to_string(),
                status: "failed".to_string(),
                reason: "generated workspace failed production matrix feedback".to_string(),
                report_path: Some(feedback_report_path.clone()),
                error_count: None,
                warning_count: None,
                semantic_warning_hazards: None,
                review_warning_hazards: None,
            });
            return Err(format!(
                "generated workspace failed production matrix {}; report written to {}",
                entry.name,
                feedback_report_path.display()
            )
            .into());
        }
        baseline_limited |= entry_baseline_limited;
    }

    validation.gates.push(ValidationGateReport {
        name: "production_matrix".to_string(),
        status: if baseline_limited {
            "baseline_limited"
        } else {
            "accepted"
        }
        .to_string(),
        reason: format!(
            "{} production feature matrix check(s) passed compiler feedback",
            entries.len()
        ),
        report_path: Some(production_matrix_feedback_report_path(
            options,
            entries.len(),
        )),
        error_count: Some(0),
        warning_count: Some(0),
        semantic_warning_hazards: Some(0),
        review_warning_hazards: Some(0),
    });
    Ok(())
}

fn production_validation_matrix_entries(
    options: &CliOptions,
    production: &opensource_core::ProductionReadinessReport,
) -> Vec<ProductionMatrixEntry> {
    if !options.production_preset
        || options
            .cargo_check_args
            .iter()
            .any(|arg| arg == "--all-features")
    {
        return Vec::new();
    }

    let mut uncovered_features = BTreeSet::new();
    for hazard in &production.hazards {
        if !matches!(
            hazard.code.as_str(),
            "cfg_gated_roots" | "conditional_compilation_attrs"
        ) {
            continue;
        }
        for detail in &hazard.details {
            if cfg_gate_detail_is_covered_by_args(options, detail) {
                continue;
            }
            let package = detail.package.as_deref();
            for feature in enabled_cargo_features(&detail.suggested_cargo_args) {
                if feature_is_enabled_for_matrix_detail(
                    &feature,
                    package,
                    &options.cargo_check_args,
                ) {
                    continue;
                }
                uncovered_features.insert(match package {
                    Some(package) if !feature.contains('/') => format!("{package}/{feature}"),
                    _ => feature,
                });
            }
        }
    }

    if uncovered_features.is_empty() {
        return Vec::new();
    }

    let mut cargo_args = options.cargo_check_args.clone();
    cargo_args.push("--features".to_string());
    cargo_args.push(
        uncovered_features
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .join(","),
    );
    vec![ProductionMatrixEntry {
        name: format!(
            "features:{}",
            uncovered_features
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(",")
        ),
        reason: "production matrix validated retained feature cfg surfaces".to_string(),
        cargo_args,
    }]
}

fn feature_is_enabled_for_matrix_detail(
    feature: &str,
    package: Option<&str>,
    cargo_args: &[String],
) -> bool {
    if cargo_args.iter().any(|arg| arg == "--all-features") {
        return true;
    }
    let enabled = enabled_cargo_features(cargo_args);
    enabled.contains(feature)
        || package
            .map(|package| enabled.contains(&format!("{package}/{feature}")))
            .unwrap_or(false)
}

fn production_matrix_iterations(options: &CliOptions) -> usize {
    options
        .feedback_repair_iterations
        .max(options.feedback_iterations)
        .max(1)
}

fn production_matrix_feedback_report_path(options: &CliOptions, index: usize) -> PathBuf {
    options
        .output_root
        .join(format!("slice-feedback-matrix-{index}.json"))
}

fn production_matrix_baseline_report_path(options: &CliOptions, index: usize) -> PathBuf {
    options
        .output_root
        .join(format!("slice-baseline-matrix-{index}.json"))
}

fn production_matrix_baseline_target_dir(options: &CliOptions, index: usize) -> PathBuf {
    sibling_output_path(
        &options.output_root,
        &format!("target-baseline-matrix-{index}"),
    )
}

fn feedback_target_dir(options: &CliOptions) -> PathBuf {
    options
        .feedback_target_dir
        .clone()
        .unwrap_or_else(|| options.output_root.join("target-feedback"))
}

fn production_readiness_gate_status(
    options: &CliOptions,
    production: &opensource_core::ProductionReadinessReport,
) -> String {
    if production.status == "hazards_detected"
        && options.production_preset
        && production_has_error_hazards(production)
        && !production_readiness_blocks_validation(options, production)
    {
        "requires_feedback".to_string()
    } else {
        production.status.clone()
    }
}

fn production_has_error_hazards(production: &opensource_core::ProductionReadinessReport) -> bool {
    production
        .hazards
        .iter()
        .any(|hazard| hazard.severity == "error")
}

fn review_warning_hazard_count(production: &opensource_core::ProductionReadinessReport) -> usize {
    production
        .hazards
        .iter()
        .filter(|hazard| {
            hazard.severity == "warning" && !production_warning_hazard_is_discharged(hazard)
        })
        .count()
}

fn production_warning_hazard_is_discharged(
    hazard: &opensource_core::ProductionHazardReport,
) -> bool {
    matches!(
        hazard.code.as_str(),
        "conditional_compilation_attrs"
            | "custom_attribute_macros"
            | "custom_derive_macros"
            | "custom_macro_invocations"
            | "dynamic_callback_boundaries"
            | "semantic_analyzer_unavailable"
            | "semantic_file_budget_exhausted"
            | "semantic_file_failures"
            | "semantic_inventory_available"
            | "semantic_inventory_not_applied"
            | "semantic_inventory_partially_applied"
            | "semantic_method_call_budget_exhausted"
            | "semantic_path_budget_exhausted"
            | "semantic_reduction_hints_applied"
            | "semantic_unresolved_method_calls"
            | "semantic_unresolved_paths"
            | "syntactic_method_fallback_cap"
            | "syntactic_method_fallbacks"
    )
}

fn production_readiness_blocks_validation(
    options: &CliOptions,
    production: &opensource_core::ProductionReadinessReport,
) -> bool {
    options.production_preset
        && production.status == "hazards_detected"
        && production.hazards.iter().any(|hazard| {
            hazard.severity == "error" && !production_error_hazard_is_discharged(options, hazard)
        })
}

fn production_error_hazard_is_discharged(
    options: &CliOptions,
    hazard: &opensource_core::ProductionHazardReport,
) -> bool {
    hazard.code == "cfg_gated_roots"
        && !hazard.details.is_empty()
        && hazard
            .details
            .iter()
            .all(|detail| cfg_gate_detail_is_covered_by_args(options, detail))
}

fn cfg_gate_detail_is_covered_by_args(
    options: &CliOptions,
    detail: &opensource_core::ProductionHazardDetail,
) -> bool {
    if let Some(expr) = detail.cfg.as_deref().and_then(cfg_attribute_expression) {
        let Some(cfg_set) = selected_rustc_cfg_set(&options.cargo_check_args) else {
            return false;
        };
        let enabled_features = enabled_cargo_features(&options.cargo_check_args);
        return evaluate_cfg_expr(
            &expr,
            &cfg_set,
            &enabled_features,
            detail.package.as_deref(),
            cargo_args_enable_all_features(&options.cargo_check_args),
        ) == CfgEval::True;
    }

    let required_features = enabled_cargo_features(&detail.suggested_cargo_args);
    !required_features.is_empty()
        && cargo_features_are_enabled(
            &required_features,
            detail.package.as_deref(),
            &enabled_cargo_features(&options.cargo_check_args),
            cargo_args_enable_all_features(&options.cargo_check_args),
        )
}

#[derive(Clone, Debug, Default)]
struct RustcCfgSet {
    values: BTreeMap<String, BTreeSet<String>>,
    flags: BTreeSet<String>,
}

impl RustcCfgSet {
    fn insert_value(&mut self, key: &str, value: &str) {
        self.values
            .entry(key.to_string())
            .or_default()
            .insert(value.to_string());
    }

    fn values(&self, key: &str) -> Option<&BTreeSet<String>> {
        self.values.get(key)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum CfgExpr {
    All(Vec<CfgExpr>),
    Any(Vec<CfgExpr>),
    Not(Box<CfgExpr>),
    Flag(String),
    KeyValue { key: String, value: String },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CfgEval {
    True,
    False,
    Unknown,
}

impl CfgEval {
    fn and(values: impl IntoIterator<Item = CfgEval>) -> Self {
        let mut saw_unknown = false;
        for value in values {
            match value {
                CfgEval::True => {}
                CfgEval::False => return CfgEval::False,
                CfgEval::Unknown => saw_unknown = true,
            }
        }
        if saw_unknown {
            CfgEval::Unknown
        } else {
            CfgEval::True
        }
    }

    fn or(values: impl IntoIterator<Item = CfgEval>) -> Self {
        let mut saw_unknown = false;
        for value in values {
            match value {
                CfgEval::True => return CfgEval::True,
                CfgEval::False => {}
                CfgEval::Unknown => saw_unknown = true,
            }
        }
        if saw_unknown {
            CfgEval::Unknown
        } else {
            CfgEval::False
        }
    }

    fn not(self) -> Self {
        match self {
            CfgEval::True => CfgEval::False,
            CfgEval::False => CfgEval::True,
            CfgEval::Unknown => CfgEval::Unknown,
        }
    }
}

fn cfg_attribute_expression(cfg: &str) -> Option<CfgExpr> {
    let normalized = compact_cfg_attribute(cfg);
    if let Some(arguments) = cfg_call_arguments(&normalized, "cfg_attr") {
        return cfg_first_top_level_argument(arguments).and_then(parse_cfg_expr);
    }
    if let Some(arguments) = cfg_call_arguments(&normalized, "cfg") {
        return parse_cfg_expr(arguments);
    }
    None
}

fn compact_cfg_attribute(cfg: &str) -> String {
    let mut normalized = String::new();
    let mut in_string = false;
    let mut escaped = false;
    for ch in cfg.chars() {
        if in_string {
            normalized.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
        } else if ch == '"' {
            in_string = true;
            normalized.push(ch);
        } else if !ch.is_whitespace() {
            normalized.push(ch);
        }
    }
    normalized
}

fn cfg_call_arguments<'a>(cfg: &'a str, call: &str) -> Option<&'a str> {
    let pattern = format!("{call}(");
    let start = cfg.find(&pattern)? + pattern.len();
    let end = matching_paren_index(cfg, start.checked_sub(1)?)?;
    Some(&cfg[start..end])
}

fn matching_paren_index(text: &str, open_index: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth = 0usize;
    let mut index = open_index;
    let mut in_string = false;
    let mut escaped = false;
    while index < bytes.len() {
        let byte = bytes[index];
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
        } else if byte == b'"' {
            in_string = true;
        } else if byte == b'(' {
            depth += 1;
        } else if byte == b')' {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(index);
            }
        }
        index += 1;
    }
    None
}

fn cfg_first_top_level_argument(arguments: &str) -> Option<&str> {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (index, byte) in arguments.bytes().enumerate() {
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
        } else if byte == b'"' {
            in_string = true;
        } else if byte == b'(' {
            depth += 1;
        } else if byte == b')' {
            depth = depth.checked_sub(1)?;
        } else if byte == b',' && depth == 0 {
            return Some(&arguments[..index]);
        }
    }
    (!arguments.is_empty()).then_some(arguments)
}

fn parse_cfg_expr(text: &str) -> Option<CfgExpr> {
    let mut parser = CfgParser::new(text);
    let expr = parser.parse_expr()?;
    parser.is_finished().then_some(expr)
}

struct CfgParser<'a> {
    text: &'a str,
    index: usize,
}

impl<'a> CfgParser<'a> {
    fn new(text: &'a str) -> Self {
        Self { text, index: 0 }
    }

    fn is_finished(&self) -> bool {
        self.index == self.text.len()
    }

    fn parse_expr(&mut self) -> Option<CfgExpr> {
        let ident = self.parse_ident()?;
        if self.consume(b'(') {
            let args = self.parse_expr_list()?;
            return match ident.as_str() {
                "all" => Some(CfgExpr::All(args)),
                "any" => Some(CfgExpr::Any(args)),
                "not" if args.len() == 1 => Some(CfgExpr::Not(Box::new(args.into_iter().next()?))),
                _ => None,
            };
        }
        if self.consume(b'=') {
            let value = self.parse_string()?;
            return Some(CfgExpr::KeyValue { key: ident, value });
        }
        Some(CfgExpr::Flag(ident))
    }

    fn parse_expr_list(&mut self) -> Option<Vec<CfgExpr>> {
        if self.consume(b')') {
            return Some(Vec::new());
        }
        let mut args = Vec::new();
        loop {
            args.push(self.parse_expr()?);
            if self.consume(b')') {
                return Some(args);
            }
            if !self.consume(b',') {
                return None;
            }
        }
    }

    fn parse_ident(&mut self) -> Option<String> {
        let start = self.index;
        while let Some(byte) = self.peek() {
            if byte.is_ascii_alphanumeric() || byte == b'_' {
                self.index += 1;
            } else {
                break;
            }
        }
        (self.index > start).then(|| self.text[start..self.index].to_string())
    }

    fn parse_string(&mut self) -> Option<String> {
        if !self.consume(b'"') {
            return None;
        }
        let mut value = String::new();
        let mut escaped = false;
        while let Some(byte) = self.peek() {
            self.index += 1;
            let ch = byte as char;
            if escaped {
                value.push(ch);
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                return Some(value);
            } else {
                value.push(ch);
            }
        }
        None
    }

    fn consume(&mut self, expected: u8) -> bool {
        if self.peek() == Some(expected) {
            self.index += 1;
            true
        } else {
            false
        }
    }

    fn peek(&self) -> Option<u8> {
        self.text.as_bytes().get(self.index).copied()
    }
}

fn evaluate_cfg_expr(
    expr: &CfgExpr,
    cfg_set: &RustcCfgSet,
    enabled_features: &BTreeSet<String>,
    package: Option<&str>,
    all_features: bool,
) -> CfgEval {
    match expr {
        CfgExpr::All(args) => {
            CfgEval::and(args.iter().map(|arg| {
                evaluate_cfg_expr(arg, cfg_set, enabled_features, package, all_features)
            }))
        }
        CfgExpr::Any(args) => {
            CfgEval::or(args.iter().map(|arg| {
                evaluate_cfg_expr(arg, cfg_set, enabled_features, package, all_features)
            }))
        }
        CfgExpr::Not(arg) => {
            evaluate_cfg_expr(arg, cfg_set, enabled_features, package, all_features).not()
        }
        CfgExpr::Flag(flag) => evaluate_cfg_flag(flag, cfg_set),
        CfgExpr::KeyValue { key, value } => {
            evaluate_cfg_key_value(key, value, cfg_set, enabled_features, package, all_features)
        }
    }
}

fn evaluate_cfg_flag(flag: &str, cfg_set: &RustcCfgSet) -> CfgEval {
    if cfg_set.flags.contains(flag) {
        CfgEval::True
    } else if known_rustc_boolean_cfg(flag) {
        CfgEval::False
    } else {
        CfgEval::Unknown
    }
}

fn known_rustc_boolean_cfg(flag: &str) -> bool {
    matches!(
        flag,
        "debug_assertions" | "proc_macro" | "target_thread_local" | "unix" | "windows"
    )
}

fn evaluate_cfg_key_value(
    key: &str,
    value: &str,
    cfg_set: &RustcCfgSet,
    enabled_features: &BTreeSet<String>,
    package: Option<&str>,
    all_features: bool,
) -> CfgEval {
    if key == "feature" {
        return if cargo_feature_is_enabled(value, package, enabled_features, all_features) {
            CfgEval::True
        } else {
            CfgEval::Unknown
        };
    }
    if cfg_set
        .values(key)
        .is_some_and(|values| values.contains(value))
    {
        return CfgEval::True;
    }
    if known_rustc_value_cfg(key) {
        CfgEval::False
    } else {
        CfgEval::Unknown
    }
}

fn known_rustc_value_cfg(key: &str) -> bool {
    key == "panic" || key.starts_with("target_")
}

fn cargo_features_are_enabled(
    required_features: &BTreeSet<String>,
    package: Option<&str>,
    enabled_features: &BTreeSet<String>,
    all_features: bool,
) -> bool {
    all_features
        || required_features.iter().all(|feature| {
            cargo_feature_is_enabled(feature, package, enabled_features, all_features)
        })
}

fn cargo_feature_is_enabled(
    feature: &str,
    package: Option<&str>,
    enabled_features: &BTreeSet<String>,
    all_features: bool,
) -> bool {
    all_features
        || enabled_features.contains(feature)
        || package
            .map(|package| enabled_features.contains(&format!("{package}/{feature}")))
            .unwrap_or(false)
}

fn cargo_args_enable_all_features(cargo_args: &[String]) -> bool {
    cargo_args.iter().any(|arg| arg == "--all-features")
}

fn cargo_check_target(cargo_args: &[String]) -> Option<String> {
    let mut args = cargo_args.iter();
    while let Some(arg) = args.next() {
        if arg == "--target" {
            return args.next().cloned();
        }
        if let Some(target) = arg.strip_prefix("--target=") {
            return Some(target.to_string());
        }
    }
    None
}

fn selected_rustc_cfg_set(cargo_args: &[String]) -> Option<RustcCfgSet> {
    if let Some(target) = cargo_check_target(cargo_args) {
        return rustc_print_cfg(Some(&target)).or_else(|| target_triple_cfg_set(&target));
    }
    host_rustc_cfg_set()
}

fn host_rustc_cfg_set() -> Option<RustcCfgSet> {
    static HOST_CFG: OnceLock<Option<RustcCfgSet>> = OnceLock::new();
    HOST_CFG
        .get_or_init(|| rustc_print_cfg(None).or_else(host_const_cfg_set))
        .clone()
}

fn rustc_print_cfg(target: Option<&str>) -> Option<RustcCfgSet> {
    let mut command = Command::new("rustc");
    command.arg("--print").arg("cfg");
    if let Some(target) = target {
        command.arg("--target").arg(target);
    }
    let output = command.output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(parse_rustc_cfg(&String::from_utf8_lossy(&output.stdout)))
}

fn parse_rustc_cfg(text: &str) -> RustcCfgSet {
    let mut set = RustcCfgSet::default();
    for line in text.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if let Some((key, value)) = line.split_once("=\"") {
            if let Some(value) = value.strip_suffix('"') {
                set.insert_value(key, value);
            }
        } else {
            set.flags.insert(line.to_string());
        }
    }
    set
}

fn host_const_cfg_set() -> Option<RustcCfgSet> {
    let mut set = RustcCfgSet::default();
    set.insert_value("target_arch", std::env::consts::ARCH);
    set.insert_value("target_os", std::env::consts::OS);
    set.insert_value("target_family", std::env::consts::FAMILY);
    set.insert_value("target_pointer_width", &usize::BITS.to_string());
    set.insert_value(
        "target_endian",
        if cfg!(target_endian = "big") {
            "big"
        } else {
            "little"
        },
    );
    if cfg!(unix) {
        set.flags.insert("unix".to_string());
    }
    if cfg!(windows) {
        set.flags.insert("windows".to_string());
    }
    Some(set)
}

fn target_triple_cfg_set(target: &str) -> Option<RustcCfgSet> {
    let mut set = RustcCfgSet::default();
    set.insert_value("target_arch", target_arch(target)?);
    if let Some(vendor) = target_vendor(target) {
        set.insert_value("target_vendor", vendor);
    }
    if let Some(env) = target_env(target) {
        set.insert_value("target_env", env);
    }
    if let Some(os) = target_os(target) {
        set.insert_value("target_os", os);
    }
    if let Some(family) = target_family(target) {
        set.insert_value("target_family", family);
        set.flags.insert(family.to_string());
    }
    if let Some(pointer_width) = target_pointer_width(target) {
        set.insert_value("target_pointer_width", pointer_width);
    }
    if let Some(endian) = target_endian(target) {
        set.insert_value("target_endian", endian);
    }
    Some(set)
}

fn target_arch(target: &str) -> Option<&str> {
    target.split('-').next()
}

fn target_vendor(target: &str) -> Option<&str> {
    target.split('-').nth(1)
}

fn target_env(target: &str) -> Option<&str> {
    ["msvc", "gnu", "musl", "sgx", "newlib", "uclibc", "wasi"]
        .into_iter()
        .find(|env| target.split('-').any(|segment| segment == *env))
}

fn target_os(target: &str) -> Option<&'static str> {
    let segments = target.split('-').collect::<Vec<_>>();
    if segments.contains(&"linux") {
        Some("linux")
    } else if segments.contains(&"darwin") {
        Some("macos")
    } else if segments.contains(&"windows") {
        Some("windows")
    } else if segments.contains(&"android") {
        Some("android")
    } else if segments.contains(&"ios") {
        Some("ios")
    } else if segments.contains(&"freebsd") {
        Some("freebsd")
    } else if segments.contains(&"netbsd") {
        Some("netbsd")
    } else if segments.contains(&"openbsd") {
        Some("openbsd")
    } else if segments.contains(&"unknown") {
        Some("unknown")
    } else {
        None
    }
}

fn target_family(target: &str) -> Option<&'static str> {
    match target_os(target) {
        Some("windows") => Some("windows"),
        Some("linux" | "macos" | "android" | "ios" | "freebsd" | "netbsd" | "openbsd") => {
            Some("unix")
        }
        _ => None,
    }
}

fn target_pointer_width(target: &str) -> Option<&'static str> {
    let arch = target_arch(target)?;
    if arch.contains("64") || matches!(arch, "s390x") {
        Some("64")
    } else if arch.contains("32")
        || matches!(
            arch,
            "x86" | "i386" | "i586" | "i686" | "arm" | "thumb" | "mips"
        )
    {
        Some("32")
    } else {
        None
    }
}

fn target_endian(target: &str) -> Option<&'static str> {
    let arch = target_arch(target)?;
    if matches!(
        arch,
        "s390x" | "powerpc" | "powerpc64" | "mips" | "mips64" | "sparc"
    ) {
        Some("big")
    } else {
        Some("little")
    }
}

fn record_final_production_readiness(options: &CliOptions, validation: &mut ValidationReport) {
    if !options.production_preset {
        return;
    }
    if validation
        .gates
        .iter()
        .any(|gate| gate.name == "production_ready")
    {
        return;
    }

    let feedback_status = validation
        .gates
        .iter()
        .rev()
        .find(|gate| matches!(gate.name.as_str(), "feedback-repair" | "feedback"))
        .map(|gate| gate.status.as_str())
        .unwrap_or("missing_feedback");
    let production_readiness_status = validation
        .gates
        .iter()
        .rev()
        .find(|gate| gate.name == "production_readiness")
        .map(|gate| gate.status.as_str())
        .unwrap_or("missing_production_readiness");
    let review_warning_hazards = validation
        .gates
        .iter()
        .rev()
        .find(|gate| gate.name == "production_readiness")
        .and_then(|gate| gate.review_warning_hazards)
        .unwrap_or(usize::MAX);
    let production_matrix_status = validation
        .gates
        .iter()
        .rev()
        .find(|gate| gate.name == "production_matrix")
        .map(|gate| gate.status.as_str())
        .unwrap_or("not_required");
    let (status, reason) = match feedback_status {
        _ if production_readiness_status == "hazards_detected" => (
            "failed",
            "production readiness reported error hazards for the final generated slice",
        ),
        _ if production_matrix_status == "failed" => (
            "failed",
            "production matrix validation failed for the final generated slice",
        ),
        _ if production_matrix_status == "baseline_limited" || feedback_status == "baseline_limited" => (
            "baseline_limited",
            "production preset matched an allowed failing source baseline; generated workspace is not cleanly production-ready",
        ),
        "accepted"
            if production_readiness_status == "ready_for_feedback"
                && matches!(production_matrix_status, "accepted" | "not_required") =>
        {
            (
            "accepted",
            "production preset passed baseline, generation, preflight, target coverage, compiler feedback, and production hazard checks",
            )
        }
        "accepted"
            if review_warning_hazards == 0
                && matches!(production_matrix_status, "accepted" | "not_required") =>
        {
            (
                "accepted",
                "production preset passed compiler feedback and all remaining production warnings were discharged by feedback or matrix validation",
            )
        }
        "accepted" if matches!(production_matrix_status, "accepted" | "not_required") => (
            "review_required",
            "production preset passed compiler feedback, but remaining warning hazards require semantic review before production-ready acceptance",
        ),
        _ => (
            "failed",
            "production preset did not produce an accepted compiler feedback gate",
        ),
    };
    validation.gates.push(ValidationGateReport {
        name: "production_ready".to_string(),
        status: status.to_string(),
        reason: reason.to_string(),
        report_path: validation_report_path(options),
        error_count: None,
        warning_count: None,
        semantic_warning_hazards: None,
        review_warning_hazards: None,
    });
}

fn baseline_target_dir(options: &CliOptions) -> PathBuf {
    options.baseline_target_dir.clone().unwrap_or_else(|| {
        shared_baseline_target_dir(&options.workspace_root, &options.cargo_check_args)
    })
}

fn shared_baseline_target_dir(workspace_root: &Path, cargo_args: &[String]) -> PathBuf {
    let mut key_parts = Vec::with_capacity(cargo_args.len() + 2);
    key_parts.push("v1".to_string());
    key_parts.push(cache_key_path_component(workspace_root));
    key_parts.extend(cargo_args.iter().cloned());
    let key = stable_hex_hash(&key_parts);
    std::env::temp_dir()
        .join("slicers-baseline-targets")
        .join(key)
}

fn cache_key_path_component(path: &Path) -> String {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|current| current.join(path))
            .unwrap_or_else(|_| path.to_path_buf())
    };
    fs::canonicalize(&absolute)
        .unwrap_or(absolute)
        .to_string_lossy()
        .into_owned()
}

fn stable_hex_hash(parts: &[String]) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for part in parts {
        for byte in part.as_bytes().iter().chain(std::iter::once(&0)) {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    format!("{hash:016x}")
}

fn sibling_output_path(output_root: &Path, suffix: &str) -> PathBuf {
    let Some(name) = output_root.file_name() else {
        return output_root.join(format!(".{suffix}"));
    };
    output_root.with_file_name(format!("{}-{suffix}", name.to_string_lossy()))
}

fn feedback_is_accepted(
    report: &CheckReport,
    baseline: Option<&CheckReport>,
    deny_warnings: bool,
) -> bool {
    report.success
        && semantic_hazard_warning_count(&report.diagnostics, baseline) == 0
        && (!deny_warnings || report.warning_count() == 0)
}

fn feedback_repair_is_accepted(
    report: &CheckReport,
    baseline: Option<&CheckReport>,
    deny_warnings: bool,
) -> bool {
    feedback_is_accepted(report, baseline, deny_warnings)
        && repairable_warning_count(&report.diagnostics) == 0
}

fn repair_has_deferred_warning_allows(report: &RepairReport) -> bool {
    report.deferred_dead_code_allows > 0 || report.deferred_lint_allows > 0
}

fn should_run_deferred_warning_repair(
    report: &CheckReport,
    baseline: Option<&CheckReport>,
    deny_warnings: bool,
    saw_deferred_warning_allows: bool,
) -> bool {
    deny_warnings
        && saw_deferred_warning_allows
        && report.error_count() == 0
        && report.warning_count() > 0
        && semantic_hazard_warning_count(&report.diagnostics, baseline) == 0
}

fn baseline_limited_feedback_is_accepted(
    report: &CheckReport,
    baseline: Option<&CheckReport>,
    deny_warnings: bool,
) -> bool {
    feedback_errors_are_baseline_known(report, baseline)
        && semantic_hazard_warning_count(&report.diagnostics, baseline) == 0
        && (!deny_warnings || report.warning_count() == 0)
}

fn baseline_limited_feedback_repair_is_accepted(
    report: &CheckReport,
    baseline: Option<&CheckReport>,
    deny_warnings: bool,
) -> bool {
    baseline_limited_feedback_is_accepted(report, baseline, deny_warnings)
        && repairable_warning_count(&report.diagnostics) == 0
}

fn feedback_errors_are_baseline_known(
    report: &CheckReport,
    baseline: Option<&CheckReport>,
) -> bool {
    let Some(baseline) = baseline else {
        return false;
    };
    if baseline.success || report.success {
        return false;
    }

    let baseline_errors = diagnostic_error_keys(&baseline.diagnostics);
    if baseline_errors.is_empty() {
        return false;
    }
    let generated_errors = diagnostic_error_keys(&report.diagnostics);
    !generated_errors.is_empty()
        && generated_errors
            .iter()
            .all(|error| baseline_errors.contains(error))
}

fn diagnostic_error_keys(diagnostics: &[CheckDiagnostic]) -> BTreeSet<String> {
    diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.level == "error")
        .map(diagnostic_baseline_key)
        .collect()
}

fn diagnostic_baseline_key(diagnostic: &CheckDiagnostic) -> String {
    let target = diagnostic
        .target
        .as_ref()
        .map(|target| format!("{}:{}", target.kind.join(","), target.name))
        .unwrap_or_default();
    let primary_files = diagnostic_primary_files(diagnostic);
    let rendered_fingerprint = if primary_files.is_empty() {
        diagnostic
            .rendered
            .as_deref()
            .map(stable_text_fingerprint)
            .unwrap_or_default()
    } else {
        String::new()
    };
    format!(
        "{}|{}|{}|{}|{}|{}|{}",
        diagnostic.level,
        diagnostic.code.as_deref().unwrap_or(""),
        diagnostic.package_id.as_deref().unwrap_or(""),
        target,
        primary_files,
        rendered_fingerprint,
        diagnostic.message
    )
}

fn diagnostic_primary_files(diagnostic: &CheckDiagnostic) -> String {
    let mut files = diagnostic
        .spans
        .iter()
        .filter(|span| span.is_primary)
        .map(|span| span.file_name.as_str())
        .collect::<Vec<_>>();
    files.sort_unstable();
    files.dedup();
    files.join(",")
}

fn stable_text_fingerprint(text: &str) -> String {
    let mut hash = 14_695_981_039_346_656_037u64;
    for byte in text.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(1_099_511_628_211);
    }
    format!("{hash:016x}")
}

fn semantic_hazard_warning_count(
    diagnostics: &[CheckDiagnostic],
    baseline: Option<&CheckReport>,
) -> usize {
    let baseline_warnings = baseline
        .map(|report| diagnostic_semantic_warning_keys(&report.diagnostics))
        .unwrap_or_default();
    diagnostics
        .iter()
        .filter(|diagnostic| semantic_warning_is_hazard(diagnostic))
        .filter(|diagnostic| !baseline_warnings.contains(&diagnostic_baseline_key(diagnostic)))
        .count()
}

fn diagnostic_semantic_warning_keys(diagnostics: &[CheckDiagnostic]) -> BTreeSet<String> {
    diagnostics
        .iter()
        .filter(|diagnostic| semantic_warning_is_hazard(diagnostic))
        .map(diagnostic_baseline_key)
        .collect()
}

fn semantic_warning_is_hazard(diagnostic: &CheckDiagnostic) -> bool {
    if diagnostic.level != "warning" {
        return false;
    }
    matches!(
        diagnostic.code.as_deref(),
        Some("unreachable_patterns" | "irrefutable_let_patterns" | "bindings_with_variant_name")
    ) || (diagnostic.code.as_deref() == Some("non_snake_case")
        && diagnostic.message.contains("variable `"))
        || diagnostic.message.contains("unreachable pattern")
        || diagnostic.message.contains("irrefutable")
}

fn repairable_warning_count(diagnostics: &[CheckDiagnostic]) -> usize {
    diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.level == "warning"
                && (diagnostic.code.as_deref() == Some("unused_imports")
                    || diagnostic.code.as_deref() == Some("unused_macros")
                    || (diagnostic.code.as_deref() == Some("dead_code")
                        && dead_code_warning_is_repairable(&diagnostic.message)))
        })
        .count()
}

fn dead_code_warning_is_repairable(message: &str) -> bool {
    message.contains("function `")
        || message.contains("method `")
        || message.contains("associated function `")
        || message.contains("associated items `")
        || message.contains("associated constant `")
        || message.contains("constant `")
        || message.contains("enum `")
        || message.contains("module `")
        || message.contains("static `")
        || message.contains("struct `")
        || message.contains("type alias `")
        || message.contains("union `")
        || message.contains("trait `")
        || message.contains("field `")
        || message.contains("fields `")
        || message.contains("variant `")
        || message.contains("variants `")
}

fn diagnostics_signature(diagnostics: &[CheckDiagnostic]) -> String {
    let mut parts = diagnostics
        .iter()
        .map(|diagnostic| {
            let spans = diagnostic
                .spans
                .iter()
                .filter(|span| span.is_primary)
                .map(|span| {
                    format!(
                        "{}:{}:{}:{}",
                        span.file_name, span.line_start, span.column_start, span.column_end
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "{}|{}|{}|{}",
                diagnostic.level,
                diagnostic.code.as_deref().unwrap_or(""),
                diagnostic.message,
                spans
            )
        })
        .collect::<Vec<_>>();
    parts.sort();
    parts.join("\n")
}

fn diagnostics_shape_signature(diagnostics: &[CheckDiagnostic]) -> String {
    let mut parts = diagnostics
        .iter()
        .map(|diagnostic| {
            let mut files = diagnostic
                .spans
                .iter()
                .filter(|span| span.is_primary)
                .map(|span| span.file_name.as_str())
                .collect::<Vec<_>>();
            files.sort_unstable();
            files.dedup();
            let target = diagnostic
                .target
                .as_ref()
                .map(|target| format!("{}:{}", target.kind.join("+"), target.name))
                .unwrap_or_default();
            format!(
                "{}|{}|{}|{}|{}|{}",
                diagnostic.level,
                diagnostic.code.as_deref().unwrap_or(""),
                diagnostic.package_id.as_deref().unwrap_or(""),
                target,
                diagnostic.message,
                files.join(",")
            )
        })
        .collect::<Vec<_>>();
    parts.sort();
    parts.join("\n")
}

fn print_preflight(report: &PreflightReport, limit: usize, report_path: &Path) {
    if report.success {
        println!(
            "preflight: passed; packages={}, rust_files={}, local_path_deps={}, external_deps={}, build_scripts={}; report: {}",
            report.packages,
            report.rust_files,
            report.local_path_dependencies,
            report.external_dependencies,
            report.build_scripts,
            report_path.display()
        );
        return;
    }

    println!(
        "preflight: failed with {} error(s), {} warning(s); report: {}",
        report.error_count(),
        report.warning_count(),
        report_path.display()
    );

    for diagnostic in prioritized_preflight_diagnostics(&report.diagnostics)
        .into_iter()
        .take(limit)
    {
        print_preflight_diagnostic(diagnostic);
    }
}

fn prioritized_preflight_diagnostics(
    diagnostics: &[PreflightDiagnostic],
) -> Vec<&PreflightDiagnostic> {
    let mut prioritized = diagnostics.iter().collect::<Vec<_>>();
    prioritized.sort_by_key(|diagnostic| match diagnostic.level.as_str() {
        "error" => 0,
        "warning" => 1,
        _ => 2,
    });
    prioritized
}

fn print_preflight_diagnostic(diagnostic: &PreflightDiagnostic) {
    println!(
        "  {}[{}]: {}",
        diagnostic.level, diagnostic.code, diagnostic.message
    );
    if let Some(path) = &diagnostic.path {
        println!("    at {}", path.display());
    }
}

fn print_baseline(report: &CheckReport, limit: usize, report_path: Option<&Path>) {
    let report_text = report_path
        .map(|path| format!("; report: {}", path.display()))
        .unwrap_or_default();
    if report.success {
        println!(
            "baseline: source cargo check passed with {} warning(s) in {}{}",
            report.warning_count(),
            format_duration_ms(report.duration_ms),
            report_text
        );
        return;
    }

    println!(
        "baseline: source cargo check failed with {} error(s), {} warning(s) in {}{}",
        report.error_count(),
        report.warning_count(),
        format_duration_ms(report.duration_ms),
        report_text
    );

    for diagnostic in prioritized_diagnostics(&report.diagnostics)
        .into_iter()
        .take(limit)
    {
        print_diagnostic(diagnostic);
    }
}

fn print_feedback(report: &CheckReport, limit: usize, report_path: &Path) {
    if report.success {
        println!(
            "feedback: cargo check passed with {} warning(s) in {}; report: {}",
            report.warning_count(),
            format_duration_ms(report.duration_ms),
            report_path.display()
        );
        return;
    }

    println!(
        "feedback: cargo check failed with {} error(s), {} warning(s), {} widening candidate(s), {} hazard(s) in {}; report: {}",
        report.error_count(),
        report.warning_count(),
        report.widening.candidates.len(),
        report.widening.hazards.len(),
        format_duration_ms(report.duration_ms),
        report_path.display()
    );

    for candidate in report.widening.candidates.iter().take(limit.min(4)) {
        let location = candidate
            .file_name
            .as_deref()
            .zip(candidate.line_start)
            .map(|(file, line)| format!(" at {file}:{line}"))
            .unwrap_or_default();
        let symbol = candidate
            .symbol
            .as_deref()
            .map(|symbol| format!(" `{symbol}`"))
            .unwrap_or_default();
        let suggestions = if candidate.machine_applicable_suggestions > 0 {
            format!(
                " ({} machine-applicable suggestion(s))",
                candidate.machine_applicable_suggestions
            )
        } else if candidate.suggestions > 0 {
            format!(" ({} compiler suggestion(s))", candidate.suggestions)
        } else {
            String::new()
        };
        println!(
            "  widening {}{}{}{}: {}",
            candidate.kind, symbol, location, suggestions, candidate.action
        );
    }

    for diagnostic in prioritized_diagnostics(&report.diagnostics)
        .into_iter()
        .take(limit)
    {
        print_diagnostic(diagnostic);
    }
}

fn prioritized_diagnostics(diagnostics: &[CheckDiagnostic]) -> Vec<&CheckDiagnostic> {
    let mut prioritized = diagnostics.iter().collect::<Vec<_>>();
    prioritized.sort_by_key(|diagnostic| match diagnostic.level.as_str() {
        "error" => 0,
        "warning" => 1,
        _ => 2,
    });
    prioritized
}

fn print_diagnostic(diagnostic: &CheckDiagnostic) {
    let code = diagnostic
        .code
        .as_deref()
        .map(|code| format!("[{code}]"))
        .unwrap_or_default();
    println!("  {}{}: {}", diagnostic.level, code, diagnostic.message);

    for span in diagnostic
        .spans
        .iter()
        .filter(|span| span.is_primary)
        .take(2)
    {
        println!(
            "    at {}:{}:{}",
            span.file_name, span.line_start, span.column_start
        );
        if let Some(line) = span.text.first() {
            println!("    | {line}");
        }
    }
}

fn format_duration_ms(duration_ms: u64) -> String {
    if duration_ms < 1_000 {
        format!("{duration_ms}ms")
    } else {
        format!("{:.2}s", duration_ms as f64 / 1_000.0)
    }
}

fn usage() -> String {
    concat!(
        "usage: slicers [--analyzer <syn|ra-hir|ra-feedback|ra-hir-proc-macros>] [--production] [--check] [--preflight] [--feedback] ",
        "[--feedback-loop <n>] [--feedback-repair-loop <n>] [--feedback-limit <n>] ",
        "[--feedback-timeout <seconds>] [--deny-warnings] [--feedback-report <path>] ",
        "[--feedback-target-dir <path>] [--cargo-check-arg <arg>] [--repair-report <path>] ",
        "[--baseline-check] [--allow-baseline-failures] [--baseline-report <path>] ",
        "[--baseline-target-dir <path>] [--slice-report <path>] [--decision-log <path>] [--validation-report <path>] ",
        "[--preflight-report <path>] [--root <selector>] [--roots-file <path>] ",
        "[--random-roots <n>] [--random-root-package <package>] [--random-seed <n>] [--batch-roots] [--batch-report <path>] ",
        "<workspace-root-or-Cargo.toml> <output-root>\n",
        "default analyzer: ra-hir when the binary is built with the ra-hir feature, otherwise syn; ",
        "ra-feedback exposes the bounded RA outgoing-call closure explicitly; ",
        "--production defaults to ra-hir-proc-macros with RA feedback closure when available; ",
        "--root selects functions/items in memory without editing source, while #[opensourced] roots still work"
    )
    .to_string()
}

fn same_path(left: &Path, right: &Path) -> bool {
    let Ok(left) = left.canonicalize() else {
        return false;
    };
    let Ok(right) = right.canonicalize() else {
        return false;
    };
    left == right
}

#[cfg(test)]
mod tests {
    use std::{ffi::OsString, fs, path::PathBuf};

    use opensource_core::{
        AnalyzerMode, CheckDiagnostic, CheckReport, CheckTarget, FeedbackWideningReport,
        GeneratedTargetReport, SemanticReport,
    };

    use super::{
        apply_default_marked_package_scope, baseline_limited_feedback_is_accepted,
        baseline_target_dir, cargo_args_have_package_scope, decision_log_path,
        diagnostics_shape_signature, diagnostics_signature, feedback_errors_are_baseline_known,
        feedback_is_accepted, feedback_repair_is_accepted, parse_args_from,
        production_readiness_blocks_validation, production_validation_matrix_entries,
        record_final_production_readiness, record_production_readiness_gate,
        refresh_generated_lockfile_for_locked_validation, run_batch_roots, run_plain_check_gate,
        semantic_hazard_warning_count, semantic_proof_block_reason, semantic_proof_status,
        should_run_deferred_warning_repair, slice_report_path, try_widen_from_feedback,
        uncovered_validation_targets, validation_report_path, FeedbackWideningState,
        ValidationGateReport, ValidationReport,
    };

    #[test]
    fn semantic_proof_status_allows_selected_root_complete_with_workspace_budget_limits() {
        let semantic = SemanticReport {
            source_files: 120,
            analyzed_files: 48,
            skipped_files: 72,
            unqueried_method_calls: 1301,
            unqueried_paths: 10933,
            selected_root_source_files: 1,
            selected_root_analyzed_files: 1,
            ..SemanticReport::default()
        };

        assert_eq!(
            semantic_proof_status(&semantic),
            "selected_root_complete_workspace_limited"
        );
        assert!(semantic_proof_block_reason(Some(&semantic)).is_none());
    }

    #[test]
    fn semantic_proof_status_blocks_selected_root_budget_limits() {
        let semantic = SemanticReport {
            source_files: 7,
            analyzed_files: 7,
            selected_root_source_files: 1,
            selected_root_analyzed_files: 1,
            selected_root_unqueried_paths: 3,
            ..SemanticReport::default()
        };

        assert_eq!(semantic_proof_status(&semantic), "selected_root_limited");
        assert!(semantic_proof_block_reason(Some(&semantic))
            .expect("selected-root limits should block production")
            .contains("selected root semantic proof is budget-limited"));
    }

    #[test]
    fn semantic_proof_status_blocks_missing_ra_semantics() {
        assert_eq!(
            semantic_proof_block_reason(None).as_deref(),
            Some("rust-analyzer semantic proof is not available")
        );
    }

    #[test]
    fn accepts_warning_bearing_feedback_only_when_warning_denial_is_disabled() {
        let clean = report(true, Vec::new());
        let warning_report = report(false, vec![warning("unused variable: `value`")]);
        let mut successful_with_warning = warning_report.clone();
        successful_with_warning.success = true;

        assert!(feedback_is_accepted(&clean, None, true));
        assert!(feedback_is_accepted(&successful_with_warning, None, false));
        assert!(!feedback_is_accepted(&successful_with_warning, None, true));
    }

    #[test]
    fn repair_loop_does_not_accept_repairable_warnings_before_cleanup() {
        let report = report(
            true,
            vec![warning_with_code(
                "unused_imports",
                "unused import: `std::fmt`",
            )],
        );

        assert!(feedback_is_accepted(&report, None, false));
        assert!(!feedback_repair_is_accepted(&report, None, false));
    }

    #[test]
    fn deferred_warning_repair_runs_only_for_denied_warning_only_reports() {
        let warning_report = report(
            true,
            vec![warning_with_code(
                "private_interfaces",
                "type `Private` is more private than the item `Public::field`",
            )],
        );
        let error_report = report(
            false,
            vec![
                diagnostic("E0425", "cannot find value `missing` in this scope"),
                warning_with_code(
                    "private_interfaces",
                    "type `Private` is more private than the item `Public::field`",
                ),
            ],
        );
        let semantic_warning_report = report(
            true,
            vec![warning_with_code(
                "unreachable_patterns",
                "unreachable pattern",
            )],
        );

        assert!(should_run_deferred_warning_repair(
            &warning_report,
            None,
            true,
            true
        ));
        assert!(!should_run_deferred_warning_repair(
            &warning_report,
            None,
            false,
            true
        ));
        assert!(!should_run_deferred_warning_repair(
            &warning_report,
            None,
            true,
            false
        ));
        assert!(!should_run_deferred_warning_repair(
            &error_report,
            None,
            true,
            true
        ));
        assert!(!should_run_deferred_warning_repair(
            &semantic_warning_report,
            None,
            true,
            true
        ));
    }

    #[test]
    fn rejects_new_semantic_warning_even_when_warning_denial_is_disabled() {
        let report = report(
            true,
            vec![warning_with_code(
                "unreachable_patterns",
                "unreachable pattern",
            )],
        );

        assert_eq!(semantic_hazard_warning_count(&report.diagnostics, None), 1);
        assert!(!feedback_is_accepted(&report, None, false));
    }

    #[test]
    fn rejects_const_pattern_binding_warning_as_semantic_hazard() {
        let report = report(
            true,
            vec![warning_with_code(
                "non_snake_case",
                "variable `HTTP_OK` should have a snake case name",
            )],
        );

        assert_eq!(semantic_hazard_warning_count(&report.diagnostics, None), 1);
        assert!(!feedback_is_accepted(&report, None, false));
    }

    #[test]
    fn accepts_semantic_warning_already_present_in_source_baseline() {
        let baseline = report(
            true,
            vec![warning_with_code(
                "unreachable_patterns",
                "unreachable pattern",
            )],
        );
        let generated = report(
            true,
            vec![warning_with_code(
                "unreachable_patterns",
                "unreachable pattern",
            )],
        );

        assert_eq!(
            semantic_hazard_warning_count(&generated.diagnostics, Some(&baseline)),
            0
        );
        assert!(feedback_is_accepted(&generated, Some(&baseline), false));
    }

    #[test]
    fn recognizes_generated_errors_present_in_failed_source_baseline() {
        let baseline = report(
            false,
            vec![diagnostic("E0425", "cannot find value `x` in this scope")],
        );
        let generated = report(
            false,
            vec![diagnostic("E0425", "cannot find value `x` in this scope")],
        );

        assert!(feedback_errors_are_baseline_known(
            &generated,
            Some(&baseline)
        ));
    }

    #[test]
    fn baseline_limited_feedback_still_honors_warning_denial() {
        let baseline = report(
            false,
            vec![diagnostic("E0425", "cannot find value `x` in this scope")],
        );
        let generated = report(
            false,
            vec![
                diagnostic("E0425", "cannot find value `x` in this scope"),
                warning("unused variable: `value`"),
            ],
        );

        assert!(baseline_limited_feedback_is_accepted(
            &generated,
            Some(&baseline),
            false
        ));
        assert!(!baseline_limited_feedback_is_accepted(
            &generated,
            Some(&baseline),
            true
        ));
    }

    #[test]
    fn rejects_generated_errors_not_present_in_source_baseline() {
        let baseline = report(
            false,
            vec![diagnostic("E0425", "cannot find value `x` in this scope")],
        );
        let generated = report(
            false,
            vec![diagnostic("E0432", "unresolved import `crate::missing`")],
        );

        assert!(!feedback_errors_are_baseline_known(
            &generated,
            Some(&baseline)
        ));
    }

    #[test]
    fn baseline_matching_keeps_package_and_target_boundaries() {
        let baseline = report(
            false,
            vec![diagnostic_for_target(
                "E0425",
                "cannot find value `x` in this scope",
                "pkg_a 0.1.0",
                "lib",
                "pkg_a",
            )],
        );
        let generated = report(
            false,
            vec![diagnostic_for_target(
                "E0425",
                "cannot find value `x` in this scope",
                "pkg_b 0.1.0",
                "bin",
                "pkg_b",
            )],
        );

        assert!(!feedback_errors_are_baseline_known(
            &generated,
            Some(&baseline)
        ));
    }

    #[test]
    fn baseline_matching_keeps_primary_span_files() {
        let baseline = report(
            false,
            vec![diagnostic_with_span(
                "E0425",
                "cannot find value `x` in this scope",
                "src/lib.rs",
                12,
                9,
                10,
            )],
        );
        let generated = report(
            false,
            vec![diagnostic_with_span(
                "E0425",
                "cannot find value `x` in this scope",
                "src/main.rs",
                12,
                9,
                10,
            )],
        );

        assert!(!feedback_errors_are_baseline_known(
            &generated,
            Some(&baseline)
        ));
    }

    #[test]
    fn baseline_matching_distinguishes_stderr_only_failures() {
        let baseline = report(
            false,
            vec![diagnostic_with_rendered(
                "cargo-stderr",
                "error: failed to select a version",
                "error: failed to select a version\ncandidate A\n",
            )],
        );
        let generated = report(
            false,
            vec![diagnostic_with_rendered(
                "cargo-stderr",
                "error: failed to select a version",
                "error: failed to select a version\ncandidate B\n",
            )],
        );

        assert!(!feedback_errors_are_baseline_known(
            &generated,
            Some(&baseline)
        ));
    }

    #[test]
    fn diagnostic_signatures_are_order_insensitive() {
        let left = vec![
            diagnostic("E0432", "unresolved import `crate::missing`"),
            diagnostic("E0425", "cannot find value `x` in this scope"),
        ];
        let right = vec![
            diagnostic("E0425", "cannot find value `x` in this scope"),
            diagnostic("E0432", "unresolved import `crate::missing`"),
        ];

        assert_eq!(diagnostics_signature(&left), diagnostics_signature(&right));
    }

    #[test]
    fn diagnostic_shape_signatures_ignore_line_churn_but_keep_files() {
        let original = vec![diagnostic_with_span(
            "E0425",
            "cannot find value `x` in this scope",
            "src/lib.rs",
            12,
            9,
            10,
        )];
        let shifted = vec![diagnostic_with_span(
            "E0425",
            "cannot find value `x` in this scope",
            "src/lib.rs",
            20,
            5,
            6,
        )];
        let other_file = vec![diagnostic_with_span(
            "E0425",
            "cannot find value `x` in this scope",
            "src/main.rs",
            20,
            5,
            6,
        )];

        assert_ne!(
            diagnostics_signature(&original),
            diagnostics_signature(&shifted)
        );
        assert_eq!(
            diagnostics_shape_signature(&original),
            diagnostics_shape_signature(&shifted)
        );
        assert_ne!(
            diagnostics_shape_signature(&original),
            diagnostics_shape_signature(&other_file)
        );
    }

    #[test]
    fn production_flag_enables_strict_validation_preset() {
        let options = parse_options(["--production", "workspace", "out"]);

        assert!(options.run_baseline_check);
        assert!(options.run_preflight);
        assert!(options.deny_warnings);
        assert!(options.production_preset);
        assert_eq!(options.feedback_repair_iterations, 3);
        assert_eq!(options.feedback_iterations, 0);
        assert_eq!(
            options.analyzer_mode,
            AnalyzerMode::production_default_for_build()
        );
        assert_eq!(
            slice_report_path(&options),
            Some(PathBuf::from("out/slice-report.json"))
        );
        assert_eq!(
            validation_report_path(&options),
            Some(PathBuf::from("out/slice-validation.json"))
        );
        assert_eq!(
            decision_log_path(&options),
            Some(PathBuf::from("out/slice-decision-log.json"))
        );
        assert_eq!(options.workspace_root, PathBuf::from("workspace"));
        assert_eq!(options.output_root, PathBuf::from("out"));
    }

    #[test]
    fn default_baseline_target_is_shared_for_same_source_workspace() {
        let first = parse_options(["--production", "workspace", "out-a"]);
        let second = parse_options(["--production", "workspace", "out-b"]);

        assert_eq!(baseline_target_dir(&first), baseline_target_dir(&second));
        assert!(baseline_target_dir(&first)
            .to_string_lossy()
            .contains("slicers-baseline-targets"));
    }

    #[test]
    fn baseline_target_cache_key_includes_cargo_check_args() {
        let default_args = parse_options(["--production", "workspace", "out"]);
        let feature_args = parse_options([
            "--production",
            "--cargo-check-arg",
            "--features",
            "--cargo-check-arg",
            "mobile",
            "workspace",
            "out",
        ]);

        assert_ne!(
            baseline_target_dir(&default_args),
            baseline_target_dir(&feature_args)
        );
    }

    #[test]
    fn explicit_baseline_target_dir_overrides_shared_default() {
        let options = parse_options([
            "--production",
            "--baseline-target-dir",
            "custom-target",
            "workspace",
            "out",
        ]);

        assert_eq!(
            baseline_target_dir(&options),
            PathBuf::from("custom-target")
        );
    }

    #[test]
    fn default_analyzer_matches_build_features() {
        let options = parse_options(["workspace", "out"]);

        assert_eq!(options.analyzer_mode, AnalyzerMode::default_for_build());
        #[cfg(feature = "ra-hir")]
        assert_eq!(options.analyzer_mode, AnalyzerMode::RustAnalyzerHir);
        #[cfg(not(feature = "ra-hir"))]
        assert_eq!(options.analyzer_mode, AnalyzerMode::Syn);
    }

    #[test]
    fn explicit_syn_analyzer_overrides_semantic_default() {
        let options = parse_options(["--analyzer", "syn", "workspace", "out"]);

        assert_eq!(options.analyzer_mode, AnalyzerMode::Syn);
    }

    #[test]
    fn explicit_ra_feedback_analyzer_is_accepted() {
        let options = parse_options(["--analyzer", "ra-feedback", "workspace", "out"]);

        assert_eq!(options.analyzer_mode, AnalyzerMode::RustAnalyzerFeedback);
    }

    #[test]
    fn explicit_syn_analyzer_overrides_production_semantic_default() {
        let options = parse_options(["--production", "--analyzer", "syn", "workspace", "out"]);

        assert_eq!(options.analyzer_mode, AnalyzerMode::Syn);
        assert!(options.production_preset);
    }

    #[test]
    fn default_validation_scope_targets_marked_packages() {
        let workspace = temp_path("cli-marked-package-scope-source");
        let output = temp_path("cli-marked-package-scope-output");
        write(
            workspace.join("Cargo.toml"),
            r#"[workspace]
members = ["app", "unrelated"]
resolver = "2"
"#,
        );
        write(
            workspace.join("app/Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        write(
            workspace.join("app/src/lib.rs"),
            "#[opensourced::opensourced]\npub fn selected() -> i32 { 1 }\n",
        );
        write(
            workspace.join("unrelated/Cargo.toml"),
            "[package]\nname = \"unrelated\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        write(
            workspace.join("unrelated/src/lib.rs"),
            "compile_error!(\"unrelated package should not define validation scope\");\n",
        );
        let mut options = parse_args_from([
            std::ffi::OsString::from("--production"),
            workspace.clone().into_os_string(),
            output.into_os_string(),
        ])
        .expect("production arguments should parse");

        apply_default_marked_package_scope(&mut options)
            .expect("marked package scope should resolve");

        assert_eq!(options.cargo_check_args, ["-p", "app"]);
    }

    #[test]
    fn default_validation_scope_respects_explicit_package_selection() {
        let mut options = parse_options([
            "--production",
            "--cargo-check-arg",
            "--workspace",
            "workspace",
            "out",
        ]);
        let original_args = options.cargo_check_args.clone();

        apply_default_marked_package_scope(&mut options)
            .expect("explicit workspace scope should skip marker scan");

        assert_eq!(options.cargo_check_args, original_args);
    }

    #[test]
    fn cargo_arg_package_scope_detection_accepts_common_forms() {
        assert!(cargo_args_have_package_scope(&["-p".to_string()]));
        assert!(cargo_args_have_package_scope(&["-papp".to_string()]));
        assert!(cargo_args_have_package_scope(
            &["--package=app".to_string()]
        ));
        assert!(cargo_args_have_package_scope(&["--workspace".to_string()]));
        assert!(!cargo_args_have_package_scope(&[
            "--all-targets".to_string()
        ]));
    }

    #[test]
    fn production_preset_locks_existing_source_lockfile() {
        let workspace = temp_path("cli-production-locked-source");
        let output = temp_path("cli-production-locked-output");
        write(workspace.join("Cargo.toml"), "[workspace]\nmembers = []\n");
        write(workspace.join("Cargo.lock"), "# lockfile\n");

        let options = parse_args_from(vec![
            std::ffi::OsString::from("--production"),
            workspace.into_os_string(),
            output.into_os_string(),
        ])
        .expect("arguments should parse");

        assert!(options.cargo_check_args.iter().any(|arg| arg == "--locked"));
    }

    #[test]
    fn production_preset_does_not_duplicate_locking_args() {
        let workspace = temp_path("cli-production-frozen-source");
        let output = temp_path("cli-production-frozen-output");
        write(workspace.join("Cargo.toml"), "[workspace]\nmembers = []\n");
        write(workspace.join("Cargo.lock"), "# lockfile\n");

        let options = parse_args_from(vec![
            std::ffi::OsString::from("--production"),
            std::ffi::OsString::from("--cargo-check-arg"),
            std::ffi::OsString::from("--frozen"),
            workspace.into_os_string(),
            output.into_os_string(),
        ])
        .expect("arguments should parse");

        assert_eq!(options.cargo_check_args, ["--frozen"]);
    }

    #[test]
    fn locked_validation_reconciles_generated_lockfile_before_check() {
        let source = temp_path("cli-lockfile-source");
        let output = temp_path("cli-lockfile-output");
        write(
            source.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\n",
        );
        write(source.join("Cargo.lock"), stale_generated_lockfile());
        write(
            output.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\n",
        );
        write(
            output.join("app/Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        write(output.join("app/src/lib.rs"), "");
        write(output.join("Cargo.lock"), stale_generated_lockfile());

        let options = parse_args_from(vec![
            std::ffi::OsString::from("--production"),
            source.into_os_string(),
            output.clone().into_os_string(),
        ])
        .expect("arguments should parse");
        let mut validation = ValidationReport::new(&options);

        refresh_generated_lockfile_for_locked_validation(&options, &mut validation)
            .expect("generated lockfile should reconcile");

        let lockfile = fs::read_to_string(output.join("Cargo.lock")).unwrap();
        assert!(!lockfile.contains("\"dead-helper\""));
        let gate = validation
            .gates
            .iter()
            .find(|gate| gate.name == "lockfile")
            .expect("lockfile gate should be recorded");
        assert_eq!(gate.status, "passed");
    }

    #[test]
    fn production_preset_fails_closed_on_error_readiness_hazards() {
        let production = parse_options(["--production", "workspace", "out"]);
        let feedback_only = parse_options(["--feedback", "workspace", "out"]);
        let error_hazard = production_report(vec![production_hazard(
            "function_pointer_surfaces",
            "error",
        )]);
        let feedback_hazard = production_report(vec![production_hazard(
            "custom_macro_invocations",
            "warning",
        )]);

        assert!(production_readiness_blocks_validation(
            &production,
            &error_hazard
        ));
        assert!(!production_readiness_blocks_validation(
            &production,
            &feedback_hazard
        ));
        assert!(!production_readiness_blocks_validation(
            &feedback_only,
            &error_hazard
        ));
    }

    #[test]
    fn production_preset_discharges_feature_cfg_root_hazards_when_args_cover_them() {
        let production = parse_options([
            "--production",
            "--cargo-check-arg",
            "--features",
            "--cargo-check-arg",
            "selected",
            "workspace",
            "out",
        ]);
        let report = production_report(vec![feature_cfg_root_hazard("app", "selected")]);

        assert!(!production_readiness_blocks_validation(
            &production,
            &report
        ));

        let mut validation = ValidationReport::new(&production);
        record_production_readiness_gate(
            &production,
            &mut validation,
            &report,
            "before compiler feedback",
        );
        let gate = validation
            .gates
            .iter()
            .find(|gate| gate.name == "production_readiness")
            .expect("production readiness gate should be recorded");
        assert_eq!(gate.status, "requires_feedback");
        assert_eq!(gate.error_count, Some(1));
        assert!(gate.reason.contains("feature cfg error hazard(s) covered"));
    }

    #[test]
    fn production_preset_discharges_feature_cfg_root_hazards_with_all_features() {
        let production = parse_options([
            "--production",
            "--cargo-check-arg",
            "--all-features",
            "workspace",
            "out",
        ]);
        let report = production_report(vec![feature_cfg_root_hazard("app", "selected")]);

        assert!(!production_readiness_blocks_validation(
            &production,
            &report
        ));
    }

    #[test]
    fn production_preset_discharges_simple_target_cfg_root_hazards_with_matching_target() {
        let production = parse_options([
            "--production",
            "--cargo-check-arg",
            "--target",
            "--cargo-check-arg",
            "wasm32-unknown-unknown",
            "workspace",
            "out",
        ]);
        let report = production_report(vec![target_cfg_root_hazard(
            "app",
            r#"#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]"#,
        )]);

        assert!(!production_readiness_blocks_validation(
            &production,
            &report
        ));
    }

    #[test]
    fn production_preset_discharges_host_cfg_root_hazards_without_explicit_target() {
        let production = parse_options(["--production", "workspace", "out"]);
        let cfg = if cfg!(unix) {
            "#[cfg(unix)]"
        } else {
            "#[cfg(windows)]"
        };
        let report = production_report(vec![target_cfg_root_hazard("app", cfg)]);

        assert!(!production_readiness_blocks_validation(
            &production,
            &report
        ));
    }

    #[test]
    fn production_preset_discharges_any_target_cfg_root_hazards_when_one_branch_matches() {
        let production = parse_options(["--production", "workspace", "out"]);
        let cfg = if cfg!(unix) {
            "#[cfg(any(windows, unix))]"
        } else {
            "#[cfg(any(unix, windows))]"
        };
        let report = production_report(vec![target_cfg_root_hazard("app", cfg)]);

        assert!(!production_readiness_blocks_validation(
            &production,
            &report
        ));
    }

    #[test]
    fn production_preset_discharges_not_target_cfg_root_hazards_when_branch_is_false() {
        let production = parse_options(["--production", "workspace", "out"]);
        let cfg = if cfg!(unix) {
            "#[cfg(not(windows))]"
        } else {
            "#[cfg(not(unix))]"
        };
        let report = production_report(vec![target_cfg_root_hazard("app", cfg)]);

        assert!(!production_readiness_blocks_validation(
            &production,
            &report
        ));
    }

    #[test]
    fn production_preset_discharges_all_feature_and_target_cfg_root_hazards() {
        let production = parse_options([
            "--production",
            "--cargo-check-arg",
            "--features",
            "--cargo-check-arg",
            "selected",
            "workspace",
            "out",
        ]);
        let target_flag = if cfg!(unix) { "unix" } else { "windows" };
        let report = production_report(vec![target_cfg_root_hazard(
            "app",
            &format!(r#"#[cfg(all(feature = "selected", {target_flag}))]"#),
        )]);

        assert!(!production_readiness_blocks_validation(
            &production,
            &report
        ));
    }

    #[test]
    fn production_preset_discharges_any_feature_or_target_cfg_root_hazards() {
        let production = parse_options(["--production", "workspace", "out"]);
        let target_flag = if cfg!(unix) { "unix" } else { "windows" };
        let report = production_report(vec![target_cfg_root_hazard(
            "app",
            &format!(r#"#[cfg(any(feature = "selected", {target_flag}))]"#),
        )]);

        assert!(!production_readiness_blocks_validation(
            &production,
            &report
        ));
    }

    #[test]
    fn production_preset_keeps_non_host_cfg_root_hazards_fail_closed() {
        let production = parse_options(["--production", "workspace", "out"]);
        let cfg = if cfg!(unix) {
            "#[cfg(windows)]"
        } else {
            "#[cfg(unix)]"
        };
        let report = production_report(vec![target_cfg_root_hazard("app", cfg)]);

        assert!(production_readiness_blocks_validation(&production, &report));
    }

    #[test]
    fn production_preset_keeps_negated_custom_cfg_root_hazards_fail_closed() {
        let production = parse_options(["--production", "workspace", "out"]);
        let report = production_report(vec![target_cfg_root_hazard(
            "app",
            "#[cfg(not(custom_platform))]",
        )]);

        assert!(production_readiness_blocks_validation(&production, &report));
    }

    #[test]
    fn production_preset_keeps_negated_unproven_feature_cfg_root_hazards_fail_closed() {
        let production = parse_options(["--production", "workspace", "out"]);
        let report = production_report(vec![target_cfg_root_hazard(
            "app",
            r#"#[cfg(not(feature = "selected"))]"#,
        )]);

        assert!(production_readiness_blocks_validation(&production, &report));
    }

    #[test]
    fn production_preset_keeps_mismatched_target_cfg_root_hazards_fail_closed() {
        let production = parse_options([
            "--production",
            "--cargo-check-arg",
            "--target=x86_64-unknown-linux-gnu",
            "workspace",
            "out",
        ]);
        let report = production_report(vec![target_cfg_root_hazard(
            "app",
            r#"#[cfg(target_arch = "wasm32")]"#,
        )]);

        assert!(production_readiness_blocks_validation(&production, &report));
    }

    #[test]
    fn production_preset_keeps_uncovered_cfg_root_hazards_fail_closed() {
        let production = parse_options(["--production", "workspace", "out"]);
        let missing_args = production_report(vec![feature_cfg_root_hazard("app", "selected")]);
        let unsupported_cfg = production_report(vec![opensource_core::ProductionHazardReport {
            code: "cfg_gated_roots".to_string(),
            severity: "error".to_string(),
            message: "root is cfg gated".to_string(),
            details: vec![opensource_core::ProductionHazardDetail {
                subject: "app::entry".to_string(),
                package: Some("app".to_string()),
                module_path: None,
                file: None,
                start_line: None,
                cfg: Some("#[cfg(custom_platform)]".to_string()),
                suggested_cargo_args: Vec::new(),
                blocked_idents: Vec::new(),
            }],
        }]);

        assert!(production_readiness_blocks_validation(
            &production,
            &missing_args
        ));
        assert!(production_readiness_blocks_validation(
            &production,
            &unsupported_cfg
        ));
    }

    #[test]
    fn production_matrix_plans_uncovered_feature_cfg_surfaces() {
        let production = parse_options(["--production", "workspace", "out"]);
        let report = production_report(vec![conditional_cfg_hazard("app", "extra")]);

        let entries = production_validation_matrix_entries(&production, &report);

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "features:app/extra");
        assert!(entries[0]
            .cargo_args
            .ends_with(&["--features".to_string(), "app/extra".to_string()]));
    }

    #[test]
    fn production_matrix_skips_cfg_surfaces_covered_by_user_args() {
        let production = parse_options([
            "--production",
            "--cargo-check-arg",
            "--features",
            "--cargo-check-arg",
            "app/extra",
            "workspace",
            "out",
        ]);
        let all_features = parse_options([
            "--production",
            "--cargo-check-arg",
            "--all-features",
            "workspace",
            "out",
        ]);
        let report = production_report(vec![conditional_cfg_hazard("app", "extra")]);

        assert!(production_validation_matrix_entries(&production, &report).is_empty());
        assert!(production_validation_matrix_entries(&all_features, &report).is_empty());
    }

    #[test]
    fn production_preset_marks_warning_hazards_as_review_required_after_feedback_accepts() {
        let options = parse_options(["--production", "workspace", "out"]);
        let mut validation = ValidationReport::new(&options);
        validation
            .gates
            .push(gate("production_readiness", "requires_feedback"));
        validation.gates.push(gate("feedback-repair", "accepted"));

        record_final_production_readiness(&options, &mut validation);

        let gate = validation
            .gates
            .iter()
            .find(|gate| gate.name == "production_ready")
            .expect("production_ready gate should be recorded");
        assert_eq!(gate.status, "review_required");
    }

    #[test]
    fn production_preset_accepts_feedback_discharged_warning_hazards() {
        let options = parse_options(["--production", "workspace", "out"]);
        let mut validation = ValidationReport::new(&options);
        let mut readiness = gate("production_readiness", "requires_feedback");
        readiness.review_warning_hazards = Some(0);
        validation.gates.push(readiness);
        validation.gates.push(gate("feedback-repair", "accepted"));

        record_final_production_readiness(&options, &mut validation);

        let gate = validation
            .gates
            .iter()
            .find(|gate| gate.name == "production_ready")
            .expect("production_ready gate should be recorded");
        assert_eq!(gate.status, "accepted");
    }

    #[test]
    fn production_preset_accepts_final_readiness_only_without_remaining_hazards() {
        let options = parse_options(["--production", "workspace", "out"]);
        let mut validation = ValidationReport::new(&options);
        validation
            .gates
            .push(gate("production_readiness", "ready_for_feedback"));
        validation.gates.push(gate("feedback-repair", "accepted"));

        record_final_production_readiness(&options, &mut validation);

        let gate = validation
            .gates
            .iter()
            .find(|gate| gate.name == "production_ready")
            .expect("production_ready gate should be recorded");
        assert_eq!(gate.status, "accepted");
    }

    #[test]
    fn production_preset_final_readiness_fails_after_widened_error_hazards() {
        let options = parse_options(["--production", "workspace", "out"]);
        let mut validation = ValidationReport::new(&options);
        validation.gates.push(gate("feedback-repair", "accepted"));
        validation
            .gates
            .push(gate("production_readiness", "hazards_detected"));

        record_final_production_readiness(&options, &mut validation);

        let gate = validation
            .gates
            .iter()
            .find(|gate| gate.name == "production_ready")
            .expect("production_ready gate should be recorded");
        assert_eq!(gate.status, "failed");
    }

    #[test]
    fn production_preset_final_readiness_tracks_matrix_baseline_limits() {
        let options = parse_options(["--production", "workspace", "out"]);
        let mut validation = ValidationReport::new(&options);
        validation
            .gates
            .push(gate("production_readiness", "requires_feedback"));
        validation.gates.push(gate("feedback-repair", "accepted"));
        validation
            .gates
            .push(gate("production_matrix", "baseline_limited"));

        record_final_production_readiness(&options, &mut validation);

        let gate = validation
            .gates
            .iter()
            .find(|gate| gate.name == "production_ready")
            .expect("production_ready gate should be recorded");
        assert_eq!(gate.status, "baseline_limited");
    }

    #[test]
    fn production_preset_final_readiness_requires_matrix_acceptance() {
        let options = parse_options(["--production", "workspace", "out"]);
        let mut validation = ValidationReport::new(&options);
        validation
            .gates
            .push(gate("production_readiness", "requires_feedback"));
        validation.gates.push(gate("feedback-repair", "accepted"));
        validation.gates.push(gate("production_matrix", "failed"));

        record_final_production_readiness(&options, &mut validation);

        let gate = validation
            .gates
            .iter()
            .find(|gate| gate.name == "production_ready")
            .expect("production_ready gate should be recorded");
        assert_eq!(gate.status, "failed");
    }

    #[test]
    fn production_preset_final_readiness_accepts_unneeded_matrix() {
        let options = parse_options(["--production", "workspace", "out"]);
        let mut validation = ValidationReport::new(&options);
        validation
            .gates
            .push(gate("production_readiness", "ready_for_feedback"));
        validation.gates.push(gate("feedback-repair", "accepted"));
        validation
            .gates
            .push(gate("production_matrix", "not_required"));

        record_final_production_readiness(&options, &mut validation);

        let gate = validation
            .gates
            .iter()
            .find(|gate| gate.name == "production_ready")
            .expect("production_ready gate should be recorded");
        assert_eq!(gate.status, "accepted");
    }

    #[test]
    fn positional_cargo_manifest_uses_parent_as_workspace_root() {
        let nested = parse_options(["repo/rust/Cargo.toml", "out"]);
        let current_dir = parse_options(["Cargo.toml", "out"]);

        assert_eq!(nested.workspace_root, PathBuf::from("repo/rust"));
        assert_eq!(current_dir.workspace_root, PathBuf::from("."));
    }

    #[test]
    fn explicit_feedback_repair_loop_can_raise_production_preset() {
        let options = parse_options([
            "--production",
            "--feedback-repair-loop",
            "5",
            "workspace",
            "out",
        ]);

        assert_eq!(options.feedback_repair_iterations, 5);
        assert!(options.run_baseline_check);
        assert!(options.deny_warnings);
    }

    #[test]
    fn explicit_slice_report_overrides_production_default() {
        let options = parse_options([
            "--production",
            "--slice-report",
            "custom.json",
            "workspace",
            "out",
        ]);

        assert_eq!(
            slice_report_path(&options),
            Some(PathBuf::from("custom.json"))
        );
    }

    #[test]
    fn explicit_decision_log_enables_decision_report() {
        let options = parse_options(["--decision-log", "decisions.json", "workspace", "out"]);

        assert_eq!(
            decision_log_path(&options),
            Some(PathBuf::from("decisions.json"))
        );
    }

    #[test]
    fn explicit_root_options_select_roots_without_source_markers() {
        let options = parse_options([
            "--analyzer",
            "syn",
            "--root",
            "app::selected",
            "--batch-roots",
            "--random-root-package",
            "codex-ipc",
            "--random-seed",
            "7",
            "workspace",
            "out",
        ]);

        assert_eq!(options.analyzer_mode, AnalyzerMode::Syn);
        assert_eq!(options.root_selectors, ["app::selected"]);
        assert_eq!(options.random_root_packages, ["codex-ipc"]);
        assert!(options.batch_roots);
        assert_eq!(options.random_seed, 7);
    }

    #[test]
    fn batch_roots_generate_without_mutating_source_markers() {
        let source = temp_path("cli-rootless-batch-source");
        let output = temp_path("cli-rootless-batch-output");
        write(
            source.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            source.join("app/Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        write(
            source.join("app/src/lib.rs"),
            r#"pub fn selected() -> usize {
    helper()
}

pub fn helper() -> usize {
    1
}

pub fn dead() -> usize {
    0
}
"#,
        );
        let options = parse_args_from(vec![
            OsString::from("--analyzer"),
            OsString::from("syn"),
            OsString::from("--batch-roots"),
            OsString::from("--root"),
            OsString::from("app::selected"),
            OsString::from("--preflight"),
            source.clone().into_os_string(),
            output.clone().into_os_string(),
        ])
        .expect("arguments should parse");

        run_batch_roots(&options).expect("rootless batch should generate");

        let generated = fs::read_to_string(output.join("0001-app_selected/app/src/lib.rs"))
            .expect("generated source should exist");
        assert!(generated.contains("pub fn selected"), "{generated}");
        assert!(generated.contains("pub fn helper"), "{generated}");
        assert!(!generated.contains("pub fn dead"), "{generated}");
        let original = fs::read_to_string(source.join("app/src/lib.rs")).unwrap();
        assert!(!original.contains("opensourced"), "{original}");
        let report = fs::read_to_string(output.join("batch-report.jsonl")).unwrap();
        assert!(report.contains("\"status\":\"generated\""), "{report}");
    }

    #[test]
    fn explicit_validation_report_overrides_default() {
        let options = parse_options([
            "--production",
            "--validation-report",
            "validation.json",
            "workspace",
            "out",
        ]);

        assert_eq!(
            validation_report_path(&options),
            Some(PathBuf::from("validation.json"))
        );
    }

    #[test]
    fn non_default_targets_require_matching_validation_args() {
        let targets = vec![
            GeneratedTargetReport {
                package: "app".to_string(),
                name: "demo".to_string(),
                kind: vec!["example".to_string()],
                src_path: PathBuf::from("/workspace/app/examples/demo.rs"),
                required_features: Vec::new(),
                default_features: Vec::new(),
            },
            GeneratedTargetReport {
                package: "app".to_string(),
                name: "behavior".to_string(),
                kind: vec!["test".to_string()],
                src_path: PathBuf::from("/workspace/app/tests/behavior.rs"),
                required_features: Vec::new(),
                default_features: Vec::new(),
            },
            GeneratedTargetReport {
                package: "app".to_string(),
                name: "throughput".to_string(),
                kind: vec!["bench".to_string()],
                src_path: PathBuf::from("/workspace/app/benches/throughput.rs"),
                required_features: Vec::new(),
                default_features: Vec::new(),
            },
        ];

        assert_eq!(
            uncovered_validation_targets(&targets, &[]),
            [
                "app example demo",
                "app test behavior",
                "app bench throughput"
            ]
        );
        assert!(uncovered_validation_targets(&targets, &["--all-targets".to_string()]).is_empty());
        assert!(uncovered_validation_targets(
            &targets,
            &[
                "--example".to_string(),
                "demo".to_string(),
                "--test=behavior".to_string(),
                "--benches".to_string()
            ]
        )
        .is_empty());
    }

    #[test]
    fn required_feature_targets_require_matching_validation_features() {
        let targets = vec![GeneratedTargetReport {
            package: "app".to_string(),
            name: "demo".to_string(),
            kind: vec!["example".to_string()],
            src_path: PathBuf::from("/workspace/app/examples/demo.rs"),
            required_features: vec!["demo-ui".to_string(), "sqlite".to_string()],
            default_features: Vec::new(),
        }];

        assert_eq!(
            uncovered_validation_targets(&targets, &["--all-targets".to_string()]),
            ["app example demo requires feature(s): demo-ui, sqlite"]
        );
        assert_eq!(
            uncovered_validation_targets(
                &targets,
                &[
                    "--all-targets".to_string(),
                    "--features".to_string(),
                    "demo-ui,app/sqlite".to_string(),
                ]
            ),
            Vec::<String>::new()
        );
        assert_eq!(
            uncovered_validation_targets(
                &targets,
                &["--example=demo".to_string(), "--all-features".to_string()]
            ),
            Vec::<String>::new()
        );
    }

    #[test]
    fn required_feature_bin_targets_require_matching_validation_features() {
        let targets = vec![GeneratedTargetReport {
            package: "app".to_string(),
            name: "cli".to_string(),
            kind: vec!["bin".to_string()],
            src_path: PathBuf::from("/workspace/app/src/bin/cli.rs"),
            required_features: vec!["cli".to_string()],
            default_features: Vec::new(),
        }];

        assert_eq!(
            uncovered_validation_targets(&targets, &[]),
            ["app bin cli requires feature(s): cli"]
        );
        assert_eq!(
            uncovered_validation_targets(
                &targets,
                &["--features".to_string(), "app/cli".to_string()]
            ),
            Vec::<String>::new()
        );
        assert_eq!(
            uncovered_validation_targets(&targets, &["--all-features".to_string()]),
            Vec::<String>::new()
        );
    }

    #[test]
    fn target_default_features_cover_required_features_unless_disabled() {
        let targets = vec![GeneratedTargetReport {
            package: "app".to_string(),
            name: "demo".to_string(),
            kind: vec!["example".to_string()],
            src_path: PathBuf::from("/workspace/app/examples/demo.rs"),
            required_features: vec!["demo-ui".to_string()],
            default_features: vec!["demo-ui".to_string()],
        }];

        assert_eq!(
            uncovered_validation_targets(&targets, &["--all-targets".to_string()]),
            Vec::<String>::new()
        );
        assert_eq!(
            uncovered_validation_targets(
                &targets,
                &[
                    "--all-targets".to_string(),
                    "--no-default-features".to_string()
                ]
            ),
            ["app example demo requires feature(s): demo-ui"]
        );
    }

    #[test]
    fn repeated_cargo_check_args_are_preserved() {
        let options = parse_options([
            "--feedback",
            "--cargo-check-arg",
            "--all-features",
            "--cargo-check-arg",
            "--target",
            "workspace",
            "out",
        ]);

        assert_eq!(options.cargo_check_args, ["--all-features", "--target"]);
    }

    #[test]
    fn feedback_widening_records_and_rerenders_from_compiler_diagnostics() {
        let source = temp_path("cli-feedback-widen-source");
        let output = temp_path("cli-feedback-widen-output");
        let slice_report = temp_path("cli-feedback-widen-report").join("slice-report.json");
        let opensourced_path = repo_root().join("crates/opensourced");
        write(
            source.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            source.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\n",
                opensourced_path
            ),
        );
        write(
            source.join("app/src/lib.rs"),
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
        let options = parse_args_from(vec![
            std::ffi::OsString::from("--feedback-loop"),
            std::ffi::OsString::from("2"),
            std::ffi::OsString::from("--slice-report"),
            slice_report.clone().into_os_string(),
            source.clone().into_os_string(),
            output.clone().into_os_string(),
        ])
        .expect("arguments should parse");
        let mut validation = ValidationReport::new(&options);
        let mut state = FeedbackWideningState::default();
        let mut missing_helper = diagnostic("E0425", "cannot find value `helper` in this scope");
        missing_helper.package_id = Some("app 0.1.0 (path+file:///tmp/app)".to_string());
        let feedback_report = report(false, vec![missing_helper]);

        let widened = try_widen_from_feedback(
            &options,
            &mut validation,
            &mut state,
            "feedback",
            1,
            &feedback_report,
            &output.join("slice-feedback.json"),
            0,
            0,
        )
        .expect("feedback widening should succeed");

        assert!(widened);
        assert_eq!(validation.attempts.len(), 1);
        assert_eq!(validation.attempts[0].status, "widened");
        assert_eq!(validation.attempts[0].feedback_widened_roots, Some(1));
        let generated = fs::read_to_string(output.join("app/src/lib.rs")).unwrap();
        assert!(generated.contains("pub fn helper"));
        let report_json: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(slice_report).unwrap()).unwrap();
        assert_eq!(report_json["feedback_widened_roots"][0], "app::helper");
    }

    #[test]
    fn feedback_widening_reconciles_lockfile_before_next_locked_check() {
        let source = temp_path("cli-feedback-widen-lock-source");
        let output = temp_path("cli-feedback-widen-lock-output");
        let slice_report = temp_path("cli-feedback-widen-lock-report").join("slice-report.json");
        let opensourced_path = repo_root().join("crates/opensourced");
        write(
            source.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(source.join("Cargo.lock"), stale_generated_lockfile());
        write(
            source.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\n",
                opensourced_path
            ),
        );
        write(
            source.join("app/src/lib.rs"),
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
        let options = parse_args_from(vec![
            std::ffi::OsString::from("--production"),
            std::ffi::OsString::from("--slice-report"),
            slice_report.into_os_string(),
            source.clone().into_os_string(),
            output.clone().into_os_string(),
        ])
        .expect("arguments should parse");
        let mut validation = ValidationReport::new(&options);
        let mut state = FeedbackWideningState::default();
        let mut missing_helper = diagnostic("E0425", "cannot find value `helper` in this scope");
        missing_helper.package_id = Some("app 0.1.0 (path+file:///tmp/app)".to_string());
        let feedback_report = report(false, vec![missing_helper]);

        let widened = try_widen_from_feedback(
            &options,
            &mut validation,
            &mut state,
            "feedback",
            1,
            &feedback_report,
            &output.join("slice-feedback.json"),
            0,
            0,
        )
        .expect("feedback widening should reconcile lockfile");

        assert!(widened);
        let lockfile = fs::read_to_string(output.join("Cargo.lock")).unwrap();
        assert!(!lockfile.contains("dead-helper"));
        assert!(validation
            .gates
            .iter()
            .any(|gate| gate.name == "lockfile" && gate.status == "passed"));
    }

    #[test]
    fn production_feedback_widening_rechecks_readiness_and_blocks_error_hazards() {
        let source = temp_path("cli-feedback-widen-cfg-source");
        let output = temp_path("cli-feedback-widen-cfg-output");
        let slice_report = temp_path("cli-feedback-widen-cfg-report").join("slice-report.json");
        let opensourced_path = repo_root().join("crates/opensourced");
        write(
            source.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            source.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[features]\nextra = []\n\n[dependencies]\nopensourced = {{ path = {:?} }}\n",
                opensourced_path
            ),
        );
        write(
            source.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;

#[opensourced]
pub fn entry() -> usize {
    1
}

#[cfg(feature = "extra")]
pub fn helper() -> usize {
    2
}
"#,
        );
        let options = parse_args_from(vec![
            std::ffi::OsString::from("--production"),
            std::ffi::OsString::from("--slice-report"),
            slice_report.clone().into_os_string(),
            source.clone().into_os_string(),
            output.clone().into_os_string(),
        ])
        .expect("arguments should parse");
        let mut validation = ValidationReport::new(&options);
        let mut state = FeedbackWideningState::default();
        let mut missing_helper = diagnostic("E0425", "cannot find value `helper` in this scope");
        missing_helper.package_id = Some("app 0.1.0 (path+file:///tmp/app)".to_string());
        let feedback_report = report(false, vec![missing_helper]);

        let error = try_widen_from_feedback(
            &options,
            &mut validation,
            &mut state,
            "feedback-repair",
            1,
            &feedback_report,
            &output.join("slice-feedback.json"),
            0,
            0,
        )
        .expect_err("cfg-gated widened root should block production");

        assert!(error
            .to_string()
            .contains("production readiness reported error hazards"));
        assert!(validation.gates.iter().any(|gate| {
            gate.name == "production_readiness" && gate.status == "hazards_detected"
        }));
        let report_json: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(slice_report).unwrap()).unwrap();
        assert_eq!(report_json["production"]["status"], "hazards_detected");
        assert_eq!(report_json["feedback_widened_roots"][0], "app::helper");
    }

    #[test]
    fn plain_check_failure_writes_rejected_validation_report() {
        let source = temp_path("cli-plain-check-source");
        let output = temp_path("cli-plain-check-output");
        let validation_path = temp_path("cli-plain-check-report").join("validation.json");
        write(output.join("Cargo.toml"), "not valid toml");
        let options = parse_args_from(vec![
            std::ffi::OsString::from("--check"),
            std::ffi::OsString::from("--validation-report"),
            validation_path.clone().into_os_string(),
            source.into_os_string(),
            output.into_os_string(),
        ])
        .expect("arguments should parse");
        let mut validation = ValidationReport::new(&options);

        let error = run_plain_check_gate(&options, &mut validation)
            .expect_err("plain check should fail for invalid Cargo.toml");

        assert!(error.to_string().contains("generated workspace failed"));
        let value: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(validation_path).unwrap()).unwrap();
        assert_eq!(value["status"], "rejected");
        assert_eq!(value["gates"][0]["name"], "check");
        assert_eq!(value["gates"][0]["status"], "failed");
    }

    fn report(success: bool, diagnostics: Vec<CheckDiagnostic>) -> CheckReport {
        CheckReport {
            manifest_path: PathBuf::from("/tmp/Cargo.toml"),
            working_dir: None,
            target_dir: None,
            timeout_ms: None,
            cargo_args: Vec::new(),
            success,
            timed_out: false,
            exit_code: if success { 0 } else { 101 },
            duration_ms: 25,
            diagnostics,
            widening: FeedbackWideningReport::default(),
            stderr: String::new(),
        }
    }

    fn production_report(
        hazards: Vec<opensource_core::ProductionHazardReport>,
    ) -> opensource_core::ProductionReadinessReport {
        let status = if hazards.iter().any(|hazard| hazard.severity == "error") {
            "hazards_detected"
        } else if hazards.is_empty() {
            "ready"
        } else {
            "requires_feedback"
        };
        opensource_core::ProductionReadinessReport {
            status: status.to_string(),
            hazards,
        }
    }

    fn production_hazard(code: &str, severity: &str) -> opensource_core::ProductionHazardReport {
        opensource_core::ProductionHazardReport {
            code: code.to_string(),
            severity: severity.to_string(),
            message: format!("{code} hazard"),
            details: Vec::new(),
        }
    }

    fn feature_cfg_root_hazard(
        package: &str,
        feature: &str,
    ) -> opensource_core::ProductionHazardReport {
        opensource_core::ProductionHazardReport {
            code: "cfg_gated_roots".to_string(),
            severity: "error".to_string(),
            message: "root is cfg gated".to_string(),
            details: vec![opensource_core::ProductionHazardDetail {
                subject: format!("{package}::entry"),
                package: Some(package.to_string()),
                module_path: None,
                file: None,
                start_line: None,
                cfg: Some(format!("#[cfg(feature = \"{feature}\")]")),
                suggested_cargo_args: vec!["--features".to_string(), feature.to_string()],
                blocked_idents: Vec::new(),
            }],
        }
    }

    fn target_cfg_root_hazard(package: &str, cfg: &str) -> opensource_core::ProductionHazardReport {
        opensource_core::ProductionHazardReport {
            code: "cfg_gated_roots".to_string(),
            severity: "error".to_string(),
            message: "root is cfg gated".to_string(),
            details: vec![opensource_core::ProductionHazardDetail {
                subject: format!("{package}::entry"),
                package: Some(package.to_string()),
                module_path: None,
                file: None,
                start_line: None,
                cfg: Some(cfg.to_string()),
                suggested_cargo_args: Vec::new(),
                blocked_idents: Vec::new(),
            }],
        }
    }

    fn conditional_cfg_hazard(
        package: &str,
        feature: &str,
    ) -> opensource_core::ProductionHazardReport {
        opensource_core::ProductionHazardReport {
            code: "conditional_compilation_attrs".to_string(),
            severity: "warning".to_string(),
            message: "retained cfg surface".to_string(),
            details: vec![opensource_core::ProductionHazardDetail {
                subject: package.to_string(),
                package: Some(package.to_string()),
                module_path: None,
                file: None,
                start_line: None,
                cfg: Some(format!("#[cfg(feature = \"{feature}\")]")),
                suggested_cargo_args: vec!["--features".to_string(), feature.to_string()],
                blocked_idents: Vec::new(),
            }],
        }
    }

    fn diagnostic(code: &str, message: &str) -> CheckDiagnostic {
        CheckDiagnostic {
            level: "error".to_string(),
            message: message.to_string(),
            code: Some(code.to_string()),
            package_id: None,
            target: None,
            rendered: None,
            spans: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    fn diagnostic_with_span(
        code: &str,
        message: &str,
        file_name: &str,
        line_start: u64,
        column_start: u64,
        column_end: u64,
    ) -> CheckDiagnostic {
        CheckDiagnostic {
            level: "error".to_string(),
            message: message.to_string(),
            code: Some(code.to_string()),
            package_id: None,
            target: None,
            rendered: None,
            spans: vec![opensource_core::CheckSpan {
                file_name: file_name.to_string(),
                line_start,
                line_end: line_start,
                column_start,
                column_end,
                is_primary: true,
                text: Vec::new(),
            }],
            suggestions: Vec::new(),
        }
    }

    fn diagnostic_with_rendered(code: &str, message: &str, rendered: &str) -> CheckDiagnostic {
        CheckDiagnostic {
            level: "error".to_string(),
            message: message.to_string(),
            code: Some(code.to_string()),
            package_id: None,
            target: None,
            rendered: Some(rendered.to_string()),
            spans: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    fn warning(message: &str) -> CheckDiagnostic {
        warning_with_code("unused_variables", message)
    }

    fn warning_with_code(code: &str, message: &str) -> CheckDiagnostic {
        CheckDiagnostic {
            level: "warning".to_string(),
            message: message.to_string(),
            code: Some(code.to_string()),
            package_id: None,
            target: None,
            rendered: None,
            spans: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    fn diagnostic_for_target(
        code: &str,
        message: &str,
        package_id: &str,
        target_kind: &str,
        target_name: &str,
    ) -> CheckDiagnostic {
        CheckDiagnostic {
            level: "error".to_string(),
            message: message.to_string(),
            code: Some(code.to_string()),
            package_id: Some(package_id.to_string()),
            target: Some(CheckTarget {
                name: target_name.to_string(),
                kind: vec![target_kind.to_string()],
                src_path: None,
            }),
            rendered: None,
            spans: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    fn stale_generated_lockfile() -> &'static str {
        r#"# This file is automatically @generated by Cargo.
# It is not intended for manual editing.
version = 4

[[package]]
name = "app"
version = "0.1.0"
dependencies = [
 "dead-helper",
]

[[package]]
name = "dead-helper"
version = "0.1.0"
"#
    }

    fn parse_options<const N: usize>(args: [&str; N]) -> super::CliOptions {
        parse_args_from(args.into_iter().map(std::ffi::OsString::from))
            .expect("arguments should parse")
    }

    fn gate(name: &str, status: &str) -> ValidationGateReport {
        ValidationGateReport {
            name: name.to_string(),
            status: status.to_string(),
            reason: String::new(),
            report_path: None,
            error_count: None,
            warning_count: None,
            semantic_warning_hazards: None,
            review_warning_hazards: None,
        }
    }

    fn repo_root() -> PathBuf {
        let mut current = std::env::current_dir().expect("current directory should resolve");
        loop {
            if current.join("crates/opensourced/Cargo.toml").exists() {
                return current;
            }
            assert!(
                current.pop(),
                "could not find repository root from current directory"
            );
        }
    }

    fn temp_path(label: &str) -> PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("slicers-{label}-{unique}"))
    }

    fn write(path: PathBuf, contents: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, contents).unwrap();
    }
}
