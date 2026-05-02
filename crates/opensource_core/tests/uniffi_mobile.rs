use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use opensource_core::{generate, GenerateOptions};

#[test]
fn slices_uniffi_shaped_mobile_bridge_workspace() {
    let workspace = temp_path("workspace");
    let output = temp_path("output");
    let target_dir = temp_path("target");
    write_fixture_workspace(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace.clone(),
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    assert_eq!(
        report.packages,
        ["crypto_box", "mobile_api", "sync_core"],
        "unused local crates should not be emitted"
    );

    let reachable = report
        .reachable
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    for expected in [
        "crypto_box::<Hasher as Signing>::sign",
        "crypto_box::Hasher::digest",
        "crypto_box::Hasher::new",
        "crypto_box::encode",
        "crypto_box::verify",
        "mobile_api::SyncResponse::from_core",
        "mobile_api::sync_preview",
        "sync_core::<SyncEngine as Planner>::plan",
        "sync_core::AccessToken::as_str",
        "sync_core::AccessToken::new",
        "sync_core::SyncEngine::build_result",
        "sync_core::SyncEngine::new",
        "sync_core::SyncEngine::preview_fast",
        "sync_core::SyncEngine::preview_full",
        "sync_core::render_core_summary",
        "sync_core::render_summary",
    ] {
        assert!(
            reachable.iter().any(|actual| actual == expected),
            "missing reachable callable {expected}; got {reachable:?}",
        );
    }

    for not_expected in [
        "diagnostics::DebugSink::flush",
        "diagnostics::debug_label",
        "mobile_api::debug_only",
        "mobile_api::SyncResponse::diagnostic_only",
        "sync_core::SyncEngine::unreachable_upload",
        "sync_core::unused_core_api",
        "crypto_box::unused_secret",
    ] {
        assert!(
            !reachable.iter().any(|actual| actual == not_expected),
            "unreachable callable {not_expected} was retained in graph: {reachable:?}",
        );
    }

    let reachable_items = report
        .reachable_items
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    for expected in [
        "crypto_box::DIGEST_SALT(Const)",
        "crypto_box::Digest(Struct)",
        "crypto_box::Hasher(Struct)",
        "crypto_box::Signing(Trait)",
        "mobile_api::SyncRequest(Struct)",
        "mobile_api::SyncResponse(Struct)",
        "mobile_api::SyncStatus(Enum)",
        "sync_core::AccessToken(Struct)",
        "sync_core::Outcome(Enum)",
        "sync_core::Planner(Trait)",
        "sync_core::SyncEngine(Struct)",
        "sync_core::SyncResult(Struct)",
    ] {
        assert!(
            reachable_items.iter().any(|actual| actual == expected),
            "missing reachable item {expected}; got {reachable_items:?}",
        );
    }

    for not_expected in [
        "diagnostics::DebugSink(Struct)",
        "mobile_api::InternalDebugRecord(Struct)",
        "sync_core::UnusedCoreRecord(Struct)",
        "crypto_box::UnusedCryptoEnum(Enum)",
    ] {
        assert!(
            !reachable_items.iter().any(|actual| actual == not_expected),
            "unreachable item {not_expected} was retained in graph: {reachable_items:?}",
        );
    }

    assert!(
        !output.join("diagnostics").exists(),
        "unused local diagnostics crate should not be written"
    );

    let root_manifest = read(output.join("Cargo.toml"));
    assert!(root_manifest.contains("\"mobile_api\""));
    assert!(root_manifest.contains("\"sync_core\""));
    assert!(root_manifest.contains("\"crypto_box\""));
    assert!(!root_manifest.contains("\"diagnostics\""));
    assert!(root_manifest.contains("serde"));
    assert!(root_manifest.contains("derive"));

    let mobile_manifest = read(output.join("mobile_api/Cargo.toml"));
    assert!(mobile_manifest.contains("sync_core"));
    assert!(mobile_manifest.contains("serde"));
    assert!(!mobile_manifest.contains("diagnostics"));
    assert!(!mobile_manifest.contains("opensourced"));

    let mobile_source = read(output.join("mobile_api/src/lib.rs"));
    for expected in [
        "cfg_attr(feature = \"ffi\", uniffi::export)",
        "cfg_attr(feature = \"ffi\", derive(uniffi::Record))",
        "pub struct SyncRequest",
        "pub enum SyncStatus",
        "pub struct SyncResponse",
        "pub fn sync_preview",
        "pub fn from_core",
    ] {
        assert!(
            mobile_source.contains(expected),
            "missing `{expected}` in reduced mobile source:\n{mobile_source}",
        );
    }
    for not_expected in [
        "#[opensourced]",
        "diagnostics",
        "debug_only",
        "diagnostic_only",
        "InternalDebugRecord",
        "mobile_api_unit_test",
        "cfg(test)",
    ] {
        assert!(
            !mobile_source.contains(not_expected),
            "unexpected `{not_expected}` in reduced mobile source:\n{mobile_source}",
        );
    }

    let sync_source = read(output.join("sync_core/src/lib.rs"));
    for expected in [
        "pub struct AccessToken",
        "pub struct SyncEngine",
        "pub struct SyncResult",
        "pub enum Outcome",
        "pub trait Planner",
        "impl Planner for SyncEngine",
        "pub fn render_summary",
        "fn render_core_summary",
    ] {
        assert!(
            sync_source.contains(expected),
            "missing `{expected}` in reduced sync source:\n{sync_source}",
        );
    }
    for not_expected in [
        "unreachable_upload",
        "unused_core_api",
        "UnusedCoreRecord",
        "sync_core_unit_test",
        "cfg(test)",
    ] {
        assert!(
            !sync_source.contains(not_expected),
            "unexpected `{not_expected}` in reduced sync source:\n{sync_source}",
        );
    }

    let crypto_source = read(output.join("crypto_box/src/lib.rs"));
    for expected in [
        "pub const DIGEST_SALT",
        "pub struct Digest",
        "pub struct Hasher",
        "pub trait Signing",
        "impl Signing for Hasher",
        "pub fn verify",
        "pub fn encode",
    ] {
        assert!(
            crypto_source.contains(expected),
            "missing `{expected}` in reduced crypto source:\n{crypto_source}",
        );
    }
    for not_expected in [
        "unused_secret",
        "UnusedCryptoEnum",
        "crypto_box_unit_test",
        "cfg(test)",
    ] {
        assert!(
            !crypto_source.contains(not_expected),
            "unexpected `{not_expected}` in reduced crypto source:\n{crypto_source}",
        );
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
        "generated workspace did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\nmobile_api/src/lib.rs:\n{}\nsync_core/src/lib.rs:\n{}\ncrypto_box/src/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        mobile_source,
        sync_source,
        crypto_source,
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
members = ["mobile_api", "sync_core", "crypto_box", "diagnostics"]
resolver = "2"

[workspace.package]
edition = "2021"
version = "0.1.0"
license = "MIT"

[workspace.dependencies]
opensourced = {{ path = "{}" }}
serde = {{ version = "1.0", features = ["derive"] }}
"#,
            manifest_path(&opensourced_path)
        ),
    );

    write(
        root.join("mobile_api/Cargo.toml"),
        r#"[package]
name = "mobile_api"
version.workspace = true
edition.workspace = true
license.workspace = true

[features]
ffi = []

[dependencies]
diagnostics = { path = "../diagnostics" }
opensourced.workspace = true
serde.workspace = true
sync_core = { path = "../sync_core" }
"#,
    );
    write(
        root.join("mobile_api/src/lib.rs"),
        r#"use diagnostics::DebugSink;
use opensourced::opensourced;
use serde::{Deserialize, Serialize};
use sync_core::{AccessToken, SyncEngine, SyncResult};

#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncRequest {
    pub user_id: String,
    pub token: String,
    pub fast: bool,
}

#[cfg_attr(feature = "ffi", derive(uniffi::Enum))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SyncStatus {
    Complete,
    Partial { retry_after_secs: u32 },
}

#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncResponse {
    pub user_id: String,
    pub summary: String,
    pub status: SyncStatus,
}

#[opensourced]
#[cfg_attr(feature = "ffi", uniffi::export)]
pub fn sync_preview(request: SyncRequest) -> SyncResponse {
    let token = AccessToken::new(request.token);
    let engine = SyncEngine::new(request.user_id, token);
    let result = if request.fast {
        engine.preview_fast()
    } else {
        engine.preview_full()
    };

    SyncResponse::from_core(result)
}

impl SyncResponse {
    pub fn from_core(result: SyncResult) -> Self {
        let summary = sync_core::render_summary(&result);
        let status = match result.outcome {
            sync_core::Outcome::Success => SyncStatus::Complete,
            sync_core::Outcome::NeedsRetry(retry_after_secs) => {
                SyncStatus::Partial { retry_after_secs }
            }
            sync_core::Outcome::Blocked => SyncStatus::Partial {
                retry_after_secs: 60,
            },
        };

        Self {
            user_id: result.user_id,
            summary,
            status,
        }
    }

    pub fn diagnostic_only() -> Self {
        let sink = DebugSink::new();
        Self {
            user_id: "debug".to_string(),
            summary: sink.flush(),
            status: SyncStatus::Partial {
                retry_after_secs: 1,
            },
        }
    }
}

pub fn debug_only() -> InternalDebugRecord {
    InternalDebugRecord {
        label: diagnostics::debug_label(),
    }
}

pub struct InternalDebugRecord {
    pub label: String,
}

#[test]
fn mobile_api_unit_test() {
    assert_eq!(1, 1);
}

#[cfg(test)]
mod tests {
    #[test]
    fn nested_mobile_api_unit_test() {
        assert_eq!(2, 2);
    }
}
"#,
    );

    write(
        root.join("sync_core/Cargo.toml"),
        r#"[package]
name = "sync_core"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
crypto_box = { path = "../crypto_box" }
serde.workspace = true
"#,
    );
    write(
        root.join("sync_core/src/lib.rs"),
        r#"use crypto_box::{Digest, Hasher, Signing};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccessToken(String);

impl AccessToken {
    pub fn new(raw: String) -> Self {
        Self(raw)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn unreachable_masked() -> String {
        "masked".to_string()
    }
}

pub struct SyncEngine {
    user_id: String,
    token: AccessToken,
    hasher: Hasher,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncResult {
    pub user_id: String,
    pub summary: String,
    pub outcome: Outcome,
    pub digest: Digest,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Outcome {
    Success,
    NeedsRetry(u32),
    Blocked,
}

pub trait Planner {
    fn plan(&self, fast: bool) -> Outcome;
}

impl SyncEngine {
    pub fn new(user_id: String, token: AccessToken) -> Self {
        Self {
            user_id,
            token,
            hasher: Hasher::new(),
        }
    }

    pub fn preview_fast(&self) -> SyncResult {
        self.build_result(true)
    }

    pub fn preview_full(&self) -> SyncResult {
        self.build_result(false)
    }

    fn build_result(&self, fast: bool) -> SyncResult {
        let outcome = <Self as Planner>::plan(self, fast);
        let digest = Hasher::digest(&self.hasher, AccessToken::as_str(&self.token));
        SyncResult {
            user_id: self.user_id.clone(),
            summary: render_core_summary(&outcome),
            outcome,
            digest,
        }
    }

    pub fn unreachable_upload(&self) -> String {
        crypto_box::unused_secret()
    }
}

impl Planner for SyncEngine {
    fn plan(&self, fast: bool) -> Outcome {
        if !crypto_box::verify(AccessToken::as_str(&self.token)) {
            return Outcome::Blocked;
        }
        if fast {
            Outcome::Success
        } else {
            Outcome::NeedsRetry(30)
        }
    }
}

pub fn render_summary(result: &SyncResult) -> String {
    let hasher = Hasher::new();
    let signature = Signing::sign(&hasher, &result.digest);
    format!("{}:{}", result.summary, signature)
}

fn render_core_summary(outcome: &Outcome) -> String {
    match outcome {
        Outcome::Success => "complete".to_string(),
        Outcome::NeedsRetry(seconds) => format!("retry-{seconds}"),
        Outcome::Blocked => "blocked".to_string(),
    }
}

pub fn unused_core_api() -> UnusedCoreRecord {
    UnusedCoreRecord {
        value: crypto_box::unused_secret(),
    }
}

pub struct UnusedCoreRecord {
    pub value: String,
}

#[test]
fn sync_core_unit_test() {
    assert_eq!(3, 3);
}

#[cfg(test)]
mod tests {
    #[test]
    fn nested_sync_core_unit_test() {
        assert_eq!(4, 4);
    }
}
"#,
    );

    write(
        root.join("crypto_box/Cargo.toml"),
        r#"[package]
name = "crypto_box"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
serde.workspace = true
"#,
    );
    write(
        root.join("crypto_box/src/lib.rs"),
        r#"use serde::{Deserialize, Serialize};

pub const DIGEST_SALT: &str = "agentic-sg";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Digest {
    pub value: String,
}

pub struct Hasher {
    salt: &'static str,
}

impl Hasher {
    pub fn new() -> Self {
        Self { salt: DIGEST_SALT }
    }

    pub fn digest(&self, input: &str) -> Digest {
        Digest {
            value: format!("{}:{input}", self.salt),
        }
    }

    pub fn unused_digest(&self) -> Digest {
        Digest {
            value: "unused".to_string(),
        }
    }
}

pub trait Signing {
    fn sign(&self, digest: &Digest) -> String;
}

impl Signing for Hasher {
    fn sign(&self, digest: &Digest) -> String {
        encode(digest)
    }
}

pub fn verify(raw: &str) -> bool {
    !raw.trim().is_empty()
}

pub fn encode(digest: &Digest) -> String {
    format!("signed-{}", digest.value)
}

pub fn unused_secret() -> String {
    "unused-secret".to_string()
}

pub enum UnusedCryptoEnum {
    Never,
}

#[test]
fn crypto_box_unit_test() {
    assert_eq!(5, 5);
}

#[cfg(test)]
mod tests {
    #[test]
    fn nested_crypto_box_unit_test() {
        assert_eq!(6, 6);
    }
}
"#,
    );

    write(
        root.join("diagnostics/Cargo.toml"),
        r#"[package]
name = "diagnostics"
version.workspace = true
edition.workspace = true
license.workspace = true
"#,
    );
    write(
        root.join("diagnostics/src/lib.rs"),
        r#"pub struct DebugSink;

impl DebugSink {
    pub fn new() -> Self {
        Self
    }

    pub fn flush(&self) -> String {
        "debug".to_string()
    }
}

pub fn debug_label() -> String {
    "debug-label".to_string()
}
"#,
    );
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("core crate should live under crates/opensource_core")
        .to_path_buf()
}

fn temp_path(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("opensource-core-uniffi-mobile-{label}-{unique}"))
}

fn manifest_path(path: &Path) -> String {
    path.display().to_string().replace('\\', "\\\\")
}

fn read(path: PathBuf) -> String {
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

fn write(path: PathBuf, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&path, contents)
        .unwrap_or_else(|error| panic!("failed to write {}: {error}", path.display()));
}
