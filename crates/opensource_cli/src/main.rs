use std::{ffi::OsStr, fs, path::PathBuf, process::Command};

use opensource_core::{generate, generate_lint_audit_workspace, GenerateOptions, LintAuditOptions};
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
    let mut positional = Vec::new();
    for arg in std::env::args_os().skip(1) {
        if arg == OsStr::new("--check") {
            run_check = true;
        } else if arg == OsStr::new("--lint-audit") {
            lint_audit = true;
        } else if arg == OsStr::new("--help") || arg == OsStr::new("-h") {
            println!("{}", usage());
            return Ok(());
        } else {
            positional.push(PathBuf::from(arg));
        }
    }

    let [workspace_root, output_root] = positional.as_slice() else {
        return Err(usage().into());
    };

    if same_path(workspace_root, output_root) {
        return Err("output root must be different from workspace root".into());
    }

    if lint_audit {
        run_lint_audit(workspace_root.clone(), output_root.clone())?;
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

    Ok(())
}

fn run_lint_audit(
    workspace_root: PathBuf,
    output_root: PathBuf,
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

    Ok(())
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
            matches!(level, Some("warning") | Some("error"))
                && code.is_some_and(|code| interesting_codes.contains(&code))
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
    "usage: slicers [--check] [--lint-audit] <workspace-root> <output-root>".to_string()
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
