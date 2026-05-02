use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use opensource_core::{generate, GenerateOptions};

#[test]
fn retains_unqualified_const_patterns_in_match_arms() {
    let workspace = temp_path("workspace");
    let output = temp_path("output");
    let target_dir = temp_path("target");
    write_fixture_workspace(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace.clone(),
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let reachable_items = report
        .reachable_items
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    for expected in ["app::KEY_ALPHA(Const)", "app::KEY_BETA(Const)"] {
        assert!(
            reachable_items.iter().any(|actual| actual == expected),
            "missing reachable const pattern {expected}; got {reachable_items:?}",
        );
    }
    assert!(
        !reachable_items
            .iter()
            .any(|actual| actual == "app::KEY_UNUSED(Const)"),
        "unreachable const was retained: {reachable_items:?}",
    );

    let source = read(output.join("app/src/lib.rs"));
    assert!(source.contains("const KEY_ALPHA"));
    assert!(source.contains("const KEY_BETA"));
    assert!(!source.contains("KEY_UNUSED"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");

    assert!(
        cargo_check.status.success(),
        "generated workspace did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\napp/src/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        source,
    );
}

fn write_fixture_workspace(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }

    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[workspace]
members = ["app"]
resolver = "2"

[workspace.dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );

    write(
        root.join("app/Cargo.toml"),
        r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced.workspace = true
"#,
    );

    write(
        root.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

const KEY_ALPHA: &str = "alpha";
const KEY_BETA: &str = "beta";
const KEY_UNUSED: &str = "unused";

#[opensourced]
pub fn classify(key: &str) -> u8 {
    match key {
        KEY_ALPHA => 1,
        KEY_BETA => beta_score(),
        _ => 0,
    }
}

fn beta_score() -> u8 {
    2
}

fn dead_score() -> u8 {
    KEY_UNUSED.len() as u8
}
"#,
    );
}

fn write(path: PathBuf, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

fn read(path: PathBuf) -> String {
    fs::read_to_string(path).unwrap()
}

fn temp_path(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("opensourced-pattern-constants-{label}-{unique}"))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn manifest_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "\\\\")
}
