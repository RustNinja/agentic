use std::{
    collections::BTreeSet,
    fs,
    path::{Component, Path, PathBuf},
};

use proc_macro2::{Span, TokenStream, TokenTree};
use quote::ToTokens;
use syn::{parse_quote, ImplItem, Item, Type, UseTree};
use toml::{value::Table, Value};

use crate::{
    manifest::Package,
    model::{CallableId, ItemId, ItemKind, Project, ReducedProject, SourceFile},
    reduce::{is_cfg_test_attr, is_opensourced_attr, is_test_attr},
};

pub fn write_reduced_workspace(
    project: &Project,
    reduced: &ReducedProject,
    output_root: &Path,
) -> Result<usize, Box<dyn std::error::Error>> {
    if output_root.exists() {
        fs::remove_dir_all(output_root)?;
    }
    fs::create_dir_all(output_root)?;

    write_workspace_manifest(project, reduced, output_root)?;

    let mut files_written = 1;
    for package_name in &reduced.packages {
        let package = project
            .workspace
            .packages
            .get(package_name)
            .ok_or_else(|| format!("unknown package {package_name}"))?;

        let package_output = output_root.join(package_name);
        fs::create_dir_all(package_output.join("src"))?;
        write_package_manifest(
            project,
            reduced,
            package_name,
            &package_output.join("Cargo.toml"),
        )?;
        files_written += 1;

        if let Some(build_script) = build_script_to_render(project, reduced, package) {
            let relative_path = build_script.strip_prefix(&package.root)?;
            let output_path = package_output.join(relative_path);
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&build_script, output_path)?;
            files_written += 1;
            files_written += copy_build_script_assets(package, &build_script, &package_output)?;
        }

        for source in project
            .files
            .values()
            .filter(|source| &source.package == package_name)
        {
            if !module_should_render(project, reduced, &source.package, &source.module_path) {
                continue;
            }
            let mut transformed = transform_file(
                project,
                reduced,
                &source.package,
                &source.module_path,
                &source.syntax,
            );
            if is_binary_entry_source(source) {
                prune_stub_binary_root_uses(&mut transformed);
                ensure_main_function(&mut transformed);
            }
            let relative_path = source.path.strip_prefix(&package.root)?;
            let output_path = package_output.join(relative_path);
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(output_path, prettyplease::unparse(&transformed))?;
            files_written += 1;
        }
    }

    Ok(files_written)
}

fn build_script_path(package: &Package) -> Option<PathBuf> {
    let package_table = package.manifest.get("package").and_then(Value::as_table);
    match package_table.and_then(|table| table.get("build")) {
        Some(Value::Boolean(false)) => None,
        Some(Value::String(path)) => {
            let path = package.root.join(path);
            path.exists().then_some(path)
        }
        _ => {
            let path = package.root.join("build.rs");
            path.exists().then_some(path)
        }
    }
}

fn build_script_to_render(
    project: &Project,
    reduced: &ReducedProject,
    package: &Package,
) -> Option<PathBuf> {
    build_script_should_render(project, reduced, package).then(|| build_script_path(package))?
}

fn build_script_should_render(
    project: &Project,
    reduced: &ReducedProject,
    package: &Package,
) -> bool {
    let Some(build_script) = build_script_path(package) else {
        return false;
    };
    if build_script_is_uniffi_only(&build_script)
        && !package_rendered_sources_mention_ident(project, reduced, &package.name, "uniffi")
    {
        return false;
    }
    true
}

fn build_script_is_uniffi_only(path: &Path) -> bool {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| syn::parse_file(&text).ok())
        .is_some_and(|syntax| {
            let tokens = syntax.to_token_stream();
            token_stream_mentions_ident(&tokens, "uniffi")
                && token_stream_mentions_ident(&tokens, "generate_scaffolding")
        })
}

fn is_binary_entry_source(source: &SourceFile) -> bool {
    source.module_path.is_empty()
        && source
            .path
            .file_name()
            .is_some_and(|file_name| file_name == "main.rs")
}

fn ensure_main_function(file: &mut syn::File) {
    if file_has_main_function(file) {
        return;
    }

    file.items.push(parse_quote! {
        fn main() {}
    });
}

fn prune_stub_binary_root_uses(file: &mut syn::File) {
    if file_has_main_function(file)
        || file
            .items
            .iter()
            .any(|item| !matches!(item, Item::Mod(_) | Item::Use(_)))
    {
        return;
    }

    file.items.retain(|item| !matches!(item, Item::Use(_)));
}

fn file_has_main_function(file: &syn::File) -> bool {
    file.items
        .iter()
        .any(|item| matches!(item, Item::Fn(function) if function.sig.ident == "main"))
}

fn copy_build_script_assets(
    package: &Package,
    build_script: &Path,
    package_output: &Path,
) -> Result<usize, Box<dyn std::error::Error>> {
    let text = fs::read_to_string(build_script)?;
    let syntax = syn::parse_file(&text)
        .map_err(|error| format!("failed to parse {}: {error}", build_script.display()))?;
    let mut candidates = BTreeSet::new();
    collect_string_literal_paths(&syntax.to_token_stream(), &mut candidates);

    let mut copied = 0;
    for candidate in candidates {
        let path = package.root.join(candidate);
        if !path.exists() {
            continue;
        }
        copied += copy_non_rust_path_assets(package, &path, package_output)?;
    }

    Ok(copied)
}

fn collect_string_literal_paths(tokens: &TokenStream, candidates: &mut BTreeSet<PathBuf>) {
    for token in tokens.clone() {
        match token {
            TokenTree::Literal(literal) => {
                let Ok(literal) = syn::parse2::<syn::LitStr>(literal.to_token_stream()) else {
                    continue;
                };
                if let Some(path) = build_script_asset_path(&literal.value()) {
                    candidates.insert(path);
                }
            }
            TokenTree::Group(group) => collect_string_literal_paths(&group.stream(), candidates),
            TokenTree::Ident(_) | TokenTree::Punct(_) => {}
        }
    }
}

fn build_script_asset_path(value: &str) -> Option<PathBuf> {
    if value.is_empty()
        || value.contains('\n')
        || value.contains('*')
        || value.starts_with("cargo:")
        || value.starts_with('$')
    {
        return None;
    }

    let path = PathBuf::from(value);
    if path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return None;
    }

    Some(path)
}

fn copy_non_rust_path_assets(
    package: &Package,
    path: &Path,
    package_output: &Path,
) -> Result<usize, Box<dyn std::error::Error>> {
    if path.is_file() {
        if should_copy_asset(path) {
            copy_asset(package, path, package_output)?;
            return Ok(1);
        }
        return Ok(0);
    }

    let mut copied = 0;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        if file_name == "target" || file_name == ".git" {
            continue;
        }

        if entry.file_type()?.is_dir() {
            copied += copy_non_rust_path_assets(package, &path, package_output)?;
            continue;
        }

        if should_copy_asset(&path) {
            copy_asset(package, &path, package_output)?;
            copied += 1;
        }
    }

    Ok(copied)
}

fn copy_asset(
    package: &Package,
    path: &Path,
    package_output: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let relative_path = path.strip_prefix(&package.root)?;
    let output_path = package_output.join(relative_path);
    if output_path.exists() {
        return Ok(());
    }
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(path, output_path)?;
    Ok(())
}

fn should_copy_asset(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    if matches!(file_name, "Cargo.toml" | "Cargo.lock") {
        return false;
    }
    !path.extension().is_some_and(|extension| extension == "rs")
}

fn write_workspace_manifest(
    project: &Project,
    reduced: &ReducedProject,
    output_root: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut root = Table::new();
    let mut workspace = Table::new();

    workspace.insert(
        "members".to_string(),
        Value::Array(
            reduced
                .packages
                .iter()
                .map(|package| Value::String(package.clone()))
                .collect(),
        ),
    );

    if let Some(resolver) = project
        .workspace
        .manifest
        .get("workspace")
        .and_then(|workspace| workspace.get("resolver"))
    {
        workspace.insert("resolver".to_string(), resolver.clone());
    } else {
        workspace.insert("resolver".to_string(), Value::String("2".to_string()));
    }

    let workspace_package = project
        .workspace
        .manifest
        .get("workspace")
        .and_then(|workspace| workspace.get("package"))
        .cloned()
        .unwrap_or_else(default_workspace_package);
    workspace.insert("package".to_string(), workspace_package);

    let workspace_dependencies = retained_workspace_dependencies(project, reduced);
    if !workspace_dependencies.is_empty() {
        workspace.insert(
            "dependencies".to_string(),
            Value::Table(workspace_dependencies),
        );
    }

    root.insert("workspace".to_string(), Value::Table(workspace));
    fs::write(
        output_root.join("Cargo.toml"),
        toml::to_string_pretty(&Value::Table(root))?,
    )?;
    Ok(())
}

fn write_package_manifest(
    project: &Project,
    reduced: &ReducedProject,
    package_name: &str,
    output_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let package = project
        .workspace
        .packages
        .get(package_name)
        .ok_or_else(|| format!("unknown package {package_name}"))?;

    let mut manifest = Table::new();
    if let Some(package_table) = package.manifest.get("package").and_then(Value::as_table) {
        manifest.insert("package".to_string(), Value::Table(package_table.clone()));
    } else {
        manifest.insert("package".to_string(), default_package(&package.name));
    }

    for key in ["lib", "bin"] {
        if let Some(value) = package.manifest.get(key) {
            manifest.insert(key.to_string(), value.clone());
        }
    }

    let mut retained_dependency_aliases = BTreeSet::new();
    let dependencies = transformed_dependencies(
        project,
        reduced,
        package_name,
        "dependencies",
        DependencyRetention::SourceMentioned,
        &mut retained_dependency_aliases,
    )?;
    if !dependencies.is_empty() {
        manifest.insert("dependencies".to_string(), Value::Table(dependencies));
    }

    if build_script_should_render(project, reduced, package) {
        let build_dependencies = transformed_dependencies(
            project,
            reduced,
            package_name,
            "build-dependencies",
            DependencyRetention::BuildScript,
            &mut retained_dependency_aliases,
        )?;
        if !build_dependencies.is_empty() {
            manifest.insert(
                "build-dependencies".to_string(),
                Value::Table(build_dependencies),
            );
        }
    }

    if let Some(target_dependencies) = transformed_target_dependencies(
        project,
        reduced,
        package_name,
        &mut retained_dependency_aliases,
    )? {
        manifest.insert("target".to_string(), Value::Table(target_dependencies));
    }

    if let Some(features) = transformed_features(package, &retained_dependency_aliases) {
        manifest.insert("features".to_string(), features);
    }

    fs::write(
        output_path,
        toml::to_string_pretty(&Value::Table(manifest))?,
    )?;
    Ok(())
}

fn default_workspace_package() -> Value {
    let mut package = Table::new();
    package.insert("edition".to_string(), Value::String("2021".to_string()));
    package.insert("version".to_string(), Value::String("0.1.0".to_string()));
    package.insert("license".to_string(), Value::String("MIT".to_string()));
    Value::Table(package)
}

fn default_package(name: &str) -> Value {
    let mut package = Table::new();
    package.insert("name".to_string(), Value::String(name.to_string()));
    package.insert("version".to_string(), workspace_reference_value());
    package.insert("edition".to_string(), workspace_reference_value());
    package.insert("license".to_string(), workspace_reference_value());
    Value::Table(package)
}

fn workspace_reference_value() -> Value {
    let mut table = Table::new();
    table.insert("workspace".to_string(), Value::Boolean(true));
    Value::Table(table)
}

fn retained_workspace_dependencies(project: &Project, reduced: &ReducedProject) -> Table {
    let Some(source_dependencies) = project
        .workspace
        .manifest
        .get("workspace")
        .and_then(|workspace| workspace.get("dependencies"))
        .and_then(Value::as_table)
    else {
        return Table::new();
    };

    let mut dependencies = Table::new();
    for package_name in &reduced.packages {
        let Some(package) = project.workspace.packages.get(package_name) else {
            continue;
        };
        for (table_name, table) in package_dependency_tables(package) {
            let retention = if table_name == "build-dependencies"
                && build_script_should_render(project, reduced, package)
            {
                DependencyRetention::BuildScript
            } else {
                DependencyRetention::SourceMentioned
            };

            for (alias, value) in table {
                let dependency_package = dependency_package_name(alias, value);
                if is_marker_dependency(alias, &dependency_package)
                    || project.workspace.packages.contains_key(&dependency_package)
                    || !dependency_uses_workspace(value)
                    || !dependency_should_render(project, reduced, package_name, alias, retention)
                {
                    continue;
                }
                if let Some(source) = source_dependencies.get(alias) {
                    dependencies.insert(alias.clone(), source.clone());
                }
            }
        }
    }

    dependencies
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum DependencyRetention {
    SourceMentioned,
    BuildScript,
}

fn transformed_dependencies(
    project: &Project,
    reduced: &ReducedProject,
    package_name: &str,
    table_name: &str,
    retention: DependencyRetention,
    retained_aliases: &mut BTreeSet<String>,
) -> Result<Table, Box<dyn std::error::Error>> {
    let package = project
        .workspace
        .packages
        .get(package_name)
        .ok_or_else(|| format!("unknown package {package_name}"))?;
    let Some(source_dependencies) = package.manifest.get(table_name).and_then(Value::as_table)
    else {
        return Ok(Table::new());
    };

    let mut dependencies = Table::new();
    for (alias, value) in source_dependencies {
        let dependency_package = dependency_package_name(alias, value);
        if is_marker_dependency(alias, &dependency_package) {
            continue;
        }

        if project.workspace.packages.contains_key(&dependency_package) {
            if reduced.packages.contains(&dependency_package) {
                retained_aliases.insert(alias.clone());
                dependencies.insert(
                    alias.clone(),
                    local_dependency_value(alias, &dependency_package, value),
                );
            }
            continue;
        }

        if dependency_should_render(project, reduced, package_name, alias, retention) {
            retained_aliases.insert(alias.clone());
            dependencies.insert(alias.clone(), value.clone());
        }
    }

    Ok(dependencies)
}

fn transformed_target_dependencies(
    project: &Project,
    reduced: &ReducedProject,
    package_name: &str,
    retained_aliases: &mut BTreeSet<String>,
) -> Result<Option<Table>, Box<dyn std::error::Error>> {
    let package = project
        .workspace
        .packages
        .get(package_name)
        .ok_or_else(|| format!("unknown package {package_name}"))?;
    let Some(targets) = package.manifest.get("target").and_then(Value::as_table) else {
        return Ok(None);
    };

    let mut rendered_targets = Table::new();
    for (target_name, target_value) in targets {
        let Some(target_table) = target_value.as_table() else {
            continue;
        };
        let Some(source_dependencies) = target_table.get("dependencies").and_then(Value::as_table)
        else {
            continue;
        };

        let dependencies = transformed_dependency_table(
            project,
            reduced,
            package_name,
            source_dependencies,
            DependencyRetention::SourceMentioned,
            retained_aliases,
        );
        if dependencies.is_empty() {
            continue;
        }

        let mut rendered_target = Table::new();
        rendered_target.insert("dependencies".to_string(), Value::Table(dependencies));
        rendered_targets.insert(target_name.clone(), Value::Table(rendered_target));
    }

    Ok((!rendered_targets.is_empty()).then_some(rendered_targets))
}

fn transformed_dependency_table(
    project: &Project,
    reduced: &ReducedProject,
    package_name: &str,
    source_dependencies: &Table,
    retention: DependencyRetention,
    retained_aliases: &mut BTreeSet<String>,
) -> Table {
    let mut dependencies = Table::new();
    for (alias, value) in source_dependencies {
        let dependency_package = dependency_package_name(alias, value);
        if is_marker_dependency(alias, &dependency_package) {
            continue;
        }

        if project.workspace.packages.contains_key(&dependency_package) {
            if reduced.packages.contains(&dependency_package) {
                retained_aliases.insert(alias.clone());
                dependencies.insert(
                    alias.clone(),
                    local_dependency_value(alias, &dependency_package, value),
                );
            }
            continue;
        }

        if dependency_should_render(project, reduced, package_name, alias, retention) {
            retained_aliases.insert(alias.clone());
            dependencies.insert(alias.clone(), value.clone());
        }
    }

    dependencies
}

fn dependency_should_render(
    project: &Project,
    reduced: &ReducedProject,
    package_name: &str,
    alias: &str,
    retention: DependencyRetention,
) -> bool {
    retention == DependencyRetention::BuildScript
        || package_mentions_dependency(project, reduced, package_name, alias)
}

fn package_mentions_dependency(
    project: &Project,
    reduced: &ReducedProject,
    package_name: &str,
    dependency_alias: &str,
) -> bool {
    let code_name = dependency_code_name(dependency_alias);
    package_rendered_sources(project, reduced, package_name)
        .any(|file| file_mentions_dependency(&file, dependency_alias, &code_name))
}

fn package_rendered_sources_mention_ident(
    project: &Project,
    reduced: &ReducedProject,
    package_name: &str,
    ident: &str,
) -> bool {
    package_rendered_sources(project, reduced, package_name)
        .any(|file| token_stream_mentions_ident(&file.to_token_stream(), ident))
}

fn package_rendered_sources<'a>(
    project: &'a Project,
    reduced: &'a ReducedProject,
    package_name: &'a str,
) -> impl Iterator<Item = syn::File> + 'a {
    project
        .files
        .values()
        .filter(move |source| source.package == package_name)
        .filter(move |source| {
            module_should_render(project, reduced, &source.package, &source.module_path)
        })
        .map(|source| {
            transform_file(
                project,
                reduced,
                &source.package,
                &source.module_path,
                &source.syntax,
            )
        })
}

fn file_mentions_dependency(file: &syn::File, alias: &str, code_name: &str) -> bool {
    let tokens = file.to_token_stream();
    token_stream_mentions_path_root(&tokens, code_name)
        || file_use_tree_starts_with(file, code_name)
        || (alias != code_name
            && (token_stream_mentions_path_root(&tokens, alias)
                || file_use_tree_starts_with(file, alias)))
        || known_macro_dependency_mentions(&tokens, alias, code_name)
}

fn token_stream_mentions_path_root(tokens: &TokenStream, ident: &str) -> bool {
    let token_trees = tokens.clone().into_iter().collect::<Vec<_>>();
    for token in &token_trees {
        if let TokenTree::Group(group) = token {
            if token_stream_mentions_path_root(&group.stream(), ident) {
                return true;
            }
        }
    }

    token_trees.windows(3).any(|window| {
        matches!(&window[0], TokenTree::Ident(candidate) if candidate == ident)
            && matches!(&window[1], TokenTree::Punct(punct) if punct.as_char() == ':')
            && matches!(&window[2], TokenTree::Punct(punct) if punct.as_char() == ':')
    })
}

fn file_use_tree_starts_with(file: &syn::File, ident: &str) -> bool {
    file.items.iter().any(|item| {
        let Item::Use(item_use) = item else {
            return false;
        };
        use_tree_starts_with(&item_use.tree, ident)
    })
}

fn use_tree_starts_with(tree: &UseTree, ident: &str) -> bool {
    match tree {
        UseTree::Path(path) => path.ident == ident || use_tree_starts_with(&path.tree, ident),
        UseTree::Name(name) => name.ident == ident,
        UseTree::Rename(rename) => rename.ident == ident,
        UseTree::Group(group) => group
            .items
            .iter()
            .any(|item| use_tree_starts_with(item, ident)),
        UseTree::Glob(_) => false,
    }
}

fn known_macro_dependency_mentions(tokens: &TokenStream, alias: &str, code_name: &str) -> bool {
    match code_name {
        "serde" | "serde_derive" => {
            token_stream_mentions_ident(tokens, "Serialize")
                || token_stream_mentions_ident(tokens, "Deserialize")
                || token_stream_mentions_ident(tokens, "serde")
        }
        "thiserror" => {
            token_stream_mentions_ident(tokens, "Error")
                && token_stream_mentions_ident(tokens, "error")
        }
        "uniffi" => {
            token_stream_mentions_ident(tokens, "Record")
                || token_stream_mentions_ident(tokens, "Object")
                || token_stream_mentions_ident(tokens, "Enum")
                || token_stream_mentions_ident(tokens, "Error")
                || token_stream_mentions_ident(tokens, "export")
        }
        "clap" => {
            token_stream_mentions_ident(tokens, "Parser")
                || token_stream_mentions_ident(tokens, "Subcommand")
                || token_stream_mentions_ident(tokens, "Args")
                || token_stream_mentions_ident(tokens, "ValueEnum")
                || token_stream_mentions_ident(tokens, "command")
                || token_stream_mentions_ident(tokens, "arg")
        }
        _ => alias != code_name && token_stream_mentions_ident(tokens, alias),
    }
}

fn known_macro_dependency_package_mentions(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    code_name: &str,
) -> bool {
    match code_name {
        "serde" | "serde_derive" => {
            reachable_package_mentions_ident(project, reduced, package, "Serialize")
                || reachable_package_mentions_ident(project, reduced, package, "Deserialize")
                || reachable_package_mentions_ident(project, reduced, package, "serde")
        }
        "thiserror" => {
            reachable_package_mentions_ident(project, reduced, package, "Error")
                && reachable_package_mentions_ident(project, reduced, package, "error")
        }
        "uniffi" => {
            reachable_package_mentions_ident(project, reduced, package, "Record")
                || reachable_package_mentions_ident(project, reduced, package, "Object")
                || reachable_package_mentions_ident(project, reduced, package, "Enum")
                || reachable_package_mentions_ident(project, reduced, package, "Error")
                || reachable_package_mentions_ident(project, reduced, package, "export")
        }
        "clap" => {
            reachable_package_mentions_ident(project, reduced, package, "Parser")
                || reachable_package_mentions_ident(project, reduced, package, "Subcommand")
                || reachable_package_mentions_ident(project, reduced, package, "Args")
                || reachable_package_mentions_ident(project, reduced, package, "ValueEnum")
                || reachable_package_mentions_ident(project, reduced, package, "command")
                || reachable_package_mentions_ident(project, reduced, package, "arg")
        }
        _ => false,
    }
}

fn dependency_code_name(alias: &str) -> String {
    alias.replace('-', "_")
}

fn dependency_name_matches(dependency: &crate::manifest::Dependency, name: &str) -> bool {
    dependency.alias == name
        || dependency.package == name
        || dependency_code_name(&dependency.alias) == name
        || dependency_code_name(&dependency.package) == name
}

fn token_stream_mentions_ident(tokens: &TokenStream, ident: &str) -> bool {
    tokens.clone().into_iter().any(|token| match token {
        TokenTree::Ident(candidate) => candidate == ident,
        TokenTree::Group(group) => token_stream_mentions_ident(&group.stream(), ident),
        TokenTree::Punct(_) | TokenTree::Literal(_) => false,
    })
}

fn dependency_package_name(alias: &str, value: &Value) -> String {
    value
        .as_table()
        .and_then(|table| table.get("package"))
        .and_then(Value::as_str)
        .unwrap_or(alias)
        .to_string()
}

fn dependency_uses_workspace(value: &Value) -> bool {
    value
        .as_table()
        .and_then(|table| table.get("workspace"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn is_marker_dependency(alias: &str, package: &str) -> bool {
    alias == "opensourced" || package == "opensourced"
}

fn package_dependency_tables(package: &Package) -> Vec<(&str, &Table)> {
    let mut tables = Vec::new();
    for table_name in ["dependencies", "build-dependencies"] {
        if let Some(table) = package.manifest.get(table_name).and_then(Value::as_table) {
            tables.push((table_name, table));
        }
    }
    if let Some(targets) = package.manifest.get("target").and_then(Value::as_table) {
        for target in targets.values() {
            let Some(target) = target.as_table() else {
                continue;
            };
            if let Some(table) = target.get("dependencies").and_then(Value::as_table) {
                tables.push(("dependencies", table));
            }
        }
    }
    tables
}

fn transformed_features(package: &Package, retained_aliases: &BTreeSet<String>) -> Option<Value> {
    let features = package.manifest.get("features")?.as_table()?;
    let source_aliases = package_dependency_tables(package)
        .into_iter()
        .flat_map(|(_, table)| table.keys().cloned())
        .collect::<BTreeSet<_>>();

    let mut transformed = Table::new();
    for (feature_name, value) in features {
        let Some(values) = value.as_array() else {
            transformed.insert(feature_name.clone(), value.clone());
            continue;
        };

        let retained_values = values
            .iter()
            .filter_map(|value| {
                let item = value.as_str()?;
                feature_reference_should_remain(item, &source_aliases, retained_aliases)
                    .then(|| Value::String(item.to_string()))
            })
            .collect::<Vec<_>>();
        transformed.insert(feature_name.clone(), Value::Array(retained_values));
    }

    Some(Value::Table(transformed))
}

fn feature_reference_should_remain(
    item: &str,
    source_aliases: &BTreeSet<String>,
    retained_aliases: &BTreeSet<String>,
) -> bool {
    let dependency = item
        .strip_prefix("dep:")
        .or_else(|| item.split_once('/').map(|(dependency, _)| dependency))
        .or_else(|| item.split_once("?/").map(|(dependency, _)| dependency))
        .unwrap_or(item);
    let dependency = dependency.strip_suffix('?').unwrap_or(dependency);

    !source_aliases.contains(dependency) || retained_aliases.contains(dependency)
}

fn local_dependency_value(alias: &str, package: &str, original: &Value) -> Value {
    let mut table = original.as_table().cloned().unwrap_or_default();
    table.remove("version");
    table.remove("workspace");
    table.insert("path".to_string(), Value::String(format!("../{package}")));

    if alias == package {
        table.remove("package");
    } else {
        table.insert("package".to_string(), Value::String(package.to_string()));
    }

    Value::Table(table)
}

fn transform_file(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
    syntax: &syn::File,
) -> syn::File {
    let mut transformed = syntax.clone();
    transformed.items = transform_items(project, reduced, package, module_path, &syntax.items);
    transformed
}

fn transform_items(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
    items: &[Item],
) -> Vec<Item> {
    let retained_macro_definitions =
        retained_macro_definitions_for_generated_items(project, reduced, package, items);

    let mut transformed = Vec::new();
    for item in items {
        let item = match item {
            _ if item_is_test(item) => None,
            Item::Use(item_use) if use_mentions_opensourced(&item_use.tree) => None,
            Item::Use(item_use) => {
                let mut item_use = item_use.clone();
                let tree = prune_use_tree(
                    project,
                    reduced,
                    package,
                    module_path,
                    &item_use.tree,
                    Vec::new(),
                );
                let Some(tree) = tree else {
                    continue;
                };
                item_use.tree = tree;
                Some(Item::Use(item_use))
            }
            Item::Fn(function) => {
                let id = CallableId::Free {
                    package: package.to_string(),
                    module_path: module_path.to_vec(),
                    name: function.sig.ident.to_string(),
                };
                reduced.reachable.contains(&id).then(|| {
                    let mut function = function.clone();
                    strip_opensourced_attrs(&mut function.attrs);
                    Item::Fn(function)
                })
            }
            Item::Macro(item_macro) if is_automod_dir_macro(&item_macro.mac.path) => {
                transformed.extend(automod_reduced_modules(
                    project,
                    reduced,
                    package,
                    module_path,
                    item_macro,
                ));
                continue;
            }
            Item::Macro(item_macro)
                if macro_definition_should_remain(item_macro, &retained_macro_definitions) =>
            {
                Some(Item::Macro(item_macro.clone()))
            }
            Item::Macro(item_macro)
                if should_retain_macro_invocation(project, reduced, package, item_macro) =>
            {
                Some(Item::Macro(item_macro.clone()))
            }
            Item::Struct(_)
            | Item::Enum(_)
            | Item::Union(_)
            | Item::Type(_)
            | Item::Trait(_)
            | Item::Const(_)
            | Item::Static(_)
            | Item::Macro(_) => item_id(package, module_path, item)
                .and_then(|id| reduced.reachable_items.contains(&id).then(|| item.clone())),
            Item::Impl(item_impl) => {
                let Some(type_path) = local_type_path(module_path, &item_impl.self_ty) else {
                    continue;
                };
                let trait_path = item_impl
                    .trait_
                    .as_ref()
                    .map(|(_, path, _)| normalized_path(module_path, path));
                let mut kept_impl_items = Vec::new();
                let mut kept_method = false;

                for impl_item in &item_impl.items {
                    if let ImplItem::Fn(method) = impl_item {
                        if attrs_are_test(&method.attrs) {
                            continue;
                        }
                        let id = CallableId::Method {
                            package: package.to_string(),
                            type_path: type_path.clone(),
                            trait_path: trait_path.clone(),
                            method: method.sig.ident.to_string(),
                        };
                        if reduced.reachable.contains(&id) {
                            let mut method = method.clone();
                            strip_opensourced_attrs(&mut method.attrs);
                            kept_impl_items.push(ImplItem::Fn(method));
                            kept_method = true;
                        }
                    }
                }

                if kept_method {
                    if trait_path.is_some() {
                        kept_impl_items.clear();
                        for impl_item in &item_impl.items {
                            if !impl_item_is_test(impl_item) {
                                kept_impl_items.push(impl_item.clone());
                            }
                        }
                    } else {
                        for impl_item in &item_impl.items {
                            if !matches!(impl_item, ImplItem::Fn(_))
                                && !impl_item_is_test(impl_item)
                            {
                                kept_impl_items.push(impl_item.clone());
                            }
                        }
                    }

                    let mut item_impl = item_impl.clone();
                    item_impl.items = kept_impl_items;
                    Some(Item::Impl(item_impl))
                } else {
                    None
                }
            }
            Item::Mod(item_mod) => {
                let mut item_mod = item_mod.clone();
                let mut child_path = module_path.to_vec();
                child_path.push(item_mod.ident.to_string());

                if let Some((brace, child_items)) = &item_mod.content {
                    let child_items =
                        transform_items(project, reduced, package, &child_path, child_items);
                    if child_items.is_empty() {
                        continue;
                    }
                    item_mod.content = Some((*brace, child_items));
                } else if !module_should_render(project, reduced, package, &child_path) {
                    continue;
                }
                Some(Item::Mod(item_mod))
            }
            _ => Some(item.clone()),
        };

        if let Some(item) = item {
            transformed.push(item);
        }
    }

    transformed
}

fn module_should_render(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
) -> bool {
    if module_path.is_empty() {
        return true;
    }

    reduced.reachable.iter().any(|callable| match callable {
        CallableId::Free {
            package: callable_package,
            module_path: callable_module,
            ..
        } => callable_package == package && path_has_prefix(callable_module, module_path),
        CallableId::Method {
            package: callable_package,
            type_path,
            ..
        } => {
            callable_package == package
                && project
                    .methods
                    .get(callable)
                    .map(|record| path_has_prefix(&record.module_path, module_path))
                    .unwrap_or_else(|| path_has_prefix(type_path, module_path))
        }
    }) || reduced
        .reachable_items
        .iter()
        .any(|item| item.package == package && path_has_prefix(&item.module_path, module_path))
}

fn path_has_prefix(path: &[String], prefix: &[String]) -> bool {
    path.len() >= prefix.len() && path.iter().zip(prefix).all(|(left, right)| left == right)
}

fn item_id(package: &str, module_path: &[String], item: &Item) -> Option<ItemId> {
    let (name, kind) = match item {
        Item::Struct(item) => (item.ident.to_string(), ItemKind::Struct),
        Item::Enum(item) => (item.ident.to_string(), ItemKind::Enum),
        Item::Union(item) => (item.ident.to_string(), ItemKind::Union),
        Item::Type(item) => (item.ident.to_string(), ItemKind::Type),
        Item::Trait(item) => (item.ident.to_string(), ItemKind::Trait),
        Item::Const(item) => (item.ident.to_string(), ItemKind::Const),
        Item::Static(item) => (item.ident.to_string(), ItemKind::Static),
        Item::Macro(item) => (item.ident.as_ref()?.to_string(), ItemKind::Macro),
        _ => return None,
    };

    Some(ItemId {
        package: package.to_string(),
        module_path: module_path.to_vec(),
        name,
        kind,
    })
}

fn automod_reduced_modules(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
    item_macro: &syn::ItemMacro,
) -> Vec<Item> {
    let mut modules = project
        .files
        .values()
        .filter(|source| source.package == package)
        .filter(|source| source.module_path.len() == module_path.len() + 1)
        .filter(|source| path_has_prefix(&source.module_path, module_path))
        .filter(|source| module_should_render(project, reduced, package, &source.module_path))
        .filter_map(|source| source.module_path.last().cloned())
        .collect::<Vec<_>>();
    modules.sort();
    modules.dedup();

    let public = automod_macro_is_public(item_macro);
    modules
        .iter()
        .map(|module| {
            let ident = syn::Ident::new(module, Span::call_site());
            if public {
                parse_quote! {
                    pub mod #ident;
                }
            } else {
                parse_quote! {
                    mod #ident;
                }
            }
        })
        .collect()
}

fn automod_macro_is_public(item_macro: &syn::ItemMacro) -> bool {
    item_macro
        .mac
        .tokens
        .clone()
        .into_iter()
        .any(|token| matches!(token, TokenTree::Ident(ident) if ident == "pub"))
}

fn is_automod_dir_macro(path: &syn::Path) -> bool {
    let mut segments = path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string());
    matches!(
        (segments.next().as_deref(), segments.next().as_deref()),
        (Some("automod"), Some("dir"))
    )
}

fn should_retain_macro_invocation(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    item_macro: &syn::ItemMacro,
) -> bool {
    if item_macro.ident.is_some() {
        return false;
    }

    let is_uniffi = macro_path_starts_with(&item_macro.mac.path, "uniffi");
    let is_uniffi_scaffolding = macro_path_ends_with(&item_macro.mac.path, "setup_scaffolding")
        || macro_path_ends_with(&item_macro.mac.path, "include_scaffolding");
    if is_uniffi_scaffolding {
        return reachable_package_mentions_ident(project, reduced, package, "uniffi");
    }
    if is_uniffi && !reachable_package_mentions_ident(project, reduced, package, "uniffi") {
        return false;
    }

    macro_invocation_feeds_reachable_code(project, reduced, package, item_macro)
}

fn retained_macro_definitions_for_generated_items(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    items: &[Item],
) -> BTreeSet<String> {
    items
        .iter()
        .filter_map(|item| {
            let Item::Macro(item_macro) = item else {
                return None;
            };
            if !should_retain_macro_invocation(project, reduced, package, item_macro) {
                return None;
            }
            item_macro
                .mac
                .path
                .segments
                .last()
                .map(|segment| segment.ident.to_string())
        })
        .collect()
}

fn macro_definition_should_remain(
    item_macro: &syn::ItemMacro,
    retained_macro_definitions: &BTreeSet<String>,
) -> bool {
    item_macro
        .ident
        .as_ref()
        .is_some_and(|ident| retained_macro_definitions.contains(&ident.to_string()))
}

fn macro_invocation_feeds_reachable_code(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    item_macro: &syn::ItemMacro,
) -> bool {
    macro_token_idents(&item_macro.mac.tokens)
        .iter()
        .any(|ident| reachable_package_mentions_ident(project, reduced, package, ident))
}

fn macro_token_idents(tokens: &TokenStream) -> BTreeSet<String> {
    let mut idents = BTreeSet::new();
    collect_macro_token_idents(tokens, &mut idents);
    idents
}

fn collect_macro_token_idents(tokens: &TokenStream, idents: &mut BTreeSet<String>) {
    for token in tokens.clone() {
        match token {
            TokenTree::Ident(ident) => {
                let ident = ident.to_string();
                if macro_generated_reference_candidate(&ident) {
                    idents.insert(ident);
                }
            }
            TokenTree::Group(group) => collect_macro_token_idents(&group.stream(), idents),
            TokenTree::Punct(_) | TokenTree::Literal(_) => {}
        }
    }
}

fn macro_generated_reference_candidate(ident: &str) -> bool {
    !matches!(
        ident,
        "as" | "async"
            | "await"
            | "break"
            | "const"
            | "continue"
            | "crate"
            | "dyn"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "bool"
            | "char"
            | "str"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "usize"
            | "i8"
            | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "isize"
            | "f32"
            | "f64"
            | "String"
            | "Vec"
            | "Result"
            | "Option"
            | "Box"
            | "Ok"
            | "Err"
    )
}

fn macro_path_ends_with(path: &syn::Path, name: &str) -> bool {
    path.segments
        .last()
        .is_some_and(|segment| segment.ident == name)
}

fn macro_path_starts_with(path: &syn::Path, name: &str) -> bool {
    path.segments
        .first()
        .is_some_and(|segment| segment.ident == name)
}

fn reachable_package_mentions_ident(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    ident: &str,
) -> bool {
    reduced
        .reachable
        .iter()
        .filter(|callable| callable.package() == package)
        .any(|callable| {
            if callable_mentions_ident(callable, ident) {
                return true;
            }
            project.functions.get(callable).is_some_and(|record| {
                token_stream_mentions_ident(&record.item.to_token_stream(), ident)
            }) || project.methods.get(callable).is_some_and(|record| {
                token_stream_mentions_ident(&record.item.to_token_stream(), ident)
            })
        })
        || reduced
            .reachable_items
            .iter()
            .filter(|item| item.package() == package)
            .any(|item| {
                project.items.get(item).is_some_and(|record| {
                    token_stream_mentions_ident(&record.item.to_token_stream(), ident)
                })
            })
}

fn callable_mentions_ident(callable: &CallableId, ident: &str) -> bool {
    match callable {
        CallableId::Free {
            module_path, name, ..
        } => name == ident || module_path.iter().any(|segment| segment == ident),
        CallableId::Method {
            type_path,
            trait_path,
            method,
            ..
        } => {
            method == ident
                || type_path.iter().any(|segment| segment == ident)
                || trait_path
                    .as_ref()
                    .is_some_and(|path| path.iter().any(|segment| segment == ident))
        }
    }
}

fn strip_opensourced_attrs(attrs: &mut Vec<syn::Attribute>) {
    attrs.retain(|attribute| !is_opensourced_attr(attribute.path()));
}

fn item_is_test(item: &Item) -> bool {
    match item {
        Item::Const(item) => attrs_are_test(&item.attrs),
        Item::Enum(item) => attrs_are_test(&item.attrs),
        Item::ExternCrate(item) => attrs_are_test(&item.attrs),
        Item::Fn(item) => attrs_are_test(&item.attrs),
        Item::ForeignMod(item) => attrs_are_test(&item.attrs),
        Item::Impl(item) => attrs_are_test(&item.attrs),
        Item::Macro(item) => attrs_are_test(&item.attrs),
        Item::Mod(item) => attrs_are_test(&item.attrs),
        Item::Static(item) => attrs_are_test(&item.attrs),
        Item::Struct(item) => attrs_are_test(&item.attrs),
        Item::Trait(item) => attrs_are_test(&item.attrs),
        Item::TraitAlias(item) => attrs_are_test(&item.attrs),
        Item::Type(item) => attrs_are_test(&item.attrs),
        Item::Union(item) => attrs_are_test(&item.attrs),
        Item::Use(item) => attrs_are_test(&item.attrs),
        Item::Verbatim(_) => false,
        _ => false,
    }
}

fn impl_item_is_test(item: &ImplItem) -> bool {
    match item {
        ImplItem::Const(item) => attrs_are_test(&item.attrs),
        ImplItem::Fn(item) => attrs_are_test(&item.attrs),
        ImplItem::Type(item) => attrs_are_test(&item.attrs),
        ImplItem::Macro(item) => attrs_are_test(&item.attrs),
        ImplItem::Verbatim(_) => false,
        _ => false,
    }
}

fn attrs_are_test(attrs: &[syn::Attribute]) -> bool {
    attrs
        .iter()
        .any(|attribute| is_cfg_test_attr(attribute) || is_test_attr(attribute.path()))
}

fn use_mentions_opensourced(tree: &UseTree) -> bool {
    match tree {
        UseTree::Path(path) => path.ident == "opensourced" || use_mentions_opensourced(&path.tree),
        UseTree::Name(name) => name.ident == "opensourced",
        UseTree::Rename(rename) => rename.ident == "opensourced",
        UseTree::Group(group) => group.items.iter().any(use_mentions_opensourced),
        UseTree::Glob(_) => false,
    }
}

fn prune_use_tree(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
    tree: &UseTree,
    mut prefix: Vec<String>,
) -> Option<UseTree> {
    match tree {
        UseTree::Path(path) => {
            if prefix.is_empty()
                && use_ident_is_pruned_local_dependency(project, reduced, package, &path.ident)
            {
                return None;
            }
            prefix.push(path.ident.to_string());
            let mut path = path.clone();
            path.tree = Box::new(prune_use_tree(
                project,
                reduced,
                package,
                module_path,
                &path.tree,
                prefix,
            )?);
            Some(UseTree::Path(path))
        }
        UseTree::Name(name) => {
            prefix.push(name.ident.to_string());
            (!use_target_should_drop(project, reduced, package, module_path, &prefix))
                .then(|| UseTree::Name(name.clone()))
        }
        UseTree::Rename(rename) => {
            prefix.push(rename.ident.to_string());
            let alias = rename.rename.to_string();
            (!use_target_should_drop(project, reduced, package, module_path, &prefix)
                || reachable_package_mentions_ident(project, reduced, package, &alias))
            .then(|| UseTree::Rename(rename.clone()))
        }
        UseTree::Group(group) => {
            let mut group = group.clone();
            group.items = group
                .items
                .iter()
                .filter_map(|item| {
                    prune_use_tree(project, reduced, package, module_path, item, prefix.clone())
                })
                .collect();
            (!group.items.is_empty()).then_some(UseTree::Group(group))
        }
        UseTree::Glob(glob) => {
            (!use_prefix_should_drop(project, reduced, package, module_path, &prefix))
                .then(|| UseTree::Glob(glob.clone()))
        }
    }
}

fn use_target_should_drop(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
    target: &[String],
) -> bool {
    if target
        .first()
        .is_some_and(|first| use_name_is_pruned_local_dependency(project, reduced, package, first))
    {
        return true;
    }

    if target
        .first()
        .is_some_and(|first| use_name_is_external_dependency(project, package, first))
    {
        return external_use_target_should_drop(project, reduced, package, target);
    }

    if target
        .first()
        .is_some_and(|first| matches!(first.as_str(), "std" | "core" | "alloc"))
    {
        return external_use_target_should_drop(project, reduced, package, target);
    }

    let Some((target_package, target_path)) =
        resolve_use_target_path(project, package, module_path, target)
    else {
        return false;
    };

    if let Some(callable) = find_use_function(project, &target_package, &target_path) {
        return !reduced.reachable.contains(&callable);
    }
    if let Some(item) = find_use_item(project, &target_package, &target_path) {
        return !reduced.reachable_items.contains(&item);
    }
    if let Some((alias_package, alias_path)) =
        resolve_reexported_use_path(project, &target_package, &target_path)
    {
        if let Some(callable) = find_use_function(project, &alias_package, &alias_path) {
            return !reduced.reachable.contains(&callable);
        }
        if let Some(item) = find_use_item(project, &alias_package, &alias_path) {
            return !reduced.reachable_items.contains(&item);
        }
        return project_has_module(project, &alias_package, &alias_path)
            && !module_should_render(project, reduced, &alias_package, &alias_path);
    }

    project_has_module(project, &target_package, &target_path)
        && !module_should_render(project, reduced, &target_package, &target_path)
}

fn use_prefix_should_drop(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
    prefix: &[String],
) -> bool {
    if prefix
        .first()
        .is_some_and(|first| use_name_is_pruned_local_dependency(project, reduced, package, first))
    {
        return true;
    }

    if prefix
        .first()
        .is_some_and(|first| use_name_is_external_dependency(project, package, first))
    {
        return external_use_target_should_drop(project, reduced, package, prefix);
    }

    if prefix
        .first()
        .is_some_and(|first| matches!(first.as_str(), "std" | "core" | "alloc"))
    {
        return external_use_target_should_drop(project, reduced, package, prefix);
    }

    resolve_use_target_path(project, package, module_path, prefix).is_some_and(
        |(target_package, target_path)| {
            project_has_module(project, &target_package, &target_path)
                && !module_should_render(project, reduced, &target_package, &target_path)
        },
    )
}

fn external_use_target_should_drop(
    project: &Project,
    reduced: &ReducedProject,
    _package: &str,
    target: &[String],
) -> bool {
    if target.first().is_some_and(|first| {
        known_macro_dependency_package_mentions(
            project,
            reduced,
            _package,
            &dependency_code_name(first),
        )
    }) {
        return false;
    }

    let Some(leaf) = external_use_leaf(target) else {
        return false;
    };

    !reachable_package_mentions_ident(project, reduced, _package, leaf)
}

fn external_use_leaf(target: &[String]) -> Option<&str> {
    let leaf = target.last()?;
    if leaf == "self" && target.len() >= 2 {
        return target.get(target.len() - 2).map(String::as_str);
    }
    Some(leaf)
}

fn use_ident_is_pruned_local_dependency(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    ident: &syn::Ident,
) -> bool {
    let ident = ident.to_string();
    use_name_is_pruned_local_dependency(project, reduced, package, &ident)
}

fn use_name_is_pruned_local_dependency(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    name: &str,
) -> bool {
    let Some(package_record) = project.workspace.packages.get(package) else {
        return false;
    };
    package_record.dependencies.iter().any(|dependency| {
        dependency_name_matches(dependency, name)
            && project.workspace.packages.contains_key(&dependency.package)
            && !reduced.packages.contains(&dependency.package)
    })
}

fn use_name_is_external_dependency(project: &Project, package: &str, name: &str) -> bool {
    let Some(package_record) = project.workspace.packages.get(package) else {
        return false;
    };
    package_record.dependencies.iter().any(|dependency| {
        dependency_name_matches(dependency, name)
            && !project.workspace.packages.contains_key(&dependency.package)
    })
}

fn resolve_use_target_path(
    project: &Project,
    package: &str,
    module_path: &[String],
    target: &[String],
) -> Option<(String, Vec<String>)> {
    let first = target.first()?;
    match first.as_str() {
        "crate" => Some((package.to_string(), target[1..].to_vec())),
        "self" => {
            let mut path = module_path.to_vec();
            path.extend_from_slice(&target[1..]);
            Some((package.to_string(), path))
        }
        "super" => {
            let mut path = module_path.to_vec();
            path.pop();
            path.extend_from_slice(&target[1..]);
            Some((package.to_string(), path))
        }
        name if name == package => Some((package.to_string(), target[1..].to_vec())),
        name => {
            if let Some(package_record) = project.workspace.packages.get(package) {
                if let Some(dependency) = package_record
                    .dependencies
                    .iter()
                    .find(|dependency| dependency_name_matches(dependency, name))
                {
                    if project.workspace.packages.contains_key(&dependency.package) {
                        return Some((dependency.package.clone(), target[1..].to_vec()));
                    }
                    return None;
                }
            }

            let mut path = module_path.to_vec();
            path.extend_from_slice(target);
            Some((package.to_string(), path))
        }
    }
}

fn resolve_reexported_use_path(
    project: &Project,
    package: &str,
    path: &[String],
) -> Option<(String, Vec<String>)> {
    let name = path.last()?;
    let module_path = &path[..path.len() - 1];
    let aliases = project
        .module_aliases
        .get(&(package.to_string(), module_path.to_vec()))?;
    let target = aliases.get(name)?;
    resolve_use_target_path(project, package, module_path, target)
}

fn find_use_function(project: &Project, package: &str, path: &[String]) -> Option<CallableId> {
    let name = path.last()?.clone();
    let module_path = path[..path.len() - 1].to_vec();
    let id = CallableId::Free {
        package: package.to_string(),
        module_path,
        name,
    };
    project.functions.contains_key(&id).then_some(id)
}

fn find_use_item(project: &Project, package: &str, path: &[String]) -> Option<ItemId> {
    let name = path.last()?.clone();
    let module_path = path[..path.len() - 1].to_vec();
    [
        ItemKind::Struct,
        ItemKind::Enum,
        ItemKind::Union,
        ItemKind::Type,
        ItemKind::Trait,
        ItemKind::Const,
        ItemKind::Static,
        ItemKind::Macro,
    ]
    .into_iter()
    .find_map(|kind| {
        let id = ItemId {
            package: package.to_string(),
            module_path: module_path.clone(),
            name: name.clone(),
            kind,
        };
        project.items.contains_key(&id).then_some(id)
    })
}

fn project_has_module(project: &Project, package: &str, module_path: &[String]) -> bool {
    module_path.is_empty()
        || project
            .files
            .values()
            .any(|source| source.package == package && source.module_path == module_path)
}

fn local_type_path(module_path: &[String], self_ty: &Type) -> Option<Vec<String>> {
    let Type::Path(type_path) = self_ty else {
        return None;
    };
    let segments = type_path
        .path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>();
    if segments.is_empty() {
        return None;
    }
    if segments[0] == "crate" {
        return Some(segments[1..].to_vec());
    }
    if segments[0] == "self" {
        let mut path = module_path.to_vec();
        path.extend_from_slice(&segments[1..]);
        return Some(path);
    }
    if segments[0] == "super" {
        let mut path = module_path.to_vec();
        path.pop();
        path.extend_from_slice(&segments[1..]);
        return Some(path);
    }

    let mut path = module_path.to_vec();
    path.extend(segments);
    Some(path)
}

fn normalized_path(module_path: &[String], path: &syn::Path) -> Vec<String> {
    let segments = path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>();
    if segments.is_empty() {
        return Vec::new();
    }
    if segments[0] == "crate" {
        return segments[1..].to_vec();
    }
    if segments[0] == "self" {
        let mut path = module_path.to_vec();
        path.extend_from_slice(&segments[1..]);
        return path;
    }
    if segments[0] == "super" {
        let mut path = module_path.to_vec();
        path.pop();
        path.extend_from_slice(&segments[1..]);
        return path;
    }
    segments
}
