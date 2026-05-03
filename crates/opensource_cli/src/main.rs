use std::{ffi::OsStr, path::PathBuf, process::Command};

use opensource_core::{
    check_workspace, generate_with_analyzer, write_report, AnalyzerMode, CheckDiagnostic,
    CheckOptions, CheckReport, GenerateOptions,
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
            "  analyzer semantic files: {}/{} analyzed ({} failed)",
            semantic.analyzed_files, semantic.source_files, semantic.failed_files
        );
        println!(
            "  analyzer method calls: {}/{} resolved ({} unresolved)",
            semantic.resolved_method_calls, semantic.method_calls, semantic.unresolved_method_calls
        );
        println!(
            "  analyzer paths: {}/{} resolved ({} unresolved)",
            semantic.resolved_paths, semantic.paths, semantic.unresolved_paths
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

    if options.feedback_iterations > 0 {
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
    feedback_limit: usize,
    feedback_report: Option<PathBuf>,
    workspace_root: PathBuf,
    output_root: PathBuf,
}

fn parse_args() -> Result<CliOptions, Box<dyn std::error::Error>> {
    let mut analyzer_mode = AnalyzerMode::Syn;
    let mut run_check = false;
    let mut feedback_iterations = 0;
    let mut feedback_limit = 12;
    let mut feedback_report = None;
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
        } else if arg == OsStr::new("--feedback-loop") {
            feedback_iterations = parse_usize_arg("--feedback-loop", args.next())?;
        } else if arg == OsStr::new("--feedback-limit") {
            feedback_limit = parse_usize_arg("--feedback-limit", args.next())?;
        } else if arg == OsStr::new("--feedback-report") {
            feedback_report = Some(PathBuf::from(
                args.next()
                    .ok_or("--feedback-report requires a following path")?,
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
        feedback_limit,
        feedback_report,
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

fn run_plain_check(output_root: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
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

fn print_feedback(report: &CheckReport, limit: usize, report_path: &PathBuf) {
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
        "usage: slicers [--analyzer <syn|ra-hir>] [--check] [--feedback] [--feedback-loop <n>] ",
        "[--feedback-limit <n>] [--feedback-report <path>] ",
        "<workspace-root> <output-root>"
    )
    .to_string()
}

fn same_path(left: &PathBuf, right: &PathBuf) -> bool {
    let Ok(left) = left.canonicalize() else {
        return false;
    };
    let Ok(right) = right.canonicalize() else {
        return false;
    };
    left == right
}
