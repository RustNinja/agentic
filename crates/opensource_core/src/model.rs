use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fmt,
    path::PathBuf,
};

use serde::{Deserialize, Serialize};
use syn::{File, ImplItem, ImplItemFn, Item, ItemFn};

use crate::manifest::Workspace;

pub struct Project {
    pub workspace: Workspace,
    pub files: HashMap<PathBuf, SourceFile>,
    pub functions: HashMap<CallableId, FunctionRecord>,
    pub methods: HashMap<CallableId, MethodRecord>,
    pub items: HashMap<ItemId, ItemRecord>,
    pub module_aliases: HashMap<(String, Vec<String>), HashMap<String, Vec<String>>>,
    pub source_files_by_module: HashMap<(String, Vec<String>), PathBuf>,
    pub methods_by_receiver: HashMap<(String, Vec<String>, String), Vec<CallableId>>,
}

pub struct SourceFile {
    pub package: String,
    pub module_path: Vec<String>,
    pub path: PathBuf,
    pub syntax: File,
}

#[derive(Clone)]
pub struct FunctionRecord {
    pub id: CallableId,
    pub package: String,
    pub module_path: Vec<String>,
    pub span: SourceSpan,
    pub item: ItemFn,
    pub aliases: HashMap<String, Vec<String>>,
}

#[derive(Clone)]
pub struct MethodRecord {
    pub module_path: Vec<String>,
    pub span: SourceSpan,
    pub item: ImplItemFn,
    pub impl_generics: syn::Generics,
    pub impl_items: Vec<ImplItem>,
    pub trait_input_type_paths: Vec<Vec<String>>,
    pub aliases: HashMap<String, Vec<String>>,
}

#[derive(Clone)]
pub struct ItemRecord {
    pub package: String,
    pub module_path: Vec<String>,
    pub span: SourceSpan,
    pub item: Item,
    pub aliases: HashMap<String, Vec<String>>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SourceSpan {
    pub file: PathBuf,
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

#[derive(Debug, Clone)]
pub struct ReducedProject {
    pub root: RootId,
    pub roots: Vec<RootId>,
    pub packages: BTreeSet<String>,
    pub reachable: BTreeSet<CallableId>,
    pub reachable_items: BTreeSet<ItemId>,
    pub evidence: ReductionEvidence,
}

#[derive(Clone, Debug, Default)]
pub struct SemanticReductionHints {
    pub callable_edges: BTreeMap<CallableId, SemanticDependencies>,
    pub item_edges: BTreeMap<ItemId, SemanticDependencies>,
    pub unresolved_queries: usize,
    pub unqueried_queries: usize,
    pub unmapped_targets: usize,
}

impl SemanticReductionHints {
    pub fn is_empty(&self) -> bool {
        self.callable_edges.is_empty() && self.item_edges.is_empty()
    }

    pub fn total_edges(&self) -> usize {
        self.callable_edges
            .values()
            .map(SemanticDependencies::len)
            .sum::<usize>()
            + self
                .item_edges
                .values()
                .map(SemanticDependencies::len)
                .sum::<usize>()
    }

    pub fn add_callable_edge(&mut self, owner: SemanticOwnerId, dependency: CallableId) {
        match owner {
            SemanticOwnerId::Callable(callable) => {
                self.callable_edges
                    .entry(callable)
                    .or_default()
                    .callables
                    .insert(dependency);
            }
            SemanticOwnerId::Item(item) => {
                self.item_edges
                    .entry(item)
                    .or_default()
                    .callables
                    .insert(dependency);
            }
        }
    }

    pub fn add_item_edge(&mut self, owner: SemanticOwnerId, dependency: ItemId) {
        match owner {
            SemanticOwnerId::Callable(callable) => {
                self.callable_edges
                    .entry(callable)
                    .or_default()
                    .items
                    .insert(dependency);
            }
            SemanticOwnerId::Item(item) => {
                self.item_edges
                    .entry(item)
                    .or_default()
                    .items
                    .insert(dependency);
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct SemanticDependencies {
    pub callables: BTreeSet<CallableId>,
    pub items: BTreeSet<ItemId>,
}

impl SemanticDependencies {
    pub fn is_empty(&self) -> bool {
        self.callables.is_empty() && self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.callables.len() + self.items.len()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum SemanticOwnerId {
    Callable(CallableId),
    Item(ItemId),
}

#[derive(Clone, Debug, Default)]
pub struct ReductionEvidence {
    pub unresolved_method_fallbacks: usize,
    pub unresolved_method_candidate_matches: usize,
    pub capped_unresolved_method_fallbacks: usize,
    pub capped_unresolved_method_details: Vec<CappedMethodFallbackEvidence>,
    pub semantic_edges_applied: usize,
}

impl ReductionEvidence {
    pub fn add(&mut self, other: &Self) {
        self.unresolved_method_fallbacks += other.unresolved_method_fallbacks;
        self.unresolved_method_candidate_matches += other.unresolved_method_candidate_matches;
        self.capped_unresolved_method_fallbacks += other.capped_unresolved_method_fallbacks;
        self.capped_unresolved_method_details
            .extend(other.capped_unresolved_method_details.iter().cloned());
        self.semantic_edges_applied += other.semantic_edges_applied;
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CappedMethodFallbackEvidence {
    pub method_name: String,
    pub candidate_count: usize,
    pub receiver_candidate_count: usize,
    pub package: Option<String>,
    pub module_path: Option<Vec<String>>,
    pub owner: Option<String>,
    pub file: Option<PathBuf>,
    pub start_line: Option<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum RootId {
    Callable(CallableId),
    Item(ItemId),
}

impl RootId {
    pub fn package(&self) -> &str {
        match self {
            Self::Callable(callable) => callable.package(),
            Self::Item(item) => item.package(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum CallableId {
    Free {
        package: String,
        module_path: Vec<String>,
        name: String,
    },
    Method {
        package: String,
        type_path: Vec<String>,
        trait_path: Option<Vec<String>>,
        trait_input_type_paths: Vec<Vec<String>>,
        method: String,
    },
}

impl CallableId {
    pub fn package(&self) -> &str {
        match self {
            Self::Free { package, .. } | Self::Method { package, .. } => package,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ItemId {
    pub package: String,
    pub module_path: Vec<String>,
    pub name: String,
    pub kind: ItemKind,
}

impl ItemId {
    pub fn package(&self) -> &str {
        &self.package
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ItemKind {
    Struct,
    Enum,
    Union,
    Type,
    Trait,
    Mod,
    Const,
    Static,
    Macro,
}

impl fmt::Display for CallableId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Free {
                package,
                module_path,
                name,
            } => {
                write!(formatter, "{package}")?;
                for segment in module_path {
                    write!(formatter, "::{segment}")?;
                }
                write!(formatter, "::{name}")
            }
            Self::Method {
                package,
                type_path,
                trait_path,
                trait_input_type_paths,
                method,
            } => {
                write!(formatter, "{package}::")?;
                if let Some(trait_path) = trait_path {
                    write!(formatter, "<")?;
                    write_segments(formatter, type_path)?;
                    write!(formatter, " as ")?;
                    write_segments(formatter, trait_path)?;
                    if !trait_input_type_paths.is_empty() {
                        write!(formatter, "<")?;
                        for (index, input_path) in trait_input_type_paths.iter().enumerate() {
                            if index > 0 {
                                write!(formatter, ", ")?;
                            }
                            write_segments(formatter, input_path)?;
                        }
                        write!(formatter, ">")?;
                    }
                    write!(formatter, ">::{method}")
                } else {
                    write_segments(formatter, type_path)?;
                    write!(formatter, "::{method}")
                }
            }
        }
    }
}

impl fmt::Display for ItemId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}::", self.package)?;
        for segment in &self.module_path {
            write!(formatter, "{segment}::")?;
        }
        write!(formatter, "{}({:?})", self.name, self.kind)
    }
}

impl fmt::Display for RootId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Callable(callable) => write!(formatter, "{callable}"),
            Self::Item(item) => write!(formatter, "{item}"),
        }
    }
}

fn write_segments(formatter: &mut fmt::Formatter<'_>, segments: &[String]) -> fmt::Result {
    for (index, segment) in segments.iter().enumerate() {
        if index > 0 {
            write!(formatter, "::")?;
        }
        write!(formatter, "{segment}")?;
    }
    Ok(())
}
