use std::{
    collections::BTreeMap,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use opensource_core::{
    generate, generate_compiler_prune_workspace, generate_lint_audit_workspace,
    refresh_compiler_prune_visibility, CompilerPruneOptions, CompilerPruneRefreshOptions,
    GenerateOptions, LintAuditOptions,
};
use serde_json::{json, Value};

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut run_check = false;
    let mut lint_audit = false;
    let mut compiler_prune = false;
    let mut rust_analyzer = RustAnalyzerOptions::default();
    let mut positional = Vec::new();
    let mut args = std::env::args_os().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_os_str() {
            value if value == OsStr::new("--check") => run_check = true,
            value if value == OsStr::new("--lint-audit") => lint_audit = true,
            value if value == OsStr::new("--compiler-prune") => compiler_prune = true,
            value if value == OsStr::new("--ra-audit") => rust_analyzer.enabled = true,
            value if value == OsStr::new("--ra-disable-build-scripts") => {
                rust_analyzer.disable_build_scripts = true
            }
            value if value == OsStr::new("--ra-disable-proc-macros") => {
                rust_analyzer.disable_proc_macros = true
            }
            value if value == OsStr::new("--rust-analyzer") => {
                let Some(path) = args.next() else {
                    return Err("--rust-analyzer requires a path".into());
                };
                rust_analyzer.binary = Some(PathBuf::from(path));
            }
            value if value == OsStr::new("--ra-proc-macro-srv") => {
                let Some(path) = args.next() else {
                    return Err("--ra-proc-macro-srv requires a path".into());
                };
                rust_analyzer.proc_macro_srv = Some(PathBuf::from(path));
            }
            value if value == OsStr::new("--help") || value == OsStr::new("-h") => {
                println!("{}", usage());
                return Ok(());
            }
            _ => positional.push(PathBuf::from(arg)),
        }
    }

    let [workspace_root, output_root] = positional.as_slice() else {
        return Err(usage().into());
    };

    if same_path(workspace_root, output_root) {
        return Err("output root must be different from workspace root".into());
    }

    if lint_audit {
        run_lint_audit(
            workspace_root.clone(),
            output_root.clone(),
            rust_analyzer.clone(),
        )?;
        return Ok(());
    }

    if compiler_prune {
        run_compiler_prune(
            workspace_root.clone(),
            output_root.clone(),
            rust_analyzer.clone(),
        )?;
        return Ok(());
    }

    let report = generate(GenerateOptions {
        workspace_root: workspace_root.clone(),
        output_root: output_root.clone(),
    })?;

    println!("root: {}", report.root);
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

    if run_check {
        let status = Command::new("cargo")
            .arg("check")
            .arg("--manifest-path")
            .arg(output_root.join("Cargo.toml"))
            .status()?;
        if !status.success() {
            return Err(format!("generated workspace failed cargo check with {status}").into());
        }
    }

    if rust_analyzer.enabled {
        run_rust_analyzer_audit(&output_root, &rust_analyzer)?;
    }

    Ok(())
}

fn run_compiler_prune(
    workspace_root: PathBuf,
    output_root: PathBuf,
    rust_analyzer: RustAnalyzerOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let report = generate_compiler_prune_workspace(CompilerPruneOptions {
        workspace_root,
        output_root: output_root.clone(),
    })?;

    println!("compiler-prune root: {}", report.root);
    println!("compiler-prune files written: {}", report.files_written);
    println!("compiler-prune packages: {}", report.packages.join(", "));
    println!(
        "compiler-prune demoted visibilities: {}",
        report.demoted_visibilities
    );
    println!("compiler-prune linted crate roots:");
    for path in &report.linted_roots {
        println!("  {}", path.display());
    }

    let mut rounds = Vec::new();
    let mut final_output = run_cargo_check_json(&output_root)?;
    let mut diagnostics = lint_audit_diagnostics(&final_output.stdout);
    for round in 0..12 {
        let removed = prune_dead_items_from_diagnostics(&output_root, &diagnostics)?;
        let redemoted = if removed == 0 {
            0
        } else {
            refresh_compiler_prune_visibility(CompilerPruneRefreshOptions {
                workspace_root: output_root.clone(),
            })?
            .demoted_visibilities
        };
        rounds.push(json!({
            "round": round,
            "cargo_status": final_output.status.code(),
            "diagnostic_count": diagnostics.len(),
            "removed_dead_items": removed,
            "redemoted_visibilities": redemoted,
        }));
        if removed == 0 && redemoted == 0 {
            break;
        }
        final_output = run_cargo_check_json(&output_root)?;
        diagnostics = lint_audit_diagnostics(&final_output.stdout);
    }

    let report_path = output_root.join("slicers-compiler-prune-report.json");
    let payload = json!({
        "root": report.root.to_string(),
        "packages": report.packages,
        "files_written": report.files_written,
        "demoted_visibilities": report.demoted_visibilities,
        "cargo_status": final_output.status.code(),
        "diagnostic_count": diagnostics.len(),
        "diagnostics": diagnostics,
        "rounds": rounds,
    });
    fs::write(&report_path, serde_json::to_string_pretty(&payload)?)?;

    println!(
        "compiler-prune diagnostics: {} ({})",
        payload["diagnostic_count"],
        report_path.display()
    );

    if !final_output.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&final_output.stderr));
    }

    if rust_analyzer.enabled {
        run_rust_analyzer_audit(&output_root, &rust_analyzer)?;
    }

    Ok(())
}

fn run_cargo_check_json(output_root: &Path) -> Result<Output, Box<dyn std::error::Error>> {
    let target_dir = std::env::var_os("SLICERS_CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or(std::env::current_dir()?.join("target/slicers-compiler-prune"));
    Ok(Command::new("cargo")
        .arg("check")
        .arg("--message-format=json")
        .arg("--manifest-path")
        .arg(output_root.join("Cargo.toml"))
        .env("CARGO_TARGET_DIR", target_dir)
        .output()?)
}

fn prune_dead_items_from_diagnostics(
    output_root: &Path,
    diagnostics: &[Value],
) -> Result<usize, Box<dyn std::error::Error>> {
    let mut by_file: BTreeMap<PathBuf, Vec<DeadItemCandidate>> = BTreeMap::new();
    for diagnostic in diagnostics {
        if diagnostic
            .get("code")
            .and_then(|code| code.get("code"))
            .and_then(Value::as_str)
            != Some("dead_code")
        {
            continue;
        }
        let Some(message) = diagnostic.get("message").and_then(Value::as_str) else {
            continue;
        };
        if !(message.contains("function `")
            || message.contains("method `")
            || message.contains("associated function `")
            || message.contains("associated constant `")
            || message.contains("constant `")
            || message.contains("enum `")
            || message.contains("module `")
            || message.contains("static `")
            || message.contains("struct `")
            || message.contains("type alias `")
            || message.contains("union `")
            || message.contains("trait `"))
        {
            continue;
        }
        let Some(name) = name_from_backticks(message) else {
            continue;
        };
        for span in diagnostic
            .get("spans")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let Some(file_name) = span.get("file_name").and_then(Value::as_str) else {
                continue;
            };
            let Some(line_start) = span.get("line_start").and_then(Value::as_u64) else {
                continue;
            };
            let path = diagnostic_path(output_root, file_name);
            by_file.entry(path).or_default().push(DeadItemCandidate {
                name: name.to_string(),
                line_start: line_start as usize,
            });
        }
    }

    let mut removed = 0;
    for (path, mut candidates) in by_file {
        if !path.exists() {
            continue;
        }
        candidates.sort_by(|left, right| right.line_start.cmp(&left.line_start));
        candidates.dedup();
        let mut source = fs::read_to_string(&path)?;
        for candidate in candidates {
            if remove_item_at_line(&mut source, &candidate) {
                removed += 1;
            }
        }
        fs::write(path, source)?;
    }
    Ok(removed)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DeadItemCandidate {
    name: String,
    line_start: usize,
}

fn diagnostic_path(output_root: &Path, file_name: &str) -> PathBuf {
    let path = PathBuf::from(file_name);
    if path.is_absolute() {
        path
    } else {
        output_root.join(path)
    }
}

fn name_from_backticks(message: &str) -> Option<&str> {
    let (_, rest) = message.split_once('`')?;
    let (name, _) = rest.split_once('`')?;
    Some(name)
}

fn remove_item_at_line(source: &mut String, candidate: &DeadItemCandidate) -> bool {
    let mut lines = source.lines().map(str::to_string).collect::<Vec<_>>();
    let Some(mut start) = candidate.line_start.checked_sub(1) else {
        return false;
    };
    if start >= lines.len() {
        return false;
    }

    let needle = candidate.name.clone();
    let raw_needle = format!("r#{}", candidate.name);
    let search_floor = start.saturating_sub(8);
    while start > search_floor
        && !lines[start].contains(&needle)
        && !lines[start].contains(&raw_needle)
    {
        start -= 1;
    }
    if !lines[start].contains(&needle) && !lines[start].contains(&raw_needle) {
        return false;
    }

    while start > 0 {
        let previous = lines[start - 1].trim_start();
        if previous.starts_with("#[") || previous.starts_with("///") {
            start -= 1;
        } else {
            break;
        }
    }

    let Some(header_end) = (start..lines.len())
        .find(|index| lines[*index].contains('{') || lines[*index].trim_end().ends_with(';'))
    else {
        return false;
    };
    if lines[header_end].trim_end().ends_with(';') && !lines[header_end].contains('{') {
        lines.drain(start..=header_end);
        *source = lines.join("\n");
        source.push('\n');
        return true;
    }

    let mut depth = 0isize;
    let mut saw_open = false;
    let mut end = None;
    for (index, line) in lines.iter().enumerate().skip(header_end) {
        for character in line.chars() {
            match character {
                '{' => {
                    saw_open = true;
                    depth += 1;
                }
                '}' if saw_open => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(index);
                        break;
                    }
                }
                _ => {}
            }
        }
        if end.is_some() {
            break;
        }
    }
    let Some(end) = end else {
        return false;
    };

    lines.drain(start..=end);
    *source = lines.join("\n");
    source.push('\n');
    true
}

#[derive(Clone, Debug, Default)]
struct RustAnalyzerOptions {
    enabled: bool,
    binary: Option<PathBuf>,
    proc_macro_srv: Option<PathBuf>,
    disable_build_scripts: bool,
    disable_proc_macros: bool,
}

fn run_lint_audit(
    workspace_root: PathBuf,
    output_root: PathBuf,
    rust_analyzer: RustAnalyzerOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let report = generate_lint_audit_workspace(LintAuditOptions {
        workspace_root,
        output_root: output_root.clone(),
    })?;

    println!("lint-audit files written: {}", report.files_written);
    println!("lint-audit packages: {}", report.packages.join(", "));
    println!("lint-audit injected crate roots:");
    for path in &report.linted_roots {
        println!("  {}", path.display());
    }

    let output = Command::new("cargo")
        .arg("check")
        .arg("--message-format=json")
        .arg("--manifest-path")
        .arg(output_root.join("Cargo.toml"))
        .output()?;

    let diagnostics = lint_audit_diagnostics(&output.stdout);
    let report_path = output_root.join("slicers-lint-audit-report.json");
    let payload = json!({
        "cargo_status": output.status.code(),
        "diagnostic_count": diagnostics.len(),
        "diagnostics": diagnostics,
    });
    fs::write(&report_path, serde_json::to_string_pretty(&payload)?)?;

    println!(
        "lint-audit diagnostics: {} ({})",
        payload["diagnostic_count"],
        report_path.display()
    );

    if !output.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }

    if rust_analyzer.enabled {
        run_rust_analyzer_audit(&output_root, &rust_analyzer)?;
    }

    Ok(())
}

fn run_rust_analyzer_audit(
    output_root: &PathBuf,
    options: &RustAnalyzerOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let binary = rust_analyzer_binary(options);
    let availability = Command::new(&binary).arg("--version").output();
    let Ok(availability) = availability else {
        write_rust_analyzer_report(
            output_root,
            &binary,
            json!({
                "available": false,
                "error": "failed to start rust-analyzer",
            }),
        )?;
        println!(
            "rust-analyzer audit unavailable: failed to start {}",
            binary.display()
        );
        return Ok(());
    };

    if !availability.status.success() {
        write_rust_analyzer_report(
            output_root,
            &binary,
            json!({
                "available": false,
                "status": availability.status.code(),
                "stdout": String::from_utf8_lossy(&availability.stdout),
                "stderr": String::from_utf8_lossy(&availability.stderr),
            }),
        )?;
        println!(
            "rust-analyzer audit unavailable: {} --version failed",
            binary.display()
        );
        return Ok(());
    }

    let diagnostics = run_rust_analyzer_command(&binary, "diagnostics", output_root, options)?;
    let unresolved =
        run_rust_analyzer_command(&binary, "unresolved-references", output_root, options)?;
    let payload = json!({
        "available": true,
        "binary": path_to_json_string(binary.as_os_str()),
        "version": String::from_utf8_lossy(&availability.stdout).trim(),
        "diagnostics": diagnostics,
        "unresolved_references": unresolved,
    });
    write_rust_analyzer_report(output_root, &binary, payload)?;
    println!(
        "rust-analyzer audit written: {}",
        output_root
            .join("slicers-rust-analyzer-report.json")
            .display()
    );
    Ok(())
}

fn run_rust_analyzer_command(
    binary: &PathBuf,
    subcommand: &str,
    output_root: &PathBuf,
    options: &RustAnalyzerOptions,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut command = Command::new(binary);
    command.arg(subcommand).arg(output_root);
    if options.disable_build_scripts {
        command.arg("--disable-build-scripts");
    }
    if options.disable_proc_macros {
        command.arg("--disable-proc-macros");
    }
    if !options.disable_proc_macros {
        if let Some(proc_macro_srv) = rust_analyzer_proc_macro_srv(options) {
            command.arg("--proc-macro-srv").arg(proc_macro_srv);
        }
    }

    let output = command.output()?;
    Ok(json!({
        "status": output.status.code(),
        "stdout": String::from_utf8_lossy(&output.stdout),
        "stderr": String::from_utf8_lossy(&output.stderr),
    }))
}

fn rust_analyzer_binary(options: &RustAnalyzerOptions) -> PathBuf {
    if let Some(binary) = &options.binary {
        return binary.clone();
    }
    if let Some(binary) = std::env::var_os("SLICERS_RUST_ANALYZER") {
        return PathBuf::from(binary);
    }
    PathBuf::from("rust-analyzer")
}

fn rust_analyzer_proc_macro_srv(options: &RustAnalyzerOptions) -> Option<PathBuf> {
    if let Some(proc_macro_srv) = &options.proc_macro_srv {
        return Some(proc_macro_srv.clone());
    }
    std::env::var_os("SLICERS_RA_PROC_MACRO_SRV").map(PathBuf::from)
}

fn write_rust_analyzer_report(
    output_root: &PathBuf,
    binary: &PathBuf,
    mut payload: Value,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(object) = payload.as_object_mut() {
        object.insert(
            "binary".to_string(),
            Value::String(path_to_json_string(binary.as_os_str())),
        );
    }
    fs::write(
        output_root.join("slicers-rust-analyzer-report.json"),
        serde_json::to_string_pretty(&payload)?,
    )?;
    Ok(())
}

fn path_to_json_string(path: &OsStr) -> String {
    path.to_string_lossy().into_owned()
}

fn lint_audit_diagnostics(stdout: &[u8]) -> Vec<Value> {
    let interesting_codes = [
        "dead_code",
        "unused_imports",
        "unused_macros",
        "unreachable_pub",
    ];
    String::from_utf8_lossy(stdout)
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter(|value| value.get("reason").and_then(Value::as_str) == Some("compiler-message"))
        .filter_map(|value| value.get("message").cloned())
        .filter(|message| {
            let level = message.get("level").and_then(Value::as_str);
            let code = message
                .get("code")
                .and_then(|code| code.get("code"))
                .and_then(Value::as_str);
            level == Some("error")
                || (level == Some("warning")
                    && code.is_some_and(|code| interesting_codes.contains(&code)))
        })
        .map(|message| {
            let primary_spans = message
                .get("spans")
                .and_then(Value::as_array)
                .map(|spans| {
                    spans
                        .iter()
                        .filter(|span| {
                            span.get("is_primary").and_then(Value::as_bool) == Some(true)
                        })
                        .cloned()
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            json!({
                "code": message.get("code").cloned().unwrap_or(Value::Null),
                "level": message.get("level").cloned().unwrap_or(Value::Null),
                "message": message.get("message").cloned().unwrap_or(Value::Null),
                "spans": primary_spans,
            })
        })
        .collect()
}

fn usage() -> String {
    [
        "usage: slicers [--check] [--lint-audit] [--ra-audit]",
        "               [--compiler-prune]",
        "               [--rust-analyzer <path>]",
        "               [--ra-proc-macro-srv <path>]",
        "               [--ra-disable-build-scripts] [--ra-disable-proc-macros]",
        "               <workspace-root> <output-root>",
    ]
    .join("\n")
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
