use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use opensource_core::{check_workspace, generate, CheckOptions, GenerateOptions};

#[test]
fn cargo_metadata_workspace_excludes_define_loaded_members() {
    let workspace = temp_path("metadata-exclude-workspace");
    let output = temp_path("metadata-exclude-output");
    let target_dir = temp_path("metadata-exclude-target");
    write_metadata_exclude_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    assert_eq!(report.packages, ["app"]);
    assert!(output.join("app/src/lib.rs").exists());
    assert!(
        !output.join("dead").exists(),
        "workspace.exclude members must not be rendered"
    );

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated metadata-exclude slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
    );
}

#[test]
fn copies_workspace_cargo_config_and_feedback_uses_generated_cargo_context() {
    let workspace = temp_path("cargo-config-workspace");
    let output = temp_path("cargo-config-output");
    let target_dir = temp_path("cargo-config-target");
    write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    );
    write(
        workspace.join(".cargo/config.toml"),
        r#"[build]
rustflags = ["--cfg", "slicers_config_probe", "--check-cfg=cfg(slicers_config_probe)"]
"#,
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&repo_root().join("crates/opensourced"))
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[cfg(not(slicers_config_probe))]
compile_error!("workspace cargo config was not applied");

#[opensourced]
pub fn selected() -> i32 {
    7
}
"#,
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    assert!(output.join(".cargo/config.toml").exists());
    let report = check_workspace(CheckOptions {
        manifest_path: output.join("Cargo.toml"),
        target_dir: Some(target_dir),
        timeout: Some(Duration::from_secs(60)),
        cargo_args: Vec::new(),
    })
    .expect("generated workspace cargo check should run");

    assert!(
        report.success,
        "generated cargo check should use copied .cargo/config.toml\nstderr:\n{}",
        report.stderr
    );
    assert_eq!(report.working_dir.as_deref(), Some(output.as_path()));
}

#[test]
fn copies_workspace_rust_toolchain_file() {
    let workspace = temp_path("toolchain-workspace");
    let output = temp_path("toolchain-output");
    write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    );
    write(
        workspace.join("rust-toolchain.toml"),
        &format!("[toolchain]\nchannel = {:?}\n", current_rustup_toolchain()),
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&repo_root().join("crates/opensourced"))
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected() -> i32 {
    7
}
"#,
    );

    generate(GenerateOptions {
        workspace_root: workspace.clone(),
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    assert_eq!(
        read(output.join("rust-toolchain.toml")),
        read(workspace.join("rust-toolchain.toml"))
    );
}

#[test]
fn preserves_cargo_lint_policy_for_generated_validation() {
    let workspace = temp_path("lint-policy-workspace");
    let output = temp_path("lint-policy-output");
    let target_dir = temp_path("lint-policy-target");
    write(
        workspace.join("Cargo.toml"),
        r#"[workspace]
members = ["app"]
resolver = "2"

[workspace.lints.rust]
unsafe_code = "deny"
"#,
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}

[lints]
workspace = true
"#,
            manifest_path(&repo_root().join("crates/opensourced"))
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected() -> i32 {
    unsafe { 7 }
}
"#,
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let root_manifest = read(output.join("Cargo.toml"));
    let package_manifest = read(output.join("app/Cargo.toml"));
    assert!(root_manifest.contains("[workspace.lints.rust]"));
    assert!(root_manifest.contains("unsafe_code = \"deny\""));
    assert!(package_manifest.contains("[lints]"));
    assert!(package_manifest.contains("workspace = true"));

    let report = check_workspace(CheckOptions {
        manifest_path: output.join("Cargo.toml"),
        target_dir: Some(target_dir),
        timeout: Some(Duration::from_secs(60)),
        cargo_args: Vec::new(),
    })
    .expect("generated workspace cargo check should run");

    let diagnostic_text = report
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !report.success && diagnostic_text.contains("unsafe"),
        "generated cargo check should enforce preserved unsafe_code lint\ndiagnostics:\n{}\nstderr:\n{}",
        diagnostic_text,
        report.stderr
    );
}

#[test]
fn preserves_workspace_profile_policy() {
    let workspace = temp_path("profile-policy-workspace");
    let output = temp_path("profile-policy-output");
    write(
        workspace.join("Cargo.toml"),
        r#"[workspace]
members = ["app"]
resolver = "2"

[profile.dev]
panic = "abort"
overflow-checks = false
"#,
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&repo_root().join("crates/opensourced"))
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected() -> i32 {
    7
}
"#,
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let manifest = read(output.join("Cargo.toml"));
    assert!(manifest.contains("[profile.dev]"));
    assert!(manifest.contains("panic = \"abort\""));
    assert!(manifest.contains("overflow-checks = false"));
}

#[cfg(unix)]
#[test]
fn source_include_symlink_inside_package_is_copied_at_link_path() {
    let workspace = temp_path("source-include-internal-link-workspace");
    let output = temp_path("source-include-internal-link-output");
    write_basic_workspace(&workspace);
    write_basic_app_manifest(&workspace);
    write(
        workspace.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected() -> usize {
    include_str!("data-link.txt").len()
}
"#,
    );
    write(
        workspace.join("app/data/message.txt"),
        "internal package data",
    );
    symlink_file(
        &workspace.join("app/data/message.txt"),
        &workspace.join("app/src/data-link.txt"),
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    assert_eq!(
        read(output.join("app/src/data-link.txt")),
        "internal package data"
    );
}

#[cfg(unix)]
#[test]
fn source_include_symlink_outside_package_is_not_copied() {
    let workspace = temp_path("source-include-external-link-workspace");
    let output = temp_path("source-include-external-link-output");
    let external = temp_path("source-include-external-link-secret");
    write_basic_workspace(&workspace);
    write_basic_app_manifest(&workspace);
    write(
        workspace.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected() -> usize {
    include_str!("secret-link.txt").len()
}
"#,
    );
    write(external.join("secret.txt"), "external secret");
    symlink_file(
        &external.join("secret.txt"),
        &workspace.join("app/src/secret-link.txt"),
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    assert!(!output.join("app/src/secret-link.txt").exists());
}

#[cfg(unix)]
#[test]
fn build_script_asset_symlink_outside_package_is_not_copied() {
    let workspace = temp_path("build-asset-external-link-workspace");
    let output = temp_path("build-asset-external-link-output");
    let external = temp_path("build-asset-external-link-secret");
    write_basic_workspace(&workspace);
    write_basic_app_manifest(&workspace);
    write(
        workspace.join("app/build.rs"),
        r#"fn main() {
    let _ = std::fs::read_to_string("asset-link.txt");
}
"#,
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected() -> i32 {
    7
}
"#,
    );
    write(external.join("secret.txt"), "external build secret");
    symlink_file(
        &external.join("secret.txt"),
        &workspace.join("app/asset-link.txt"),
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    assert!(output.join("app/build.rs").exists());
    assert!(!output.join("app/asset-link.txt").exists());
}

#[test]
fn slices_single_package_binary_crate_with_stub_main() {
    let workspace = temp_path("single-workspace");
    let output = temp_path("single-output");
    let target_dir = temp_path("single-target");
    write_single_binary_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    assert_eq!(report.packages, ["rtk_like"]);

    let source = read(output.join("rtk_like/src/main.rs"));
    assert!(source.contains("fn selected"));
    assert!(source.contains("fn helper"));
    assert!(source.contains("fn main()"));
    assert!(!source.contains("noisy_entrypoint"));
    assert!(!source.contains("#[opensourced]"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated binary slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/main.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        source,
    );
}

#[test]
fn slices_marked_bin_target_when_package_also_has_lib_target() {
    let workspace = temp_path("bin-with-lib-workspace");
    let output = temp_path("bin-with-lib-output");
    let target_dir = temp_path("bin-with-lib-target");
    write_bin_with_lib_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    assert_eq!(report.packages, ["app"]);
    let source = read(output.join("app/src/bin/tool/main.rs"));
    let command = read(output.join("app/src/bin/tool/command.rs"));
    assert!(source.contains("mod command"));
    assert!(source.contains("fn main()"));
    assert!(command.contains("pub fn selected"));
    assert!(command.contains("fn helper"));
    assert!(!source.contains("dead_lib"));
    assert!(!source.contains("dead_bin"));
    assert!(!command.contains("dead_command"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated bin-with-lib slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/bin/tool/main.rs:\n{}\nsrc/bin/tool/command.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        source,
        command,
    );
}

#[test]
fn slices_marked_example_target_and_preserves_manifest_entry() {
    let workspace = temp_path("example-target-workspace");
    let output = temp_path("example-target-output");
    let target_dir = temp_path("example-target-build");
    write_example_target_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    assert_eq!(report.packages, ["app", "lib_support", "support"]);
    let app_target = report
        .targets
        .iter()
        .find(|target| target.package == "app")
        .expect("app target should be reported");
    assert_eq!(app_target.required_features, ["demo-mode"]);
    assert_eq!(app_target.default_features, ["demo-mode"]);
    let manifest = read(output.join("app/Cargo.toml"));
    assert!(manifest.contains("[[example]]"));
    assert!(manifest.contains("name = \"demo\""));
    assert!(manifest.contains("path = \"examples/demo.rs\""));
    assert!(manifest.contains("required-features = [\"demo-mode\"]"));
    assert!(manifest.contains("[dependencies.lib_support]"));
    assert!(manifest.contains("[dev-dependencies.support]"));

    let source = read(output.join("app/examples/demo.rs"));
    let lib = read(output.join("app/src/lib.rs"));
    let lib_support = read(output.join("lib_support/src/lib.rs"));
    let support = read(output.join("support/src/lib.rs"));
    assert!(lib.contains("pub fn library_label"));
    assert!(lib_support.contains("pub fn label"));
    assert!(source.contains("fn main()"));
    assert!(source.contains("pub fn selected"));
    assert!(support.contains("pub fn format_value"));
    assert!(!source.contains("dead_example"));
    assert!(!support.contains("dead_support"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .arg("--package")
        .arg("app")
        .arg("--example")
        .arg("demo")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated example-target slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\napp/Cargo.toml:\n{}\napp/src/lib.rs:\n{}\napp/examples/demo.rs:\n{}\nlib_support/src/lib.rs:\n{}\nsupport/src/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        manifest,
        lib,
        source,
        lib_support,
        support,
    );
}

#[test]
fn slices_marked_integration_test_target_and_preserves_manifest_entry() {
    let workspace = temp_path("test-target-workspace");
    let output = temp_path("test-target-output");
    let target_dir = temp_path("test-target-build");
    write_test_target_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    assert_eq!(
        report.packages,
        ["app", "lib_support", "platform_support", "support"]
    );
    let manifest = read(output.join("app/Cargo.toml"));
    assert!(manifest.contains("[[test]]"));
    assert!(manifest.contains("name = \"behavior\""));
    assert!(manifest.contains("path = \"tests/behavior.rs\""));
    assert!(manifest.contains("[dependencies.lib_support]"));
    assert!(manifest.contains("[dev-dependencies.support]"));
    assert!(manifest.contains("[target.\"cfg(unix)\".dev-dependencies.platform_support]"));

    let source = read(output.join("app/tests/behavior.rs"));
    let lib = read(output.join("app/src/lib.rs"));
    let lib_support = read(output.join("lib_support/src/lib.rs"));
    let platform_support = read(output.join("platform_support/src/lib.rs"));
    let support = read(output.join("support/src/lib.rs"));
    assert!(lib.contains("pub fn library_label"));
    assert!(lib_support.contains("pub fn label"));
    assert!(platform_support.contains("pub fn format_value"));
    assert!(source.contains("#[test]"));
    assert!(source.contains("fn selected_behavior"));
    assert!(support.contains("pub fn format_value"));
    assert!(!source.contains("dead_test"));
    assert!(!support.contains("dead_support"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .arg("--package")
        .arg("app")
        .arg("--test")
        .arg("behavior")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated test-target slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\napp/Cargo.toml:\n{}\napp/src/lib.rs:\n{}\napp/tests/behavior.rs:\n{}\nlib_support/src/lib.rs:\n{}\nplatform_support/src/lib.rs:\n{}\nsupport/src/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        manifest,
        lib,
        source,
        lib_support,
        platform_support,
        support,
    );
}

#[test]
fn slices_marked_bench_target_and_preserves_manifest_entry() {
    let workspace = temp_path("bench-target-workspace");
    let output = temp_path("bench-target-output");
    let target_dir = temp_path("bench-target-build");
    write_bench_target_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    assert_eq!(report.packages, ["app", "lib_support", "support"]);
    let manifest = read(output.join("app/Cargo.toml"));
    assert!(manifest.contains("[[bench]]"));
    assert!(manifest.contains("name = \"throughput\""));
    assert!(manifest.contains("path = \"benches/throughput.rs\""));
    assert!(manifest.contains("[dependencies.lib_support]"));
    assert!(manifest.contains("[dev-dependencies.support]"));

    let source = read(output.join("app/benches/throughput.rs"));
    let lib = read(output.join("app/src/lib.rs"));
    let lib_support = read(output.join("lib_support/src/lib.rs"));
    let support = read(output.join("support/src/lib.rs"));
    assert!(lib.contains("pub fn library_label"));
    assert!(lib_support.contains("pub fn label"));
    assert!(source.contains("pub fn selected_bench_value"));
    assert!(support.contains("pub fn format_value"));
    assert!(!source.contains("dead_bench"));
    assert!(!support.contains("dead_support"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .arg("--package")
        .arg("app")
        .arg("--bench")
        .arg("throughput")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated bench-target slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\napp/Cargo.toml:\n{}\napp/src/lib.rs:\n{}\napp/benches/throughput.rs:\n{}\nlib_support/src/lib.rs:\n{}\nsupport/src/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        manifest,
        lib,
        source,
        lib_support,
        support,
    );
}

#[test]
fn slices_path_attributed_external_modules() {
    let workspace = temp_path("path-attr-workspace");
    let output = temp_path("path-attr-output");
    let target_dir = temp_path("path-attr-target");
    write_path_attr_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    assert_eq!(report.packages, ["path_attr_like"]);
    let lib = read(output.join("path_attr_like/src/lib.rs"));
    let custom = read(output.join("path_attr_like/src/generated/custom.rs"));
    assert!(lib.contains("mod custom"));
    assert!(lib.contains("path = \"generated/custom.rs\""));
    assert!(custom.contains("pub fn selected"));
    assert!(custom.contains("fn helper"));
    assert!(!lib.contains("dead_lib"));
    assert!(!custom.contains("dead_custom"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated path-attr slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/lib.rs:\n{}\nsrc/generated/custom.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        lib,
        custom,
    );
}

#[test]
fn rejects_markers_spread_across_multiple_package_targets() {
    let workspace = temp_path("multi-target-marker-workspace");
    let output = temp_path("multi-target-marker-output");
    write_multi_target_marker_fixture(&workspace);

    let error = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output,
    })
    .expect_err("markers across multiple target roots should fail closed");

    assert!(
        error
            .to_string()
            .contains("multiple package targets contain #[opensourced] markers"),
        "unexpected error: {error}"
    );
}

#[test]
fn rejects_markers_spread_across_lib_and_integration_test_targets() {
    let workspace = temp_path("lib-test-marker-workspace");
    let output = temp_path("lib-test-marker-output");
    write_lib_and_test_marker_fixture(&workspace);

    let error = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output,
    })
    .expect_err("markers across lib and integration test roots should fail closed");

    assert!(
        error
            .to_string()
            .contains("multiple package targets contain #[opensourced] markers"),
        "unexpected error: {error}"
    );
}

#[test]
fn slices_multiple_marked_function_roots() {
    let workspace = temp_path("multi-root-workspace");
    let output = temp_path("multi-root-output");
    let target_dir = temp_path("multi-root-target");
    write_multi_root_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let roots = report
        .roots
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    assert_eq!(
        roots,
        [
            "multi_root_like::selected_a",
            "multi_root_like::selected_b",
            "multi_root_like::hidden::selected_hidden"
        ]
    );

    let source = read(output.join("multi_root_like/src/lib.rs"));
    assert!(source.contains("pub fn selected_a"));
    assert!(source.contains("pub fn selected_b"));
    assert!(source.contains("pub mod hidden"));
    assert!(source.contains("pub fn selected_hidden"));
    assert!(source.contains("fn helper_a"));
    assert!(source.contains("fn helper_b"));
    assert!(source.contains("fn helper_hidden"));
    assert!(!source.contains("dead_root"));
    assert!(!source.contains("dead_helper"));
    assert!(!source.contains("#[opensourced]"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated multi-root slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        source,
    );
}

#[test]
fn slices_marked_data_item_roots() {
    let workspace = temp_path("item-root-workspace");
    let output = temp_path("item-root-output");
    let target_dir = temp_path("item-root-target");
    write_item_root_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let roots = report
        .roots
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    assert_eq!(
        roots,
        [
            "item_root_like::Api(Struct)",
            "item_root_like::PrivateKind(Enum)"
        ]
    );

    let reachable_items = report
        .reachable_items
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    for expected in [
        "item_root_like::Api(Struct)",
        "item_root_like::Handler(Trait)",
        "item_root_like::Mode(Enum)",
        "item_root_like::PrivateKind(Enum)",
    ] {
        assert!(
            reachable_items.iter().any(|actual| actual == expected),
            "missing reachable item {expected}; got {reachable_items:?}",
        );
    }

    let source = read(output.join("item_root_like/src/lib.rs"));
    assert!(source.contains("#[allow(dead_code)]\npub struct Api"));
    assert!(source.contains("pub struct Api"));
    assert!(source.contains("state: PrivateState"));
    assert!(source.contains("struct PrivateState"));
    assert!(source.contains("pub enum Mode"));
    assert!(source.contains("#[allow(dead_code)]\nenum PrivateKind"));
    assert!(source.contains("pub trait Handler"));
    assert!(!source.contains("Dead"));
    assert!(!source.contains("#[opensourced]"));
    assert!(!source.contains("opensourced::opensourced"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated item-root slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        source,
    );
}

#[test]
fn slices_marked_module_roots() {
    let workspace = temp_path("module-root-workspace");
    let output = temp_path("module-root-output");
    let target_dir = temp_path("module-root-target");
    write_module_root_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let roots = report
        .roots
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    assert_eq!(roots, ["module_root_like::api(Mod)"]);

    let source = read(output.join("module_root_like/src/lib.rs"));
    assert!(source.contains("pub mod api"));
    assert!(source.contains("pub struct Request"));
    assert!(source.contains("pub enum Mode"));
    assert!(source.contains("pub fn run"));
    assert!(source.contains("fn helper"));
    assert!(!source.contains("pub mod dead"));
    assert!(!source.contains("module_test_helper"));
    assert!(!source.contains("#[opensourced]"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated module-root slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        source,
    );
}

#[test]
fn slices_binary_child_module_with_generated_items_and_pruned_stub_imports() {
    let workspace = temp_path("child-binary-workspace");
    let output = temp_path("child-binary-output");
    let target_dir = temp_path("child-binary-target");
    write_child_binary_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    assert_eq!(report.packages, ["rtk_child_like"]);

    let main_source = read(output.join("rtk_child_like/src/main.rs"));
    assert!(main_source.contains("mod exposed"));
    assert!(main_source.contains("fn main()"));
    assert!(!main_source.contains("use noisy"));
    assert!(!output.join("rtk_child_like/src/noisy.rs").exists());

    let exposed_source = read(output.join("rtk_child_like/src/exposed.rs"));
    assert!(exposed_source.contains("macro_rules! define_generated"));
    assert!(exposed_source.contains("define_generated!(GENERATED_LABEL)"));
    assert!(exposed_source.contains("pub fn selected"));
    assert!(!exposed_source.contains("#[opensourced]"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated child binary slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/main.rs:\n{}\nsrc/exposed.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        main_source,
        exposed_source,
    );
}

#[test]
fn glob_reexports_only_pull_names_mentioned_by_reachable_code() {
    let workspace = temp_path("glob-reexport-workspace");
    let output = temp_path("glob-reexport-output");
    let target_dir = temp_path("glob-reexport-target");
    write_glob_reexport_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let reachable = report
        .reachable
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    assert!(
        !reachable
            .iter()
            .any(|callable| callable.contains("unused_api_function")),
        "glob import retained an unmentioned api function: {reachable:?}",
    );
    assert!(
        !reachable
            .iter()
            .any(|callable| callable.contains("unused_noise_function")),
        "glob import retained an unmentioned noise function: {reachable:?}",
    );

    let lib = read(output.join("glob_reexport_like/src/lib.rs"));
    let api = read(output.join("glob_reexport_like/src/api.rs"));
    assert!(lib.contains("pub use api::*"));
    assert!(!lib.contains("pub use noisy::*"));
    assert!(!lib.contains("pub mod noisy"));
    assert!(api.contains("pub struct Chosen"));
    assert!(!api.contains("UnusedApi"));
    assert!(!api.contains("unused_api_function"));
    assert!(!output.join("glob_reexport_like/src/noisy.rs").exists());

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated glob reexport slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/lib.rs:\n{}\nsrc/api.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        lib,
        api,
    );
}

#[test]
fn slices_trait_method_roots_without_marker_attrs_or_unrelated_bins() {
    let workspace = temp_path("trait-method-bin-workspace");
    let output = temp_path("trait-method-bin-output");
    let target_dir = temp_path("trait-method-bin-target");
    write_trait_method_binary_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let manifest = read(output.join("trait_method_bin_like/Cargo.toml"));
    let source = read(output.join("trait_method_bin_like/src/main.rs"));
    assert!(manifest.contains("name = \"trait-method-bin\""));
    assert!(!manifest.contains("other-bin"));
    assert!(!source.contains("#[opensourced]"));
    assert!(source.contains("impl Drop for TerminalGuard"));
    assert!(source.contains("fn main()"));
    assert!(!output.join("trait_method_bin_like/src/other.rs").exists());

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated trait-method binary slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nCargo.toml:\n{}\nsrc/main.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        manifest,
        source,
    );
}

#[test]
fn slices_automod_directory_modules_without_expanding_macro_usage() {
    let workspace = temp_path("automod-workspace");
    let output = temp_path("automod-output");
    let target_dir = temp_path("automod-target");
    write_automod_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let plugins_mod = read(output.join("automod_like/src/plugins/mod.rs"));
    assert!(plugins_mod.contains("automod::dir!(pub \"src/plugins\")"));
    assert!(!plugins_mod.contains("dead_slice"));
    assert!(output
        .join("automod_like/src/plugins/public_slice.rs")
        .exists());
    assert!(!output
        .join("automod_like/src/plugins/dead_slice.rs")
        .exists());

    let package_manifest = read(output.join("automod_like/Cargo.toml"));
    assert!(package_manifest.contains("automod ="));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated automod slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nplugins/mod.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        plugins_mod,
    );
}

#[test]
fn copies_assets_referenced_by_retained_include_macros() {
    let workspace = temp_path("include-assets-workspace");
    let output = temp_path("include-assets-output");
    let target_dir = temp_path("include-assets-target");
    write_include_assets_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let lib = read(output.join("include_assets_like/src/lib.rs"));
    assert!(lib.contains("include_str!(\"guidelines/core.md\")"));
    assert!(lib.contains("include_str!(\"../assets/extra.txt\")"));
    assert!(lib.contains("include_str!(concat!(\"guidelines/\", \"concat.md\"))"));
    assert!(lib.contains("env!(\"CARGO_MANIFEST_DIR\")"));
    assert!(lib.contains("\"/assets/manifest.txt\""));
    assert!(!lib.contains("UNUSED"));
    assert!(output
        .join("include_assets_like/src/guidelines/core.md")
        .exists());
    assert!(output
        .join("include_assets_like/src/guidelines/concat.md")
        .exists());
    assert!(output.join("include_assets_like/assets/extra.txt").exists());
    assert!(output
        .join("include_assets_like/assets/manifest.txt")
        .exists());
    assert!(!output
        .join("include_assets_like/src/guidelines/unused.md")
        .exists());

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated include-assets slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        lib,
    );
}

#[test]
fn prunes_unused_external_pub_reexport_names() {
    let workspace = temp_path("external-reexport-workspace");
    let output = temp_path("external-reexport-output");
    let target_dir = temp_path("external-reexport-target");
    write_external_pub_reexport_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let lib = read(output.join("external_reexport_like/src/lib.rs"));
    assert!(lib.contains("Value"));
    assert!(!lib.contains("Map"));
    assert!(!lib.contains("Number"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated external-reexport slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        lib,
    );
}

#[test]
fn prunes_public_glob_reexports_to_removed_local_modules() {
    let workspace = temp_path("public-glob-removed-module-workspace");
    let output = temp_path("public-glob-removed-module-output");
    let target_dir = temp_path("public-glob-removed-module-target");
    let opensourced_path = repo_root().join("crates/opensourced");

    write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

pub mod live;
pub mod removed;

pub use live::*;
pub use removed::*;

#[opensourced]
pub fn selected() -> usize {
    live::value()
}
"#,
    );
    write(
        workspace.join("app/src/live.rs"),
        r#"pub fn value() -> usize {
    7
}
"#,
    );
    write(
        workspace.join("app/src/removed.rs"),
        r#"pub struct Removed;
"#,
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let lib = read(output.join("app/src/lib.rs"));
    assert!(!lib.contains("pub mod removed"), "{lib}");
    assert!(!lib.contains("pub use removed::*"), "{lib}");

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .env("RUSTFLAGS", "-D warnings")
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated public glob removed module slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\napp/src/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        lib,
    );
}

#[test]
fn prunes_external_group_imports_shadowed_by_local_bindings() {
    let workspace = temp_path("external-shadowed-import-workspace");
    let output = temp_path("external-shadowed-import-output");
    let target_dir = temp_path("external-shadowed-import-target");
    write_external_shadowed_import_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let lib = read(output.join("external_shadowed_import_like/src/lib.rs"));
    assert!(
        !lib.contains("std::fmt") && !lib.contains("{fmt"),
        "local variable named like a removed external import must not retain the import\n{lib}",
    );
    assert!(
        lib.contains("PathBuf"),
        "live sibling import should remain\n{lib}",
    );

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated external-shadowed-import slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        lib,
    );
}

#[test]
fn retains_renamed_external_imports_used_only_by_retained_macro_invocations() {
    let workspace = temp_path("macro-rename-workspace");
    let output = temp_path("macro-rename-output");
    let target_dir = temp_path("macro-rename-target");
    write_macro_renamed_import_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let lib = read(output.join("macro_rename_like/src/lib.rs"));
    assert!(
        lib.contains("use serde_json::Value as JsonValue"),
        "macro-only renamed import should remain\n{lib}"
    );
    assert!(lib.contains("define_api"));
    assert!(lib.contains("generated"));
    assert!(!lib.contains("dead"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated macro rename slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        lib,
    );
}

#[test]
fn prunes_renamed_imports_when_resolved_target_is_removed() {
    let workspace = temp_path("removed-rename-workspace");
    let output = temp_path("removed-rename-output");
    let target_dir = temp_path("removed-rename-target");
    write_removed_renamed_import_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let lib = read(output.join("removed_rename_like/src/lib.rs"));
    assert!(lib.contains("pub fn selected"));
    assert!(lib.contains("let selected_value = 1"));
    assert!(!lib.contains("dead_fn as selected_value"));
    assert!(!lib.contains("dead_fn"));
    assert!(!lib.contains("mod dead"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated removed rename slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        lib,
    );
}

#[test]
fn retains_renamed_imports_used_after_inner_shadow() {
    let workspace = temp_path("shadowed-rename-workspace");
    let output = temp_path("shadowed-rename-output");
    let target_dir = temp_path("shadowed-rename-target");
    write_shadowed_renamed_import_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let lib = read(output.join("shadowed_rename_like/src/lib.rs"));
    assert!(lib.contains("pub fn selected"));
    assert!(lib.contains("live_fn as selected_value"));
    assert!(lib.contains("pub fn live_fn"));
    assert!(lib.contains("let selected_value = 1"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated shadowed rename slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        lib,
    );
}

#[test]
fn prunes_renamed_imports_shadowed_by_local_binding_when_target_is_retained_elsewhere() {
    let workspace = temp_path("retained-target-shadowed-rename-workspace");
    let output = temp_path("retained-target-shadowed-rename-output");
    let target_dir = temp_path("retained-target-shadowed-rename-target");
    write_retained_target_shadowed_renamed_import_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let feature = read(output.join("retained_target_shadowed_rename_like/src/feature.rs"));
    let other = read(output.join("retained_target_shadowed_rename_like/src/other.rs"));
    assert!(feature.contains("pub fn selected_feature"));
    assert!(feature.contains("let local_shadow = 1"));
    assert!(
        !feature.contains("OtherValue as local_shadow"),
        "local binding named like a renamed import alias must not retain the import\n{feature}",
    );
    assert!(
        other.contains("OtherValue"),
        "the aliased target should still be retained where it is actually used\n{other}",
    );

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated retained-target shadowed rename slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nfeature.rs:\n{}\nother.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        feature,
        other,
    );
}

#[test]
fn prunes_direct_imports_shadowed_by_local_binding_when_target_is_retained_elsewhere() {
    let workspace = temp_path("retained-target-shadowed-direct-workspace");
    let output = temp_path("retained-target-shadowed-direct-output");
    let target_dir = temp_path("retained-target-shadowed-direct-target");
    write_retained_target_shadowed_direct_import_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let feature = read(output.join("retained_target_shadowed_direct_like/src/feature.rs"));
    let other = read(output.join("retained_target_shadowed_direct_like/src/other.rs"));
    assert!(feature.contains("pub fn selected_feature"));
    assert!(feature.contains("let helper = 1"));
    assert!(
        !feature.contains("shared::{FeatureValue, helper}")
            && !feature.contains("shared::{helper, FeatureValue}"),
        "local binding named like a direct import leaf must not retain the import\n{feature}",
    );
    assert!(
        other.contains("helper()"),
        "the helper should still be retained where it is actually used\n{other}",
    );

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated retained-target shadowed direct slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nfeature.rs:\n{}\nother.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        feature,
        other,
    );
}

#[test]
fn retains_parent_imports_used_by_inline_child_super_glob() {
    let workspace = temp_path("inline-super-workspace");
    let output = temp_path("inline-super-output");
    let target_dir = temp_path("inline-super-target");
    write_inline_super_glob_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let lib = read(output.join("inline_super_like/src/lib.rs"));
    assert!(lib.contains("Path"));
    assert!(lib.contains("use super::*"));
    assert!(!lib.contains("PathBuf"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated inline super-glob slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        lib,
    );
}

#[test]
fn retains_string_literal_callback_paths_with_keyword_segments() {
    let workspace = temp_path("serde-callback-workspace");
    let output = temp_path("serde-callback-output");
    let target_dir = temp_path("serde-callback-target");
    write_serde_callback_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let lib = read(output.join("serde_callback_like/src/lib.rs"));
    assert!(lib.contains("fn skip_if_default"));
    assert!(lib.contains("crate::skip_if_default"));
    assert!(!lib.contains("pub fn dead"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated serde callback slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        lib,
    );
}

#[test]
fn retains_local_public_reexports_referenced_through_dependency_crate_paths() {
    let workspace = temp_path("public-reexport-workspace");
    let output = temp_path("public-reexport-output");
    let target_dir = temp_path("public-reexport-target");
    write_local_public_reexport_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let provider = read(output.join("provider/src/lib.rs"));
    assert!(provider.contains("mod error"));
    assert!(provider.contains("pub use crate::error::Error"));
    assert!(output.join("provider/src/error.rs").exists());

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated local public reexport slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nprovider/src/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        provider,
    );
}

#[test]
fn closes_public_reexports_referenced_from_retained_build_dependencies() {
    let workspace = temp_path("build-reexport-workspace");
    let output = temp_path("build-reexport-output");
    let target_dir = temp_path("build-reexport-target");
    write_build_dependency_public_reexport_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let provider = read(output.join("provider/src/lib.rs"));
    assert!(provider.contains("mod error"));
    assert!(provider.contains("pub use crate::error::Error"));
    assert!(output.join("provider/src/error.rs").exists());

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated build dependency public reexport slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nprovider/src/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        provider,
    );
}

#[test]
fn retains_macro_generated_public_reexports_referenced_through_dependency_crate_paths() {
    let workspace = temp_path("macro-public-reexport-workspace");
    let output = temp_path("macro-public-reexport-output");
    let target_dir = temp_path("macro-public-reexport-target");
    write_macro_generated_public_reexport_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let provider = read(output.join("provider/src/lib.rs"));
    let error = read(output.join("provider/src/error.rs"));
    let macro_provider = read(output.join("macro-provider/src/lib.rs"));
    assert!(provider.contains("mod error"));
    assert!(provider.contains("pub use crate::error::Error"));
    assert!(error.contains("pub enum ErrorKind"));
    assert!(error.contains("macro_provider::define_error!"));
    assert!(macro_provider.contains("macro_rules! define_error"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated macro public reexport slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nprovider/src/lib.rs:\n{}\nprovider/src/error.rs:\n{}\nmacro-provider/src/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        provider,
        error,
        macro_provider,
    );
}

#[test]
fn support_proc_macro_expansion_dependencies_are_scoped_to_live_export() {
    let workspace = temp_path("support-proc-macro-scope-workspace");
    let output = temp_path("support-proc-macro-scope-output");
    let target_dir = temp_path("support-proc-macro-scope-target");
    let helper = temp_path("support-proc-macro-scope-helper");
    let macros = temp_path("support-proc-macro-scope-macros");
    write_support_proc_macro_scope_fixture(&workspace, &helper, &macros);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("support proc macro scope fixture should reduce");

    let helper_lib = read(output.join("support/provider-helper/src/lib.rs"));
    let helper_helpers = read(output.join("support/provider-helper/src/helpers.rs"));
    assert!(helper_lib.contains("LiveRecord"), "{helper_lib}");
    assert!(!helper_lib.contains("DeadRecord"), "{helper_lib}");
    assert!(helper_lib.contains("LiveDerive"), "{helper_lib}");
    assert!(!helper_lib.contains("DeadDerive"), "{helper_lib}");
    assert!(helper_helpers.contains("live_helper"), "{helper_helpers}");
    assert!(!helper_helpers.contains("dead_helper"), "{helper_helpers}");

    fs::rename(&helper, helper.with_extension("moved"))
        .expect("original helper package should move away");
    fs::rename(&macros, macros.with_extension("moved"))
        .expect("original macro package should move away");
    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated support proc macro scope slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nhelper/src/lib.rs:\n{}\nhelper/src/helpers.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        helper_lib,
        helper_helpers,
    );
}

#[test]
fn slices_nested_modules_declared_from_file_modules() {
    let workspace = temp_path("nested-file-workspace");
    let output = temp_path("nested-file-output");
    let target_dir = temp_path("nested-file-target");
    write_nested_file_module_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    assert!(output.join("nested_file_like/src/feature.rs").exists());
    assert!(output
        .join("nested_file_like/src/feature/nested.rs")
        .exists());

    let feature = read(output.join("nested_file_like/src/feature.rs"));
    assert!(feature.contains("mod nested"));
    assert!(feature.contains("pub fn selected"));
    assert!(!feature.contains("#[opensourced]"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated nested file module slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nfeature.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        feature,
    );
}

#[test]
fn slices_explicit_nested_lib_path_modules() {
    let workspace = temp_path("explicit-lib-workspace");
    let output = temp_path("explicit-lib-output");
    let target_dir = temp_path("explicit-lib-target");
    write_explicit_nested_lib_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    assert!(output.join("explicit_lib_like/src/dump/lib.rs").exists());
    assert!(output.join("explicit_lib_like/src/dump/client.rs").exists());

    let lib = read(output.join("explicit_lib_like/src/dump/lib.rs"));
    assert!(lib.contains("pub mod client"));
    assert!(lib.contains("pub fn selected"));
    assert!(!lib.contains("#[opensourced]"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated explicit lib path slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/dump/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        lib,
    );
}

#[test]
fn slices_raw_identifier_file_modules() {
    let workspace = temp_path("raw-module-workspace");
    let output = temp_path("raw-module-output");
    let target_dir = temp_path("raw-module-target");
    write_raw_identifier_module_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let lib = read(output.join("raw_module_like/src/lib.rs"));
    assert!(lib.contains("mod r#type"));
    assert!(lib.contains("pub fn selected"));
    assert!(!lib.contains("#[opensourced]"));
    assert!(output.join("raw_module_like/src/type.rs").exists());

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated raw identifier module slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        lib,
    );
}

#[test]
fn slices_methods_marked_as_roots() {
    let workspace = temp_path("method-root-workspace");
    let output = temp_path("method-root-output");
    let target_dir = temp_path("method-root-target");
    write_method_root_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let lib = read(output.join("method_root_like/src/lib.rs"));
    assert!(lib.contains("pub fn selected"));
    assert!(lib.contains("fn helper"));
    assert!(!lib.contains("dead_method"));
    assert!(!lib.contains("dead_function"));
    assert!(!lib.contains("#[opensourced]"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated method-root slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        lib,
    );
}

#[test]
fn retains_trait_impls_required_by_derive_field_bounds() {
    let workspace = temp_path("derive-field-workspace");
    let output = temp_path("derive-field-output");
    let target_dir = temp_path("derive-field-target");
    write_derive_field_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let helper = read(output.join("derive_helper/src/lib.rs"));
    let formatting = read(output.join("derive_helper/src/formatting.rs"));
    assert!(formatting.contains("impl std::fmt::Debug for Guid"));
    assert!(helper.contains("impl Default for Guid"));
    assert!(formatting.contains("fn as_str") || helper.contains("fn as_str"));
    assert!(!helper.contains("dead_guid_method"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated derive-field slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nderive_helper/src/lib.rs:\n{}\nderive_helper/src/formatting.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        helper,
        formatting,
    );
}

#[test]
fn retains_default_impls_required_by_serde_default_fields() {
    let workspace = temp_path("serde-default-workspace");
    let output = temp_path("serde-default-output");
    let target_dir = temp_path("serde-default-target");
    write_serde_default_field_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let source = read(output.join("serde_default_like/src/lib.rs"));
    assert!(source.contains("impl Default for HomeSelection"));
    assert!(source.contains("#[serde(default)]"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated serde-default slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        source,
    );
}

#[test]
fn retains_deref_impls_for_autoderef_method_calls() {
    let workspace = temp_path("deref-method-workspace");
    let output = temp_path("deref-method-output");
    let target_dir = temp_path("deref-method-target");
    write_deref_method_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let lib = read(output.join("deref_method_like/src/lib.rs"));
    assert!(lib.contains("impl std::ops::Deref for Slug"));
    assert!(lib.contains("fn as_str"));
    assert!(!lib.contains("fn dead("));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated deref-method slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        lib,
    );
}

#[test]
fn retains_return_type_methods_and_trait_impls_for_dependency_items() {
    let workspace = temp_path("trait-return-workspace");
    let output = temp_path("trait-return-output");
    let target_dir = temp_path("trait-return-target");
    write_trait_return_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let helper = read(output.join("helper_dep/src/lib.rs"));
    assert!(helper.contains("impl Write for ByteCountWriter"));
    assert!(helper.contains("fn count"));
    assert!(helper.contains("fn as_some"));
    assert!(!helper.contains("dead_method"));
    assert!(!helper.contains("dead_function"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated trait/return slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nhelper_dep/src/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        helper,
    );
}

#[test]
fn retains_methods_called_through_borrowed_transparent_wrapper_return() {
    let workspace = temp_path("borrowed-wrapper-return-workspace");
    let output = temp_path("borrowed-wrapper-return-output");
    let target_dir = temp_path("borrowed-wrapper-return-target");
    let opensourced_path = repo_root().join("crates/opensourced");

    write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;
use std::sync::Arc;

pub struct PendingRequests {
    count: usize,
}

impl PendingRequests {
    pub fn new() -> Self {
        Self { count: 1 }
    }

    pub fn insert(&self, value: usize) -> usize {
        self.count + value
    }

    pub fn unused(&self) -> usize {
        0
    }
}

pub struct Connection {
    pending: Arc<PendingRequests>,
}

impl Connection {
    pub fn new() -> Self {
        Self {
            pending: Arc::new(PendingRequests::new()),
        }
    }

    pub fn pending(&self) -> &Arc<PendingRequests> {
        &self.pending
    }
}

#[opensourced]
pub fn selected() -> usize {
    let connection = Connection::new();
    connection.pending().insert(41)
}
"#,
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let source = read(output.join("app/src/lib.rs"));
    assert!(source.contains("pub fn insert"), "{source}");
    assert!(!source.contains("pub fn unused"), "{source}");

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .env("RUSTFLAGS", "-D warnings")
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated borrowed wrapper return slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\napp/src/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        source,
    );
}

#[test]
fn retains_display_impls_used_only_by_format_macro() {
    let workspace = temp_path("format-display-workspace");
    let output = temp_path("format-display-output");
    let target_dir = temp_path("format-display-target");
    write_format_display_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let helper = read(output.join("format_helper/src/lib.rs"));
    assert!(helper.contains("impl fmt::Display for Name"));
    assert!(helper.contains("fn as_str"));
    assert!(!helper.contains("dead_function"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated format display slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nformat_helper/src/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        helper,
    );
}

#[test]
fn retains_display_impls_used_by_to_string_method() {
    let workspace = temp_path("to-string-display-workspace");
    let output = temp_path("to-string-display-output");
    let target_dir = temp_path("to-string-display-target");
    write_to_string_display_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let helper = read(output.join("display_helper/src/lib.rs"));
    assert!(helper.contains("impl fmt::Display for CacheKey"));
    assert!(helper.contains("fn fmt"));
    assert!(!helper.contains("dead_function"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated to_string display slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\ndisplay_helper/src/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        helper,
    );
}

#[test]
fn slices_field_receivers_fold_builders_enum_patterns_and_parse_impls() {
    let workspace = temp_path("generic-inference-workspace");
    let output = temp_path("generic-inference-output");
    let target_dir = temp_path("generic-inference-target");
    write_generic_inference_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let model = read(output.join("model_dep/src/lib.rs"));
    assert!(model.contains("self.registry.register"));
    assert!(model.contains(".fold(Report::new()"));
    assert!(model.contains("impl CLayout for Type"));
    assert!(model.contains("impl CLayout for Primitive"));
    assert!(model.contains("fn with_part"));
    assert!(model.contains("fn label"));
    assert!(model.contains("fn as_usize"));
    assert!(!model.contains("dead_model_function"));
    assert!(!model.contains("dead_report_method"));

    let primitive = read(output.join("primitive_dep/src/lib.rs"));
    assert!(primitive.contains("impl FromStr for Primitive"));
    assert!(primitive.contains("fn weight"));
    assert!(!primitive.contains("dead_primitive_function"));
    assert!(!primitive.contains("dead_weight"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated generic inference slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nmodel_dep/src/lib.rs:\n{}\nprimitive_dep/src/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        model,
        primitive,
    );
}

#[test]
fn rewrites_features_and_keeps_target_dependencies_and_build_script() {
    let workspace = temp_path("manifest-workspace");
    let output = temp_path("manifest-output");
    let target_dir = temp_path("manifest-target");
    write_manifest_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let app_manifest = read(output.join("app/Cargo.toml"));
    assert!(app_manifest.contains("build = \"build.rs\""));
    assert!(app_manifest.contains("cfg(unix)"));
    assert!(app_manifest.contains("platform_dep"));
    assert!(app_manifest.contains("live_dep"));
    assert!(!app_manifest.contains("dead_dep"));
    assert!(!app_manifest.contains("dep:dead_dep"));
    assert!(!app_manifest.contains("dead_dep?/extra"));
    assert!(!app_manifest.contains("dev_only/std"));
    assert!(output.join("app/build.rs").exists());
    assert!(output.join("app/assets/required.txt").exists());
    assert!(!output.join("app/docs/noise.md").exists());

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated manifest slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\napp/Cargo.toml:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        app_manifest,
    );
}

#[test]
fn preserves_ffi_crate_types_target_patch_dependencies_and_workspace_context() {
    let workspace = temp_path("ffi-manifest-bundle-workspace");
    let output = temp_path("ffi-manifest-bundle-output");
    let target_dir = temp_path("ffi-manifest-bundle-target");
    write_ffi_manifest_bundle_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace.clone(),
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let root_manifest = read(output.join("Cargo.toml"));
    let app_manifest = read(output.join("app/Cargo.toml"));
    let platform_source = read(output.join("support/platform-dep/src/lib.rs"));
    assert!(app_manifest.contains("crate-type"), "{app_manifest}");
    assert!(app_manifest.contains("cdylib"), "{app_manifest}");
    assert!(app_manifest.contains("staticlib"), "{app_manifest}");
    assert!(app_manifest.contains("cfg(unix)"), "{app_manifest}");
    assert!(app_manifest.contains("platform_dep"), "{app_manifest}");
    assert!(app_manifest.contains("uniffi"), "{app_manifest}");
    assert!(app_manifest.contains("dep:uniffi"), "{app_manifest}");
    assert!(!app_manifest.contains("dead_dep"), "{app_manifest}");
    assert!(root_manifest.contains("[patch.crates-io.platform-dep]"));
    assert!(root_manifest.contains("path = \"support/platform-dep\""));
    assert!(
        !root_manifest.contains("unused-patch"),
        "unused workspace patch entries must not be emitted\n{root_manifest}"
    );
    assert!(output.join("Cargo.lock").exists());
    assert!(output.join("rust-toolchain.toml").exists());
    assert!(output.join("uniffi/Cargo.toml").exists());
    assert!(!output.join("support/unused-patch").exists());
    assert!(
        platform_source.contains("pub fn platform"),
        "{platform_source}"
    );
    assert!(
        !platform_source.contains("unused_platform"),
        "{platform_source}"
    );

    fs::rename(
        workspace.join("platform-dep"),
        workspace.join("platform-dep.moved"),
    )
    .unwrap();
    fs::rename(
        workspace.join("unused-patch"),
        workspace.join("unused-patch.moved"),
    )
    .unwrap();
    fs::rename(workspace.join("uniffi"), workspace.join("uniffi.moved")).unwrap();

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated ffi manifest bundle slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nCargo.toml:\n{}\napp/Cargo.toml:\n{}\nplatform-dep/src/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        root_manifest,
        app_manifest,
        platform_source,
    );
}

#[test]
fn retains_local_build_dependencies_for_retained_build_scripts() {
    let workspace = temp_path("build-dependency-workspace");
    let output = temp_path("build-dependency-output");
    let target_dir = temp_path("build-dependency-target");
    write_build_dependency_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    assert_eq!(report.packages, ["app", "build_helper"]);
    let app_manifest = read(output.join("app/Cargo.toml"));
    assert!(
        app_manifest.contains("build_helper"),
        "app manifest should retain local build-dependency\n{app_manifest}"
    );
    assert!(output.join("build_helper/src/lib.rs").exists());

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated build-dependency slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\napp/Cargo.toml:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        app_manifest,
    );
}

#[test]
fn prunes_local_build_dependencies_without_retained_build_scripts() {
    let workspace = temp_path("unused-build-dependency-workspace");
    let output = temp_path("unused-build-dependency-output");
    let target_dir = temp_path("unused-build-dependency-target");
    write_unused_build_dependency_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    assert_eq!(report.packages, ["app"]);
    let app_manifest = read(output.join("app/Cargo.toml"));
    assert!(
        !app_manifest.contains("build-dependencies"),
        "app manifest should prune local build-dependencies when no build script is retained\n{app_manifest}"
    );
    assert!(
        !output.join("build_helper").exists(),
        "unused local build helper package should not be copied"
    );

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated unused-build-dependency slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\napp/Cargo.toml:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        app_manifest,
    );
}

#[test]
fn prunes_unused_local_dependency_edges_to_retained_packages() {
    let workspace = temp_path("unused-local-edge-workspace");
    let output = temp_path("unused-local-edge-output");
    let target_dir = temp_path("unused-local-edge-target");
    write_unused_local_dependency_edge_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    assert_eq!(report.packages, ["app", "hub", "shared"]);
    let app_manifest = read(output.join("app/Cargo.toml"));
    let hub_manifest = read(output.join("hub/Cargo.toml"));
    assert!(
        app_manifest.contains("hub"),
        "app manifest should retain the dependency used by selected code\n{app_manifest}"
    );
    assert!(
        !app_manifest.contains("shared"),
        "app manifest should prune unused direct local dependency even when the package is retained elsewhere\n{app_manifest}"
    );
    assert!(
        hub_manifest.contains("shared"),
        "hub manifest should keep its live local dependency\n{hub_manifest}"
    );

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated unused-local-edge slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\napp/Cargo.toml:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        app_manifest,
    );
}

#[test]
fn prunes_local_dependency_alias_when_only_local_ident_matches() {
    let workspace = temp_path("dependency-alias-ident-workspace");
    let output = temp_path("dependency-alias-ident-output");
    let target_dir = temp_path("dependency-alias-ident-target");
    write_dependency_alias_ident_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    assert_eq!(report.packages, ["app"]);
    let app_manifest = read(output.join("app/Cargo.toml"));
    let lib = read(output.join("app/src/lib.rs"));
    assert!(
        !app_manifest.contains("payload"),
        "app manifest should not retain dependency aliases mentioned only as local identifiers\n{app_manifest}"
    );
    assert!(
        !output.join("payload").exists(),
        "local dependency named like a retained local variable should not be copied"
    );
    assert!(lib.contains("let payload = 7"), "{lib}");

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated dependency-alias-ident slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\napp/Cargo.toml:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        app_manifest,
    );
}

#[test]
fn retains_target_build_dependencies_for_retained_build_scripts() {
    let workspace = temp_path("target-build-dependency-workspace");
    let output = temp_path("target-build-dependency-output");
    let target_dir = temp_path("target-build-dependency-target");
    write_target_build_dependency_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    assert_eq!(report.packages, ["app", "build_helper"]);
    let app_manifest = read(output.join("app/Cargo.toml"));
    assert!(
        app_manifest.contains("cfg(all())"),
        "app manifest should preserve target cfg\n{app_manifest}"
    );
    assert!(
        app_manifest.contains("build-dependencies"),
        "app manifest should retain target build-dependencies\n{app_manifest}"
    );
    assert!(
        app_manifest.contains("build_helper"),
        "app manifest should retain target build helper\n{app_manifest}"
    );

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated target build-dependency slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\napp/Cargo.toml:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        app_manifest,
    );
}

#[test]
fn keeps_optional_dependency_requested_by_retained_local_package_feature() {
    let workspace = temp_path("implicit-feature-workspace");
    let output = temp_path("implicit-feature-output");
    let target_dir = temp_path("implicit-feature-target");
    write_implicit_feature_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let app_manifest = read(output.join("app/Cargo.toml"));
    let helper_manifest = read(output.join("helper/Cargo.toml"));
    assert!(app_manifest.contains("features = [\"feature_dep\"]"));
    assert!(
        helper_manifest.contains("feature_dep"),
        "helper manifest should retain optional dependency requested through app feature\n{helper_manifest}"
    );
    assert!(
        helper_manifest.contains("optional = true"),
        "helper manifest should preserve optional dependency metadata\n{helper_manifest}"
    );

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated implicit feature slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\napp/Cargo.toml:\n{}\nhelper/Cargo.toml:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        app_manifest,
        helper_manifest,
    );
}

#[test]
fn retains_mutex_guard_field_method_and_prunes_unused_associated_const_imports() {
    let workspace = temp_path("mutex-field-workspace");
    let output = temp_path("mutex-field-output");
    let target_dir = temp_path("mutex-field-target");
    write_mutex_field_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let source = read(output.join("app/src/lib.rs"));
    assert!(source.contains("fn push"));
    assert!(source.contains("Weak"));
    assert!(source.contains("SqlInterruptHandle"));
    assert!(source.contains("RateLimiter::new"));
    assert!(!source.contains("Duration"));
    assert!(!source.contains("Instant"));
    assert!(!source.contains("const INTERVAL"));
    assert!(!source.contains("last_report"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated mutex field slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        source,
    );
}

#[test]
fn retains_marker_trait_impls_external_trait_imports_and_method_arg_impls() {
    let workspace = temp_path("external-trait-workspace");
    let output = temp_path("external-trait-output");
    let target_dir = temp_path("external-trait-target");
    write_external_trait_method_arg_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let lib = read(output.join("app/src/lib.rs"));
    assert!(lib.contains("Engine"));
    assert!(lib.contains("impl Eq for Guid"));
    assert!(lib.contains("unsafe impl Sync for Guid"));
    assert!(lib.contains("impl Visitor"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated external trait slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        lib,
    );
}

#[test]
fn retains_associated_conversion_impls_for_external_targets() {
    let workspace = temp_path("associated-conversion-workspace");
    let output = temp_path("associated-conversion-output");
    let target_dir = temp_path("associated-conversion-target");
    write_associated_conversion_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let lib = read(output.join("app/src/lib.rs"));
    assert!(lib.contains("impl From<Timestamp> for SystemTime"));
    assert!(lib.contains("impl From<SystemTime> for Timestamp"));
    if cfg!(unix) {
        assert!(lib.contains("ExitStatusExt"));
    }

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated associated conversion slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        lib,
    );
}

#[test]
fn retains_extension_traits_macro_reexports_try_conversions_and_impl_trait_bounds() {
    let workspace = temp_path("feedback-layer-workspace");
    let output = temp_path("feedback-layer-output");
    let target_dir = temp_path("feedback-layer-target");
    write_feedback_layer_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let lib = read(output.join("app/src/lib.rs"));
    let db = read(output.join("app/src/db.rs"));
    assert!(lib.contains("error_support"));
    assert!(db.contains("trait ConnExt"));
    assert!(db.contains("impl ConnExt for SystemTime"));
    assert!(db.contains("fmt::Display for RepeatDisplay"));
    assert!(db.contains("impl From<std::io::Error> for LocalError"));
    assert!(db.contains("impl std::error::Error for Interrupted"));
    assert!(db.contains("fn is_valid"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated feedback layer slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/lib.rs:\n{}\nsrc/db.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        lib,
        db,
    );
}

#[test]
fn retains_struct_literals_glob_reexported_functions_collect_impls_and_attr_macro_helpers() {
    let workspace = temp_path("feedback-places-workspace");
    let output = temp_path("feedback-places-output");
    let target_dir = temp_path("feedback-places-target");
    write_feedback_places_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let app_db = read(output.join("app/src/db.rs"));
    let helper_lib = read(output.join("helper/src/lib.rs"));
    let helper_chunks = read(output.join("helper/src/chunks.rs"));
    let error_support = read(output.join("error-support/src/lib.rs"));
    assert!(app_db.contains("struct PlacesInitializer"));
    assert!(app_db.contains("impl Initializer for PlacesInitializer"));
    assert!(app_db.contains("struct HistoryRecord"));
    assert!(app_db.contains("impl std::iter::FromIterator<VisitType> for VisitTransitionSet"));
    assert!(helper_lib.contains("pub use chunks::*"));
    assert!(helper_chunks.contains("pub fn each_chunk"));
    assert!(error_support.contains("convert_log_report_error"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated feedback places slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\napp/src/db.rs:\n{}\nhelper/src/lib.rs:\n{}\nhelper/src/chunks.rs:\n{}\nerror-support/src/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        app_db,
        helper_lib,
        helper_chunks,
        error_support,
    );
}

#[test]
fn retains_inline_generated_modules_macro_deref_helpers_and_backend_bridges() {
    let workspace = temp_path("feedback-viaduct-workspace");
    let output = temp_path("feedback-viaduct-output");
    let target_dir = temp_path("feedback-viaduct-target");
    write_feedback_viaduct_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let app = read(output.join("app/src/lib.rs"));
    assert!(app.contains("mod msg_types"));
    assert!(app.contains("include!(\"generated.rs\")"));
    assert!(app.contains("use crate::msg_types"));
    assert!(app.contains("impl std::ops::Deref for HeaderName"));
    assert!(app.contains("fn set_value"));
    assert!(app.contains("impl old_backend::Backend for Arc<dyn Backend>"));
    assert!(!app.contains("ErrorHandling"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated feedback viaduct slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        app,
    );
}

#[test]
fn resolves_arbitrary_untyped_closure_and_wrapper_trait_methods_without_name_allowlist() {
    let workspace = temp_path("generic-method-recovery-workspace");
    let output = temp_path("generic-method-recovery-output");
    let target_dir = temp_path("generic-method-recovery-target");
    write_generic_method_recovery_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let source = read(output.join("app/src/lib.rs"));
    assert!(source.contains("fn persist_widget"));
    assert!(source.contains("fn calculate_marker"));
    assert!(source.contains("impl MarkerSum for Vec<Widget>"));
    assert!(!source.contains("discarded_noise"));
    assert!(!source.contains("#[opensourced]"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated generic method recovery slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        source,
    );
}

#[test]
fn resolves_generic_trait_bound_receiver_methods_without_name_only_fallback() {
    let workspace = temp_path("generic-bound-method-workspace");
    let output = temp_path("generic-bound-method-output");
    let target_dir = temp_path("generic-bound-method-target");
    write_basic_workspace(&workspace);
    write_basic_app_manifest(&workspace);
    write(
        workspace.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

pub trait Worker {
    fn work(&self) -> usize;
}

pub trait Factory {
    fn make(&self) -> Product;
}

pub struct Product;

impl Product {
    fn finish(&self) -> usize {
        3
    }

    fn unused_finish(&self) -> usize {
        30
    }
}

pub struct Concrete;

impl Worker for Concrete {
    fn work(&self) -> usize {
        concrete_work()
    }
}

fn concrete_work() -> usize {
    11
}

pub struct Decoy;

impl Decoy {
    fn work(&self) -> usize {
        dead_work()
    }

    fn make(&self) -> Product {
        Product
    }
}

fn dead_work() -> usize {
    99
}

#[opensourced]
pub fn selected<T>(worker: T, other: impl Worker, factory: impl Factory) -> usize
where
    T: Worker,
{
    let rebound: T = worker;
    rebound.work() + other.work() + factory.make().finish()
}
"#,
    );

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    assert!(
        !report
            .production
            .hazards
            .iter()
            .any(|hazard| hazard.code == "syntactic_method_fallbacks"),
        "generic trait-bound receiver calls should not use name-only fallback: {:?}",
        report.production.hazards
    );

    let source = read(output.join("app/src/lib.rs"));
    assert!(source.contains("pub trait Worker"));
    assert!(source.contains("pub trait Factory"));
    assert!(source.contains("fn finish"));
    assert!(!source.contains("struct Decoy"));
    assert!(!source.contains("dead_work"));
    assert!(!source.contains("unused_finish"));
    assert!(!source.contains("#[opensourced]"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated generic-bound method slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        source,
    );
}

#[test]
fn resolves_external_workspace_path_dependencies_from_original_root() {
    let workspace = temp_path("external-workspace-dep-workspace");
    let output = temp_path("external-workspace-dep-output");
    let target_dir = temp_path("external-workspace-dep-target");
    write_external_workspace_path_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace.clone(),
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let root_manifest = read(output.join("Cargo.toml"));
    let app_manifest = read(output.join("app/Cargo.toml"));
    let lockfile = read(output.join("Cargo.lock"));
    let support_manifest = read(output.join("support/external-helper/Cargo.toml"));
    let support_leaf_manifest = read(output.join("support/external-leaf/Cargo.toml"));
    let support_source = read(output.join("support/external-helper/src/lib.rs"));
    assert!(root_manifest.contains("external_helper"));
    assert!(root_manifest.contains("path = \"support/external-helper\""));
    assert!(root_manifest.contains("path = \"support/external-leaf\""));
    assert!(
        !root_manifest.contains("external-unused"),
        "unused patch packages must not be emitted into the generated root manifest\n{root_manifest}"
    );
    assert!(!root_manifest.contains("path = \"/"));
    assert!(!root_manifest.contains("path = \"../"));
    assert!(root_manifest.contains("[patch.crates-io.external-helper]"));
    assert!(root_manifest.contains("[patch.crates-io.external-leaf]"));
    assert!(app_manifest.contains("external_helper"));
    assert!(support_manifest.contains("name = \"external-helper\""));
    assert!(support_leaf_manifest.contains("name = \"external-leaf\""));
    assert!(
        !output.join("support/external-unused").exists(),
        "unused patch packages must not be copied as support dependencies"
    );
    assert!(support_source.contains("pub fn decorate"));
    assert!(lockfile.contains("version = 3"));

    let external_name = format!(
        "{}-external-helper",
        workspace.file_name().unwrap().to_string_lossy()
    );
    let external_leaf_name = format!(
        "{}-external-leaf",
        workspace.file_name().unwrap().to_string_lossy()
    );
    let external_unused_name = format!(
        "{}-external-unused",
        workspace.file_name().unwrap().to_string_lossy()
    );
    let external_root = workspace.parent().unwrap().join(external_name);
    let external_leaf_root = workspace.parent().unwrap().join(external_leaf_name);
    let external_unused_root = workspace.parent().unwrap().join(external_unused_name);
    fs::rename(&external_root, external_root.with_extension("moved")).unwrap();
    fs::rename(
        &external_leaf_root,
        external_leaf_root.with_extension("moved"),
    )
    .unwrap();
    fs::rename(
        &external_unused_root,
        external_unused_root.with_extension("moved"),
    )
    .unwrap();

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated external workspace dependency slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nCargo.toml:\n{}\napp/Cargo.toml:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        root_manifest,
        app_manifest,
    );
}

#[test]
fn copies_workspace_assets_referenced_by_copied_support_include_macros() {
    let workspace = temp_path("support-include-asset-workspace");
    let output = temp_path("support-include-asset-output");
    let target_dir = temp_path("support-include-asset-target");
    let support_ws_name = format!(
        "{}-support-ws",
        workspace.file_name().unwrap().to_string_lossy()
    );
    let support_ws = workspace.parent().unwrap().join(&support_ws_name);
    if workspace.exists() {
        fs::remove_dir_all(&workspace).unwrap();
    }
    if support_ws.exists() {
        fs::remove_dir_all(&support_ws).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");

    write(
        workspace.join("Cargo.toml"),
        &format!(
            r#"[workspace]
members = ["app"]
resolver = "2"

[workspace.dependencies]
helper = {{ path = "../{support_ws_name}/helper" }}
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        workspace.join("app/Cargo.toml"),
        r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
helper.workspace = true
opensourced.workspace = true
"#,
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected(value: &str) -> String {
    helper::decorate(value)
}
"#,
    );
    write(
        support_ws.join("Cargo.toml"),
        r#"[workspace]
members = ["helper"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        support_ws.join("helper/Cargo.toml"),
        r#"[package]
name = "helper"
version.workspace = true
edition.workspace = true
"#,
    );
    write(
        support_ws.join("helper/src/lib.rs"),
        r#"const SHARED: &str = include_str!("../../shared.txt");

pub fn decorate(value: &str) -> String {
    format!("{value}:{SHARED}")
}
"#,
    );
    write(support_ws.join("shared.txt"), "support-shared");

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    assert!(output.join("support/helper/src/lib.rs").exists());
    assert!(output.join("support/shared.txt").exists());

    fs::rename(&support_ws, support_ws.with_extension("moved")).unwrap();

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated support include asset slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nCargo.toml:\n{}\nhelper lib:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        read(output.join("Cargo.toml")),
        read(output.join("support/helper/src/lib.rs")),
    );
}

#[test]
fn retains_imports_used_only_by_format_string_captures() {
    let workspace = temp_path("format-capture-import-workspace");
    let output = temp_path("format-capture-import-output");
    let target_dir = temp_path("format-capture-import-target");
    let opensourced_path = repo_root().join("crates/opensourced");

    write(
        workspace.join("Cargo.toml"),
        r#"[workspace]
members = ["app"]
resolver = "2"
"#,
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"mod constants {
    pub const PROFILE_INIT: &str = "init";
}

mod launcher {
    use opensourced::opensourced;
    use super::constants::PROFILE_INIT;

    #[opensourced]
    pub fn selected() -> String {
        format!("{PROFILE_INIT} run")
    }
}
"#,
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let generated = read(output.join("app/src/lib.rs"));
    assert!(generated.contains("PROFILE_INIT"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated format-capture import slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nlib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        generated,
    );
}

#[test]
fn retains_parent_trait_imports_used_by_child_super_glob() {
    let workspace = temp_path("super-glob-trait-import-workspace");
    let output = temp_path("super-glob-trait-import-output");
    let target_dir = temp_path("super-glob-trait-import-target");
    let opensourced_path = repo_root().join("crates/opensourced");

    write(
        workspace.join("Cargo.toml"),
        r#"[workspace]
members = ["app"]
resolver = "2"
"#,
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

mod child {
    use opensourced::opensourced;
    use super::*;

    #[opensourced]
    pub fn selected(value: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }
}
"#,
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let generated = read(output.join("app/src/lib.rs"));
    assert!(generated.contains("Hash"));
    assert!(generated.contains("Hasher"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated super-glob trait import slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nlib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        generated,
    );
}

#[test]
fn retains_from_str_trait_import_for_associated_function_calls() {
    let workspace = temp_path("from-str-import-workspace");
    let output = temp_path("from-str-import-output");
    let target_dir = temp_path("from-str-import-target");
    let opensourced_path = repo_root().join("crates/opensourced");

    write(
        workspace.join("Cargo.toml"),
        r#"[workspace]
members = ["app"]
resolver = "2"
"#,
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;
use std::str::FromStr;

#[opensourced]
pub fn selected(value: &str) -> Result<u16, std::num::ParseIntError> {
    u16::from_str(value)
}
"#,
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let generated = read(output.join("app/src/lib.rs"));
    assert!(generated.contains("FromStr"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated FromStr import slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nlib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        generated,
    );
}

#[test]
fn copies_direct_external_path_dependency_closure_into_generated_output() {
    let workspace = temp_path("direct-external-path-dep-workspace");
    let output = temp_path("direct-external-path-dep-output");
    let target_dir = temp_path("direct-external-path-dep-target");
    let helper = temp_path("direct-external-path-dep-helper");
    let leaf = temp_path("direct-external-path-dep-leaf");
    let opensourced_path = repo_root().join("crates/opensourced");

    write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
external-helper = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path),
            manifest_path(&helper)
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected(value: &str) -> String {
    external_helper::decorate(value)
}
"#,
    );
    write(
        helper.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "external-helper"
version = "0.1.0"
edition = "2021"

[dependencies]
external-leaf = {{ path = "{}" }}
"#,
            manifest_path(&leaf)
        ),
    );
    write(
        helper.join("src/lib.rs"),
        r#"pub fn decorate(value: &str) -> String {
    format!("{value}{}", external_leaf::suffix())
}
"#,
    );
    write(
        leaf.join("Cargo.toml"),
        r#"[package]
name = "external-leaf"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        leaf.join("src/lib.rs"),
        r#"pub fn suffix() -> &'static str {
    ":leaf"
}
"#,
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let app_manifest = read(output.join("app/Cargo.toml"));
    let helper_manifest = read(output.join("support/external-helper/Cargo.toml"));
    let leaf_manifest = read(output.join("support/external-leaf/Cargo.toml"));
    assert!(app_manifest.contains("path = \"../support/external-helper\""));
    assert!(helper_manifest.contains("path = \"../external-leaf\""));
    assert!(leaf_manifest.contains("name = \"external-leaf\""));
    assert!(!app_manifest.contains(&manifest_path(&helper)));
    assert!(!helper_manifest.contains(&manifest_path(&leaf)));

    fs::rename(&helper, helper.with_extension("moved")).unwrap();
    fs::rename(&leaf, leaf.with_extension("moved")).unwrap();

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated direct external dependency slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\napp/Cargo.toml:\n{}\nhelper/Cargo.toml:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        app_manifest,
        helper_manifest,
    );
}

#[test]
fn prunes_unused_support_package_dev_build_and_tool_surfaces() {
    let workspace = temp_path("minimal-external-path-dep-workspace");
    let output = temp_path("minimal-external-path-dep-output");
    let target_dir = temp_path("minimal-external-path-dep-target");
    let helper = temp_path("minimal-external-path-dep-helper");
    let leaf = temp_path("minimal-external-path-dep-leaf");
    let unused_build = temp_path("minimal-external-path-dep-unused-build");
    let unused_dev = temp_path("minimal-external-path-dep-unused-dev");
    let unused_target_build = temp_path("minimal-external-path-dep-unused-target-build");
    let unused_target_dev = temp_path("minimal-external-path-dep-unused-target-dev");
    let opensourced_path = repo_root().join("crates/opensourced");

    write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
external-helper = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path),
            manifest_path(&helper)
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected(value: &str) -> String {
    external_helper::decorate(value)
}
"#,
    );
    write(
        helper.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "external-helper"
version = "0.1.0"
edition = "2021"

[dependencies]
external-leaf = {{ path = "{}" }}

[build-dependencies]
unused-build = {{ path = "{}" }}

[dev-dependencies]
unused-dev = {{ path = "{}" }}

[target.'cfg(unix)'.build-dependencies]
unused-target-build = {{ path = "{}" }}

[target.'cfg(unix)'.dev-dependencies]
unused-target-dev = {{ path = "{}" }}

[[example]]
name = "unused_example"
path = "examples/unused.rs"

[[test]]
name = "unused_test"
path = "tests/unused.rs"

[[bench]]
name = "unused_bench"
path = "benches/unused.rs"
"#,
            manifest_path(&leaf),
            manifest_path(&unused_build),
            manifest_path(&unused_dev),
            manifest_path(&unused_target_build),
            manifest_path(&unused_target_dev),
        ),
    );
    write(
        helper.join("src/lib.rs"),
        r#"pub fn decorate(value: &str) -> String {
    format!("{value}{}", external_leaf::suffix())
}
"#,
    );
    write(helper.join("examples/unused.rs"), "fn main() {}\n");
    write(helper.join("src/bin/unused.rs"), "fn main() {}\n");
    write(helper.join("tests/unused.rs"), "#[test]\nfn unused() {}\n");
    write(helper.join("benches/unused.rs"), "fn main() {}\n");
    write(helper.join("fixtures/dead.txt"), "dead support fixture");
    write(
        leaf.join("Cargo.toml"),
        r#"[package]
name = "external-leaf"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        leaf.join("src/lib.rs"),
        r#"pub fn suffix() -> &'static str {
    ":leaf"
}
"#,
    );
    for package in [
        (&unused_build, "unused-build"),
        (&unused_dev, "unused-dev"),
        (&unused_target_build, "unused-target-build"),
        (&unused_target_dev, "unused-target-dev"),
    ] {
        write(
            package.0.join("Cargo.toml"),
            &format!(
                r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"
"#,
                package.1
            ),
        );
        write(package.0.join("src/lib.rs"), "pub fn unused() {}\n");
    }

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let helper_manifest = read(output.join("support/external-helper/Cargo.toml"));
    assert!(helper_manifest.contains("external-leaf"));
    assert!(!helper_manifest.contains("build-dependencies"));
    assert!(!helper_manifest.contains("dev-dependencies"));
    assert!(!helper_manifest.contains("unused-build"));
    assert!(!helper_manifest.contains("unused-dev"));
    assert!(!helper_manifest.contains("unused-target-build"));
    assert!(!helper_manifest.contains("unused-target-dev"));
    assert!(!helper_manifest.contains("[[example]]"));
    assert!(!helper_manifest.contains("[[test]]"));
    assert!(!helper_manifest.contains("[[bench]]"));
    assert!(output.join("support/external-helper/src/lib.rs").exists());
    assert!(output.join("support/external-leaf/src/lib.rs").exists());
    assert!(!output.join("support/external-helper/src/bin").exists());
    assert!(!output.join("support/external-helper/examples").exists());
    assert!(!output.join("support/external-helper/tests").exists());
    assert!(!output.join("support/external-helper/benches").exists());
    assert!(!output.join("support/external-helper/fixtures").exists());
    assert!(!output.join("support/unused-build").exists());
    assert!(!output.join("support/unused-dev").exists());
    assert!(!output.join("support/unused-target-build").exists());
    assert!(!output.join("support/unused-target-dev").exists());

    for original in [
        &helper,
        &leaf,
        &unused_build,
        &unused_dev,
        &unused_target_build,
        &unused_target_dev,
    ] {
        fs::rename(original, original.with_extension("moved")).unwrap();
    }

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated minimal external dependency slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nhelper/Cargo.toml:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        helper_manifest,
    );
}

#[test]
fn prunes_support_package_items_referenced_through_crate_alias() {
    let workspace = temp_path("support-crate-alias-workspace");
    let output = temp_path("support-crate-alias-output");
    let target_dir = temp_path("support-crate-alias-target");
    let helper = temp_path("support-crate-alias-helper");
    let opensourced_path = repo_root().join("crates/opensourced");

    write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
external-helper = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path),
            manifest_path(&helper)
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use external_helper as upstream;
use opensourced::opensourced;

#[opensourced]
pub fn selected(value: &str) -> String {
    upstream::decorate(upstream::LivePayload::new(value))
}
"#,
    );
    write(
        helper.join("Cargo.toml"),
        r#"[package]
name = "external-helper"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        helper.join("src/lib.rs"),
        r#"pub mod dead;
pub mod live;

pub use dead::{DeadPayload, dead_export};
pub use live::{LivePayload, decorate};
"#,
    );
    write(
        helper.join("src/live.rs"),
        r#"pub struct LivePayload {
    value: String,
}

impl LivePayload {
    pub fn new(value: &str) -> Self {
        Self {
            value: value.to_string(),
        }
    }
}

pub fn decorate(payload: LivePayload) -> String {
    format!("live:{}", payload.value)
}
"#,
    );
    write(
        helper.join("src/dead.rs"),
        r#"pub struct DeadPayload {
    value: String,
}

pub fn dead_export(payload: DeadPayload) -> String {
    payload.value
}
"#,
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let app = read(output.join("app/src/lib.rs"));
    let helper_lib = read(output.join("support/external-helper/src/lib.rs"));
    let helper_live = read(output.join("support/external-helper/src/live.rs"));
    assert!(app.contains("use external_helper as upstream"), "{app}");
    assert!(helper_lib.contains("pub mod live"), "{helper_lib}");
    assert!(
        helper_lib.contains("pub use live::{LivePayload, decorate}"),
        "{helper_lib}"
    );
    assert!(!helper_lib.contains("dead"), "{helper_lib}");
    assert!(
        helper_live.contains("pub struct LivePayload"),
        "{helper_live}"
    );
    assert!(helper_live.contains("pub fn decorate"), "{helper_live}");
    assert!(!output.join("support/external-helper/src/dead.rs").exists());

    fs::rename(&helper, helper.with_extension("moved"))
        .expect("original helper package should move away");
    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated aliased support dependency slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\napp/src/lib.rs:\n{}\nhelper/src/lib.rs:\n{}\nhelper/src/live.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        app,
        helper_lib,
        helper_live,
    );
}

#[test]
fn prunes_support_public_globs_that_do_not_export_required_names() {
    let workspace = temp_path("support-public-glob-workspace");
    let output = temp_path("support-public-glob-output");
    let target_dir = temp_path("support-public-glob-target");
    let helper = temp_path("support-public-glob-helper");
    let opensourced_path = repo_root().join("crates/opensourced");

    write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
external-helper = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path),
            manifest_path(&helper)
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use external_helper::{LiveOp, LivePatch, LiveSegment};
use opensourced::opensourced;

#[opensourced]
pub fn selected(patch: LivePatch) -> usize {
    match patch.op {
        LiveOp::Add => patch.path.len(),
        LiveOp::Remove => 0,
    }
}

pub fn build_patch(path: Vec<LiveSegment>) -> LivePatch {
    LivePatch {
        op: LiveOp::Add,
        path,
    }
}
"#,
    );
    write(
        helper.join("Cargo.toml"),
        r#"[package]
name = "external-helper"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        helper.join("src/lib.rs"),
        r#"pub mod client;
pub mod error;
pub mod protocol;

pub use client::pending::*;
pub use error::*;
pub use protocol::envelope::*;
pub use protocol::params::*;
"#,
    );
    write(helper.join("src/client/mod.rs"), "pub mod pending;\n");
    write(
        helper.join("src/client/pending.rs"),
        r#"pub struct PendingRequests;

pub fn dead_pending() -> PendingRequests {
    PendingRequests
}
"#,
    );
    write(
        helper.join("src/error.rs"),
        r#"pub enum HelperError {
    Dead,
}
"#,
    );
    write(
        helper.join("src/protocol/mod.rs"),
        r#"pub mod envelope;
pub mod params;
"#,
    );
    write(
        helper.join("src/protocol/envelope.rs"),
        r#"pub struct DeadEnvelope {
    pub id: String,
}
"#,
    );
    write(
        helper.join("src/protocol/params.rs"),
        r#"pub struct LivePatch {
    pub op: LiveOp,
    pub path: Vec<LiveSegment>,
}

pub enum LiveOp {
    Add,
    Remove,
}

pub enum LiveSegment {
    Index(usize),
    Key(String),
}
"#,
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let helper_lib = read(output.join("support/external-helper/src/lib.rs"));
    let protocol_mod = read(output.join("support/external-helper/src/protocol/mod.rs"));
    let params = read(output.join("support/external-helper/src/protocol/params.rs"));
    assert!(helper_lib.contains("pub mod protocol"), "{helper_lib}");
    assert!(
        helper_lib.contains("pub use protocol::params::*"),
        "{helper_lib}"
    );
    assert!(!helper_lib.contains("client"), "{helper_lib}");
    assert!(!helper_lib.contains("error"), "{helper_lib}");
    assert!(!helper_lib.contains("envelope"), "{helper_lib}");
    assert!(protocol_mod.contains("pub mod params"), "{protocol_mod}");
    assert!(!protocol_mod.contains("envelope"), "{protocol_mod}");
    assert!(params.contains("pub struct LivePatch"), "{params}");
    assert!(!output.join("support/external-helper/src/client").exists());
    assert!(!output.join("support/external-helper/src/error.rs").exists());
    assert!(!output
        .join("support/external-helper/src/protocol/envelope.rs")
        .exists());

    fs::rename(&helper, helper.with_extension("moved"))
        .expect("original helper package should move away");
    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated support public glob slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nhelper/src/lib.rs:\n{}\nhelper/src/protocol/mod.rs:\n{}\nhelper/src/protocol/params.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        helper_lib,
        protocol_mod,
        params,
    );
}

#[test]
fn prunes_support_packages_reached_through_reexported_module_aliases() {
    let workspace = temp_path("support-module-alias-workspace");
    let output = temp_path("support-module-alias-output");
    let target_dir = temp_path("support-module-alias-target");
    let helper = temp_path("support-module-alias-helper");
    let opensourced_path = repo_root().join("crates/opensourced");

    write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
external-helper = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path),
            manifest_path(&helper)
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use external_helper::protocol_params::LivePatch;
use opensourced::opensourced;

#[opensourced]
pub fn selected(patch: LivePatch) -> usize {
    patch.path.len()
}
"#,
    );
    write(
        helper.join("Cargo.toml"),
        r#"[package]
name = "external-helper"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        helper.join("src/lib.rs"),
        r#"pub mod protocol;

pub use protocol::dead as dead_protocol;
pub use protocol::params as protocol_params;
"#,
    );
    write(
        helper.join("src/protocol/mod.rs"),
        r#"pub mod dead;
pub mod params;
"#,
    );
    write(
        helper.join("src/protocol/params.rs"),
        r#"pub struct LivePatch {
    pub path: Vec<LiveSegment>,
}

pub enum LiveSegment {
    Index(usize),
    Key(String),
}
"#,
    );
    write(
        helper.join("src/protocol/dead.rs"),
        r#"pub struct DeadPatch {
    pub value: String,
}

pub fn dead_helper() -> DeadPatch {
    DeadPatch {
        value: String::new(),
    }
}
"#,
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reexported support module alias should reduce");

    let helper_lib = read(output.join("support/external-helper/src/lib.rs"));
    let protocol_mod = read(output.join("support/external-helper/src/protocol/mod.rs"));
    let params = read(output.join("support/external-helper/src/protocol/params.rs"));
    assert!(
        helper_lib.contains("pub use protocol::params as protocol_params"),
        "{helper_lib}"
    );
    assert!(!helper_lib.contains("dead_protocol"), "{helper_lib}");
    assert!(protocol_mod.contains("pub mod params"), "{protocol_mod}");
    assert!(!protocol_mod.contains("dead"), "{protocol_mod}");
    assert!(params.contains("pub struct LivePatch"), "{params}");
    assert!(!output
        .join("support/external-helper/src/protocol/dead.rs")
        .exists());

    fs::rename(&helper, helper.with_extension("moved"))
        .expect("original helper package should move away");
    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated support module alias slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nhelper/src/lib.rs:\n{}\nhelper/src/protocol/mod.rs:\n{}\nhelper/src/protocol/params.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        helper_lib,
        protocol_mod,
        params,
    );
}

#[test]
fn prunes_support_packages_reached_through_grouped_self_module_aliases() {
    let workspace = temp_path("support-self-module-alias-workspace");
    let output = temp_path("support-self-module-alias-output");
    let target_dir = temp_path("support-self-module-alias-target");
    let helper = temp_path("support-self-module-alias-helper");
    let opensourced_path = repo_root().join("crates/opensourced");

    write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
external-helper = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path),
            manifest_path(&helper)
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use external_helper::protocol_params::LivePatch;
use opensourced::opensourced;

#[opensourced]
pub fn selected(patch: LivePatch) -> String {
    patch.path.join("/")
}
"#,
    );
    write(
        helper.join("Cargo.toml"),
        r#"[package]
name = "external-helper"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        helper.join("src/lib.rs"),
        r#"pub mod protocol;

pub use protocol::dead::{self as dead_protocol};
pub use protocol::params::{self as protocol_params};
"#,
    );
    write(
        helper.join("src/protocol/mod.rs"),
        r#"pub mod dead;
pub mod params;
"#,
    );
    write(
        helper.join("src/protocol/params.rs"),
        r#"pub struct LivePatch {
    pub path: Vec<String>,
}
"#,
    );
    write(
        helper.join("src/protocol/dead.rs"),
        r#"pub struct DeadPatch {
    pub path: Vec<String>,
}
"#,
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("grouped self support module alias should reduce");

    let helper_lib = read(output.join("support/external-helper/src/lib.rs"));
    let protocol_mod = read(output.join("support/external-helper/src/protocol/mod.rs"));
    assert!(helper_lib.contains("protocol_params"), "{helper_lib}");
    assert!(!helper_lib.contains("dead_protocol"), "{helper_lib}");
    assert!(protocol_mod.contains("pub mod params"), "{protocol_mod}");
    assert!(!protocol_mod.contains("dead"), "{protocol_mod}");
    assert!(!output
        .join("support/external-helper/src/protocol/dead.rs")
        .exists());

    fs::rename(&helper, helper.with_extension("moved"))
        .expect("original helper package should move away");
    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated support grouped self alias slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nhelper/src/lib.rs:\n{}\nhelper/src/protocol/mod.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        helper_lib,
        protocol_mod,
    );
}

#[test]
fn prunes_support_packages_reached_through_private_self_module_aliases() {
    let workspace = temp_path("support-private-self-alias-workspace");
    let output = temp_path("support-private-self-alias-output");
    let target_dir = temp_path("support-private-self-alias-target");
    let helper = temp_path("support-private-self-alias-helper");
    let opensourced_path = repo_root().join("crates/opensourced");

    write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
external-helper = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path),
            manifest_path(&helper)
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use external_helper::selected_patch;
use opensourced::opensourced;

#[opensourced]
pub fn selected() -> usize {
    selected_patch().path.len()
}
"#,
    );
    write(
        helper.join("Cargo.toml"),
        r#"[package]
name = "external-helper"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        helper.join("src/lib.rs"),
        r#"pub mod protocol;

use protocol::dead::{self as dead_protocol};
use protocol::params::{self as protocol_params};

pub fn selected_patch() -> protocol_params::LivePatch {
    protocol_params::LivePatch {
        path: vec![protocol_params::LiveSegment::Index(0)],
    }
}

pub fn dead_patch() -> dead_protocol::DeadPatch {
    dead_protocol::DeadPatch {
        value: String::new(),
    }
}
"#,
    );
    write(
        helper.join("src/protocol/mod.rs"),
        r#"pub mod dead;
pub mod params;
"#,
    );
    write(
        helper.join("src/protocol/params.rs"),
        r#"pub struct LivePatch {
    pub path: Vec<LiveSegment>,
}

pub enum LiveSegment {
    Index(usize),
    Key(String),
}
"#,
    );
    write(
        helper.join("src/protocol/dead.rs"),
        r#"pub struct DeadPatch {
    pub value: String,
}
"#,
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("private self support module alias should reduce");

    let helper_lib = read(output.join("support/external-helper/src/lib.rs"));
    let protocol_mod = read(output.join("support/external-helper/src/protocol/mod.rs"));
    let params = read(output.join("support/external-helper/src/protocol/params.rs"));
    assert!(helper_lib.contains("protocol_params"), "{helper_lib}");
    assert!(!helper_lib.contains("dead_protocol"), "{helper_lib}");
    assert!(!helper_lib.contains("dead_patch"), "{helper_lib}");
    assert!(protocol_mod.contains("pub mod params"), "{protocol_mod}");
    assert!(!protocol_mod.contains("dead"), "{protocol_mod}");
    assert!(params.contains("pub struct LivePatch"), "{params}");
    assert!(params.contains("pub enum LiveSegment"), "{params}");
    assert!(!output
        .join("support/external-helper/src/protocol/dead.rs")
        .exists());

    fs::rename(&helper, helper.with_extension("moved"))
        .expect("original helper package should move away");
    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated support private self alias slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nhelper/src/lib.rs:\n{}\nhelper/src/protocol/mod.rs:\n{}\nhelper/src/protocol/params.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        helper_lib,
        protocol_mod,
        params,
    );
}

#[test]
fn prunes_support_packages_reached_through_private_direct_module_aliases() {
    let workspace = temp_path("support-private-direct-alias-workspace");
    let output = temp_path("support-private-direct-alias-output");
    let target_dir = temp_path("support-private-direct-alias-target");
    let helper = temp_path("support-private-direct-alias-helper");
    let opensourced_path = repo_root().join("crates/opensourced");

    write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
external-helper = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path),
            manifest_path(&helper)
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use external_helper::selected_patch;
use opensourced::opensourced;

#[opensourced]
pub fn selected() -> String {
    selected_patch().path.join("/")
}
"#,
    );
    write(
        helper.join("Cargo.toml"),
        r#"[package]
name = "external-helper"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        helper.join("src/lib.rs"),
        r#"pub mod protocol;

use protocol::dead as dead_protocol;
use protocol::params as protocol_params;

pub fn selected_patch() -> protocol_params::LivePatch {
    protocol_params::LivePatch {
        path: vec![String::from("live")],
    }
}

pub fn dead_patch() -> dead_protocol::DeadPatch {
    dead_protocol::DeadPatch {
        path: vec![String::from("dead")],
    }
}
"#,
    );
    write(
        helper.join("src/protocol/mod.rs"),
        r#"pub mod dead;
pub mod params;
"#,
    );
    write(
        helper.join("src/protocol/params.rs"),
        r#"pub struct LivePatch {
    pub path: Vec<String>,
}
"#,
    );
    write(
        helper.join("src/protocol/dead.rs"),
        r#"pub struct DeadPatch {
    pub path: Vec<String>,
}
"#,
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("private direct support module alias should reduce");

    let helper_lib = read(output.join("support/external-helper/src/lib.rs"));
    let protocol_mod = read(output.join("support/external-helper/src/protocol/mod.rs"));
    let params = read(output.join("support/external-helper/src/protocol/params.rs"));
    assert!(helper_lib.contains("protocol_params"), "{helper_lib}");
    assert!(!helper_lib.contains("dead_protocol"), "{helper_lib}");
    assert!(!helper_lib.contains("dead_patch"), "{helper_lib}");
    assert!(protocol_mod.contains("pub mod params"), "{protocol_mod}");
    assert!(!protocol_mod.contains("dead"), "{protocol_mod}");
    assert!(params.contains("pub struct LivePatch"), "{params}");
    assert!(!output
        .join("support/external-helper/src/protocol/dead.rs")
        .exists());

    fs::rename(&helper, helper.with_extension("moved"))
        .expect("original helper package should move away");
    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated support private direct alias slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nhelper/src/lib.rs:\n{}\nhelper/src/protocol/mod.rs:\n{}\nhelper/src/protocol/params.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        helper_lib,
        protocol_mod,
        params,
    );
}

#[test]
fn prunes_support_packages_reached_through_inline_facade_modules() {
    let workspace = temp_path("support-inline-facade-workspace");
    let output = temp_path("support-inline-facade-output");
    let target_dir = temp_path("support-inline-facade-target");
    let helper = temp_path("support-inline-facade-helper");
    let opensourced_path = repo_root().join("crates/opensourced");

    write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
external-helper = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path),
            manifest_path(&helper)
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use external_helper::prelude::LivePatch;
use opensourced::opensourced;

#[opensourced]
pub fn selected(patch: LivePatch) -> usize {
    patch.path.len()
}
"#,
    );
    write(
        helper.join("Cargo.toml"),
        r#"[package]
name = "external-helper"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        helper.join("src/lib.rs"),
        r#"pub mod protocol;

pub mod prelude {
    pub use crate::protocol::dead::*;
    pub use crate::protocol::params::*;

    pub fn dead_facade() -> crate::protocol::dead::DeadPatch {
        crate::protocol::dead::DeadPatch {
            value: String::new(),
        }
    }
}
"#,
    );
    write(
        helper.join("src/protocol/mod.rs"),
        r#"pub mod dead;
pub mod params;
"#,
    );
    write(
        helper.join("src/protocol/params.rs"),
        r#"pub struct LivePatch {
    pub path: Vec<LiveSegment>,
}

pub enum LiveSegment {
    Index(usize),
    Key(String),
}
"#,
    );
    write(
        helper.join("src/protocol/dead.rs"),
        r#"pub struct DeadPatch {
    pub value: String,
}
"#,
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("inline support facade module should reduce");

    let helper_lib = read(output.join("support/external-helper/src/lib.rs"));
    let protocol_mod = read(output.join("support/external-helper/src/protocol/mod.rs"));
    let params = read(output.join("support/external-helper/src/protocol/params.rs"));
    assert!(helper_lib.contains("pub mod prelude"), "{helper_lib}");
    assert!(
        helper_lib.contains("pub use crate::protocol::params::*"),
        "{helper_lib}"
    );
    assert!(!helper_lib.contains("dead_facade"), "{helper_lib}");
    assert!(!helper_lib.contains("protocol::dead"), "{helper_lib}");
    assert!(protocol_mod.contains("pub mod params"), "{protocol_mod}");
    assert!(!protocol_mod.contains("dead"), "{protocol_mod}");
    assert!(params.contains("pub struct LivePatch"), "{params}");
    assert!(params.contains("pub enum LiveSegment"), "{params}");
    assert!(!output
        .join("support/external-helper/src/protocol/dead.rs")
        .exists());

    fs::rename(&helper, helper.with_extension("moved"))
        .expect("original helper package should move away");
    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated support inline facade slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nhelper/src/lib.rs:\n{}\nhelper/src/protocol/mod.rs:\n{}\nhelper/src/protocol/params.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        helper_lib,
        protocol_mod,
        params,
    );
}

#[test]
fn prunes_support_packages_reached_through_nested_inline_facade_modules() {
    let workspace = temp_path("support-nested-inline-facade-workspace");
    let output = temp_path("support-nested-inline-facade-output");
    let target_dir = temp_path("support-nested-inline-facade-target");
    let helper = temp_path("support-nested-inline-facade-helper");
    let opensourced_path = repo_root().join("crates/opensourced");

    write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
external-helper = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path),
            manifest_path(&helper)
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use external_helper::facade::nested::LivePatch;
use opensourced::opensourced;

#[opensourced]
pub fn selected(patch: LivePatch) -> usize {
    patch.path.len()
}
"#,
    );
    write(
        helper.join("Cargo.toml"),
        r#"[package]
name = "external-helper"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        helper.join("src/lib.rs"),
        r#"pub mod protocol;

pub mod facade {
    pub mod nested {
        pub use crate::protocol::dead::*;
        pub use crate::protocol::params::*;

        pub fn dead_nested() -> crate::protocol::dead::DeadPatch {
            crate::protocol::dead::DeadPatch {
                value: String::new(),
            }
        }
    }

    pub mod dead_facade {
        pub use crate::protocol::dead::*;
    }
}
"#,
    );
    write(
        helper.join("src/protocol/mod.rs"),
        r#"pub mod dead;
pub mod params;
"#,
    );
    write(
        helper.join("src/protocol/params.rs"),
        r#"pub struct LivePatch {
    pub path: Vec<LiveSegment>,
}

pub enum LiveSegment {
    Index(usize),
    Key(String),
}
"#,
    );
    write(
        helper.join("src/protocol/dead.rs"),
        r#"pub struct DeadPatch {
    pub value: String,
}
"#,
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("nested inline support facade module should reduce");

    let helper_lib = read(output.join("support/external-helper/src/lib.rs"));
    let protocol_mod = read(output.join("support/external-helper/src/protocol/mod.rs"));
    let params = read(output.join("support/external-helper/src/protocol/params.rs"));
    assert!(helper_lib.contains("pub mod facade"), "{helper_lib}");
    assert!(helper_lib.contains("pub mod nested"), "{helper_lib}");
    assert!(
        helper_lib.contains("pub use crate::protocol::params::*"),
        "{helper_lib}"
    );
    assert!(!helper_lib.contains("dead_nested"), "{helper_lib}");
    assert!(!helper_lib.contains("dead_facade"), "{helper_lib}");
    assert!(!helper_lib.contains("protocol::dead"), "{helper_lib}");
    assert!(protocol_mod.contains("pub mod params"), "{protocol_mod}");
    assert!(!protocol_mod.contains("dead"), "{protocol_mod}");
    assert!(params.contains("pub struct LivePatch"), "{params}");
    assert!(params.contains("pub enum LiveSegment"), "{params}");
    assert!(!output
        .join("support/external-helper/src/protocol/dead.rs")
        .exists());

    fs::rename(&helper, helper.with_extension("moved"))
        .expect("original helper package should move away");
    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated support nested inline facade slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nhelper/src/lib.rs:\n{}\nhelper/src/protocol/mod.rs:\n{}\nhelper/src/protocol/params.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        helper_lib,
        protocol_mod,
        params,
    );
}

#[test]
fn prunes_support_packages_reached_through_inline_facade_external_children() {
    let workspace = temp_path("support-inline-external-child-workspace");
    let output = temp_path("support-inline-external-child-output");
    let target_dir = temp_path("support-inline-external-child-target");
    let helper = temp_path("support-inline-external-child-helper");
    let opensourced_path = repo_root().join("crates/opensourced");

    write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
external-helper = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path),
            manifest_path(&helper)
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use external_helper::facade::LivePatch;
use opensourced::opensourced;

#[opensourced]
pub fn selected(patch: LivePatch) -> usize {
    patch.path.len()
}
"#,
    );
    write(
        helper.join("Cargo.toml"),
        r#"[package]
name = "external-helper"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        helper.join("src/lib.rs"),
        r#"pub mod protocol;

pub mod facade {
    pub mod dead_child;
    pub mod nested;

    pub use dead_child::*;
    pub use nested::*;
}
"#,
    );
    write(
        helper.join("src/facade/nested.rs"),
        r#"pub use crate::protocol::dead::*;
pub use crate::protocol::params::*;

pub fn dead_nested() -> crate::protocol::dead::DeadPatch {
    crate::protocol::dead::DeadPatch {
        value: String::new(),
    }
}
"#,
    );
    write(
        helper.join("src/facade/dead_child.rs"),
        r#"pub use crate::protocol::dead::*;

pub fn dead_child_patch() -> crate::protocol::dead::DeadPatch {
    crate::protocol::dead::DeadPatch {
        value: String::new(),
    }
}
"#,
    );
    write(
        helper.join("src/protocol/mod.rs"),
        r#"pub mod dead;
pub mod params;
"#,
    );
    write(
        helper.join("src/protocol/params.rs"),
        r#"pub struct LivePatch {
    pub path: Vec<LiveSegment>,
}

pub enum LiveSegment {
    Index(usize),
    Key(String),
}
"#,
    );
    write(
        helper.join("src/protocol/dead.rs"),
        r#"pub struct DeadPatch {
    pub value: String,
}
"#,
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("inline support facade with external child should reduce");

    let helper_lib = read(output.join("support/external-helper/src/lib.rs"));
    let nested = read(output.join("support/external-helper/src/facade/nested.rs"));
    let protocol_mod = read(output.join("support/external-helper/src/protocol/mod.rs"));
    let params = read(output.join("support/external-helper/src/protocol/params.rs"));
    assert!(helper_lib.contains("pub mod facade"), "{helper_lib}");
    assert!(helper_lib.contains("pub mod nested"), "{helper_lib}");
    assert!(helper_lib.contains("pub use nested::*"), "{helper_lib}");
    assert!(!helper_lib.contains("dead_child"), "{helper_lib}");
    assert!(
        nested.contains("pub use crate::protocol::params::*"),
        "{nested}"
    );
    assert!(!nested.contains("protocol::dead"), "{nested}");
    assert!(!nested.contains("dead_nested"), "{nested}");
    assert!(protocol_mod.contains("pub mod params"), "{protocol_mod}");
    assert!(!protocol_mod.contains("dead"), "{protocol_mod}");
    assert!(params.contains("pub struct LivePatch"), "{params}");
    assert!(params.contains("pub enum LiveSegment"), "{params}");
    assert!(!output
        .join("support/external-helper/src/facade/dead_child.rs")
        .exists());
    assert!(!output
        .join("support/external-helper/src/protocol/dead.rs")
        .exists());

    fs::rename(&helper, helper.with_extension("moved"))
        .expect("original helper package should move away");
    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated support inline external child slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nhelper/src/lib.rs:\n{}\nhelper/src/facade/nested.rs:\n{}\nhelper/src/protocol/mod.rs:\n{}\nhelper/src/protocol/params.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        helper_lib,
        nested,
        protocol_mod,
        params,
    );
}

#[test]
fn prunes_unused_support_pub_crate_globs_when_direct_module_path_is_used() {
    let workspace = temp_path("support-pub-crate-glob-workspace");
    let output = temp_path("support-pub-crate-glob-output");
    let target_dir = temp_path("support-pub-crate-glob-target");
    let helper = temp_path("support-pub-crate-glob-helper");
    let opensourced_path = repo_root().join("crates/opensourced");

    write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
external-helper = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path),
            manifest_path(&helper)
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use external_helper::selected_patch;
use opensourced::opensourced;

#[opensourced]
pub fn selected() -> usize {
    selected_patch().path.len()
}
"#,
    );
    write(
        helper.join("Cargo.toml"),
        r#"[package]
name = "external-helper"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        helper.join("src/lib.rs"),
        r#"pub mod details;

pub(crate) use details::*;

pub fn selected_patch() -> details::LivePatch {
    details::LivePatch {
        path: vec![String::from("live")],
    }
}
"#,
    );
    write(
        helper.join("src/details.rs"),
        r#"pub struct LivePatch {
    pub path: Vec<String>,
}

pub struct DeadPatch {
    pub path: Vec<String>,
}

pub fn dead_helper() -> DeadPatch {
    DeadPatch {
        path: vec![String::from("dead")],
    }
}
"#,
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("support pub(crate) glob should reduce");

    let helper_lib = read(output.join("support/external-helper/src/lib.rs"));
    let details = read(output.join("support/external-helper/src/details.rs"));
    assert!(helper_lib.contains("pub mod details"), "{helper_lib}");
    assert!(helper_lib.contains("details::LivePatch"), "{helper_lib}");
    assert!(
        !helper_lib.contains("pub(crate) use details::*"),
        "{helper_lib}"
    );
    assert!(details.contains("pub struct LivePatch"), "{details}");
    assert!(!details.contains("DeadPatch"), "{details}");
    assert!(!details.contains("dead_helper"), "{details}");

    fs::rename(&helper, helper.with_extension("moved"))
        .expect("original helper package should move away");
    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated support pub(crate) glob slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nhelper/src/lib.rs:\n{}\nhelper/src/details.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        helper_lib,
        details,
    );
}

#[test]
fn prunes_support_package_items_with_parent_module_dependencies() {
    let workspace = temp_path("support-parent-module-workspace");
    let output = temp_path("support-parent-module-output");
    let target_dir = temp_path("support-parent-module-target");
    let helper = temp_path("support-parent-module-helper");
    let opensourced_path = repo_root().join("crates/opensourced");

    write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
external-helper = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path),
            manifest_path(&helper)
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected(value: &str) -> String {
    external_helper::LivePayload::new(value).decorate()
}
"#,
    );
    write(
        helper.join("Cargo.toml"),
        r#"[package]
name = "external-helper"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        helper.join("src/lib.rs"),
        r#"mod dead;
mod helper;
mod nested;

pub use nested::LivePayload;
"#,
    );
    write(
        helper.join("src/helper.rs"),
        r#"pub fn label(value: &str) -> String {
    format!("parent:{value}")
}

pub fn dead_label(value: &str) -> String {
    format!("dead:{value}")
}
"#,
    );
    write(
        helper.join("src/nested.rs"),
        r#"pub struct LivePayload {
    value: String,
}

impl LivePayload {
    pub fn new(value: &str) -> Self {
        Self {
            value: value.to_string(),
        }
    }

    pub fn decorate(self) -> String {
        super::helper::label(&self.value)
    }
}
"#,
    );
    write(
        helper.join("src/dead.rs"),
        r#"pub fn dead_export() -> String {
    super::helper::dead_label("unused")
}
"#,
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let helper_lib = read(output.join("support/external-helper/src/lib.rs"));
    let helper_nested = read(output.join("support/external-helper/src/nested.rs"));
    let helper_helper = read(output.join("support/external-helper/src/helper.rs"));
    assert!(helper_lib.contains("mod helper"), "{helper_lib}");
    assert!(helper_lib.contains("mod nested"), "{helper_lib}");
    assert!(!helper_lib.contains("dead"), "{helper_lib}");
    assert!(
        helper_nested.contains("super::helper::label"),
        "{helper_nested}"
    );
    assert!(helper_helper.contains("pub fn label"), "{helper_helper}");
    assert!(!helper_helper.contains("dead_label"), "{helper_helper}");
    assert!(!output.join("support/external-helper/src/dead.rs").exists());

    fs::rename(&helper, helper.with_extension("moved"))
        .expect("original helper package should move away");
    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated parent-module support dependency slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nhelper/src/lib.rs:\n{}\nhelper/src/nested.rs:\n{}\nhelper/src/helper.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        helper_lib,
        helper_nested,
        helper_helper,
    );
}

#[test]
fn retains_support_methods_called_through_typed_locals_and_prunes_dead_siblings() {
    let workspace = temp_path("support-typed-local-method-workspace");
    let output = temp_path("support-typed-local-method-output");
    let target_dir = temp_path("support-typed-local-method-target");
    let provider = temp_path("support-typed-local-method-provider");
    let opensourced_path = repo_root().join("crates/opensourced");

    write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
provider = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path),
            manifest_path(&provider)
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected(value: &str) -> String {
    let payload = provider::Payload::new(value);
    payload.render()
}
"#,
    );
    write(
        provider.join("Cargo.toml"),
        r#"[package]
name = "provider"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        provider.join("src/lib.rs"),
        r#"pub struct Payload {
    value: String,
}

impl Payload {
    pub fn new(value: &str) -> Self {
        Self {
            value: value.to_string(),
        }
    }

    pub fn render(self) -> String {
        format!("live:{}", self.value)
    }

    pub fn dead_public(self) -> String {
        format!("dead:{}", self.value)
    }
}

pub fn unrelated() -> String {
    "unused".to_string()
}
"#,
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let provider_lib = read(output.join("support/provider/src/lib.rs"));
    assert!(
        provider_lib.contains("pub struct Payload"),
        "{provider_lib}"
    );
    assert!(provider_lib.contains("pub fn new"), "{provider_lib}");
    assert!(provider_lib.contains("pub fn render"), "{provider_lib}");
    assert!(
        !provider_lib.contains("dead_public"),
        "unused public support methods must not survive typed-local method slicing\n{provider_lib}"
    );
    assert!(!provider_lib.contains("unrelated"), "{provider_lib}");

    fs::rename(&provider, provider.with_extension("moved"))
        .expect("original provider package should move away");
    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated typed-local support method slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nprovider/src/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        provider_lib,
    );
}

#[test]
fn prunes_type_only_support_public_inherent_methods() {
    let workspace = temp_path("support-type-only-method-workspace");
    let output = temp_path("support-type-only-method-output");
    let target_dir = temp_path("support-type-only-method-target");
    let provider = temp_path("support-type-only-method-provider");
    let opensourced_path = repo_root().join("crates/opensourced");

    write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
provider = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path),
            manifest_path(&provider)
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected(value: &str) -> String {
    provider::label(provider::Payload {
        value: value.to_string(),
    })
}
"#,
    );
    write(
        provider.join("Cargo.toml"),
        r#"[package]
name = "provider"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        provider.join("src/lib.rs"),
        r#"pub struct Payload {
    pub value: String,
}

impl Payload {
    pub fn dead_public(&self) -> String {
        format!("dead:{}", self.value)
    }
}

pub fn label(payload: Payload) -> String {
    format!("live:{}", payload.value)
}
"#,
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let provider_lib = read(output.join("support/provider/src/lib.rs"));
    assert!(
        provider_lib.contains("pub struct Payload"),
        "{provider_lib}"
    );
    assert!(provider_lib.contains("pub fn label"), "{provider_lib}");
    assert!(
        !provider_lib.contains("dead_public"),
        "type-only support retention must not keep unused public inherent methods\n{provider_lib}"
    );

    fs::rename(&provider, provider.with_extension("moved"))
        .expect("original provider package should move away");
    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated type-only support method slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nprovider/src/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        provider_lib,
    );
}

#[test]
fn retains_support_package_runtime_used_by_proc_macro_expansion() {
    let workspace = temp_path("support-proc-macro-runtime-workspace");
    let output = temp_path("support-proc-macro-runtime-output");
    let target_dir = temp_path("support-proc-macro-runtime-target");
    let external = temp_path("support-proc-macro-runtime-external");
    let provider = external.join("provider");
    let derive_runtime = external.join("derive-runtime");
    let runtime_dep = external.join("runtime-dep");
    let opensourced_path = repo_root().join("crates/opensourced");

    write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
provider = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path),
            manifest_path(&provider)
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected(value: &str) -> String {
    provider::render_wire(provider::Wire {
        value: value.to_string(),
    })
}
"#,
    );
    write(
        provider.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "provider"
version = "0.1.0"
edition = "2021"

[dependencies]
derive-runtime = {{ path = "{}" }}
runtime-dep = {{ path = "{}" }}
"#,
            manifest_path(&derive_runtime),
            manifest_path(&runtime_dep)
        ),
    );
    write(
        provider.join("src/lib.rs"),
        r#"mod dead;
mod runtime;
mod wire;

pub use wire::{Wire, render_wire};
"#,
    );
    write(
        provider.join("src/runtime.rs"),
        r#"pub trait RuntimeTrait {
    fn marker(&self) -> &'static str;
}

pub fn dead_runtime() -> &'static str {
    "dead"
}
"#,
    );
    write(
        provider.join("src/wire.rs"),
        r#"use crate::runtime::RuntimeTrait;
use derive_runtime::UseRuntime;

#[derive(UseRuntime)]
pub struct Wire {
    pub value: String,
}

pub fn render_wire(wire: Wire) -> String {
    format!("{}:{}", wire.value, wire.marker())
}
"#,
    );
    write(
        provider.join("src/dead.rs"),
        r#"pub fn dead_provider() -> &'static str {
    "dead"
}
"#,
    );
    write(
        derive_runtime.join("Cargo.toml"),
        r#"[package]
name = "derive-runtime"
version = "0.1.0"
edition = "2021"

[lib]
proc-macro = true
"#,
    );
    write(
        derive_runtime.join("src/lib.rs"),
        r#"extern crate proc_macro;

use proc_macro::TokenStream;

macro_rules! quote {
    ($($tokens:tt)*) => {
        stringify!($($tokens)*)
    };
}

#[proc_macro_derive(UseRuntime)]
pub fn use_runtime(input: TokenStream) -> TokenStream {
    let input = input.to_string();
    let name = input
        .split_whitespace()
        .skip_while(|token| *token != "struct")
        .nth(1)
        .expect("derive input should contain a struct name")
        .trim_matches('{')
        .trim_matches(';')
        .split('<')
        .next()
        .expect("struct name should not be empty");
    quote! {
        impl crate::runtime::RuntimeTrait for __SLICERS_DERIVE_TARGET__ {
            fn marker(&self) -> &'static str {
                ::runtime_dep::marker()
            }
        }
    }
    .replace("__SLICERS_DERIVE_TARGET__", name)
    .parse()
    .expect("generated derive output should parse")
}
"#,
    );
    write(
        runtime_dep.join("Cargo.toml"),
        r#"[package]
name = "runtime-dep"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        runtime_dep.join("src/lib.rs"),
        r#"pub fn marker() -> &'static str {
    "macro"
}

pub fn dead_marker() -> &'static str {
    "dead"
}
"#,
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let provider_lib = read(output.join("support/provider/src/lib.rs"));
    let provider_runtime = read(output.join("support/provider/src/runtime.rs"));
    let provider_wire = read(output.join("support/provider/src/wire.rs"));
    let provider_manifest = read(output.join("support/provider/Cargo.toml"));
    let runtime_dep_source = read(output.join("support/runtime-dep/src/lib.rs"));
    assert!(provider_lib.contains("mod runtime"), "{provider_lib}");
    assert!(
        provider_lib.contains("pub use wire::{Wire, render_wire}"),
        "{provider_lib}"
    );
    assert!(!provider_lib.contains("dead"), "{provider_lib}");
    assert!(
        provider_runtime.contains("pub trait RuntimeTrait"),
        "{provider_runtime}"
    );
    assert!(
        !provider_runtime.contains("dead_runtime"),
        "{provider_runtime}"
    );
    assert!(
        provider_wire.contains("derive_runtime::UseRuntime"),
        "{provider_wire}"
    );
    assert!(
        provider_manifest.contains("[dependencies.runtime-dep]"),
        "{provider_manifest}"
    );
    assert!(
        runtime_dep_source.contains("pub fn marker"),
        "{runtime_dep_source}"
    );
    assert!(
        !runtime_dep_source.contains("dead_marker"),
        "{runtime_dep_source}"
    );
    assert!(!output.join("support/provider/src/dead.rs").exists());

    fs::rename(&external, external.with_extension("moved"))
        .expect("original support packages should move away");
    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated proc-macro runtime support slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nprovider/Cargo.toml:\n{}\nprovider/src/lib.rs:\n{}\nprovider/src/runtime.rs:\n{}\nprovider/src/wire.rs:\n{}\nruntime-dep/src/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        provider_manifest,
        provider_lib,
        provider_runtime,
        provider_wire,
        runtime_dep_source,
    );
}

#[test]
fn prunes_support_impls_without_live_local_header_types() {
    let workspace = temp_path("support-dead-impl-workspace");
    let output = temp_path("support-dead-impl-output");
    let target_dir = temp_path("support-dead-impl-target");
    let external = temp_path("support-dead-impl-external");
    let provider = external.join("provider");
    let external_core = external.join("external-core");
    let opensourced_path = repo_root().join("crates/opensourced");

    write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
provider = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path),
            manifest_path(&provider)
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected(value: &str) -> String {
    provider::LivePayload::new(value).render()
}
"#,
    );
    write(
        provider.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "provider"
version = "0.1.0"
edition = "2021"

[dependencies]
external-core = {{ path = "{}" }}
"#,
            manifest_path(&external_core)
        ),
    );
    write(
        provider.join("src/lib.rs"),
        r#"mod dead;
mod live;

pub use live::LivePayload;

impl From<dead::DeadPayload> for external_core::ExternalPayload {
    fn from(value: dead::DeadPayload) -> Self {
        external_core::ExternalPayload(value.value)
    }
}
"#,
    );
    write(
        provider.join("src/live.rs"),
        r#"pub struct LivePayload {
    value: String,
}

impl LivePayload {
    pub fn new(value: &str) -> Self {
        Self {
            value: value.to_string(),
        }
    }

    pub fn render(self) -> String {
        self.value
    }
}
"#,
    );
    write(
        provider.join("src/dead.rs"),
        r#"pub struct DeadPayload {
    pub value: String,
}
"#,
    );
    write(
        external_core.join("Cargo.toml"),
        r#"[package]
name = "external-core"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        external_core.join("src/lib.rs"),
        r#"pub struct ExternalPayload(pub String);

pub fn dead_external() -> &'static str {
    "dead"
}
"#,
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let provider_manifest = read(output.join("support/provider/Cargo.toml"));
    let provider_lib = read(output.join("support/provider/src/lib.rs"));
    let provider_live = read(output.join("support/provider/src/live.rs"));
    assert!(provider_lib.contains("mod live"), "{provider_lib}");
    assert!(
        provider_lib.contains("pub use live::LivePayload"),
        "{provider_lib}"
    );
    assert!(!provider_lib.contains("mod dead"), "{provider_lib}");
    assert!(
        !provider_lib.contains("impl From<dead::DeadPayload>"),
        "{provider_lib}"
    );
    assert!(!provider_lib.contains("external_core"), "{provider_lib}");
    assert!(
        !provider_manifest.contains("external-core"),
        "{provider_manifest}"
    );
    assert!(
        provider_live.contains("pub struct LivePayload"),
        "{provider_live}"
    );
    assert!(!output.join("support/provider/src/dead.rs").exists());
    assert!(!output.join("support/external-core").exists());

    fs::rename(&external, external.with_extension("moved"))
        .expect("original support packages should move away");
    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated dead support impl slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nprovider/Cargo.toml:\n{}\nprovider/src/lib.rs:\n{}\nprovider/src/live.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        provider_manifest,
        provider_lib,
        provider_live,
    );
}

#[test]
fn prunes_support_conversion_impls_when_any_local_header_type_is_dead() {
    let workspace = temp_path("support-dead-conversion-impl-workspace");
    let output = temp_path("support-dead-conversion-impl-output");
    let target_dir = temp_path("support-dead-conversion-impl-target");
    let provider = temp_path("support-dead-conversion-impl-provider");
    let opensourced_path = repo_root().join("crates/opensourced");

    write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
provider = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path),
            manifest_path(&provider)
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected(value: &str) -> String {
    provider::LiveModel::new(value).render()
}
"#,
    );
    write(
        provider.join("Cargo.toml"),
        r#"[package]
name = "provider"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        provider.join("src/lib.rs"),
        r#"pub struct LiveModel {
    value: String,
}

pub struct DeadPolicy {
    value: String,
}

impl LiveModel {
    pub fn new(value: &str) -> Self {
        Self {
            value: value.to_string(),
        }
    }

    pub fn render(self) -> String {
        self.value
    }
}

impl From<DeadPolicy> for LiveModel {
    fn from(value: DeadPolicy) -> Self {
        Self { value: value.value }
    }
}
"#,
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let provider_lib = read(output.join("support/provider/src/lib.rs"));
    assert!(
        provider_lib.contains("pub struct LiveModel"),
        "{provider_lib}"
    );
    assert!(!provider_lib.contains("DeadPolicy"), "{provider_lib}");
    assert!(
        !provider_lib.contains("impl From<DeadPolicy>"),
        "{provider_lib}"
    );

    fs::rename(&provider, provider.with_extension("moved"))
        .expect("original provider package should move away");
    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated dead conversion impl support slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nprovider/src/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        provider_lib,
    );
}

#[test]
fn prunes_unused_support_inherent_impl_methods_when_assoc_requirements_are_known() {
    let workspace = temp_path("support-inherent-impl-method-workspace");
    let output = temp_path("support-inherent-impl-method-output");
    let target_dir = temp_path("support-inherent-impl-method-target");
    let provider = temp_path("support-inherent-impl-method-provider");
    let opensourced_path = repo_root().join("crates/opensourced");

    write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
provider = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path),
            manifest_path(&provider)
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected(value: &str) -> String {
    let direct = provider::LiveModel::new(value).render();
    format!("{direct}:{}", provider::selected_text(value))
}
"#,
    );
    write(
        provider.join("Cargo.toml"),
        r#"[package]
name = "provider"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        provider.join("src/lib.rs"),
        r#"pub struct LiveModel {
    value: String,
}

pub struct DeadPayload {
    value: String,
}

pub fn selected_text(value: &str) -> String {
    LiveModel::new(value).into_string()
}

impl LiveModel {
    pub fn new(value: &str) -> Self {
        Self {
            value: Self::normalize(value),
        }
    }

    fn normalize(value: &str) -> String {
        value.trim().to_string()
    }

    pub fn render(self) -> String {
        self.value
    }

    pub fn into_string(self) -> String {
        self.value
    }

    pub fn dead_payload(self) -> DeadPayload {
        DeadPayload { value: self.value }
    }
}

impl From<LiveModel> for String {
    fn from(value: LiveModel) -> Self {
        value.into_string()
    }
}
"#,
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let provider_lib = read(output.join("support/provider/src/lib.rs"));
    assert!(
        provider_lib.contains("pub struct LiveModel"),
        "{provider_lib}"
    );
    assert!(provider_lib.contains("pub fn new"), "{provider_lib}");
    assert!(provider_lib.contains("fn normalize"), "{provider_lib}");
    assert!(provider_lib.contains("pub fn render"), "{provider_lib}");
    assert!(
        provider_lib.contains("pub fn into_string"),
        "{provider_lib}"
    );
    assert!(
        !provider_lib.contains("pub fn dead_payload"),
        "{provider_lib}"
    );
    assert!(!provider_lib.contains("DeadPayload"), "{provider_lib}");

    fs::rename(&provider, provider.with_extension("moved"))
        .expect("original provider package should move away");
    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated inherent impl method support slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nprovider/src/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        provider_lib,
    );
}

#[test]
fn propagates_support_assoc_method_requirements_through_imported_dependency_types() {
    let workspace = temp_path("support-imported-assoc-workspace");
    let output = temp_path("support-imported-assoc-output");
    let target_dir = temp_path("support-imported-assoc-target");
    let external = temp_path("support-imported-assoc-external");
    let provider = external.join("provider");
    let path_model = external.join("path-model");
    let opensourced_path = repo_root().join("crates/opensourced");

    write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
provider = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path),
            manifest_path(&provider)
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected(value: &str) -> bool {
    provider::selected(value)
}
"#,
    );
    write(
        provider.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "provider"
version = "0.1.0"
edition = "2021"

[dependencies]
path-model = {{ path = "{}" }}
"#,
            manifest_path(&path_model)
        ),
    );
    write(
        provider.join("src/lib.rs"),
        r#"use path_model::PathModel;

pub fn selected(value: &str) -> bool {
    let model = PathModel::new(value);
    model.as_path().is_absolute() && model.join("child").as_path().ends_with("child")
}
"#,
    );
    write(
        path_model.join("Cargo.toml"),
        r#"[package]
name = "path-model"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        path_model.join("src/lib.rs"),
        r#"use std::path::{Path, PathBuf};

pub struct PathModel(PathBuf);

impl PathModel {
    pub fn new(value: &str) -> Self {
        Self(PathBuf::from(value))
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }

    pub fn join<P: AsRef<Path>>(&self, path: P) -> Self {
        Self(self.0.join(path))
    }

    pub fn dead_marker(&self) -> String {
        self.0.display().to_string()
    }
}
"#,
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let provider_lib = read(output.join("support/provider/src/lib.rs"));
    let path_model_lib = read(output.join("support/path-model/src/lib.rs"));
    assert!(provider_lib.contains("PathModel"), "{provider_lib}");
    assert!(path_model_lib.contains("pub fn new"), "{path_model_lib}");
    assert!(
        path_model_lib.contains("pub fn as_path"),
        "{path_model_lib}"
    );
    assert!(path_model_lib.contains("pub fn join"), "{path_model_lib}");
    assert!(!path_model_lib.contains("dead_marker"), "{path_model_lib}");

    fs::rename(&external, external.with_extension("moved"))
        .expect("original support packages should move away");
    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated imported assoc support slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nprovider/src/lib.rs:\n{}\npath-model/src/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        provider_lib,
        path_model_lib,
    );
}

#[test]
fn propagates_support_assoc_methods_from_macro_body_receivers_through_reexported_aliases() {
    let workspace = temp_path("support-macro-reexport-assoc-workspace");
    let output = temp_path("support-macro-reexport-assoc-output");
    let target_dir = temp_path("support-macro-reexport-assoc-target");
    let external = temp_path("support-macro-reexport-assoc-external");
    let provider = external.join("provider");
    let path_model = external.join("path-model");
    let opensourced_path = repo_root().join("crates/opensourced");

    write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
provider = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path),
            manifest_path(&provider)
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected(value: &str) -> String {
    provider::selected(value)
}
"#,
    );
    write(
        provider.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "provider"
version = "0.1.0"
edition = "2021"

[dependencies]
path-model = {{ path = "{}" }}
"#,
            manifest_path(&path_model)
        ),
    );
    write(
        provider.join("src/lib.rs"),
        r#"pub use path_model::PathModel as PublicPath;

macro_rules! render_model_path {
    ($model:expr) => {{
        $model.as_path().display().to_string()
    }};
}

pub fn selected(value: &str) -> String {
    let model = PublicPath::new(value);
    render_model_path!(model)
}
"#,
    );
    write(
        path_model.join("Cargo.toml"),
        r#"[package]
name = "path-model"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        path_model.join("src/lib.rs"),
        r#"use std::path::{Path, PathBuf};

pub struct PathModel(PathBuf);

impl PathModel {
    pub fn new(value: &str) -> Self {
        Self(PathBuf::from(value))
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }

    pub fn dead_marker(&self) -> String {
        self.0.display().to_string()
    }
}
"#,
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let provider_lib = read(output.join("support/provider/src/lib.rs"));
    let path_model_lib = read(output.join("support/path-model/src/lib.rs"));
    assert!(provider_lib.contains("render_model_path"), "{provider_lib}");
    assert!(provider_lib.contains("PublicPath::new"), "{provider_lib}");
    assert!(path_model_lib.contains("pub fn new"), "{path_model_lib}");
    assert!(
        path_model_lib.contains("pub fn as_path"),
        "macro body receiver method on a reexported support alias should retain the dependency method\n{path_model_lib}"
    );
    assert!(!path_model_lib.contains("dead_marker"), "{path_model_lib}");

    fs::rename(&external, external.with_extension("moved"))
        .expect("original support packages should move away");
    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .env("RUSTFLAGS", "-D warnings")
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated macro reexport assoc support slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nprovider/src/lib.rs:\n{}\npath-model/src/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        provider_lib,
        path_model_lib,
    );
}

#[test]
fn retains_support_strum_runtime_dependency_for_enum_iter_derive() {
    let workspace = temp_path("support-strum-runtime-workspace");
    let output = temp_path("support-strum-runtime-output");
    let target_dir = temp_path("support-strum-runtime-target");
    let provider = temp_path("support-strum-runtime-provider");
    let opensourced_path = repo_root().join("crates/opensourced");

    write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
provider = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path),
            manifest_path(&provider)
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected() -> provider::Mode {
    provider::selected()
}
"#,
    );
    write(
        provider.join("Cargo.toml"),
        r#"[package]
name = "provider"
version = "0.1.0"
edition = "2021"

[dependencies]
strum = "0.27"
strum_macros = "0.27"
"#,
    );
    write(
        provider.join("src/lib.rs"),
        r#"use strum_macros::EnumIter;

#[derive(Debug, Clone, Copy, EnumIter)]
pub enum Mode {
    Fast,
    Slow,
}

pub fn selected() -> Mode {
    Mode::Fast
}
"#,
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let provider_manifest = read(output.join("support/provider/Cargo.toml"));
    let provider_lib = read(output.join("support/provider/src/lib.rs"));
    assert!(provider_manifest.contains("strum"), "{provider_manifest}");
    assert!(
        provider_manifest.contains("strum_macros"),
        "{provider_manifest}"
    );
    assert!(provider_lib.contains("EnumIter"), "{provider_lib}");

    fs::rename(&provider, provider.with_extension("moved"))
        .expect("original provider package should move away");
    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated strum runtime support slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nprovider/Cargo.toml:\n{}\nprovider/src/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        provider_manifest,
        provider_lib,
    );
}

#[test]
fn retains_support_serde_attr_helpers_and_variant_payload_imports() {
    let workspace = temp_path("support-serde-attr-variant-workspace");
    let output = temp_path("support-serde-attr-variant-output");
    let target_dir = temp_path("support-serde-attr-variant-target");
    let provider = temp_path("support-serde-attr-variant-provider");
    let opensourced_path = repo_root().join("crates/opensourced");

    write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
provider = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path),
            manifest_path(&provider)
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected() -> (provider::Event, provider::Model) {
    provider::selected()
}
"#,
    );
    write(
        provider.join("Cargo.toml"),
        r#"[package]
name = "provider"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
"#,
    );
    write(
        provider.join("src/lib.rs"),
        r#"mod dynamic_tools;

use crate::dynamic_tools::DynamicToolCallRequest;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Event {
    DynamicToolCallRequest(DynamicToolCallRequest),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Model {
    #[serde(default, skip_serializing_if = "should_serialize_reasoning_content")]
    pub content: Option<String>,
    #[serde(default, deserialize_with = "deserialize_lossy_opt_i64")]
    pub count: Option<i64>,
}

fn should_serialize_reasoning_content(content: &Option<String>) -> bool {
    content.is_some()
}

fn deserialize_lossy_opt_i64<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    Ok(value.and_then(|value| value.as_i64()))
}

pub fn selected() -> (Event, Model) {
    (
        Event::DynamicToolCallRequest(DynamicToolCallRequest {
            call_id: "call".to_string(),
            arguments: Value::Null,
        }),
        Model {
            content: Some("ok".to_string()),
            count: Some(1),
        },
    )
}
"#,
    );
    write(
        provider.join("src/dynamic_tools.rs"),
        r#"use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DynamicToolCallRequest {
    pub call_id: String,
    pub arguments: Value,
}
"#,
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let provider_lib = read(output.join("support/provider/src/lib.rs"));
    assert!(
        provider_lib.contains("use crate::dynamic_tools::DynamicToolCallRequest"),
        "{provider_lib}"
    );
    assert!(
        provider_lib.contains("should_serialize_reasoning_content"),
        "{provider_lib}"
    );
    assert!(
        provider_lib.contains("deserialize_lossy_opt_i64"),
        "{provider_lib}"
    );
    assert!(output
        .join("support/provider/src/dynamic_tools.rs")
        .exists());

    fs::rename(&provider, provider.with_extension("moved"))
        .expect("original provider package should move away");
    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated serde attr variant support slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nprovider/src/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        provider_lib,
    );
}

#[test]
fn retains_support_enum_variant_payload_methods_without_widening_same_named_methods() {
    let workspace = temp_path("support-variant-payload-methods-workspace");
    let output = temp_path("support-variant-payload-methods-output");
    let target_dir = temp_path("support-variant-payload-methods-target");
    let provider = temp_path("support-variant-payload-methods-provider");
    let opensourced_path = repo_root().join("crates/opensourced");

    write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
provider = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path),
            manifest_path(&provider)
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected() -> String {
    provider::selected()
}
"#,
    );
    write(
        provider.join("Cargo.toml"),
        r#"[package]
name = "provider"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        provider.join("src/lib.rs"),
        r#"pub enum TurnItem {
    User(UserItem),
    Agent(AgentItem),
    Search(SearchItem),
    Compact(CompactItem),
    Marker(MarkerItem),
}

pub struct UserItem;
pub struct AgentItem;
pub struct SearchItem;
pub struct CompactItem;
pub struct MarkerItem;
pub struct LooseItem;

impl UserItem {
    pub fn as_event(&self) -> String {
        "user".to_string()
    }

    pub fn helper(&self) -> String {
        "helper".to_string()
    }
}

impl AgentItem {
    pub fn as_event(&self) -> String {
        "agent".to_string()
    }
}

impl SearchItem {
    pub fn as_event(&self) -> String {
        "search".to_string()
    }
}

impl CompactItem {
    pub fn as_event(&self) -> String {
        "compact".to_string()
    }
}

impl MarkerItem {
    fn unused_private(&self) -> String {
        "marker".to_string()
    }
}

impl LooseItem {
    pub fn as_event(&self) -> String {
        "loose".to_string()
    }
}

impl TurnItem {
    pub fn as_events(&self) -> Vec<String> {
        match self {
            TurnItem::User(item) => vec![item.as_event()],
            TurnItem::Agent(item) => vec![item.as_event()],
            TurnItem::Search(item) => vec![item.as_event()],
            TurnItem::Compact(item) => vec![item.as_event()],
            TurnItem::Marker(_) => Vec::new(),
        }
    }
}

pub fn selected() -> String {
    let user = UserItem;
    let helper = user.helper();
    format!("{helper}:{}", TurnItem::User(user).as_events().join(","))
}
"#,
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let provider_lib = read(output.join("support/provider/src/lib.rs"));
    assert!(provider_lib.contains("impl UserItem"), "{provider_lib}");
    assert!(
        provider_lib.contains("pub fn as_event(&self)"),
        "{provider_lib}"
    );
    assert!(
        provider_lib.contains("pub fn helper(&self)"),
        "{provider_lib}"
    );
    assert!(
        !provider_lib.contains("LooseItem"),
        "unrelated same-named method receiver should stay pruned\n{provider_lib}"
    );
    assert!(
        !provider_lib.contains("unused_private"),
        "unused private inherent methods on live payload types should stay pruned\n{provider_lib}"
    );

    fs::rename(&provider, provider.with_extension("moved"))
        .expect("original provider package should move away");
    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .env("RUSTFLAGS", "-D warnings")
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated variant payload method support slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nprovider/src/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        provider_lib,
    );
}

#[test]
fn retains_support_assoc_constructor_on_imported_self_alias() {
    let workspace = temp_path("support-imported-self-alias-workspace");
    let output = temp_path("support-imported-self-alias-output");
    let target_dir = temp_path("support-imported-self-alias-target");
    let provider_a = temp_path("support-imported-self-alias-provider-a");
    let provider_b = temp_path("support-imported-self-alias-provider-b");
    let opensourced_path = repo_root().join("crates/opensourced");

    write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
provider_b = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path),
            manifest_path(&provider_b)
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected() -> provider_b::TextElement {
    provider_b::selected()
}
"#,
    );
    write(
        provider_a.join("Cargo.toml"),
        r#"[package]
name = "provider_a"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        provider_a.join("src/lib.rs"),
        r#"#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ByteRange {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextElement {
    pub byte_range: ByteRange,
    placeholder: Option<String>,
}

impl TextElement {
    pub fn new(byte_range: ByteRange, placeholder: Option<String>) -> Self {
        Self { byte_range, placeholder }
    }

    pub fn placeholder(&self) -> Option<&str> {
        self.placeholder.as_deref()
    }
}
"#,
    );
    write(
        provider_b.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "provider_b"
version = "0.1.0"
edition = "2021"

[dependencies]
provider_a = {{ path = "{}" }}
"#,
            manifest_path(&provider_a)
        ),
    );
    write(
        provider_b.join("src/lib.rs"),
        r#"use provider_a::ByteRange as CoreByteRange;
use provider_a::TextElement as CoreTextElement;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ByteRange {
    pub start: usize,
    pub end: usize,
}

impl From<ByteRange> for CoreByteRange {
    fn from(value: ByteRange) -> Self {
        Self { start: value.start, end: value.end }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextElement {
    pub byte_range: ByteRange,
    placeholder: Option<String>,
}

impl TextElement {
    pub fn new(byte_range: ByteRange, placeholder: Option<String>) -> Self {
        Self { byte_range, placeholder }
    }

    pub fn from_core(value: CoreTextElement) -> Self {
        Self::new(
            ByteRange {
                start: value.byte_range.start,
                end: value.byte_range.end,
            },
            value.placeholder().map(str::to_string),
        )
    }
}

impl From<TextElement> for CoreTextElement {
    fn from(value: TextElement) -> Self {
        Self::new(value.byte_range.into(), value.placeholder)
    }
}

pub fn selected() -> TextElement {
    let local = TextElement::new(ByteRange { start: 1, end: 2 }, Some("x".to_string()));
    let core: CoreTextElement = local.into();
    TextElement::from_core(core)
}
"#,
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let provider_a_lib = read(output.join("support/provider_a/src/lib.rs"));
    assert!(
        provider_a_lib.contains("pub fn new"),
        "imported Self::new target should stay live\n{provider_a_lib}"
    );

    fs::rename(&provider_a, provider_a.with_extension("moved"))
        .expect("original provider_a package should move away");
    fs::rename(&provider_b, provider_b.with_extension("moved"))
        .expect("original provider_b package should move away");
    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .env("RUSTFLAGS", "-D warnings")
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated imported self alias support slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nprovider_a/src/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        provider_a_lib,
    );
}

#[test]
fn prunes_support_items_named_only_as_macro_variant_tokens() {
    let workspace = temp_path("support-macro-variant-workspace");
    let output = temp_path("support-macro-variant-output");
    let target_dir = temp_path("support-macro-variant-target");
    let provider = temp_path("support-macro-variant-provider");
    let opensourced_path = repo_root().join("crates/opensourced");

    write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
provider = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path),
            manifest_path(&provider)
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected() -> String {
    format!("{:?}", provider::selected_status())
}
"#,
    );
    write(
        provider.join("Cargo.toml"),
        r#"[package]
name = "provider"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        provider.join("src/lib.rs"),
        r#"macro_rules! define_status {
    (pub enum $Name:ident { $($Variant:ident),+ $(,)? }) => {
        #[derive(Debug)]
        pub enum $Name {
            $($Variant),+
        }
    };
}

pub struct DeadVariant;

define_status!(pub enum LiveStatus {
    Ready,
    DeadVariant,
});

pub fn selected_status() -> LiveStatus {
    LiveStatus::Ready
}
"#,
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let provider_lib = read(output.join("support/provider/src/lib.rs"));
    assert!(
        provider_lib.contains("macro_rules! define_status"),
        "{provider_lib}"
    );
    assert!(provider_lib.contains("define_status!"), "{provider_lib}");
    assert!(
        provider_lib.contains("pub fn selected_status"),
        "{provider_lib}"
    );
    assert!(
        !provider_lib.contains("pub struct DeadVariant"),
        "{provider_lib}"
    );

    fs::rename(&provider, provider.with_extension("moved"))
        .expect("original provider package should move away");
    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated macro variant support slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nprovider/src/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        provider_lib,
    );
}

#[test]
fn prunes_support_derive_imports_when_only_macro_tokens_match_name() {
    let workspace = temp_path("support-macro-derived-name-workspace");
    let output = temp_path("support-macro-derived-name-output");
    let target_dir = temp_path("support-macro-derived-name-target");
    let provider = temp_path("support-macro-derived-name-provider");
    let opensourced_path = repo_root().join("crates/opensourced");

    write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
provider = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path),
            manifest_path(&provider)
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected() -> String {
    format!("{:?}", provider::selected_status())
}
"#,
    );
    write(
        provider.join("Cargo.toml"),
        r#"[package]
name = "provider"
version = "0.1.0"
edition = "2021"

[dependencies]
thiserror = "2"
"#,
    );
    write(
        provider.join("src/lib.rs"),
        r#"use thiserror::Error;

macro_rules! define_status {
    (pub enum $Name:ident { $($Variant:ident),+ $(,)? }) => {
        #[derive(Debug)]
        pub enum $Name {
            $($Variant),+
        }
    };
}

define_status!(pub enum LiveStatus {
    Ready,
    Error,
});

pub fn selected_status() -> LiveStatus {
    LiveStatus::Ready
}
"#,
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let provider_manifest = read(output.join("support/provider/Cargo.toml"));
    let provider_lib = read(output.join("support/provider/src/lib.rs"));
    assert!(
        !provider_manifest.contains("thiserror"),
        "{provider_manifest}"
    );
    assert!(!provider_lib.contains("thiserror::Error"), "{provider_lib}");
    assert!(provider_lib.contains("Error"), "{provider_lib}");

    fs::rename(&provider, provider.with_extension("moved"))
        .expect("original provider package should move away");
    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated macro derive-name support slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nprovider/Cargo.toml:\n{}\nprovider/src/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        provider_manifest,
        provider_lib,
    );
}

#[test]
fn restricts_nested_support_dependency_paths_without_variant_name_leaks() {
    let workspace = temp_path("support-nested-dependency-path-workspace");
    let output = temp_path("support-nested-dependency-path-output");
    let target_dir = temp_path("support-nested-dependency-path-target");
    let bridge = temp_path("support-nested-dependency-path-bridge");
    let core = temp_path("support-nested-dependency-path-core");
    let opensourced_path = repo_root().join("crates/opensourced");

    write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
bridge = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path),
            manifest_path(&bridge)
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected() -> String {
    bridge::selected_wire_label()
}
"#,
    );
    write(
        bridge.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "bridge"
version = "0.1.0"
edition = "2021"

[dependencies]
core-helper = {{ path = "{}" }}
"#,
            manifest_path(&core)
        ),
    );
    write(
        bridge.join("src/lib.rs"),
        r#"mod api;
pub use api::selected_wire_label;
"#,
    );
    write(
        bridge.join("src/api.rs"),
        r#"use core_helper::protocol::Decision as CoreDecision;

pub enum WireDecision {
    Accept,
    Cancel,
}

pub fn selected_wire_label() -> String {
    match CoreDecision::Approved {
        CoreDecision::Approved => "accept".to_string(),
        CoreDecision::Abort => "cancel".to_string(),
    }
}
"#,
    );
    write(
        core.join("Cargo.toml"),
        r#"[package]
name = "core-helper"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        core.join("src/lib.rs"),
        r#"pub mod dead;
pub mod protocol;
"#,
    );
    write(
        core.join("src/protocol.rs"),
        r#"pub enum Decision {
    Approved,
    Abort,
}
"#,
    );
    write(
        core.join("src/dead.rs"),
        r#"pub struct DeadPayload;
"#,
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let bridge_api = read(output.join("support/bridge/src/api.rs"));
    let core_lib = read(output.join("support/core-helper/src/lib.rs"));
    let core_protocol = read(output.join("support/core-helper/src/protocol.rs"));
    assert!(
        bridge_api.contains("CoreDecision::Approved"),
        "{bridge_api}"
    );
    assert!(core_lib.contains("pub mod protocol"), "{core_lib}");
    assert!(!core_lib.contains("dead"), "{core_lib}");
    assert!(
        core_protocol.contains("pub enum Decision"),
        "{core_protocol}"
    );
    assert!(
        !output.join("support/core-helper/src/dead.rs").exists(),
        "{core_lib}"
    );

    fs::rename(&bridge, bridge.with_extension("moved"))
        .expect("original bridge package should move away");
    fs::rename(&core, core.with_extension("moved"))
        .expect("original core package should move away");
    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated nested support dependency path slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nbridge/src/api.rs:\n{}\ncore/src/lib.rs:\n{}\ncore/src/protocol.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        bridge_api,
        core_lib,
        core_protocol,
    );
}

#[test]
fn prunes_support_imports_named_only_as_qualified_variant_segments() {
    let workspace = temp_path("support-qualified-variant-import-workspace");
    let output = temp_path("support-qualified-variant-import-output");
    let target_dir = temp_path("support-qualified-variant-import-target");
    let provider = temp_path("support-qualified-variant-import-provider");
    let opensourced_path = repo_root().join("crates/opensourced");

    write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
provider = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path),
            manifest_path(&provider)
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected() -> bool {
    provider::selected()
}
"#,
    );
    write(
        provider.join("Cargo.toml"),
        r#"[package]
name = "provider"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        provider.join("src/lib.rs"),
        r#"use std::path::Path;

pub enum FileSystemPath {
    Path { raw: String },
    Other,
}

pub fn selected() -> bool {
    matches!(FileSystemPath::Path { raw: String::new() }, FileSystemPath::Path { .. })
}

pub fn dead(path: &Path) -> bool {
    path.exists()
}
"#,
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let provider_lib = read(output.join("support/provider/src/lib.rs"));
    assert!(
        provider_lib.contains("FileSystemPath::Path"),
        "{provider_lib}"
    );
    assert!(
        !provider_lib.contains("use std::path::Path"),
        "{provider_lib}"
    );
    assert!(!provider_lib.contains("pub fn dead"), "{provider_lib}");

    fs::rename(&provider, provider.with_extension("moved"))
        .expect("original provider package should move away");
    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated qualified variant import support slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nprovider/src/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        provider_lib,
    );
}

#[test]
fn prunes_support_package_with_local_enum_glob_imports() {
    let workspace = temp_path("support-enum-glob-workspace");
    let output = temp_path("support-enum-glob-output");
    let target_dir = temp_path("support-enum-glob-target");
    let provider = temp_path("support-enum-glob-provider");
    let opensourced_path = repo_root().join("crates/opensourced");

    write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
provider = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path),
            manifest_path(&provider)
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected(value: i32) -> u8 {
    provider::classify(value)
}
"#,
    );
    write(
        provider.join("Cargo.toml"),
        r#"[package]
name = "provider"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        provider.join("src/lib.rs"),
        r#"mod dead;
mod errors;

use errors::ParseError::*;

pub fn classify(value: i32) -> u8 {
    match parse(value) {
        Ok(value) => value as u8,
        Err(InvalidPatchError(_)) => 1,
        Err(InvalidHunkError { .. }) => 2,
    }
}

fn parse(value: i32) -> Result<i32, errors::ParseError> {
    match value {
        0 => Err(InvalidPatchError(String::from("zero"))),
        value if value < 0 => Err(InvalidHunkError {
            message: String::from("negative"),
            line_number: 1,
        }),
        _ => Ok(value),
    }
}

pub fn dead() -> dead::Dead {
    dead::Dead
}
"#,
    );
    write(
        provider.join("src/errors.rs"),
        r#"pub enum ParseError {
    InvalidPatchError(String),
    InvalidHunkError { message: String, line_number: usize },
}
"#,
    );
    write(provider.join("src/dead.rs"), "pub struct Dead;\n");

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let provider_lib = read(output.join("support/provider/src/lib.rs"));
    assert!(
        provider_lib.contains("use errors::ParseError::*"),
        "{provider_lib}"
    );
    assert!(!provider_lib.contains("mod dead"), "{provider_lib}");
    assert!(!provider_lib.contains("pub fn dead"), "{provider_lib}");
    assert!(output.join("support/provider/src/errors.rs").exists());
    assert!(!output.join("support/provider/src/dead.rs").exists());

    fs::rename(&provider, provider.with_extension("moved"))
        .expect("original provider package should move away");
    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated enum glob support slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nprovider/src/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        provider_lib,
    );
}

#[test]
fn copies_only_reachable_support_library_modules_and_static_assets() {
    let workspace = temp_path("support-module-closure-workspace");
    let output = temp_path("support-module-closure-output");
    let target_dir = temp_path("support-module-closure-target");
    let external = temp_path("support-module-closure-external");
    let helper = external.join("external-helper");
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
external-helper = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path),
            manifest_path(&helper),
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected(value: &str) -> String {
    external_helper::decorate(value)
}
"#,
    );
    write(
        helper.join("Cargo.toml"),
        r#"[package]
name = "external-helper"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        helper.join("src/lib.rs"),
        r#"mod live;

#[cfg(test)]
mod test_only;

pub fn decorate(value: &str) -> String {
    live::decorate(value)
}

pub fn unused_root(value: &str) -> String {
    format!("dead:{value}")
}
"#,
    );
    write(
        helper.join("src/live.rs"),
        r#"const LABEL: &str = include_str!("live.txt");

pub struct LiveRecord {
    pub value: String,
    pub dead_note: Option<String>,
}

pub fn decorate(value: &str) -> String {
    let record = LiveRecord {
        value: format!("{value}:{}", LABEL.trim()),
        dead_note: None,
    };
    record.value
}

pub fn unused_live(value: &str) -> String {
    format!("dead:{value}")
}
"#,
    );
    write(
        helper.join("src/test_only.rs"),
        r#"const TEST_LABEL: &str = include_str!("test.txt");

pub fn test_value() -> &'static str {
    TEST_LABEL
}
"#,
    );
    write(
        helper.join("src/orphan.rs"),
        r#"const ORPHAN_LABEL: &str = include_str!("orphan.txt");

pub fn orphan_value() -> &'static str {
    ORPHAN_LABEL
}
"#,
    );
    write(helper.join("src/live.txt"), "live");
    write(helper.join("src/test.txt"), "test");
    write(helper.join("src/orphan.txt"), "orphan");

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    assert!(output.join("support/external-helper/src/lib.rs").exists());
    assert!(output.join("support/external-helper/src/live.rs").exists());
    assert!(output.join("support/external-helper/src/live.txt").exists());
    let support_root = read(output.join("support/external-helper/src/lib.rs"));
    let support_live = read(output.join("support/external-helper/src/live.rs"));
    assert!(
        !support_root.contains("unused_root"),
        "direct dependency path usage should restrict support root items\n{support_root}"
    );
    assert!(
        !support_live.contains("unused_live"),
        "direct dependency path usage should restrict support child items\n{support_live}"
    );
    assert!(
        support_live.contains("pub struct LiveRecord"),
        "{support_live}"
    );
    assert!(support_live.contains("pub value: String"), "{support_live}");
    assert!(
        !support_live.contains("dead_note"),
        "restricted support structs should prune unused pure fields and their initializers\n{support_live}"
    );
    assert!(!output
        .join("support/external-helper/src/test_only.rs")
        .exists());
    assert!(!output.join("support/external-helper/src/test.txt").exists());
    assert!(!output
        .join("support/external-helper/src/orphan.rs")
        .exists());
    assert!(!output
        .join("support/external-helper/src/orphan.txt")
        .exists());

    fs::rename(&helper, helper.with_extension("moved")).unwrap();

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated support module closure slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
    );
}

#[test]
fn preserves_direct_required_support_struct_field_surface() {
    let workspace = temp_path("support-required-struct-surface-workspace");
    let output = temp_path("support-required-struct-surface-output");
    let target_dir = temp_path("support-required-struct-surface-target");
    let external = temp_path("support-required-struct-surface-external");
    let helper = external.join("external-helper");
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
external-helper = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path),
            manifest_path(&helper),
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected(value: &str) -> external_helper::LiveRecord {
    external_helper::make_record(value)
}
"#,
    );
    write(
        helper.join("Cargo.toml"),
        r#"[package]
name = "external-helper"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        helper.join("src/lib.rs"),
        r#"pub struct LiveRecord {
    pub value: String,
    pub dead_note: Option<String>,
}

pub fn make_record(value: &str) -> LiveRecord {
    LiveRecord {
        value: value.to_string(),
        dead_note: None,
    }
}

pub fn unused_record() -> LiveRecord {
    LiveRecord {
        value: "dead".to_string(),
        dead_note: Some("dead".to_string()),
    }
}
"#,
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let support = read(output.join("support/external-helper/src/lib.rs"));
    assert!(support.contains("pub struct LiveRecord"), "{support}");
    assert!(support.contains("pub value: String"), "{support}");
    assert!(
        support.contains("pub dead_note: Option<String>"),
        "direct required support API structs must preserve their public field surface\n{support}"
    );
    assert!(!support.contains("unused_record"), "{support}");

    fs::rename(&helper, helper.with_extension("moved")).unwrap();

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated required support struct slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsupport/external-helper/src/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        support,
    );
}

#[test]
fn preserves_replace_tables_with_resolved_paths() {
    let workspace = temp_path("replace-table-workspace");
    let output = temp_path("replace-table-output");
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        workspace.join("Cargo.toml"),
        r#"[workspace]
members = ["app"]
resolver = "2"

[replace]
"replace-helper:0.1.0" = { path = "replace-helper" }
"#,
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected() -> i32 {
    7
}
"#,
    );
    write(
        workspace.join("replace-helper/Cargo.toml"),
        r#"[package]
name = "replace-helper"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        workspace.join("replace-helper/src/lib.rs"),
        r#"pub fn value() -> i32 {
    7
}
"#,
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let root_manifest = read(output.join("Cargo.toml"));
    let support_manifest = read(output.join("support/replace-helper/Cargo.toml"));
    assert!(root_manifest.contains("replace-helper:0.1.0"));
    assert!(root_manifest.contains("path = \"support/replace-helper\""));
    assert!(!root_manifest.contains("path = \"/"));
    assert!(!root_manifest.contains("path = \"replace-helper\""));
    assert!(support_manifest.contains("name = \"replace-helper\""));
}

#[test]
fn retains_external_extension_trait_imports_for_method_resolution() {
    let workspace = temp_path("extension-trait-workspace");
    let output = temp_path("extension-trait-output");
    let target_dir = temp_path("extension-trait-target");
    write_extension_trait_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let source = read(output.join("extension_trait_like/src/lib.rs"));
    assert!(source.contains("AsyncReadExt"));
    assert!(!source.contains("#[opensourced]"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated extension trait slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        source,
    );
}

#[test]
fn prunes_unused_external_extension_trait_imports_after_dead_items_are_removed() {
    let workspace = temp_path("unused-extension-trait-workspace");
    let output = temp_path("unused-extension-trait-output");
    let target_dir = temp_path("unused-extension-trait-target");
    write_unused_extension_trait_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let source = read(output.join("unused_extension_trait_like/src/lib.rs"));
    assert!(!source.contains("AsyncReadExt"));
    assert!(!source.contains("AsyncWriteExt"));
    assert!(!source.contains("tokio::io"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated unused extension trait slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        source,
    );
}

#[test]
fn retains_external_trait_imports_without_ext_suffix_for_method_resolution() {
    let workspace = temp_path("external-trait-workspace");
    let output = temp_path("external-trait-output");
    let target_dir = temp_path("external-trait-target");
    write_external_trait_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let source = read(output.join("external_trait_like/src/lib.rs"));
    assert!(source.contains("base64::Engine"));
    assert!(!source.contains("#[opensourced]"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated external trait slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        source,
    );
}

#[test]
fn prunes_unused_serde_derive_imports_after_dead_items_are_removed() {
    let workspace = temp_path("serde-derive-import-workspace");
    let output = temp_path("serde-derive-import-output");
    let target_dir = temp_path("serde-derive-import-target");
    write_unused_serde_derive_import_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let source = read(output.join("serde_derive_import_like/src/lib.rs"));
    assert!(source.contains("pub fn selected"));
    assert!(source.contains("serde::Serialize"));
    assert!(!source.contains("Deserialize"));
    assert!(!source.contains("DeadWire"));
    assert!(!source.contains("#[opensourced]"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated serde derive import slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        source,
    );
}

#[test]
fn prunes_unused_serde_deserialize_import_when_owned_bound_is_enough() {
    let workspace = temp_path("serde-owned-bound-import-workspace");
    let output = temp_path("serde-owned-bound-import-output");
    let target_dir = temp_path("serde-owned-bound-import-target");
    let opensourced_path = repo_root().join("crates/opensourced");

    write(
        workspace.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "serde_owned_bound_import_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1"
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        workspace.join("src/lib.rs"),
        r#"use opensourced::opensourced;
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::Value;

fn deserialize_json_value<T>(value: &Value) -> Result<T, serde_json::Error>
where
    T: DeserializeOwned,
{
    T::deserialize(value)
}

#[opensourced]
pub fn selected(value: &Value) -> Result<String, serde_json::Error> {
    deserialize_json_value(value)
}
"#,
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let source = read(output.join("serde_owned_bound_import_like/src/lib.rs"));
    assert!(source.contains("DeserializeOwned"), "{source}");
    assert!(!source.contains("use serde::{Deserialize"), "{source}");
    assert!(!source.contains("Deserialize,"), "{source}");
    assert!(!source.contains("#[opensourced]"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated serde owned bound import slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        source,
    );
}

#[test]
fn prunes_module_scoped_imports_used_only_by_dead_items() {
    let workspace = temp_path("module-scoped-unused-import-workspace");
    let output = temp_path("module-scoped-unused-import-output");
    let target_dir = temp_path("module-scoped-unused-import-target");
    write_module_scoped_unused_import_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let source = read(output.join("module_scoped_unused_imports/src/lib.rs"));
    assert!(source.contains("pub trait Handler"));
    assert!(source.contains("pub struct LiveDto"));
    assert!(source.contains("pub struct ClientConfig"));
    assert_eq!(
        source.matches("use serde::Serialize").count(),
        1,
        "{source}"
    );
    assert!(!source.contains("use crate::api::Handler"), "{source}");
    assert!(!source.contains("DeadDto"), "{source}");
    assert!(!source.contains("pub fn dead"), "{source}");
    assert!(!source.contains("#[opensourced]"), "{source}");

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated module-scoped import slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        source,
    );
}

#[test]
fn retains_local_proc_macro_derive_and_helper_attr_dependencies() {
    let workspace = temp_path("proc-macro-helper-workspace");
    let output = temp_path("proc-macro-helper-output");
    let target_dir = temp_path("proc-macro-helper-target");
    write_local_proc_macro_helper_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let app = read(output.join("app/src/lib.rs"));
    let root_manifest = read(output.join("Cargo.toml"));
    let app_manifest = read(output.join("app/Cargo.toml"));
    let derive_manifest = read(output.join("derive-support/Cargo.toml"));
    let macro_source = read(output.join("derive-support/src/lib.rs"));
    assert!(app.contains("derive_support::UseHelper"));
    assert!(app.contains("default_token"));
    assert!(!app.contains("unused_helper"));
    assert!(!app.contains("DeadWire"));
    assert!(app_manifest.contains("[dependencies.derive-support]"));
    assert!(!root_manifest.contains("\"macro-support\""));
    assert!(!root_manifest.contains("\"unused-proc-support\""));
    assert!(root_manifest.contains("support/macro-support"));
    assert!(!root_manifest.contains("support/unused-proc-support"));
    assert!(derive_manifest.contains("[dependencies.macro-support]"));
    assert!(derive_manifest.contains("support/macro-support"));
    assert!(!derive_manifest.contains("unused-proc-support"));
    assert!(macro_source.contains("proc_macro_derive(UseHelper"));
    assert!(macro_source.contains("generated_method_name"));
    assert!(macro_source.contains("macro_support"));
    assert!(!macro_source.contains("proc_macro_derive(UnusedDerive"));
    assert!(!macro_source.contains("proc_macro_attribute"));
    assert!(!macro_source.contains("pub fn unused_macro"));
    assert!(!macro_source.contains("dead_macro_helper"));
    assert!(output.join("support/macro-support/src/lib.rs").exists());
    assert!(output.join("support/macro-support/src/live.rs").exists());
    assert!(!output.join("support/macro-support/src/orphan.rs").exists());
    assert!(!output.join("support/macro-support/examples").exists());
    assert!(!output.join("support/macro-support/tests").exists());
    assert!(!output.join("support/macro-support/benches").exists());
    assert!(!output.join("support/unused-proc-support").exists());
    assert!(!app.contains("#[opensourced]"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated proc-macro helper slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nCargo.toml:\n{}\napp/Cargo.toml:\n{}\nderive-support/Cargo.toml:\n{}\napp/src/lib.rs:\n{}\nderive-support/src/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        root_manifest,
        app_manifest,
        derive_manifest,
        app,
        macro_source,
    );
}

#[test]
fn retains_serde_trait_import_for_associated_deserialize_call() {
    let workspace = temp_path("serde-trait-associated-workspace");
    let output = temp_path("serde-trait-associated-output");
    let target_dir = temp_path("serde-trait-associated-target");
    write_serde_trait_associated_call_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let source = read(output.join("serde_trait_associated_like/src/lib.rs"));
    assert!(source.contains("use serde::Deserialize"));
    assert!(source.contains("Option::<serde_json::Value>::deserialize"));
    assert!(!source.contains("#[opensourced]"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated serde trait associated slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        source,
    );
}

#[test]
fn retains_digest_trait_import_for_associated_new_call() {
    let workspace = temp_path("digest-trait-associated-workspace");
    let output = temp_path("digest-trait-associated-output");
    let target_dir = temp_path("digest-trait-associated-target");
    write_digest_trait_associated_call_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let source = read(output.join("digest_trait_associated_like/src/lib.rs"));
    assert!(source.contains("sha1::{Digest, Sha1}"));
    assert!(source.contains("Sha1::new"));
    assert!(!source.contains("#[opensourced]"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated digest trait associated slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        source,
    );
}

#[test]
fn prunes_digest_trait_import_when_hash_usage_is_dead() {
    let workspace = temp_path("digest-trait-dead-hash-workspace");
    let output = temp_path("digest-trait-dead-hash-output");
    let target_dir = temp_path("digest-trait-dead-hash-target");
    write_digest_trait_dead_hash_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let source = read(output.join("digest_trait_dead_hash_like/src/lib.rs"));
    assert!(source.contains("pub fn selected"));
    assert!(source.contains("Mutex::new"));
    assert!(!source.contains("sha1"));
    assert!(!source.contains("Digest"));
    assert!(!source.contains("Sha1"));
    assert!(!source.contains("#[opensourced]"));

    let manifest = read(output.join("digest_trait_dead_hash_like/Cargo.toml"));
    assert!(!manifest.contains("sha1"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .env("RUSTFLAGS", "-Dwarnings")
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated dead digest trait import slice did not compile cleanly\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        source,
    );
}

#[test]
fn prunes_unused_private_external_type_imports_after_dead_items_are_removed() {
    let workspace = temp_path("external-type-import-workspace");
    let output = temp_path("external-type-import-output");
    let target_dir = temp_path("external-type-import-target");
    write_unused_external_type_import_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let source = read(output.join("external_type_import_like/src/lib.rs"));
    assert!(source.contains("pub fn selected"));
    assert!(!source.contains("serde_json"));
    assert!(!source.contains("Map"));
    assert!(!source.contains("Value"));
    assert!(!source.contains("#[opensourced]"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated external type import slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        source,
    );
}

#[test]
fn prunes_unused_private_local_imports_even_when_item_is_reachable_elsewhere() {
    let workspace = temp_path("local-import-workspace");
    let output = temp_path("local-import-output");
    let target_dir = temp_path("local-import-target");
    write_unused_local_import_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let feature = read(output.join("local_import_like/src/feature.rs"));
    let other = read(output.join("local_import_like/src/other.rs"));
    assert!(feature.contains("FeatureValue"));
    assert!(!feature.contains("OtherValue"));
    assert!(other.contains("OtherValue"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated local import slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nfeature.rs:\n{}\nother.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        feature,
        other,
    );
}

#[test]
fn retains_parent_imports_used_by_child_super_glob_and_prunes_unused_child_glob() {
    let workspace = temp_path("super-glob-workspace");
    let output = temp_path("super-glob-output");
    let target_dir = temp_path("super-glob-target");
    write_super_glob_parent_import_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let parent = read(output.join("super_glob_like/src/parent/mod.rs"));
    let child = read(output.join("super_glob_like/src/parent/child.rs"));
    assert!(parent.contains("PendingApproval"));
    assert!(parent.contains("PendingApprovalSeed"));
    assert!(!parent.contains("self::child"));
    assert!(child.contains("use super::*"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated super glob slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nparent/mod.rs:\n{}\nparent/child.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        parent,
        child,
    );
}

#[test]
fn prunes_parent_imports_shadowed_by_child_super_glob_local_binding() {
    let workspace = temp_path("super-glob-shadowed-parent-workspace");
    let output = temp_path("super-glob-shadowed-parent-output");
    let target_dir = temp_path("super-glob-shadowed-parent-target");
    write_super_glob_shadowed_parent_import_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let parent = read(output.join("super_glob_shadowed_parent_like/src/parent/mod.rs"));
    let child = read(output.join("super_glob_shadowed_parent_like/src/parent/child.rs"));
    let other = read(output.join("super_glob_shadowed_parent_like/src/other.rs"));
    assert!(parent.contains("FeatureValue"));
    assert!(
        !parent.contains("shared::{FeatureValue, helper}")
            && !parent.contains("shared::{helper, FeatureValue}"),
        "child local binding named like a parent import must not retain the parent import leaf\n{parent}",
    );
    assert!(child.contains("use super::*"));
    assert!(child.contains("let helper = 1"));
    assert!(
        other.contains("helper()"),
        "the helper target should remain where it is actually used\n{other}",
    );

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated super glob shadowed parent slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nparent/mod.rs:\n{}\nparent/child.rs:\n{}\nother.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        parent,
        child,
        other,
    );
}

#[test]
fn prunes_unused_private_struct_fields_and_their_imports() {
    let workspace = temp_path("private-field-workspace");
    let output = temp_path("private-field-output");
    let target_dir = temp_path("private-field-target");
    write_unused_private_struct_field_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let source = read(output.join("private_field_like/src/lib.rs"));
    assert!(source.contains("raw_params"));
    assert!(source.contains("Serialize"));
    assert!(source.contains("Deserialize"));
    assert!(!source.contains("request_id"));
    assert!(!source.contains("serde_json"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated private field slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        source,
    );
}

#[test]
fn retains_std_trait_imports_for_method_resolution() {
    let workspace = temp_path("std-trait-import-workspace");
    let output = temp_path("std-trait-import-output");
    let target_dir = temp_path("std-trait-import-target");
    write_std_trait_import_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let source = read(output.join("std_trait_import_like/src/lib.rs"));
    assert!(source.contains("Hash"));
    assert!(source.contains("Hasher"));
    assert!(source.contains("Write"));
    assert!(!source.contains("AtomicI64"));
    assert!(!source.contains("Ordering"));
    assert!(!source.contains("#[opensourced]"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated std trait import slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        source,
    );
}

#[test]
fn retains_dyn_trait_bounds_used_by_reachable_struct_fields() {
    let workspace = temp_path("dyn-trait-field-workspace");
    let output = temp_path("dyn-trait-field-output");
    let target_dir = temp_path("dyn-trait-field-target");
    write_dyn_trait_field_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let source = read(output.join("dyn_trait_field_like/src/lib.rs"));
    assert!(source.contains("trait PlatformService"));
    assert!(source.contains("Arc<dyn PlatformService>"));
    assert!(!source.contains("DeadService"));
    assert!(!source.contains("#[opensourced]"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated dyn trait field slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        source,
    );
}

#[test]
fn retains_trait_impls_for_generic_field_type_bounds() {
    let workspace = temp_path("generic-field-bound-workspace");
    let output = temp_path("generic-field-bound-output");
    let target_dir = temp_path("generic-field-bound-target");
    write_generic_field_bound_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let source = read(output.join("generic_field_bound_like/src/lib.rs"));
    assert!(source.contains("trait ExternalHandler"));
    assert!(source.contains("impl ExternalHandler for ClientHandler"));
    assert!(source.contains("ExternalHandle<ClientHandler>"));
    assert!(!source.contains("DeadHandler"));
    assert!(!source.contains("#[opensourced]"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated generic field bound slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        source,
    );
}

#[test]
fn resolves_methods_after_if_let_option_unwrap() {
    let workspace = temp_path("if-let-option-workspace");
    let output = temp_path("if-let-option-output");
    let target_dir = temp_path("if-let-option-target");
    write_if_let_option_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let source = read(output.join("if_let_option_like/src/lib.rs"));
    assert!(source.contains("fn emit"));
    assert!(source.contains("shared_client"));
    assert!(!source.contains("unused_emit"));
    assert!(!source.contains("#[opensourced]"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated if-let option slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        source,
    );
}

#[test]
fn resolves_type_paths_through_visible_glob_reexports() {
    let workspace = temp_path("glob-type-reexport-workspace");
    let output = temp_path("glob-type-reexport-output");
    let target_dir = temp_path("glob-type-reexport-target");
    write_glob_type_reexport_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let models = read(output.join("glob_type_reexport_like/src/types/models.rs"));
    assert!(models.contains("pub enum AppAskForApproval"));
    assert!(models.contains("crate::types::AppAskForApproval"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated glob type reexport slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\ntypes/models.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        models,
    );
}

#[test]
fn prunes_unused_pub_crate_reexports_when_target_item_is_only_used_in_source_module() {
    let workspace = temp_path("pub-crate-reexport-prune-workspace");
    let output = temp_path("pub-crate-reexport-prune-output");
    let target_dir = temp_path("pub-crate-reexport-prune-target");
    let opensourced_path = repo_root().join("crates/opensourced");

    write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    );
    write(
        workspace.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        workspace.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

pub mod snapshot;
pub(crate) use snapshot::QueuedFollowUpDraft;
pub use snapshot::ThreadSnapshot;

#[opensourced]
pub fn selected() -> ThreadSnapshot {
    ThreadSnapshot {
        queued_follow_up_drafts: Vec::new(),
    }
}
"#,
    );
    write(
        workspace.join("app/src/snapshot.rs"),
        r#"pub struct ThreadSnapshot {
    pub(crate) queued_follow_up_drafts: Vec<QueuedFollowUpDraft>,
}

pub(crate) struct QueuedFollowUpDraft;
"#,
    );

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let lib = read(output.join("app/src/lib.rs"));
    let snapshot = read(output.join("app/src/snapshot.rs"));
    assert!(lib.contains("pub use snapshot::ThreadSnapshot"), "{lib}");
    assert!(
        !lib.contains("QueuedFollowUpDraft"),
        "restricted reexport should be pruned when only the source module uses the item\n{lib}"
    );
    assert!(snapshot.contains("QueuedFollowUpDraft"), "{snapshot}");

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .env("RUSTFLAGS", "-D warnings")
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated pub(crate) reexport slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nlib.rs:\n{}\nsnapshot.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        lib,
        snapshot,
    );
}

#[test]
fn retains_parent_public_glob_reexport_for_child_super_imports() {
    let workspace = temp_path("parent-glob-super-import-workspace");
    let output = temp_path("parent-glob-super-import-output");
    let target_dir = temp_path("parent-glob-super-import-target");
    write_parent_glob_super_import_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let types_mod = read(output.join("parent_glob_super_import_like/src/types/mod.rs"));
    let server_requests =
        read(output.join("parent_glob_super_import_like/src/types/server_requests.rs"));
    assert!(
        types_mod.contains("pub use models::*;"),
        "parent module must retain the public glob reexport that backs child use super::Name\n{}",
        types_mod
    );
    assert!(server_requests.contains("use super::AppAskForApproval;"));
    assert!(!types_mod.contains("dead"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated parent glob super-import slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\ntypes/mod.rs:\n{}\ntypes/server_requests.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        types_mod,
        server_requests,
    );
}

#[test]
fn removes_test_and_runtime_benchmark_cfg_modules() {
    let workspace = temp_path("cfg-benchmark-workspace");
    let output = temp_path("cfg-benchmark-output");
    let target_dir = temp_path("cfg-benchmark-target");
    write_cfg_benchmark_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let source = read(output.join("app/src/lib.rs"));
    assert!(source.contains("pub fn selected"));
    assert!(source.contains("mod live"));
    assert!(!source.contains("runtime-benchmarks"));
    assert!(!source.contains("mod benchmarks"));
    assert!(!output.join("app/src/benchmarks.rs").exists());

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated cfg benchmark slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\napp/src/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        source,
    );
}

#[test]
fn resolves_associated_methods_across_crate_root_reexports() {
    let workspace = temp_path("reexport-method-workspace");
    let output = temp_path("reexport-method-output");
    let target_dir = temp_path("reexport-method-target");
    write_reexport_method_fixture(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let reachable = report
        .reachable
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    assert!(
        reachable
            .iter()
            .any(|callable| callable == "app::inner::Service::selected"),
        "missing selected method; got {reachable:?}",
    );
    assert!(
        reachable
            .iter()
            .any(|callable| callable == "app::Service::helper"),
        "missing helper method through crate-root reexport; got {reachable:?}",
    );

    let lib = read(output.join("app/src/lib.rs"));
    let inner = read(output.join("app/src/inner.rs"));
    let impls = read(output.join("app/src/impls.rs"));
    assert!(lib.contains("mod impls"));
    assert!(inner.contains("pub fn selected"));
    assert!(impls.contains("pub fn helper"));
    assert!(!impls.contains("dead_helper"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated reexport method slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nimpls.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        impls,
    );
}

#[test]
fn preserves_macro_included_generated_dependency_sources() {
    let workspace = temp_path("included-generated-workspace");
    let output = temp_path("included-generated-output");
    let target_dir = temp_path("included-generated-target");
    write_included_generated_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let proto_manifest = read(output.join("proto_dep/Cargo.toml"));
    let proto = read(output.join("proto_dep/src/lib.rs"));
    assert!(proto_manifest.contains("itoa"));
    assert!(proto.contains("include_proto"));
    assert!(proto.contains("pub mod generated"));
    assert!(output.join("proto_dep/src/prost/generated.rs").exists());
    assert!(output.join("proto_dep/src/prost/unused.rs").exists());

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated include-backed dependency slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nproto_dep/src/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        proto,
    );
}

#[test]
fn keeps_trait_impls_distinct_by_trait_input_type() {
    let workspace = temp_path("trait-input-workspace");
    let output = temp_path("trait-input-output");
    let target_dir = temp_path("trait-input-target");
    write_trait_input_impl_fixture(&workspace);

    generate(GenerateOptions {
        workspace_root: workspace,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let source = read(output.join("trait_input_like/src/lib.rs"));
    assert!(source.contains("impl From<LiveEvent> for ApiEvent"));
    assert!(!source.contains("impl From<DeadEvent> for ApiEvent"));
    assert!(!source.contains("DEAD_EVENT_KIND"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated trait input impl slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nsrc/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        source,
    );
}

fn write_single_binary_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "rtk_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}

[[bin]]
name = "tool"
path = "src/bin/tool/main.rs"
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("src/main.rs"),
        r#"use opensourced::opensourced;

fn main() {
    noisy_entrypoint();
}

#[opensourced]
fn selected(value: &str) -> String {
    helper(value)
}

fn helper(value: &str) -> String {
    format!("selected:{value}")
}

fn noisy_entrypoint() {
    let _ = dead();
}

fn dead() -> String {
    "dead".to_string()
}
"#,
    );
}

fn write_bin_with_lib_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        r#"[workspace]
members = ["app"]
resolver = "2"
"#,
    );
    write(
        root.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("app/src/lib.rs"),
        r#"pub fn dead_lib() -> i32 {
    1
}
"#,
    );
    write(
        root.join("app/src/bin/tool/main.rs"),
        r#"mod command;

fn dead_bin() -> String {
    "dead".to_string()
}
"#,
    );
    write(
        root.join("app/src/bin/tool/command.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected(value: &str) -> String {
    helper(value)
}

fn helper(value: &str) -> String {
    format!("bin:{value}")
}

fn dead_command() -> String {
    "dead".to_string()
}
"#,
    );
}

fn write_example_target_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        r#"[workspace]
members = ["app", "lib_support", "support"]
resolver = "2"
"#,
    );
    write(
        root.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[[example]]
name = "demo"
path = "examples/demo.rs"
required-features = ["demo-mode"]

[features]
default = ["demo-mode"]
demo-mode = []

[dependencies]
lib_support = {{ path = "../lib_support" }}

[dev-dependencies]
opensourced = {{ path = "{}" }}
support = {{ path = "../support" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("app/src/lib.rs"),
        r#"pub fn library_label(value: &str) -> String {
    lib_support::label(value)
}

pub fn dead_lib() -> i32 {
    1
}
"#,
    );
    write(
        root.join("app/examples/demo.rs"),
        r#"use opensourced::opensourced;

fn main() {
    println!("{}", selected("runtime"));
}

#[opensourced]
pub fn selected(value: &str) -> String {
    format!("{}:{}", app::library_label(value), support::format_value(value))
}

fn dead_example() -> String {
    "dead".to_string()
}
"#,
    );
    write(
        root.join("lib_support/Cargo.toml"),
        r#"[package]
name = "lib_support"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        root.join("lib_support/src/lib.rs"),
        r#"pub fn label(value: &str) -> String {
    format!("lib:{value}")
}

pub fn dead_label() -> String {
    "dead".to_string()
}
"#,
    );
    write(
        root.join("support/Cargo.toml"),
        r#"[package]
name = "support"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        root.join("support/src/lib.rs"),
        r#"pub fn format_value(value: &str) -> String {
    format!("example:{value}")
}

pub fn dead_support() -> String {
    "dead".to_string()
}
"#,
    );
}

fn write_test_target_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        r#"[workspace]
members = ["app", "lib_support", "platform_support", "support"]
resolver = "2"
"#,
    );
    write(
        root.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[[test]]
name = "behavior"
path = "tests/behavior.rs"

[dependencies]
lib_support = {{ path = "../lib_support" }}

[dev-dependencies]
opensourced = {{ path = "{}" }}
support = {{ path = "../support" }}

[target.'cfg(unix)'.dev-dependencies]
platform_support = {{ path = "../platform_support" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("app/src/lib.rs"),
        r#"pub fn library_label(value: &str) -> String {
    lib_support::label(value)
}

pub fn dead_lib() -> i32 {
    1
}
"#,
    );
    write(
        root.join("app/tests/behavior.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
#[test]
fn selected_behavior() {
    assert_eq!(app::library_label("runtime"), "lib:runtime");
    assert_eq!(support::format_value("runtime"), "test:runtime");
    #[cfg(unix)]
    assert_eq!(platform_support::format_value("runtime"), "platform:runtime");
}

fn dead_test() -> String {
    "dead".to_string()
}
"#,
    );
    write(
        root.join("lib_support/Cargo.toml"),
        r#"[package]
name = "lib_support"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        root.join("lib_support/src/lib.rs"),
        r#"pub fn label(value: &str) -> String {
    format!("lib:{value}")
}

pub fn dead_label() -> String {
    "dead".to_string()
}
"#,
    );
    write(
        root.join("platform_support/Cargo.toml"),
        r#"[package]
name = "platform_support"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        root.join("platform_support/src/lib.rs"),
        r#"pub fn format_value(value: &str) -> String {
    format!("platform:{value}")
}

pub fn dead_platform_support() -> String {
    "dead".to_string()
}
"#,
    );
    write(
        root.join("support/Cargo.toml"),
        r#"[package]
name = "support"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        root.join("support/src/lib.rs"),
        r#"pub fn format_value(value: &str) -> String {
    format!("test:{value}")
}

pub fn dead_support() -> String {
    "dead".to_string()
}
"#,
    );
}

fn write_bench_target_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        r#"[workspace]
members = ["app", "lib_support", "support"]
resolver = "2"
"#,
    );
    write(
        root.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[[bench]]
name = "throughput"
path = "benches/throughput.rs"

[dependencies]
lib_support = {{ path = "../lib_support" }}

[dev-dependencies]
opensourced = {{ path = "{}" }}
support = {{ path = "../support" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("app/src/lib.rs"),
        r#"pub fn library_label(value: &str) -> String {
    lib_support::label(value)
}

pub fn dead_lib() -> i32 {
    1
}
"#,
    );
    write(
        root.join("app/benches/throughput.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected_bench_value(value: &str) -> String {
    format!("{}:{}", app::library_label(value), support::format_value(value))
}

fn dead_bench() -> String {
    "dead".to_string()
}
"#,
    );
    write(
        root.join("lib_support/Cargo.toml"),
        r#"[package]
name = "lib_support"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        root.join("lib_support/src/lib.rs"),
        r#"pub fn label(value: &str) -> String {
    format!("lib:{value}")
}

pub fn dead_label() -> String {
    "dead".to_string()
}
"#,
    );
    write(
        root.join("support/Cargo.toml"),
        r#"[package]
name = "support"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        root.join("support/src/lib.rs"),
        r#"pub fn format_value(value: &str) -> String {
    format!("bench:{value}")
}

pub fn dead_support() -> String {
    "dead".to_string()
}
"#,
    );
}

fn write_path_attr_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "path_attr_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("src/lib.rs"),
        r#"#[path = "generated/custom.rs"]
mod custom;

fn dead_lib() -> i32 {
    1
}
"#,
    );
    write(
        root.join("src/generated/custom.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected(value: &str) -> String {
    helper(value)
}

fn helper(value: &str) -> String {
    format!("custom:{value}")
}

fn dead_custom() -> String {
    "dead".to_string()
}
"#,
    );
}

fn write_multi_target_marker_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "multi_target_marker_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}

[[bin]]
name = "tool"
path = "src/bin/tool.rs"
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn lib_selected() -> i32 {
    1
}
"#,
    );
    write(
        root.join("src/bin/tool.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn bin_selected() -> i32 {
    2
}
"#,
    );
}

fn write_lib_and_test_marker_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "lib_test_marker_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}

[[test]]
name = "behavior"
path = "tests/behavior.rs"
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn lib_selected() -> i32 {
    1
}
"#,
    );
    write(
        root.join("tests/behavior.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
#[test]
fn selected_behavior() {
    assert_eq!(lib_test_marker_like::lib_selected(), 1);
}
"#,
    );
}

fn write_multi_root_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "multi_root_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected_a(value: i32) -> i32 {
    helper_a(value)
}

#[opensourced]
pub fn selected_b(value: i32) -> i32 {
    helper_b(value)
}

fn helper_a(value: i32) -> i32 {
    value + 1
}

fn helper_b(value: i32) -> i32 {
    value * 2
}

mod hidden {
    use opensourced::opensourced;

    #[opensourced]
    pub fn selected_hidden(value: i32) -> i32 {
        helper_hidden(value)
    }

    fn helper_hidden(value: i32) -> i32 {
        value * value
    }

    pub fn dead_hidden(value: i32) -> i32 {
        value - 10
    }
}

pub fn dead_root(value: i32) -> i32 {
    dead_helper(value)
}

fn dead_helper(value: i32) -> i32 {
    value - 1
}
"#,
    );
}

fn write_item_root_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "item_root_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub struct Api {
    pub mode: Mode,
    pub handler: Box<dyn Handler>,
    state: PrivateState,
}

struct PrivateState {
    generation: usize,
}

pub enum Mode {
    Read,
    Write,
}

#[opensourced]
enum PrivateKind {
    Fast,
    Slow,
}

pub trait Handler {
    #[opensourced]
    fn handle(&self, mode: Mode) -> usize;
}

pub struct Dead;

pub fn dead_factory() -> Dead {
    Dead
}
"#,
    );
}

fn write_module_root_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "module_root_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub mod api {
    pub struct Request {
        pub mode: Mode,
    }

    pub enum Mode {
        Read,
        Write,
    }

    pub fn run(request: Request) -> usize {
        helper(request.mode)
    }

    fn helper(mode: Mode) -> usize {
        match mode {
            Mode::Read => 1,
            Mode::Write => 2,
        }
    }

    #[cfg(test)]
    mod tests {
        pub fn module_test_helper() -> usize {
            99
        }
    }
}

pub mod dead {
    pub fn unused() -> usize {
        0
    }
}
"#,
    );
}

fn write_child_binary_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "rtk_child_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("src/main.rs"),
        r#"mod exposed;
mod noisy;

use noisy::dead_entry;

fn main() {
    dead_entry();
}
"#,
    );
    write(
        root.join("src/exposed.rs"),
        r#"use opensourced::opensourced;

macro_rules! define_generated {
    ($name:ident) => {
        static $name: &str = "generated";
    };
}

define_generated!(GENERATED_LABEL);

#[opensourced]
pub fn selected() -> &'static str {
    GENERATED_LABEL
}
"#,
    );
    write(
        root.join("src/noisy.rs"),
        r#"pub fn dead_entry() {
    let _ = dead_value();
}

fn dead_value() -> &'static str {
    "dead"
}
"#,
    );
}

fn write_automod_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "automod_like"
version = "0.1.0"
edition = "2021"

[dependencies]
automod = "1"
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("src/main.rs"),
        r#"mod plugins;

fn main() {}
"#,
    );
    write(
        root.join("src/plugins/mod.rs"),
        r#"automod::dir!(pub "src/plugins");
"#,
    );
    write(
        root.join("src/plugins/public_slice.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected() -> String {
    helper()
}

fn helper() -> String {
    "live".to_string()
}
"#,
    );
    write(
        root.join("src/plugins/dead_slice.rs"),
        r#"pub fn dead() -> String {
    "dead".to_string()
}
"#,
    );
}

fn write_nested_file_module_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "nested_file_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("src/main.rs"),
        r#"mod feature;

fn main() {}
"#,
    );
    write(
        root.join("src/feature.rs"),
        r#"use opensourced::opensourced;

mod nested;

#[opensourced]
pub fn selected() -> String {
    nested::helper()
}
"#,
    );
    write(
        root.join("src/feature/nested.rs"),
        r#"pub fn helper() -> String {
    "nested".to_string()
}
"#,
    );
}

fn write_explicit_nested_lib_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "explicit_lib_like"
version = "0.1.0"
edition = "2021"

[lib]
path = "src/dump/lib.rs"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("src/dump/lib.rs"),
        r#"use opensourced::opensourced;

pub mod client;

#[opensourced]
pub fn selected() -> String {
    client::helper()
}
"#,
    );
    write(
        root.join("src/dump/client.rs"),
        r#"pub fn helper() -> String {
    "client".to_string()
}
"#,
    );
}

fn write_raw_identifier_module_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "raw_module_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("src/lib.rs"),
        r#"use opensourced::opensourced;

mod r#type;

#[opensourced]
pub fn selected() -> String {
    r#type::helper()
}
"#,
    );
    write(
        root.join("src/type.rs"),
        r#"pub fn helper() -> String {
    "raw".to_string()
}

pub fn dead() -> String {
    "dead".to_string()
}
"#,
    );
}

fn write_method_root_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "method_root_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("src/lib.rs"),
        r#"use opensourced::opensourced;

pub struct Processor(String);

impl Processor {
    #[opensourced]
    pub fn selected(&self) -> String {
        helper(self.0.as_str())
    }

    pub fn dead_method(&self) -> String {
        dead_function()
    }
}

fn helper(value: &str) -> String {
    format!("live:{value}")
}

fn dead_function() -> String {
    "dead".to_string()
}
"#,
    );
}

fn write_derive_field_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[workspace]
members = ["app", "derive_helper"]
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
derive_helper = { path = "../derive_helper" }
opensourced.workspace = true
"#,
    );
    write(
        root.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[derive(Debug, Default)]
pub struct Envelope {
    pub id: derive_helper::Guid,
}

impl Envelope {
    #[opensourced]
    pub fn new(id: derive_helper::Guid) -> Self {
        Self { id }
    }
}
"#,
    );
    write(
        root.join("derive_helper/Cargo.toml"),
        r#"[package]
name = "derive_helper"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        root.join("derive_helper/src/lib.rs"),
        r#"mod formatting;

pub struct Guid(String);

impl Guid {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn dead_guid_method(&self) -> &str {
        "dead"
    }
}

impl Default for Guid {
    fn default() -> Self {
        Self(String::new())
    }
}
"#,
    );
    write(
        root.join("derive_helper/src/formatting.rs"),
        r#"use crate::Guid;

impl std::fmt::Debug for Guid {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
"#,
    );
}

fn write_serde_default_field_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "serde_default_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1"
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("src/lib.rs"),
        r#"use opensourced::opensourced;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct HomeSelection {
    selected_project_id: Option<String>,
}

impl Default for HomeSelection {
    fn default() -> Self {
        Self {
            selected_project_id: None,
        }
    }
}

#[derive(Deserialize)]
pub struct PersistedPreferences {
    #[serde(default)]
    home_selection: HomeSelection,
}

#[opensourced]
pub fn selected(input: &str) -> bool {
    serde_json::from_str::<PersistedPreferences>(input).is_ok()
}
"#,
    );
}

fn write_deref_method_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "deref_method_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("src/lib.rs"),
        r#"use opensourced::opensourced;

pub struct Slug(String);

impl Slug {
    #[opensourced]
    pub fn is_short(&self) -> bool {
        self.len() < 12 && self.bytes().all(|byte| byte != b',')
    }

    fn as_str(&self) -> &str {
        self.0.as_str()
    }

    fn dead(&self) -> bool {
        false
    }
}

impl std::ops::Deref for Slug {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_str()
    }
}
"#,
    );
}

fn write_trait_return_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[workspace]
members = ["app", "helper_dep"]
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
helper_dep = { path = "../helper_dep" }
opensourced.workspace = true
"#,
    );
    write(
        root.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected(value: &str) -> String {
    let copied = helper_dep::count_copy(value).unwrap();
    format!("{copied}:{}", helper_dep::make_fit().as_some())
}
"#,
    );
    write(
        root.join("helper_dep/Cargo.toml"),
        r#"[package]
name = "helper_dep"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        root.join("helper_dep/src/lib.rs"),
        r#"use std::io::{self, Write};

#[derive(Default)]
pub struct ByteCountWriter(usize);

impl ByteCountWriter {
    pub fn count(self) -> usize {
        self.0
    }

    pub fn dead_method(self) -> usize {
        999
    }
}

impl Write for ByteCountWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0 += buf.len();
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub enum Fit {
    All,
    None,
}

impl Fit {
    pub fn as_some(&self) -> bool {
        matches!(self, Self::All)
    }

    pub fn dead_method(&self) -> bool {
        false
    }
}

pub fn count_copy(value: &str) -> io::Result<usize> {
    let mut reader = value.as_bytes();
    let mut writer = ByteCountWriter::default();
    io::copy(&mut reader, &mut writer)?;
    Ok(writer.count())
}

pub fn make_fit() -> Fit {
    Fit::All
}

pub fn dead_function() -> usize {
    42
}
"#,
    );
}

fn write_format_display_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[workspace]
members = ["app", "format_helper"]
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
format_helper = { path = "../format_helper" }
opensourced.workspace = true
"#,
    );
    write(
        root.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected(value: &str) -> String {
    format_helper::render_name(value)
}
"#,
    );
    write(
        root.join("format_helper/Cargo.toml"),
        r#"[package]
name = "format_helper"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        root.join("format_helper/src/lib.rs"),
        r#"use std::fmt;

pub struct Name(String);

impl Name {
    pub fn new(value: String) -> Self {
        Self(value)
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for Name {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

pub fn class_prefix(value: &str) -> Name {
    Name::new(format!("prefix_{value}"))
}

pub fn render_name(value: &str) -> String {
    format!("{}_new", class_prefix(value))
}

pub fn dead_function(value: &str) -> String {
    format!("dead_{value}")
}
"#,
    );
}

fn write_to_string_display_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[workspace]
members = ["app", "display_helper"]
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
display_helper = { path = "../display_helper" }
opensourced.workspace = true
"#,
    );
    write(
        root.join("app/src/lib.rs"),
        r#"use display_helper::CacheKey;
use opensourced::opensourced;

#[opensourced]
pub fn selected(key: CacheKey) -> String {
    key.to_string()
}
"#,
    );
    write(
        root.join("display_helper/Cargo.toml"),
        r#"[package]
name = "display_helper"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        root.join("display_helper/src/lib.rs"),
        r#"use std::fmt;

pub struct CacheKey {
    pub message_id: String,
    pub revision_token: String,
}

impl fmt::Display for CacheKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}", self.message_id, self.revision_token)
    }
}

pub fn dead_function() -> &'static str {
    "dead"
}
"#,
    );
}

fn write_generic_inference_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[workspace]
members = ["app", "model_dep", "primitive_dep"]
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
model_dep = { path = "../model_dep" }
opensourced.workspace = true
"#,
    );
    write(
        root.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected(source: &str) -> String {
    model_dep::scan(source)
}
"#,
    );
    write(
        root.join("model_dep/Cargo.toml"),
        r#"[package]
name = "model_dep"
version = "0.1.0"
edition = "2021"

[dependencies]
primitive_dep = { path = "../primitive_dep" }
"#,
    );
    write(
        root.join("model_dep/src/lib.rs"),
        r#"pub type Primitive = primitive_dep::Primitive;

pub struct Size(usize);

impl Size {
    pub fn as_usize(self) -> usize {
        self.0
    }
}

pub struct Layout {
    pub size: Size,
}

impl Layout {
    pub fn new(size: usize) -> Self {
        Self { size: Size(size) }
    }
}

pub trait CLayout {
    fn c_layout(&self) -> Layout;
}

pub enum Type {
    Primitive(Primitive),
    Named { inner: Box<Type> },
    Empty,
}

impl CLayout for Primitive {
    fn c_layout(&self) -> Layout {
        Layout::new(self.weight())
    }
}

impl CLayout for Type {
    fn c_layout(&self) -> Layout {
        match self {
            Self::Primitive(primitive) => primitive.c_layout(),
            Self::Named { inner } => {
                let layout = inner.c_layout();
                Layout::new(layout.size.as_usize() + 1)
            }
            Self::Empty => Layout::new(0),
        }
    }
}

#[derive(Default)]
pub struct Registry {
    values: Vec<usize>,
}

impl Registry {
    pub fn register(&mut self, ty: Type) {
        let layout = ty.c_layout();
        self.values.push(layout.size.as_usize());
    }

    pub fn drain(self) -> impl Iterator<Item = usize> {
        self.values.into_iter()
    }
}

pub struct Scanner {
    registry: Registry,
}

impl Scanner {
    pub fn new() -> Self {
        Self {
            registry: Registry::default(),
        }
    }

    pub fn scan(mut self, source: &str) -> String {
        if let Some(primitive) = parse_primitive(source) {
            self.registry.register(Type::Primitive(primitive));
        }

        self.registry
            .drain()
            .fold(Report::new(), |report, size| report.with_part(size))
            .finish()
    }
}

pub struct Report {
    parts: Vec<usize>,
}

impl Report {
    pub fn new() -> Self {
        Self { parts: Vec::new() }
    }

    pub fn with_part(mut self, size: usize) -> Self {
        self.parts.push(size);
        self
    }

    pub fn finish(self) -> String {
        format!("report:{}", self.label())
    }

    pub fn label(&self) -> String {
        self.parts
            .iter()
            .map(|part| part.to_string())
            .collect::<Vec<_>>()
            .join(",")
    }

    pub fn dead_report_method(self) -> Self {
        self
    }
}

fn parse_primitive(source: &str) -> Option<Primitive> {
    source.parse().ok()
}

pub fn scan(source: &str) -> String {
    Scanner::new().scan(source)
}

pub fn dead_model_function() -> String {
    "dead".to_string()
}
"#,
    );
    write(
        root.join("primitive_dep/Cargo.toml"),
        r#"[package]
name = "primitive_dep"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        root.join("primitive_dep/src/lib.rs"),
        r#"use std::str::FromStr;

#[derive(Clone, Copy)]
pub enum Primitive {
    I32,
    U8,
}

impl Primitive {
    pub fn weight(&self) -> usize {
        match self {
            Self::I32 => 4,
            Self::U8 => 1,
        }
    }

    pub fn dead_weight(&self) -> usize {
        100
    }
}

impl FromStr for Primitive {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "i32" => Ok(Self::I32),
            "u8" => Ok(Self::U8),
            _ => Err(()),
        }
    }
}

pub fn dead_primitive_function() -> usize {
    9
}
"#,
    );
}

fn write_manifest_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[workspace]
members = ["app", "dead_dep", "live_dep", "platform_dep"]
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
build = "build.rs"

[features]
default = ["dep:dead_dep", "dep:live_dep"]
ffi = ["dep:live_dep", "dead_dep?/extra", "dev_only/std", "custom-flag"]
custom-flag = []

[dependencies]
dead_dep = { path = "../dead_dep", optional = true }
live_dep = { path = "../live_dep", optional = true }
opensourced.workspace = true

[target.'cfg(unix)'.dependencies]
platform_dep = { path = "../platform_dep" }

[dev-dependencies]
dev_only = { path = "../dead_dep" }
"#,
    );
    write(
        root.join("app/build.rs"),
        r#"fn main() {
    let _required = std::fs::read_to_string("assets/required.txt").unwrap();
    println!("cargo:rerun-if-changed=assets/required.txt");
    println!("cargo:rerun-if-changed=build.rs");
}
"#,
    );
    write(root.join("app/assets/required.txt"), "required");
    write(root.join("app/docs/noise.md"), "noise");
    write(
        root.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected() -> String {
    let mut value = live_dep::live();
    #[cfg(unix)]
    {
        value.push_str(&platform_dep::platform());
    }
    value
}

pub fn dead() -> String {
    dead_dep::dead()
}
"#,
    );
    write_package(
        root,
        "live_dep",
        r#"
pub fn live() -> String {
    "live".to_string()
}
"#,
    );
    write_package(
        root,
        "platform_dep",
        r#"
pub fn platform() -> String {
    ":platform".to_string()
}
"#,
    );
    write_package(
        root,
        "dead_dep",
        r#"
pub fn dead() -> String {
    "dead".to_string()
}
"#,
    );
}

fn write_ffi_manifest_bundle_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[workspace]
members = ["app", "dead_dep", "uniffi"]
resolver = "2"

[workspace.dependencies]
opensourced = {{ path = "{}" }}

[patch.crates-io]
platform-dep = {{ path = "platform-dep" }}
unused-patch = {{ path = "unused-patch" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("Cargo.lock"),
        r#"# This file is automatically @generated by Cargo.
version = 3
"#,
    );
    write(
        root.join("rust-toolchain.toml"),
        &format!("[toolchain]\nchannel = {:?}\n", current_rustup_toolchain()),
    );
    write(
        root.join("app/Cargo.toml"),
        r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["lib", "cdylib", "staticlib"]

[features]
ffi = ["dep:uniffi"]
dead = ["dep:dead_dep"]

[dependencies]
dead_dep = { path = "../dead_dep", optional = true }
opensourced.workspace = true
uniffi = { path = "../uniffi", optional = true }

[target.'cfg(unix)'.dependencies]
platform_dep = { package = "platform-dep", version = "0.1.0" }
"#,
    );
    write(
        root.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[cfg_attr(feature = "ffi", uniffi::export)]
#[opensourced]
pub fn selected() -> String {
    let mut value = "selected".to_string();
    #[cfg(unix)]
    {
        value.push_str(platform_dep::platform());
    }
    value
}

pub fn dead() -> &'static str {
    dead_dep::dead()
}
"#,
    );
    write_package(
        root,
        "dead_dep",
        r#"pub fn dead() -> &'static str {
    "dead"
}
"#,
    );
    write_package(
        root,
        "uniffi",
        r#"pub fn marker() {}
"#,
    );
    write(
        root.join("platform-dep/Cargo.toml"),
        r#"[package]
name = "platform-dep"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        root.join("platform-dep/src/lib.rs"),
        r#"pub fn platform() -> &'static str {
    ":unix"
}

pub fn unused_platform() -> &'static str {
    "unused"
}
"#,
    );
    write(
        root.join("unused-patch/Cargo.toml"),
        r#"[package]
name = "unused-patch"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        root.join("unused-patch/src/lib.rs"),
        r#"pub fn unused_patch() -> &'static str {
    "unused"
}
"#,
    );
}

fn write_build_dependency_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[workspace]
members = ["app", "build_helper"]
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
build = "build.rs"

[dependencies]
opensourced.workspace = true

[build-dependencies]
build_helper = { path = "../build_helper" }
"#,
    );
    write(
        root.join("app/build.rs"),
        r#"fn main() {
    build_helper::emit();
}
"#,
    );
    write(
        root.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected() -> &'static str {
    "selected"
}
"#,
    );
    write_package(
        root,
        "build_helper",
        r#"pub fn emit() {
    println!("cargo:rerun-if-changed=build.rs");
}
"#,
    );
}

fn write_unused_build_dependency_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[workspace]
members = ["app", "build_helper"]
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

[build-dependencies]
build_helper = { path = "../build_helper" }
"#,
    );
    write(
        root.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected() -> &'static str {
    "selected"
}
"#,
    );
    write_package(
        root,
        "build_helper",
        r#"pub fn emit() {
    println!("cargo:rerun-if-changed=build.rs");
}
"#,
    );
}

fn write_unused_local_dependency_edge_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[workspace]
members = ["app", "hub", "shared"]
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
hub = { path = "../hub" }
opensourced.workspace = true
shared = { path = "../shared" }
"#,
    );
    write(
        root.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected() -> &'static str {
    hub::selected()
}
"#,
    );
    write(
        root.join("hub/Cargo.toml"),
        r#"[package]
name = "hub"
version = "0.1.0"
edition = "2021"

[dependencies]
shared = { path = "../shared" }
"#,
    );
    write(
        root.join("hub/src/lib.rs"),
        r#"pub fn selected() -> &'static str {
    shared::message()
}
"#,
    );
    write_package(
        root,
        "shared",
        r#"pub fn message() -> &'static str {
    "shared"
}
"#,
    );
}

fn write_dependency_alias_ident_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[workspace]
members = ["app", "payload"]
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
payload = { path = "../payload" }
"#,
    );
    write(
        root.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected() -> u32 {
    let payload = 7;
    payload + 1
}
"#,
    );
    write_package(
        root,
        "payload",
        r#"pub struct Payload;

pub fn dead_payload() -> Payload {
    Payload
}
"#,
    );
}

fn write_target_build_dependency_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[workspace]
members = ["app", "build_helper"]
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
build = "build.rs"

[dependencies]
opensourced.workspace = true

[target.'cfg(all())'.build-dependencies]
build_helper = { path = "../build_helper" }
"#,
    );
    write(
        root.join("app/build.rs"),
        r#"fn main() {
    build_helper::emit();
}
"#,
    );
    write(
        root.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected() -> &'static str {
    "selected"
}
"#,
    );
    write_package(
        root,
        "build_helper",
        r#"pub fn emit() {
    println!("cargo:rerun-if-changed=build.rs");
}
"#,
    );
}

fn write_external_workspace_path_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let external_name = format!(
        "{}-external-helper",
        root.file_name().unwrap().to_string_lossy()
    );
    let external_leaf_name = format!(
        "{}-external-leaf",
        root.file_name().unwrap().to_string_lossy()
    );
    let external_unused_name = format!(
        "{}-external-unused",
        root.file_name().unwrap().to_string_lossy()
    );
    let external_root = root.parent().unwrap().join(&external_name);
    if external_root.exists() {
        fs::remove_dir_all(&external_root).unwrap();
    }
    let external_leaf_root = root.parent().unwrap().join(&external_leaf_name);
    if external_leaf_root.exists() {
        fs::remove_dir_all(&external_leaf_root).unwrap();
    }
    let external_unused_root = root.parent().unwrap().join(&external_unused_name);
    if external_unused_root.exists() {
        fs::remove_dir_all(&external_unused_root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");

    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[workspace]
members = ["app"]
resolver = "2"

[workspace.dependencies]
external_helper = {{ package = "external-helper", path = "../{external_name}" }}
opensourced = {{ path = "{}" }}

[patch.crates-io]
external-helper = {{ path = "../{external_name}" }}
external-leaf = {{ path = "../{external_leaf_name}" }}
external-unused = {{ path = "../{external_unused_name}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("Cargo.lock"),
        r#"# This file is automatically @generated by Cargo.
version = 3
"#,
    );
    write(
        root.join("app/Cargo.toml"),
        r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
external_helper.workspace = true
opensourced.workspace = true
"#,
    );
    write(
        root.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected(value: &str) -> String {
    external_helper::decorate(value)
}

pub fn dead() -> String {
    "dead".to_string()
}
"#,
    );
    write(
        external_root.join("Cargo.toml"),
        r#"[package]
name = "external-helper"
version = "0.1.0"
edition = "2021"

[dependencies]
external_leaf = { package = "external-leaf", version = "0.1.0" }
"#,
    );
    write(
        external_root.join("src/lib.rs"),
        r#"pub fn decorate(value: &str) -> String {
    format!("external:{value}:{}", external_leaf::suffix())
}

pub fn unused() -> &'static str {
    "unused"
}
"#,
    );
    write(
        external_leaf_root.join("Cargo.toml"),
        r#"[package]
name = "external-leaf"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        external_leaf_root.join("src/lib.rs"),
        r#"pub fn suffix() -> &'static str {
    "leaf"
}
"#,
    );
    write(
        external_unused_root.join("Cargo.toml"),
        r#"[package]
name = "external-unused"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        external_unused_root.join("src/lib.rs"),
        r#"pub fn unused_patch() -> &'static str {
    "unused"
}
"#,
    );
}

fn write_extension_trait_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "extension_trait_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
tokio = {{ version = "1", features = ["io-util"] }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("src/lib.rs"),
        r#"use tokio::io::{AsyncRead, AsyncReadExt};

#[opensourced]
pub async fn selected<R: AsyncRead + Unpin>(reader: &mut R) -> std::io::Result<String> {
    let mut value = String::new();
    reader.read_to_string(&mut value).await?;
    Ok(value)
}

pub async fn dead() -> &'static str {
    "dead"
}
"#,
    );
}

fn write_unused_extension_trait_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "unused_extension_trait_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
tokio = {{ version = "1", features = ["io-util"] }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("src/lib.rs"),
        r#"use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[opensourced]
pub fn selected(value: &str) -> String {
    let _ = std::fs::read_to_string(value).ok();
    value.to_string()
}

pub async fn dead<R: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin>(
    stream: &mut R,
) -> std::io::Result<String> {
    let mut value = String::new();
    stream.read_to_string(&mut value).await?;
    stream.write_all(value.as_bytes()).await?;
    Ok(value)
}
"#,
    );
}

fn write_external_trait_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "external_trait_like"
version = "0.1.0"
edition = "2021"

[dependencies]
base64 = "0.22"
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("src/lib.rs"),
        r#"use base64::Engine;
use opensourced::opensourced;

#[opensourced]
pub fn selected(input: &str) -> Option<Vec<u8>> {
    let engine = base64::engine::general_purpose::STANDARD;
    engine.decode(input).ok()
}

pub fn dead() -> &'static str {
    "dead"
}
"#,
    );
}

fn write_unused_serde_derive_import_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "serde_derive_import_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
serde = {{ version = "1.0", features = ["derive"] }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("src/lib.rs"),
        r#"use opensourced::opensourced;
use serde::Deserialize;

#[derive(Deserialize)]
struct DeadWire {
    value: String,
}

#[derive(serde::Serialize)]
pub struct KeptWire {
    value: i32,
}

#[opensourced]
pub fn selected(value: i32) -> KeptWire {
    KeptWire { value }
}
"#,
    );
}

fn write_module_scoped_unused_import_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "module_scoped_unused_imports"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
serde = {{ version = "1.0", features = ["derive"] }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("src/lib.rs"),
        r#"pub mod api {
    use opensourced::opensourced;

    #[opensourced]
    pub trait Handler {
        fn handle(&self) -> String;
    }
}

pub mod live {
    use opensourced::opensourced;
    use serde::Serialize;

    #[derive(Serialize)]
    #[opensourced]
    pub struct LiveDto {
        pub value: u8,
    }
}

pub mod client {
    use opensourced::opensourced;
    use crate::api::Handler;
    use serde::Serialize;

    #[opensourced]
    pub struct ClientConfig {
        pub name: String,
    }

    #[derive(Serialize)]
    struct DeadDto {
        value: u8,
    }

    pub fn dead(handler: &dyn Handler) -> DeadDto {
        let _ = handler.handle();
        DeadDto { value: 7 }
    }
}
"#,
    );
}

fn write_local_proc_macro_helper_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        r#"[workspace]
members = ["app", "derive-support", "macro-support", "unused-proc-support"]
resolver = "2"
"#,
    );
    write(
        root.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
derive-support = {{ path = "../derive-support" }}
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("app/src/lib.rs"),
        r#"use derive_support::UseHelper;
use opensourced::opensourced;

#[derive(UseHelper)]
pub struct RetainedWire {
    #[helper(default = default_token)]
    value: u32,
}

#[derive(UseHelper)]
pub struct DeadWire {
    #[helper(default = unused_helper)]
    value: u32,
}

#[opensourced]
pub fn selected(value: u32) -> u32 {
    RetainedWire { value }.generated()
}

fn default_token() -> u32 {
    7
}

fn unused_helper() -> u32 {
    99
}
"#,
    );
    write(
        root.join("derive-support/Cargo.toml"),
        r#"[package]
name = "derive-support"
version = "0.1.0"
edition = "2021"

[dependencies]
macro-support = { path = "../macro-support" }
unused-proc-support = { path = "../unused-proc-support" }

[lib]
proc-macro = true
"#,
    );
    write(
        root.join("derive-support/src/lib.rs"),
        r#"extern crate proc_macro;

use proc_macro::TokenStream;
use macro_support::generated_method_name;

#[proc_macro_derive(UseHelper, attributes(helper))]
pub fn use_helper(input: TokenStream) -> TokenStream {
    let input = input.to_string();
    let name = input
        .split_whitespace()
        .skip_while(|token| *token != "struct")
        .nth(1)
        .expect("derive input should contain a struct name")
        .trim_matches('{')
        .trim_matches(';')
        .split('<')
        .next()
        .expect("struct name should not be empty");
    let method = generated_method_name();
    format!("impl {name} {{ pub fn {method}(&self) -> u32 {{ default_token() }} }}")
        .parse()
        .expect("generated derive output should parse")
}

#[proc_macro_derive(UnusedDerive)]
pub fn unused_derive(input: TokenStream) -> TokenStream {
    let _ = unused_proc_support::dead_marker();
    dead_macro_helper(input)
}

#[proc_macro_attribute]
pub fn unused_attr(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let _ = unused_proc_support::dead_marker();
    dead_macro_helper(item)
}

#[proc_macro]
pub fn unused_macro(input: TokenStream) -> TokenStream {
    let _ = unused_proc_support::dead_marker();
    dead_macro_helper(input)
}

fn dead_macro_helper(input: TokenStream) -> TokenStream {
    input
}
"#,
    );
    write(
        root.join("macro-support/Cargo.toml"),
        r#"[package]
name = "macro-support"
version = "0.1.0"
edition = "2021"

[dev-dependencies]
pretty_assertions = "1"
"#,
    );
    write(
        root.join("macro-support/src/lib.rs"),
        r#"pub mod live;

pub fn generated_method_name() -> &'static str {
    live::generated_method_name()
}
"#,
    );
    write(
        root.join("macro-support/src/live.rs"),
        r#"pub fn generated_method_name() -> &'static str {
    "generated"
}
"#,
    );
    write(
        root.join("macro-support/src/orphan.rs"),
        r#"pub fn unused_orphan() -> &'static str {
    "unused"
}
"#,
    );
    write(
        root.join("macro-support/examples/unused.rs"),
        "fn main() {}\n",
    );
    write(
        root.join("macro-support/tests/unused.rs"),
        "#[test]\nfn unused() {}\n",
    );
    write(
        root.join("macro-support/benches/unused.rs"),
        "fn main() {}\n",
    );
    write(
        root.join("unused-proc-support/Cargo.toml"),
        r#"[package]
name = "unused-proc-support"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        root.join("unused-proc-support/src/lib.rs"),
        r#"pub fn dead_marker() -> &'static str {
    "dead"
}
"#,
    );
}

fn write_serde_trait_associated_call_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "serde_trait_associated_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1"
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("src/lib.rs"),
        r#"use opensourced::opensourced;
use serde::Deserialize;

#[opensourced]
pub fn selected<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    Ok(value.and_then(|value| match value {
        serde_json::Value::Null => None,
        serde_json::Value::String(text) => Some(text),
        other => serde_json::to_string(&other).ok(),
    }))
}
"#,
    );
}

fn write_digest_trait_associated_call_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "digest_trait_associated_like"
version = "0.1.0"
edition = "2021"

[dependencies]
hex = "0.4"
opensourced = {{ path = "{}" }}
sha1 = "0.10"
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("src/lib.rs"),
        r#"use opensourced::opensourced;
use sha1::{Digest, Sha1};

#[opensourced]
pub fn selected(input: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}
"#,
    );
}

fn write_digest_trait_dead_hash_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "digest_trait_dead_hash_like"
version = "0.1.0"
edition = "2021"

[dependencies]
hex = "0.4"
opensourced = {{ path = "{}" }}
sha1 = "0.10"
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("src/lib.rs"),
        r#"use opensourced::opensourced;
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::sync::Mutex;

pub type Cache = Mutex<HashMap<String, String>>;

#[opensourced]
pub fn selected() -> Cache {
    Mutex::new(HashMap::new())
}

fn dead_bucket(input: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}
"#,
    );
}

fn write_unused_external_type_import_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "external_type_import_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
serde_json = "1"
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("src/lib.rs"),
        r#"use opensourced::opensourced;
use serde_json::{Map, Value};

#[opensourced]
pub fn selected(value: i32) -> i32 {
    value + 1
}

pub fn dead(input: Map<String, Value>) -> Option<Value> {
    input.into_iter().next().map(|(_, value)| value)
}
"#,
    );
}

fn write_unused_local_import_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "local_import_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("src/lib.rs"),
        r#"pub mod feature;
pub mod other;
pub mod shared;
"#,
    );
    write(
        root.join("src/feature.rs"),
        r#"use crate::shared::{FeatureValue, OtherValue};
use opensourced::opensourced;

#[opensourced]
pub fn selected_feature() -> FeatureValue {
    FeatureValue { value: 1 }
}

pub fn dead() -> OtherValue {
    OtherValue { value: 2 }
}
"#,
    );
    write(
        root.join("src/other.rs"),
        r#"use crate::shared::OtherValue;
use opensourced::opensourced;

#[opensourced]
pub fn selected_other() -> OtherValue {
    OtherValue { value: 3 }
}
"#,
    );
    write(
        root.join("src/shared.rs"),
        r#"pub struct FeatureValue {
    pub value: i32,
}

pub struct OtherValue {
    pub value: i32,
}
"#,
    );
}

fn write_super_glob_parent_import_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "super_glob_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("src/lib.rs"),
        r#"pub mod parent;
pub mod types;
"#,
    );
    write(
        root.join("src/types.rs"),
        r#"pub struct PendingApproval {
    pub id: String,
}

pub struct PendingApprovalSeed {
    pub raw_params: String,
}
"#,
    );
    write(
        root.join("src/parent/mod.rs"),
        r#"use crate::types::{PendingApproval, PendingApprovalSeed};

pub mod child;
use self::child::*;
"#,
    );
    write(
        root.join("src/parent/child.rs"),
        r#"use opensourced::opensourced;
use super::*;

#[opensourced]
pub fn selected(
    approval: &PendingApproval,
    seed: Option<&PendingApprovalSeed>,
) -> Option<String> {
    seed.map(|seed| format!("{}:{}", approval.id, seed.raw_params))
}
"#,
    );
}

fn write_super_glob_shadowed_parent_import_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "super_glob_shadowed_parent_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("src/lib.rs"),
        r#"pub mod other;
pub mod parent;
pub mod shared;
"#,
    );
    write(
        root.join("src/shared.rs"),
        r#"pub struct FeatureValue {
    pub value: i32,
}

pub fn helper() -> u32 {
    41
}
"#,
    );
    write(
        root.join("src/parent/mod.rs"),
        r#"use crate::shared::{FeatureValue, helper};

pub mod child;
"#,
    );
    write(
        root.join("src/parent/child.rs"),
        r#"use opensourced::opensourced;
use super::*;

#[opensourced]
pub fn selected_child() -> FeatureValue {
    let helper = 1;
    let _ = helper + 1;
    FeatureValue { value: 1 }
}
"#,
    );
    write(
        root.join("src/other.rs"),
        r#"use opensourced::opensourced;
use crate::shared::helper;

#[opensourced]
pub fn selected_other() -> u32 {
    helper()
}
"#,
    );
}

fn write_unused_private_struct_field_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "private_field_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1"
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("src/lib.rs"),
        r#"use opensourced::opensourced;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Seed {
    request_id: Value,
    raw_params: String,
}

#[opensourced]
pub(crate) fn selected(seed: &Seed) -> String {
    seed.raw_params.clone()
}
"#,
    );
}

fn write_std_trait_import_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "std_trait_import_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("src/lib.rs"),
        r#"use std::hash::{Hash, Hasher};
use std::io::Write;
use std::sync::atomic::{AtomicI64, Ordering};

#[opensourced]
pub fn selected(value: &str) -> u64 {
    let mut buffer = Vec::new();
    buffer.write_all(value.as_bytes()).unwrap();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    buffer.hash(&mut hasher);
    hasher.finish()
}

pub fn dead() -> &'static str {
    "dead"
}
"#,
    );
}

fn write_dyn_trait_field_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "dyn_trait_field_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("src/lib.rs"),
        r#"use std::sync::Arc;

trait PlatformService: Send + Sync {
    fn label(&self) -> &'static str;
}

pub struct RuntimeHandle {
    service: Option<Arc<dyn PlatformService>>,
}

#[opensourced]
pub fn selected() -> RuntimeHandle {
    RuntimeHandle { service: None }
}

trait DeadService {}

pub struct DeadHandle {
    service: Option<Arc<dyn DeadService>>,
}
"#,
    );
}

fn write_generic_field_bound_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "generic_field_bound_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("src/lib.rs"),
        r#"trait ExternalHandler {
    fn ready(&self) -> bool;
}

pub struct ExternalHandle<T: ExternalHandler> {
    inner: T,
}

struct ClientHandler;

impl ExternalHandler for ClientHandler {
    fn ready(&self) -> bool {
        true
    }
}

pub struct RuntimeHandle {
    handle: Option<ExternalHandle<ClientHandler>>,
}

#[opensourced]
pub fn selected() -> RuntimeHandle {
    RuntimeHandle { handle: None }
}

struct DeadHandler;

impl ExternalHandler for DeadHandler {
    fn ready(&self) -> bool {
        false
    }
}
"#,
    );
}

fn write_if_let_option_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "if_let_option_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("src/lib.rs"),
        r#"use std::sync::Arc;

pub struct Client {
    store: Store,
}

pub struct Store;

impl Store {
    fn emit(&self) {}

    fn unused_emit(&self) {}
}

fn shared_client() -> Option<Arc<Client>> {
    None
}

#[opensourced]
pub fn selected() {
    if let Some(client) = shared_client() {
        client.store.emit();
    }
}
"#,
    );
}

fn write_glob_type_reexport_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "glob_type_reexport_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("src/lib.rs"),
        r#"use opensourced::opensourced;

pub mod types;

#[opensourced]
pub fn selected() -> types::ThreadSnapshot {
    types::ThreadSnapshot {
        effective_approval_policy: None,
    }
}
"#,
    );
    write(
        root.join("src/types/mod.rs"),
        r#"pub mod models;

pub use models::*;
"#,
    );
    write(
        root.join("src/types/models.rs"),
        r#"pub struct ThreadSnapshot {
    pub effective_approval_policy: Option<crate::types::AppAskForApproval>,
}

pub enum AppAskForApproval {
    Never,
}

pub enum DeadPolicy {
    Noise,
}
"#,
    );
}

fn write_parent_glob_super_import_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "parent_glob_super_import_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("src/lib.rs"),
        r#"use opensourced::opensourced;

pub mod types;

#[opensourced]
pub fn selected() -> u8 {
    types::server_requests::convert(types::models::AppAskForApproval::Never)
}
"#,
    );
    write(
        root.join("src/types/mod.rs"),
        r#"pub mod dead;
pub mod models;
pub mod server_requests;

pub use dead::*;
pub use models::*;
pub use server_requests::*;
"#,
    );
    write(
        root.join("src/types/models.rs"),
        r#"pub enum AppAskForApproval {
    Never,
}

pub enum DeadPolicy {
    Noise,
}
"#,
    );
    write(
        root.join("src/types/server_requests.rs"),
        r#"use super::AppAskForApproval;

pub fn convert(value: AppAskForApproval) -> u8 {
    match value {
        AppAskForApproval::Never => 0,
    }
}

pub fn unused_convert() -> u8 {
    99
}
"#,
    );
    write(
        root.join("src/types/dead.rs"),
        r#"pub struct Dead;
"#,
    );
}

fn write_implicit_feature_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[workspace]
members = ["app", "helper"]
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
helper = { path = "../helper", features = ["feature_dep"] }
opensourced.workspace = true
"#,
    );
    write(
        root.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected() -> String {
    helper::value()
}
"#,
    );
    write(
        root.join("helper/Cargo.toml"),
        r#"[package]
name = "helper"
version = "0.1.0"
edition = "2021"

[dependencies]
feature_dep = { path = "../feature_dep", optional = true }
"#,
    );
    write(
        root.join("helper/src/lib.rs"),
        r#"pub fn value() -> String {
    "helper".to_string()
}
"#,
    );
    write_package(
        root,
        "feature_dep",
        r#"pub fn value() -> &'static str {
    "feature"
}
"#,
    );
}

fn write_mutex_field_fixture(root: &Path) {
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
lazy_static = "1"
opensourced.workspace = true
parking_lot = "0.12"
"#,
    );
    write(
        root.join("app/src/lib.rs"),
        r#"use std::{
    collections::HashMap,
    sync::Weak,
    time::{Duration, Instant},
};

use opensourced::opensourced;
use parking_lot::Mutex;

static GLOBALS: Mutex<Globals> = Mutex::new(Globals::new());

lazy_static::lazy_static! {
    static ref REGISTERED_INTERRUPTS: Mutex<Vec<Weak<dyn AsRef<SqlInterruptHandle> + Send + Sync>>> = Mutex::new(Vec::new());
}

#[opensourced]
pub fn selected(message: String) {
    GLOBALS.lock().breadcrumbs.push(message);
    let _registered_count = REGISTERED_INTERRUPTS.lock().len();
}

pub struct SqlInterruptHandle;

impl AsRef<SqlInterruptHandle> for SqlInterruptHandle {
    fn as_ref(&self) -> &SqlInterruptHandle {
        self
    }
}

struct Globals {
    breadcrumbs: BreadcrumbRingBuffer,
    rate_limiter: RateLimiter,
}

impl Globals {
    const fn new() -> Self {
        Self {
            breadcrumbs: BreadcrumbRingBuffer::new(),
            rate_limiter: RateLimiter::new(),
        }
    }
}

#[derive(Default)]
struct BreadcrumbRingBuffer {
    breadcrumbs: Vec<String>,
    pos: usize,
}

impl BreadcrumbRingBuffer {
    const MAX_ITEMS: usize = 20;

    const fn new() -> Self {
        Self {
            breadcrumbs: Vec::new(),
            pos: 0,
        }
    }

    fn push(&mut self, breadcrumb: String) {
        if self.breadcrumbs.len() < Self::MAX_ITEMS {
            self.breadcrumbs.push(breadcrumb);
        } else {
            self.breadcrumbs[self.pos] = breadcrumb;
            self.pos = (self.pos + 1) % Self::MAX_ITEMS;
        }
    }
}

struct RateLimiter {
    last_report: Option<HashMap<String, Instant>>,
}

impl RateLimiter {
    const INTERVAL: Duration = Duration::from_secs(180);

    const fn new() -> Self {
        Self { last_report: None }
    }
}
"#,
    );
}

fn write_external_trait_method_arg_fixture(root: &Path) {
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
base64 = "0.21"
opensourced.workspace = true
serde = "1"
"#,
    );
    write(
        root.join("app/src/lib.rs"),
        r#"use std::{cmp::Ordering, fmt};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use opensourced::opensourced;
use serde::de::{self, Deserializer, Visitor};

#[derive(Clone)]
pub struct Guid(String);

impl Guid {
    fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl Ord for Guid {
    #[opensourced]
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_bytes().cmp(other.as_bytes())
    }
}

impl PartialOrd for Guid {
    #[opensourced]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Guid {
    #[opensourced]
    fn eq(&self, other: &Self) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl Eq for Guid {}

unsafe impl Sync for Guid {}

struct GuidVisitor;

impl Visitor<'_> for GuidVisitor {
    type Value = Guid;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a guid")
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
        Ok(Guid(value.to_string()))
    }
}

#[opensourced]
pub fn selected<'de, D>(deserializer: D) -> Result<Guid, D::Error>
where
    D: Deserializer<'de>,
{
    let bytes = [0u8; 9];
    let mut output = [0u8; 12];
    URL_SAFE_NO_PAD.encode_slice(bytes, &mut output).unwrap();
    let left = Guid("a".to_string());
    let right = Guid("b".to_string());
    let _ordering = Ord::cmp(&left, &right);
    let _same = PartialEq::eq(&left, &right);
    deserializer.deserialize_str(GuidVisitor)
}
"#,
    );
}

fn write_associated_conversion_fixture(root: &Path) {
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
        r#"#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
use std::process::ExitStatus;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use opensourced::opensourced;

#[derive(Copy, Clone)]
pub struct Timestamp(pub u64);

impl Timestamp {
    #[opensourced]
    pub fn checked_sub(self, d: Duration) -> Option<Timestamp> {
        SystemTime::from(self).checked_sub(d).map(Into::into)
    }
}

#[cfg(unix)]
#[opensourced]
pub fn status_from_code(code: i32) -> ExitStatus {
    ExitStatus::from_raw(code << 8)
}

impl From<SystemTime> for Timestamp {
    fn from(st: SystemTime) -> Self {
        let d = st.duration_since(UNIX_EPOCH).unwrap();
        Timestamp((d.as_secs()) * 1000 + (u64::from(d.subsec_nanos()) / 1_000_000))
    }
}

impl From<Timestamp> for SystemTime {
    fn from(ts: Timestamp) -> Self {
        UNIX_EPOCH + Duration::from_millis(ts.0)
    }
}

impl From<u64> for Timestamp {
    fn from(ts: u64) -> Self {
        Timestamp(ts)
    }
}
"#,
    );
}

fn write_feedback_layer_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[workspace]
members = ["app", "error-support"]
resolver = "2"

[workspace.dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("error-support/Cargo.toml"),
        r#"[package]
name = "error-support"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        root.join("error-support/src/lib.rs"),
        r#"#[macro_export]
macro_rules! debug {
    ($($tokens:tt)*) => {};
}

#[macro_export]
macro_rules! warn {
    ($($tokens:tt)*) => {};
}
"#,
    );
    write(
        root.join("app/Cargo.toml"),
        r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
error-support = { path = "../error-support" }
opensourced.workspace = true
thiserror = "2"
"#,
    );
    write(
        root.join("app/src/lib.rs"),
        r#"#![allow(unused_macros)]

mod db;

use error_support::{debug, warn};

pub use db::selected;
"#,
    );
    write(
        root.join("app/src/db.rs"),
        r#"use std::{fmt, time::SystemTime};

use crate::{debug, warn};
use opensourced::opensourced;
use thiserror::Error;

type Result<T> = std::result::Result<T, LocalError>;

#[derive(Debug, Error)]
pub enum LocalError {
    #[error("io")]
    Io,
    #[error("interrupted {0}")]
    Interrupted(#[from] Interrupted),
}

impl From<std::io::Error> for LocalError {
    fn from(_: std::io::Error) -> Self {
        Self::Io
    }
}

#[derive(Debug)]
pub struct Interrupted;

impl Interrupted {
    fn is_valid(&self) -> bool {
        true
    }
}

impl fmt::Display for Interrupted {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("interrupted")
    }
}

impl std::error::Error for Interrupted {}

pub trait ConnExt {
    fn conn(&self) -> &SystemTime;

    fn set_pragma(&self) -> std::result::Result<(), std::io::Error> {
        let _ = self.conn();
        fallible()
    }
}

impl ConnExt for SystemTime {
    fn conn(&self) -> &SystemTime {
        self
    }
}

#[derive(Debug, Clone)]
pub struct RepeatDisplay<'a, F> {
    count: usize,
    sep: &'a str,
    fmt_one: F,
}

impl<F> fmt::Display for RepeatDisplay<'_, F>
where
    F: Fn(usize, &mut fmt::Formatter<'_>) -> fmt::Result,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in 0..self.count {
            if i != 0 {
                f.write_str(self.sep)?;
            }
            (self.fmt_one)(i, f)?;
        }
        Ok(())
    }
}

fn repeat_display<F>(count: usize, sep: &str, fmt_one: F) -> RepeatDisplay<'_, F>
where
    F: Fn(usize, &mut fmt::Formatter<'_>) -> fmt::Result,
{
    RepeatDisplay { count, sep, fmt_one }
}

pub fn repeat_sql_vars(count: usize) -> impl fmt::Display {
    repeat_display(count, ",", |_, f| write!(f, "?"))
}

fn fallible() -> std::result::Result<(), std::io::Error> {
    Ok(())
}

#[opensourced]
pub fn selected(conn: &SystemTime, interrupted: Interrupted) -> Result<impl fmt::Display> {
    debug!("selected");
    warn!("selected");
    assert!(interrupted.is_valid(), "invalid interrupted");
    conn.set_pragma()?;
    fallible()?;
    Ok(repeat_sql_vars(1))
}
"#,
    );
}

fn write_feedback_viaduct_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[workspace]
members = ["app", "error-support"]
resolver = "2"

[workspace.dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("error-support/Cargo.toml"),
        r#"[package]
name = "error-support"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        root.join("error-support/src/lib.rs"),
        r#"#[macro_export]
macro_rules! warn {
    ($($tokens:tt)*) => {};
}
"#,
    );
    write(
        root.join("app/Cargo.toml"),
        r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
error-support = { path = "../error-support" }
opensourced.workspace = true
"#,
    );
    write(
        root.join("app/src/lib.rs"),
        r#"use error_support::{warn, ErrorHandling};
use opensourced::opensourced;
use std::borrow::Cow;
use std::sync::Arc;

pub(crate) mod msg_types {
    include!("generated.rs");
}

mod ffi {
    use crate::msg_types;

    pub fn message() -> msg_types::Message {
        msg_types::Message
    }
}

mod old_backend {
    pub trait Backend: Send + Sync + 'static {
        fn send(&self) -> u8;
    }

    pub fn set_backend(_: &'static dyn Backend) -> Result<(), ()> {
        Ok(())
    }
}

pub trait Backend: Send + Sync + 'static {
    fn send_request(&self) -> u8;
}

pub fn init_backend(backend: Arc<dyn Backend>) -> Result<(), ()> {
    old_backend::set_backend(Box::leak(Box::new(backend.clone())))?;
    Ok(())
}

impl old_backend::Backend for Arc<dyn Backend> {
    fn send(&self) -> u8 {
        self.send_request()
    }
}

#[derive(PartialEq)]
pub struct HeaderName(Cow<'static, str>);

impl HeaderName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::ops::Deref for HeaderName {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_str()
    }
}

macro_rules! partialeq_boilerplate {
    ($T0:ty, $T1:ty) => {
        impl<'a> PartialEq<$T0> for $T1 {
            fn eq(&self, other: &$T0) -> bool {
                (&*self).eq_ignore_ascii_case(&*other)
            }
        }
        impl<'a> PartialEq<$T1> for $T0 {
            fn eq(&self, other: &$T1) -> bool {
                PartialEq::eq(other, self)
            }
        }
    };
}

partialeq_boilerplate!(HeaderName, str);
partialeq_boilerplate!(HeaderName, &'a str);

pub struct Header {
    name: HeaderName,
    value: String,
}

impl Header {
    pub fn new(name: HeaderName, value: impl Into<String>) -> Self {
        Self {
            name,
            value: value.into(),
        }
    }

    fn set_value(&mut self, value: impl AsRef<str>) -> Result<(), ()> {
        self.value.clear();
        self.value.push_str(value.as_ref());
        Ok(())
    }
}

#[derive(Default)]
pub struct Headers {
    headers: Vec<Header>,
}

impl Headers {
    pub fn insert(&mut self, name: HeaderName, value: impl AsRef<str> + Into<String>) -> Result<(), ()> {
        if let Some(entry) = self.headers.iter_mut().find(|header| header.name == name) {
            entry.set_value(value)?;
        } else {
            self.headers.push(Header::new(name, value));
        }
        Ok(())
    }
}

#[opensourced]
pub fn selected(
    backend: Arc<dyn Backend>,
    headers: &mut Headers,
    name: HeaderName,
) -> Result<bool, ()> {
    warn!("selected");
    let _message = ffi::message();
    init_backend(backend)?;
    headers.insert(name, "updated")?;
    Ok(true)
}
"#,
    );
    write(
        root.join("app/src/generated.rs"),
        r#"pub struct Message;
"#,
    );
}

fn write_generic_method_recovery_fixture(root: &Path) {
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

pub struct Processor {
    widgets: Vec<Widget>,
}

impl Processor {
    pub fn new() -> Self {
        Self {
            widgets: vec![Widget::new(1)],
        }
    }

    fn with_writer(
        &mut self,
        op: impl FnOnce(&mut Writer) -> Result<usize, ()>,
    ) -> Result<usize, ()> {
        let mut writer = Writer { widgets: Vec::new() };
        let written = op(&mut writer)?;
        self.widgets.extend(writer.widgets);
        Ok(written + self.widgets.calculate_marker())
    }
}

pub struct Writer {
    widgets: Vec<Widget>,
}

impl Writer {
    fn persist_widget(&mut self, widget: Widget) -> Result<usize, ()> {
        self.widgets.push(widget);
        Ok(self.widgets.calculate_marker())
    }

    fn discarded_noise(&self) -> usize {
        999
    }
}

#[derive(Clone)]
pub struct Widget {
    value: usize,
}

impl Widget {
    fn new(value: usize) -> Self {
        Self { value }
    }
}

trait MarkerSum {
    fn calculate_marker(&self) -> usize;
}

impl MarkerSum for Vec<Widget> {
    fn calculate_marker(&self) -> usize {
        self.iter().map(|widget| widget.value).sum()
    }
}

#[opensourced]
pub fn selected(processor: &mut Processor) -> Result<usize, ()> {
    processor.with_writer(|writer| writer.persist_widget(Widget::new(41)))
}
"#,
    );
}

fn write_feedback_places_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[workspace]
members = ["app", "helper", "error-support", "error-support-macros"]
resolver = "2"

[workspace.dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("error-support-macros/Cargo.toml"),
        r#"[package]
name = "error-support-macros"
version = "0.1.0"
edition = "2021"

[lib]
proc-macro = true
"#,
    );
    write(
        root.join("error-support-macros/src/lib.rs"),
        r#"use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn handle_error(_args: TokenStream, input: TokenStream) -> TokenStream {
    input
}
"#,
    );
    write(
        root.join("error-support/Cargo.toml"),
        r#"[package]
name = "error-support"
version = "0.1.0"
edition = "2021"

[dependencies]
error-support-macros = { path = "../error-support-macros" }
"#,
    );
    write(
        root.join("error-support/src/lib.rs"),
        r#"pub use error_support_macros::handle_error;

pub fn convert_log_report_error<E>(error: E) -> E {
    error
}
"#,
    );
    write(
        root.join("helper/Cargo.toml"),
        r#"[package]
name = "helper"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        root.join("helper/src/lib.rs"),
        r#"mod chunks;
pub use chunks::*;

pub fn unused_root() {}
"#,
    );
    write(
        root.join("helper/src/chunks.rs"),
        r#"pub fn each_chunk<T, F>(items: &[T], mut do_chunk: F)
where
    F: FnMut(&[T]),
{
    do_chunk(items);
}

pub fn unused_chunk() {}
"#,
    );
    write(
        root.join("app/Cargo.toml"),
        r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
error-support = { path = "../error-support" }
helper = { path = "../helper" }
opensourced.workspace = true
"#,
    );
    write(
        root.join("app/src/lib.rs"),
        r#"mod db;

pub use db::selected;
"#,
    );
    write(
        root.join("app/src/db.rs"),
        r#"use error_support::handle_error;
use opensourced::opensourced;

#[derive(Debug)]
pub struct LocalError;

struct PlacesInitializer {
    seed: usize,
}

trait Initializer {
    fn seed(&self) -> usize;
}

impl Initializer for PlacesInitializer {
    fn seed(&self) -> usize {
        self.seed
    }
}

fn open_with_initializer<I: Initializer>(initializer: &I) -> usize {
    initializer.seed()
}

struct HistoryRecord {
    id: String,
}

#[derive(Clone)]
pub enum VisitType {
    Link,
}

pub struct VisitTransitionSet(Vec<VisitType>);

impl VisitTransitionSet {
    fn for_specific(types: &[VisitType]) -> VisitTransitionSet {
        types.iter().cloned().collect()
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}

impl std::iter::FromIterator<VisitType> for VisitTransitionSet {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = VisitType>,
    {
        let mut items = Vec::new();
        for item in iter {
            items.push(item);
        }
        Self(items)
    }
}

#[opensourced]
#[handle_error(LocalError)]
pub fn selected(values: Vec<VisitType>) -> std::result::Result<usize, LocalError> {
    let initializer = PlacesInitializer { seed: 1 };
    let record = HistoryRecord {
        id: "history".to_string(),
    };
    let transitions = VisitTransitionSet::for_specific(&values);
    helper::each_chunk(&values, |_| {});
    Ok(open_with_initializer(&initializer) + record.id.len() + transitions.len())
}
"#,
    );
}

fn write_cfg_benchmark_fixture(root: &Path) {
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

[features]
runtime-benchmarks = []

[dependencies]
opensourced.workspace = true
"#,
    );
    write(
        root.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[cfg(any(test, feature = "runtime-benchmarks"))]
mod benchmarks;
mod live;

#[opensourced]
pub fn selected() -> u32 {
    live::value()
}
"#,
    );
    write(
        root.join("app/src/live.rs"),
        r#"pub fn value() -> u32 {
    7
}
"#,
    );
    write(
        root.join("app/src/benchmarks.rs"),
        r#"pub fn set_timestamp() -> u64 {
    42
}
"#,
    );
}

fn write_reexport_method_fixture(root: &Path) {
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
        r#"mod impls;
mod inner;

pub use inner::*;
"#,
    );
    write(
        root.join("app/src/inner.rs"),
        r#"use opensourced::opensourced;

pub struct Service;

impl Service {
    #[opensourced]
    pub fn selected() -> u32 {
        Service::helper()
    }
}
"#,
    );
    write(
        root.join("app/src/impls.rs"),
        r#"use crate::Service;

impl Service {
    pub fn helper() -> u32 {
        11
    }

    pub fn dead_helper() -> u32 {
        99
    }
}
"#,
    );
}

fn write_included_generated_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[workspace]
members = ["app", "proto_dep"]
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
proto_dep = { path = "../proto_dep" }
"#,
    );
    write(
        root.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected() -> String {
    proto_dep::generated::Generated::label()
}
"#,
    );
    write(
        root.join("proto_dep/Cargo.toml"),
        r#"[package]
name = "proto_dep"
version = "0.1.0"
edition = "2021"

[dependencies]
itoa = "1"
"#,
    );
    write(
        root.join("proto_dep/src/lib.rs"),
        r#"#[macro_export]
macro_rules! include_proto {
    ($path:literal) => {
        include!(concat!("prost/", $path));
    };
}

pub mod generated {
    include_proto!("generated.rs");
}

pub mod unused {
    include_proto!("unused.rs");
}
"#,
    );
    write(
        root.join("proto_dep/src/prost/generated.rs"),
        r#"pub struct Generated;

impl Generated {
    pub fn label() -> String {
        let mut buffer = itoa::Buffer::new();
        buffer.format(7).to_string()
    }
}
"#,
    );
    write(
        root.join("proto_dep/src/prost/unused.rs"),
        r#"pub struct Unused;
"#,
    );
}

fn write_trait_input_impl_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "trait_input_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected() -> ApiEvent {
    LiveEvent {}.into()
}

pub enum ApiEvent {
    Live,
    Dead,
}

pub struct LiveEvent {}

pub struct DeadEvent {}

const DEAD_EVENT_KIND: &str = "dead";

impl From<LiveEvent> for ApiEvent {
    fn from(_: LiveEvent) -> Self {
        ApiEvent::Live
    }
}

impl From<DeadEvent> for ApiEvent {
    fn from(_: DeadEvent) -> Self {
        let _ = DEAD_EVENT_KIND;
        ApiEvent::Dead
    }
}
"#,
    );
}

fn write_glob_reexport_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "glob_reexport_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("src/lib.rs"),
        r#"use opensourced::opensourced;

pub mod api;
pub mod noisy;

pub use api::*;
pub use noisy::*;

#[opensourced]
pub fn selected() -> Chosen {
    Chosen
}
"#,
    );
    write(
        root.join("src/api.rs"),
        r#"pub struct Chosen;

pub struct UnusedApi;

pub fn unused_api_function() -> UnusedApi {
    UnusedApi
}
"#,
    );
    write(
        root.join("src/noisy.rs"),
        r#"pub struct UnusedNoise;

pub fn unused_noise_function() -> UnusedNoise {
    UnusedNoise
}
"#,
    );
}

fn write_trait_method_binary_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        r#"[workspace]
members = ["trait_method_bin_like"]
resolver = "2"
"#,
    );
    write(
        root.join("trait_method_bin_like/Cargo.toml"),
        &format!(
            r#"[package]
name = "trait_method_bin_like"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "trait-method-bin"
path = "src/main.rs"

[[bin]]
name = "other-bin"
path = "src/other.rs"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("trait_method_bin_like/src/main.rs"),
        r#"use opensourced::opensourced;
use std::sync::atomic::{AtomicBool, Ordering};

static RAW_MODE: AtomicBool = AtomicBool::new(true);

struct TerminalGuard;

impl Drop for TerminalGuard {
    #[opensourced]
    fn drop(&mut self) {
        RAW_MODE.store(false, Ordering::Relaxed);
    }
}

fn main() {}
"#,
    );
    write(
        root.join("trait_method_bin_like/src/other.rs"),
        r#"fn main() {
    println!("not part of this slice");
}
"#,
    );
}

fn write_include_assets_fixture(root: &Path) {
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        r#"[workspace]
members = ["include_assets_like"]
resolver = "2"
"#,
    );
    write(
        root.join("include_assets_like/Cargo.toml"),
        &format!(
            r#"[package]
name = "include_assets_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("include_assets_like/src/lib.rs"),
        r#"use opensourced::opensourced;

const CORE: &str = include_str!("guidelines/core.md");
const EXTRA: &str = include_str!("../assets/extra.txt");
const CONCAT: &str = include_str!(concat!("guidelines/", "concat.md"));
const MANIFEST: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/manifest.txt"));
const UNUSED: &str = include_str!("guidelines/unused.md");

#[opensourced]
pub fn selected(include_extra: bool) -> String {
    if include_extra {
        format!("{CORE}\n{EXTRA}\n{CONCAT}\n{MANIFEST}")
    } else {
        CORE.to_string()
    }
}

pub fn noisy() -> &'static str {
    UNUSED
}
"#,
    );
    write(
        root.join("include_assets_like/src/guidelines/core.md"),
        "core guideline",
    );
    write(
        root.join("include_assets_like/src/guidelines/concat.md"),
        "concat guideline",
    );
    write(
        root.join("include_assets_like/src/guidelines/unused.md"),
        "unused guideline",
    );
    write(root.join("include_assets_like/assets/extra.txt"), "extra");
    write(
        root.join("include_assets_like/assets/manifest.txt"),
        "manifest",
    );
}

fn write_external_pub_reexport_fixture(root: &Path) {
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        r#"[workspace]
members = ["external_reexport_like"]
resolver = "2"
"#,
    );
    write(
        root.join("external_reexport_like/Cargo.toml"),
        &format!(
            r#"[package]
name = "external_reexport_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
serde_json = "1"
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("external_reexport_like/src/lib.rs"),
        r#"use opensourced::opensourced;
pub use serde_json::{Map, Number, Value};

#[opensourced]
pub fn selected(input: &Value) -> bool {
    input.is_object()
}

pub fn noisy(map: Map<String, Value>, number: Number) -> usize {
    map.len() + number.as_u64().unwrap_or_default() as usize
}
"#,
    );
}

fn write_macro_renamed_import_fixture(root: &Path) {
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        r#"[workspace]
members = ["macro_rename_like"]
resolver = "2"
"#,
    );
    write(
        root.join("macro_rename_like/Cargo.toml"),
        &format!(
            r#"[package]
name = "macro_rename_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
serde_json = "1"
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("macro_rename_like/src/lib.rs"),
        r#"use opensourced::opensourced;
use serde_json::Value as JsonValue;

macro_rules! define_api {
    ($name:ident, $ty:ty) => {
        pub fn $name(value: $ty) -> usize {
            value.to_string().len()
        }
    };
}

define_api!(generated, JsonValue);

#[opensourced]
pub fn selected() -> usize {
    generated(serde_json::json!({"live": true}))
}

pub fn dead() -> usize {
    0
}
"#,
    );
}

fn write_removed_renamed_import_fixture(root: &Path) {
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        r#"[workspace]
members = ["removed_rename_like"]
resolver = "2"
"#,
    );
    write(
        root.join("removed_rename_like/Cargo.toml"),
        &format!(
            r#"[package]
name = "removed_rename_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("removed_rename_like/src/lib.rs"),
        r#"use opensourced::opensourced;
use crate::dead::dead_fn as selected_value;

mod dead {
    pub fn dead_fn() -> u32 {
        99
    }
}

#[opensourced]
pub fn selected() -> u32 {
    let selected_value = 1;
    let closure = |selected_value: u32| selected_value + 1;
    let via_match = match Some(2) {
        Some(selected_value) => selected_value,
        None => 0,
    };
    let mut via_loop = 0;
    for selected_value in [3] {
        via_loop += selected_value;
    }
    selected_value + closure(2) + via_match + via_loop
}
"#,
    );
}

fn write_external_shadowed_import_fixture(root: &Path) {
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        r#"[workspace]
members = ["external_shadowed_import_like"]
resolver = "2"
"#,
    );
    write(
        root.join("external_shadowed_import_like/Cargo.toml"),
        &format!(
            r#"[package]
name = "external_shadowed_import_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("external_shadowed_import_like/src/lib.rs"),
        r#"use opensourced::opensourced;
use std::{fmt, path::PathBuf};

#[opensourced]
pub fn selected() -> PathBuf {
    let fmt = "local binding only";
    let _ = fmt.len();
    PathBuf::new()
}
"#,
    );
}

fn write_shadowed_renamed_import_fixture(root: &Path) {
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        r#"[workspace]
members = ["shadowed_rename_like"]
resolver = "2"
"#,
    );
    write(
        root.join("shadowed_rename_like/Cargo.toml"),
        &format!(
            r#"[package]
name = "shadowed_rename_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("shadowed_rename_like/src/lib.rs"),
        r#"use opensourced::opensourced;
use crate::live::live_fn as selected_value;

mod live {
    pub fn live_fn() -> u32 {
        41
    }
}

#[opensourced]
pub fn selected() -> u32 {
    let shadow = {
        let selected_value = 1;
        selected_value
    };
    shadow + selected_value()
}
"#,
    );
}

fn write_retained_target_shadowed_renamed_import_fixture(root: &Path) {
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        r#"[workspace]
members = ["retained_target_shadowed_rename_like"]
resolver = "2"
"#,
    );
    write(
        root.join("retained_target_shadowed_rename_like/Cargo.toml"),
        &format!(
            r#"[package]
name = "retained_target_shadowed_rename_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("retained_target_shadowed_rename_like/src/lib.rs"),
        r#"pub mod feature;
pub mod other;
pub mod shared;
"#,
    );
    write(
        root.join("retained_target_shadowed_rename_like/src/feature.rs"),
        r#"use opensourced::opensourced;
use crate::shared::{FeatureValue, OtherValue as local_shadow};

#[opensourced]
pub fn selected_feature() -> FeatureValue {
    let local_shadow = 1;
    let _ = local_shadow + 1;
    FeatureValue { value: 1 }
}
"#,
    );
    write(
        root.join("retained_target_shadowed_rename_like/src/other.rs"),
        r#"use opensourced::opensourced;
use crate::shared::OtherValue;

#[opensourced]
pub fn selected_other() -> OtherValue {
    OtherValue { value: 2 }
}
"#,
    );
    write(
        root.join("retained_target_shadowed_rename_like/src/shared.rs"),
        r#"pub struct FeatureValue {
    pub value: i32,
}

pub struct OtherValue {
    pub value: i32,
}
"#,
    );
}

fn write_retained_target_shadowed_direct_import_fixture(root: &Path) {
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        r#"[workspace]
members = ["retained_target_shadowed_direct_like"]
resolver = "2"
"#,
    );
    write(
        root.join("retained_target_shadowed_direct_like/Cargo.toml"),
        &format!(
            r#"[package]
name = "retained_target_shadowed_direct_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("retained_target_shadowed_direct_like/src/lib.rs"),
        r#"pub mod feature;
pub mod other;
pub mod shared;
"#,
    );
    write(
        root.join("retained_target_shadowed_direct_like/src/feature.rs"),
        r#"use opensourced::opensourced;
use crate::shared::{FeatureValue, helper};

#[opensourced]
pub fn selected_feature() -> FeatureValue {
    let helper = 1;
    let _ = helper + 1;
    FeatureValue { value: 1 }
}
"#,
    );
    write(
        root.join("retained_target_shadowed_direct_like/src/other.rs"),
        r#"use opensourced::opensourced;
use crate::shared::helper;

#[opensourced]
pub fn selected_other() -> u32 {
    helper()
}
"#,
    );
    write(
        root.join("retained_target_shadowed_direct_like/src/shared.rs"),
        r#"pub struct FeatureValue {
    pub value: i32,
}

pub fn helper() -> u32 {
    41
}
"#,
    );
}

fn write_inline_super_glob_fixture(root: &Path) {
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        r#"[workspace]
members = ["inline_super_like"]
resolver = "2"
"#,
    );
    write(
        root.join("inline_super_like/Cargo.toml"),
        &format!(
            r#"[package]
name = "inline_super_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("inline_super_like/src/lib.rs"),
        r#"use opensourced::opensourced;
use std::path::{Path, PathBuf};

#[opensourced]
pub mod child {
    use super::*;

    pub fn selected() -> bool {
        Path::new("live").is_relative()
    }
}

pub fn dead() -> PathBuf {
    PathBuf::from("dead")
}
"#,
    );
}

fn write_serde_callback_fixture(root: &Path) {
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        r#"[workspace]
members = ["serde_callback_like"]
resolver = "2"
"#,
    );
    write(
        root.join("serde_callback_like/Cargo.toml"),
        &format!(
            r#"[package]
name = "serde_callback_like"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
serde = {{ version = "1", features = ["derive"] }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("serde_callback_like/src/lib.rs"),
        r#"use opensourced::opensourced;
use serde::Serialize;

#[opensourced]
#[derive(Serialize)]
pub struct Api {
    #[serde(skip_serializing_if = "crate::skip_if_default")]
    value: u32,
}

fn skip_if_default<T: PartialEq + Default>(v: &T) -> bool {
    *v == T::default()
}

pub fn dead() -> bool {
    false
}
"#,
    );
}

fn write_local_public_reexport_fixture(root: &Path) {
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        r#"[workspace]
members = ["app", "provider"]
resolver = "2"
"#,
    );
    write(
        root.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
provider = {{ path = "../provider", optional = true }}

[features]
default = ["provider"]
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected() -> Result<(), provider::Error> {
    Err(provider::Error::Bad)
}
"#,
    );
    write(
        root.join("provider/Cargo.toml"),
        r#"[package]
name = "provider"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        root.join("provider/src/lib.rs"),
        r#"mod error;
pub use crate::error::Error;

pub fn dead() -> usize {
    0
}
"#,
    );
    write(
        root.join("provider/src/error.rs"),
        r#"#[derive(Debug)]
pub enum Error {
    Bad,
}
"#,
    );
}

fn write_build_dependency_public_reexport_fixture(root: &Path) {
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        r#"[workspace]
members = ["app", "bridge", "provider"]
resolver = "2"
"#,
    );
    write(
        root.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"
build = "build.rs"

[dependencies]
opensourced = {{ path = "{}" }}

[build-dependencies]
bridge = {{ path = "../bridge" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(root.join("app/build.rs"), "fn main() {}\n");
    write(
        root.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected() -> usize {
    7
}
"#,
    );
    write(
        root.join("bridge/Cargo.toml"),
        r#"[package]
name = "bridge"
version = "0.1.0"
edition = "2021"

[dependencies]
provider = { path = "../provider", optional = true }

[features]
default = []
with-provider = ["provider"]
"#,
    );
    write(
        root.join("bridge/src/lib.rs"),
        r#"#[cfg(feature = "with-provider")]
pub enum BridgeError {
    Provider(provider::Error),
}
"#,
    );
    write(
        root.join("provider/Cargo.toml"),
        r#"[package]
name = "provider"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        root.join("provider/src/lib.rs"),
        r#"mod error;
pub use crate::error::{Error};
"#,
    );
    write(
        root.join("provider/src/error.rs"),
        r#"#[derive(Debug)]
pub enum Error {
    Bad,
}
"#,
    );
}

fn write_macro_generated_public_reexport_fixture(root: &Path) {
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        r#"[workspace]
members = ["app", "macro-provider", "provider"]
resolver = "2"
"#,
    );
    write(
        root.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
provider = {{ path = "../provider" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );
    write(
        root.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected() -> Result<(), provider::Error> {
    unreachable!()
}
"#,
    );
    write(
        root.join("provider/Cargo.toml"),
        r#"[package]
name = "provider"
version = "0.1.0"
edition = "2021"

[dependencies]
macro-provider = { path = "../macro-provider" }
"#,
    );
    write(
        root.join("provider/src/lib.rs"),
        r#"mod error;
pub use crate::error::Error;
"#,
    );
    write(
        root.join("provider/src/error.rs"),
        r#"#[derive(Debug)]
pub enum ErrorKind {
    Bad,
}

macro_provider::define_error!(ErrorKind);
"#,
    );
    write(
        root.join("macro-provider/Cargo.toml"),
        r#"[package]
name = "macro-provider"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        root.join("macro-provider/src/lib.rs"),
        r#"#[macro_export]
macro_rules! define_error {
    ($kind:ident) => {
        #[derive(Debug)]
        pub struct Error(pub $kind);
    };
}
"#,
    );
}

fn write_support_proc_macro_scope_fixture(root: &Path, helper: &Path, macros: &Path) {
    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        r#"[workspace]
members = ["app"]
resolver = "2"
"#,
    );
    write(
        root.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
provider-helper = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path),
            manifest_path(helper)
        ),
    );
    write(
        root.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected() -> u32 {
    provider_helper::LiveRecord::live_generated()
}
"#,
    );
    write(
        helper.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "provider-helper"
version = "0.1.0"
edition = "2021"

[dependencies]
derive-helper-macros = {{ path = "{}" }}
"#,
            manifest_path(macros)
        ),
    );
    write(
        helper.join("src/lib.rs"),
        r#"mod helpers;

#[derive(derive_helper_macros::LiveDerive)]
pub struct LiveRecord;

#[derive(derive_helper_macros::DeadDerive)]
pub struct DeadRecord;
"#,
    );
    write(
        helper.join("src/helpers.rs"),
        r#"pub fn live_helper() -> u32 {
    7
}

pub fn dead_helper() -> u32 {
    13
}
"#,
    );
    write(
        macros.join("Cargo.toml"),
        r#"[package]
name = "derive-helper-macros"
version = "0.1.0"
edition = "2021"

[lib]
proc-macro = true

[dependencies]
proc-macro2 = "1"
quote = "1"
"#,
    );
    write(
        macros.join("src/lib.rs"),
        r#"extern crate proc_macro;

use proc_macro::TokenStream;

mod builders;
use builders::live_tokens as render_live;

#[proc_macro_derive(LiveDerive)]
pub fn live_derive(_input: TokenStream) -> TokenStream {
    let dead_tokens = proc_macro2::TokenStream::new();
    let _ = dead_tokens;
    render_live().into()
}

#[proc_macro_derive(DeadDerive)]
pub fn dead_derive(_input: TokenStream) -> TokenStream {
    builders::dead_tokens().into()
}
"#,
    );
    write(
        macros.join("src/builders.rs"),
        r#"use quote::quote;

pub fn live_tokens() -> proc_macro2::TokenStream {
    quote! {
        impl LiveRecord {
            pub fn live_generated() -> u32 {
                crate::helpers::live_helper()
            }
        }
    }
}

pub fn dead_tokens() -> proc_macro2::TokenStream {
    quote! {
        impl DeadRecord {
            pub fn dead_generated() -> u32 {
                crate::helpers::dead_helper()
            }
        }
    }
}
"#,
    );
}

fn write_package(root: &Path, name: &str, source: &str) {
    write(
        root.join(name).join("Cargo.toml"),
        &format!(
            r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"
"#
        ),
    );
    write(root.join(name).join("src/lib.rs"), source);
}

fn write_basic_workspace(root: &Path) {
    write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    );
}

fn write_basic_app_manifest(root: &Path) {
    write(
        root.join("app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&repo_root().join("crates/opensourced"))
        ),
    );
}

fn write_metadata_exclude_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    write(
        root.join("Cargo.toml"),
        r#"[workspace]
members = ["crates/*"]
exclude = ["crates/dead"]
resolver = "2"
"#,
    );
    write(
        root.join("crates/app/Cargo.toml"),
        &format!(
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&repo_root().join("crates/opensourced"))
        ),
    );
    write(
        root.join("crates/app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn selected() -> i32 {
    7
}
"#,
    );
    write(
        root.join("crates/dead/Cargo.toml"),
        r#"[not_a_package]
name = "dead"
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

#[cfg(unix)]
fn symlink_file(original: &Path, link: &Path) {
    if let Some(parent) = link.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    std::os::unix::fs::symlink(original, link).unwrap();
}

fn temp_path(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("opensourced-manifest-hardening-{label}-{unique}"))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn current_rustup_toolchain() -> String {
    std::env::var("RUSTUP_TOOLCHAIN").unwrap_or_else(|_| "stable".to_string())
}

fn manifest_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "\\\\")
}
