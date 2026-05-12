use std::{
    collections::{BTreeSet, HashMap},
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use serde::Deserialize;
use syn::{Expr, ImplItem, Item, Lit, Meta, TraitItem};
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
    pub required_features: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Dependency {
    pub alias: String,
    pub package: String,
}

pub fn load_workspace(root: &Path) -> Result<Workspace, Box<dyn std::error::Error>> {
    load_workspace_with_marker_targets(root, true)
}

pub fn load_workspace_without_marker_targets(
    root: &Path,
) -> Result<Workspace, Box<dyn std::error::Error>> {
    load_workspace_with_marker_targets(root, false)
}

fn load_workspace_with_marker_targets(
    root: &Path,
    prefer_marker_targets: bool,
) -> Result<Workspace, Box<dyn std::error::Error>> {
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

        let entry_target = entry_target(
            &package_root,
            &manifest,
            &metadata_package.targets,
            prefer_marker_targets,
        )?;
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

pub fn marked_workspace_packages(root: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let root = root.canonicalize()?;
    let root_manifest = root.join("Cargo.toml");
    let metadata = load_cargo_metadata(&root_manifest)?;
    let workspace_members = metadata
        .workspace_members
        .into_iter()
        .collect::<BTreeSet<_>>();
    let mut packages = metadata
        .packages
        .into_iter()
        .filter(|package| workspace_members.contains(&package.id))
        .filter(|package| {
            package
                .targets
                .iter()
                .filter(|target| target_is_parse_candidate(target))
                .any(|target| source_contains_opensourced_marker(&target.src_path))
        })
        .map(|package| package.name)
        .collect::<Vec<_>>();
    packages.sort();
    packages.dedup();
    Ok(packages)
}

fn read_manifest(path: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let text = fs::read_to_string(path)?;
    Ok(text.parse::<Value>()?)
}

fn load_cargo_metadata(manifest_path: &Path) -> Result<CargoMetadata, Box<dyn std::error::Error>> {
    let manifest_path_for_cargo = absolute_path(manifest_path)?;
    let working_dir = manifest_working_dir(&manifest_path_for_cargo);
    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--format-version=1")
        .arg("--no-deps")
        .arg("--manifest-path")
        .arg(&manifest_path_for_cargo)
        .current_dir(&working_dir)
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

fn absolute_path(path: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    Ok(std::env::current_dir()?.join(path))
}

fn manifest_working_dir(manifest_path: &Path) -> PathBuf {
    manifest_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}

fn entry_target(
    package_root: &Path,
    manifest: &Value,
    targets: &[MetadataTarget],
    prefer_marker_targets: bool,
) -> Result<PackageTarget, Box<dyn std::error::Error>> {
    if prefer_marker_targets {
        if let Some(target) = metadata_marker_target(targets)? {
            return Ok(package_target_from_metadata(package_root, manifest, target));
        }
    }

    if let Some(target) = metadata_entry_target(package_root, targets) {
        return Ok(package_target_from_metadata(package_root, manifest, target));
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
            required_features: Vec::new(),
        });
    }

    let default_lib = package_root.join("src/lib.rs");
    if default_lib.exists() {
        return Ok(PackageTarget {
            name: package_name_from_manifest(manifest),
            kind: vec!["lib".to_string()],
            src_path: default_lib,
            required_features: Vec::new(),
        });
    }

    Ok(PackageTarget {
        name: package_name_from_manifest(manifest),
        kind: vec!["bin".to_string()],
        src_path: package_root.join("src/main.rs"),
        required_features: Vec::new(),
    })
}

fn package_target_from_metadata(
    package_root: &Path,
    manifest: &Value,
    target: MetadataTarget,
) -> PackageTarget {
    let mut package_target: PackageTarget = target.into();
    if package_target.required_features.is_empty() {
        package_target.required_features =
            manifest_required_features_for_target(package_root, manifest, &package_target);
    }
    package_target
}

fn manifest_required_features_for_target(
    package_root: &Path,
    manifest: &Value,
    target: &PackageTarget,
) -> Vec<String> {
    target_manifest_tables(&target.kind)
        .into_iter()
        .find_map(|target_table| {
            manifest_required_features_from_table(
                package_root,
                manifest,
                target_table,
                &target.name,
                &target.src_path,
            )
        })
        .unwrap_or_default()
}

fn target_manifest_tables(kind: &[String]) -> Vec<&'static str> {
    let mut tables = Vec::new();
    for target_table in ["lib", "bin", "example", "test", "bench"] {
        if kind
            .iter()
            .any(|kind| kind == target_table || (target_table == "lib" && kind == "proc-macro"))
        {
            tables.push(target_table);
        }
    }
    tables
}

fn manifest_required_features_from_table(
    package_root: &Path,
    manifest: &Value,
    target_table: &str,
    target_name: &str,
    entry_source: &Path,
) -> Option<Vec<String>> {
    let value = manifest.get(target_table)?;
    if target_table == "lib" {
        return value
            .as_table()
            .and_then(required_features_from_target_table);
    }

    value.as_array()?.iter().find_map(|target| {
        named_manifest_target_matches_source(
            package_root,
            target_table,
            target,
            target_name,
            entry_source,
        )
        .then(|| {
            target
                .as_table()
                .and_then(required_features_from_target_table)
                .unwrap_or_default()
        })
    })
}

fn named_manifest_target_matches_source(
    package_root: &Path,
    target_table: &str,
    target: &Value,
    target_name: &str,
    entry_source: &Path,
) -> bool {
    let Some(table) = target.as_table() else {
        return false;
    };

    if let Some(path) = table.get("path").and_then(Value::as_str) {
        return package_root
            .join(path)
            .canonicalize()
            .is_ok_and(|path| path == entry_source);
    }

    let name = table
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or(target_name);

    default_named_target_paths(package_root, target_table, name)
        .into_iter()
        .filter_map(|path| path.canonicalize().ok())
        .any(|path| path == entry_source)
}

fn required_features_from_target_table(
    table: &toml::map::Map<String, Value>,
) -> Option<Vec<String>> {
    let mut features = table
        .get("required-features")?
        .as_array()?
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect::<Vec<_>>();
    features.sort();
    features.dedup();
    Some(features)
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
    let mut marked_targets = targets
        .iter()
        .filter(|target| target_is_parse_candidate(target))
        .filter(|target| source_contains_opensourced_marker(&target.src_path))
        .cloned()
        .collect::<Vec<_>>();
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

    let Ok(syntax) = syn::parse_file(&text) else {
        return raw_text_contains_opensourced_attr(&text);
    };
    if syntax.items.iter().any(item_contains_opensourced_marker) {
        return true;
    }

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

fn raw_text_contains_opensourced_attr(text: &str) -> bool {
    text.lines().any(|line| {
        let line = line.trim_start();
        line.starts_with("#[opensourced") || line.starts_with("#[ opensourced")
    })
}

fn item_contains_opensourced_marker(item: &Item) -> bool {
    if item_has_opensourced_attr(item) {
        return true;
    }
    match item {
        Item::Impl(item_impl) => item_impl.items.iter().any(impl_item_has_opensourced_attr),
        Item::Mod(item_mod) => item_mod
            .content
            .as_ref()
            .is_some_and(|(_, items)| items.iter().any(item_contains_opensourced_marker)),
        Item::Trait(item_trait) => item_trait.items.iter().any(trait_item_has_opensourced_attr),
        _ => false,
    }
}

fn item_has_opensourced_attr(item: &Item) -> bool {
    match item {
        Item::Const(item) => attrs_contain_opensourced_marker(&item.attrs),
        Item::Enum(item) => attrs_contain_opensourced_marker(&item.attrs),
        Item::ExternCrate(item) => attrs_contain_opensourced_marker(&item.attrs),
        Item::Fn(item) => attrs_contain_opensourced_marker(&item.attrs),
        Item::ForeignMod(item) => attrs_contain_opensourced_marker(&item.attrs),
        Item::Impl(item) => attrs_contain_opensourced_marker(&item.attrs),
        Item::Macro(item) => attrs_contain_opensourced_marker(&item.attrs),
        Item::Mod(item) => attrs_contain_opensourced_marker(&item.attrs),
        Item::Static(item) => attrs_contain_opensourced_marker(&item.attrs),
        Item::Struct(item) => attrs_contain_opensourced_marker(&item.attrs),
        Item::Trait(item) => attrs_contain_opensourced_marker(&item.attrs),
        Item::TraitAlias(item) => attrs_contain_opensourced_marker(&item.attrs),
        Item::Type(item) => attrs_contain_opensourced_marker(&item.attrs),
        Item::Union(item) => attrs_contain_opensourced_marker(&item.attrs),
        Item::Use(item) => attrs_contain_opensourced_marker(&item.attrs),
        Item::Verbatim(_) => false,
        _ => false,
    }
}

fn impl_item_has_opensourced_attr(item: &ImplItem) -> bool {
    match item {
        ImplItem::Const(item) => attrs_contain_opensourced_marker(&item.attrs),
        ImplItem::Fn(item) => attrs_contain_opensourced_marker(&item.attrs),
        ImplItem::Macro(item) => attrs_contain_opensourced_marker(&item.attrs),
        ImplItem::Type(item) => attrs_contain_opensourced_marker(&item.attrs),
        ImplItem::Verbatim(_) => false,
        _ => false,
    }
}

fn trait_item_has_opensourced_attr(item: &TraitItem) -> bool {
    match item {
        TraitItem::Const(item) => attrs_contain_opensourced_marker(&item.attrs),
        TraitItem::Fn(item) => attrs_contain_opensourced_marker(&item.attrs),
        TraitItem::Macro(item) => attrs_contain_opensourced_marker(&item.attrs),
        TraitItem::Type(item) => attrs_contain_opensourced_marker(&item.attrs),
        TraitItem::Verbatim(_) => false,
        _ => false,
    }
}

fn attrs_contain_opensourced_marker(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attribute| {
        attribute
            .path()
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "opensourced")
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

fn default_named_target_paths(package_root: &Path, target_table: &str, name: &str) -> Vec<PathBuf> {
    match target_table {
        "bin" => vec![
            package_root.join("src/main.rs"),
            package_root.join("src/bin").join(format!("{name}.rs")),
            package_root.join("src/bin").join(name).join("main.rs"),
        ],
        "example" => vec![
            package_root.join("examples").join(format!("{name}.rs")),
            package_root.join("examples").join(name).join("main.rs"),
        ],
        "test" => vec![
            package_root.join("tests").join(format!("{name}.rs")),
            package_root.join("tests").join(name).join("main.rs"),
        ],
        "bench" => vec![
            package_root.join("benches").join(format!("{name}.rs")),
            package_root.join("benches").join(name).join("main.rs"),
        ],
        _ => Vec::new(),
    }
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
    #[serde(default)]
    required_features: Vec<String>,
}

impl From<MetadataTarget> for PackageTarget {
    fn from(target: MetadataTarget) -> Self {
        Self {
            name: target.name,
            kind: target.kind,
            src_path: target.src_path,
            required_features: target.required_features,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn explicit_root_workspace_load_skips_marker_target_scan() {
        let root = temp_workspace("manifest-explicit-root-entry-target");
        fs::create_dir_all(root.join("app/src/bin")).expect("fixture dirs should create");
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        )
        .expect("workspace manifest should write");
        fs::write(
            root.join("app/Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n\n[[bin]]\nname = \"tool\"\npath = \"src/bin/tool.rs\"\n",
        )
        .expect("package manifest should write");
        fs::write(root.join("app/src/lib.rs"), "pub fn library_root() {}\n")
            .expect("lib source should write");
        fs::write(
            root.join("app/src/bin/tool.rs"),
            "#[opensourced::opensourced]\npub fn marked_bin_root() {}\nfn main() {}\n",
        )
        .expect("bin source should write");

        let marker_workspace = load_workspace(&root).expect("marker workspace should load");
        let marker_target = &marker_workspace.packages["app"].entry_target;
        assert!(
            marker_target.kind.iter().any(|kind| kind == "bin"),
            "{marker_target:?}"
        );
        assert!(marker_target.src_path.ends_with("src/bin/tool.rs"));

        let explicit_workspace = load_workspace_without_marker_targets(&root)
            .expect("explicit-root workspace should load");
        let explicit_target = &explicit_workspace.packages["app"].entry_target;
        assert!(
            explicit_target.kind.iter().any(|kind| kind == "lib"),
            "{explicit_target:?}"
        );
        assert!(explicit_target.src_path.ends_with("src/lib.rs"));

        fs::remove_dir_all(root).ok();
    }

    fn temp_workspace(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("slicer-{label}-{}-{nanos}", std::process::id()))
    }
}
