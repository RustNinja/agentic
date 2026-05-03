use std::{
    collections::{BTreeSet, HashMap},
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use serde::Deserialize;
use syn::{Expr, Item, Lit, Meta};
use toml::Value;

#[derive(Debug)]
pub struct Workspace {
    pub root: PathBuf,
    pub packages: HashMap<String, Package>,
    pub manifest: Value,
}

#[derive(Debug, Clone)]
pub struct Package {
    pub name: String,
    pub root: PathBuf,
    pub lib_path: PathBuf,
    pub entry_target: PackageTarget,
    pub dependencies: Vec<Dependency>,
    pub manifest: Value,
}

#[derive(Debug, Clone)]
pub struct PackageTarget {
    pub name: String,
    pub kind: Vec<String>,
    pub src_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Dependency {
    pub alias: String,
    pub package: String,
}

pub fn load_workspace(root: &Path) -> Result<Workspace, Box<dyn std::error::Error>> {
    let root = root.canonicalize()?;
    let root_manifest = root.join("Cargo.toml");
    let root_value = read_manifest(&root_manifest)?;
    let metadata = load_cargo_metadata(&root_manifest)?;
    let workspace_members = metadata
        .workspace_members
        .into_iter()
        .collect::<BTreeSet<_>>();

    let mut packages = HashMap::new();
    for metadata_package in metadata
        .packages
        .into_iter()
        .filter(|package| workspace_members.contains(&package.id))
    {
        let manifest_path = metadata_package.manifest_path;
        let package_root = manifest_path
            .parent()
            .ok_or_else(|| {
                format!(
                    "package {} manifest path has no parent: {}",
                    metadata_package.name,
                    manifest_path.display()
                )
            })?
            .canonicalize()?;
        let manifest = read_manifest(&manifest_path)?;

        let entry_target = entry_target(&package_root, &manifest, &metadata_package.targets)?;
        let lib_path = entry_target.src_path.clone();

        let dependencies = metadata_dependencies(&metadata_package.dependencies, &entry_target);

        packages.insert(
            metadata_package.name.clone(),
            Package {
                name: metadata_package.name,
                root: package_root,
                lib_path,
                entry_target,
                dependencies,
                manifest,
            },
        );
    }

    Ok(Workspace {
        root,
        packages,
        manifest: root_value,
    })
}

fn read_manifest(path: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let text = fs::read_to_string(path)?;
    Ok(text.parse::<Value>()?)
}

fn load_cargo_metadata(manifest_path: &Path) -> Result<CargoMetadata, Box<dyn std::error::Error>> {
    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--format-version=1")
        .arg("--no-deps")
        .arg("--manifest-path")
        .arg(manifest_path)
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "cargo metadata failed for {}\n{}",
            manifest_path.display(),
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    Ok(serde_json::from_slice(&output.stdout)?)
}

fn entry_target(
    package_root: &Path,
    manifest: &Value,
    targets: &[MetadataTarget],
) -> Result<PackageTarget, Box<dyn std::error::Error>> {
    if let Some(target) = metadata_marker_target(targets)? {
        return Ok(target.into());
    }

    if let Some(target) = metadata_entry_target(package_root, targets) {
        return Ok(target.into());
    }

    if let Some(path) = manifest
        .get("lib")
        .and_then(|lib| lib.get("path"))
        .and_then(Value::as_str)
    {
        return Ok(PackageTarget {
            name: package_name_from_manifest(manifest),
            kind: vec!["lib".to_string()],
            src_path: package_root.join(path),
        });
    }

    let default_lib = package_root.join("src/lib.rs");
    if default_lib.exists() {
        return Ok(PackageTarget {
            name: package_name_from_manifest(manifest),
            kind: vec!["lib".to_string()],
            src_path: default_lib,
        });
    }

    Ok(PackageTarget {
        name: package_name_from_manifest(manifest),
        kind: vec!["bin".to_string()],
        src_path: package_root.join("src/main.rs"),
    })
}

fn package_name_from_manifest(manifest: &Value) -> String {
    manifest
        .get("package")
        .and_then(|package| package.get("name"))
        .and_then(Value::as_str)
        .unwrap_or("package")
        .to_string()
}

fn metadata_marker_target(
    targets: &[MetadataTarget],
) -> Result<Option<MetadataTarget>, Box<dyn std::error::Error>> {
    let marked_targets = targets
        .iter()
        .filter(|target| target_is_parse_candidate(target))
        .filter(|target| source_contains_opensourced_marker(&target.src_path))
        .cloned()
        .collect::<Vec<_>>();
    let mut marked_targets = if marked_targets
        .iter()
        .any(|target| !metadata_target_uses_dev_dependencies(target))
    {
        marked_targets
            .into_iter()
            .filter(|target| !metadata_target_uses_dev_dependencies(target))
            .collect()
    } else {
        marked_targets
    };
    marked_targets.sort_by(|left, right| {
        left.src_path
            .cmp(&right.src_path)
            .then_with(|| left.name.cmp(&right.name))
            .then_with(|| left.kind.cmp(&right.kind))
    });
    marked_targets.dedup_by(|left, right| left.src_path == right.src_path);
    if marked_targets.len() > 1 {
        let paths = marked_targets
            .iter()
            .map(|target| target.src_path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!(
            "multiple package targets contain #[opensourced] markers; slice one target at a time: {paths}"
        )
        .into());
    }
    Ok(marked_targets.into_iter().next())
}

fn target_is_parse_candidate(target: &MetadataTarget) -> bool {
    target.kind.iter().any(|kind| {
        matches!(
            kind.as_str(),
            "lib" | "proc-macro" | "bin" | "example" | "test" | "bench"
        )
    })
}

fn source_contains_opensourced_marker(path: &Path) -> bool {
    let Some(module_dir) = entry_module_dir(path) else {
        return false;
    };
    let mut visited = BTreeSet::new();
    source_tree_contains_opensourced_marker(path, &module_dir, &mut visited)
}

fn source_tree_contains_opensourced_marker(
    path: &Path,
    module_dir: &Path,
    visited: &mut BTreeSet<PathBuf>,
) -> bool {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    if !visited.insert(canonical) {
        return false;
    }

    let Ok(text) = fs::read_to_string(path) else {
        return false;
    };
    if text.contains("#[opensourced")
        || text.contains("#[ opensourced")
        || text.contains("opensourced::opensourced")
    {
        return true;
    }

    let Ok(syntax) = syn::parse_file(&text) else {
        return false;
    };
    syntax.items.iter().any(|item| {
        let Item::Mod(item_mod) = item else {
            return false;
        };
        if item_mod.content.is_some() {
            return false;
        }
        let name = item_mod.ident.to_string();
        let Some((child_path, child_module_dir)) =
            resolve_external_module(module_dir, &name, path_attr(item_mod))
        else {
            return false;
        };
        source_tree_contains_opensourced_marker(&child_path, &child_module_dir, visited)
    })
}

fn entry_module_dir(entry_path: &Path) -> Option<PathBuf> {
    entry_path.parent().map(Path::to_path_buf)
}

fn path_attr(item_mod: &syn::ItemMod) -> Option<PathBuf> {
    item_mod.attrs.iter().find_map(|attribute| {
        if !attribute.path().is_ident("path") {
            return None;
        }
        let Meta::NameValue(name_value) = &attribute.meta else {
            return None;
        };
        let Expr::Lit(expr_lit) = &name_value.value else {
            return None;
        };
        let Lit::Str(lit) = &expr_lit.lit else {
            return None;
        };
        Some(PathBuf::from(lit.value()))
    })
}

fn resolve_external_module(
    module_dir: &Path,
    name: &str,
    path_attr: Option<PathBuf>,
) -> Option<(PathBuf, PathBuf)> {
    if let Some(path_attr) = path_attr {
        let path = if path_attr.is_absolute() {
            path_attr
        } else {
            module_dir.join(path_attr)
        };
        let next_dir = path.parent().unwrap_or(module_dir).to_path_buf();
        return path.exists().then_some((path, next_dir));
    }

    let source_name = module_source_name(name);
    let file_module = module_dir.join(format!("{source_name}.rs"));
    if file_module.exists() {
        return Some((file_module, module_dir.join(source_name)));
    }
    let mod_module = module_dir.join(source_name).join("mod.rs");
    mod_module
        .exists()
        .then_some((mod_module, module_dir.join(source_name)))
}

fn module_source_name(name: &str) -> &str {
    name.strip_prefix("r#").unwrap_or(name)
}

fn metadata_entry_target(
    package_root: &Path,
    targets: &[MetadataTarget],
) -> Option<MetadataTarget> {
    for preferred_kind in ["lib", "proc-macro"] {
        if let Some(target) = targets
            .iter()
            .find(|target| target.kind.iter().any(|kind| kind == preferred_kind))
        {
            return Some(target.clone());
        }
    }
    let default_bin = package_root.join("src/main.rs");
    if let Some(target) = targets.iter().find(|target| {
        target.kind.iter().any(|kind| kind == "bin") && target.src_path == default_bin
    }) {
        return Some(target.clone());
    }
    if let Some(target) = targets
        .iter()
        .find(|target| target.kind.iter().any(|kind| kind == "bin"))
    {
        return Some(target.clone());
    }
    targets
        .iter()
        .find(|target| {
            target
                .kind
                .iter()
                .all(|kind| !matches!(kind.as_str(), "test" | "bench" | "example"))
        })
        .cloned()
}

fn metadata_dependencies(
    metadata_dependencies: &[MetadataDependency],
    entry_target: &PackageTarget,
) -> Vec<Dependency> {
    let include_dev_dependencies = target_uses_dev_dependencies(entry_target);
    let mut dependencies = metadata_dependencies
        .iter()
        .filter(|dependency| dependency.kind.as_deref() != Some("dev") || include_dev_dependencies)
        .map(|dependency| Dependency {
            alias: dependency
                .rename
                .clone()
                .unwrap_or_else(|| dependency.name.clone()),
            package: dependency.name.clone(),
        })
        .collect::<Vec<_>>();
    dependencies.sort_by(|left, right| {
        left.alias
            .cmp(&right.alias)
            .then_with(|| left.package.cmp(&right.package))
    });
    dependencies.dedup_by(|left, right| left.alias == right.alias && left.package == right.package);

    dependencies
}

fn target_uses_dev_dependencies(target: &PackageTarget) -> bool {
    target
        .kind
        .iter()
        .any(|kind| matches!(kind.as_str(), "example" | "test" | "bench"))
}

fn metadata_target_uses_dev_dependencies(target: &MetadataTarget) -> bool {
    target
        .kind
        .iter()
        .any(|kind| matches!(kind.as_str(), "example" | "test" | "bench"))
}

#[derive(Debug, Deserialize)]
struct CargoMetadata {
    packages: Vec<MetadataPackage>,
    workspace_members: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct MetadataPackage {
    id: String,
    name: String,
    dependencies: Vec<MetadataDependency>,
    targets: Vec<MetadataTarget>,
    manifest_path: PathBuf,
}

#[derive(Debug, Deserialize)]
struct MetadataDependency {
    name: String,
    kind: Option<String>,
    rename: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct MetadataTarget {
    name: String,
    kind: Vec<String>,
    src_path: PathBuf,
}

impl From<MetadataTarget> for PackageTarget {
    fn from(target: MetadataTarget) -> Self {
        Self {
            name: target.name,
            kind: target.kind,
            src_path: target.src_path,
        }
    }
}
