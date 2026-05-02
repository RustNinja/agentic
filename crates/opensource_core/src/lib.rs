mod manifest;
mod model;
mod parse;
mod reduce;
mod render;

use std::path::PathBuf;

pub use model::{CallableId, ItemId};

#[derive(Debug, Clone)]
pub struct GenerateOptions {
    pub workspace_root: PathBuf,
    pub output_root: PathBuf,
}

#[derive(Debug, Clone)]
pub struct GenerateReport {
    pub root: CallableId,
    pub packages: Vec<String>,
    pub reachable: Vec<CallableId>,
    pub reachable_items: Vec<ItemId>,
    pub files_written: usize,
}

#[derive(Debug, Clone)]
pub struct LintAuditOptions {
    pub workspace_root: PathBuf,
    pub output_root: PathBuf,
}

#[derive(Debug, Clone)]
pub struct LintAuditReport {
    pub packages: Vec<String>,
    pub linted_roots: Vec<PathBuf>,
    pub files_written: usize,
}

pub fn generate(options: GenerateOptions) -> Result<GenerateReport, Box<dyn std::error::Error>> {
    let workspace = manifest::load_workspace(&options.workspace_root)?;
    let project = parse::parse_workspace(workspace)?;
    let reduced = reduce::reduce(&project)?;
    let files_written = render::write_reduced_workspace(&project, &reduced, &options.output_root)?;

    let mut packages = reduced.packages.iter().cloned().collect::<Vec<_>>();
    packages.sort();

    let mut reachable = reduced.reachable.iter().cloned().collect::<Vec<_>>();
    reachable.sort();

    let mut reachable_items = reduced.reachable_items.iter().cloned().collect::<Vec<_>>();
    reachable_items.sort();

    Ok(GenerateReport {
        root: reduced.root,
        packages,
        reachable,
        reachable_items,
        files_written,
    })
}

pub fn generate_lint_audit_workspace(
    options: LintAuditOptions,
) -> Result<LintAuditReport, Box<dyn std::error::Error>> {
    let workspace = manifest::load_workspace(&options.workspace_root)?;
    let report = render::write_lint_audit_workspace(&workspace, &options.output_root)?;

    let mut packages = workspace.packages.keys().cloned().collect::<Vec<_>>();
    packages.sort();

    Ok(LintAuditReport {
        packages,
        linted_roots: report.linted_roots,
        files_written: report.files_written,
    })
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{generate, generate_lint_audit_workspace, GenerateOptions, LintAuditOptions};

    #[test]
    fn reduces_fixture_to_reachable_callables() {
        let output = temp_output("callables");
        let report = generate(GenerateOptions {
            workspace_root: workspace_root(),
            output_root: output.clone(),
        })
        .expect("reduction should succeed");

        let reachable = report
            .reachable
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        for expected in [
            "a::open_source_entry",
            "b::compute",
            "b::helper",
            "c::adjust",
            "d::Worker::new",
            "d::Worker::run",
            "d::hash",
            "d::normalize",
            "d::shared",
            "e::finish",
            "e::mix",
            "e::seed",
            "d::<Worker as Transform>::transform",
        ] {
            assert!(
                reachable.iter().any(|actual| actual == expected),
                "missing reachable callable {expected}; got {reachable:?}",
            );
        }

        for not_expected in [
            "a::internal_entry",
            "b::unused_public",
            "c::unused",
            "d::Worker::unused_method",
            "d::unused_private",
            "d::unused_public",
            "e::unused_leaf",
            "e::tempting_but_unused",
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
            "d::Mode(Enum)",
            "d::Score(Type)",
            "d::Transform(Trait)",
            "d::Worker(Struct)",
            "e::DEFAULT_SEED(Const)",
            "e::Score(Type)",
        ] {
            assert!(
                reachable_items.iter().any(|actual| actual == expected),
                "missing reachable item {expected}; got {reachable_items:?}",
            );
        }
        assert!(
            !reachable_items
                .iter()
                .any(|actual| actual == "d::UnusedEnum(Enum)"),
            "unreachable enum was retained in item graph",
        );

        let d_source = fs::read_to_string(output.join("d/src/lib.rs")).unwrap();
        assert!(d_source.contains("pub enum Mode"));
        assert!(d_source.contains("pub trait Transform"));
        assert!(d_source.contains("impl Transform for Worker"));
        assert!(d_source.contains("fn transform"));
        assert!(d_source.contains("pub fn hash"));
        assert!(d_source.contains("pub fn run"));
        assert!(!d_source.contains("UnusedEnum"));
        assert!(!d_source.contains("cfg(test)"));
        assert!(!d_source.contains("unit_test_that_must_not_be_exported"));
        assert!(!d_source.contains("unused_public"));
        assert!(!d_source.contains("unused_private"));
        assert!(!d_source.contains("unused_method"));

        let b_source = fs::read_to_string(output.join("b/src/lib.rs")).unwrap();
        assert!(b_source.contains("use d::Transform"));
        assert!(b_source.contains("worker.transform(value)"));
        assert!(!b_source.contains("cfg(test)"));
        assert!(!b_source.contains("unit_test_that_must_not_be_exported"));

        let a_source = fs::read_to_string(output.join("a/src/lib.rs")).unwrap();
        assert!(a_source.contains("pub fn open_source_entry"));
        assert!(!a_source.contains("#[opensourced]"));
        assert!(!a_source.contains("internal_entry"));
    }

    #[test]
    fn generated_fixture_workspace_compiles() {
        let output = temp_output("compile");
        generate(GenerateOptions {
            workspace_root: workspace_root(),
            output_root: output.clone(),
        })
        .expect("reduction should succeed");

        let target_dir = temp_output("target");
        let status = Command::new("cargo")
            .arg("check")
            .current_dir(&output)
            .env("CARGO_TARGET_DIR", target_dir)
            .status()
            .expect("cargo check should start");

        assert!(status.success(), "generated workspace did not compile");
    }

    #[test]
    fn lint_audit_workspace_copies_full_tree_and_injects_lints() {
        let output = temp_output("lint-audit");
        let report = generate_lint_audit_workspace(LintAuditOptions {
            workspace_root: workspace_root(),
            output_root: output.clone(),
        })
        .expect("lint audit workspace should generate");

        assert!(report.files_written > 0);
        assert!(report.packages.iter().any(|package| package == "a"));
        assert!(report
            .linted_roots
            .iter()
            .any(|path| path.ends_with("a/src/lib.rs")));
        assert!(output.join("Cargo.lock").exists());

        let source = fs::read_to_string(output.join("fixtures/a/src/lib.rs")).unwrap();
        assert!(source.contains("slicers lint-audit"));
        assert!(source.contains("warn(dead_code, unused_imports, unused_macros, unreachable_pub)"));
        assert!(source.contains("internal_entry"));
    }

    fn workspace_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .expect("core crate should live under crates/opensource_core")
            .to_path_buf()
    }

    fn temp_output(label: &str) -> PathBuf {
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
}
