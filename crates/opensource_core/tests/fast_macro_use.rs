use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use opensource_core::{generate, GenerateOptions};

#[test]
fn slices_fast_macro_use_fixture_and_compiles() {
    let fixture = repo_root().join("fixtures/fast_macro_use");
    let output = temp_path("fast-macro-use-output");
    let target_dir = temp_path("fast-macro-use-target");

    let report = generate(GenerateOptions {
        workspace_root: fixture,
        output_root: output.clone(),
    })
    .expect("fast macro/use fixture should reduce");

    assert_eq!(
        report.packages,
        ["fast_macro_app", "macro_helpers", "shared", "util"]
    );

    let app = read(output.join("fast_macro_app/src/lib.rs"));
    assert!(app.contains("pub fn open_macro_use_entry"));
    assert!(app.contains("pub async fn open_async_macro_use"));
    assert!(app.contains("pub fn open_callback_bridge"));
    assert!(app.contains("FixtureDerive"));
    assert!(app.contains("FixtureRecord"));
    assert!(app.contains("FixtureEnum"));
    assert!(app.contains("FixtureObject"));
    assert!(app.contains("fixture_attr"));
    assert!(app.contains("fixture_export"));
    assert!(app.contains("fixture_constructor"));
    assert!(app.contains("use crate::generated_types"));
    assert!(app.contains("include!(\"generated.rs\")"));
    assert!(app.contains("mod inline_child"));
    assert!(app.contains("mod platform_bridge"));
    assert!(app.contains("mod reexports"));
    assert!(app.contains("reexported_nested"));
    assert!(app.contains("SharedMode"));
    assert!(app.contains("SharedAlias"));
    assert!(app.contains("FEATURE_FLAG"));
    assert!(app.contains("SHARED_STATIC"));
    assert!(app.contains("UtilValue::BONUS"));
    assert!(app.contains("impl From"));
    assert!(app.contains("for RootDto"));
    assert!(app.contains("pub struct WireDto"));
    assert!(app.contains("pub enum WireKind"));
    assert!(app.contains("pub struct BridgeObject"));
    assert!(app.contains("pub trait BridgeCallback"));
    assert!(app.contains("fn async_bridge_value"));
    assert!(!app.contains("dead_fn as selected_shadow"));
    assert!(!app.contains("dead_grouped as local_shadow"));
    assert!(!app.contains("dead_reexport"));
    assert!(!app.contains("dead_platform"));
    assert!(!app.contains("unused_macro"));
    assert!(!app.contains("dead_public_api"));

    let shared = read(output.join("shared/src/lib.rs"));
    assert!(shared.contains("pub type SharedAlias"));
    assert!(shared.contains("pub const FEATURE_FLAG"));
    assert!(shared.contains("pub static SHARED_STATIC"));
    assert!(shared.contains("pub enum SharedMode"));
    assert!(shared.contains("pub mod prelude"));
    assert!(shared.contains("pub fn fixture_value"));
    assert!(shared.contains("pub fn helper_marker"));
    assert!(!shared.contains("dead_shared"));
    assert!(!shared.contains("dead_nested"));
    assert!(!shared.contains("DeadEnum"));
    assert!(!shared.contains("dead_prelude"));

    let util = read(output.join("util/src/lib.rs"));
    assert!(util.contains("pub trait Useful"));
    assert!(util.contains("type Output"));
    assert!(util.contains("const BONUS"));
    assert!(util.contains("pub fn make_util"));
    assert!(!util.contains("dead_util"));

    let macro_helpers = read(output.join("macro_helpers/src/lib.rs"));
    assert!(macro_helpers.contains("proc_macro_derive(FixtureRecord"));
    assert!(macro_helpers.contains("proc_macro_derive(FixtureEnum"));
    assert!(macro_helpers.contains("proc_macro_derive(FixtureObject"));
    assert!(macro_helpers.contains("pub fn fixture_export"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated fast macro/use slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\napp/src/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        app,
    );
}

fn read(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path).expect("file should be readable")
}

fn temp_path(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("opensourced-{label}-{unique}"))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}
