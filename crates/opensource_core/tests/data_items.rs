use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use opensource_core::{generate, GenerateOptions};

#[test]
fn keeps_reachable_data_items_and_prunes_dead_code() {
    let fixture = temp_dir("data-items-fixture");
    let output = temp_dir("data-items-output");
    let target = temp_dir("data-items-target");
    write_fixture(&fixture);

    let report = generate(GenerateOptions {
        workspace_root: fixture.clone(),
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let reachable = report
        .reachable
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    for expected in [
        "app::export",
        "helper::HelperData::amount",
        "helper::HelperData::new",
        "helper::finalize",
        "helper::helper_checksum",
        "model::Model::new",
        "model::Model::score",
        "model::build_model",
    ] {
        assert!(
            reachable.iter().any(|actual| actual == expected),
            "missing reachable callable {expected}; got {reachable:?}",
        );
    }
    for not_expected in [
        "app::unreachable_app_fn",
        "helper::HelperData::unreachable_method",
        "helper::unreachable_helper_fn",
        "model::Model::test_only_constructor_must_not_render",
        "model::Model::unreachable_constructor",
        "model::unreachable_model_fn",
        "model::unused_helper_fn",
    ] {
        assert!(
            !reachable.iter().any(|actual| actual == not_expected),
            "unreachable callable {not_expected} was retained in graph",
        );
    }

    let reachable_items = report
        .reachable_items
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    for expected in [
        "helper::HELPER_STATIC(Static)",
        "helper::HelperData(Struct)",
        "helper::OFFSET(Const)",
        "model::BodyOnly(Struct)",
        "model::DataEnum(Enum)",
        "model::InputAlias(Type)",
        "model::Model(Struct)",
        "model::OutputAlias(Type)",
        "model::PUBLIC_LIMIT(Const)",
        "model::RUNTIME_NAME(Static)",
    ] {
        assert!(
            reachable_items.iter().any(|actual| actual == expected),
            "missing reachable item {expected}; got {reachable_items:?}",
        );
    }
    for not_expected in [
        "helper::UNREACHABLE_HELPER_CONST(Const)",
        "helper::UNREACHABLE_HELPER_STATIC(Static)",
        "helper::UnreachableHelper(Struct)",
        "model::UNREACHABLE_CONST(Const)",
        "model::UNREACHABLE_STATIC(Static)",
        "model::UnreachableAlias(Type)",
        "model::UnreachableEnum(Enum)",
        "model::UnreachableModel(Struct)",
    ] {
        assert!(
            !reachable_items.iter().any(|actual| actual == not_expected),
            "unreachable item {not_expected} was retained in graph",
        );
    }

    let app_source = fs::read_to_string(output.join("app/src/lib.rs")).unwrap();
    assert!(app_source.contains("pub fn export"));
    assert!(app_source.contains("Model::new"));
    assert!(app_source.contains("DataEnum::Tuple"));
    assert!(app_source.contains("DataEnum::Unit"));
    assert!(!app_source.contains("#[opensourced]"));
    assert!(!app_source.contains("opensourced::opensourced"));
    assert!(!app_source.contains("unreachable_app_fn"));
    assert!(!app_source.contains("app_unit_test_must_not_render"));
    assert!(!app_source.contains("cfg(test)"));

    let model_source = fs::read_to_string(output.join("model/src/lib.rs")).unwrap();
    assert!(model_source.contains("pub type InputAlias"));
    assert!(model_source.contains("pub type OutputAlias"));
    assert!(model_source.contains("pub const PUBLIC_LIMIT"));
    assert!(model_source.contains("pub static RUNTIME_NAME"));
    assert!(model_source.contains("pub struct Model"));
    assert!(model_source.contains("pub struct BodyOnly"));
    assert!(model_source.contains("pub enum DataEnum"));
    assert!(model_source.contains("Unit"));
    assert!(model_source.contains("Tuple(usize)"));
    assert!(model_source.contains("pub fn new"));
    assert!(model_source.contains("pub fn score"));
    assert!(model_source.contains("pub fn build_model"));
    assert!(!model_source.contains("UnreachableModel"));
    assert!(!model_source.contains("UnreachableEnum"));
    assert!(!model_source.contains("UnreachableAlias"));
    assert!(!model_source.contains("UNREACHABLE_CONST"));
    assert!(!model_source.contains("UNREACHABLE_STATIC"));
    assert!(!model_source.contains("unreachable_model_fn"));
    assert!(!model_source.contains("unused_helper_fn"));
    assert!(!model_source.contains("unreachable_constructor"));
    assert!(!model_source.contains("test_only_constructor_must_not_render"));
    assert!(!model_source.contains("model_unit_test_must_not_render"));
    assert!(!model_source.contains("cfg(test)"));

    let helper_source = fs::read_to_string(output.join("helper/src/lib.rs")).unwrap();
    assert!(helper_source.contains("pub struct HelperData"));
    assert!(helper_source.contains("pub const OFFSET"));
    assert!(helper_source.contains("pub static HELPER_STATIC"));
    assert!(helper_source.contains("pub fn new"));
    assert!(helper_source.contains("pub fn amount"));
    assert!(helper_source.contains("pub fn helper_checksum"));
    assert!(helper_source.contains("pub fn finalize"));
    assert!(!helper_source.contains("UnreachableHelper"));
    assert!(!helper_source.contains("UNREACHABLE_HELPER_CONST"));
    assert!(!helper_source.contains("UNREACHABLE_HELPER_STATIC"));
    assert!(!helper_source.contains("unreachable_helper_fn"));
    assert!(!helper_source.contains("unreachable_method"));
    assert!(!helper_source.contains("helper_unit_test_must_not_render"));
    assert!(!helper_source.contains("cfg(test)"));

    let status = Command::new("cargo")
        .arg("check")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", target)
        .status()
        .expect("cargo check should start");
    assert!(status.success(), "generated workspace should compile");
}

#[test]
fn retains_private_field_type_items_needed_by_rendered_surfaces() {
    let fixture = temp_dir("field-type-surface-fixture");
    let output = temp_dir("field-type-surface-output");
    let target = temp_dir("field-type-surface-target");
    let opensourced_path = opensourced_crate_path();

    write(
        &fixture.join("Cargo.toml"),
        r#"
[workspace]
members = ["app", "helper"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT"
"#,
    );
    write(
        &fixture.join("app/Cargo.toml"),
        &format!(
            r#"
[package]
name = "app"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
helper = {{ path = "../helper" }}
opensourced = {{ path = "{}" }}
"#,
            toml_path(&opensourced_path)
        ),
    );
    write(
        &fixture.join("app/src/lib.rs"),
        r#"
use opensourced::opensourced;

#[opensourced]
pub fn entry(outer: &helper::Outer) -> usize {
    helper::touch_outer(outer)
}
"#,
    );
    write(
        &fixture.join("helper/Cargo.toml"),
        r#"
[package]
name = "helper"
version.workspace = true
edition.workspace = true
license.workspace = true
"#,
    );
    write(
        &fixture.join("helper/src/lib.rs"),
        r#"
pub struct Inner {
    value: usize,
}

pub mod inputs {
    pub struct SurfaceInput {
        pub value: usize,
        pub label: String,
    }

    pub enum SurfaceKind {
        Alpha,
        Beta,
    }
}

use inputs::{SurfaceInput, SurfaceKind};

struct DebugInner;

impl std::fmt::Debug for DebugInner {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("DebugInner")
    }
}

struct DebugBox<T: std::fmt::Debug> {
    value: T,
}

enum Status {
    Ready,
    Blocked,
}

struct Adapter {
    value: usize,
    label_len: usize,
}

impl From<SurfaceInput> for Adapter {
    fn from(value: SurfaceInput) -> Self {
        Adapter {
            value: value.value,
            label_len: value.label.len(),
        }
    }
}

enum AdapterKind {
    Alpha,
    Beta,
}

impl From<SurfaceKind> for AdapterKind {
    fn from(value: SurfaceKind) -> Self {
        match value {
            SurfaceKind::Alpha => Self::Alpha,
            SurfaceKind::Beta => Self::Beta,
        }
    }
}

pub struct Outer {
    inner: Inner,
    debug: DebugBox<DebugInner>,
    adapter: Adapter,
    kind: AdapterKind,
    status: Status,
}

impl Outer {
    pub fn touch(&self) -> usize {
        let _ = &self.inner;
        let _ = &self.debug.value;
        let _ = self.adapter.value;
        let _ = self.adapter.label_len;
        let _ = match &self.kind {
            AdapterKind::Alpha => 1,
            AdapterKind::Beta => 2,
        };
        match &self.status {
            Status::Ready => 1,
            Status::Blocked => 0,
        }
    }
}

pub fn touch_outer(outer: &Outer) -> usize {
    outer.touch()
}
"#,
    );

    generate(GenerateOptions {
        workspace_root: fixture,
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let helper_source = fs::read_to_string(output.join("helper/src/lib.rs")).unwrap();
    assert!(helper_source.contains("pub struct Outer"));
    assert!(
        helper_source.contains("pub struct Inner"),
        "field type item must be rendered when the retained field uses it:\n{helper_source}"
    );
    assert!(
        helper_source.contains("pub struct SurfaceInput"),
        "external impl headers must retain local input type surfaces:\n{helper_source}"
    );
    assert!(
        helper_source.contains("pub label: String"),
        "type-surface dependencies must keep the full struct shape:\n{helper_source}"
    );
    assert!(
        helper_source.contains("pub enum SurfaceKind"),
        "external impl headers must retain local enum input surfaces:\n{helper_source}"
    );
    assert!(
        helper_source.contains("Alpha") && helper_source.contains("Beta"),
        "type-surface dependencies must keep enum variants needed by retained impls:\n{helper_source}"
    );
    assert!(
        helper_source.contains("impl From<SurfaceInput> for Adapter"),
        "compiler-required external trait impl should keep its original header:\n{helper_source}"
    );

    let status = Command::new("cargo")
        .arg("check")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", target)
        .status()
        .expect("cargo check should start");
    assert!(status.success(), "generated workspace should compile");
}

fn write_fixture(root: &Path) {
    let opensourced_path = opensourced_crate_path();

    write(
        &root.join("Cargo.toml"),
        r#"
[workspace]
members = ["app", "model", "helper"]
resolver = "2"

[workspace.package]
edition = "2021"
version = "0.1.0"
license = "MIT"
"#,
    );

    write(
        &root.join("app/Cargo.toml"),
        &format!(
            r#"
[package]
name = "app"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
opensourced = {{ path = "{}" }}
model = {{ path = "../model" }}
helper = {{ path = "../helper" }}
"#,
            toml_path(&opensourced_path),
        ),
    );
    write(
        &root.join("app/src/lib.rs"),
        r#"
use helper::{finalize, helper_checksum, HelperData};
use model::{
    build_model, BodyOnly, DataEnum, InputAlias, Model, OutputAlias, PUBLIC_LIMIT, RUNTIME_NAME,
};
use opensourced::opensourced;

#[opensourced]
pub fn export(input: InputAlias, mode: DataEnum) -> OutputAlias {
    let model = Model::new(input, DataEnum::Tuple(3));
    let direct = build_model(input);
    let support = HelperData::new(input + PUBLIC_LIMIT);
    let body_only: BodyOnly = BodyOnly {
        value: helper_checksum(&support) + support.amount() + RUNTIME_NAME.len() + direct.score(),
    };

    match mode {
        DataEnum::Unit => model.score() + body_only.value,
        DataEnum::Tuple(offset) => finalize(body_only.value, offset),
        DataEnum::StructLike { value } => finalize(body_only.value, value),
    }
}

pub fn unreachable_app_fn() -> usize {
    model::unreachable_model_fn() + helper::unreachable_helper_fn()
}

#[test]
fn app_unit_test_must_not_render() {
    assert_eq!(2 + 2, 4);
}

#[cfg(test)]
mod tests {
    #[test]
    fn nested_app_unit_test_must_not_render() {
        assert_eq!(1, 1);
    }
}
"#,
    );

    write(
        &root.join("model/Cargo.toml"),
        r#"
[package]
name = "model"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
util = { package = "helper", path = "../helper" }
"#,
    );
    write(
        &root.join("model/src/lib.rs"),
        r#"
use util::{HelperData, OFFSET};

pub type InputAlias = usize;
pub type OutputAlias = usize;

pub const PUBLIC_LIMIT: usize = 11;
pub static RUNTIME_NAME: &str = "fixture-runtime";

pub struct Model {
    value: InputAlias,
    marker: DataEnum,
    helper: HelperData,
}

pub struct BodyOnly {
    pub value: usize,
}

pub enum DataEnum {
    Unit,
    Tuple(usize),
    StructLike { value: usize },
}

impl Model {
    pub fn new(value: InputAlias, marker: DataEnum) -> Self {
        let helper = HelperData::new(value + OFFSET);
        Self {
            value,
            marker,
            helper,
        }
    }

    pub fn score(&self) -> OutputAlias {
        let variant = match self.marker {
            DataEnum::Unit => 1,
            DataEnum::Tuple(value) => value,
            DataEnum::StructLike { value } => value,
        };

        self.value + self.helper.amount() + variant + PUBLIC_LIMIT + RUNTIME_NAME.len()
    }

    pub fn unreachable_constructor() -> Self {
        Self::new(0, DataEnum::Unit)
    }

    #[cfg(test)]
    pub fn test_only_constructor_must_not_render() -> Self {
        Self::new(1, DataEnum::Tuple(1))
    }
}

pub fn build_model(input: InputAlias) -> Model {
    Model::new(input, DataEnum::Unit)
}

pub fn unreachable_model_fn() -> usize {
    unused_helper_fn()
}

fn unused_helper_fn() -> usize {
    9
}

pub struct UnreachableModel;

pub enum UnreachableEnum {
    Unit,
    Tuple(usize),
}

pub type UnreachableAlias = usize;
pub const UNREACHABLE_CONST: usize = 1;
pub static UNREACHABLE_STATIC: usize = 2;

#[test]
fn model_unit_test_must_not_render() {
    assert_eq!(3, 3);
}

#[cfg(test)]
mod tests {
    #[test]
    fn nested_model_unit_test_must_not_render() {
        assert_eq!(4, 4);
    }
}
"#,
    );

    write(
        &root.join("helper/Cargo.toml"),
        r#"
[package]
name = "helper"
version.workspace = true
edition.workspace = true
license.workspace = true
"#,
    );
    write(
        &root.join("helper/src/lib.rs"),
        r#"
pub const OFFSET: usize = 5;
pub static HELPER_STATIC: &str = "helper-static";

pub struct HelperData {
    amount: usize,
}

impl HelperData {
    pub fn new(amount: usize) -> Self {
        Self { amount }
    }

    pub fn amount(&self) -> usize {
        self.amount + HELPER_STATIC.len()
    }

    pub fn unreachable_method(&self) -> usize {
        self.amount
    }
}

pub fn helper_checksum(data: &HelperData) -> usize {
    data.amount() + OFFSET
}

pub fn finalize(value: usize, offset: usize) -> usize {
    value + offset + HELPER_STATIC.len()
}

pub fn unreachable_helper_fn() -> usize {
    UNREACHABLE_HELPER_CONST
}

pub struct UnreachableHelper;
pub const UNREACHABLE_HELPER_CONST: usize = 10;
pub static UNREACHABLE_HELPER_STATIC: usize = 20;

#[test]
fn helper_unit_test_must_not_render() {
    assert_eq!(5, 5);
}

#[cfg(test)]
mod tests {
    #[test]
    fn nested_helper_unit_test_must_not_render() {
        assert_eq!(6, 6);
    }
}
"#,
    );
}

fn write(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents.trim_start()).unwrap();
}

fn opensourced_crate_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("opensource_core should be under crates")
        .join("opensourced")
}

fn toml_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "\\\\")
}

fn temp_dir(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("opensourced-{label}-{unique}"));
    if path.exists() {
        fs::remove_dir_all(&path).unwrap();
    }
    path
}
