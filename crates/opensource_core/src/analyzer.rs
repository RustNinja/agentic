use std::path::Path;

use crate::model::{Project, SemanticReductionHints};

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
    pub semantic_hints: SemanticReductionHints,
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
            semantic_hints: SemanticReductionHints::default(),
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
        AnalyzerMode::RustAnalyzerHir => rust_analyzer::load_report(workspace_root, project),
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
        sync::atomic::{AtomicUsize, Ordering},
    };

    use ra_ap_hir::{Adt, HasSource, ModuleDef, PathResolution};
    use ra_ap_load_cargo::{load_workspace_at, LoadCargoConfig, ProcMacroServerChoice};
    use ra_ap_project_model::CargoConfig;
    use ra_ap_syntax::{ast, AstNode, TextSize};

    use crate::model::{
        CallableId, ItemId, ItemKind, Project, SemanticOwnerId, SemanticReductionHints, SourceSpan,
    };

    use super::{AnalyzerMode, AnalyzerReport, SemanticProvider, SemanticReport};

    const DEFAULT_SEMANTIC_FILE_BUDGET: usize = 48;
    const DEFAULT_METHOD_CALL_BUDGET: usize = 1_000;
    const DEFAULT_PATH_BUDGET: usize = 2_000;

    pub struct RustAnalyzerSemanticProvider {
        report: AnalyzerReport,
        _database: ra_ap_ide::RootDatabase,
    }

    impl RustAnalyzerSemanticProvider {
        fn load(
            workspace_root: &Path,
            project: Option<&Project>,
        ) -> Result<Self, Box<dyn std::error::Error>> {
            let cargo_config = CargoConfig {
                set_test: true,
                no_deps: true,
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

            let semantic = collect_semantic_report(&database, &vfs, workspace_root, project);
            let mut notes = vec![
                "rust-analyzer RootDatabase loaded".to_string(),
                "HIR Semantics initialized".to_string(),
                "dependency crates excluded from HIR load for bounded slicer analysis".to_string(),
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
            notes.push(format!(
                "HIR reduction hints: {} project-local semantic edge(s), {} unresolved query/queries, {} unqueried query/queries, {} unmapped target(s)",
                semantic.hints.total_edges(),
                semantic.hints.unresolved_queries,
                semantic.hints.unqueried_queries,
                semantic.hints.unmapped_targets
            ));
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
                    mode: AnalyzerMode::RustAnalyzerHir,
                    loaded: true,
                    engine: "rust-analyzer HIR".to_string(),
                    notes,
                    semantic: Some(semantic.report),
                    semantic_hints: semantic.hints,
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
    ) -> Result<AnalyzerReport, Box<dyn std::error::Error>> {
        let provider = RustAnalyzerSemanticProvider::load(workspace_root, project)?;
        Ok(provider.report().clone())
    }

    fn collect_semantic_report(
        database: &ra_ap_ide::RootDatabase,
        vfs: &ra_ap_vfs::Vfs,
        workspace_root: &Path,
        project: Option<&Project>,
    ) -> SemanticCollection {
        ra_ap_hir::attach_db(database, || {
            collect_semantic_report_attached(database, vfs, workspace_root, project)
        })
    }

    struct SemanticCollection {
        report: SemanticReport,
        hints: SemanticReductionHints,
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

            report.source_files += 1;
            if report.analyzed_files + report.failed_files >= report.file_budget {
                report.skipped_files += 1;
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
        hints.unresolved_queries = report.unresolved_method_calls + report.unresolved_paths;
        hints.unqueried_queries = report.unqueried_method_calls + report.unqueried_paths;
        SemanticCollection { report, hints }
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
    }

    impl ProjectSemanticIndex {
        fn build(project: &Project) -> Self {
            let mut files = HashMap::new();
            let mut root_files = BTreeSet::new();
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

            Self { files, root_files }
        }

        fn contains_vfs_path(&self, vfs_path: &ra_ap_vfs::VfsPath) -> bool {
            self.indexed_file(vfs_path).is_some()
        }

        fn file_priority(&self, vfs_path: &ra_ap_vfs::VfsPath) -> usize {
            let Some(path) = normalize_vfs_path(vfs_path) else {
                return 1;
            };
            usize::from(!self.root_files.contains(&path))
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

        fn indexed_file(&self, vfs_path: &ra_ap_vfs::VfsPath) -> Option<&IndexedSourceFile> {
            let path = normalize_vfs_path(vfs_path)?;
            self.files.get(&path)
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
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{load_report, load_report_for_project, AnalyzerMode};

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

    use super::AnalyzerReport;

    pub fn load_report(
        _workspace_root: &Path,
        _project: Option<&Project>,
    ) -> Result<AnalyzerReport, Box<dyn std::error::Error>> {
        Err(
            "ra-hir analyzer requested, but opensource_core was built without the ra-hir feature"
                .into(),
        )
    }
}
