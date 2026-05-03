use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

use opensource_core::{
    check_workspace, generate_with_analyzer, preflight_workspace, repair_workspace,
    write_generate_report, write_preflight_report, write_repair_report, write_report, AnalyzerMode,
    CheckDiagnostic, CheckOptions, CheckReport, GenerateOptions, PreflightDiagnostic,
    PreflightOptions, PreflightReport, RepairOptions,
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
    if report.roots.len() == 1 {
        println!("root: {}", report.root);
    } else {
        println!("roots:");
        for root in &report.roots {
            println!("  {root}");
        }
    }
    println!("files written: {}", report.files_written);
    println!("packages: {}", report.packages.join(", "));
    println!("reachable callables:");
    for callable in &report.reachable {
        println!("  {callable}");
    }
    println!("reachable items:");
    for item in &report.reachable_items {
        println!("  {item}");
    }
    if let Some(report_path) = &options.slice_report {
        write_generate_report(&report, report_path)?;
        println!("slice report: {}", report_path.display());
    }

    if options.run_preflight
        || options.feedback_iterations > 0
        || options.feedback_repair_iterations > 0
    {
        let report = run_preflight(&options)?;
        if !report.success {
            return Err("generated workspace failed fast preflight validation".into());
        }
    }

    if options.feedback_repair_iterations > 0 {
        run_feedback_repair_loop(&options)?;
    } else if options.feedback_iterations > 0 {
        run_feedback_loop(&options)?;
    } else if options.run_check {
        run_plain_check(&options.output_root)?;
    }

    Ok(())
}

struct CliOptions {
    analyzer_mode: AnalyzerMode,
    run_check: bool,
    feedback_iterations: usize,
    feedback_repair_iterations: usize,
    feedback_limit: usize,
    feedback_report: Option<PathBuf>,
    feedback_timeout: Option<Duration>,
    repair_report: Option<PathBuf>,
    slice_report: Option<PathBuf>,
    run_preflight: bool,
    preflight_report: Option<PathBuf>,
    workspace_root: PathBuf,
    output_root: PathBuf,
}

fn parse_args() -> Result<CliOptions, Box<dyn std::error::Error>> {
    let mut analyzer_mode = AnalyzerMode::Syn;
    let mut run_check = false;
    let mut feedback_iterations = 0;
    let mut feedback_repair_iterations = 0;
    let mut feedback_limit = 12;
    let mut feedback_report = None;
    let mut feedback_timeout = Some(Duration::from_secs(600));
    let mut repair_report = None;
    let mut slice_report = None;
    let mut run_preflight = false;
    let mut preflight_report = None;
    let mut positional = Vec::new();
    let mut args = std::env::args_os().skip(1);

    while let Some(arg) = args.next() {
        if arg == OsStr::new("--check") {
            run_check = true;
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
        } else if arg == OsStr::new("--feedback-loop") {
            feedback_iterations = parse_usize_arg("--feedback-loop", args.next())?;
        } else if arg == OsStr::new("--feedback-repair-loop") {
            feedback_repair_iterations = parse_usize_arg("--feedback-repair-loop", args.next())?;
        } else if arg == OsStr::new("--feedback-limit") {
            feedback_limit = parse_usize_arg("--feedback-limit", args.next())?;
        } else if arg == OsStr::new("--feedback-timeout") {
            feedback_timeout = parse_feedback_timeout(args.next())?;
        } else if arg == OsStr::new("--feedback-report") {
            feedback_report = Some(PathBuf::from(
                args.next()
                    .ok_or("--feedback-report requires a following path")?,
            ));
        } else if arg == OsStr::new("--repair-report") {
            repair_report = Some(PathBuf::from(
                args.next()
                    .ok_or("--repair-report requires a following path")?,
            ));
        } else if arg == OsStr::new("--slice-report") {
            slice_report = Some(PathBuf::from(
                args.next()
                    .ok_or("--slice-report requires a following path")?,
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

    Ok(CliOptions {
        analyzer_mode,
        run_check,
        feedback_iterations,
        feedback_repair_iterations,
        feedback_limit,
        feedback_report,
        feedback_timeout,
        repair_report,
        slice_report,
        run_preflight,
        preflight_report,
        workspace_root: workspace_root.clone(),
        output_root: output_root.clone(),
    })
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

fn run_plain_check(output_root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new("cargo")
        .arg("check")
        .arg("--manifest-path")
        .arg(output_root.join("Cargo.toml"))
        .status()?;
    if !status.success() {
        return Err(format!("generated workspace failed cargo check with {status}").into());
    }
    Ok(())
}

fn run_preflight(options: &CliOptions) -> Result<PreflightReport, Box<dyn std::error::Error>> {
    let report_path = options
        .preflight_report
        .clone()
        .unwrap_or_else(|| options.output_root.join("slice-preflight.json"));
    let report = preflight_workspace(PreflightOptions {
        manifest_path: options.output_root.join("Cargo.toml"),
    })?;
    write_preflight_report(&report, &report_path)?;
    print_preflight(&report, options.feedback_limit, &report_path);
    Ok(report)
}

fn run_feedback_loop(options: &CliOptions) -> Result<(), Box<dyn std::error::Error>> {
    let report_path = options
        .feedback_report
        .clone()
        .unwrap_or_else(|| options.output_root.join("slice-feedback.json"));

    for attempt in 1..=options.feedback_iterations {
        println!(
            "feedback attempt {attempt}/{}: cargo check --message-format=json",
            options.feedback_iterations
        );
        let report = check_workspace(CheckOptions {
            manifest_path: options.output_root.join("Cargo.toml"),
            target_dir: Some(options.output_root.join("target-feedback")),
            timeout: options.feedback_timeout,
        })?;
        write_report(&report, &report_path)?;
        print_feedback(&report, options.feedback_limit, &report_path);

        if report.success {
            return Ok(());
        }
    }

    Err(format!(
        "generated workspace failed compiler feedback loop; report written to {}",
        report_path.display()
    )
    .into())
}

fn run_feedback_repair_loop(options: &CliOptions) -> Result<(), Box<dyn std::error::Error>> {
    let feedback_report_path = options
        .feedback_report
        .clone()
        .unwrap_or_else(|| options.output_root.join("slice-feedback.json"));
    let repair_report_path = options
        .repair_report
        .clone()
        .unwrap_or_else(|| options.output_root.join("slice-repair.json"));
    let mut seen_diagnostics = std::collections::BTreeSet::new();

    for attempt in 1..=options.feedback_repair_iterations {
        println!(
            "feedback repair attempt {attempt}/{}: cargo check --message-format=json",
            options.feedback_repair_iterations
        );
        let report = check_workspace(CheckOptions {
            manifest_path: options.output_root.join("Cargo.toml"),
            target_dir: Some(options.output_root.join("target-feedback")),
            timeout: options.feedback_timeout,
        })?;
        write_report(&report, &feedback_report_path)?;
        print_feedback(&report, options.feedback_limit, &feedback_report_path);

        let repairable_warnings = repairable_warning_count(&report.diagnostics);
        if report.success && repairable_warnings == 0 {
            return Ok(());
        }
        if report.success {
            println!(
                "feedback: cargo check passed but {repairable_warnings} repairable warning(s) remain; attempting conservative repair"
            );
        }
        if report.timed_out {
            break;
        }

        let signature = diagnostics_signature(&report.diagnostics);
        if !seen_diagnostics.insert(signature) {
            return Err(format!(
                "feedback repair made no diagnostic progress; report written to {}",
                feedback_report_path.display()
            )
            .into());
        }

        let repair_report = repair_workspace(RepairOptions {
            output_root: options.output_root.clone(),
            diagnostics: report.diagnostics,
        })?;
        write_repair_report(&repair_report, &repair_report_path)?;
        println!(
            "repair: removed_items={}, removed_imports={}, skipped_diagnostics={}, total_changes={}; report: {}",
            repair_report.removed_items,
            repair_report.removed_imports,
            repair_report.skipped_diagnostics,
            repair_report.total_changes(),
            repair_report_path.display()
        );

        if repair_report.total_changes() == 0 {
            break;
        }

        let preflight = run_preflight(options)?;
        if !preflight.success {
            return Err("repair produced a structurally invalid generated workspace".into());
        }
    }

    Err(format!(
        "generated workspace failed compiler repair loop; feedback report written to {}, repair report written to {}",
        feedback_report_path.display(),
        repair_report_path.display()
    )
    .into())
}

fn repairable_warning_count(diagnostics: &[CheckDiagnostic]) -> usize {
    diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.level == "warning"
                && matches!(
                    diagnostic.code.as_deref(),
                    Some("dead_code" | "unused_imports")
                )
        })
        .count()
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

fn print_feedback(report: &CheckReport, limit: usize, report_path: &Path) {
    if report.success {
        println!(
            "feedback: cargo check passed with {} warning(s); report: {}",
            report.warning_count(),
            report_path.display()
        );
        return;
    }

    println!(
        "feedback: cargo check failed with {} error(s), {} warning(s); report: {}",
        report.error_count(),
        report.warning_count(),
        report_path.display()
    );

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

fn usage() -> String {
    concat!(
        "usage: slicers [--analyzer <syn|ra-hir>] [--check] [--preflight] [--feedback] ",
        "[--feedback-loop <n>] [--feedback-repair-loop <n>] [--feedback-limit <n>] ",
        "[--feedback-timeout <seconds>] [--feedback-report <path>] [--repair-report <path>] ",
        "[--slice-report <path>] [--preflight-report <path>] ",
        "<workspace-root> <output-root>"
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
