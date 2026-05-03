mod analyzer;
mod feedback;
mod manifest;
mod model;
mod parse;
mod reduce;
mod render;

use std::{
    fs,
    path::{Path, PathBuf},
};

pub use analyzer::{AnalyzerMode, AnalyzerReport, SemanticReport};
pub use feedback::{check_workspace, write_report, CheckDiagnostic, CheckOptions, CheckReport};
pub use model::{CallableId, ItemId, RootId};
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct GenerateOptions {
    pub workspace_root: PathBuf,
    pub output_root: PathBuf,
}

#[derive(Debug, Clone)]
pub struct GenerateReport {
    pub analyzer: AnalyzerReport,
    pub root: RootId,
    pub roots: Vec<RootId>,
    pub packages: Vec<String>,
    pub reachable: Vec<CallableId>,
    pub reachable_items: Vec<ItemId>,
    pub files_written: usize,
}

pub fn generate(options: GenerateOptions) -> Result<GenerateReport, Box<dyn std::error::Error>> {
    generate_with_analyzer(options, AnalyzerMode::Syn)
}

pub fn generate_with_analyzer(
    options: GenerateOptions,
    analyzer_mode: AnalyzerMode,
) -> Result<GenerateReport, Box<dyn std::error::Error>> {
    let analyzer = analyzer::load_report(&options.workspace_root, analyzer_mode)?;
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
        analyzer,
        root: reduced.root,
        roots: reduced.roots,
        packages,
        reachable,
        reachable_items,
        files_written,
    })
}

pub fn write_generate_report(
    report: &GenerateReport,
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(&GenerateReportJson::from_report(report))?;
    fs::write(path, json)?;
    Ok(())
}

#[derive(Serialize)]
struct GenerateReportJson {
    analyzer: AnalyzerReportJson,
    root: String,
    roots: Vec<String>,
    packages: Vec<String>,
    reachable: Vec<String>,
    reachable_items: Vec<String>,
    files_written: usize,
}

impl GenerateReportJson {
    fn from_report(report: &GenerateReport) -> Self {
        Self {
            analyzer: AnalyzerReportJson::from_report(&report.analyzer),
            root: report.root.to_string(),
            roots: report.roots.iter().map(ToString::to_string).collect(),
            packages: report.packages.clone(),
            reachable: report.reachable.iter().map(ToString::to_string).collect(),
            reachable_items: report
                .reachable_items
                .iter()
                .map(ToString::to_string)
                .collect(),
            files_written: report.files_written,
        }
    }
}

#[derive(Serialize)]
struct AnalyzerReportJson {
    mode: String,
    loaded: bool,
    engine: String,
    notes: Vec<String>,
    semantic: Option<SemanticReportJson>,
}

impl AnalyzerReportJson {
    fn from_report(report: &AnalyzerReport) -> Self {
        Self {
            mode: report.mode.as_str().to_string(),
            loaded: report.loaded,
            engine: report.engine.clone(),
            notes: report.notes.clone(),
            semantic: report
                .semantic
                .as_ref()
                .map(SemanticReportJson::from_report),
        }
    }
}

#[derive(Serialize)]
struct SemanticReportJson {
    source_files: usize,
    analyzed_files: usize,
    failed_files: usize,
    skipped_files: usize,
    file_budget: usize,
    method_call_budget: usize,
    path_budget: usize,
    method_calls: usize,
    queried_method_calls: usize,
    resolved_method_calls: usize,
    callable_method_calls: usize,
    fallback_method_calls: usize,
    unresolved_method_calls: usize,
    unqueried_method_calls: usize,
    paths: usize,
    queried_paths: usize,
    resolved_paths: usize,
    unresolved_paths: usize,
    unqueried_paths: usize,
}

impl SemanticReportJson {
    fn from_report(report: &SemanticReport) -> Self {
        Self {
            source_files: report.source_files,
            analyzed_files: report.analyzed_files,
            failed_files: report.failed_files,
            skipped_files: report.skipped_files,
            file_budget: report.file_budget,
            method_call_budget: report.method_call_budget,
            path_budget: report.path_budget,
            method_calls: report.method_calls,
            queried_method_calls: report.queried_method_calls,
            resolved_method_calls: report.resolved_method_calls,
            callable_method_calls: report.callable_method_calls,
            fallback_method_calls: report.fallback_method_calls,
            unresolved_method_calls: report.unresolved_method_calls,
            unqueried_method_calls: report.unqueried_method_calls,
            paths: report.paths,
            queried_paths: report.queried_paths,
            resolved_paths: report.resolved_paths,
            unresolved_paths: report.unresolved_paths,
            unqueried_paths: report.unqueried_paths,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{generate, write_generate_report, GenerateOptions};

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
    fn writes_generation_report_json() {
        let output = temp_output("generation-report-output");
        let report = generate(GenerateOptions {
            workspace_root: workspace_root(),
            output_root: output,
        })
        .expect("reduction should succeed");
        let report_path = temp_output("generation-report-json").join("slice-report.json");

        write_generate_report(&report, &report_path).expect("generation report should be written");

        let value: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(report_path).unwrap()).unwrap();
        assert_eq!(value["analyzer"]["mode"], "syn");
        assert_eq!(value["root"], report.root.to_string());
        assert_eq!(value["files_written"], report.files_written);
        assert_eq!(
            value["packages"].as_array().unwrap().len(),
            report.packages.len()
        );
    }

    #[test]
    fn refuses_to_overwrite_unmarked_nonempty_output() {
        let output = temp_output("unmarked-output");
        fs::create_dir_all(&output).unwrap();
        fs::write(output.join("keep.txt"), "do not delete").unwrap();

        let error = generate(GenerateOptions {
            workspace_root: workspace_root(),
            output_root: output.clone(),
        })
        .expect_err("unmarked non-empty outputs should fail closed");

        assert!(
            error.to_string().contains("not created by slicers"),
            "unexpected error: {error}"
        );
        assert!(
            output.join("keep.txt").exists(),
            "unmarked output contents must not be deleted"
        );
    }

    #[test]
    fn reruns_can_replace_marked_outputs() {
        let output = temp_output("marked-output");
        generate(GenerateOptions {
            workspace_root: workspace_root(),
            output_root: output.clone(),
        })
        .expect("initial reduction should succeed");
        assert!(output.join(".slicers-output").exists());
        fs::write(output.join("stale.txt"), "stale").unwrap();

        generate(GenerateOptions {
            workspace_root: workspace_root(),
            output_root: output.clone(),
        })
        .expect("marked output should be replaceable");

        assert!(output.join(".slicers-output").exists());
        assert!(
            !output.join("stale.txt").exists(),
            "rerender should replace stale generated output contents"
        );
    }

    #[test]
    fn refuses_output_inside_input_workspace() {
        let output = workspace_root().join("target/slicers-inside-workspace-output");
        if output.exists() {
            fs::remove_dir_all(&output).unwrap();
        }

        let error = generate(GenerateOptions {
            workspace_root: workspace_root(),
            output_root: output.clone(),
        })
        .expect_err("output inside source workspace should be rejected");

        assert!(
            error.to_string().contains("inside input workspace"),
            "unexpected error: {error}"
        );
        assert!(
            !output.exists(),
            "rejected inside-workspace output should not be created"
        );
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
