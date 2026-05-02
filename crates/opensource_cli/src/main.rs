use std::{ffi::OsStr, path::PathBuf, process::Command};

use opensource_core::{generate, GenerateOptions};

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut run_check = false;
    let mut positional = Vec::new();
    for arg in std::env::args_os().skip(1) {
        if arg == OsStr::new("--check") {
            run_check = true;
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

fn usage() -> String {
    "usage: slicers [--check] <workspace-root> <output-root>".to_string()
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
