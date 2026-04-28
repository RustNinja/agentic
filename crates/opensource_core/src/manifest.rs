use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use toml::Value;

#[derive(Debug)]
pub struct Workspace {
    pub root: PathBuf,
    pub members: Vec<String>,
    pub packages: HashMap<String, Package>,
}

#[derive(Debug, Clone)]
pub struct Package {
    pub name: String,
    pub root: PathBuf,
    pub lib_path: PathBuf,
    pub dependencies: Vec<Dependency>,
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
    let members = root_value
        .get("workspace")
        .and_then(|workspace| workspace.get("members"))
        .and_then(Value::as_array)
        .ok_or("workspace Cargo.toml must define [workspace].members")?
        .iter()
        .map(|member| {
            member
                .as_str()
                .map(str::to_string)
                .ok_or("workspace member must be a string")
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut packages = HashMap::new();
    for member in &members {
        let package_root = root.join(member).canonicalize()?;
        let manifest_path = package_root.join("Cargo.toml");
        let manifest = read_manifest(&manifest_path)?;
        let package_table = manifest
            .get("package")
            .and_then(Value::as_table)
            .ok_or_else(|| format!("missing [package] in {}", manifest_path.display()))?;
        let name = package_table
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("missing package.name in {}", manifest_path.display()))?
            .to_string();

        let lib_path = manifest
            .get("lib")
            .and_then(|lib| lib.get("path"))
            .and_then(Value::as_str)
            .map(|path| package_root.join(path))
            .unwrap_or_else(|| package_root.join("src/lib.rs"));

        let dependencies = parse_dependencies(&manifest);

        packages.insert(
            name.clone(),
            Package {
                name,
                root: package_root,
                lib_path,
                dependencies,
            },
        );
    }

    Ok(Workspace {
        root,
        members,
        packages,
    })
}

fn read_manifest(path: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let text = fs::read_to_string(path)?;
    Ok(text.parse::<Value>()?)
}

fn parse_dependencies(manifest: &Value) -> Vec<Dependency> {
    let mut dependencies = Vec::new();

    for table_name in ["dependencies"] {
        let Some(table) = manifest.get(table_name).and_then(Value::as_table) else {
            continue;
        };

        for (alias, value) in table {
            let package = match value {
                Value::String(_) => alias.clone(),
                Value::Table(table) => table
                    .get("package")
                    .and_then(Value::as_str)
                    .unwrap_or(alias)
                    .to_string(),
                _ => alias.clone(),
            };

            dependencies.push(Dependency {
                alias: alias.clone(),
                package,
            });
        }
    }

    dependencies
}
