use std::{
    collections::BTreeSet,
    ffi::{OsStr, OsString},
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

use serde::Serialize;

use opensource_core::{
    check_workspace, generate_with_analyzer, preflight_workspace, repair_workspace,
    write_generate_report, write_preflight_report, write_repair_report, write_report, AnalyzerMode,
    CheckDiagnostic, CheckOptions, CheckReport, GenerateOptions, GeneratedTargetReport,
    PreflightDiagnostic, PreflightOptions, PreflightReport, RepairOptions,
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
    validation.gates.push(ValidationGateReport {
        name: "production_readiness".to_string(),
        status: report.production.status.clone(),
        reason: format!(
            "{} production hazard(s) reported before compiler feedback",
            report.production.hazards.len()
        ),
        report_path: slice_report_path(&options),
        error_count: None,
        warning_count: None,
        semantic_warning_hazards: None,
    });
    if production_readiness_blocks_validation(&options, &report.production.status) {
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
        run_plain_check(&options)?;
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
    repair_report_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    repair_total_changes: Option<usize>,
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
    let mut analyzer_mode = AnalyzerMode::Syn;
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
    let status = Command::new("cargo")
        .arg("check")
        .arg("--manifest-path")
        .arg(options.output_root.join("Cargo.toml"))
        .args(&options.cargo_check_args)
        .status()?;
    if !status.success() {
        return Err(format!("generated workspace failed cargo check with {status}").into());
    }
    Ok(())
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
    check_workspace(CheckOptions {
        manifest_path: options.workspace_root.join("Cargo.toml"),
        target_dir: Some(baseline_target_dir(options)),
        timeout: options.feedback_timeout,
        cargo_args: options.cargo_check_args.clone(),
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
            let target_kind = validation_target_kind(target)?;
            (!validation_target_is_covered(target_kind, &target.name, cargo_args))
                .then(|| format!("{} {} {}", target.package, target_kind, target.name))
        })
        .collect()
}

fn validation_target_kind(target: &GeneratedTargetReport) -> Option<&'static str> {
    ["example", "test", "bench"]
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
        "example" => "--examples",
        "test" => "--tests",
        "bench" => "--benches",
        _ => "--all-targets",
    }
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
        if feedback_is_accepted(&report, baseline, options.deny_warnings) {
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
            && baseline_limited_feedback_is_accepted(&report, baseline, options.deny_warnings)
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

fn feedback_target_dir(options: &CliOptions) -> PathBuf {
    options
        .feedback_target_dir
        .clone()
        .unwrap_or_else(|| options.output_root.join("target-feedback"))
}

fn production_readiness_blocks_validation(options: &CliOptions, status: &str) -> bool {
    options.production_preset && status == "hazards_detected"
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
    let (status, reason) = match feedback_status {
        "accepted" => (
            "accepted",
            "production preset passed baseline, generation, preflight, target coverage, and compiler feedback",
        ),
        "baseline_limited" => (
            "baseline_limited",
            "production preset matched an allowed failing source baseline; generated workspace is not cleanly production-ready",
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

fn baseline_limited_feedback_is_accepted(
    report: &CheckReport,
    baseline: Option<&CheckReport>,
    deny_warnings: bool,
) -> bool {
    feedback_errors_are_baseline_known(report, baseline)
        && semantic_hazard_warning_count(&report.diagnostics, baseline) == 0
        && (!deny_warnings || report.warning_count() == 0)
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
        "usage: slicers [--analyzer <syn|ra-hir>] [--production] [--check] [--preflight] [--feedback] ",
        "[--feedback-loop <n>] [--feedback-repair-loop <n>] [--feedback-limit <n>] ",
        "[--feedback-timeout <seconds>] [--deny-warnings] [--feedback-report <path>] ",
        "[--feedback-target-dir <path>] [--cargo-check-arg <arg>] [--repair-report <path>] ",
        "[--baseline-check] [--allow-baseline-failures] [--baseline-report <path>] ",
        "[--baseline-target-dir <path>] [--slice-report <path>] [--validation-report <path>] ",
        "[--preflight-report <path>] ",
        "<workspace-root-or-Cargo.toml> <output-root>"
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
    use std::path::PathBuf;

    use opensource_core::{
        CheckDiagnostic, CheckReport, CheckTarget, FeedbackWideningReport, GeneratedTargetReport,
    };

    use super::{
        baseline_limited_feedback_is_accepted, diagnostics_shape_signature, diagnostics_signature,
        feedback_errors_are_baseline_known, feedback_is_accepted, parse_args_from,
        production_readiness_blocks_validation, record_final_production_readiness,
        semantic_hazard_warning_count, slice_report_path, uncovered_validation_targets,
        validation_report_path, ValidationGateReport, ValidationReport,
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
    fn production_preset_fails_closed_on_error_readiness_hazards() {
        let production = parse_options(["--production", "workspace", "out"]);
        let feedback_only = parse_options(["--feedback", "workspace", "out"]);

        assert!(production_readiness_blocks_validation(
            &production,
            "hazards_detected"
        ));
        assert!(!production_readiness_blocks_validation(
            &production,
            "requires_feedback"
        ));
        assert!(!production_readiness_blocks_validation(
            &feedback_only,
            "hazards_detected"
        ));
    }

    #[test]
    fn production_preset_records_final_readiness_after_feedback_accepts() {
        let options = parse_options(["--production", "workspace", "out"]);
        let mut validation = ValidationReport::new(&options);
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
            },
            GeneratedTargetReport {
                package: "app".to_string(),
                name: "behavior".to_string(),
                kind: vec!["test".to_string()],
                src_path: PathBuf::from("/workspace/app/tests/behavior.rs"),
            },
            GeneratedTargetReport {
                package: "app".to_string(),
                name: "throughput".to_string(),
                kind: vec!["bench".to_string()],
                src_path: PathBuf::from("/workspace/app/benches/throughput.rs"),
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

    fn report(success: bool, diagnostics: Vec<CheckDiagnostic>) -> CheckReport {
        CheckReport {
            manifest_path: PathBuf::from("/tmp/Cargo.toml"),
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
}
