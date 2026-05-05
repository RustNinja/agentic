use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

use crate::model::{CallableId, ItemId, Project, SemanticOwnerId, SemanticReductionHints};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AnalyzerMode {
    Syn,
    RustAnalyzerHir,
    RustAnalyzerFeedback,
    RustAnalyzerHirProcMacros,
}

impl AnalyzerMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Syn => "syn",
            Self::RustAnalyzerHir => "ra-hir",
            Self::RustAnalyzerFeedback => "ra-feedback",
            Self::RustAnalyzerHirProcMacros => "ra-hir-proc-macros",
        }
    }

    pub fn default_for_build() -> Self {
        #[cfg(feature = "ra-hir")]
        {
            Self::RustAnalyzerHir
        }
        #[cfg(not(feature = "ra-hir"))]
        {
            Self::Syn
        }
    }

    pub fn production_default_for_build() -> Self {
        #[cfg(feature = "ra-hir")]
        {
            Self::RustAnalyzerHirProcMacros
        }
        #[cfg(not(feature = "ra-hir"))]
        {
            Self::Syn
        }
    }
}

impl Default for AnalyzerMode {
    fn default() -> Self {
        Self::default_for_build()
    }
}

impl std::str::FromStr for AnalyzerMode {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "syn" => Ok(Self::Syn),
            "ra" | "ra-hir" | "rust-analyzer" | "rust-analyzer-hir" => Ok(Self::RustAnalyzerHir),
            "ra-feedback"
            | "rust-analyzer-feedback"
            | "rust-analyzer-hir-feedback" => Ok(Self::RustAnalyzerFeedback),
            "ra-hir-proc-macros"
            | "ra-proc-macros"
            | "rust-analyzer-proc-macros"
            | "rust-analyzer-hir-proc-macros" => Ok(Self::RustAnalyzerHirProcMacros),
            _ => Err(format!(
                "unknown analyzer {value:?}; expected syn, ra-hir, ra-feedback, or ra-hir-proc-macros"
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
    pub semantic_hints: SemanticReductionHints,
    pub semantic_usage: Option<SemanticUsageReport>,
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
    pub selected_root_source_files: usize,
    pub selected_root_analyzed_files: usize,
    pub selected_root_failed_files: usize,
    pub selected_root_skipped_files: usize,
    pub selected_root_method_calls: usize,
    pub selected_root_queried_method_calls: usize,
    pub selected_root_resolved_method_calls: usize,
    pub selected_root_callable_method_calls: usize,
    pub selected_root_fallback_method_calls: usize,
    pub selected_root_unresolved_method_calls: usize,
    pub selected_root_unqueried_method_calls: usize,
    pub selected_root_paths: usize,
    pub selected_root_queried_paths: usize,
    pub selected_root_resolved_paths: usize,
    pub selected_root_unresolved_paths: usize,
    pub selected_root_unqueried_paths: usize,
    pub file_reports: Vec<SemanticFileReport>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct SemanticFileReport {
    pub path: PathBuf,
    pub selected_root_file: bool,
    pub analyzed: bool,
    pub failed: bool,
    pub skipped_by_file_budget: bool,
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

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct SemanticUsageReport {
    pub indexed_callables: usize,
    pub indexed_items: usize,
    pub mapped_callables: usize,
    pub mapped_items: usize,
    pub unmapped_callables: usize,
    pub unmapped_items: usize,
    pub reference_queries: usize,
    pub reference_query_failures: usize,
    pub callable_reference_edges: usize,
    pub item_reference_edges: usize,
    pub referenced_callables: usize,
    pub referenced_items: usize,
    pub mapped_callable_ids: BTreeSet<CallableId>,
    pub mapped_item_ids: BTreeSet<ItemId>,
    pub failed_callable_reference_ids: BTreeSet<CallableId>,
    pub failed_item_reference_ids: BTreeSet<ItemId>,
    pub referenced_callable_ids: BTreeSet<CallableId>,
    pub referenced_item_ids: BTreeSet<ItemId>,
    pub callable_reference_owners: BTreeMap<CallableId, BTreeSet<SemanticOwnerId>>,
    pub item_reference_owners: BTreeMap<ItemId, BTreeSet<SemanticOwnerId>>,
    pub callable_unowned_reference_files: BTreeMap<CallableId, BTreeSet<PathBuf>>,
    pub item_unowned_reference_files: BTreeMap<ItemId, BTreeSet<PathBuf>>,
}

impl SemanticUsageReport {
    pub fn is_callable_mapped(&self, callable: &CallableId) -> bool {
        self.mapped_callable_ids.contains(callable)
    }

    pub fn is_item_mapped(&self, item: &ItemId) -> bool {
        self.mapped_item_ids.contains(item)
    }

    pub fn callable_reference_query_failed(&self, callable: &CallableId) -> bool {
        self.failed_callable_reference_ids.contains(callable)
    }

    pub fn item_reference_query_failed(&self, item: &ItemId) -> bool {
        self.failed_item_reference_ids.contains(item)
    }

    pub fn callable_has_retained_reference(
        &self,
        callable: &CallableId,
        retained_callables: &BTreeSet<CallableId>,
        retained_items: &BTreeSet<ItemId>,
    ) -> bool {
        self.callable_reference_owners
            .get(callable)
            .is_some_and(|owners| {
                owners.iter().any(|owner| {
                    semantic_owner_is_retained(owner, retained_callables, retained_items)
                })
            })
    }

    pub fn item_has_retained_reference(
        &self,
        item: &ItemId,
        retained_callables: &BTreeSet<CallableId>,
        retained_items: &BTreeSet<ItemId>,
    ) -> bool {
        self.item_reference_owners.get(item).is_some_and(|owners| {
            owners
                .iter()
                .any(|owner| semantic_owner_is_retained(owner, retained_callables, retained_items))
        })
    }

    pub fn unmapped_total(&self) -> usize {
        self.unmapped_callables + self.unmapped_items
    }
}

fn semantic_owner_is_retained(
    owner: &SemanticOwnerId,
    retained_callables: &BTreeSet<CallableId>,
    retained_items: &BTreeSet<ItemId>,
) -> bool {
    match owner {
        SemanticOwnerId::Callable(callable) => retained_callables.contains(callable),
        SemanticOwnerId::Item(item) => retained_items.contains(item),
    }
}

impl AnalyzerReport {
    fn syn() -> Self {
        Self {
            mode: AnalyzerMode::Syn,
            loaded: true,
            engine: "syn".to_string(),
            notes: vec!["using syntactic resolver".to_string()],
            semantic: None,
            semantic_hints: SemanticReductionHints::default(),
            semantic_usage: None,
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

#[allow(dead_code)]
pub fn load_report(
    workspace_root: &Path,
    mode: AnalyzerMode,
) -> Result<AnalyzerReport, Box<dyn std::error::Error>> {
    load_report_with_project(workspace_root, mode, None)
}

pub fn load_report_for_project(
    workspace_root: &Path,
    mode: AnalyzerMode,
    project: &Project,
) -> Result<AnalyzerReport, Box<dyn std::error::Error>> {
    load_report_with_project(workspace_root, mode, Some(project))
}

fn load_report_with_project(
    workspace_root: &Path,
    mode: AnalyzerMode,
    project: Option<&Project>,
) -> Result<AnalyzerReport, Box<dyn std::error::Error>> {
    match mode {
        AnalyzerMode::Syn => {
            let provider = SynSemanticProvider::new();
            Ok(provider.report().clone())
        }
        AnalyzerMode::RustAnalyzerHir => rust_analyzer::load_report(
            workspace_root,
            project,
            AnalyzerMode::RustAnalyzerHir,
            rust_analyzer::ProcMacroExpansionMode::Disabled,
            rust_analyzer::RaFeedbackMode::Disabled,
        ),
        AnalyzerMode::RustAnalyzerFeedback => rust_analyzer::load_report(
            workspace_root,
            project,
            AnalyzerMode::RustAnalyzerFeedback,
            rust_analyzer::ProcMacroExpansionMode::Disabled,
            rust_analyzer::RaFeedbackMode::Enabled,
        ),
        AnalyzerMode::RustAnalyzerHirProcMacros => rust_analyzer::load_report(
            workspace_root,
            project,
            AnalyzerMode::RustAnalyzerHirProcMacros,
            rust_analyzer::ProcMacroExpansionMode::Enabled,
            rust_analyzer::RaFeedbackMode::Enabled,
        ),
    }
}

#[cfg(feature = "ra-hir")]
mod rust_analyzer {
    use std::{
        collections::{BTreeSet, HashMap},
        ffi::OsStr,
        fs,
        panic::{self, AssertUnwindSafe},
        path::{Component, Path, PathBuf},
        sync::{
            atomic::{AtomicUsize, Ordering},
            Arc, Mutex,
        },
    };

    use ra_ap_hir::{Adt, HasSource, ModuleDef, PathResolution};
    use ra_ap_load_cargo::{load_workspace_at, LoadCargoConfig, ProcMacroServerChoice};
    use ra_ap_project_model::CargoConfig;
    use ra_ap_syntax::{ast, AstNode, TextSize};

    use crate::{
        model::{
            CallableId, ItemId, ItemKind, Project, ReducedProject, RootId, SemanticOwnerId,
            SemanticReductionHints, SourceSpan,
        },
        reduce,
    };

    use super::{
        AnalyzerMode, AnalyzerReport, SemanticFileReport, SemanticProvider, SemanticReport,
        SemanticUsageReport,
    };

    const DEFAULT_SEMANTIC_FILE_BUDGET: usize = 48;
    const DEFAULT_METHOD_CALL_BUDGET: usize = 1_000;
    const DEFAULT_PATH_BUDGET: usize = 2_000;
    static RA_WORKSPACE_LOAD_PANIC_HOOK_LOCK: Mutex<()> = Mutex::new(());

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum ProcMacroExpansionMode {
        Disabled,
        Enabled,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum RaFeedbackMode {
        Disabled,
        Enabled,
    }

    pub struct RustAnalyzerSemanticProvider {
        report: AnalyzerReport,
        _database: ra_ap_ide::RootDatabase,
    }

    impl RustAnalyzerSemanticProvider {
        fn load(
            workspace_root: &Path,
            project: Option<&Project>,
            requested_mode: AnalyzerMode,
            proc_macro_mode: ProcMacroExpansionMode,
            feedback_mode: RaFeedbackMode,
        ) -> Result<Self, Box<dyn std::error::Error>> {
            if proc_macro_mode == ProcMacroExpansionMode::Enabled
                && !proc_macro_dependency_loading_enabled()
            {
                let mut provider = Self::load_once(
                    workspace_root,
                    project,
                    requested_mode,
                    ProcMacroExpansionMode::Disabled,
                    feedback_mode,
                )?;
                provider.report.notes.push(
                    "proc macro expansion requested, but dependency artifact discovery is opt-in; skipped proc-macro load in bounded default mode and continued with HIR semantics"
                        .to_string(),
                );
                return Ok(provider);
            }

            match Self::load_once(
                workspace_root,
                project,
                requested_mode,
                proc_macro_mode,
                feedback_mode,
            ) {
                Ok(provider) => Ok(provider),
                Err(error) if proc_macro_mode == ProcMacroExpansionMode::Enabled => {
                    let mut provider = Self::load_once(
                        workspace_root,
                        project,
                        requested_mode,
                        ProcMacroExpansionMode::Disabled,
                        feedback_mode,
                    )?;
                    provider.report.notes.push(format!(
                        "proc macro semantic load failed; fell back to bounded HIR without proc macro expansion: {error}"
                    ));
                    Ok(provider)
                }
                Err(error) => Err(error),
            }
        }

        fn load_once(
            workspace_root: &Path,
            project: Option<&Project>,
            requested_mode: AnalyzerMode,
            proc_macro_mode: ProcMacroExpansionMode,
            feedback_mode: RaFeedbackMode,
        ) -> Result<Self, Box<dyn std::error::Error>> {
            let load_dependencies_for_proc_macros = proc_macro_mode
                == ProcMacroExpansionMode::Enabled
                && proc_macro_dependency_loading_enabled();
            let cargo_config = CargoConfig {
                set_test: true,
                no_deps: !load_dependencies_for_proc_macros,
                ..CargoConfig::default()
            };
            let load_config = LoadCargoConfig {
                load_out_dirs_from_check: proc_macro_mode == ProcMacroExpansionMode::Enabled,
                with_proc_macro_server: match proc_macro_mode {
                    ProcMacroExpansionMode::Disabled => ProcMacroServerChoice::None,
                    ProcMacroExpansionMode::Enabled => ProcMacroServerChoice::Sysroot,
                },
                prefill_caches: false,
                num_worker_threads: 1,
                proc_macro_processes: 1,
            };
            let progress_events = AtomicUsize::new(0);
            let progress = |_: String| {
                progress_events.fetch_add(1, Ordering::Relaxed);
            };

            let _panic_hook_guard = RA_WORKSPACE_LOAD_PANIC_HOOK_LOCK
                .lock()
                .expect("RA workspace load panic hook mutex should not be poisoned");
            let previous_hook = panic::take_hook();
            let previous_hook = Arc::new(Mutex::new(Some(previous_hook)));
            let hook_previous = Arc::clone(&previous_hook);
            let load_thread = std::thread::current().id();
            panic::set_hook(Box::new(move |info| {
                if std::thread::current().id() != load_thread {
                    let guard = hook_previous
                        .lock()
                        .expect("RA workspace load panic hook should remain available");
                    if let Some(hook) = guard.as_ref() {
                        hook(info);
                    }
                }
            }));
            let loaded = panic::catch_unwind(AssertUnwindSafe(|| {
                load_workspace_at(workspace_root, &cargo_config, &load_config, &progress)
            }));
            let previous_hook = previous_hook
                .lock()
                .expect("RA workspace load panic hook should be restorable")
                .take()
                .expect("RA workspace load panic hook should be present");
            panic::set_hook(previous_hook);
            let loaded = loaded.map_err(|_| "rust-analyzer workspace load panicked")?;
            let (database, vfs, proc_macro_client) = loaded?;

            let semantic =
                collect_semantic_report(&database, &vfs, workspace_root, project, feedback_mode);
            let mut notes = vec![
                "rust-analyzer RootDatabase loaded".to_string(),
                "HIR Semantics initialized".to_string(),
            ];
            match proc_macro_mode {
                ProcMacroExpansionMode::Disabled => {
                    notes.push(
                        "dependency crates excluded from HIR load for bounded slicer analysis"
                            .to_string(),
                    );
                    notes.push(
                        "proc macro expansion disabled for fast bounded semantic inventory"
                            .to_string(),
                    );
                }
                ProcMacroExpansionMode::Enabled => {
                    if load_dependencies_for_proc_macros {
                        notes.push(
                            "Cargo dependency graph loaded for proc-macro/build-script discovery; semantic inventory remains workspace-file bounded"
                                .to_string(),
                        );
                    } else {
                        notes.push(
                            "dependency crates excluded from HIR load for bounded production semantic inventory; set OPENSOURCE_RA_PROC_MACRO_LOAD_DEPS=1 to allow full Cargo proc-macro discovery"
                                .to_string(),
                        );
                    }
                    notes.push(
                        "proc macro expansion requested through rust-analyzer sysroot proc-macro server with build-script output discovery"
                            .to_string(),
                    );
                }
            }
            notes.push(format!(
                "workspace load progress events: {}",
                progress_events.load(Ordering::Relaxed)
            ));
            notes.push(format!(
                "proc macro client active: {}",
                proc_macro_client.is_some()
            ));
            if proc_macro_mode == ProcMacroExpansionMode::Enabled && proc_macro_client.is_none() {
                notes.push(
                    "proc macro server unavailable; HIR semantic inventory continued without active proc macro expansion"
                        .to_string(),
                );
            }
            notes.push(format!(
                "HIR semantic inventory: {}/{} files analyzed, {} skipped by budget, {}/{} queried method calls resolved to functions, {}/{} callable, {}/{} fallback, {} method calls unqueried, {}/{} queried paths resolved, {} paths unqueried",
                semantic.report.analyzed_files,
                semantic.report.source_files,
                semantic.report.skipped_files,
                semantic.report.resolved_method_calls,
                semantic.report.queried_method_calls,
                semantic.report.callable_method_calls,
                semantic.report.queried_method_calls,
                semantic.report.fallback_method_calls,
                semantic.report.queried_method_calls,
                semantic.report.unqueried_method_calls,
                semantic.report.resolved_paths,
                semantic.report.queried_paths,
                semantic.report.unqueried_paths
            ));
            if semantic.report.selected_root_source_files > 0 {
                notes.push(format!(
                    "HIR selected-root semantic inventory: {}/{} root file(s) analyzed, {} failed, {} skipped by budget, {} unresolved method call(s), {} unqueried method call(s), {} unresolved path(s), {} unqueried path(s)",
                    semantic.report.selected_root_analyzed_files,
                    semantic.report.selected_root_source_files,
                    semantic.report.selected_root_failed_files,
                    semantic.report.selected_root_skipped_files,
                    semantic.report.selected_root_unresolved_method_calls,
                    semantic.report.selected_root_unqueried_method_calls,
                    semantic.report.selected_root_unresolved_paths,
                    semantic.report.selected_root_unqueried_paths
                ));
            }
            notes.push(format!(
                "HIR reduction hints: {} project-local semantic edge(s), {} unresolved query/queries, {} unqueried query/queries, {} unmapped target(s)",
                semantic.hints.total_edges(),
                semantic.hints.unresolved_queries,
                semantic.hints.unqueried_queries,
                semantic.hints.unmapped_targets
            ));
            if let Some(usage) = &semantic.usage {
                notes.push(format!(
                    "HIR usage mapping: {}/{} callable(s) and {}/{} item(s) mapped to rust-analyzer definitions; {} unmapped; {} reference query/queries, {} failure(s), {} promoted reference edge(s), {} referenced callable(s), {} referenced item(s)",
                    usage.mapped_callables,
                    usage.indexed_callables,
                    usage.mapped_items,
                    usage.indexed_items,
                    usage.unmapped_total(),
                    usage.reference_queries,
                    usage.reference_query_failures,
                    usage.callable_reference_edges + usage.item_reference_edges,
                    usage.referenced_callables,
                    usage.referenced_items
                ));
            }
            if let Some(feedback) = semantic.ra_feedback {
                notes.push(format!(
                    "RA feedback closure: {} callable owner(s) queried, {} outgoing call target(s), {} project-local edge(s) observed, {} unmapped target(s)",
                    feedback.queried_callables,
                    feedback.outgoing_calls,
                    feedback.edges,
                    feedback.unmapped_targets
                ));
            }
            notes.push(format!(
                "HIR semantic budgets: files={}, method_calls={}, paths={}",
                semantic.report.file_budget,
                semantic.report.method_call_budget,
                semantic.report.path_budget
            ));
            if semantic.report.failed_files > 0 {
                notes.push(format!(
                    "HIR semantic inventory skipped {} files after analyzer panics",
                    semantic.report.failed_files
                ));
            }

            Ok(Self {
                report: AnalyzerReport {
                    mode: requested_mode,
                    loaded: true,
                    engine: "rust-analyzer HIR".to_string(),
                    notes,
                    semantic: Some(semantic.report),
                    semantic_hints: semantic.hints,
                    semantic_usage: semantic.usage,
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
        project: Option<&Project>,
        requested_mode: AnalyzerMode,
        proc_macro_mode: ProcMacroExpansionMode,
        feedback_mode: RaFeedbackMode,
    ) -> Result<AnalyzerReport, Box<dyn std::error::Error>> {
        let provider = RustAnalyzerSemanticProvider::load(
            workspace_root,
            project,
            requested_mode,
            proc_macro_mode,
            feedback_mode,
        )?;
        Ok(provider.report().clone())
    }

    fn collect_semantic_report(
        database: &ra_ap_ide::RootDatabase,
        vfs: &ra_ap_vfs::Vfs,
        workspace_root: &Path,
        project: Option<&Project>,
        feedback_mode: RaFeedbackMode,
    ) -> SemanticCollection {
        ra_ap_hir::attach_db(database, || {
            collect_semantic_report_attached(database, vfs, workspace_root, project, feedback_mode)
        })
    }

    struct SemanticCollection {
        report: SemanticReport,
        hints: SemanticReductionHints,
        usage: Option<SemanticUsageReport>,
        ra_feedback: Option<RaFeedbackReport>,
    }

    #[derive(Debug, Clone, Copy, Default)]
    struct RaFeedbackReport {
        queried_callables: usize,
        outgoing_calls: usize,
        edges: usize,
        unmapped_targets: usize,
    }

    struct FileSemanticContext<'a> {
        database: &'a ra_ap_ide::RootDatabase,
        vfs: &'a ra_ap_vfs::Vfs,
        vfs_path: &'a ra_ap_vfs::VfsPath,
        semantics: &'a ra_ap_ide::Semantics<'a, ra_ap_ide::RootDatabase>,
        semantic_index: Option<&'a ProjectSemanticIndex>,
        hints: &'a mut SemanticReductionHints,
    }

    fn collect_semantic_report_attached(
        database: &ra_ap_ide::RootDatabase,
        vfs: &ra_ap_vfs::Vfs,
        workspace_root: &Path,
        project: Option<&Project>,
        feedback_mode: RaFeedbackMode,
    ) -> SemanticCollection {
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
        let semantic_index = project.map(ProjectSemanticIndex::build);
        let mut hints = SemanticReductionHints::default();

        let mut source_files = vfs
            .iter()
            .filter(|(_, vfs_path)| {
                is_workspace_rust_file(vfs_path, workspace_root, &canonical_workspace_root)
            })
            .collect::<Vec<_>>();
        if let Some(index) = semantic_index.as_ref() {
            source_files.sort_by_key(|(_, vfs_path)| index.file_priority(vfs_path));
        }

        for (file_id, vfs_path) in source_files {
            if !is_workspace_rust_file(vfs_path, workspace_root, &canonical_workspace_root) {
                continue;
            }

            let Some(path) = normalize_vfs_path(vfs_path) else {
                continue;
            };
            let selected_root_file = semantic_index
                .as_ref()
                .is_some_and(|index| index.is_root_file(vfs_path));
            let mut file_summary = SemanticFileReport {
                path,
                selected_root_file,
                ..SemanticFileReport::default()
            };
            report.source_files += 1;
            if selected_root_file {
                report.selected_root_source_files += 1;
            }
            if report.analyzed_files + report.failed_files >= report.file_budget {
                report.skipped_files += 1;
                file_summary.skipped_by_file_budget = true;
                if selected_root_file {
                    report.selected_root_skipped_files += 1;
                }
                report.file_reports.push(file_summary);
                continue;
            }

            match panic::catch_unwind(AssertUnwindSafe(|| {
                let mut context = FileSemanticContext {
                    database,
                    vfs,
                    vfs_path,
                    semantics: &semantics,
                    semantic_index: semantic_index.as_ref(),
                    hints: &mut hints,
                };
                collect_file_semantics(file_id, &mut budget, &mut context)
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
                    file_summary.analyzed = true;
                    file_summary.method_calls = file_report.method_calls;
                    file_summary.queried_method_calls = file_report.queried_method_calls;
                    file_summary.resolved_method_calls = file_report.resolved_method_calls;
                    file_summary.callable_method_calls = file_report.callable_method_calls;
                    file_summary.fallback_method_calls = file_report.fallback_method_calls;
                    file_summary.unresolved_method_calls = file_report
                        .queried_method_calls
                        .saturating_sub(file_report.resolved_method_calls);
                    file_summary.unqueried_method_calls = file_report.unqueried_method_calls;
                    file_summary.paths = file_report.paths;
                    file_summary.queried_paths = file_report.queried_paths;
                    file_summary.resolved_paths = file_report.resolved_paths;
                    file_summary.unresolved_paths = file_report
                        .queried_paths
                        .saturating_sub(file_report.resolved_paths);
                    file_summary.unqueried_paths = file_report.unqueried_paths;
                    if selected_root_file {
                        report.selected_root_analyzed_files += 1;
                        report.selected_root_method_calls += file_summary.method_calls;
                        report.selected_root_queried_method_calls +=
                            file_summary.queried_method_calls;
                        report.selected_root_resolved_method_calls +=
                            file_summary.resolved_method_calls;
                        report.selected_root_callable_method_calls +=
                            file_summary.callable_method_calls;
                        report.selected_root_fallback_method_calls +=
                            file_summary.fallback_method_calls;
                        report.selected_root_unresolved_method_calls +=
                            file_summary.unresolved_method_calls;
                        report.selected_root_unqueried_method_calls +=
                            file_summary.unqueried_method_calls;
                        report.selected_root_paths += file_summary.paths;
                        report.selected_root_queried_paths += file_summary.queried_paths;
                        report.selected_root_resolved_paths += file_summary.resolved_paths;
                        report.selected_root_unresolved_paths += file_summary.unresolved_paths;
                        report.selected_root_unqueried_paths += file_summary.unqueried_paths;
                    }
                    report.file_reports.push(file_summary);
                }
                Err(_) => {
                    report.failed_files += 1;
                    file_summary.failed = true;
                    if selected_root_file {
                        report.selected_root_failed_files += 1;
                    }
                    report.file_reports.push(file_summary);
                }
            }
        }

        report.unresolved_method_calls = report
            .queried_method_calls
            .saturating_sub(report.resolved_method_calls);
        report.unresolved_paths = report.queried_paths.saturating_sub(report.resolved_paths);
        hints.unresolved_queries = report.unresolved_method_calls + report.unresolved_paths;
        hints.unqueried_queries = report.unqueried_method_calls + report.unqueried_paths;
        let usage = project
            .zip(semantic_index.as_ref())
            .map(|(project, index)| {
                collect_semantic_usage_report(database, &semantics, vfs, project, index, &mut hints)
            });
        let ra_feedback = if feedback_mode == RaFeedbackMode::Enabled {
            project
                .zip(semantic_index.as_ref())
                .map(|(project, index)| {
                    collect_ra_feedback_edges(database, vfs, project, index, &mut hints)
                })
        } else {
            None
        };
        SemanticCollection {
            report,
            hints,
            usage,
            ra_feedback,
        }
    }

    fn collect_semantic_usage_report(
        database: &ra_ap_ide::RootDatabase,
        semantics: &ra_ap_ide::Semantics<'_, ra_ap_ide::RootDatabase>,
        vfs: &ra_ap_vfs::Vfs,
        project: &Project,
        index: &ProjectSemanticIndex,
        hints: &mut SemanticReductionHints,
    ) -> SemanticUsageReport {
        let mut report = SemanticUsageReport {
            indexed_callables: project.functions.len() + project.methods.len(),
            indexed_items: project.items.len(),
            ..SemanticUsageReport::default()
        };

        for (file_id, vfs_path) in vfs.iter() {
            if !index.contains_vfs_path(vfs_path) {
                continue;
            }
            let source = semantics.parse_guess_edition(file_id);
            for node in source.syntax().descendants() {
                if let Some(function) = ast::Fn::cast(node.clone()) {
                    if semantics.to_def(&function).is_some() {
                        if let Some(callable) = index.callable_at_vfs_offset(
                            vfs_path,
                            function.syntax().text_range().start(),
                        ) {
                            report.mapped_callable_ids.insert(callable);
                        }
                    }
                    continue;
                }
                if let Some(item) = mapped_item_at_node(semantics, vfs_path, index, node) {
                    report.mapped_item_ids.insert(item);
                }
            }
        }

        report.mapped_callables = report
            .mapped_callable_ids
            .iter()
            .filter(|id| project.functions.contains_key(*id) || project.methods.contains_key(*id))
            .count();
        report.mapped_items = report
            .mapped_item_ids
            .iter()
            .filter(|id| project.items.contains_key(*id))
            .count();
        report.unmapped_callables = report
            .indexed_callables
            .saturating_sub(report.mapped_callables);
        report.unmapped_items = report.indexed_items.saturating_sub(report.mapped_items);
        collect_semantic_reference_report(database, vfs, project, index, &mut report, hints);
        report
    }

    fn collect_semantic_reference_report(
        database: &ra_ap_ide::RootDatabase,
        vfs: &ra_ap_vfs::Vfs,
        project: &Project,
        index: &ProjectSemanticIndex,
        report: &mut SemanticUsageReport,
        hints: &mut SemanticReductionHints,
    ) {
        let analysis = ra_ap_ide::AnalysisHost::with_database(database.clone()).analysis();
        let file_ids = vfs_file_ids(vfs);
        let config = ra_ap_ide::FindAllRefsConfig {
            search_scope: None,
            ra_fixture: ra_ap_ide::RaFixtureConfig::default(),
            exclude_imports: false,
            exclude_tests: true,
        };

        let mut callables = project
            .functions
            .keys()
            .chain(project.methods.keys())
            .collect::<Vec<_>>();
        callables.sort();
        for callable in callables {
            if !report.is_callable_mapped(callable) {
                continue;
            }
            report.reference_queries += 1;
            match find_all_refs_from_candidates(
                &analysis,
                &file_ids,
                index.callable_focus_offsets(callable),
                &config,
            ) {
                Some(results) => {
                    collect_callable_reference_results(vfs, index, callable, results, report, hints)
                }
                None => record_callable_reference_query_failure(report, callable),
            }
        }

        let mut items = project.items.keys().collect::<Vec<_>>();
        items.sort();
        for item in items {
            if item.kind == ItemKind::Mod || !report.is_item_mapped(item) {
                continue;
            }
            report.reference_queries += 1;
            match find_all_refs_from_candidates(
                &analysis,
                &file_ids,
                index.item_focus_offsets(item),
                &config,
            ) {
                Some(results) => {
                    collect_item_reference_results(vfs, index, item, results, report, hints)
                }
                None => record_item_reference_query_failure(report, item),
            }
        }

        report.reference_query_failures =
            report.failed_callable_reference_ids.len() + report.failed_item_reference_ids.len();
        report.callable_reference_edges = report
            .callable_reference_owners
            .values()
            .map(BTreeSet::len)
            .sum();
        report.item_reference_edges = report
            .item_reference_owners
            .values()
            .map(BTreeSet::len)
            .sum();
        report.referenced_callables = report.referenced_callable_ids.len();
        report.referenced_items = report.referenced_item_ids.len();
    }

    fn find_all_refs_from_candidates(
        analysis: &ra_ap_ide::Analysis,
        file_ids: &HashMap<PathBuf, ra_ap_ide::FileId>,
        candidates: Vec<(PathBuf, TextSize)>,
        config: &ra_ap_ide::FindAllRefsConfig,
    ) -> Option<Vec<ra_ap_ide::ReferenceSearchResult>> {
        for (path, offset) in candidates {
            let Some(file_id) = file_ids.get(&path).copied() else {
                continue;
            };
            let position = ra_ap_ide::FilePosition { file_id, offset };
            if let Ok(Some(results)) = analysis.find_all_refs(position, config) {
                return Some(results);
            }
        }
        None
    }

    fn collect_callable_reference_results(
        vfs: &ra_ap_vfs::Vfs,
        index: &ProjectSemanticIndex,
        target: &CallableId,
        results: Vec<ra_ap_ide::ReferenceSearchResult>,
        report: &mut SemanticUsageReport,
        hints: &mut SemanticReductionHints,
    ) {
        for result in results {
            for (file_id, references) in result.references {
                let vfs_path = vfs.file_path(file_id);
                if !index.contains_vfs_path(vfs_path) {
                    continue;
                }
                for (range, _) in references {
                    match index.owner_at_vfs_offset(vfs_path, range.start()) {
                        Some(SemanticOwnerId::Callable(owner)) if owner == *target => {}
                        Some(owner) => {
                            let owner_for_hint = owner.clone();
                            report
                                .callable_reference_owners
                                .entry(target.clone())
                                .or_default()
                                .insert(owner);
                            hints.add_callable_edge(owner_for_hint, target.clone());
                            report.referenced_callable_ids.insert(target.clone());
                        }
                        None => {
                            if let Some(path) = normalize_vfs_path(vfs_path) {
                                report
                                    .callable_unowned_reference_files
                                    .entry(target.clone())
                                    .or_default()
                                    .insert(path);
                                report.referenced_callable_ids.insert(target.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    fn collect_item_reference_results(
        vfs: &ra_ap_vfs::Vfs,
        index: &ProjectSemanticIndex,
        target: &ItemId,
        results: Vec<ra_ap_ide::ReferenceSearchResult>,
        report: &mut SemanticUsageReport,
        hints: &mut SemanticReductionHints,
    ) {
        for result in results {
            for (file_id, references) in result.references {
                let vfs_path = vfs.file_path(file_id);
                if !index.contains_vfs_path(vfs_path) {
                    continue;
                }
                for (range, _) in references {
                    match index.owner_at_vfs_offset(vfs_path, range.start()) {
                        Some(SemanticOwnerId::Item(owner)) if owner == *target => {}
                        Some(owner) => {
                            let owner_for_hint = owner.clone();
                            report
                                .item_reference_owners
                                .entry(target.clone())
                                .or_default()
                                .insert(owner);
                            hints.add_item_edge(owner_for_hint, target.clone());
                            report.referenced_item_ids.insert(target.clone());
                        }
                        None => {
                            if let Some(path) = normalize_vfs_path(vfs_path) {
                                report
                                    .item_unowned_reference_files
                                    .entry(target.clone())
                                    .or_default()
                                    .insert(path);
                                report.referenced_item_ids.insert(target.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    fn record_callable_reference_query_failure(
        report: &mut SemanticUsageReport,
        callable: &CallableId,
    ) {
        report
            .failed_callable_reference_ids
            .insert(callable.clone());
    }

    fn record_item_reference_query_failure(report: &mut SemanticUsageReport, item: &ItemId) {
        report.failed_item_reference_ids.insert(item.clone());
    }

    fn mapped_item_at_node(
        semantics: &ra_ap_ide::Semantics<'_, ra_ap_ide::RootDatabase>,
        vfs_path: &ra_ap_vfs::VfsPath,
        index: &ProjectSemanticIndex,
        node: ra_ap_syntax::SyntaxNode,
    ) -> Option<ItemId> {
        if let Some(item) = ast::Struct::cast(node.clone()) {
            return semantics.to_def(&item).is_some().then(|| {
                index.item_at_vfs_offset(
                    vfs_path,
                    item.syntax().text_range().start(),
                    Some(ItemKind::Struct),
                )
            })?;
        }
        if let Some(item) = ast::Enum::cast(node.clone()) {
            return semantics.to_def(&item).is_some().then(|| {
                index.item_at_vfs_offset(
                    vfs_path,
                    item.syntax().text_range().start(),
                    Some(ItemKind::Enum),
                )
            })?;
        }
        if let Some(item) = ast::Union::cast(node.clone()) {
            return semantics.to_def(&item).is_some().then(|| {
                index.item_at_vfs_offset(
                    vfs_path,
                    item.syntax().text_range().start(),
                    Some(ItemKind::Union),
                )
            })?;
        }
        if let Some(item) = ast::Trait::cast(node.clone()) {
            return semantics.to_def(&item).is_some().then(|| {
                index.item_at_vfs_offset(
                    vfs_path,
                    item.syntax().text_range().start(),
                    Some(ItemKind::Trait),
                )
            })?;
        }
        if let Some(item) = ast::TypeAlias::cast(node.clone()) {
            return semantics.to_def(&item).is_some().then(|| {
                index.item_at_vfs_offset(
                    vfs_path,
                    item.syntax().text_range().start(),
                    Some(ItemKind::Type),
                )
            })?;
        }
        if let Some(item) = ast::Const::cast(node.clone()) {
            return semantics.to_def(&item).is_some().then(|| {
                index.item_at_vfs_offset(
                    vfs_path,
                    item.syntax().text_range().start(),
                    Some(ItemKind::Const),
                )
            })?;
        }
        if let Some(item) = ast::Static::cast(node.clone()) {
            return semantics.to_def(&item).is_some().then(|| {
                index.item_at_vfs_offset(
                    vfs_path,
                    item.syntax().text_range().start(),
                    Some(ItemKind::Static),
                )
            })?;
        }
        if let Some(item) = ast::Module::cast(node) {
            return semantics.to_def(&item).is_some().then(|| {
                index.item_at_vfs_offset(
                    vfs_path,
                    item.syntax().text_range().start(),
                    Some(ItemKind::Mod),
                )
            })?;
        }
        None
    }

    fn collect_ra_feedback_edges(
        database: &ra_ap_ide::RootDatabase,
        vfs: &ra_ap_vfs::Vfs,
        project: &Project,
        index: &ProjectSemanticIndex,
        hints: &mut SemanticReductionHints,
    ) -> RaFeedbackReport {
        let analysis = ra_ap_ide::AnalysisHost::with_database(database.clone()).analysis();
        let file_ids = vfs_file_ids(vfs);
        let config = ra_ap_ide::CallHierarchyConfig {
            exclude_tests: true,
            ra_fixture: ra_ap_ide::RaFixtureConfig::default(),
        };
        let mut report = RaFeedbackReport::default();
        let mut callables = project
            .functions
            .keys()
            .chain(project.methods.keys())
            .collect::<Vec<_>>();
        callables.sort();

        for callable in callables {
            let candidates = index
                .callable_focus_offsets(callable)
                .into_iter()
                .filter(|(path, _)| index.is_feedback_owner_path(path))
                .collect::<Vec<_>>();
            if candidates.is_empty() {
                continue;
            }
            report.queried_callables += 1;
            let Some(outgoing) =
                outgoing_calls_from_candidates(&analysis, &file_ids, candidates, &config)
            else {
                continue;
            };
            for call in outgoing {
                report.outgoing_calls += 1;
                let target_vfs_path = vfs.file_path(call.target.file_id);
                let target_offset = call
                    .target
                    .focus_range
                    .unwrap_or(call.target.full_range)
                    .start();
                match index.callable_at_vfs_offset(target_vfs_path, target_offset) {
                    Some(dependency) if dependency != *callable => {
                        report.edges += 1;
                        hints.add_callable_edge(
                            SemanticOwnerId::Callable(callable.clone()),
                            dependency,
                        );
                    }
                    Some(_) => {}
                    None if index.contains_vfs_path(target_vfs_path) => {
                        report.unmapped_targets += 1;
                    }
                    None => {}
                }
            }
        }

        report
    }

    fn outgoing_calls_from_candidates(
        analysis: &ra_ap_ide::Analysis,
        file_ids: &HashMap<PathBuf, ra_ap_ide::FileId>,
        candidates: Vec<(PathBuf, TextSize)>,
        config: &ra_ap_ide::CallHierarchyConfig<'_>,
    ) -> Option<Vec<ra_ap_ide::CallItem>> {
        for (path, offset) in candidates {
            let Some(file_id) = file_ids.get(&path).copied() else {
                continue;
            };
            let position = ra_ap_ide::FilePosition { file_id, offset };
            if let Ok(Some(outgoing)) = analysis.outgoing_calls(config, position) {
                return Some(outgoing);
            }
        }
        None
    }

    fn vfs_file_ids(vfs: &ra_ap_vfs::Vfs) -> HashMap<PathBuf, ra_ap_ide::FileId> {
        vfs.iter()
            .filter_map(|(file_id, vfs_path)| {
                normalize_vfs_path(vfs_path).map(|path| (path, file_id))
            })
            .collect()
    }

    fn collect_file_semantics(
        file_id: ra_ap_ide::FileId,
        budget: &mut SemanticBudget,
        context: &mut FileSemanticContext<'_>,
    ) -> SemanticReport {
        let source = context.semantics.parse_guess_edition(file_id);
        let mut report = SemanticReport::default();

        for node in source.syntax().descendants() {
            if let Some(method_call) = ast::MethodCallExpr::cast(node.clone()) {
                report.method_calls += 1;
                if !budget.take_method_call() {
                    report.unqueried_method_calls += 1;
                    continue;
                }

                report.queried_method_calls += 1;
                if let Some(function) = context.semantics.resolve_method_call(&method_call) {
                    report.resolved_method_calls += 1;
                    if let Some(index) = context.semantic_index {
                        add_resolved_function_hint(
                            context.database,
                            context.vfs,
                            context.vfs_path,
                            method_call.syntax().text_range().start(),
                            function,
                            index,
                            context.hints,
                        );
                    }
                }
                if context
                    .semantics
                    .resolve_method_call_as_callable(&method_call)
                    .is_some()
                {
                    report.callable_method_calls += 1;
                }
                if context
                    .semantics
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
                if let Some(resolution) = context.semantics.resolve_path(&path) {
                    report.resolved_paths += 1;
                    if let Some(index) = context.semantic_index {
                        add_resolved_path_hint(
                            context.database,
                            context.vfs,
                            context.vfs_path,
                            path.syntax().text_range().start(),
                            resolution,
                            index,
                            context.hints,
                        );
                    }
                }
            }
        }

        report
    }

    fn add_resolved_function_hint(
        database: &ra_ap_ide::RootDatabase,
        vfs: &ra_ap_vfs::Vfs,
        owner_vfs_path: &ra_ap_vfs::VfsPath,
        owner_offset: TextSize,
        function: ra_ap_hir::Function,
        index: &ProjectSemanticIndex,
        hints: &mut SemanticReductionHints,
    ) {
        let Some(owner) = index.owner_at_vfs_offset(owner_vfs_path, owner_offset) else {
            return;
        };
        let Some(source) = function.source(database) else {
            return;
        };
        let source_file_id = source.file_id.original_file(database).file_id(database);
        let source_vfs_path = vfs.file_path(source_file_id);
        let source_offset = source.value.syntax().text_range().start();
        match index.callable_at_vfs_offset(source_vfs_path, source_offset) {
            Some(dependency) => {
                if SemanticOwnerId::Callable(dependency.clone()) != owner {
                    hints.add_callable_edge(owner, dependency);
                }
            }
            None if index.contains_vfs_path(source_vfs_path) => {
                hints.unmapped_targets += 1;
            }
            None => {}
        }
    }

    fn add_resolved_path_hint(
        database: &ra_ap_ide::RootDatabase,
        vfs: &ra_ap_vfs::Vfs,
        owner_vfs_path: &ra_ap_vfs::VfsPath,
        owner_offset: TextSize,
        resolution: PathResolution,
        index: &ProjectSemanticIndex,
        hints: &mut SemanticReductionHints,
    ) {
        let Some(owner) = index.owner_at_vfs_offset(owner_vfs_path, owner_offset) else {
            return;
        };
        match resolution {
            PathResolution::Def(ModuleDef::Function(function)) => {
                let Some(source) = function.source(database) else {
                    return;
                };
                let source_file_id = source.file_id.original_file(database).file_id(database);
                let source_vfs_path = vfs.file_path(source_file_id);
                let source_offset = source.value.syntax().text_range().start();
                match index.callable_at_vfs_offset(source_vfs_path, source_offset) {
                    Some(dependency) => {
                        if SemanticOwnerId::Callable(dependency.clone()) != owner {
                            hints.add_callable_edge(owner, dependency);
                        }
                    }
                    None if index.contains_vfs_path(source_vfs_path) => {
                        hints.unmapped_targets += 1;
                    }
                    None => {}
                }
            }
            PathResolution::Def(definition) => {
                if let Some(dependency) = item_from_module_def(database, vfs, definition, index) {
                    if SemanticOwnerId::Item(dependency.clone()) != owner {
                        hints.add_item_edge(owner, dependency);
                    }
                }
            }
            _ => {}
        }
    }

    fn item_from_module_def(
        database: &ra_ap_ide::RootDatabase,
        vfs: &ra_ap_vfs::Vfs,
        definition: ModuleDef,
        index: &ProjectSemanticIndex,
    ) -> Option<ItemId> {
        match definition {
            ModuleDef::Adt(Adt::Struct(definition)) => {
                let source = definition.source(database)?;
                item_from_source(
                    database,
                    vfs,
                    source.file_id,
                    source.value.syntax().text_range().start(),
                    index,
                    Some(ItemKind::Struct),
                )
            }
            ModuleDef::Adt(Adt::Enum(definition)) => {
                let source = definition.source(database)?;
                item_from_source(
                    database,
                    vfs,
                    source.file_id,
                    source.value.syntax().text_range().start(),
                    index,
                    Some(ItemKind::Enum),
                )
            }
            ModuleDef::Adt(Adt::Union(definition)) => {
                let source = definition.source(database)?;
                item_from_source(
                    database,
                    vfs,
                    source.file_id,
                    source.value.syntax().text_range().start(),
                    index,
                    Some(ItemKind::Union),
                )
            }
            ModuleDef::EnumVariant(definition) => {
                let source = definition.source(database)?;
                item_from_source(
                    database,
                    vfs,
                    source.file_id,
                    source.value.syntax().text_range().start(),
                    index,
                    Some(ItemKind::Enum),
                )
            }
            ModuleDef::Const(definition) => {
                let source = definition.source(database)?;
                item_from_source(
                    database,
                    vfs,
                    source.file_id,
                    source.value.syntax().text_range().start(),
                    index,
                    Some(ItemKind::Const),
                )
            }
            ModuleDef::Static(definition) => {
                let source = definition.source(database)?;
                item_from_source(
                    database,
                    vfs,
                    source.file_id,
                    source.value.syntax().text_range().start(),
                    index,
                    Some(ItemKind::Static),
                )
            }
            ModuleDef::Trait(definition) => {
                let source = definition.source(database)?;
                item_from_source(
                    database,
                    vfs,
                    source.file_id,
                    source.value.syntax().text_range().start(),
                    index,
                    Some(ItemKind::Trait),
                )
            }
            ModuleDef::TypeAlias(definition) => {
                let source = definition.source(database)?;
                item_from_source(
                    database,
                    vfs,
                    source.file_id,
                    source.value.syntax().text_range().start(),
                    index,
                    Some(ItemKind::Type),
                )
            }
            ModuleDef::Module(_)
            | ModuleDef::Macro(_)
            | ModuleDef::BuiltinType(_)
            | ModuleDef::Function(_) => None,
        }
    }

    fn item_from_source(
        database: &ra_ap_ide::RootDatabase,
        vfs: &ra_ap_vfs::Vfs,
        file_id: ra_ap_hir::HirFileId,
        offset: TextSize,
        index: &ProjectSemanticIndex,
        kind: Option<ItemKind>,
    ) -> Option<ItemId> {
        let source_file_id = file_id.original_file(database).file_id(database);
        let source_vfs_path = vfs.file_path(source_file_id);
        index.item_at_vfs_offset(source_vfs_path, offset, kind)
    }

    struct ProjectSemanticIndex {
        files: HashMap<PathBuf, IndexedSourceFile>,
        root_files: BTreeSet<PathBuf>,
        retained_files: BTreeSet<PathBuf>,
    }

    impl ProjectSemanticIndex {
        fn build(project: &Project) -> Self {
            let mut files = HashMap::new();
            let mut root_files = BTreeSet::new();
            let retained_files = syntactic_retained_file_paths(project);
            for source in project.files.values() {
                let path = normalize_fs_path(&source.path);
                files.entry(path).or_insert_with(|| IndexedSourceFile {
                    text: fs::read_to_string(&source.path).unwrap_or_default(),
                    callables: Vec::new(),
                    items: Vec::new(),
                });
            }

            for (id, record) in &project.functions {
                if has_opensourced_attr(&record.item.attrs) {
                    root_files.insert(normalize_fs_path(&record.span.file));
                }
                if let Some(file) = files.get_mut(&normalize_fs_path(&record.span.file)) {
                    file.callables.push(IndexedCallable {
                        id: id.clone(),
                        span: record.span.clone(),
                    });
                }
            }
            for (id, record) in &project.methods {
                if has_opensourced_attr(&record.item.attrs) {
                    root_files.insert(normalize_fs_path(&record.span.file));
                }
                if let Some(file) = files.get_mut(&normalize_fs_path(&record.span.file)) {
                    file.callables.push(IndexedCallable {
                        id: id.clone(),
                        span: record.span.clone(),
                    });
                }
            }
            for (id, record) in &project.items {
                if item_has_opensourced_attr(&record.item) {
                    root_files.insert(normalize_fs_path(&record.span.file));
                }
                if let Some(file) = files.get_mut(&normalize_fs_path(&record.span.file)) {
                    file.items.push(IndexedItem {
                        id: id.clone(),
                        span: record.span.clone(),
                    });
                }
            }

            for file in files.values_mut() {
                file.callables
                    .sort_by_key(|callable| span_extent(&callable.span));
                file.items.sort_by_key(|item| span_extent(&item.span));
            }

            Self {
                files,
                root_files,
                retained_files,
            }
        }

        fn contains_vfs_path(&self, vfs_path: &ra_ap_vfs::VfsPath) -> bool {
            self.indexed_file(vfs_path).is_some()
        }

        fn file_priority(&self, vfs_path: &ra_ap_vfs::VfsPath) -> usize {
            let Some(path) = normalize_vfs_path(vfs_path) else {
                return 2;
            };
            if self.root_files.contains(&path) {
                0
            } else if self.retained_files.contains(&path) {
                1
            } else {
                2
            }
        }

        fn is_root_file(&self, vfs_path: &ra_ap_vfs::VfsPath) -> bool {
            normalize_vfs_path(vfs_path).is_some_and(|path| self.root_files.contains(&path))
        }

        fn is_feedback_owner_path(&self, path: &Path) -> bool {
            self.root_files.contains(path) || self.retained_files.contains(path)
        }

        fn owner_at_vfs_offset(
            &self,
            vfs_path: &ra_ap_vfs::VfsPath,
            offset: TextSize,
        ) -> Option<SemanticOwnerId> {
            let file = self.indexed_file(vfs_path)?;
            let (line, column) = file.line_column(offset);
            file.callables
                .iter()
                .find(|callable| span_contains(&callable.span, line, column))
                .map(|callable| SemanticOwnerId::Callable(callable.id.clone()))
                .or_else(|| {
                    file.items
                        .iter()
                        .find(|item| span_contains(&item.span, line, column))
                        .map(|item| SemanticOwnerId::Item(item.id.clone()))
                })
        }

        fn callable_at_vfs_offset(
            &self,
            vfs_path: &ra_ap_vfs::VfsPath,
            offset: TextSize,
        ) -> Option<CallableId> {
            let file = self.indexed_file(vfs_path)?;
            let (line, column) = file.line_column(offset);
            file.callables
                .iter()
                .find(|callable| span_contains(&callable.span, line, column))
                .map(|callable| callable.id.clone())
        }

        fn item_at_vfs_offset(
            &self,
            vfs_path: &ra_ap_vfs::VfsPath,
            offset: TextSize,
            kind: Option<ItemKind>,
        ) -> Option<ItemId> {
            let file = self.indexed_file(vfs_path)?;
            let (line, column) = file.line_column(offset);
            file.items
                .iter()
                .find(|item| {
                    kind.is_none_or(|kind| item.id.kind == kind)
                        && span_contains(&item.span, line, column)
                })
                .map(|item| item.id.clone())
        }

        fn callable_focus_offsets(&self, callable: &CallableId) -> Vec<(PathBuf, TextSize)> {
            let name = callable_name(callable);
            for (path, file) in &self.files {
                let Some(indexed) = file
                    .callables
                    .iter()
                    .find(|indexed| indexed.id == *callable)
                else {
                    continue;
                };
                let start = file.byte_offset(indexed.span.start_line, indexed.span.start_column);
                let end = file.byte_offset(indexed.span.end_line, indexed.span.end_column);
                let range = file.text.get(start..end).unwrap_or_default();
                return focus_offsets_for_name(path, start, range, name, &["fn"]);
            }
            Vec::new()
        }

        fn item_focus_offsets(&self, item: &ItemId) -> Vec<(PathBuf, TextSize)> {
            let name = &item.name;
            for (path, file) in &self.files {
                let Some(indexed) = file.items.iter().find(|indexed| indexed.id == *item) else {
                    continue;
                };
                let start = file.byte_offset(indexed.span.start_line, indexed.span.start_column);
                let end = file.byte_offset(indexed.span.end_line, indexed.span.end_column);
                let range = file.text.get(start..end).unwrap_or_default();
                return focus_offsets_for_name(
                    path,
                    start,
                    range,
                    name,
                    item_declaration_keywords(item.kind),
                );
            }
            Vec::new()
        }

        fn indexed_file(&self, vfs_path: &ra_ap_vfs::VfsPath) -> Option<&IndexedSourceFile> {
            let path = normalize_vfs_path(vfs_path)?;
            self.files.get(&path)
        }
    }

    fn callable_name(callable: &CallableId) -> &str {
        match callable {
            CallableId::Free { name, .. } => name,
            CallableId::Method { method, .. } => method,
        }
    }

    fn syntactic_retained_file_paths(project: &Project) -> BTreeSet<PathBuf> {
        reduce::reduce_with_extra_roots(project, &[])
            .map(|reduced| retained_file_paths(project, &reduced))
            .unwrap_or_default()
    }

    fn retained_file_paths(project: &Project, reduced: &ReducedProject) -> BTreeSet<PathBuf> {
        let mut paths = BTreeSet::new();
        for root in &reduced.roots {
            match root {
                RootId::Callable(callable) => {
                    add_callable_source_path(project, callable, &mut paths);
                }
                RootId::Item(item) => {
                    add_item_source_path(project, item, &mut paths);
                }
            }
        }
        for callable in &reduced.reachable {
            add_callable_source_path(project, callable, &mut paths);
        }
        for item in &reduced.reachable_items {
            add_item_source_path(project, item, &mut paths);
        }
        paths
    }

    fn add_callable_source_path(
        project: &Project,
        callable: &CallableId,
        paths: &mut BTreeSet<PathBuf>,
    ) {
        if let Some(record) = project.functions.get(callable) {
            paths.insert(normalize_fs_path(&record.span.file));
        }
        if let Some(record) = project.methods.get(callable) {
            paths.insert(normalize_fs_path(&record.span.file));
        }
    }

    fn add_item_source_path(project: &Project, item: &ItemId, paths: &mut BTreeSet<PathBuf>) {
        if let Some(record) = project.items.get(item) {
            paths.insert(normalize_fs_path(&record.span.file));
        }
    }

    struct IndexedSourceFile {
        text: String,
        callables: Vec<IndexedCallable>,
        items: Vec<IndexedItem>,
    }

    impl IndexedSourceFile {
        fn line_column(&self, offset: TextSize) -> (usize, usize) {
            let offset = text_size_to_usize(offset).min(self.text.len());
            let mut line = 1;
            let mut line_start = 0;
            for (index, ch) in self.text.char_indices() {
                if index >= offset {
                    break;
                }
                if ch == '\n' {
                    line += 1;
                    line_start = index + 1;
                }
            }
            (line, offset.saturating_sub(line_start))
        }

        fn byte_offset(&self, line: usize, column: usize) -> usize {
            let mut current_line = 1;
            let mut line_start = 0;
            for (index, ch) in self.text.char_indices() {
                if current_line == line {
                    return (line_start + column).min(self.text.len());
                }
                if ch == '\n' {
                    current_line += 1;
                    line_start = index + 1;
                }
            }
            if current_line == line {
                (line_start + column).min(self.text.len())
            } else {
                self.text.len()
            }
        }
    }

    fn focus_offsets_for_name(
        path: &Path,
        start: usize,
        range: &str,
        name: &str,
        declaration_keywords: &[&str],
    ) -> Vec<(PathBuf, TextSize)> {
        let mut declaration_offsets = Vec::new();
        let mut fallback_offsets = Vec::new();
        for relative in identifier_occurrences(range, name) {
            let offset = TextSize::new((start + relative) as u32);
            if is_declaration_name_occurrence(range, relative, declaration_keywords) {
                declaration_offsets.push(offset);
            } else {
                fallback_offsets.push(offset);
            }
        }
        declaration_offsets.extend(fallback_offsets);
        declaration_offsets.dedup();
        if declaration_offsets.is_empty() {
            declaration_offsets.push(TextSize::new(start as u32));
        }
        declaration_offsets
            .into_iter()
            .map(|offset| (path.to_path_buf(), offset))
            .collect()
    }

    fn identifier_occurrences(haystack: &str, needle: &str) -> Vec<usize> {
        if needle.is_empty() {
            return Vec::new();
        }
        let mut occurrences = Vec::new();
        let mut cursor = 0;
        while let Some(relative) = haystack[cursor..].find(needle) {
            let start = cursor + relative;
            let end = start + needle.len();
            if is_identifier_boundary(haystack, start, end) {
                occurrences.push(start);
            }
            cursor = end;
        }
        occurrences
    }

    fn is_identifier_boundary(text: &str, start: usize, end: usize) -> bool {
        !previous_char(text, start).is_some_and(is_rust_ident_continue)
            && !next_char(text, end).is_some_and(is_rust_ident_continue)
    }

    fn previous_char(text: &str, offset: usize) -> Option<char> {
        text.get(..offset)?.chars().next_back()
    }

    fn next_char(text: &str, offset: usize) -> Option<char> {
        text.get(offset..)?.chars().next()
    }

    fn is_rust_ident_continue(ch: char) -> bool {
        ch == '_' || ch.is_ascii_alphanumeric()
    }

    fn is_declaration_name_occurrence(
        range: &str,
        relative: usize,
        declaration_keywords: &[&str],
    ) -> bool {
        previous_word(range, relative).is_some_and(|word| declaration_keywords.contains(&word))
    }

    fn previous_word(text: &str, offset: usize) -> Option<&str> {
        let prefix = text.get(..offset)?.trim_end();
        let end = prefix.len();
        let start = prefix
            .char_indices()
            .rev()
            .find_map(|(index, ch)| (!is_rust_ident_continue(ch)).then_some(index + ch.len_utf8()))
            .unwrap_or(0);
        (start < end).then(|| &prefix[start..end])
    }

    fn item_declaration_keywords(kind: ItemKind) -> &'static [&'static str] {
        match kind {
            ItemKind::Struct => &["struct"],
            ItemKind::Enum => &["enum"],
            ItemKind::Union => &["union"],
            ItemKind::Type => &["type"],
            ItemKind::Trait => &["trait"],
            ItemKind::Mod => &["mod"],
            ItemKind::Const => &["const"],
            ItemKind::Static => &["static"],
            ItemKind::Macro => &[],
        }
    }

    struct IndexedCallable {
        id: CallableId,
        span: SourceSpan,
    }

    struct IndexedItem {
        id: ItemId,
        span: SourceSpan,
    }

    fn item_has_opensourced_attr(item: &syn::Item) -> bool {
        let attrs = match item {
            syn::Item::Const(item) => &item.attrs,
            syn::Item::Enum(item) => &item.attrs,
            syn::Item::Macro(item) => &item.attrs,
            syn::Item::Mod(item) => &item.attrs,
            syn::Item::Static(item) => &item.attrs,
            syn::Item::Struct(item) => &item.attrs,
            syn::Item::Trait(item) => &item.attrs,
            syn::Item::Type(item) => &item.attrs,
            syn::Item::Union(item) => &item.attrs,
            _ => return false,
        };
        has_opensourced_attr(attrs)
    }

    fn has_opensourced_attr(attrs: &[syn::Attribute]) -> bool {
        attrs.iter().any(|attribute| {
            attribute
                .path()
                .segments
                .last()
                .is_some_and(|segment| segment.ident == "opensourced")
        })
    }

    fn normalize_vfs_path(vfs_path: &ra_ap_vfs::VfsPath) -> Option<PathBuf> {
        let path: &Path = vfs_path.as_path()?.as_ref();
        Some(normalize_fs_path(path))
    }

    fn normalize_fs_path(path: &Path) -> PathBuf {
        path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
    }

    fn span_contains(span: &SourceSpan, line: usize, column: usize) -> bool {
        (line > span.start_line || (line == span.start_line && column >= span.start_column))
            && (line < span.end_line || (line == span.end_line && column <= span.end_column))
    }

    fn span_extent(span: &SourceSpan) -> usize {
        span.end_line
            .saturating_sub(span.start_line)
            .saturating_mul(10_000)
            + span.end_column.saturating_sub(span.start_column)
    }

    fn text_size_to_usize(offset: TextSize) -> usize {
        u32::from(offset) as usize
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

    fn proc_macro_dependency_loading_enabled() -> bool {
        env_flag_is_enabled("OPENSOURCE_RA_PROC_MACRO_LOAD_DEPS")
    }

    fn env_flag_is_enabled(name: &str) -> bool {
        std::env::var(name)
            .map(|value| {
                matches!(
                    value.trim().to_ascii_lowercase().as_str(),
                    "1" | "true" | "yes" | "on"
                )
            })
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    #[cfg(feature = "ra-hir")]
    use super::load_report_for_project;
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
        assert_eq!(
            "ra-feedback".parse::<AnalyzerMode>().unwrap(),
            AnalyzerMode::RustAnalyzerFeedback
        );
        assert_eq!(
            "ra-hir-proc-macros".parse::<AnalyzerMode>().unwrap(),
            AnalyzerMode::RustAnalyzerHirProcMacros
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
        let workspace_root = workspace_root();
        let workspace = crate::manifest::load_workspace(&workspace_root).unwrap();
        let project = crate::parse::parse_workspace(workspace).unwrap();
        let report =
            load_report_for_project(&workspace_root, AnalyzerMode::RustAnalyzerHir, &project)
                .unwrap();
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
        assert!(report.semantic_hints.total_edges() > 0);
        assert!(!report.semantic_hints.callable_edges.is_empty());
    }

    #[test]
    #[cfg(feature = "ra-hir")]
    fn ra_hir_proc_macro_mode_skips_dependency_artifacts_by_default() {
        if proc_macro_dependency_loading_enabled_for_test() {
            return;
        }

        let workspace_root = workspace_root();
        let workspace = crate::manifest::load_workspace(&workspace_root).unwrap();
        let project = crate::parse::parse_workspace(workspace).unwrap();
        let report = load_report_for_project(
            &workspace_root,
            AnalyzerMode::RustAnalyzerHirProcMacros,
            &project,
        )
        .unwrap();

        assert_eq!(report.mode, AnalyzerMode::RustAnalyzerHirProcMacros);
        assert!(report.loaded);
        assert!(report
            .notes
            .iter()
            .any(|note| { note.contains("skipped proc-macro load in bounded default mode") }));
        assert!(
            !report
                .notes
                .iter()
                .any(|note| note.contains("workspace load panicked")),
            "bounded default mode should not attempt the known incompatible proc-macro load path: {:?}",
            report.notes
        );
    }

    #[cfg(feature = "ra-hir")]
    fn proc_macro_dependency_loading_enabled_for_test() -> bool {
        std::env::var("OPENSOURCE_RA_PROC_MACRO_LOAD_DEPS")
            .map(|value| {
                matches!(
                    value.trim().to_ascii_lowercase().as_str(),
                    "1" | "true" | "yes" | "on"
                )
            })
            .unwrap_or(false)
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

    use crate::model::Project;

    use super::{AnalyzerMode, AnalyzerReport};

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum ProcMacroExpansionMode {
        Disabled,
        Enabled,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum RaFeedbackMode {
        Disabled,
        Enabled,
    }

    pub fn load_report(
        _workspace_root: &Path,
        _project: Option<&Project>,
        _requested_mode: AnalyzerMode,
        _proc_macro_mode: ProcMacroExpansionMode,
        _feedback_mode: RaFeedbackMode,
    ) -> Result<AnalyzerReport, Box<dyn std::error::Error>> {
        Err(
            "ra-hir analyzer requested, but opensource_core was built without the ra-hir feature"
                .into(),
        )
    }
}
