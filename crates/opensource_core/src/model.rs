use std::{
    collections::{BTreeSet, HashMap},
    fmt,
    path::PathBuf,
};

use syn::{File, ImplItemFn, ItemFn};

use crate::manifest::Workspace;

pub struct Project {
    pub workspace: Workspace,
    pub files: HashMap<PathBuf, SourceFile>,
    pub functions: HashMap<CallableId, FunctionRecord>,
    pub methods: HashMap<CallableId, MethodRecord>,
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
    pub aliases: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct ReducedProject {
    pub root: CallableId,
    pub packages: BTreeSet<String>,
    pub reachable: BTreeSet<CallableId>,
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
                method,
            } => {
                write!(formatter, "{package}")?;
                for segment in type_path {
                    write!(formatter, "::{segment}")?;
                }
                write!(formatter, "::{method}")
            }
        }
    }
}
