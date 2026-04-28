use std::path::PathBuf;

use opensource_core::{generate, GenerateOptions};

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let workspace_root = args
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| "usage: opensource-reduce <workspace-root> <output-root>".to_string())?;
    let output_root = args
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| "usage: opensource-reduce <workspace-root> <output-root>".to_string())?;

    if args.next().is_some() {
        return Err("usage: opensource-reduce <workspace-root> <output-root>".into());
    }

    let report = generate(GenerateOptions {
        workspace_root,
        output_root,
    })?;

    println!("root: {}", report.root);
    println!("packages: {}", report.packages.join(", "));
    println!("reachable callables:");
    for callable in report.reachable {
        println!("  {callable}");
    }

    Ok(())
}
