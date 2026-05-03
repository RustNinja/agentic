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
}

impl AnalyzerReport {
    fn syn() -> Self {
        Self {
            mode: AnalyzerMode::Syn,
            loaded: true,
            engine: "syn".to_string(),
            notes: vec!["using syntactic resolver".to_string()],
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
        path::Path,
        sync::atomic::{AtomicUsize, Ordering},
    };

    use ra_ap_load_cargo::{load_workspace_at, LoadCargoConfig, ProcMacroServerChoice};
    use ra_ap_project_model::CargoConfig;

    use super::{AnalyzerMode, AnalyzerReport, SemanticProvider};

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

            let (database, _vfs, proc_macro_client) =
                load_workspace_at(workspace_root, &cargo_config, &load_config, &progress)?;

            let _semantics = ra_ap_ide::Semantics::new(&database);
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

            Ok(Self {
                report: AnalyzerReport {
                    mode: AnalyzerMode::RustAnalyzerHir,
                    loaded: true,
                    engine: "rust-analyzer HIR".to_string(),
                    notes,
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
