use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use syn::{Expr, Item, Lit, Meta};
use toml::Value;

#[derive(Debug, Clone)]
pub struct PreflightOptions {
    pub manifest_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightReport {
    pub manifest_path: PathBuf,
    pub success: bool,
    pub diagnostics: Vec<PreflightDiagnostic>,
    pub packages: usize,
    pub rust_files: usize,
    pub local_path_dependencies: usize,
    pub external_dependencies: usize,
    pub build_scripts: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightDiagnostic {
    pub level: String,
    pub code: String,
    pub message: String,
    pub path: Option<PathBuf>,
}

impl PreflightReport {
    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.level == "error")
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.level == "warning")
            .count()
    }
}

pub fn preflight_workspace(
    options: PreflightOptions,
) -> Result<PreflightReport, Box<dyn std::error::Error>> {
    let mut checker = Checker {
        report: PreflightReport {
            manifest_path: options.manifest_path.clone(),
            success: false,
            diagnostics: Vec::new(),
            packages: 0,
            rust_files: 0,
            local_path_dependencies: 0,
            external_dependencies: 0,
            build_scripts: 0,
        },
        parsed_rust_files: BTreeSet::new(),
        visited_module_files: BTreeSet::new(),
    };
    checker.check_workspace(&options.manifest_path)?;
    checker.report.success = checker.report.error_count() == 0;
    Ok(checker.report)
}

pub fn write_preflight_report(
    report: &PreflightReport,
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(report)?;
    fs::write(path, json)?;
    Ok(())
}

struct Checker {
    report: PreflightReport,
    parsed_rust_files: BTreeSet<PathBuf>,
    visited_module_files: BTreeSet<PathBuf>,
}

impl Checker {
    fn check_workspace(&mut self, manifest_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if !manifest_path.exists() {
            self.error(
                "missing-workspace-manifest",
                format!(
                    "workspace manifest does not exist: {}",
                    manifest_path.display()
                ),
                Some(manifest_path.to_path_buf()),
            );
            return Ok(());
        }
        let workspace_root = manifest_path
            .parent()
            .ok_or_else(|| format!("manifest path has no parent: {}", manifest_path.display()))?;
        let manifest = read_manifest(manifest_path, &mut self.report.diagnostics);
        self.check_dependency_sections(workspace_root, &manifest);

        let members = workspace_members(&manifest);
        for member in members {
            self.check_package(workspace_root, &member);
        }

        Ok(())
    }

    fn check_package(&mut self, workspace_root: &Path, member: &str) {
        let package_root = workspace_root.join(member);
        let manifest_path = package_root.join("Cargo.toml");
        if !manifest_path.exists() {
            self.error(
                "missing-package-manifest",
                format!("workspace member {member:?} has no Cargo.toml"),
                Some(manifest_path),
            );
            return;
        }

        let manifest = read_manifest(&manifest_path, &mut self.report.diagnostics);
        self.report.packages += 1;
        self.check_dependency_sections(&package_root, &manifest);
        self.check_build_script(&package_root, &manifest);
        self.check_package_targets(&package_root, &manifest);
        self.check_rust_syntax_under(&package_root);
    }

    fn check_package_targets(&mut self, package_root: &Path, manifest: &Value) {
        let mut entries = BTreeSet::new();
        if let Some(path) = manifest
            .get("lib")
            .and_then(|lib| lib.get("path"))
            .and_then(Value::as_str)
        {
            entries.insert(package_root.join(path));
        } else {
            let lib = package_root.join("src/lib.rs");
            let main = package_root.join("src/main.rs");
            if lib.exists() {
                entries.insert(lib);
            } else if main.exists() {
                entries.insert(main);
            }
        }

        if let Some(bins) = manifest.get("bin").and_then(Value::as_array) {
            for bin in bins {
                if let Some(path) = bin.get("path").and_then(Value::as_str) {
                    entries.insert(package_root.join(path));
                } else if let Some(name) = bin.get("name").and_then(Value::as_str) {
                    entries.extend(default_bin_entries(package_root, name));
                }
            }
        }
        entries.extend(auto_discovered_bin_entries(package_root));

        if entries.is_empty() {
            self.error(
                "missing-package-entry",
                format!(
                    "package has no src/lib.rs, src/main.rs, explicit target path, or auto-discovered bin under {}",
                    package_root.display()
                ),
                Some(package_root.to_path_buf()),
            );
            return;
        }

        for entry in entries {
            if !entry.exists() {
                self.error(
                    "missing-target-source",
                    format!("package target source does not exist: {}", entry.display()),
                    Some(entry),
                );
                continue;
            }
            let module_dir = entry.parent().unwrap_or(package_root).to_path_buf();
            self.check_module_tree(&entry, &module_dir);
        }
    }

    fn check_dependency_sections(&mut self, manifest_dir: &Path, manifest: &Value) {
        for section in ["dependencies", "build-dependencies", "dev-dependencies"] {
            if let Some(dependencies) = manifest.get(section).and_then(Value::as_table) {
                self.check_dependencies_table(manifest_dir, dependencies);
            }
        }

        let Some(targets) = manifest.get("target").and_then(Value::as_table) else {
            return;
        };
        for target in targets.values() {
            for section in ["dependencies", "build-dependencies", "dev-dependencies"] {
                if let Some(dependencies) = target.get(section).and_then(Value::as_table) {
                    self.check_dependencies_table(manifest_dir, dependencies);
                }
            }
        }
    }

    fn check_dependencies_table(
        &mut self,
        manifest_dir: &Path,
        dependencies: &toml::map::Map<String, Value>,
    ) {
        for (name, dependency) in dependencies {
            let Some(table) = dependency.as_table() else {
                self.report.external_dependencies += 1;
                continue;
            };
            if table.get("workspace").and_then(Value::as_bool) == Some(true) {
                continue;
            }
            if let Some(path) = table.get("path").and_then(Value::as_str) {
                self.report.local_path_dependencies += 1;
                let dependency_root = manifest_dir.join(path);
                let dependency_manifest = dependency_root.join("Cargo.toml");
                if !dependency_manifest.exists() {
                    self.error(
                        "missing-path-dependency",
                        format!(
                            "local dependency {name:?} points to missing Cargo.toml: {}",
                            dependency_manifest.display()
                        ),
                        Some(dependency_manifest),
                    );
                }
            } else {
                self.report.external_dependencies += 1;
            }
        }
    }

    fn check_build_script(&mut self, package_root: &Path, manifest: &Value) {
        let build = manifest
            .get("package")
            .and_then(|package| package.get("build"));
        let Some(build_path) = build_script_path(package_root, build) else {
            return;
        };
        self.report.build_scripts += 1;
        if !build_path.exists() {
            self.error(
                "missing-build-script",
                format!("build script does not exist: {}", build_path.display()),
                Some(build_path),
            );
        }
    }

    fn check_rust_syntax_under(&mut self, root: &Path) {
        let mut stack = vec![root.to_path_buf()];
        while let Some(path) = stack.pop() {
            let Ok(metadata) = fs::metadata(&path) else {
                continue;
            };
            if metadata.is_dir() {
                if is_ignored_dir(&path) {
                    continue;
                }
                let Ok(entries) = fs::read_dir(&path) else {
                    continue;
                };
                for entry in entries.flatten() {
                    stack.push(entry.path());
                }
            } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
                self.check_rust_syntax(&path);
            }
        }
    }

    fn check_rust_syntax(&mut self, path: &Path) -> Option<syn::File> {
        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) => {
                self.error(
                    "unreadable-rust-file",
                    format!("failed to read {}: {error}", path.display()),
                    Some(path.to_path_buf()),
                );
                return None;
            }
        };
        match syn::parse_file(&text) {
            Ok(syntax) => {
                let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
                if self.parsed_rust_files.insert(canonical) {
                    self.report.rust_files += 1;
                }
                Some(syntax)
            }
            Err(error) => {
                self.error(
                    "rust-syntax-error",
                    format!("failed to parse {}: {error}", path.display()),
                    Some(path.to_path_buf()),
                );
                None
            }
        }
    }

    fn check_module_tree(&mut self, file_path: &Path, module_dir: &Path) {
        let canonical = file_path
            .canonicalize()
            .unwrap_or_else(|_| file_path.to_path_buf());
        if !self.visited_module_files.insert(canonical) {
            return;
        }
        let Some(syntax) = self.check_rust_syntax(file_path) else {
            return;
        };
        self.check_module_items(file_path, module_dir, &syntax.items);
    }

    fn check_module_items(&mut self, file_path: &Path, module_dir: &Path, items: &[Item]) {
        for item in items {
            let Item::Mod(item_mod) = item else {
                continue;
            };
            let name = item_mod.ident.to_string();
            let child_module_dir = module_dir.join(&name);
            if let Some((_, inline_items)) = &item_mod.content {
                self.check_module_items(file_path, &child_module_dir, inline_items);
                continue;
            }

            let Some(child_path) = resolve_external_module(module_dir, &name, path_attr(item_mod))
            else {
                self.error(
                    "missing-module-file",
                    format!(
                        "module {name:?} declared in {} has no generated source file",
                        file_path.display()
                    ),
                    Some(file_path.to_path_buf()),
                );
                continue;
            };
            let child_dir =
                if child_path.file_name().and_then(|name| name.to_str()) == Some("mod.rs") {
                    child_path.parent().unwrap_or(module_dir).to_path_buf()
                } else {
                    child_path.parent().unwrap_or(module_dir).join(&name)
                };
            self.check_module_tree(&child_path, &child_dir);
        }
    }

    fn error(&mut self, code: &str, message: String, path: Option<PathBuf>) {
        self.report.diagnostics.push(PreflightDiagnostic {
            level: "error".to_string(),
            code: code.to_string(),
            message,
            path,
        });
    }
}

fn read_manifest(path: &Path, diagnostics: &mut Vec<PreflightDiagnostic>) -> Value {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) => {
            diagnostics.push(PreflightDiagnostic {
                level: "error".to_string(),
                code: "unreadable-manifest".to_string(),
                message: format!("failed to read {}: {error}", path.display()),
                path: Some(path.to_path_buf()),
            });
            return Value::Table(toml::map::Map::new());
        }
    };
    match text.parse::<Value>() {
        Ok(value) => value,
        Err(error) => {
            diagnostics.push(PreflightDiagnostic {
                level: "error".to_string(),
                code: "manifest-syntax-error".to_string(),
                message: format!("failed to parse {}: {error}", path.display()),
                path: Some(path.to_path_buf()),
            });
            Value::Table(toml::map::Map::new())
        }
    }
}

fn workspace_members(manifest: &Value) -> Vec<String> {
    let members = manifest
        .get("workspace")
        .and_then(|workspace| workspace.get("members"))
        .and_then(Value::as_array)
        .map(|members| {
            members
                .iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .filter(|members| !members.is_empty());
    members.unwrap_or_else(|| vec![".".to_string()])
}

fn build_script_path(package_root: &Path, build: Option<&Value>) -> Option<PathBuf> {
    match build {
        Some(Value::Boolean(false)) => None,
        Some(Value::String(path)) => Some(package_root.join(path)),
        Some(_) => Some(package_root.join("build.rs")),
        None => {
            let default = package_root.join("build.rs");
            default.exists().then_some(default)
        }
    }
}

fn is_ignored_dir(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| matches!(name, ".git" | "target"))
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
) -> Option<PathBuf> {
    if let Some(path_attr) = path_attr {
        let path = if path_attr.is_absolute() {
            path_attr
        } else {
            module_dir.join(path_attr)
        };
        return path.exists().then_some(path);
    }

    let file_module = module_dir.join(format!("{name}.rs"));
    if file_module.exists() {
        return Some(file_module);
    }
    let mod_module = module_dir.join(name).join("mod.rs");
    mod_module.exists().then_some(mod_module)
}

fn auto_discovered_bin_entries(package_root: &Path) -> BTreeSet<PathBuf> {
    let bin_root = package_root.join("src/bin");
    let Ok(entries) = fs::read_dir(bin_root) else {
        return BTreeSet::new();
    };

    entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            let file_type = entry.file_type().ok()?;
            if file_type.is_file() && path.extension().is_some_and(|extension| extension == "rs") {
                return Some(path);
            }
            if file_type.is_dir() {
                let main = path.join("main.rs");
                return main.exists().then_some(main);
            }
            None
        })
        .collect()
}

fn default_bin_entries(package_root: &Path, name: &str) -> Vec<PathBuf> {
    [
        package_root.join("src/main.rs"),
        package_root.join("src/bin").join(format!("{name}.rs")),
        package_root.join("src/bin").join(name).join("main.rs"),
    ]
    .into_iter()
    .filter(|path| path.exists())
    .collect()
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::{generate, GenerateOptions};

    use super::{preflight_workspace, PreflightOptions};

    #[test]
    fn validates_generated_fixture_without_cargo_check() {
        let output = temp_output("preflight-output");
        generate(GenerateOptions {
            workspace_root: workspace_root(),
            output_root: output.clone(),
        })
        .expect("reduction should succeed");

        let report = preflight_workspace(PreflightOptions {
            manifest_path: output.join("Cargo.toml"),
        })
        .expect("preflight should run");

        assert!(report.success, "{:?}", report.diagnostics);
        assert_eq!(report.error_count(), 0);
        assert!(report.packages > 0);
        assert!(report.rust_files > 0);
    }

    #[test]
    fn reports_missing_external_module_without_building() {
        let root = temp_output("preflight-missing-module");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\n",
        );
        write(
            root.join("app/Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        write(root.join("app/src/lib.rs"), "pub mod missing;\n");

        let report = preflight_workspace(PreflightOptions {
            manifest_path: root.join("Cargo.toml"),
        })
        .expect("preflight should run");

        assert!(!report.success);
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "missing-module-file"));
    }

    #[test]
    fn accepts_auto_discovered_bin_targets() {
        let root = temp_output("preflight-auto-bin");
        write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\n",
        );
        write(
            root.join("app/Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        write(
            root.join("app/src/bin/tool.rs"),
            "fn main() {\n    let _ = 1 + 1;\n}\n",
        );

        let report = preflight_workspace(PreflightOptions {
            manifest_path: root.join("Cargo.toml"),
        })
        .expect("preflight should run");

        assert!(report.success, "{:?}", report.diagnostics);
        assert_eq!(report.error_count(), 0);
        assert!(report.rust_files > 0);
    }

    fn workspace_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .expect("core crate should live under crates/opensource_core")
            .to_path_buf()
    }

    fn temp_output(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("opensourced-{label}-{unique}"));
        if path.exists() {
            fs::remove_dir_all(&path).unwrap();
        }
        path
    }

    fn write(path: PathBuf, contents: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, contents).unwrap();
    }
}
