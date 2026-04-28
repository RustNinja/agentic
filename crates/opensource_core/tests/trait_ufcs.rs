use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use opensource_core::{generate, GenerateOptions};

#[test]
fn trait_impls_reached_through_explicit_ufcs_are_rendered() {
    let workspace = temp_path("workspace");
    let output = temp_path("output");
    let target_dir = temp_path("target");
    write_fixture_workspace(&workspace);

    let report = generate(GenerateOptions {
        workspace_root: workspace.clone(),
        output_root: output.clone(),
    })
    .expect("reduction should succeed");

    let reachable = report
        .reachable
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    for expected in [
        "app::entry",
        "domain::Thing::new",
        "domain::Thing::flavor_score",
        "domain::inspect",
        "domain::<Thing as Describe>::describe",
        "util::kind",
        "util::mark",
        "util::score",
    ] {
        assert!(
            reachable.iter().any(|actual| actual == expected),
            "missing reachable callable {expected}; got {reachable:?}",
        );
    }

    for not_expected in [
        "app::cold_entry",
        "domain::Thing::hidden",
        "domain::dormant",
        "util::unused_helper",
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
        "domain::Describe(Trait)",
        "domain::Id(Struct)",
        "domain::State(Enum)",
        "domain::Thing(Struct)",
        "util::Flavor(Enum)",
        "util::Token(Struct)",
    ] {
        assert!(
            reachable_items.iter().any(|actual| actual == expected),
            "missing reachable item {expected}; got {reachable_items:?}",
        );
    }

    for not_expected in ["domain::UnusedDomainEnum(Enum)", "util::UnusedUtil(Struct)"] {
        assert!(
            !reachable_items.iter().any(|actual| actual == not_expected),
            "unreachable item {not_expected} was retained in graph: {reachable_items:?}",
        );
    }

    let app_source = read(output.join("app/src/lib.rs"));
    assert!(app_source.contains("pub fn entry"));
    assert!(app_source.contains("<domain::Thing as domain::Describe>::describe"));
    assert!(!app_source.contains("#[opensourced]"));
    assert!(!app_source.contains("cold_entry"));

    let domain_source = read(output.join("domain/src/lib.rs"));
    for expected in [
        "pub struct Id",
        "pub struct Thing",
        "pub enum State",
        "pub trait Describe",
        "impl Describe for Thing",
        "fn describe",
        "pub fn flavor_score",
        "pub fn inspect",
        "State::Ready(Id(value))",
        "State::Gone(flavor)",
    ] {
        assert!(
            domain_source.contains(expected),
            "missing `{expected}` in reduced domain source:\n{domain_source}",
        );
    }
    for not_expected in ["hidden", "dormant", "UnusedDomainEnum", "unused_helper"] {
        assert!(
            !domain_source.contains(not_expected),
            "unexpected `{not_expected}` in reduced domain source:\n{domain_source}",
        );
    }

    let util_source = read(output.join("util/src/lib.rs"));
    for expected in [
        "pub enum Flavor",
        "pub struct Token",
        "pub fn kind",
        "pub fn mark",
        "pub fn score",
    ] {
        assert!(
            util_source.contains(expected),
            "missing `{expected}` in reduced util source:\n{util_source}",
        );
    }
    for not_expected in ["unused_helper", "UnusedUtil"] {
        assert!(
            !util_source.contains(not_expected),
            "unexpected `{not_expected}` in reduced util source:\n{util_source}",
        );
    }

    let cargo_check = Command::new("cargo")
        .arg("check")
        .current_dir(&output)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("cargo check should start");

    assert!(
        cargo_check.status.success(),
        "generated workspace did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\napp/src/lib.rs:\n{}\ndomain/src/lib.rs:\n{}\nutil/src/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        app_source,
        domain_source,
        util_source,
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
members = ["app", "domain", "util"]
resolver = "2"

[workspace.package]
edition = "2021"
version = "0.1.0"
license = "MIT"

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
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
domain = { path = "../domain" }
opensourced.workspace = true
util = { path = "../util" }
"#,
    );
    write(
        root.join("app/src/lib.rs"),
        r#"use opensourced::opensourced;

#[opensourced]
pub fn entry() -> String {
    let thing = domain::Thing::new(domain::Id(41), util::Flavor::Fresh);
    let state = domain::State::Ready(domain::Id(1));
    let observed = domain::inspect(state);
    let label = <domain::Thing as domain::Describe>::describe(&thing, util::Flavor::Fresh);
    label + ":" + observed
}

pub fn cold_entry() -> String {
    let thing = domain::Thing::new(domain::Id(0), util::Flavor::Hidden);
    thing.hidden()
}
"#,
    );

    write(
        root.join("domain/Cargo.toml"),
        r#"[package]
name = "domain"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
util = { path = "../util" }
"#,
    );
    write(
        root.join("domain/src/lib.rs"),
        r#"#[derive(Clone, Copy)]
pub struct Id(pub u32);

pub struct Thing {
    id: Id,
    flavor: util::Flavor,
}

pub enum State {
    Ready(Id),
    Blocked,
    Gone(util::Flavor),
}

pub enum UnusedDomainEnum {
    Never,
}

pub trait Describe {
    fn describe(&self, flavor: util::Flavor) -> String;
}

impl Thing {
    pub fn new(id: Id, flavor: util::Flavor) -> Self {
        Self { id, flavor }
    }

    pub fn flavor_score(&self) -> u32 {
        util::score(self.flavor)
    }

    pub fn hidden(&self) -> String {
        util::unused_helper()
    }
}

impl Describe for Thing {
    fn describe(&self, flavor: util::Flavor) -> String {
        let state = State::Ready(Id(self.id.0 + 1));
        let status = inspect(state);
        let marked = util::mark(flavor);

        if self.flavor_score() > 0 {
            marked
        } else {
            status.to_string()
        }
    }
}

pub fn inspect(state: State) -> &'static str {
    match state {
        State::Ready(Id(value)) if value > 0 => util::kind(util::Flavor::Fresh),
        State::Ready(_) => "empty",
        State::Blocked => "blocked",
        State::Gone(flavor) => util::kind(flavor),
    }
}

pub fn dormant(state: State) -> &'static str {
    match state {
        State::Blocked => "blocked",
        _ => "unused",
    }
}
"#,
    );

    write(
        root.join("util/Cargo.toml"),
        r#"[package]
name = "util"
version.workspace = true
edition.workspace = true
license.workspace = true
"#,
    );
    write(
        root.join("util/src/lib.rs"),
        r#"#[derive(Clone, Copy)]
pub enum Flavor {
    Fresh,
    Stale,
    Hidden,
}

pub struct Token(pub &'static str);

pub struct UnusedUtil;

pub fn kind(flavor: Flavor) -> &'static str {
    match flavor {
        Flavor::Fresh => "fresh",
        Flavor::Stale => "stale",
        Flavor::Hidden => "hidden",
    }
}

pub fn mark(flavor: Flavor) -> String {
    let token = match flavor {
        Flavor::Fresh => Token("fresh"),
        Flavor::Stale => Token("stale"),
        Flavor::Hidden => Token("hidden"),
    };
    token.0.to_string()
}

pub fn score(flavor: Flavor) -> u32 {
    match flavor {
        Flavor::Fresh => 10,
        Flavor::Stale => 1,
        Flavor::Hidden => 0,
    }
}

pub fn unused_helper() -> String {
    "unused".to_string()
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
    std::env::temp_dir().join(format!("opensource-core-trait-ufcs-{label}-{unique}"))
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
