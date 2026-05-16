use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

use crate::model::{CallableId, ItemId, Project, RootId, SemanticOwnerId, SemanticReductionHints};

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
    pub top_down_skipped_files: usize,
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
    pub unresolved_diagnostics: Vec<SemanticUnresolvedDiagnostic>,
    pub file_reports: Vec<SemanticFileReport>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct SemanticFileReport {
    pub path: PathBuf,
    pub selected_root_file: bool,
    pub analyzed: bool,
    pub failed: bool,
    pub skipped_by_file_budget: bool,
    pub skipped_by_top_down_scope: bool,
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
    pub unresolved_diagnostics: Vec<SemanticUnresolvedDiagnostic>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SemanticUnresolvedDiagnostic {
    pub kind: SemanticUnresolvedKind,
    pub category: SemanticUnresolvedCategory,
    pub reason: String,
    pub file: PathBuf,
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
    pub snippet: String,
    pub ast_kind: String,
    pub symbol: Option<String>,
    pub owner: Option<SemanticOwnerId>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SemanticUnresolvedKind {
    MethodCall,
    Path,
}

impl SemanticUnresolvedKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MethodCall => "method_call",
            Self::Path => "path",
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SemanticUnresolvedCategory {
    Benign,
    MacroBlocked,
    DependencyRisk,
}

impl SemanticUnresolvedCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Benign => "benign",
            Self::MacroBlocked => "macro_blocked",
            Self::DependencyRisk => "dependency_risk",
        }
    }
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
    pub reference_queries_skipped: usize,
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

    pub fn reference_search_complete(&self) -> bool {
        self.reference_queries_skipped == 0
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
    load_report_with_project(workspace_root, mode, None, &[])
}

#[allow(dead_code)]
pub fn load_report_for_project(
    workspace_root: &Path,
    mode: AnalyzerMode,
    project: &Project,
) -> Result<AnalyzerReport, Box<dyn std::error::Error>> {
    load_report_with_project(workspace_root, mode, Some(project), &[])
}

pub fn load_report_for_project_and_roots(
    workspace_root: &Path,
    mode: AnalyzerMode,
    project: &Project,
    selected_roots: &[RootId],
) -> Result<AnalyzerReport, Box<dyn std::error::Error>> {
    load_report_with_project(workspace_root, mode, Some(project), selected_roots)
}

fn load_report_with_project(
    workspace_root: &Path,
    mode: AnalyzerMode,
    project: Option<&Project>,
    selected_roots: &[RootId],
) -> Result<AnalyzerReport, Box<dyn std::error::Error>> {
    match mode {
        AnalyzerMode::Syn => {
            let provider = SynSemanticProvider::new();
            Ok(provider.report().clone())
        }
        AnalyzerMode::RustAnalyzerHir => rust_analyzer::load_report(
            workspace_root,
            project,
            selected_roots,
            AnalyzerMode::RustAnalyzerHir,
            rust_analyzer::ProcMacroExpansionMode::Disabled,
            rust_analyzer::RaFeedbackMode::Disabled,
        ),
        AnalyzerMode::RustAnalyzerFeedback => rust_analyzer::load_report(
            workspace_root,
            project,
            selected_roots,
            AnalyzerMode::RustAnalyzerFeedback,
            rust_analyzer::ProcMacroExpansionMode::Disabled,
            rust_analyzer::RaFeedbackMode::Enabled,
        ),
        AnalyzerMode::RustAnalyzerHirProcMacros => rust_analyzer::load_report(
            workspace_root,
            project,
            selected_roots,
            AnalyzerMode::RustAnalyzerHirProcMacros,
            rust_analyzer::ProcMacroExpansionMode::Enabled,
            rust_analyzer::RaFeedbackMode::Enabled,
        ),
    }
}

#[cfg(feature = "ra-hir")]
mod rust_analyzer {
    use std::{
        collections::{BTreeSet, HashMap, VecDeque},
        ffi::OsStr,
        fs,
        panic::{self, AssertUnwindSafe},
        path::{Component, Path, PathBuf},
        sync::{
            atomic::{AtomicUsize, Ordering},
            Arc, Mutex,
        },
        time::Instant,
    };

    use ra_ap_hir::{Adt, HasSource, ModuleDef, PathResolution};
    use ra_ap_load_cargo::{load_workspace, LoadCargoConfig, ProcMacroServerChoice};
    use ra_ap_project_model::{CargoConfig, ProjectManifest, ProjectWorkspace};
    use ra_ap_syntax::{ast, AstNode, TextSize};
    use ra_ap_vfs::AbsPathBuf;

    use crate::{
        model::{
            CallableId, ItemId, ItemKind, Project, ReducedProject, RootId, SemanticOwnerId,
            SemanticReductionHints, SourceSpan,
        },
        reduce,
    };

    use super::{
        AnalyzerMode, AnalyzerReport, SemanticFileReport, SemanticProvider, SemanticReport,
        SemanticUnresolvedCategory, SemanticUnresolvedDiagnostic, SemanticUnresolvedKind,
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

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum ReferenceSearchMode {
        Complete,
        TopDownOnly,
    }

    #[derive(Debug, Default)]
    struct RaWorkspaceLoadTrace {
        requested_root: String,
        absolute_root: Option<String>,
        manifest_path: Option<String>,
        workspace_manifest_or_root: Option<String>,
        workspace_package_count: Option<usize>,
        proc_macro_mode: &'static str,
        manifest_discovery_ms: u64,
        project_workspace_load_ms: u64,
        build_scripts_ms: Option<u64>,
        crate_graph_vfs_load_ms: u64,
        build_script_error: Option<String>,
        progress_events: usize,
        progress_samples: Vec<String>,
    }

    impl RaWorkspaceLoadTrace {
        fn new(workspace_root: &Path, proc_macro_mode: ProcMacroExpansionMode) -> Self {
            Self {
                requested_root: workspace_root.display().to_string(),
                proc_macro_mode: match proc_macro_mode {
                    ProcMacroExpansionMode::Disabled => "disabled",
                    ProcMacroExpansionMode::Enabled => "enabled",
                },
                ..Self::default()
            }
        }

        fn notes(&self) -> Vec<String> {
            let mut notes = vec![
                format!(
                    "RA load scope: requested_root={}, absolute_root={}, manifest={}, workspace_manifest_or_root={}, loaded_packages={}, proc_macro_mode={}",
                    self.requested_root,
                    self.absolute_root.as_deref().unwrap_or("<unknown>"),
                    self.manifest_path.as_deref().unwrap_or("<unknown>"),
                    self.workspace_manifest_or_root.as_deref().unwrap_or("<unknown>"),
                    self.workspace_package_count
                        .map(|count| count.to_string())
                        .unwrap_or_else(|| "<unknown>".to_string()),
                    self.proc_macro_mode
                ),
                format!(
                    "RA load phase timings: manifest_discovery={}ms, project_workspace={}ms, build_scripts={}, crate_graph_vfs={}ms",
                    self.manifest_discovery_ms,
                    self.project_workspace_load_ms,
                    self.build_scripts_ms
                        .map(|ms| format!("{ms}ms"))
                        .unwrap_or_else(|| "skipped".to_string()),
                    self.crate_graph_vfs_load_ms
                ),
                format!("workspace load progress events: {}", self.progress_events),
            ];
            if !self.progress_samples.is_empty() {
                notes.push(format!(
                    "workspace load progress samples: {}",
                    self.progress_samples.join(" | ")
                ));
            }
            if let Some(error) = &self.build_script_error {
                notes.push(format!(
                    "RA build-script discovery reported errors: {error}"
                ));
            }
            notes
        }
    }

    fn elapsed_ms_since(started: Instant) -> u64 {
        started.elapsed().as_millis().try_into().unwrap_or(u64::MAX)
    }

    pub struct RustAnalyzerSemanticProvider {
        report: AnalyzerReport,
        _database: ra_ap_ide::RootDatabase,
    }

    impl RustAnalyzerSemanticProvider {
        fn load(
            workspace_root: &Path,
            project: Option<&Project>,
            selected_roots: &[RootId],
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
                    selected_roots,
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
                selected_roots,
                requested_mode,
                proc_macro_mode,
                feedback_mode,
            ) {
                Ok(provider) => Ok(provider),
                Err(error) if proc_macro_mode == ProcMacroExpansionMode::Enabled => {
                    let mut provider = Self::load_once(
                        workspace_root,
                        project,
                        selected_roots,
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
            selected_roots: &[RootId],
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
            let progress_samples = Mutex::new(Vec::new());
            let progress = |message: String| {
                let index = progress_events.fetch_add(1, Ordering::Relaxed);
                if index < 24 {
                    if let Ok(mut samples) = progress_samples.lock() {
                        samples.push(message);
                    }
                }
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
                let result: Result<_, Box<dyn std::error::Error>> = (|| {
                    let mut load_trace = RaWorkspaceLoadTrace::new(workspace_root, proc_macro_mode);

                    let started = Instant::now();
                    let absolute_root =
                        AbsPathBuf::assert_utf8(std::env::current_dir()?.join(workspace_root));
                    load_trace.absolute_root = Some(absolute_root.to_string());
                    let manifest = ProjectManifest::discover_single(&absolute_root)?;
                    load_trace.manifest_path = Some(manifest.manifest_path().to_string());
                    load_trace.manifest_discovery_ms = elapsed_ms_since(started);

                    let started = Instant::now();
                    let mut workspace = ProjectWorkspace::load(manifest, &cargo_config, &progress)?;
                    load_trace.workspace_manifest_or_root =
                        Some(workspace.manifest_or_root().to_string());
                    load_trace.workspace_package_count = Some(workspace.n_packages());
                    load_trace.project_workspace_load_ms = elapsed_ms_since(started);

                    if load_config.load_out_dirs_from_check {
                        let started = Instant::now();
                        let build_scripts =
                            workspace.run_build_scripts(&cargo_config, &progress)?;
                        if let Some(error) = build_scripts.error() {
                            load_trace.build_script_error = Some(error.to_string());
                        }
                        workspace.set_build_scripts(build_scripts);
                        load_trace.build_scripts_ms = Some(elapsed_ms_since(started));
                    }

                    let started = Instant::now();
                    let loaded = load_workspace(workspace, &cargo_config.extra_env, &load_config)?;
                    load_trace.crate_graph_vfs_load_ms = elapsed_ms_since(started);

                    Ok((loaded, load_trace))
                })();
                result
            }));
            let previous_hook = previous_hook
                .lock()
                .expect("RA workspace load panic hook should be restorable")
                .take()
                .expect("RA workspace load panic hook should be present");
            panic::set_hook(previous_hook);
            let loaded = loaded.map_err(|_| "rust-analyzer workspace load panicked")?;
            let ((database, vfs, proc_macro_client), mut load_trace) = loaded?;
            load_trace.progress_events = progress_events.load(Ordering::Relaxed);
            load_trace.progress_samples = progress_samples
                .lock()
                .map(|samples| samples.clone())
                .unwrap_or_default();

            let reference_mode = match requested_mode {
                AnalyzerMode::RustAnalyzerFeedback => ReferenceSearchMode::TopDownOnly,
                AnalyzerMode::Syn
                | AnalyzerMode::RustAnalyzerHir
                | AnalyzerMode::RustAnalyzerHirProcMacros => ReferenceSearchMode::Complete,
            };
            let semantic = collect_semantic_report(
                &database,
                &vfs,
                workspace_root,
                project,
                selected_roots,
                feedback_mode,
                reference_mode,
            );
            let mut notes = vec![
                "rust-analyzer RootDatabase loaded".to_string(),
                "HIR Semantics initialized".to_string(),
            ];
            notes.extend(load_trace.notes());
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
                "HIR semantic inventory: {}/{} files analyzed, {} skipped by budget, {} skipped by top-down scope, {}/{} queried method calls resolved to functions, {}/{} callable, {}/{} fallback, {} method calls unqueried, {}/{} queried paths resolved, {} paths unqueried",
                semantic.report.analyzed_files,
                semantic.report.source_files,
                semantic.report.skipped_files.saturating_sub(semantic.report.top_down_skipped_files),
                semantic.report.top_down_skipped_files,
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
                    "HIR usage mapping: {}/{} callable(s) and {}/{} item(s) mapped to rust-analyzer definitions; {} unmapped; {} reference query/queries, {} skipped, {} failure(s), {} promoted reference edge(s), {} referenced callable(s), {} referenced item(s)",
                    usage.mapped_callables,
                    usage.indexed_callables,
                    usage.mapped_items,
                    usage.indexed_items,
                    usage.unmapped_total(),
                    usage.reference_queries,
                    usage.reference_queries_skipped,
                    usage.reference_query_failures,
                    usage.callable_reference_edges + usage.item_reference_edges,
                    usage.referenced_callables,
                    usage.referenced_items
                ));
                if reference_mode == ReferenceSearchMode::TopDownOnly {
                    notes.push(
                        "HIR usage mapping scoped to top-down retained/root files; skipped whole-project reference proof is recorded as proof debt"
                            .to_string(),
                    );
                }
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
        selected_roots: &[RootId],
        requested_mode: AnalyzerMode,
        proc_macro_mode: ProcMacroExpansionMode,
        feedback_mode: RaFeedbackMode,
    ) -> Result<AnalyzerReport, Box<dyn std::error::Error>> {
        let provider = RustAnalyzerSemanticProvider::load(
            workspace_root,
            project,
            selected_roots,
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
        selected_roots: &[RootId],
        feedback_mode: RaFeedbackMode,
        reference_mode: ReferenceSearchMode,
    ) -> SemanticCollection {
        ra_ap_hir::attach_db(database, || {
            collect_semantic_report_attached(
                database,
                vfs,
                workspace_root,
                project,
                selected_roots,
                feedback_mode,
                reference_mode,
            )
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
        external_imported_symbols: BTreeSet<String>,
    }

    impl FileSemanticContext<'_> {
        fn should_collect_node(&self, offset: TextSize) -> bool {
            self.semantic_index
                .is_none_or(|index| index.should_collect_semantic_node(self.vfs_path, offset))
        }
    }

    fn collect_semantic_report_attached(
        database: &ra_ap_ide::RootDatabase,
        vfs: &ra_ap_vfs::Vfs,
        workspace_root: &Path,
        project: Option<&Project>,
        selected_roots: &[RootId],
        feedback_mode: RaFeedbackMode,
        reference_mode: ReferenceSearchMode,
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
        let mut semantic_index =
            project.map(|project| ProjectSemanticIndex::build(project, selected_roots));
        let mut hints = SemanticReductionHints::default();
        let ra_feedback = if feedback_mode == RaFeedbackMode::Enabled {
            project
                .zip(semantic_index.as_ref())
                .map(|(project, index)| {
                    collect_ra_feedback_edges(database, vfs, project, index, &mut hints)
                })
        } else {
            None
        };
        if let Some((project, index)) = project.zip(semantic_index.as_mut()) {
            index.refresh_retention(project, &hints);
        }

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
            if semantic_index
                .as_ref()
                .is_some_and(|index| !index.should_collect_semantic_file(vfs_path, reference_mode))
            {
                report.skipped_files += 1;
                report.top_down_skipped_files += 1;
                file_summary.skipped_by_top_down_scope = true;
                report.file_reports.push(file_summary);
                continue;
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
                    external_imported_symbols: BTreeSet::new(),
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
                    file_summary.unresolved_diagnostics = file_report.unresolved_diagnostics;
                    report
                        .unresolved_diagnostics
                        .extend(file_summary.unresolved_diagnostics.iter().cloned());
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
                collect_semantic_usage_report(
                    database,
                    &semantics,
                    vfs,
                    project,
                    index,
                    &mut hints,
                    reference_mode,
                )
            });
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
        reference_mode: ReferenceSearchMode,
    ) -> SemanticUsageReport {
        let mut report = SemanticUsageReport {
            indexed_callables: project.functions.len() + project.methods.len(),
            indexed_items: project.items.len(),
            ..SemanticUsageReport::default()
        };

        for (file_id, vfs_path) in vfs.iter() {
            if !index.should_map_usage_file(vfs_path, reference_mode) {
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
        match reference_mode {
            ReferenceSearchMode::Complete => {
                collect_semantic_reference_report(
                    database,
                    vfs,
                    project,
                    index,
                    &mut report,
                    hints,
                );
            }
            ReferenceSearchMode::TopDownOnly => {
                report.reference_queries_skipped = indexed_reference_query_candidates(project);
            }
        }
        report
    }

    fn indexed_reference_query_candidates(project: &Project) -> usize {
        project.functions.len()
            + project.methods.len()
            + project
                .items
                .keys()
                .filter(|item| item.kind != ItemKind::Mod)
                .count()
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
        let mut pending = VecDeque::from(index.feedback_owner_callables());
        let mut queried = BTreeSet::new();

        while let Some(callable) = pending.pop_front() {
            if !queried.insert(callable.clone()) {
                continue;
            }
            let candidates = index.callable_focus_offsets(&callable);
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
                    Some(dependency) if dependency != callable => {
                        report.edges += 1;
                        hints.add_callable_edge(
                            SemanticOwnerId::Callable(callable.clone()),
                            dependency.clone(),
                        );
                        if !queried.contains(&dependency)
                            && (project.functions.contains_key(&dependency)
                                || project.methods.contains_key(&dependency))
                        {
                            pending.push_back(dependency);
                        }
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
        let source_text = source.syntax().text().to_string();
        context.external_imported_symbols = external_imported_symbols(&source_text);
        let file_path = normalize_vfs_path(context.vfs_path).unwrap_or_default();

        for node in source.syntax().descendants() {
            if let Some(method_call) = ast::MethodCallExpr::cast(node.clone()) {
                if !context.should_collect_node(method_call.syntax().text_range().start()) {
                    continue;
                }
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
                } else {
                    report
                        .unresolved_diagnostics
                        .push(unresolved_method_diagnostic(
                            &file_path,
                            &source_text,
                            context,
                            &method_call,
                        ));
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
                if !context.should_collect_node(path.syntax().text_range().start()) {
                    continue;
                }
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
                } else {
                    report
                        .unresolved_diagnostics
                        .push(unresolved_path_diagnostic(
                            &file_path,
                            &source_text,
                            context,
                            &path,
                        ));
                }
            }
        }

        report
    }

    fn unresolved_method_diagnostic(
        file_path: &Path,
        source_text: &str,
        context: &FileSemanticContext<'_>,
        method_call: &ast::MethodCallExpr,
    ) -> SemanticUnresolvedDiagnostic {
        let syntax = method_call.syntax();
        let symbol = method_call
            .name_ref()
            .map(|name| name.syntax().text().to_string());
        let (category, reason) = classify_unresolved_method(
            context.semantic_index,
            &context.external_imported_symbols,
            &symbol,
            syntax,
        );
        unresolved_diagnostic(
            SemanticUnresolvedKind::MethodCall,
            category,
            reason,
            file_path,
            source_text,
            context,
            syntax,
            symbol,
        )
    }

    fn unresolved_path_diagnostic(
        file_path: &Path,
        source_text: &str,
        context: &FileSemanticContext<'_>,
        path: &ast::Path,
    ) -> SemanticUnresolvedDiagnostic {
        let syntax = path.syntax();
        let symbol = path
            .segment()
            .and_then(|segment| segment.name_ref())
            .map(|name| name.syntax().text().to_string());
        let segments = path_segments(path);
        let (category, reason) = classify_unresolved_path(
            context.semantic_index,
            &context.external_imported_symbols,
            syntax,
            &segments,
        );
        unresolved_diagnostic(
            SemanticUnresolvedKind::Path,
            category,
            reason,
            file_path,
            source_text,
            context,
            syntax,
            symbol,
        )
    }

    fn unresolved_diagnostic(
        kind: SemanticUnresolvedKind,
        category: SemanticUnresolvedCategory,
        reason: &'static str,
        file_path: &Path,
        source_text: &str,
        context: &FileSemanticContext<'_>,
        syntax: &ra_ap_syntax::SyntaxNode,
        symbol: Option<String>,
    ) -> SemanticUnresolvedDiagnostic {
        let range = syntax.text_range();
        let start = text_size_to_usize(range.start());
        let end = text_size_to_usize(range.end());
        let (start_line, start_column) = line_column_from_text(source_text, start);
        let (end_line, end_column) = line_column_from_text(source_text, end);
        let owner = context
            .semantic_index
            .and_then(|index| index.owner_at_vfs_offset(context.vfs_path, range.start()));
        SemanticUnresolvedDiagnostic {
            kind,
            category,
            reason: reason.to_string(),
            file: file_path.to_path_buf(),
            start_line,
            start_column,
            end_line,
            end_column,
            snippet: compact_node_snippet(syntax),
            ast_kind: format!("{:?}", syntax.kind()),
            symbol,
            owner,
        }
    }

    fn classify_unresolved_method(
        index: Option<&ProjectSemanticIndex>,
        external_imported_symbols: &BTreeSet<String>,
        symbol: &Option<String>,
        syntax: &ra_ap_syntax::SyntaxNode,
    ) -> (SemanticUnresolvedCategory, &'static str) {
        if syntax_has_macro_or_attr_ancestor(syntax) {
            return (
                SemanticUnresolvedCategory::MacroBlocked,
                "unresolved_method_inside_macro_or_attribute_context",
            );
        }
        let snippet = compact_node_snippet(syntax);
        if snippet_has_only_primitive_turbofish(&snippet) {
            return (
                SemanticUnresolvedCategory::Benign,
                "unresolved_method_has_primitive_turbofish",
            );
        }
        if unresolved_method_receiver_has_external_anchor(
            external_imported_symbols,
            symbol,
            &snippet,
        ) {
            return (
                SemanticUnresolvedCategory::Benign,
                "unresolved_method_receiver_has_external_anchor",
            );
        }
        if symbol.as_deref().is_some_and(|name| {
            unresolved_method_looks_like_common_external_receiver(name, &snippet)
        }) {
            return (
                SemanticUnresolvedCategory::Benign,
                "unresolved_method_looks_like_common_external_receiver",
            );
        }
        if symbol
            .as_deref()
            .is_some_and(|name| index.is_some_and(|index| index.has_project_method_name(name)))
        {
            return (
                SemanticUnresolvedCategory::DependencyRisk,
                "unresolved_method_name_matches_project_method",
            );
        }
        (
            SemanticUnresolvedCategory::Benign,
            "unresolved_method_has_no_project_local_method_match",
        )
    }

    fn classify_unresolved_path(
        index: Option<&ProjectSemanticIndex>,
        external_imported_symbols: &BTreeSet<String>,
        syntax: &ra_ap_syntax::SyntaxNode,
        segments: &[String],
    ) -> (SemanticUnresolvedCategory, &'static str) {
        if syntax_has_macro_or_attr_ancestor(syntax) {
            return (
                SemanticUnresolvedCategory::MacroBlocked,
                "unresolved_path_inside_macro_or_attribute_context",
            );
        }
        if path_segments_have_external_root(segments) {
            return (
                SemanticUnresolvedCategory::Benign,
                "unresolved_path_has_external_root",
            );
        }
        if index.is_some_and(|index| index.path_has_external_dependency_root(segments)) {
            return (
                SemanticUnresolvedCategory::Benign,
                "unresolved_path_has_external_dependency_root",
            );
        }
        if external_imported_symbols_have_path(external_imported_symbols, segments) {
            return (
                SemanticUnresolvedCategory::Benign,
                "unresolved_path_has_retained_external_import",
            );
        }
        if index.is_some_and(|index| index.path_is_derived_default_call(segments)) {
            return (
                SemanticUnresolvedCategory::Benign,
                "unresolved_path_is_derived_default_call",
            );
        }
        if index.is_some_and(|index| index.path_has_project_local_anchor(segments)) {
            return (
                SemanticUnresolvedCategory::DependencyRisk,
                "unresolved_path_anchor_matches_project_local_identifier",
            );
        }
        (
            SemanticUnresolvedCategory::Benign,
            "unresolved_path_has_no_project_local_identifier",
        )
    }

    fn snippet_has_only_primitive_turbofish(snippet: &str) -> bool {
        let Some(arguments) = turbofish_arguments(snippet) else {
            return false;
        };
        let idents = arguments
            .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
            .filter(|ident| !ident.is_empty())
            .collect::<Vec<_>>();
        !idents.is_empty() && idents.iter().all(|ident| is_primitive_type_ident(ident))
    }

    fn turbofish_arguments(snippet: &str) -> Option<&str> {
        let start = snippet.find("::<")? + 3;
        let rest = &snippet[start..];
        let end = rest.rfind('>')?;
        Some(&rest[..end])
    }

    fn is_primitive_type_ident(ident: &str) -> bool {
        matches!(
            ident,
            "bool"
                | "char"
                | "str"
                | "f32"
                | "f64"
                | "i8"
                | "i16"
                | "i32"
                | "i64"
                | "i128"
                | "isize"
                | "u8"
                | "u16"
                | "u32"
                | "u64"
                | "u128"
                | "usize"
        )
    }

    fn unresolved_method_receiver_has_external_anchor(
        external_imported_symbols: &BTreeSet<String>,
        symbol: &Option<String>,
        snippet: &str,
    ) -> bool {
        let Some(method_name) = symbol.as_deref() else {
            return false;
        };
        let Some(receiver) = method_receiver_snippet(snippet, method_name) else {
            return false;
        };
        let Some(anchor) = leading_receiver_ident(&receiver) else {
            return false;
        };
        matches!(anchor, "std" | "core" | "alloc")
            || external_imported_symbols_have_symbols(external_imported_symbols, &[anchor])
    }

    fn unresolved_method_looks_like_common_external_receiver(
        method_name: &str,
        snippet: &str,
    ) -> bool {
        if !is_common_external_method_name(method_name) {
            return false;
        }
        method_receiver_snippet(snippet, method_name)
            .is_some_and(|receiver| receiver_is_plain_value_or_field_chain(&receiver))
    }

    fn method_receiver_snippet(snippet: &str, method_name: &str) -> Option<String> {
        let normalized = snippet.replace(" .", ".").replace(". ", ".");
        let needle = format!(".{method_name}");
        let index = normalized.rfind(&needle)?;
        Some(normalized[..index].trim().to_string())
    }

    fn leading_receiver_ident(receiver: &str) -> Option<&str> {
        let receiver = receiver.trim_start_matches(['&', '*', '(', ' ']);
        let end = receiver
            .char_indices()
            .find_map(|(index, character)| {
                (!(character.is_ascii_alphanumeric() || character == '_')).then_some(index)
            })
            .unwrap_or(receiver.len());
        (end > 0).then_some(&receiver[..end])
    }

    fn receiver_is_plain_value_or_field_chain(receiver: &str) -> bool {
        !receiver.is_empty()
            && !receiver.contains("::")
            && receiver.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '_' | '.')
            })
    }

    fn is_common_external_method_name(name: &str) -> bool {
        matches!(
            name,
            "as_ref"
                | "as_mut"
                | "borrow"
                | "borrow_mut"
                | "clone"
                | "contains"
                | "get"
                | "get_mut"
                | "insert"
                | "is_empty"
                | "iter"
                | "iter_mut"
                | "join"
                | "len"
                | "parent"
                | "push"
                | "read"
                | "remove"
                | "to_path_buf"
                | "to_string"
                | "write"
        )
    }

    fn path_segments_have_external_root(segments: &[String]) -> bool {
        segments
            .first()
            .is_some_and(|segment| matches!(segment.as_str(), "std" | "core" | "alloc"))
    }

    fn rust_crate_root_ident(package_or_alias: &str) -> String {
        package_or_alias.replace('-', "_")
    }

    #[cfg(test)]
    fn source_has_external_imported_path(source_text: &str, segments: &[String]) -> bool {
        let mut symbols = Vec::new();
        if let Some(symbol) = segments.last() {
            symbols.push(symbol.as_str());
        }
        if segments.len() > 1 {
            if let Some(symbol) = segments.first() {
                symbols.push(symbol.as_str());
            }
        }
        source_has_external_imported_symbols(source_text, &symbols)
    }

    #[cfg(test)]
    fn source_has_external_imported_symbol(source_text: &str, symbol: &str) -> bool {
        source_has_external_imported_symbols(source_text, &[symbol])
    }

    #[cfg(test)]
    fn source_has_external_imported_symbols(source_text: &str, symbols: &[&str]) -> bool {
        if symbols.is_empty() {
            return false;
        }
        let imported_symbols = external_imported_symbols(source_text);
        external_imported_symbols_have_symbols(&imported_symbols, symbols)
    }

    fn external_imported_symbols(source_text: &str) -> BTreeSet<String> {
        let Ok(file) = syn::parse_file(source_text) else {
            return BTreeSet::new();
        };
        let mut symbols = BTreeSet::new();
        for item in &file.items {
            let syn::Item::Use(item_use) = item else {
                continue;
            };
            collect_external_imported_symbols(&item_use.tree, UseRootKind::Unknown, &mut symbols);
        }
        symbols
    }

    fn external_imported_symbols_have_path(
        imported_symbols: &BTreeSet<String>,
        segments: &[String],
    ) -> bool {
        let mut symbols = Vec::new();
        if let Some(symbol) = segments.last() {
            symbols.push(symbol.as_str());
        }
        if segments.len() > 1 {
            if let Some(symbol) = segments.first() {
                symbols.push(symbol.as_str());
            }
        }
        external_imported_symbols_have_symbols(imported_symbols, &symbols)
    }

    fn external_imported_symbols_have_symbols(
        imported_symbols: &BTreeSet<String>,
        symbols: &[&str],
    ) -> bool {
        !symbols.is_empty()
            && symbols
                .iter()
                .any(|symbol| imported_symbols.contains(*symbol))
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum UseRootKind {
        Unknown,
        External,
        Local,
    }

    fn collect_external_imported_symbols(
        tree: &syn::UseTree,
        root: UseRootKind,
        symbols: &mut BTreeSet<String>,
    ) {
        match tree {
            syn::UseTree::Path(path) => {
                let ident = path.ident.to_string();
                let root = match root {
                    UseRootKind::Unknown if is_relative_path_anchor(&ident) => UseRootKind::Local,
                    UseRootKind::Unknown => UseRootKind::External,
                    existing => existing,
                };
                collect_external_imported_symbols(&path.tree, root, symbols);
            }
            syn::UseTree::Name(name) => {
                if root == UseRootKind::External {
                    symbols.insert(name.ident.to_string());
                }
            }
            syn::UseTree::Rename(rename) => {
                if root == UseRootKind::External {
                    symbols.insert(rename.rename.to_string());
                }
            }
            syn::UseTree::Glob(_) => {}
            syn::UseTree::Group(group) => {
                for tree in &group.items {
                    collect_external_imported_symbols(tree, root, symbols);
                }
            }
        }
    }

    fn path_segments(path: &ast::Path) -> Vec<String> {
        let mut segments = Vec::new();
        collect_path_segments(path, &mut segments);
        segments
    }

    fn collect_path_segments(path: &ast::Path, segments: &mut Vec<String>) {
        if let Some(qualifier) = path.qualifier() {
            collect_path_segments(&qualifier, segments);
        }
        if let Some(segment) = path.segment() {
            if let Some(name) = segment.name_ref() {
                segments.push(name.syntax().text().to_string());
            }
        }
    }

    fn is_relative_path_anchor(segment: &str) -> bool {
        matches!(segment, "crate" | "self" | "super")
    }

    fn syntax_has_macro_or_attr_ancestor(syntax: &ra_ap_syntax::SyntaxNode) -> bool {
        syntax.ancestors().any(|ancestor| {
            let kind = format!("{:?}", ancestor.kind());
            kind.contains("MACRO") || kind.contains("ATTR")
        })
    }

    fn compact_node_snippet(syntax: &ra_ap_syntax::SyntaxNode) -> String {
        let mut snippet = syntax
            .text()
            .to_string()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        const MAX_SNIPPET_LEN: usize = 160;
        if snippet.len() > MAX_SNIPPET_LEN {
            snippet.truncate(MAX_SNIPPET_LEN);
        }
        snippet
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

    fn item_derives_default(item: &syn::Item) -> bool {
        item_attrs(item).iter().any(attribute_derives_default)
    }

    fn item_attrs(item: &syn::Item) -> &[syn::Attribute] {
        match item {
            syn::Item::Const(item) => &item.attrs,
            syn::Item::Enum(item) => &item.attrs,
            syn::Item::Mod(item) => &item.attrs,
            syn::Item::Static(item) => &item.attrs,
            syn::Item::Struct(item) => &item.attrs,
            syn::Item::Trait(item) => &item.attrs,
            syn::Item::Type(item) => &item.attrs,
            syn::Item::Union(item) => &item.attrs,
            _ => &[],
        }
    }

    fn attribute_derives_default(attribute: &syn::Attribute) -> bool {
        if !attribute.path().is_ident("derive") {
            return false;
        }
        let Ok(list) = attribute.meta.require_list() else {
            return false;
        };
        list.tokens
            .to_string()
            .split(|character: char| {
                !(character.is_ascii_alphanumeric() || matches!(character, '_' | ':'))
            })
            .any(|token| token.rsplit("::").next() == Some("Default"))
    }

    struct ProjectSemanticIndex {
        files: HashMap<PathBuf, IndexedSourceFile>,
        root_files: BTreeSet<PathBuf>,
        retained_files: BTreeSet<PathBuf>,
        retained_owners: BTreeSet<SemanticOwnerId>,
        selected_roots: Vec<RootId>,
        local_idents: BTreeSet<String>,
        external_crate_roots: BTreeSet<String>,
        method_names: BTreeSet<String>,
        derived_default_types: BTreeSet<String>,
    }

    impl ProjectSemanticIndex {
        fn build(project: &Project, selected_roots: &[RootId]) -> Self {
            let mut files = HashMap::new();
            let mut root_files = BTreeSet::new();
            let mut local_idents = BTreeSet::new();
            let mut external_crate_roots = BTreeSet::new();
            let mut method_names = BTreeSet::new();
            let mut derived_default_types = BTreeSet::new();
            let retention =
                retained_scope(project, &SemanticReductionHints::default(), selected_roots);
            for package in project.workspace.packages.values() {
                for dependency in &package.dependencies {
                    if !project.workspace.packages.contains_key(&dependency.package) {
                        external_crate_roots.insert(rust_crate_root_ident(&dependency.alias));
                    }
                }
            }
            for source in project.files.values() {
                let path = normalize_fs_path(&source.path);
                files.entry(path).or_insert_with(|| {
                    IndexedSourceFile::new(fs::read_to_string(&source.path).unwrap_or_default())
                });
            }

            for (id, record) in &project.functions {
                if has_opensourced_attr(&record.item.attrs)
                    || selected_roots.contains(&RootId::Callable(id.clone()))
                {
                    root_files.insert(normalize_fs_path(&record.span.file));
                }
                local_idents.insert(callable_name(id).to_string());
                if let Some(file) = files.get_mut(&normalize_fs_path(&record.span.file)) {
                    file.callables.push(IndexedCallable {
                        id: id.clone(),
                        span: record.span.clone(),
                    });
                }
            }
            for (id, record) in &project.methods {
                if has_opensourced_attr(&record.item.attrs)
                    || selected_roots.contains(&RootId::Callable(id.clone()))
                {
                    root_files.insert(normalize_fs_path(&record.span.file));
                }
                local_idents.insert(callable_name(id).to_string());
                method_names.insert(callable_name(id).to_string());
                if let CallableId::Method { type_path, .. } = id {
                    local_idents.extend(type_path.iter().cloned());
                }
                if let Some(file) = files.get_mut(&normalize_fs_path(&record.span.file)) {
                    file.callables.push(IndexedCallable {
                        id: id.clone(),
                        span: record.span.clone(),
                    });
                }
            }
            for (id, record) in &project.items {
                if item_has_opensourced_attr(&record.item)
                    || selected_roots.contains(&RootId::Item(id.clone()))
                {
                    root_files.insert(normalize_fs_path(&record.span.file));
                }
                local_idents.insert(id.name.clone());
                if item_derives_default(&record.item) {
                    derived_default_types.insert(id.name.clone());
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
                retained_files: retention.files,
                retained_owners: retention.owners,
                selected_roots: selected_roots.to_vec(),
                local_idents,
                external_crate_roots,
                method_names,
                derived_default_types,
            }
        }

        fn refresh_retention(
            &mut self,
            project: &Project,
            semantic_hints: &SemanticReductionHints,
        ) {
            let retention = retained_scope(project, semantic_hints, &self.selected_roots);
            self.retained_files = retention.files;
            self.retained_owners = retention.owners;
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

        fn should_collect_semantic_file(
            &self,
            vfs_path: &ra_ap_vfs::VfsPath,
            reference_mode: ReferenceSearchMode,
        ) -> bool {
            match reference_mode {
                ReferenceSearchMode::Complete => true,
                ReferenceSearchMode::TopDownOnly => {
                    (self.root_files.is_empty() && self.retained_files.is_empty())
                        || self.file_priority(vfs_path) <= 1
                }
            }
        }

        fn should_map_usage_file(
            &self,
            vfs_path: &ra_ap_vfs::VfsPath,
            reference_mode: ReferenceSearchMode,
        ) -> bool {
            if !self.contains_vfs_path(vfs_path) {
                return false;
            }
            match reference_mode {
                ReferenceSearchMode::Complete => true,
                ReferenceSearchMode::TopDownOnly => {
                    (self.root_files.is_empty() && self.retained_files.is_empty())
                        || self.file_priority(vfs_path) <= 1
                }
            }
        }

        fn should_collect_semantic_node(
            &self,
            vfs_path: &ra_ap_vfs::VfsPath,
            offset: TextSize,
        ) -> bool {
            if self.retained_owners.is_empty() {
                return true;
            }
            self.owner_at_vfs_offset(vfs_path, offset)
                .is_some_and(|owner| self.retained_owners.contains(&owner))
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

        fn feedback_owner_callables(&self) -> Vec<CallableId> {
            self.retained_owners
                .iter()
                .filter_map(|owner| match owner {
                    SemanticOwnerId::Callable(callable) => Some(callable.clone()),
                    SemanticOwnerId::Item(_) => None,
                })
                .collect()
        }

        fn has_project_method_name(&self, name: &str) -> bool {
            self.method_names.contains(name)
        }

        fn path_has_project_local_anchor(&self, segments: &[String]) -> bool {
            match segments {
                [single] => self.local_idents.contains(single),
                [first, rest @ ..] if is_relative_path_anchor(first) => rest
                    .iter()
                    .any(|segment| self.local_idents.contains(segment)),
                [first, rest @ ..] => {
                    self.local_idents.contains(first)
                        || rest
                            .iter()
                            .take(rest.len().saturating_sub(1))
                            .any(|segment| self.local_idents.contains(segment))
                }
                [] => false,
            }
        }

        fn path_has_external_dependency_root(&self, segments: &[String]) -> bool {
            matches!(
                segments,
                [first, ..] if !is_relative_path_anchor(first)
                    && self.external_crate_roots.contains(first)
            )
        }

        fn path_is_derived_default_call(&self, segments: &[String]) -> bool {
            matches!(
                segments,
                [type_name, method_name]
                    if method_name == "default" && self.derived_default_types.contains(type_name)
            )
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

    struct RetainedScope {
        files: BTreeSet<PathBuf>,
        owners: BTreeSet<SemanticOwnerId>,
    }

    fn retained_scope(
        project: &Project,
        semantic_hints: &SemanticReductionHints,
        selected_roots: &[RootId],
    ) -> RetainedScope {
        reduce::reduce_with_extra_roots_and_semantics(project, selected_roots, semantic_hints)
            .map(|reduced| RetainedScope {
                files: retained_file_paths(project, &reduced),
                owners: retained_owners(&reduced),
            })
            .unwrap_or_else(|_| RetainedScope {
                files: BTreeSet::new(),
                owners: BTreeSet::new(),
            })
    }

    fn retained_owners(reduced: &ReducedProject) -> BTreeSet<SemanticOwnerId> {
        let mut owners = BTreeSet::new();
        for root in &reduced.roots {
            match root {
                RootId::Callable(callable) => {
                    owners.insert(SemanticOwnerId::Callable(callable.clone()));
                }
                RootId::Item(item) => {
                    owners.insert(SemanticOwnerId::Item(item.clone()));
                }
            }
        }
        owners.extend(
            reduced
                .reachable
                .iter()
                .cloned()
                .map(SemanticOwnerId::Callable),
        );
        owners.extend(
            reduced
                .reachable_items
                .iter()
                .cloned()
                .map(SemanticOwnerId::Item),
        );
        owners
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
        line_starts: Vec<usize>,
        callables: Vec<IndexedCallable>,
        items: Vec<IndexedItem>,
    }

    impl IndexedSourceFile {
        fn new(text: String) -> Self {
            let mut line_starts = vec![0];
            for (index, byte) in text.bytes().enumerate() {
                if byte == b'\n' {
                    line_starts.push(index + 1);
                }
            }
            Self {
                text,
                line_starts,
                callables: Vec::new(),
                items: Vec::new(),
            }
        }

        fn line_column(&self, offset: TextSize) -> (usize, usize) {
            let offset = text_size_to_usize(offset).min(self.text.len());
            let line_index = match self.line_starts.binary_search(&offset) {
                Ok(index) => index,
                Err(0) => 0,
                Err(index) => index - 1,
            };
            (
                line_index + 1,
                offset.saturating_sub(self.line_starts[line_index]),
            )
        }

        fn byte_offset(&self, line: usize, column: usize) -> usize {
            let Some(line_start) = line
                .checked_sub(1)
                .and_then(|index| self.line_starts.get(index))
            else {
                return self.text.len();
            };
            (line_start + column).min(self.text.len())
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

    fn line_column_from_text(text: &str, offset: usize) -> (usize, usize) {
        let offset = offset.min(text.len());
        let mut line = 1;
        let mut line_start = 0;
        for (index, ch) in text.char_indices() {
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

    #[cfg(test)]
    mod tests {
        use super::{
            classify_unresolved_path, external_imported_symbols, item_derives_default,
            method_receiver_snippet, path_segments, receiver_is_plain_value_or_field_chain,
            rust_crate_root_ident, snippet_has_only_primitive_turbofish,
            source_has_external_imported_path, source_has_external_imported_symbol,
            unresolved_method_looks_like_common_external_receiver,
            unresolved_method_receiver_has_external_anchor, ProjectSemanticIndex,
        };
        use crate::analyzer::SemanticUnresolvedCategory;
        use ra_ap_syntax::AstNode;

        #[test]
        fn external_use_imported_symbols_are_benign_unresolved_candidates() {
            let source = r#"
pub use external_protocol::{TurnStartParams, UserInput};
pub use codex_protocol::openai_models::ReasoningEffort;
pub use external_protocol::Model as RenamedModel;
use crate::local::LocalType;
use self::local::SelfLocalType;
use super::parent::ParentLocalType;
"#;

            assert!(source_has_external_imported_symbol(
                source,
                "TurnStartParams"
            ));
            assert!(source_has_external_imported_symbol(
                source,
                "ReasoningEffort"
            ));
            assert!(source_has_external_imported_symbol(source, "RenamedModel"));
            assert!(!source_has_external_imported_symbol(source, "LocalType"));
            assert!(!source_has_external_imported_symbol(
                source,
                "SelfLocalType"
            ));
            assert!(!source_has_external_imported_symbol(
                source,
                "ParentLocalType"
            ));
            assert!(!source_has_external_imported_symbol(source, "Missing"));
        }

        #[test]
        fn external_module_import_qualifies_unresolved_child_paths_as_benign() {
            let source = r#"
use std::{fmt, io as std_io};
use crate::local::fmt as local_fmt;
"#;

            assert!(source_has_external_imported_path(
                source,
                &["fmt".to_string(), "Formatter".to_string()]
            ));
            assert!(source_has_external_imported_path(
                source,
                &["std_io".to_string(), "Error".to_string()]
            ));
            assert!(!source_has_external_imported_path(
                source,
                &["local_fmt".to_string(), "Formatter".to_string()]
            ));
        }

        #[test]
        fn external_dependency_roots_qualify_fully_qualified_paths_as_benign() {
            let source = "fn demo() { let _ = ratatui::widgets::BorderType::Rounded; }";
            let file =
                ra_ap_syntax::SourceFile::parse(source, ra_ap_syntax::Edition::Edition2021).tree();
            let path = file
                .syntax()
                .descendants()
                .filter_map(ra_ap_syntax::ast::Path::cast)
                .find(|path| {
                    path_segments(path)
                        == ["ratatui", "widgets", "BorderType", "Rounded"]
                            .map(str::to_string)
                            .to_vec()
                })
                .expect("fully qualified external path should parse");
            let mut index = empty_project_semantic_index();
            index.external_crate_roots.insert("ratatui".to_string());
            index.local_idents.insert("BorderType".to_string());
            let segments = path_segments(&path);

            let imported_symbols = external_imported_symbols(source);
            let (category, reason) =
                classify_unresolved_path(Some(&index), &imported_symbols, path.syntax(), &segments);

            assert_eq!(category, SemanticUnresolvedCategory::Benign);
            assert_eq!(reason, "unresolved_path_has_external_dependency_root");
        }

        #[test]
        fn dependency_aliases_use_rust_crate_root_spelling() {
            assert_eq!(
                rust_crate_root_ident("my-external-crate"),
                "my_external_crate"
            );
            assert_eq!(rust_crate_root_ident("renamed_crate"), "renamed_crate");
        }

        #[test]
        fn primitive_turbofish_methods_are_benign_unresolved_candidates() {
            assert!(snippet_has_only_primitive_turbofish("text.parse::<i64>()"));
            assert!(snippet_has_only_primitive_turbofish("text.parse::<f64>()"));
            assert!(!snippet_has_only_primitive_turbofish(
                "value.parse::<ProjectType>()"
            ));
            assert!(!snippet_has_only_primitive_turbofish("value.parse()"));
        }

        #[test]
        fn external_receiver_methods_are_benign_unresolved_candidates() {
            let source = "use std::{fs, path::PathBuf};";
            let imported_symbols = external_imported_symbols(source);
            assert!(unresolved_method_receiver_has_external_anchor(
                &imported_symbols,
                &Some("join".to_string()),
                "PathBuf::from(directory).join(PREFERENCES_FILE)"
            ));
            assert!(unresolved_method_receiver_has_external_anchor(
                &imported_symbols,
                &Some("write".to_string()),
                "fs::OpenOptions::new().write(true)"
            ));
            assert!(!unresolved_method_receiver_has_external_anchor(
                &imported_symbols,
                &Some("create".to_string()),
                "manager.create()"
            ));
        }

        #[test]
        fn common_external_receiver_methods_are_benign_unresolved_candidates() {
            assert_eq!(
                method_receiver_snippet("prefs.hidden_threads.insert(0, key)", "insert"),
                Some("prefs.hidden_threads".to_string())
            );
            assert!(receiver_is_plain_value_or_field_chain(
                "prefs.hidden_threads"
            ));
            assert!(unresolved_method_looks_like_common_external_receiver(
                "insert",
                "prefs.hidden_threads.insert(0, key)"
            ));
            assert!(unresolved_method_looks_like_common_external_receiver(
                "len",
                "prefs.pinned_threads.len()"
            ));
            assert!(unresolved_method_looks_like_common_external_receiver(
                "parent",
                "path.parent()"
            ));
            assert!(!unresolved_method_looks_like_common_external_receiver(
                "create",
                "manager.create()"
            ));
        }

        #[test]
        fn derived_default_items_qualify_default_calls_as_benign() {
            let item: syn::Item = syn::parse_quote! {
                #[derive(Debug, Clone, Default)]
                pub struct TranscriptBuffer {
                    pub text: String,
                }
            };
            let non_default: syn::Item = syn::parse_quote! {
                #[derive(Debug, Clone)]
                pub struct ManualBuffer {
                    pub text: String,
                }
            };

            assert!(item_derives_default(&item));
            assert!(!item_derives_default(&non_default));
        }

        fn empty_project_semantic_index() -> ProjectSemanticIndex {
            ProjectSemanticIndex {
                files: std::collections::HashMap::new(),
                root_files: std::collections::BTreeSet::new(),
                retained_files: std::collections::BTreeSet::new(),
                retained_owners: std::collections::BTreeSet::new(),
                selected_roots: Vec::new(),
                local_idents: std::collections::BTreeSet::new(),
                external_crate_roots: std::collections::BTreeSet::new(),
                method_names: std::collections::BTreeSet::new(),
                derived_default_types: std::collections::BTreeSet::new(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    #[cfg(feature = "ra-hir")]
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

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
        let workspace_root = ra_fixture_workspace("ra-hir-load");
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
    fn ra_feedback_mode_skips_global_reference_proof() {
        let workspace_root = ra_fixture_workspace("ra-feedback-reference-scope");
        let workspace = crate::manifest::load_workspace(&workspace_root).unwrap();
        let project = crate::parse::parse_workspace(workspace).unwrap();
        let report = load_report_for_project(
            &workspace_root,
            AnalyzerMode::RustAnalyzerFeedback,
            &project,
        )
        .unwrap();
        let usage = report
            .semantic_usage
            .as_ref()
            .expect("ra-feedback should still map semantic usage");

        assert_eq!(report.mode, AnalyzerMode::RustAnalyzerFeedback);
        assert_eq!(usage.reference_queries, 0);
        assert!(usage.reference_queries_skipped > 0);
        assert!(report
            .notes
            .iter()
            .any(|note| note.contains("RA feedback closure:")));
        assert!(report.notes.iter().any(|note| note.contains("skipped")));
    }

    #[test]
    #[cfg(feature = "ra-hir")]
    fn ra_feedback_usage_mapping_is_scoped_to_retained_files() {
        let workspace_root = ra_fixture_workspace_with_dead_module("ra-feedback-retained-scope");
        let workspace = crate::manifest::load_workspace(&workspace_root).unwrap();
        let project = crate::parse::parse_workspace(workspace).unwrap();
        let report = load_report_for_project(
            &workspace_root,
            AnalyzerMode::RustAnalyzerFeedback,
            &project,
        )
        .unwrap();
        let usage = report
            .semantic_usage
            .as_ref()
            .expect("ra-feedback should still map semantic usage");

        assert!(usage.mapped_callables < usage.indexed_callables);
        assert!(usage.unmapped_callables > 0);
        let semantic = report
            .semantic
            .as_ref()
            .expect("ra-feedback should collect semantic inventory");
        assert!(semantic.analyzed_files < semantic.source_files);
        assert!(semantic.top_down_skipped_files > 0);
        assert_eq!(semantic.selected_root_skipped_files, 0);
        assert_eq!(usage.reference_queries, 0);
        assert!(usage.reference_queries_skipped > usage.mapped_callables);
        assert!(
            usage
                .mapped_callable_ids
                .iter()
                .map(ToString::to_string)
                .all(|callable| !callable.contains("dead::unused")),
            "dead module callables should not be definition-mapped in top-down mode: {:?}",
            usage.mapped_callable_ids
        );
        assert!(report
            .notes
            .iter()
            .any(|note| note.contains("scoped to top-down retained/root files")));
    }

    #[test]
    #[cfg(feature = "ra-hir")]
    fn ra_hir_proc_macro_mode_skips_dependency_artifacts_by_default() {
        if proc_macro_dependency_loading_enabled_for_test() {
            return;
        }

        let workspace_root = ra_fixture_workspace("ra-hir-proc-macro-mode");
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

    #[cfg(feature = "ra-hir")]
    fn ra_fixture_workspace(label: &str) -> PathBuf {
        let root = temp_path(label);
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\n",
                opensourced_path
            ),
        );
        write(
            root.join("app/src/lib.rs"),
            r#"
use opensourced::opensourced;

pub struct Worker;

impl Worker {
    pub fn value(&self) -> u32 {
        7
    }
}

pub fn helper(worker: Worker) -> u32 {
    worker.value()
}

#[opensourced]
pub fn entry(worker: Worker) -> u32 {
    helper(worker)
}
"#,
        );
        root
    }

    #[cfg(feature = "ra-hir")]
    fn ra_fixture_workspace_with_dead_module(label: &str) -> PathBuf {
        let root = temp_path(label);
        let opensourced_path = workspace_root().join("crates/opensourced");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            root.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\n",
                opensourced_path
            ),
        );
        write(
            root.join("app/src/lib.rs"),
            r#"
pub mod dead;
pub mod live;
"#,
        );
        write(
            root.join("app/src/live.rs"),
            r#"
use opensourced::opensourced;

pub fn helper() -> u32 {
    7
}

#[opensourced]
pub fn entry() -> u32 {
    helper()
}
"#,
        );
        write(
            root.join("app/src/dead.rs"),
            r#"
pub fn unused() -> u32 {
    11
}

pub struct Dead;

impl Dead {
    pub fn unused_method(&self) -> u32 {
        unused()
    }
}
"#,
        );
        root
    }

    #[cfg(feature = "ra-hir")]
    fn temp_path(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("opensource-core-{label}-{nanos}"));
        if path.exists() {
            fs::remove_dir_all(&path).expect("old temp path should be removable");
        }
        path
    }

    #[cfg(feature = "ra-hir")]
    fn write(path: PathBuf, contents: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("fixture parent should be creatable");
        }
        fs::write(path, contents).expect("fixture file should be writable");
    }
}

#[cfg(not(feature = "ra-hir"))]
mod rust_analyzer {
    use std::path::Path;

    use crate::model::{Project, RootId};

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
        _selected_roots: &[RootId],
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
