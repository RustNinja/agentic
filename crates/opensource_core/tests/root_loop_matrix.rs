use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use opensource_core::{generate, GenerateOptions};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RootCase {
    FreeFunction,
    InherentMethod,
    TraitImplMethod,
    EnumItem,
    TraitItem,
    GenericFunction,
    MacroFunction,
    ReexportFunction,
    AssociatedConstFunction,
    NestedModuleFunction,
    ModuleItem,
}

impl RootCase {
    const ALL: [Self; 11] = [
        Self::FreeFunction,
        Self::InherentMethod,
        Self::TraitImplMethod,
        Self::EnumItem,
        Self::TraitItem,
        Self::GenericFunction,
        Self::MacroFunction,
        Self::ReexportFunction,
        Self::AssociatedConstFunction,
        Self::NestedModuleFunction,
        Self::ModuleItem,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::FreeFunction => "free-function",
            Self::InherentMethod => "inherent-method",
            Self::TraitImplMethod => "trait-impl-method",
            Self::EnumItem => "enum-item",
            Self::TraitItem => "trait-item",
            Self::GenericFunction => "generic-function",
            Self::MacroFunction => "macro-function",
            Self::ReexportFunction => "reexport-function",
            Self::AssociatedConstFunction => "associated-const-function",
            Self::NestedModuleFunction => "nested-module-function",
            Self::ModuleItem => "module-item",
        }
    }
}

#[test]
fn root_loop_slices_functions_methods_items_and_modules() {
    let mut summaries = Vec::new();

    for case in RootCase::ALL {
        let workspace = temp_path(&format!("loop-{}-workspace", case.label()));
        let output = temp_path(&format!("loop-{}-output", case.label()));
        let original_target = temp_path(&format!("loop-{}-original-target", case.label()));
        let sliced_target = temp_path(&format!("loop-{}-sliced-target", case.label()));
        write_fixture_workspace(&workspace, case);

        let original_check = cargo_check(&workspace, &original_target);
        assert!(
            original_check.status.success(),
            "original marked workspace failed for {}\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
            case.label(),
            original_check.status,
            String::from_utf8_lossy(&original_check.stdout),
            String::from_utf8_lossy(&original_check.stderr),
        );

        let report = generate(GenerateOptions {
            workspace_root: workspace.clone(),
            output_root: output.clone(),
        })
        .unwrap_or_else(|error| panic!("slice generation failed for {}: {error}", case.label()));

        let sliced_check = cargo_check(&output, &sliced_target);
        assert!(
            sliced_check.status.success(),
            "sliced workspace failed for {}\nroot: {}\npackages: {:?}\nreachable: {:?}\nitems: {:?}\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
            case.label(),
            report.root,
            report.packages,
            report.reachable,
            report.reachable_items,
            sliced_check.status,
            String::from_utf8_lossy(&sliced_check.stdout),
            String::from_utf8_lossy(&sliced_check.stderr),
        );

        let rendered_sources = rendered_rust_sources(&output);
        assert!(
            !rendered_sources.contains("#[opensourced]"),
            "marker attr leaked into slice for {}\n{}",
            case.label(),
            rendered_sources,
        );
        assert!(
            !rendered_sources.contains("opensourced::opensourced"),
            "marker import leaked into slice for {}\n{}",
            case.label(),
            rendered_sources,
        );
        assert!(
            !rendered_sources.contains("dead_"),
            "dead code leaked into slice for {}\n{}",
            case.label(),
            rendered_sources,
        );
        assert!(
            !output.join("noise").exists(),
            "unused dependency crate leaked into slice for {}",
            case.label(),
        );

        summaries.push(format!(
            "{} => root {}, packages {:?}, callables {}, items {}",
            case.label(),
            report.root,
            report.packages,
            report.reachable.len(),
            report.reachable_items.len()
        ));
    }

    eprintln!("root slicer loop:\n{}", summaries.join("\n"));
}

fn write_fixture_workspace(root: &Path, root_case: RootCase) {
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }

    let opensourced_path = repo_root().join("crates/opensourced");
    write(
        root.join("Cargo.toml"),
        r#"
[workspace]
members = ["app", "domain", "support", "noise"]
resolver = "2"

[workspace.package]
edition = "2021"
version = "0.1.0"
"#,
    );

    write(
        root.join("app/Cargo.toml"),
        &format!(
            r#"
[package]
name = "app"
version.workspace = true
edition.workspace = true

[dependencies]
domain = {{ path = "../domain" }}
support = {{ path = "../support" }}
noise = {{ path = "../noise" }}
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );

    write(
        root.join("app/src/lib.rs"),
        &format!(
            r#"
use domain::prelude::{{Payload, Status}};
use domain::{{make_payload, Multiplier, Runner, Scale}};
use opensourced::opensourced;
use support::SupportId;

macro_rules! build_payload {{
    ($value:expr) => {{
        domain::make_payload($value)
    }};
}}

pub struct Api {{
    seed: i32,
}}

impl Api {{
    pub fn new(seed: i32) -> Self {{
        Self {{ seed }}
    }}

    {}pub fn render(&self, value: i32) -> String {{
        let payload = make_payload(value + self.seed);
        domain::format_status(Status::Ready(payload, SupportId::new(1)))
    }}

    pub fn dead_method(&self) -> i32 {{
        noise::dead_noise()
    }}
}}

impl Runner for Api {{
    {}fn run(&self, payload: Payload) -> Status {{
        let value = payload.value + self.seed + <Multiplier as Scale>::FACTOR;
        Status::Ready(make_payload(value), SupportId::new(value as u64))
    }}
}}

{}pub fn export_report(value: i32) -> String {{
    let api = Api::new(value);
    api.render(2)
}}

{}pub fn export_generic<T: Scale>(item: T, value: i32) -> i32 {{
    let multiplier = Multiplier::new();
    item.scale(value) + multiplier.scale(value)
}}

{}pub fn export_macro(value: i32) -> i32 {{
    let payload = build_payload!(value);
    payload.value + support::normalize(value)
}}

{}pub fn export_reexport(value: i32) -> Status {{
    domain::prelude::start(value)
}}

{}pub fn export_associated_const(value: i32) -> i32 {{
    let multiplier = Multiplier::new();
    multiplier.scale(value) + <Multiplier as Scale>::FACTOR
}}

pub mod nested {{
    use super::*;

    {}pub fn entry(value: i32) -> String {{
        domain::deep::inner::describe(value)
    }}

    pub fn dead_nested() -> i32 {{
        noise::dead_noise()
    }}
}}

{}pub mod module_root {{
    pub fn alpha(value: i32) -> String {{
        domain::deep::inner::describe(value)
    }}

    pub fn beta(value: i32) -> i32 {{
        domain::make_payload(value).value
    }}
}}

pub fn dead_app_function() -> i32 {{
    noise::dead_noise()
}}
"#,
            attr(root_case, RootCase::InherentMethod),
            attr(root_case, RootCase::TraitImplMethod),
            attr(root_case, RootCase::FreeFunction),
            attr(root_case, RootCase::GenericFunction),
            attr(root_case, RootCase::MacroFunction),
            attr(root_case, RootCase::ReexportFunction),
            attr(root_case, RootCase::AssociatedConstFunction),
            attr(root_case, RootCase::NestedModuleFunction),
            attr(root_case, RootCase::ModuleItem),
        ),
    );

    write(
        root.join("domain/Cargo.toml"),
        &format!(
            r#"
[package]
name = "domain"
version.workspace = true
edition.workspace = true

[dependencies]
support = {{ path = "../support" }}
noise = {{ path = "../noise" }}
opensourced = {{ path = "{}" }}
"#,
            manifest_path(&opensourced_path)
        ),
    );

    write(
        root.join("domain/src/lib.rs"),
        &format!(
            r#"
use opensourced::opensourced;
use support::{{describe_id, normalize, SupportId}};

pub mod prelude {{
    pub use crate::{{Payload, Runner, Status}};

    pub fn start(value: i32) -> Status {{
        crate::Status::Ready(crate::make_payload(value), support::SupportId::new(value as u64))
    }}

    pub fn dead_prelude() -> i32 {{
        noise::dead_noise()
    }}
}}

pub mod deep {{
    pub mod inner {{
        pub fn describe(value: i32) -> String {{
            crate::format_status(crate::prelude::start(value))
        }}

        pub fn dead_inner() -> i32 {{
            noise::dead_noise()
        }}
    }}
}}

#[derive(Clone)]
pub struct Payload {{
    pub value: i32,
    pub id: SupportId,
}}

{}#[derive(Clone)]
pub enum Status {{
    Ready(Payload, SupportId),
    Failed {{ id: SupportId, message: String }},
}}

{}pub trait Runner {{
    fn run(&self, payload: Payload) -> Status;

    fn label(&self, payload: Payload) -> String {{
        describe_id(&payload.id)
    }}
}}

pub trait Scale {{
    const FACTOR: i32;

    fn scale(&self, value: i32) -> i32;
}}

pub struct Multiplier;

impl Multiplier {{
    pub fn new() -> Self {{
        Self
    }}

    pub fn dead_multiplier() -> Self {{
        Self
    }}
}}

impl Scale for Multiplier {{
    const FACTOR: i32 = 3;

    fn scale(&self, value: i32) -> i32 {{
        normalize(value) * Self::FACTOR
    }}
}}

pub fn make_payload(value: i32) -> Payload {{
    Payload {{
        value: normalize(value),
        id: SupportId::new(value as u64),
    }}
}}

pub fn format_status(status: Status) -> String {{
    match status {{
        Status::Ready(payload, id) => format!("{{}}:{{}}", describe_id(&payload.id), id.value()),
        Status::Failed {{ id, message }} => format!("{{}}:{{}}", describe_id(&id), message),
    }}
}}

pub fn dead_domain_function() -> i32 {{
    noise::dead_noise()
}}
"#,
            attr(root_case, RootCase::EnumItem),
            attr(root_case, RootCase::TraitItem),
        ),
    );

    write(
        root.join("support/Cargo.toml"),
        r#"
[package]
name = "support"
version.workspace = true
edition.workspace = true
"#,
    );

    write(
        root.join("support/src/lib.rs"),
        r#"
#[derive(Clone)]
pub struct SupportId(u64);

impl SupportId {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn value(&self) -> u64 {
        self.0
    }

    pub fn dead_id() -> Self {
        Self(999)
    }
}

pub fn normalize(value: i32) -> i32 {
    value.max(0)
}

pub fn describe_id(id: &SupportId) -> String {
    format!("id-{}", id.value())
}

pub fn dead_support_function() -> i32 {
    99
}
"#,
    );

    write(
        root.join("noise/Cargo.toml"),
        r#"
[package]
name = "noise"
version.workspace = true
edition.workspace = true
"#,
    );

    write(
        root.join("noise/src/lib.rs"),
        r#"
pub fn dead_noise() -> i32 {
    42
}
"#,
    );
}

fn attr(actual: RootCase, expected: RootCase) -> &'static str {
    if actual == expected {
        "#[opensourced]\n"
    } else {
        ""
    }
}

fn cargo_check(workspace: &Path, target_dir: &Path) -> std::process::Output {
    Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(workspace)
        .env("CARGO_TARGET_DIR", target_dir)
        .output()
        .expect("cargo check should start")
}

fn rendered_rust_sources(root: &Path) -> String {
    let mut paths = Vec::new();
    collect_rust_paths(root, &mut paths);
    paths.sort();
    paths
        .into_iter()
        .map(|path| fs::read_to_string(path).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
}

fn collect_rust_paths(root: &Path, paths: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.file_name().is_some_and(|name| name == "target") {
            continue;
        }
        if path.is_dir() {
            collect_rust_paths(&path, paths);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            paths.push(path);
        }
    }
}

fn write(path: impl AsRef<Path>, contents: &str) {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents.trim_start()).unwrap();
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("opensource_core should live under crates/opensource_core")
        .to_path_buf()
}

fn manifest_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "\\\\")
}

fn temp_path(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("opensourced-root-loop-{label}-{unique}"))
}
