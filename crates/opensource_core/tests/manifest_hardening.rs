use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use opensource_core::{generate, GenerateOptions};

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
    assert!(!lib.contains("UNUSED"));
    assert!(output
        .join("include_assets_like/src/guidelines/core.md")
        .exists());
    assert!(output.join("include_assets_like/assets/extra.txt").exists());
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
fn retains_mutex_guard_field_method_and_associated_const_imports() {
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
    assert!(source.contains("Duration"));
    assert!(source.contains("fn push"));
    assert!(source.contains("const INTERVAL"));
    assert!(source.contains("Weak"));
    assert!(source.contains("SqlInterruptHandle"));

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
    assert!(root_manifest.contains("external_helper"));
    assert!(root_manifest.contains("path = \"/"));
    assert!(!root_manifest.contains("path = \"../"));
    assert!(root_manifest.contains("[patch.crates-io.external-helper]"));
    assert!(app_manifest.contains("external_helper"));
    assert!(lockfile.contains("version = 3"));

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

fn write_external_workspace_path_fixture(root: &Path) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
    let external_name = format!(
        "{}-external-helper",
        root.file_name().unwrap().to_string_lossy()
    );
    let external_root = root.parent().unwrap().join(&external_name);
    if external_root.exists() {
        fs::remove_dir_all(&external_root).unwrap();
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
"#,
    );
    write(
        external_root.join("src/lib.rs"),
        r#"pub fn decorate(value: &str) -> String {
    format!("external:{value}")
}

pub fn unused() -> &'static str {
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
        r#"use std::time::{Duration, SystemTime, UNIX_EPOCH};

use opensourced::opensourced;

#[derive(Copy, Clone)]
pub struct Timestamp(pub u64);

impl Timestamp {
    #[opensourced]
    pub fn checked_sub(self, d: Duration) -> Option<Timestamp> {
        SystemTime::from(self).checked_sub(d).map(Timestamp::from)
    }
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
const UNUSED: &str = include_str!("guidelines/unused.md");

#[opensourced]
pub fn selected(include_extra: bool) -> String {
    if include_extra {
        format!("{CORE}\n{EXTRA}")
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
        root.join("include_assets_like/src/guidelines/unused.md"),
        "unused guideline",
    );
    write(root.join("include_assets_like/assets/extra.txt"), "extra");
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

fn manifest_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "\\\\")
}
