use std::{
    collections::{BTreeSet, HashMap},
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use serde::Deserialize;
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
    pub dependencies: Vec<Dependency>,
    pub manifest: Value,
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

        let lib_path = entry_source_path(&package_root, &manifest, &metadata_package.targets);

        let dependencies = metadata_dependencies(&metadata_package.dependencies);

        packages.insert(
            metadata_package.name.clone(),
            Package {
                name: metadata_package.name,
                root: package_root,
                lib_path,
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

fn entry_source_path(package_root: &Path, manifest: &Value, targets: &[MetadataTarget]) -> PathBuf {
    if let Some(path) = metadata_marker_source_path(targets) {
        return path;
    }

    if let Some(path) = metadata_entry_source_path(package_root, targets) {
        return path;
    }

    if let Some(path) = manifest
        .get("lib")
        .and_then(|lib| lib.get("path"))
        .and_then(Value::as_str)
    {
        return package_root.join(path);
    }

    let default_lib = package_root.join("src/lib.rs");
    if default_lib.exists() {
        return default_lib;
    }

    package_root.join("src/main.rs")
}

fn metadata_marker_source_path(targets: &[MetadataTarget]) -> Option<PathBuf> {
    let mut paths = targets
        .iter()
        .filter(|target| target_is_parse_candidate(target))
        .map(|target| target.src_path.clone())
        .collect::<Vec<_>>();
    paths.sort();
    paths.dedup();
    paths
        .into_iter()
        .find(|path| source_contains_opensourced_marker(path))
}

fn target_is_parse_candidate(target: &MetadataTarget) -> bool {
    target
        .kind
        .iter()
        .any(|kind| matches!(kind.as_str(), "lib" | "proc-macro" | "bin" | "example"))
}

fn source_contains_opensourced_marker(path: &Path) -> bool {
    fs::read_to_string(path).is_ok_and(|text| {
        text.contains("#[opensourced")
            || text.contains("#[ opensourced")
            || text.contains("opensourced::opensourced")
    })
}

fn metadata_entry_source_path(package_root: &Path, targets: &[MetadataTarget]) -> Option<PathBuf> {
    for preferred_kind in ["lib", "proc-macro"] {
        if let Some(target) = targets
            .iter()
            .find(|target| target.kind.iter().any(|kind| kind == preferred_kind))
        {
            return Some(target.src_path.clone());
        }
    }
    let default_bin = package_root.join("src/main.rs");
    if let Some(target) = targets.iter().find(|target| {
        target.kind.iter().any(|kind| kind == "bin") && target.src_path == default_bin
    }) {
        return Some(target.src_path.clone());
    }
    if let Some(target) = targets
        .iter()
        .find(|target| target.kind.iter().any(|kind| kind == "bin"))
    {
        return Some(target.src_path.clone());
    }
    targets
        .iter()
        .find(|target| {
            target
                .kind
                .iter()
                .all(|kind| !matches!(kind.as_str(), "test" | "bench" | "example"))
        })
        .map(|target| target.src_path.clone())
}

fn metadata_dependencies(metadata_dependencies: &[MetadataDependency]) -> Vec<Dependency> {
    let mut dependencies = metadata_dependencies
        .iter()
        .filter(|dependency| dependency.kind.as_deref() != Some("dev"))
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

#[derive(Debug, Deserialize)]
struct MetadataTarget {
    kind: Vec<String>,
    src_path: PathBuf,
}
