use std::path::Path;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AnalyzerMode {
    Syn,
    RustAnalyzerHir,
}

impl AnalyzerMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Syn => "syn",
            Self::RustAnalyzerHir => "ra-hir",
        }
    }
}

impl std::str::FromStr for AnalyzerMode {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "syn" => Ok(Self::Syn),
            "ra" | "ra-hir" | "rust-analyzer" | "rust-analyzer-hir" => Ok(Self::RustAnalyzerHir),
            _ => Err(format!(
                "unknown analyzer {value:?}; expected syn or ra-hir"
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AnalyzerReport {
    pub mode: AnalyzerMode,
    pub loaded: bool,
    pub engine: String,
    pub notes: Vec<String>,
    pub semantic: Option<SemanticReport>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct SemanticReport {
    pub source_files: usize,
    pub analyzed_files: usize,
    pub failed_files: usize,
    pub skipped_files: usize,
    pub file_budget: usize,
    pub method_call_budget: usize,
    pub path_budget: usize,
    pub method_calls: usize,
    pub queried_method_calls: usize,
    pub resolved_method_calls: usize,
    pub callable_method_calls: usize,
    pub fallback_method_calls: usize,
    pub unresolved_method_calls: usize,
    pub unqueried_method_calls: usize,
    pub paths: usize,
    pub queried_paths: usize,
    pub resolved_paths: usize,
    pub unresolved_paths: usize,
    pub unqueried_paths: usize,
}

impl AnalyzerReport {
    fn syn() -> Self {
        Self {
            mode: AnalyzerMode::Syn,
            loaded: true,
            engine: "syn".to_string(),
            notes: vec!["using syntactic resolver".to_string()],
            semantic: None,
        }
    }
}

pub trait SemanticProvider {
    fn report(&self) -> &AnalyzerReport;
}

pub struct SynSemanticProvider {
    report: AnalyzerReport,
}

impl SynSemanticProvider {
    pub fn new() -> Self {
        Self {
            report: AnalyzerReport::syn(),
        }
    }
}

impl SemanticProvider for SynSemanticProvider {
    fn report(&self) -> &AnalyzerReport {
        &self.report
    }
}

pub fn load_report(
    workspace_root: &Path,
    mode: AnalyzerMode,
) -> Result<AnalyzerReport, Box<dyn std::error::Error>> {
    match mode {
        AnalyzerMode::Syn => {
            let provider = SynSemanticProvider::new();
            Ok(provider.report().clone())
        }
        AnalyzerMode::RustAnalyzerHir => rust_analyzer::load_report(workspace_root),
    }
}

#[cfg(feature = "ra-hir")]
mod rust_analyzer {
    use std::{
        ffi::OsStr,
        panic::{self, AssertUnwindSafe},
        path::{Component, Path},
        sync::atomic::{AtomicUsize, Ordering},
    };

    use ra_ap_load_cargo::{load_workspace_at, LoadCargoConfig, ProcMacroServerChoice};
    use ra_ap_project_model::CargoConfig;
    use ra_ap_syntax::{ast, AstNode};

    use super::{AnalyzerMode, AnalyzerReport, SemanticProvider, SemanticReport};

    const DEFAULT_SEMANTIC_FILE_BUDGET: usize = 48;
    const DEFAULT_METHOD_CALL_BUDGET: usize = 1_000;
    const DEFAULT_PATH_BUDGET: usize = 2_000;

    pub struct RustAnalyzerSemanticProvider {
        report: AnalyzerReport,
        _database: ra_ap_ide::RootDatabase,
    }

    impl RustAnalyzerSemanticProvider {
        fn load(workspace_root: &Path) -> Result<Self, Box<dyn std::error::Error>> {
            let cargo_config = CargoConfig {
                set_test: true,
                ..CargoConfig::default()
            };
            let load_config = LoadCargoConfig {
                load_out_dirs_from_check: false,
                with_proc_macro_server: ProcMacroServerChoice::None,
                prefill_caches: false,
                num_worker_threads: 1,
                proc_macro_processes: 1,
            };
            let progress_events = AtomicUsize::new(0);
            let progress = |_: String| {
                progress_events.fetch_add(1, Ordering::Relaxed);
            };

            let (database, vfs, proc_macro_client) =
                load_workspace_at(workspace_root, &cargo_config, &load_config, &progress)?;

            let semantic = collect_semantic_report(&database, &vfs, workspace_root);
            let mut notes = vec![
                "rust-analyzer RootDatabase loaded".to_string(),
                "HIR Semantics initialized".to_string(),
                "proc macro expansion disabled for first integration pass".to_string(),
            ];
            notes.push(format!(
                "workspace load progress events: {}",
                progress_events.load(Ordering::Relaxed)
            ));
            notes.push(format!(
                "proc macro client active: {}",
                proc_macro_client.is_some()
            ));
            notes.push(format!(
                "HIR semantic inventory: {}/{} files analyzed, {} skipped by budget, {}/{} queried method calls resolved to functions, {}/{} callable, {}/{} fallback, {} method calls unqueried, {}/{} queried paths resolved, {} paths unqueried",
                semantic.analyzed_files,
                semantic.source_files,
                semantic.skipped_files,
                semantic.resolved_method_calls,
                semantic.queried_method_calls,
                semantic.callable_method_calls,
                semantic.queried_method_calls,
                semantic.fallback_method_calls,
                semantic.queried_method_calls,
                semantic.unqueried_method_calls,
                semantic.resolved_paths,
                semantic.queried_paths,
                semantic.unqueried_paths
            ));
            notes.push(format!(
                "HIR semantic budgets: files={}, method_calls={}, paths={}",
                semantic.file_budget, semantic.method_call_budget, semantic.path_budget
            ));
            if semantic.failed_files > 0 {
                notes.push(format!(
                    "HIR semantic inventory skipped {} files after analyzer panics",
                    semantic.failed_files
                ));
            }

            Ok(Self {
                report: AnalyzerReport {
                    mode: AnalyzerMode::RustAnalyzerHir,
                    loaded: true,
                    engine: "rust-analyzer HIR".to_string(),
                    notes,
                    semantic: Some(semantic),
                },
                _database: database,
            })
        }
    }

    impl SemanticProvider for RustAnalyzerSemanticProvider {
        fn report(&self) -> &AnalyzerReport {
            &self.report
        }
    }

    pub fn load_report(
        workspace_root: &Path,
    ) -> Result<AnalyzerReport, Box<dyn std::error::Error>> {
        let provider = RustAnalyzerSemanticProvider::load(workspace_root)?;
        Ok(provider.report().clone())
    }

    fn collect_semantic_report(
        database: &ra_ap_ide::RootDatabase,
        vfs: &ra_ap_vfs::Vfs,
        workspace_root: &Path,
    ) -> SemanticReport {
        ra_ap_hir::attach_db(database, || {
            collect_semantic_report_attached(database, vfs, workspace_root)
        })
    }

    fn collect_semantic_report_attached(
        database: &ra_ap_ide::RootDatabase,
        vfs: &ra_ap_vfs::Vfs,
        workspace_root: &Path,
    ) -> SemanticReport {
        let canonical_workspace_root = workspace_root
            .canonicalize()
            .unwrap_or_else(|_| workspace_root.to_path_buf());
        let semantics = ra_ap_ide::Semantics::new(database);
        let mut report = SemanticReport {
            file_budget: semantic_budget_from_env(
                "OPENSOURCE_RA_SEMANTIC_FILE_BUDGET",
                DEFAULT_SEMANTIC_FILE_BUDGET,
            ),
            method_call_budget: semantic_budget_from_env(
                "OPENSOURCE_RA_METHOD_CALL_BUDGET",
                DEFAULT_METHOD_CALL_BUDGET,
            ),
            path_budget: semantic_budget_from_env("OPENSOURCE_RA_PATH_BUDGET", DEFAULT_PATH_BUDGET),
            ..SemanticReport::default()
        };
        let mut budget = SemanticBudget {
            remaining_method_calls: report.method_call_budget,
            remaining_paths: report.path_budget,
        };

        for (file_id, vfs_path) in vfs.iter() {
            if !is_workspace_rust_file(vfs_path, workspace_root, &canonical_workspace_root) {
                continue;
            }

            report.source_files += 1;
            if report.analyzed_files + report.failed_files >= report.file_budget {
                report.skipped_files += 1;
                continue;
            }

            match panic::catch_unwind(AssertUnwindSafe(|| {
                collect_file_semantics(&semantics, file_id, &mut budget)
            })) {
                Ok(file_report) => {
                    report.analyzed_files += 1;
                    report.method_calls += file_report.method_calls;
                    report.queried_method_calls += file_report.queried_method_calls;
                    report.resolved_method_calls += file_report.resolved_method_calls;
                    report.callable_method_calls += file_report.callable_method_calls;
                    report.fallback_method_calls += file_report.fallback_method_calls;
                    report.unqueried_method_calls += file_report.unqueried_method_calls;
                    report.paths += file_report.paths;
                    report.queried_paths += file_report.queried_paths;
                    report.resolved_paths += file_report.resolved_paths;
                    report.unqueried_paths += file_report.unqueried_paths;
                }
                Err(_) => {
                    report.failed_files += 1;
                }
            }
        }

        report.unresolved_method_calls = report
            .queried_method_calls
            .saturating_sub(report.resolved_method_calls);
        report.unresolved_paths = report.queried_paths.saturating_sub(report.resolved_paths);
        report
    }

    fn collect_file_semantics(
        semantics: &ra_ap_ide::Semantics<'_, ra_ap_ide::RootDatabase>,
        file_id: ra_ap_ide::FileId,
        budget: &mut SemanticBudget,
    ) -> SemanticReport {
        let source = semantics.parse_guess_edition(file_id);
        let mut report = SemanticReport::default();

        for node in source.syntax().descendants() {
            if let Some(method_call) = ast::MethodCallExpr::cast(node.clone()) {
                report.method_calls += 1;
                if !budget.take_method_call() {
                    report.unqueried_method_calls += 1;
                    continue;
                }

                report.queried_method_calls += 1;
                if semantics.resolve_method_call(&method_call).is_some() {
                    report.resolved_method_calls += 1;
                }
                if semantics
                    .resolve_method_call_as_callable(&method_call)
                    .is_some()
                {
                    report.callable_method_calls += 1;
                }
                if semantics
                    .resolve_method_call_fallback(&method_call)
                    .is_some()
                {
                    report.fallback_method_calls += 1;
                }
            }

            if let Some(path) = ast::Path::cast(node) {
                report.paths += 1;
                if !budget.take_path() {
                    report.unqueried_paths += 1;
                    continue;
                }

                report.queried_paths += 1;
                if semantics.resolve_path(&path).is_some() {
                    report.resolved_paths += 1;
                }
            }
        }

        report
    }

    fn is_workspace_rust_file(
        vfs_path: &ra_ap_vfs::VfsPath,
        workspace_root: &Path,
        canonical_workspace_root: &Path,
    ) -> bool {
        let Some(abs_path) = vfs_path.as_path() else {
            return false;
        };
        let path: &Path = abs_path.as_ref();
        let is_rust_file = path.extension().and_then(OsStr::to_str) == Some("rs");
        let is_local =
            path.starts_with(workspace_root) || path.starts_with(canonical_workspace_root);
        let is_target_artifact = path
            .strip_prefix(workspace_root)
            .ok()
            .or_else(|| path.strip_prefix(canonical_workspace_root).ok())
            .is_some_and(|relative| {
                relative.components().any(|component| {
                    matches!(component, Component::Normal(name) if name == OsStr::new("target"))
                })
            });

        is_rust_file && is_local && !is_target_artifact
    }

    struct SemanticBudget {
        remaining_method_calls: usize,
        remaining_paths: usize,
    }

    impl SemanticBudget {
        fn take_method_call(&mut self) -> bool {
            if self.remaining_method_calls == 0 {
                return false;
            }
            self.remaining_method_calls -= 1;
            true
        }

        fn take_path(&mut self) -> bool {
            if self.remaining_paths == 0 {
                return false;
            }
            self.remaining_paths -= 1;
            true
        }
    }

    fn semantic_budget_from_env(name: &str, default: usize) -> usize {
        std::env::var(name)
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(default)
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{load_report, AnalyzerMode};

    #[test]
    fn parses_analyzer_modes() {
        assert_eq!("syn".parse::<AnalyzerMode>().unwrap(), AnalyzerMode::Syn);
        assert_eq!(
            "ra-hir".parse::<AnalyzerMode>().unwrap(),
            AnalyzerMode::RustAnalyzerHir
        );
        assert_eq!(
            "rust-analyzer".parse::<AnalyzerMode>().unwrap(),
            AnalyzerMode::RustAnalyzerHir
        );
        assert!("bogus".parse::<AnalyzerMode>().is_err());
    }

    #[test]
    fn syn_report_is_available_without_optional_analyzer() {
        let report = load_report(&workspace_root(), AnalyzerMode::Syn).unwrap();
        assert_eq!(report.mode, AnalyzerMode::Syn);
        assert!(report.loaded);
        assert_eq!(report.engine, "syn");
        assert!(report.semantic.is_none());
    }

    #[test]
    #[cfg(not(feature = "ra-hir"))]
    fn ra_hir_reports_clear_error_when_feature_is_disabled() {
        let error = load_report(&workspace_root(), AnalyzerMode::RustAnalyzerHir)
            .expect_err("ra-hir should require the ra-hir feature");
        assert!(error.to_string().contains("without the ra-hir feature"));
    }

    #[test]
    #[cfg(feature = "ra-hir")]
    fn ra_hir_loads_workspace_and_initializes_semantics() {
        let report = load_report(&workspace_root(), AnalyzerMode::RustAnalyzerHir).unwrap();
        assert_eq!(report.mode, AnalyzerMode::RustAnalyzerHir);
        assert!(report.loaded);
        assert_eq!(report.engine, "rust-analyzer HIR");
        assert!(report
            .notes
            .iter()
            .any(|note| note.contains("RootDatabase")));
        assert!(report.notes.iter().any(|note| note.contains("Semantics")));
        let semantic = report
            .semantic
            .as_ref()
            .expect("ra-hir should collect semantic inventory");
        assert!(semantic.source_files > 0);
        assert!(semantic.analyzed_files > 0);
        assert!(semantic.method_calls >= semantic.queried_method_calls);
        assert!(semantic.queried_method_calls >= semantic.resolved_method_calls);
        assert!(semantic.callable_method_calls >= semantic.resolved_method_calls);
        assert!(semantic.fallback_method_calls >= semantic.resolved_method_calls);
        assert!(semantic.paths >= semantic.queried_paths);
        assert!(semantic.queried_paths >= semantic.resolved_paths);
    }

    fn workspace_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .expect("core crate should live under crates/opensource_core")
            .to_path_buf()
    }
}

#[cfg(not(feature = "ra-hir"))]
mod rust_analyzer {
    use std::path::Path;

    use super::AnalyzerReport;

    pub fn load_report(
        _workspace_root: &Path,
    ) -> Result<AnalyzerReport, Box<dyn std::error::Error>> {
        Err(
            "ra-hir analyzer requested, but opensource_core was built without the ra-hir feature"
                .into(),
        )
    }
}
