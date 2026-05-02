use std::{
    collections::{BTreeSet, HashMap},
    fmt,
    path::PathBuf,
};

use syn::{File, ImplItem, ImplItemFn, Item, ItemFn};

use crate::manifest::Workspace;

pub struct Project {
    pub workspace: Workspace,
    pub files: HashMap<PathBuf, SourceFile>,
    pub functions: HashMap<CallableId, FunctionRecord>,
    pub methods: HashMap<CallableId, MethodRecord>,
    pub items: HashMap<ItemId, ItemRecord>,
    pub module_aliases: HashMap<(String, Vec<String>), HashMap<String, Vec<String>>>,
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
    pub item: ItemFn,
    pub aliases: HashMap<String, Vec<String>>,
}

#[derive(Clone)]
pub struct MethodRecord {
    pub module_path: Vec<String>,
    pub item: ImplItemFn,
    pub impl_items: Vec<ImplItem>,
    pub trait_input_type_paths: Vec<Vec<String>>,
    pub aliases: HashMap<String, Vec<String>>,
}

#[derive(Clone)]
pub struct ItemRecord {
    pub package: String,
    pub module_path: Vec<String>,
    pub item: Item,
    pub aliases: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct ReducedProject {
    pub root: RootId,
    pub packages: BTreeSet<String>,
    pub reachable: BTreeSet<CallableId>,
    pub reachable_items: BTreeSet<ItemId>,
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
    Const,
    Static,
    Macro,
    Module,
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

impl fmt::Display for RootId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Callable(callable) => write!(formatter, "{callable}"),
            Self::Item(item) => write!(formatter, "{item}"),
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

fn write_segments(formatter: &mut fmt::Formatter<'_>, segments: &[String]) -> fmt::Result {
    for (index, segment) in segments.iter().enumerate() {
        if index > 0 {
            write!(formatter, "::")?;
        }
        write!(formatter, "{segment}")?;
    }
    Ok(())
}
