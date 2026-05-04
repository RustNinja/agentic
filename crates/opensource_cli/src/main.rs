use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::{OsStr, OsString},
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
    time::Duration,
};

use serde::Serialize;

use opensource_core::{
    check_workspace, generate_with_analyzer, generate_with_analyzer_feedback, preflight_workspace,
    repair_workspace, write_generate_report, write_preflight_report, write_repair_report,
    write_report, AnalyzerMode, CheckDiagnostic, CheckOptions, CheckReport, GenerateOptions,
    GeneratedTargetReport, PreflightDiagnostic, PreflightOptions, PreflightReport, RepairOptions,
};

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let options = parse_args()?;

    if same_path(&options.workspace_root, &options.output_root) {
        return Err("output root must be different from workspace root".into());
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

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: options.workspace_root.clone(),
            output_root: options.output_root.clone(),
        },
        options.analyzer_mode,
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
    println!("reachable callables:");
    for callable in &report.reachable {
        println!("  {callable}");
    }
    println!("reachable items:");
    for item in &report.reachable_items {
        println!("  {item}");
    }
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
    });
    record_production_readiness_gate(
        &options,
        &mut validation,
        &report.production,
        "before compiler feedback",
    );
    if production_readiness_blocks_validation(&options, &report.production) {
        let reason =
            "production readiness reported error hazards before compiler feedback".to_string();
        finish_validation(&options, &mut validation, "rejected", Some(&reason))?;
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
        });
        finish_validation(&options, &mut validation, "rejected", Some(&reason))?;
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
    });
    if let Err(error) = refresh_generated_lockfile_for_locked_validation(&options, &mut validation)
    {
        let reason = error.to_string();
        finish_validation(&options, &mut validation, "rejected", Some(&reason))?;
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
        let report = run_preflight(&options)?;
        validation.gates.push(ValidationGateReport {
            name: "preflight".to_string(),
            status: if report.success { "passed" } else { "failed" }.to_string(),
            reason: if report.success {
                "generated workspace passed fast structural validation".to_string()
            } else {
                "generated workspace failed fast structural validation".to_string()
            },
            report_path: Some(preflight_report_path(&options)),
            error_count: Some(report.error_count()),
            warning_count: Some(report.warning_count()),
            semantic_warning_hazards: None,
        });
        if !report.success {
            finish_validation(
                &options,
                &mut validation,
                "rejected",
                Some("generated workspace failed fast preflight validation"),
            )?;
            return Err("generated workspace failed fast preflight validation".into());
        }
    }

    if options.feedback_repair_iterations > 0 {
        if let Err(error) = run_feedback_repair_loop(&options, baseline.as_ref(), &mut validation) {
            let reason = error.to_string();
            finish_validation(&options, &mut validation, "rejected", Some(&reason))?;
            return Err(reason.into());
        }
    } else if options.feedback_iterations > 0 {
        if let Err(error) = run_feedback_loop(&options, baseline.as_ref(), &mut validation) {
            let reason = error.to_string();
            finish_validation(&options, &mut validation, "rejected", Some(&reason))?;
            return Err(reason.into());
        }
    } else if options.run_check {
        run_plain_check_gate(&options, &mut validation)?;
    }

    if let Err(error) =
        run_production_validation_matrix(&options, &report.production, &mut validation)
    {
        let reason = error.to_string();
        finish_validation(&options, &mut validation, "rejected", Some(&reason))?;
        return Err(reason.into());
    }

    record_final_production_readiness(&options, &mut validation);
    finish_validation(&options, &mut validation, "accepted", None)?;
    Ok(())
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
    validation_report: Option<PathBuf>,
    run_preflight: bool,
    preflight_report: Option<PathBuf>,
    production_preset: bool,
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
    let mut validation_report = None;
    let mut run_preflight = false;
    let mut preflight_report = None;
    let mut production_preset = false;
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
                .ok_or("--analyzer requires a following value: syn or ra-hir")?;
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
        validation_report,
        run_preflight,
        preflight_report,
        production_preset,
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
    println!("baseline: cargo check --message-format=json");
    run_baseline_check_with_args(
        options,
        options.cargo_check_args.clone(),
        baseline_target_dir(options),
    )
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

fn preflight_report_path(options: &CliOptions) -> PathBuf {
    options
        .preflight_report
        .clone()
        .unwrap_or_else(|| options.output_root.join("slice-preflight.json"))
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
    if !locked_validation_requested(&options.cargo_check_args)
        || !options.workspace_root.join("Cargo.lock").exists()
    {
        return Ok(());
    }

    let lockfile_path = options.output_root.join("Cargo.lock");
    if !lockfile_path.exists() {
        let reason = "generated Cargo.lock is missing before locked validation";
        validation.gates.push(ValidationGateReport {
            name: "lockfile".to_string(),
            status: "failed".to_string(),
            reason: reason.to_string(),
            report_path: Some(lockfile_path),
            error_count: None,
            warning_count: None,
            semantic_warning_hazards: None,
        });
        return Err(reason.into());
    }

    let manifest_path = absolute_path(&options.output_root.join("Cargo.toml"))?;
    let working_dir = manifest_working_dir(&manifest_path);
    let mut command = Command::new("cargo");
    command
        .arg("generate-lockfile")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .current_dir(&working_dir);
    if options
        .cargo_check_args
        .iter()
        .any(|arg| matches!(arg.as_str(), "--offline" | "--frozen"))
    {
        command.arg("--offline");
    }
    let output = command.output()?;
    if !output.status.success() {
        let reason = format!(
            "generated Cargo.lock could not be reconciled before locked validation\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        validation.gates.push(ValidationGateReport {
            name: "lockfile".to_string(),
            status: "failed".to_string(),
            reason: reason.clone(),
            report_path: Some(lockfile_path),
            error_count: None,
            warning_count: None,
            semantic_warning_hazards: None,
        });
        return Err(reason.into());
    }

    println!("lockfile: generated Cargo.lock reconciled before locked validation");
    validation.gates.push(ValidationGateReport {
        name: "lockfile".to_string(),
        status: "passed".to_string(),
        reason: "generated Cargo.lock was reconciled before locked validation".to_string(),
        report_path: Some(lockfile_path),
        error_count: None,
        warning_count: None,
        semantic_warning_hazards: None,
    });
    Ok(())
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
    let widened_report = generate_with_analyzer_feedback(
        GenerateOptions {
            workspace_root: options.workspace_root.clone(),
            output_root: options.output_root.clone(),
        },
        options.analyzer_mode,
        &state.diagnostics,
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
        "accepted" if matches!(production_matrix_status, "accepted" | "not_required") => (
            "accepted",
            "production preset passed baseline, generation, preflight, target coverage, and compiler feedback",
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
    });
}

fn baseline_target_dir(options: &CliOptions) -> PathBuf {
    options
        .baseline_target_dir
        .clone()
        .unwrap_or_else(|| sibling_output_path(&options.output_root, "target-baseline"))
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
        "usage: slicers [--analyzer <syn|ra-hir|ra-hir-proc-macros>] [--production] [--check] [--preflight] [--feedback] ",
        "[--feedback-loop <n>] [--feedback-repair-loop <n>] [--feedback-limit <n>] ",
        "[--feedback-timeout <seconds>] [--deny-warnings] [--feedback-report <path>] ",
        "[--feedback-target-dir <path>] [--cargo-check-arg <arg>] [--repair-report <path>] ",
        "[--baseline-check] [--allow-baseline-failures] [--baseline-report <path>] ",
        "[--baseline-target-dir <path>] [--slice-report <path>] [--validation-report <path>] ",
        "[--preflight-report <path>] ",
        "<workspace-root-or-Cargo.toml> <output-root>\n",
        "default analyzer: ra-hir when the binary is built with the ra-hir feature, otherwise syn; ",
        "--production defaults to ra-hir-proc-macros when available"
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
    use std::{fs, path::PathBuf};

    use opensource_core::{
        AnalyzerMode, CheckDiagnostic, CheckReport, CheckTarget, FeedbackWideningReport,
        GeneratedTargetReport,
    };

    use super::{
        baseline_limited_feedback_is_accepted, diagnostics_shape_signature, diagnostics_signature,
        feedback_errors_are_baseline_known, feedback_is_accepted, feedback_repair_is_accepted,
        parse_args_from, production_readiness_blocks_validation,
        production_validation_matrix_entries, record_final_production_readiness,
        record_production_readiness_gate, refresh_generated_lockfile_for_locked_validation,
        run_plain_check_gate, semantic_hazard_warning_count, slice_report_path,
        try_widen_from_feedback, uncovered_validation_targets, validation_report_path,
        FeedbackWideningState, ValidationGateReport, ValidationReport,
    };

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
        assert_eq!(options.workspace_root, PathBuf::from("workspace"));
        assert_eq!(options.output_root, PathBuf::from("out"));
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
    fn explicit_syn_analyzer_overrides_production_semantic_default() {
        let options = parse_options(["--production", "--analyzer", "syn", "workspace", "out"]);

        assert_eq!(options.analyzer_mode, AnalyzerMode::Syn);
        assert!(options.production_preset);
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
    fn production_preset_records_final_readiness_after_feedback_accepts() {
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
            .push(gate("production_readiness", "requires_feedback"));
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
