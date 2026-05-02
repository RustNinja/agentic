use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use opensource_core::{generate, GenerateOptions};

#[test]
fn slices_generics_traits_impls_modules_reexports_macros_and_unions() {
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
        ["api", "domain", "support"],
        "unused local workspace crates should not be emitted"
    );

    let reachable = report
        .reachable
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    for expected in [
        "api::export",
        "domain::details::decorate",
        "domain::details::hidden_macro_helper",
        "domain::format_report",
        "domain::service::Service::new",
        "domain::service::Service::run",
        "support::<Auditor as Audit>::audit",
        "support::<Auditor as Audit>::explain",
        "support::AuditRecord::new",
        "support::Auditor::new",
        "support::helper_explain",
        "support::normalize",
    ] {
        assert!(
            reachable.iter().any(|actual| actual == expected),
            "missing reachable callable {expected}; got {reachable:?}",
        );
    }

    for not_expected in [
        "api::unused_api",
        "domain::details::unused_detail",
        "domain::service::Service::unused",
        "domain::unused_domain_function",
        "support::unused_support",
        "unused_local::dead_function",
        "unused_local::DeadThing::new",
    ] {
        assert!(
            !reachable.iter().any(|actual| actual == not_expected),
            "unreachable callable {not_expected} was retained: {reachable:?}",
        );
    }

    let reachable_items = report
        .reachable_items
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    for expected in [
        "domain::DomainError(Enum)",
        "domain::details::wrap_label(Macro)",
        "domain::model::Count(Type)",
        "domain::model::Envelope(Struct)",
        "domain::model::Mode(Enum)",
        "domain::model::NAME(Static)",
        "domain::model::RawBits(Union)",
        "domain::model::Report(Struct)",
        "domain::model::LIMIT(Const)",
        "domain::service::Service(Struct)",
        "support::Audit(Trait)",
        "support::AuditRecord(Struct)",
        "support::Auditor(Struct)",
        "support::SUPPORT_OFFSET(Const)",
        "support::SupportScore(Struct)",
    ] {
        assert!(
            reachable_items.iter().any(|actual| actual == expected),
            "missing reachable item {expected}; got {reachable_items:?}",
        );
    }

    for not_expected in [
        "domain::details::unused_macro(Macro)",
        "domain::model::DeadMode(Enum)",
        "domain::service::UnusedService(Struct)",
        "support::UnusedSupport(Enum)",
        "unused_local::DeadThing(Struct)",
    ] {
        assert!(
            !reachable_items.iter().any(|actual| actual == not_expected),
            "unreachable item {not_expected} was retained: {reachable_items:?}",
        );
    }

    assert!(
        !output.join("unused_local").exists(),
        "unused local crate should not be written"
    );

    let api_source = read(output.join("api/src/lib.rs"));
    assert!(api_source.contains("pub async fn export"));
    assert!(api_source.contains("use domain::prelude::{Envelope, Mode};"));
    assert!(!api_source.contains("DeadMode"));
    assert!(!api_source.contains("unused_local"));
    assert!(!api_source.contains("unused_api"));
    assert!(!api_source.contains("#[opensourced]"));

    let root_manifest = read(output.join("Cargo.toml"));
    let api_manifest = read(output.join("api/Cargo.toml"));
    assert!(
        !root_manifest.contains("serde"),
        "unused workspace external dependency should be pruned:\n{root_manifest}"
    );
    assert!(
        !api_manifest.contains("serde"),
        "unused package external dependency should be pruned:\n{api_manifest}"
    );

    let domain_lib = read(output.join("domain/src/lib.rs"));
    assert!(domain_lib.contains("pub mod prelude"));
    assert!(domain_lib.contains("pub use crate::model::{Envelope, Mode};"));
    assert!(domain_lib.contains("pub use model::{Envelope, Mode, Report};"));
    assert!(domain_lib.contains("pub use service::Service;"));
    assert!(!domain_lib.contains("DeadMode"));
    assert!(!domain_lib.contains("UnusedService"));
    assert!(!domain_lib.contains("unused_domain_function"));

    let domain_model = read(output.join("domain/src/model.rs"));
    for expected in [
        "pub struct Envelope",
        "pub enum Mode",
        "pub type Count",
        "pub const LIMIT",
        "pub static NAME",
        "pub union RawBits",
        "pub struct Report",
    ] {
        assert!(
            domain_model.contains(expected),
            "missing `{expected}` in reduced model:\n{domain_model}",
        );
    }
    assert!(!domain_model.contains("DeadMode"));

    let domain_service = read(output.join("domain/src/service.rs"));
    assert!(domain_service.contains("pub struct Service"));
    assert!(domain_service.contains("pub async fn run"));
    assert!(domain_service.contains("<Auditor as Audit>::audit"));
    assert!(!domain_service.contains("UnusedService"));
    assert!(!domain_service.contains("unused"));

    let domain_details = read(output.join("domain/src/details.rs"));
    assert!(domain_details.contains("macro_rules! wrap_label"));
    assert!(domain_details.contains("pub fn decorate"));
    assert!(domain_details.contains("fn hidden_macro_helper"));
    assert!(!domain_details.contains("unused_macro"));
    assert!(!domain_details.contains("unused_detail"));

    let support_source = read(output.join("support/src/lib.rs"));
    for expected in [
        "pub trait Audit",
        "type Output = SupportScore",
        "const OFFSET",
        "fn audit",
        "fn explain",
        "fn helper_explain",
    ] {
        assert!(
            support_source.contains(expected),
            "missing `{expected}` in reduced support:\n{support_source}",
        );
    }
    assert!(!support_source.contains("unused_support"));
    assert!(!support_source.contains("UnusedSupport"));

    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");

    assert!(
        cargo_check.status.success(),
        "generated workspace did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\napi/src/lib.rs:\n{}\ndomain/src/lib.rs:\n{}\ndomain/src/service.rs:\n{}\nsupport/src/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        api_source,
        domain_lib,
        domain_service,
        support_source,
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
members = ["crates/*"]
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
        root.join("crates/api/Cargo.toml"),
        r#"[package]
name = "api"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
domain = { path = "../domain" }
opensourced.workspace = true
serde.workspace = true
unused_local = { path = "../unused_local" }
"#,
    );
    write(
        root.join("crates/api/src/lib.rs"),
        r#"use domain::prelude::{DeadMode, Envelope, Mode};
use opensourced::opensourced;
use unused_local::{dead_function, DeadThing};

#[opensourced]
pub async fn export(input: Envelope<u32>, mode: Mode) -> Result<String, domain::DomainError> {
    let service = domain::Service::new(input);
    let report = service.run(mode).await?;
    Ok(domain::format_report(report))
}

pub fn unused_api() -> String {
    let thing = DeadThing::new();
    format!("{}:{:?}", dead_function(), thing.value)
}
"#,
    );

    write(
        root.join("crates/domain/Cargo.toml"),
        r#"[package]
name = "domain"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
support = { path = "../support" }
"#,
    );
    write(
        root.join("crates/domain/src/lib.rs"),
        r#"pub mod model;
pub mod service;
mod details;

pub mod prelude {
    pub use crate::model::{DeadMode, Envelope, Mode};
}

pub use model::{DeadMode, Envelope, Mode, Report};
pub use service::{Service, UnusedService};

#[derive(Debug)]
pub enum DomainError {
    Blocked,
}

pub fn format_report(report: Report) -> String {
    let raw = format!("{}:{}:{:?}", report.name, report.count, report.mode);
    details::decorate(&raw)
}

pub fn unused_domain_function() -> String {
    details::unused_detail()
}

#[cfg(test)]
mod tests {
    #[test]
    fn domain_test_is_not_sliced() {
        assert_eq!(1, 1);
    }
}
"#,
    );
    write(
        root.join("crates/domain/src/model.rs"),
        r#"#[derive(Clone, Copy)]
pub struct Envelope<T> {
    pub payload: T,
    pub marker: Mode,
}

#[derive(Clone, Copy, Debug)]
pub enum Mode {
    Fast,
    Full,
    Offline,
}

pub enum DeadMode {
    Never,
}

pub type Count = u32;
pub const LIMIT: Count = 7;
pub static NAME: &str = "matrix";

pub union RawBits {
    pub count: u32,
    pub bytes: [u8; 4],
}

pub struct Report {
    pub mode: Mode,
    pub count: Count,
    pub bits: RawBits,
    pub name: &'static str,
}
"#,
    );
    write(
        root.join("crates/domain/src/service.rs"),
        r#"use crate::model::{Count, Envelope, Mode, RawBits, Report, LIMIT, NAME};
use support::{Audit, AuditRecord, Auditor};

pub struct Service<T> {
    envelope: Envelope<T>,
    auditor: Auditor,
}

impl<T> Service<T>
where
    T: Copy + Into<u32>,
{
    pub fn new(envelope: Envelope<T>) -> Self {
        Self {
            envelope,
            auditor: Auditor::new(),
        }
    }

    pub async fn run(&self, mode: Mode) -> Result<Report, crate::DomainError> {
        let record = AuditRecord::new(self.envelope.payload.into());
        let score = <Auditor as Audit>::audit(&self.auditor, record);
        let count: Count = support::normalize(score.0) + LIMIT;
        Ok(Report {
            mode,
            count,
            bits: RawBits { count },
            name: NAME,
        })
    }

    pub fn unused(&self) -> String {
        support::unused_support()
    }
}

pub struct UnusedService;
"#,
    );
    write(
        root.join("crates/domain/src/details.rs"),
        r#"macro_rules! wrap_label {
    ($value:expr) => {
        format!("report:{}", hidden_macro_helper($value))
    };
}

macro_rules! unused_macro {
    () => {
        "unused"
    };
}

pub fn decorate(value: &str) -> String {
    wrap_label!(value)
}

fn hidden_macro_helper(value: &str) -> String {
    value.trim().to_string()
}

pub fn unused_detail() -> String {
    unused_macro!().to_string()
}
"#,
    );

    write(
        root.join("crates/support/Cargo.toml"),
        r#"[package]
name = "support"
version.workspace = true
edition.workspace = true
license.workspace = true
"#,
    );
    write(
        root.join("crates/support/src/lib.rs"),
        r#"pub const SUPPORT_OFFSET: u32 = 11;

pub struct Auditor;

impl Auditor {
    pub fn new() -> Self {
        Self
    }
}

pub struct AuditRecord {
    value: u32,
}

impl AuditRecord {
    pub fn new(value: u32) -> Self {
        Self { value }
    }
}

pub struct SupportScore(pub u32);

pub trait Audit {
    type Output;
    const OFFSET: u32;

    fn audit(&self, record: AuditRecord) -> Self::Output;
    fn explain(&self) -> String;
}

impl Audit for Auditor {
    type Output = SupportScore;
    const OFFSET: u32 = SUPPORT_OFFSET;

    fn audit(&self, record: AuditRecord) -> Self::Output {
        SupportScore(normalize(record.value) + Self::OFFSET)
    }

    fn explain(&self) -> String {
        helper_explain()
    }
}

pub fn normalize(value: u32) -> u32 {
    value + 1
}

fn helper_explain() -> String {
    "compile-required-trait-peer".to_string()
}

pub fn unused_support() -> String {
    "unused support".to_string()
}

pub enum UnusedSupport {
    Never,
}
"#,
    );

    write(
        root.join("crates/unused_local/Cargo.toml"),
        r#"[package]
name = "unused_local"
version.workspace = true
edition.workspace = true
license.workspace = true
"#,
    );
    write(
        root.join("crates/unused_local/src/lib.rs"),
        r#"#[derive(Debug)]
pub struct DeadThing {
    pub value: u32,
}

impl DeadThing {
    pub fn new() -> Self {
        Self { value: 0 }
    }
}

pub fn dead_function() -> String {
    "dead".to_string()
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
    std::env::temp_dir().join(format!("opensource-core-component-matrix-{label}-{unique}"))
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
