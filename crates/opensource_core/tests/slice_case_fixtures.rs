use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use opensource_core::{generate_with_analyzer, AnalyzerMode, GenerateOptions};

#[test]
fn trims_unused_checked_fixture_workspace_with_default_analyzer() {
    let fixture = repo_root().join("fixtures/slice_cases/trim_unused");
    let output = temp_path("slice-case-trim-unused-output");
    let target_dir = temp_path("slice-case-trim-unused-target");

    let report = generate_with_analyzer(
        GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        },
        AnalyzerMode::default_for_build(),
    )
    .expect("trim_unused fixture should slice");

    assert_eq!(report.packages, ["root", "used"]);
    assert!(
        report
            .roots
            .iter()
            .any(|root| root.to_string() == "root::selected_total"),
        "selected root should be recorded: {:?}",
        report.roots
    );

    let root_manifest = read(output.join("root/Cargo.toml"));
    let root_source = read(output.join("root/src/lib.rs"));
    let used_root = read(output.join("used/src/lib.rs"));
    let used_live = read(output.join("used/src/live.rs"));
    let workspace = read(output.join("Cargo.toml"));

    assert!(workspace.contains("members = ["), "{workspace}");
    assert!(workspace.contains("\"root\""), "{workspace}");
    assert!(workspace.contains("\"used\""), "{workspace}");
    assert!(!workspace.contains("\"unused\""), "{workspace}");
    assert!(root_manifest.contains("used"), "{root_manifest}");
    assert!(!root_manifest.contains("unused"), "{root_manifest}");
    assert!(
        root_source.contains("pub fn selected_total"),
        "{root_source}"
    );
    assert!(root_source.contains("used::format_live"), "{root_source}");
    assert!(!root_source.contains("dead_entry"), "{root_source}");
    assert!(
        !root_source.contains("unused::format_dead"),
        "{root_source}"
    );

    assert!(used_root.contains("mod live"), "{used_root}");
    assert!(
        used_root.contains("pub use live::{format_live, LiveRecord}"),
        "{used_root}"
    );
    assert!(!used_root.contains("mod dead"), "{used_root}");
    assert!(!used_root.contains("DeadRecord"), "{used_root}");
    assert!(used_live.contains("pub struct LiveRecord"), "{used_live}");
    assert!(used_live.contains("pub fn new"), "{used_live}");
    assert!(used_live.contains("pub fn render"), "{used_live}");
    assert!(used_live.contains("pub fn format_live"), "{used_live}");
    assert!(used_live.contains("fn normalize"), "{used_live}");
    assert!(!used_live.contains("dead_method"), "{used_live}");
    assert!(!used_live.contains("dead_live_helper"), "{used_live}");
    assert!(!output.join("used/src/dead.rs").exists());
    assert!(!output.join("unused").exists());

    assert_cargo_check(&output, &target_dir, &root_source);
}

fn assert_cargo_check(workspace: &Path, target_dir: &Path, root_source: &str) {
    let output = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(workspace)
        .env("CARGO_TARGET_DIR", target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        output.status.success(),
        "generated trim_unused fixture slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nroot/src/lib.rs:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
        root_source,
    );
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root should exist")
        .to_path_buf()
}

fn temp_path(label: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    path.push(format!("slicer-{label}-{nanos}"));
    if path.exists() {
        fs::remove_dir_all(&path).expect("old temp directory should remove");
    }
    path
}

fn read(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path.as_ref())
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.as_ref().display()))
}
