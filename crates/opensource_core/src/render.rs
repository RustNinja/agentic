use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, HashMap},
    env, fs,
    path::{Component, Path, PathBuf},
};

use proc_macro2::{TokenStream, TokenTree};
use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::visit::{self, Visit};
use syn::{
    parse_quote, Field, GenericArgument, ImplItem, Item, PathArguments, TraitItem, Type, UseTree,
    Variant,
};
use toml::{value::Table, Value};

use crate::{
    manifest::Package,
    model::{CallableId, ItemId, ItemKind, Project, ReducedProject, RootId, SourceFile},
    reduce::{is_cfg_test_attr, is_opensourced_attr, is_test_attr},
};

const OUTPUT_MARKER: &str = ".slicers-output";

struct RenderPlan {
    reachable_items: BTreeSet<ItemId>,
    callable_idents_by_package: BTreeMap<String, BTreeSet<String>>,
    mentions: ReachableMentionIndex,
    import_scope_mentions: RefCell<BTreeMap<ImportScopeMentionKey, bool>>,
}

impl RenderPlan {
    fn build(project: &Project, reduced: &ReducedProject) -> Self {
        let mut reachable_items = BTreeSet::new();
        let mut rendered_item_idents = BTreeSet::new();
        let callable_idents = reachable_reduced_callable_ident_index(project, reduced);
        let retained_surface_idents = retained_surface_idents_by_package(project, reduced);

        for item in &reduced.reachable_items {
            if root_item_should_render(reduced, item)
                || !package_has_reachable_callables(reduced, &item.package)
                || matches!(item.kind, ItemKind::Const | ItemKind::Static)
                || callable_idents.all.contains(&item.name)
                || retained_surface_idents
                    .get(&item.package)
                    .is_some_and(|idents| idents.contains(&item.name))
            {
                insert_render_plan_item(
                    project,
                    reduced,
                    &mut reachable_items,
                    &mut rendered_item_idents,
                    item,
                );
            }
        }

        loop {
            let mut added = false;
            for item in &reduced.reachable_items {
                if reachable_items.contains(item) {
                    continue;
                }
                if rendered_item_idents.contains(&item.name) {
                    insert_render_plan_item(
                        project,
                        reduced,
                        &mut reachable_items,
                        &mut rendered_item_idents,
                        item,
                    );
                    added = true;
                }
            }
            if !added {
                break;
            }
        }

        Self {
            reachable_items,
            callable_idents_by_package: callable_idents.by_package,
            mentions: ReachableMentionIndex::build(project, reduced),
            import_scope_mentions: RefCell::new(BTreeMap::new()),
        }
    }

    fn item_should_render(&self, item: &ItemId) -> bool {
        self.reachable_items.contains(item)
    }

    fn package_mentions_ident(&self, package: &str, ident: &str) -> bool {
        self.mentions.package_mentions_ident(package, ident)
    }

    fn module_mentions_ident(&self, package: &str, module_path: &[String], ident: &str) -> bool {
        self.mentions
            .module_mentions_ident(package, module_path, ident)
    }

    fn package_callable_mentions_ident(&self, package: &str, ident: &str) -> bool {
        self.callable_idents_by_package
            .get(package)
            .is_some_and(|idents| idents.contains(ident))
    }
}

#[derive(Default)]
struct CallableMentionIndex {
    all: BTreeSet<String>,
    by_package: BTreeMap<String, BTreeSet<String>>,
}

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
struct ImportScopeMentionKey {
    package: String,
    module_path: Vec<String>,
    ident: String,
    excluded_module_path: Option<Vec<String>>,
}

impl ImportScopeMentionKey {
    fn new(
        package: &str,
        module_path: &[String],
        ident: &str,
        excluded_module_path: Option<&[String]>,
    ) -> Self {
        Self {
            package: package.to_string(),
            module_path: module_path.to_vec(),
            ident: ident.to_string(),
            excluded_module_path: excluded_module_path.map(<[String]>::to_vec),
        }
    }
}

#[derive(Default)]
struct ReachableMentionIndex {
    packages: BTreeMap<String, BTreeSet<String>>,
    modules: BTreeMap<ModuleMentionKey, BTreeSet<String>>,
}

impl ReachableMentionIndex {
    fn build(project: &Project, reduced: &ReducedProject) -> Self {
        let mut index = Self::default();
        for callable in &reduced.reachable {
            let package = callable.package();
            let mut idents = BTreeSet::new();
            collect_callable_idents(callable, &mut idents);
            let module_path = if let Some(record) = project.functions.get(callable) {
                collect_token_idents(&record.item.to_token_stream(), &mut idents);
                record.module_path.as_slice()
            } else if let Some(record) = project.methods.get(callable) {
                collect_token_idents(&record.item.to_token_stream(), &mut idents);
                record.module_path.as_slice()
            } else {
                callable_module_path(callable)
            };
            index.add_package_idents(package, idents.iter().cloned());
            index.add_module_idents(package, module_path, idents);
        }

        for item in &reduced.reachable_items {
            let idents = rendered_item_surface_idents(project, reduced, item);
            index.add_package_idents(&item.package, idents.iter().cloned());
            index.add_module_idents(&item.package, &item.module_path, idents);
        }

        for source in project
            .files
            .values()
            .filter(|source| reduced.packages.contains(&source.package))
        {
            let mut idents = BTreeSet::new();
            collect_retained_module_surface_idents(project, reduced, source, &mut idents);
            index.add_module_idents(&source.package, &source.module_path, idents);
        }

        index
    }

    fn add_package_idents<I>(&mut self, package: &str, idents: I)
    where
        I: IntoIterator<Item = String>,
    {
        self.packages
            .entry(package.to_string())
            .or_default()
            .extend(idents);
    }

    fn add_module_idents<I>(&mut self, package: &str, module_path: &[String], idents: I)
    where
        I: IntoIterator<Item = String>,
    {
        self.modules
            .entry(ModuleMentionKey::new(package, module_path))
            .or_default()
            .extend(idents);
    }

    fn package_mentions_ident(&self, package: &str, ident: &str) -> bool {
        self.packages
            .get(package)
            .is_some_and(|idents| idents.contains(ident))
    }

    fn module_mentions_ident(&self, package: &str, module_path: &[String], ident: &str) -> bool {
        self.modules
            .get(&ModuleMentionKey::new(package, module_path))
            .is_some_and(|idents| idents.contains(ident))
    }
}

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
struct ModuleMentionKey {
    package: String,
    module_path: Vec<String>,
}

impl ModuleMentionKey {
    fn new(package: &str, module_path: &[String]) -> Self {
        Self {
            package: package.to_string(),
            module_path: module_path.to_vec(),
        }
    }
}

fn callable_module_path(callable: &CallableId) -> &[String] {
    match callable {
        CallableId::Free { module_path, .. } => module_path,
        CallableId::Method { type_path, .. } => type_path
            .split_last()
            .map_or(type_path.as_slice(), |(_, module_path)| module_path),
    }
}

fn insert_render_plan_item(
    project: &Project,
    reduced: &ReducedProject,
    reachable_items: &mut BTreeSet<ItemId>,
    rendered_item_idents: &mut BTreeSet<String>,
    item: &ItemId,
) {
    if !reachable_items.insert(item.clone()) {
        return;
    }
    rendered_item_idents.extend(rendered_item_surface_idents(project, reduced, item));
}

pub fn write_reduced_workspace(
    project: &Project,
    reduced: &ReducedProject,
    output_root: &Path,
) -> Result<usize, Box<dyn std::error::Error>> {
    prepare_output_root(&project.workspace.root, output_root)?;
    write_output_marker(output_root)?;

    let render_plan = RenderPlan::build(project, reduced);
    let package_usages = package_source_usages(project, reduced, &render_plan);

    write_workspace_manifest(project, reduced, output_root, &package_usages)?;

    let mut files_written = 1
        + copy_workspace_lockfile(project, output_root)?
        + copy_workspace_cargo_config(project, output_root)?;
    for package_name in &reduced.packages {
        let package = project
            .workspace
            .packages
            .get(package_name)
            .ok_or_else(|| format!("unknown package {package_name}"))?;
        let package_usage = package_usages
            .get(package_name)
            .ok_or_else(|| format!("missing source usage for package {package_name}"))?;

        let package_output = output_root.join(package_name);
        fs::create_dir_all(package_output.join("src"))?;
        write_package_manifest(
            project,
            reduced,
            package_name,
            &package_output.join("Cargo.toml"),
            package_usage,
        )?;
        files_written += 1;

        if let Some(build_script) = build_script_to_render(package, package_usage) {
            let relative_path = build_script.strip_prefix(&package.root)?;
            let output_path = package_output.join(relative_path);
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&build_script, output_path)?;
            files_written += 1;
            files_written += copy_build_script_assets(package, &build_script, &package_output)?;
        }

        let copied_support_source_tree = if package_should_copy_library_support_source(package) {
            files_written += copy_library_support_source_tree(package, &package_output)?;
            true
        } else {
            false
        };

        if package_should_preserve_source_tree(project, reduced, package) {
            if !copied_support_source_tree {
                files_written += copy_source_tree(package, &package_output)?;
            }
            continue;
        }

        for source in project
            .files
            .values()
            .filter(|source| &source.package == package_name)
        {
            if !module_should_render(
                project,
                reduced,
                &render_plan,
                &source.package,
                &source.module_path,
            ) {
                continue;
            }
            let mut transformed = transform_file(
                project,
                reduced,
                &render_plan,
                &source.package,
                &source.module_path,
                &source.syntax,
                package_target_preserves_test_items(package),
            );
            if is_main_like_entry_source(package, source) {
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
            files_written +=
                copy_source_include_assets(package, source, &transformed, &package_output)?;
        }
    }

    Ok(files_written)
}

fn prepare_output_root(
    workspace_root: &Path,
    output_root: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let workspace_root = workspace_root.canonicalize()?;
    let output_root_abs = absolute_normalized_path(output_root)?;

    if output_root_abs == workspace_root || output_root_abs.starts_with(&workspace_root) {
        return Err(format!(
            "output root {} must not be inside input workspace {}",
            output_root_abs.display(),
            workspace_root.display()
        )
        .into());
    }
    if workspace_root.starts_with(&output_root_abs) {
        return Err(format!(
            "output root {} must not contain input workspace {}",
            output_root_abs.display(),
            workspace_root.display()
        )
        .into());
    }

    if output_root.exists() {
        if fs::symlink_metadata(output_root)?.file_type().is_symlink() {
            return Err(format!(
                "refusing to use symlink output root {}",
                output_root.display()
            )
            .into());
        }
        if !fs::metadata(output_root)?.is_dir() {
            return Err(format!("output root {} is not a directory", output_root.display()).into());
        }
        if !directory_is_empty(output_root)? && !output_root.join(OUTPUT_MARKER).is_file() {
            return Err(format!(
                "refusing to overwrite non-empty output root {} because it was not created by slicers",
                output_root.display()
            )
            .into());
        }
        fs::remove_dir_all(output_root)?;
    }
    fs::create_dir_all(output_root)?;
    Ok(())
}

fn write_output_marker(output_root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(
        output_root.join(OUTPUT_MARKER),
        "generated by slicers; safe to replace on the next slicers run\n",
    )?;
    Ok(())
}

fn directory_is_empty(path: &Path) -> Result<bool, Box<dyn std::error::Error>> {
    Ok(fs::read_dir(path)?.next().transpose()?.is_none())
}

fn absolute_normalized_path(path: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()?.join(path)
    };
    let mut cursor = absolute.as_path();
    let mut missing_components = Vec::new();

    while !cursor.exists() {
        let Some(file_name) = cursor.file_name() else {
            break;
        };
        missing_components.push(PathBuf::from(file_name));
        cursor = cursor
            .parent()
            .ok_or_else(|| format!("output root {} has no existing parent", path.display()))?;
    }

    let mut resolved = cursor.canonicalize()?;
    for component in missing_components.iter().rev() {
        resolved.push(component);
    }
    Ok(normalize_path(&resolved))
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(segment) => normalized.push(segment),
        }
    }
    normalized
}

fn copy_workspace_lockfile(
    project: &Project,
    output_root: &Path,
) -> Result<usize, Box<dyn std::error::Error>> {
    let lockfile = project.workspace.root.join("Cargo.lock");
    if !lockfile.exists() {
        return Ok(0);
    }

    fs::copy(lockfile, output_root.join("Cargo.lock"))?;
    Ok(1)
}

fn copy_workspace_cargo_config(
    project: &Project,
    output_root: &Path,
) -> Result<usize, Box<dyn std::error::Error>> {
    let source_dir = project.workspace.root.join(".cargo");
    let mut copied = 0;
    for file_name in ["config.toml", "config"] {
        let source = source_dir.join(file_name);
        if !source.is_file() {
            continue;
        }
        let output = output_root.join(".cargo").join(file_name);
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(source, output)?;
        copied += 1;
    }
    Ok(copied)
}

fn package_should_preserve_source_tree(
    project: &Project,
    reduced: &ReducedProject,
    package: &Package,
) -> bool {
    let has_reachable_rust = reduced
        .reachable
        .iter()
        .any(|callable| callable.package() == package.name)
        || reduced
            .reachable_items
            .iter()
            .any(|item| item.package == package.name);
    !has_reachable_rust
        && package_sources_mention_include_macro(project, &package.name)
        && package_has_unparsed_source_files(project, package)
}

fn package_sources_mention_include_macro(project: &Project, package: &str) -> bool {
    project
        .files
        .values()
        .filter(|source| source.package == package)
        .any(|source| token_stream_mentions_ident(&source.syntax.to_token_stream(), "include"))
}

fn package_has_unparsed_source_files(project: &Project, package: &Package) -> bool {
    let parsed = project
        .files
        .values()
        .filter(|source| source.package == package.name)
        .map(|source| source.path.clone())
        .collect::<BTreeSet<_>>();
    package.root.join("src").exists()
        && source_tree_rs_files(&package.root.join("src"))
            .is_ok_and(|files| files.into_iter().any(|file| !parsed.contains(&file)))
}

fn root_item_should_render(reduced: &ReducedProject, item: &ItemId) -> bool {
    reduced
        .roots
        .iter()
        .any(|root| matches!(root, RootId::Item(root_item) if root_item == item))
}

fn package_has_reachable_callables(reduced: &ReducedProject, package: &str) -> bool {
    reduced
        .reachable
        .iter()
        .any(|callable| callable.package() == package)
}

fn rendered_item_surface_idents(
    project: &Project,
    reduced: &ReducedProject,
    item: &ItemId,
) -> BTreeSet<String> {
    let mut idents = BTreeSet::new();
    let Some(record) = project.items.get(item) else {
        return idents;
    };
    match &record.item {
        Item::Struct(item_struct) => {
            collect_struct_surface_idents(project, reduced, item, item_struct, &mut idents);
        }
        Item::Trait(item_trait) => {
            collect_trait_surface_idents(project, reduced, item, item_trait, &mut idents);
        }
        _ => collect_token_idents(&record.item.to_token_stream(), &mut idents),
    }
    idents
}

fn collect_struct_surface_idents(
    project: &Project,
    reduced: &ReducedProject,
    item_id: &ItemId,
    item_struct: &syn::ItemStruct,
    idents: &mut BTreeSet<String>,
) {
    idents.insert(item_id.name.clone());
    idents.extend(item_id.module_path.iter().cloned());
    for attr in &item_struct.attrs {
        collect_token_idents(&attr.to_token_stream(), idents);
    }
    if root_item_should_render(reduced, item_id) {
        collect_token_idents(&item_struct.to_token_stream(), idents);
        return;
    }
    let syn::Fields::Named(fields) = &item_struct.fields else {
        collect_token_idents(&item_struct.to_token_stream(), idents);
        return;
    };
    for field in &fields.named {
        if struct_field_should_remain(project, reduced, None, &item_id.package, item_struct, field)
        {
            collect_token_idents(&field.to_token_stream(), idents);
        }
    }
}

fn collect_trait_surface_idents(
    project: &Project,
    reduced: &ReducedProject,
    item_id: &ItemId,
    item_trait: &syn::ItemTrait,
    idents: &mut BTreeSet<String>,
) {
    if trait_has_reachable_impl_methods(
        reduced,
        &item_id.package,
        &item_id.module_path,
        &item_id.name,
    ) {
        collect_token_idents(&item_trait.to_token_stream(), idents);
        return;
    }
    if trait_items_are_referenced_by_reachable_surfaces(
        project,
        reduced,
        &item_id.package,
        item_trait,
    ) {
        collect_token_idents(&item_trait.to_token_stream(), idents);
        return;
    }
    idents.insert(item_id.name.clone());
    idents.extend(item_id.module_path.iter().cloned());
    for bound in &item_trait.supertraits {
        collect_token_idents(&bound.to_token_stream(), idents);
    }
    for attr in item_trait
        .attrs
        .iter()
        .filter(|attr| is_inert_type_surface_attr(attr))
    {
        collect_token_idents(&attr.to_token_stream(), idents);
    }
}

fn reachable_reduced_callable_ident_index(
    project: &Project,
    reduced: &ReducedProject,
) -> CallableMentionIndex {
    let mut index = CallableMentionIndex::default();
    for callable in &reduced.reachable {
        let package = callable.package().to_string();
        let mut idents = BTreeSet::new();
        collect_callable_idents(callable, &mut idents);
        if let Some(record) = project.functions.get(callable) {
            collect_token_idents(&record.item.to_token_stream(), &mut idents);
        }
        if let Some(record) = project.methods.get(callable) {
            collect_token_idents(&record.item.to_token_stream(), &mut idents);
        }
        index.all.extend(idents.iter().cloned());
        index.by_package.entry(package).or_default().extend(idents);
    }
    index
}

fn collect_callable_idents(callable: &CallableId, idents: &mut BTreeSet<String>) {
    match callable {
        CallableId::Free {
            module_path, name, ..
        } => {
            idents.extend(module_path.iter().cloned());
            idents.insert(name.clone());
        }
        CallableId::Method {
            type_path,
            trait_path,
            method,
            ..
        } => {
            idents.extend(type_path.iter().cloned());
            if let Some(trait_path) = trait_path {
                idents.extend(trait_path.iter().cloned());
            }
            idents.insert(method.clone());
        }
    }
}

fn collect_token_idents(tokens: &TokenStream, idents: &mut BTreeSet<String>) {
    for token in tokens.clone() {
        match token {
            TokenTree::Ident(ident) => {
                idents.insert(ident.to_string());
            }
            TokenTree::Group(group) => collect_token_idents(&group.stream(), idents),
            TokenTree::Punct(_) | TokenTree::Literal(_) => {}
        }
    }
}

fn reachable_reduced_callables_mention_ident(
    project: &Project,
    reduced: &ReducedProject,
    ident: &str,
) -> bool {
    reduced.reachable.iter().any(|callable| {
        if callable_mentions_ident(callable, ident) {
            return true;
        }
        project.functions.get(callable).is_some_and(|record| {
            token_stream_mentions_ident(&record.item.to_token_stream(), ident)
        }) || project.methods.get(callable).is_some_and(|record| {
            token_stream_mentions_ident(&record.item.to_token_stream(), ident)
        })
    })
}

fn retained_impl_surfaces_mention_ident(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    ident: &str,
) -> bool {
    project
        .files
        .values()
        .filter(|source| source.package == package)
        .any(|source| {
            retained_impl_attrs_mention_ident(project, reduced, package, &source.module_path, ident)
                || retained_impl_non_fn_items_mention_ident(
                    project,
                    reduced,
                    package,
                    &source.module_path,
                    ident,
                )
                || retained_impl_items_mention_ident(
                    project,
                    reduced,
                    package,
                    &source.module_path,
                    ident,
                )
        })
}

fn retained_surface_idents_by_package(
    project: &Project,
    reduced: &ReducedProject,
) -> BTreeMap<String, BTreeSet<String>> {
    let mut packages = BTreeMap::<String, BTreeSet<String>>::new();
    for source in project
        .files
        .values()
        .filter(|source| reduced.packages.contains(&source.package))
    {
        let mut idents = BTreeSet::new();
        collect_retained_module_surface_idents(project, reduced, source, &mut idents);
        packages
            .entry(source.package.clone())
            .or_default()
            .extend(idents);
    }
    packages
}

fn source_tree_rs_files(path: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    if !path.exists() {
        return Ok(files);
    }
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            files.extend(source_tree_rs_files(&path)?);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path.canonicalize()?);
        }
    }
    Ok(files)
}

fn copy_source_tree(
    package: &Package,
    package_output: &Path,
) -> Result<usize, Box<dyn std::error::Error>> {
    copy_source_tree_path(package, &package.root.join("src"), package_output)
}

fn copy_library_support_source_tree(
    package: &Package,
    package_output: &Path,
) -> Result<usize, Box<dyn std::error::Error>> {
    let Some(lib_path) = package_library_source_path(package) else {
        return Ok(0);
    };
    let src_dir = package.root.join("src");
    if lib_path.starts_with(&src_dir) {
        return copy_source_tree_path(package, &src_dir, package_output);
    }
    let source_root = lib_path.parent().unwrap_or(&lib_path);
    copy_source_tree_path(package, source_root, package_output)
}

fn copy_source_tree_path(
    package: &Package,
    path: &Path,
    package_output: &Path,
) -> Result<usize, Box<dyn std::error::Error>> {
    if !path.exists() {
        return Ok(0);
    }
    if path.is_file() {
        let relative_path = path.strip_prefix(&package.root)?;
        let output_path = package_output.join(relative_path);
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(path, output_path)?;
        return Ok(1);
    }

    let mut copied = 0;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        copied += copy_source_tree_path(package, &entry.path(), package_output)?;
    }
    Ok(copied)
}

fn package_should_copy_library_support_source(package: &Package) -> bool {
    package_target_uses_dev_dependencies(package) && package_library_source_path(package).is_some()
}

fn package_library_source_path(package: &Package) -> Option<PathBuf> {
    if package
        .entry_target
        .kind
        .iter()
        .any(|kind| matches!(kind.as_str(), "lib" | "proc-macro"))
    {
        return None;
    }
    if let Some(path) = package
        .manifest
        .get("lib")
        .and_then(|lib| lib.get("path"))
        .and_then(Value::as_str)
    {
        let path = package.root.join(path);
        return path.exists().then_some(path);
    }
    let path = package.root.join("src/lib.rs");
    path.exists().then_some(path)
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
    package: &Package,
    package_usage: &PackageSourceUsage,
) -> Option<PathBuf> {
    build_script_should_render_with_usage(package, package_usage)
        .then(|| build_script_path(package))?
}

fn build_script_should_render_with_usage(
    package: &Package,
    package_usage: &PackageSourceUsage,
) -> bool {
    let Some(build_script) = build_script_path(package) else {
        return false;
    };
    if build_script_is_uniffi_only(&build_script) && !package_usage.mentions_ident("uniffi") {
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

fn is_main_like_entry_source(package: &Package, source: &SourceFile) -> bool {
    if !source.module_path.is_empty() {
        return false;
    }
    if package
        .entry_target
        .kind
        .iter()
        .any(|kind| matches!(kind.as_str(), "bin" | "example"))
        && source.path.canonicalize().is_ok_and(|path| {
            package
                .lib_path
                .canonicalize()
                .is_ok_and(|entry| path == entry)
        })
    {
        return true;
    }
    if source
        .path
        .file_name()
        .is_some_and(|file_name| file_name == "main.rs")
    {
        return true;
    }
    source
        .path
        .parent()
        .and_then(Path::file_name)
        .is_some_and(|file_name| file_name == "bin")
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

fn copy_source_include_assets(
    package: &Package,
    source: &SourceFile,
    syntax: &syn::File,
    package_output: &Path,
) -> Result<usize, Box<dyn std::error::Error>> {
    let Some(source_dir) = source.path.parent() else {
        return Ok(0);
    };

    let mut candidates = BTreeSet::new();
    collect_include_macro_paths(&syntax.to_token_stream(), &mut candidates);

    let mut copied = 0;
    for candidate in candidates {
        let path = if candidate.is_absolute() {
            candidate
        } else {
            source_dir.join(candidate)
        };
        let Ok(path) = path.canonicalize() else {
            continue;
        };
        if !path.starts_with(&package.root) || !path.is_file() {
            continue;
        }
        copy_asset(package, &path, package_output)?;
        copied += 1;
    }

    Ok(copied)
}

fn collect_include_macro_paths(tokens: &TokenStream, candidates: &mut BTreeSet<PathBuf>) {
    let mut tokens = tokens.clone().into_iter().peekable();
    while let Some(token) = tokens.next() {
        match token {
            TokenTree::Ident(ident) if is_file_include_macro(&ident.to_string()) => {
                let Some(TokenTree::Punct(punct)) = tokens.next() else {
                    continue;
                };
                if punct.as_char() != '!' {
                    continue;
                }
                let Some(TokenTree::Group(group)) = tokens.next() else {
                    continue;
                };
                if let Some(path) = include_macro_literal_path(&group.stream()) {
                    candidates.insert(path);
                }
            }
            TokenTree::Group(group) => collect_include_macro_paths(&group.stream(), candidates),
            TokenTree::Ident(_) | TokenTree::Punct(_) | TokenTree::Literal(_) => {}
        }
    }
}

fn is_file_include_macro(ident: &str) -> bool {
    matches!(ident, "include" | "include_str" | "include_bytes")
}

fn include_macro_literal_path(tokens: &TokenStream) -> Option<PathBuf> {
    for token in tokens.clone() {
        match token {
            TokenTree::Literal(literal) => {
                let Ok(literal) = syn::parse2::<syn::LitStr>(literal.to_token_stream()) else {
                    continue;
                };
                let value = literal.value();
                if value.is_empty() || value.contains('\0') {
                    return None;
                }
                return Some(PathBuf::from(value));
            }
            TokenTree::Group(group) => {
                if let Some(path) = include_macro_literal_path(&group.stream()) {
                    return Some(path);
                }
            }
            TokenTree::Ident(_) | TokenTree::Punct(_) => {}
        }
    }
    None
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
    path.extension().is_none_or(|extension| extension != "rs")
}

fn write_workspace_manifest(
    project: &Project,
    reduced: &ReducedProject,
    output_root: &Path,
    package_usages: &HashMap<String, PackageSourceUsage>,
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

    let workspace_dependencies = retained_workspace_dependencies(project, reduced, package_usages);
    if !workspace_dependencies.is_empty() {
        workspace.insert(
            "dependencies".to_string(),
            Value::Table(workspace_dependencies),
        );
    }

    root.insert("workspace".to_string(), Value::Table(workspace));
    if let Some(patch) = transformed_patch_tables(project) {
        root.insert("patch".to_string(), patch);
    }
    fs::write(
        output_root.join("Cargo.toml"),
        toml::to_string_pretty(&Value::Table(root))?,
    )?;
    Ok(())
}

fn transformed_patch_tables(project: &Project) -> Option<Value> {
    let source_patches = project.workspace.manifest.get("patch")?.as_table()?;
    let mut patches = Table::new();
    for (source, value) in source_patches {
        let Some(source_table) = value.as_table() else {
            patches.insert(source.clone(), value.clone());
            continue;
        };

        let mut transformed = Table::new();
        for (name, dependency) in source_table {
            transformed.insert(
                name.clone(),
                dependency_value_with_resolved_path(dependency, &project.workspace.root),
            );
        }
        patches.insert(source.clone(), Value::Table(transformed));
    }

    (!patches.is_empty()).then_some(Value::Table(patches))
}

fn write_package_manifest(
    project: &Project,
    reduced: &ReducedProject,
    package_name: &str,
    output_path: &Path,
    package_usage: &PackageSourceUsage,
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

    if let Some(value) = package.manifest.get("lib") {
        manifest.insert(
            "lib".to_string(),
            transformed_lib_table(project, reduced, package, value),
        );
    }

    for target_table in ["bin", "example", "test", "bench"] {
        if let Some(value) = transformed_named_targets(package, target_table)? {
            manifest.insert(target_table.to_string(), value);
        }
    }

    let requested_features = requested_local_features(project, reduced, package_name);
    let feature_required_aliases =
        dependency_aliases_required_by_features(package, &requested_features);
    let mut retained_dependency_aliases = BTreeSet::new();
    let dependencies = transformed_dependencies(
        project,
        reduced,
        package_name,
        "dependencies",
        DependencyRetention::SourceMentioned,
        DependencyUsageScope::General,
        package_usage,
        &feature_required_aliases,
        &mut retained_dependency_aliases,
    )?;
    if !dependencies.is_empty() {
        manifest.insert("dependencies".to_string(), Value::Table(dependencies));
    }

    if package_target_uses_dev_dependencies(package) {
        let dev_dependencies = transformed_dependencies(
            project,
            reduced,
            package_name,
            "dev-dependencies",
            DependencyRetention::SourceMentioned,
            DependencyUsageScope::General,
            package_usage,
            &feature_required_aliases,
            &mut retained_dependency_aliases,
        )?;
        if !dev_dependencies.is_empty() {
            manifest.insert(
                "dev-dependencies".to_string(),
                Value::Table(dev_dependencies),
            );
        }
    }

    if build_script_should_render_with_usage(package, package_usage) {
        let build_dependencies = transformed_dependencies(
            project,
            reduced,
            package_name,
            "build-dependencies",
            DependencyRetention::BuildScript,
            DependencyUsageScope::Any,
            package_usage,
            &feature_required_aliases,
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
        package_usage,
        &feature_required_aliases,
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

fn package_target_uses_dev_dependencies(package: &Package) -> bool {
    package
        .entry_target
        .kind
        .iter()
        .any(|kind| matches!(kind.as_str(), "example" | "test" | "bench"))
}

fn package_target_preserves_test_items(package: &Package) -> bool {
    package
        .entry_target
        .kind
        .iter()
        .any(|kind| matches!(kind.as_str(), "test" | "bench"))
}

fn transformed_named_targets(
    package: &Package,
    target_table: &str,
) -> Result<Option<Value>, Box<dyn std::error::Error>> {
    let Some(value) = package.manifest.get(target_table) else {
        return Ok(None);
    };
    let Some(targets) = value.as_array() else {
        return Ok(Some(value.clone()));
    };

    let entry_source = package.lib_path.canonicalize()?;
    let retained = targets
        .iter()
        .filter(|target| {
            named_target_matches_entry_source(package, target_table, target, &entry_source)
        })
        .cloned()
        .collect::<Vec<_>>();

    Ok((!retained.is_empty()).then_some(Value::Array(retained)))
}

fn transformed_lib_table(
    project: &Project,
    reduced: &ReducedProject,
    package: &Package,
    value: &Value,
) -> Value {
    let mut value = value.clone();
    if !package_depends_on(package, "uniffi")
        || package_preserves_uniffi_surface(project, reduced, &package.name)
    {
        return value;
    }

    let Some(table) = value.as_table_mut() else {
        return value;
    };
    let Some(crate_types) = table.get_mut("crate-type").and_then(Value::as_array_mut) else {
        return value;
    };
    crate_types.retain(|crate_type| {
        crate_type
            .as_str()
            .is_none_or(|crate_type| !matches!(crate_type, "cdylib" | "staticlib"))
    });
    if crate_types.is_empty() {
        crate_types.push(Value::String("lib".to_string()));
    }
    value
}

fn package_depends_on(package: &Package, dependency_name: &str) -> bool {
    package
        .dependencies
        .iter()
        .any(|dependency| dependency_name_matches(dependency, dependency_name))
}

fn named_target_matches_entry_source(
    package: &Package,
    target_table: &str,
    target: &Value,
    entry_source: &Path,
) -> bool {
    let Some(table) = target.as_table() else {
        return false;
    };

    if let Some(path) = table.get("path").and_then(Value::as_str) {
        return package
            .root
            .join(path)
            .canonicalize()
            .is_ok_and(|path| path == entry_source);
    }

    let Some(name) = table.get("name").and_then(Value::as_str) else {
        return false;
    };

    default_named_target_paths(package, target_table, name)
        .into_iter()
        .filter_map(|path| path.canonicalize().ok())
        .any(|path| path == entry_source)
}

fn default_named_target_paths(package: &Package, target_table: &str, name: &str) -> Vec<PathBuf> {
    match target_table {
        "bin" => vec![
            package.root.join("src/main.rs"),
            package.root.join("src/bin").join(format!("{name}.rs")),
            package.root.join("src/bin").join(name).join("main.rs"),
        ],
        "example" => vec![
            package.root.join("examples").join(format!("{name}.rs")),
            package.root.join("examples").join(name).join("main.rs"),
        ],
        "test" => vec![
            package.root.join("tests").join(format!("{name}.rs")),
            package.root.join("tests").join(name).join("main.rs"),
        ],
        "bench" => vec![
            package.root.join("benches").join(format!("{name}.rs")),
            package.root.join("benches").join(name).join("main.rs"),
        ],
        _ => Vec::new(),
    }
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

fn retained_workspace_dependencies(
    project: &Project,
    reduced: &ReducedProject,
    package_usages: &HashMap<String, PackageSourceUsage>,
) -> Table {
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
        let Some(package_usage) = package_usages.get(package_name) else {
            continue;
        };
        for (table_name, table) in package_dependency_tables(package) {
            let retention = if table_name == "build-dependencies"
                && build_script_should_render_with_usage(package, package_usage)
            {
                DependencyRetention::BuildScript
            } else {
                DependencyRetention::SourceMentioned
            };
            let retain_for_copied_support_source =
                table_name == "dependencies" && package_should_copy_library_support_source(package);

            for (alias, value) in table {
                let dependency_package = dependency_package_name(alias, value);
                if is_marker_dependency(alias, &dependency_package)
                    || project.workspace.packages.contains_key(&dependency_package)
                    || !dependency_uses_workspace(value)
                    || !dependency_should_render(
                        project,
                        reduced,
                        package_name,
                        alias,
                        retention,
                        DependencyUsageScope::Any,
                        package_usage,
                        retain_for_copied_support_source,
                    )
                {
                    continue;
                }
                if let Some(source) = source_dependencies.get(alias) {
                    dependencies.insert(
                        alias.clone(),
                        dependency_value_with_resolved_path(source, &project.workspace.root),
                    );
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

#[derive(Clone, Copy)]
enum DependencyUsageScope<'a> {
    Any,
    General,
    Target(&'a str),
}

#[allow(clippy::too_many_arguments)]
fn transformed_dependencies(
    project: &Project,
    reduced: &ReducedProject,
    package_name: &str,
    table_name: &str,
    retention: DependencyRetention,
    usage_scope: DependencyUsageScope<'_>,
    package_usage: &PackageSourceUsage,
    feature_required_aliases: &BTreeSet<String>,
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
    let retain_for_copied_support_source =
        table_name == "dependencies" && package_should_copy_library_support_source(package);

    let mut dependencies = Table::new();
    for (alias, value) in source_dependencies {
        let dependency_package = dependency_package_name(alias, value);
        let is_feature_required = feature_required_aliases.contains(alias);
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
            } else if is_feature_required {
                retained_aliases.insert(alias.clone());
                dependencies.insert(
                    alias.clone(),
                    dependency_value_with_resolved_path(value, &package.root),
                );
            }
            continue;
        }

        if dependency_should_render(
            project,
            reduced,
            package_name,
            alias,
            retention,
            usage_scope,
            package_usage,
            retain_for_copied_support_source,
        ) || is_feature_required
        {
            retained_aliases.insert(alias.clone());
            dependencies.insert(
                alias.clone(),
                dependency_value_with_resolved_path(value, &package.root),
            );
        }
    }

    Ok(dependencies)
}

fn transformed_target_dependencies(
    project: &Project,
    reduced: &ReducedProject,
    package_name: &str,
    package_usage: &PackageSourceUsage,
    feature_required_aliases: &BTreeSet<String>,
    retained_aliases: &mut BTreeSet<String>,
) -> Result<Option<Table>, Box<dyn std::error::Error>> {
    let package = project
        .workspace
        .packages
        .get(package_name)
        .ok_or_else(|| format!("unknown package {package_name}"))?;
    let retain_for_copied_support_source = package_should_copy_library_support_source(package);

    let source_targets = package.manifest.get("target").and_then(Value::as_table);
    let mut target_names = package_usage
        .target_names()
        .cloned()
        .collect::<BTreeSet<_>>();
    if let Some(targets) = source_targets {
        target_names.extend(targets.keys().cloned());
    }
    if target_names.is_empty() {
        return Ok(None);
    }

    let mut rendered_targets = Table::new();
    for target_name in target_names {
        let mut rendered_target = Table::new();
        let mut rendered_dependencies = Table::new();

        if let Some(source_dependencies) = source_targets
            .and_then(|targets| targets.get(&target_name))
            .and_then(Value::as_table)
            .and_then(|target_table| target_table.get("dependencies"))
            .and_then(Value::as_table)
        {
            let dependencies = transformed_dependency_table(
                project,
                reduced,
                package_name,
                source_dependencies,
                DependencyRetention::SourceMentioned,
                DependencyUsageScope::Any,
                package_usage,
                feature_required_aliases,
                retained_aliases,
                retain_for_copied_support_source,
            );
            rendered_dependencies.extend(dependencies);
        }

        if let Some(source_dependencies) = package
            .manifest
            .get("dependencies")
            .and_then(Value::as_table)
        {
            let mut promoted_retained_aliases = BTreeSet::new();
            let promoted_dependencies = transformed_dependency_table(
                project,
                reduced,
                package_name,
                source_dependencies,
                DependencyRetention::SourceMentioned,
                DependencyUsageScope::Target(&target_name),
                package_usage,
                feature_required_aliases,
                &mut promoted_retained_aliases,
                retain_for_copied_support_source,
            );
            for (alias, value) in promoted_dependencies {
                if package_usage.mentions_dependency_in_scope(
                    &alias,
                    DependencyUsageScope::Target(&target_name),
                ) && !package_usage
                    .mentions_dependency_in_scope(&alias, DependencyUsageScope::General)
                {
                    retained_aliases.insert(alias.clone());
                    rendered_dependencies.entry(alias).or_insert(value);
                }
            }
        }

        if !rendered_dependencies.is_empty() {
            rendered_target.insert(
                "dependencies".to_string(),
                Value::Table(rendered_dependencies),
            );
        }

        if build_script_should_render_with_usage(package, package_usage) {
            if let Some(source_dependencies) = source_targets
                .and_then(|targets| targets.get(&target_name))
                .and_then(Value::as_table)
                .and_then(|target_table| target_table.get("build-dependencies"))
                .and_then(Value::as_table)
            {
                let build_dependencies = transformed_dependency_table(
                    project,
                    reduced,
                    package_name,
                    source_dependencies,
                    DependencyRetention::BuildScript,
                    DependencyUsageScope::Any,
                    package_usage,
                    feature_required_aliases,
                    retained_aliases,
                    retain_for_copied_support_source,
                );
                if !build_dependencies.is_empty() {
                    rendered_target.insert(
                        "build-dependencies".to_string(),
                        Value::Table(build_dependencies),
                    );
                }
            }
        }

        if package_target_uses_dev_dependencies(package) {
            if let Some(source_dependencies) = source_targets
                .and_then(|targets| targets.get(&target_name))
                .and_then(Value::as_table)
                .and_then(|target_table| target_table.get("dev-dependencies"))
                .and_then(Value::as_table)
            {
                let dev_dependencies = transformed_dependency_table(
                    project,
                    reduced,
                    package_name,
                    source_dependencies,
                    DependencyRetention::SourceMentioned,
                    DependencyUsageScope::Any,
                    package_usage,
                    feature_required_aliases,
                    retained_aliases,
                    retain_for_copied_support_source,
                );
                if !dev_dependencies.is_empty() {
                    rendered_target.insert(
                        "dev-dependencies".to_string(),
                        Value::Table(dev_dependencies),
                    );
                }
            }
        }

        if !rendered_target.is_empty() {
            rendered_targets.insert(target_name, Value::Table(rendered_target));
        }
    }

    Ok((!rendered_targets.is_empty()).then_some(rendered_targets))
}

#[allow(clippy::too_many_arguments)]
fn transformed_dependency_table(
    project: &Project,
    reduced: &ReducedProject,
    package_name: &str,
    source_dependencies: &Table,
    retention: DependencyRetention,
    usage_scope: DependencyUsageScope<'_>,
    package_usage: &PackageSourceUsage,
    feature_required_aliases: &BTreeSet<String>,
    retained_aliases: &mut BTreeSet<String>,
    retain_for_copied_support_source: bool,
) -> Table {
    let Some(package) = project.workspace.packages.get(package_name) else {
        return Table::new();
    };
    let mut dependencies = Table::new();
    for (alias, value) in source_dependencies {
        let dependency_package = dependency_package_name(alias, value);
        let is_feature_required = feature_required_aliases.contains(alias);
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
            } else if is_feature_required {
                retained_aliases.insert(alias.clone());
                dependencies.insert(
                    alias.clone(),
                    dependency_value_with_resolved_path(value, &package.root),
                );
            }
            continue;
        }

        if dependency_should_render(
            project,
            reduced,
            package_name,
            alias,
            retention,
            usage_scope,
            package_usage,
            retain_for_copied_support_source,
        ) || is_feature_required
        {
            retained_aliases.insert(alias.clone());
            let value = project
                .workspace
                .packages
                .get(package_name)
                .map(|package| dependency_value_with_resolved_path(value, &package.root))
                .unwrap_or_else(|| value.clone());
            dependencies.insert(alias.clone(), value);
        }
    }

    dependencies
}

#[allow(clippy::too_many_arguments)]
fn dependency_should_render(
    project: &Project,
    reduced: &ReducedProject,
    package_name: &str,
    alias: &str,
    retention: DependencyRetention,
    usage_scope: DependencyUsageScope<'_>,
    package_usage: &PackageSourceUsage,
    retain_for_copied_support_source: bool,
) -> bool {
    retention == DependencyRetention::BuildScript
        || project
            .workspace
            .packages
            .get(package_name)
            .is_some_and(|package| package_should_preserve_source_tree(project, reduced, package))
        || retain_for_copied_support_source
        || package_usage.mentions_dependency_in_scope(alias, usage_scope)
}

#[derive(Default)]
struct PackageSourceUsage {
    all: TokenUsage,
    general: TokenUsage,
    targets: BTreeMap<String, TokenUsage>,
}

impl PackageSourceUsage {
    fn record_file(&mut self, file: &syn::File, target_names: &BTreeSet<String>) {
        self.all.record_file(file);
        if target_names.is_empty() {
            self.general.record_file(file);
        } else {
            for target_name in target_names {
                self.targets
                    .entry(target_name.clone())
                    .or_default()
                    .record_file(file);
            }
        }
    }

    fn mentions_ident(&self, ident: &str) -> bool {
        self.all.mentions_ident(ident)
    }

    fn mentions_dependency(&self, alias: &str) -> bool {
        self.all.mentions_dependency(alias)
    }

    fn mentions_dependency_in_scope(
        &self,
        alias: &str,
        usage_scope: DependencyUsageScope<'_>,
    ) -> bool {
        match usage_scope {
            DependencyUsageScope::Any => self.mentions_dependency(alias),
            DependencyUsageScope::General => self.general.mentions_dependency(alias),
            DependencyUsageScope::Target(target_name) => self
                .targets
                .get(target_name)
                .is_some_and(|usage| usage.mentions_dependency(alias)),
        }
    }

    fn target_names(&self) -> impl Iterator<Item = &String> {
        self.targets.keys()
    }
}

#[derive(Default)]
struct TokenUsage {
    idents: BTreeSet<String>,
    path_roots: BTreeSet<String>,
    use_idents: BTreeSet<String>,
}

impl TokenUsage {
    fn record_file(&mut self, file: &syn::File) {
        collect_token_usage(&file.to_token_stream(), self);
        for item in &file.items {
            if let Item::Use(item_use) = item {
                collect_use_tree_idents(&item_use.tree, &mut self.use_idents);
            }
        }
    }

    fn mentions_ident(&self, ident: &str) -> bool {
        self.idents.contains(ident)
    }

    fn mentions_dependency(&self, alias: &str) -> bool {
        let code_name = dependency_code_name(alias);
        self.path_roots.contains(&code_name)
            || self.use_idents.contains(&code_name)
            || (alias != code_name
                && (self.path_roots.contains(alias) || self.use_idents.contains(alias)))
            || known_macro_dependency_usage(self, alias, &code_name)
    }
}

fn package_source_usage(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package_name: &str,
) -> PackageSourceUsage {
    let mut usage = PackageSourceUsage::default();
    let retain_test_items = project
        .workspace
        .packages
        .get(package_name)
        .is_some_and(package_target_preserves_test_items);
    for source in project
        .files
        .values()
        .filter(|source| source.package == package_name)
        .filter(|source| {
            module_should_render(
                project,
                reduced,
                render_plan,
                &source.package,
                &source.module_path,
            )
        })
    {
        let file = transform_file(
            project,
            reduced,
            render_plan,
            &source.package,
            &source.module_path,
            &source.syntax,
            retain_test_items,
        );
        let target_names = module_cfg_target_names(project, &source.package, &source.module_path);
        usage.record_file(&file, &target_names);
    }
    usage
}

fn module_cfg_target_names(
    project: &Project,
    package: &str,
    module_path: &[String],
) -> BTreeSet<String> {
    let mut target_names = BTreeSet::new();
    for depth in 1..=module_path.len() {
        let parent_path = &module_path[..depth - 1];
        let module_name = &module_path[depth - 1];
        let Some(source) = project
            .files
            .values()
            .find(|source| source.package == package && source.module_path == parent_path)
        else {
            continue;
        };
        let Some(item_mod) = source.syntax.items.iter().find_map(|item| {
            let Item::Mod(item_mod) = item else {
                return None;
            };
            (item_mod.ident == module_name.as_str()).then_some(item_mod)
        }) else {
            continue;
        };
        target_names.extend(item_mod.attrs.iter().filter_map(|attr| {
            cfg_attr_target_name(attr)
                .map(|target_name| canonical_target_name(project, package, &target_name))
        }));
    }
    target_names
}

fn canonical_target_name(project: &Project, package: &str, target_name: &str) -> String {
    let compact_name = compact_target_name(target_name);
    project
        .workspace
        .packages
        .get(package)
        .and_then(|package| package.manifest.get("target"))
        .and_then(Value::as_table)
        .and_then(|targets| {
            targets
                .keys()
                .find(|existing| compact_target_name(existing) == compact_name)
                .cloned()
        })
        .unwrap_or_else(|| target_name.to_string())
}

fn compact_target_name(target_name: &str) -> String {
    let mut output = String::new();
    let mut in_string = false;
    let mut escaped = false;
    for ch in target_name.chars() {
        if in_string {
            output.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
        } else if ch == '"' {
            in_string = true;
            output.push(ch);
        } else if !ch.is_whitespace() {
            output.push(ch);
        }
    }
    output
}

fn cfg_attr_target_name(attr: &syn::Attribute) -> Option<String> {
    if !attr.path().is_ident("cfg") {
        return None;
    }
    let meta = attr.parse_args::<syn::Meta>().ok()?;
    Some(format!(
        "cfg({})",
        compact_token_stream(&meta.to_token_stream())
    ))
}

fn compact_token_stream(tokens: &TokenStream) -> String {
    let mut output = String::new();
    for token in tokens.clone() {
        match token {
            TokenTree::Ident(ident) => output.push_str(&ident.to_string()),
            TokenTree::Punct(punct) => output.push(punct.as_char()),
            TokenTree::Literal(literal) => output.push_str(&literal.to_string()),
            TokenTree::Group(group) => {
                let (open, close) = match group.delimiter() {
                    proc_macro2::Delimiter::Parenthesis => ('(', ')'),
                    proc_macro2::Delimiter::Brace => ('{', '}'),
                    proc_macro2::Delimiter::Bracket => ('[', ']'),
                    proc_macro2::Delimiter::None => (' ', ' '),
                };
                if group.delimiter() == proc_macro2::Delimiter::None {
                    output.push_str(&compact_token_stream(&group.stream()));
                } else {
                    output.push(open);
                    output.push_str(&compact_token_stream(&group.stream()));
                    output.push(close);
                }
            }
        }
    }
    output
}

fn package_source_usages(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
) -> HashMap<String, PackageSourceUsage> {
    reduced
        .packages
        .iter()
        .map(|package| {
            (
                package.clone(),
                package_source_usage(project, reduced, render_plan, package.as_str()),
            )
        })
        .collect()
}

fn collect_token_usage(tokens: &TokenStream, usage: &mut TokenUsage) {
    let token_trees = tokens.clone().into_iter().collect::<Vec<_>>();
    for token in &token_trees {
        match token {
            TokenTree::Ident(ident) => {
                usage.idents.insert(ident.to_string());
            }
            TokenTree::Group(group) => collect_token_usage(&group.stream(), usage),
            TokenTree::Punct(_) | TokenTree::Literal(_) => {}
        }
    }

    for window in token_trees.windows(3) {
        if let (TokenTree::Ident(candidate), TokenTree::Punct(first), TokenTree::Punct(second)) =
            (&window[0], &window[1], &window[2])
        {
            if first.as_char() == ':' && second.as_char() == ':' {
                usage.path_roots.insert(candidate.to_string());
            }
        }
    }
}

fn collect_use_tree_idents(tree: &UseTree, idents: &mut BTreeSet<String>) {
    match tree {
        UseTree::Path(path) => {
            idents.insert(path.ident.to_string());
            collect_use_tree_idents(&path.tree, idents);
        }
        UseTree::Name(name) => {
            idents.insert(name.ident.to_string());
        }
        UseTree::Rename(rename) => {
            idents.insert(rename.ident.to_string());
            idents.insert(rename.rename.to_string());
        }
        UseTree::Group(group) => {
            for item in &group.items {
                collect_use_tree_idents(item, idents);
            }
        }
        UseTree::Glob(_) => {}
    }
}

fn known_macro_dependency_usage(usage: &TokenUsage, alias: &str, code_name: &str) -> bool {
    match code_name {
        "serde" | "serde_derive" => {
            usage.mentions_ident("Serialize")
                || usage.mentions_ident("Deserialize")
                || usage.mentions_ident("serde")
        }
        "thiserror" => usage.mentions_ident("Error") && usage.mentions_ident("error"),
        "bitflags" => usage.mentions_ident("bitflags"),
        "error_support" => error_support_macro_idents()
            .iter()
            .any(|ident| usage.mentions_ident(ident)),
        "error_support_macros" => usage.mentions_ident("handle_error"),
        "lazy_static" => usage.mentions_ident("lazy_static"),
        "uniffi" => usage.mentions_ident("uniffi"),
        "clap" => {
            usage.mentions_ident("Parser")
                || usage.mentions_ident("Subcommand")
                || usage.mentions_ident("Args")
                || usage.mentions_ident("ValueEnum")
                || usage.mentions_ident("command")
                || usage.mentions_ident("arg")
        }
        _ => alias != code_name && usage.mentions_ident(alias),
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
        "bitflags" => reachable_package_mentions_ident(project, reduced, package, "bitflags"),
        "error_support" => error_support_macro_idents()
            .iter()
            .any(|ident| reachable_package_mentions_ident(project, reduced, package, ident)),
        "error_support_macros" => {
            reachable_package_mentions_ident(project, reduced, package, "handle_error")
        }
        "lazy_static" => reachable_package_mentions_ident(project, reduced, package, "lazy_static"),
        "uniffi" => package_preserves_uniffi_surface(project, reduced, package),
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

fn known_macro_dependency_target_should_remain(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    target: &[String],
) -> bool {
    let Some(first) = target.first() else {
        return false;
    };
    let Some(leaf) = target.last() else {
        return false;
    };
    match dependency_code_name(first).as_str() {
        "bitflags" if leaf == "bitflags" => true,
        "error_support_macros" if leaf == "handle_error" => true,
        "lazy_static" if leaf == "lazy_static" => true,
        "serde" | "serde_derive" if matches!(leaf.as_str(), "Deserialize" | "Serialize") => {
            reachable_package_mentions_ident(project, reduced, package, leaf)
        }
        "serde" | "serde_derive" => false,
        "thiserror" if leaf == "Error" => true,
        "error_support" if error_support_macro_idents().contains(&leaf.as_str()) => {
            reachable_package_mentions_ident(project, reduced, package, leaf)
        }
        "error_support" => false,
        code_name => known_macro_dependency_package_mentions(project, reduced, package, code_name),
    }
}

fn error_support_macro_idents() -> &'static [&'static str] {
    &[
        "breadcrumb",
        "debug",
        "error",
        "info",
        "report_error",
        "trace",
        "trace_error",
        "warn",
    ]
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

fn token_stream_mentions_dependency_public_name(
    project: &Project,
    caller_package: &str,
    dependency_package: &str,
    visible_name: &str,
    tokens: &TokenStream,
) -> bool {
    let token_trees = tokens.clone().into_iter().collect::<Vec<_>>();
    if token_trees.iter().any(|token| {
        matches!(token, TokenTree::Group(group)
        if token_stream_mentions_dependency_public_name(
            project,
            caller_package,
            dependency_package,
            visible_name,
            &group.stream(),
        ))
    }) {
        return true;
    }

    let mut index = 0;
    while index < token_trees.len() {
        let TokenTree::Ident(ident) = &token_trees[index] else {
            index += 1;
            continue;
        };

        let mut segments = vec![ident.to_string()];
        let mut cursor = index + 1;
        while token_trees_have_path_separator(&token_trees, cursor) {
            let Some(TokenTree::Ident(next)) = token_trees.get(cursor + 2) else {
                break;
            };
            segments.push(next.to_string());
            cursor += 3;
        }

        if segments.len() >= 2
            && segments[1] == visible_name
            && first_segment_targets_dependency(
                project,
                caller_package,
                dependency_package,
                &segments[0],
            )
        {
            return true;
        }

        index = cursor.max(index + 1);
    }

    false
}

fn token_trees_have_path_separator(tokens: &[TokenTree], index: usize) -> bool {
    matches!(tokens.get(index), Some(TokenTree::Punct(punct)) if punct.as_char() == ':')
        && matches!(tokens.get(index + 1), Some(TokenTree::Punct(punct)) if punct.as_char() == ':')
}

fn first_segment_targets_dependency(
    project: &Project,
    caller_package: &str,
    dependency_package: &str,
    first: &str,
) -> bool {
    if first == dependency_package || first == dependency_code_name(dependency_package) {
        return true;
    }

    project
        .workspace
        .packages
        .get(caller_package)
        .is_some_and(|package| {
            package.dependencies.iter().any(|dependency| {
                dependency.package == dependency_package
                    && dependency_name_matches(dependency, first)
            })
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

fn dependency_value_with_resolved_path(value: &Value, manifest_dir: &Path) -> Value {
    let Some(table) = value.as_table() else {
        return value.clone();
    };
    let Some(path) = table.get("path").and_then(Value::as_str) else {
        return value.clone();
    };

    let mut table = table.clone();
    let path = PathBuf::from(path);
    let path = if path.is_absolute() {
        path
    } else {
        manifest_dir.join(path)
    };
    let path = path.canonicalize().unwrap_or(path);
    table.insert("path".to_string(), Value::String(toml_path(&path)));
    Value::Table(table)
}

fn toml_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn is_marker_dependency(alias: &str, package: &str) -> bool {
    alias == "opensourced" || package == "opensourced"
}

fn package_dependency_tables(package: &Package) -> Vec<(&str, &Table)> {
    let mut tables = Vec::new();
    for table_name in ["dependencies", "build-dependencies", "dev-dependencies"] {
        if table_name == "dev-dependencies" && !package_target_uses_dev_dependencies(package) {
            continue;
        }
        if let Some(table) = package.manifest.get(table_name).and_then(Value::as_table) {
            tables.push((table_name, table));
        }
    }
    if let Some(targets) = package.manifest.get("target").and_then(Value::as_table) {
        for target in targets.values() {
            let Some(target) = target.as_table() else {
                continue;
            };
            for table_name in ["dependencies", "build-dependencies", "dev-dependencies"] {
                if table_name == "dev-dependencies"
                    && !package_target_uses_dev_dependencies(package)
                {
                    continue;
                }
                if let Some(table) = target.get(table_name).and_then(Value::as_table) {
                    tables.push((table_name, table));
                }
            }
        }
    }
    tables
}

fn transformed_features(package: &Package, retained_aliases: &BTreeSet<String>) -> Option<Value> {
    let features = package.manifest.get("features")?.as_table()?;
    let source_aliases = package_dependency_aliases(package);

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

fn requested_local_features(
    project: &Project,
    reduced: &ReducedProject,
    package_name: &str,
) -> BTreeSet<String> {
    let mut features = BTreeSet::new();
    for dependent_name in &reduced.packages {
        let Some(dependent) = project.workspace.packages.get(dependent_name) else {
            continue;
        };
        for (_table_name, table) in package_dependency_tables(dependent) {
            for (alias, value) in table {
                if dependency_package_name(alias, value) == package_name {
                    collect_dependency_requested_features(value, &mut features);
                }
            }
        }
    }
    features
}

fn collect_dependency_requested_features(value: &Value, features: &mut BTreeSet<String>) {
    let Some(table) = value.as_table() else {
        return;
    };
    let default_features_enabled = table
        .get("default-features")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    if default_features_enabled {
        features.insert("default".to_string());
    }
    if let Some(requested) = table.get("features").and_then(Value::as_array) {
        features.extend(
            requested
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string),
        );
    }
}

fn dependency_aliases_required_by_features(
    package: &Package,
    requested_features: &BTreeSet<String>,
) -> BTreeSet<String> {
    let source_aliases = package_dependency_aliases(package);
    let Some(features) = package.manifest.get("features").and_then(Value::as_table) else {
        return requested_features
            .intersection(&source_aliases)
            .cloned()
            .collect();
    };

    let mut required = BTreeSet::new();
    let mut visited = BTreeSet::new();
    let mut pending = requested_features.iter().cloned().collect::<Vec<_>>();
    while let Some(feature) = pending.pop() {
        if !visited.insert(feature.clone()) {
            continue;
        }
        if source_aliases.contains(&feature) {
            required.insert(feature.clone());
        }
        let Some(values) = features.get(&feature).and_then(Value::as_array) else {
            continue;
        };
        for value in values {
            let Some(item) = value.as_str() else {
                continue;
            };
            if let Some(alias) = feature_dependency_alias(item, &source_aliases) {
                required.insert(alias);
            } else {
                pending.push(item.to_string());
            }
        }
    }

    required
}

fn package_dependency_aliases(package: &Package) -> BTreeSet<String> {
    let mut aliases = BTreeSet::new();
    for table_name in ["dependencies", "build-dependencies", "dev-dependencies"] {
        if let Some(table) = package.manifest.get(table_name).and_then(Value::as_table) {
            aliases.extend(table.keys().cloned());
        }
    }
    if let Some(targets) = package.manifest.get("target").and_then(Value::as_table) {
        for target in targets.values() {
            let Some(target) = target.as_table() else {
                continue;
            };
            for table_name in ["dependencies", "build-dependencies", "dev-dependencies"] {
                if let Some(table) = target.get(table_name).and_then(Value::as_table) {
                    aliases.extend(table.keys().cloned());
                }
            }
        }
    }
    aliases
}

fn feature_dependency_alias(item: &str, source_aliases: &BTreeSet<String>) -> Option<String> {
    let dependency = item
        .strip_prefix("dep:")
        .or_else(|| item.split_once('/').map(|(dependency, _)| dependency))
        .or_else(|| item.split_once("?/").map(|(dependency, _)| dependency))
        .unwrap_or(item);
    let dependency = dependency.strip_suffix('?').unwrap_or(dependency);
    source_aliases
        .contains(dependency)
        .then(|| dependency.to_string())
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
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
    syntax: &syn::File,
    retain_test_items: bool,
) -> syn::File {
    let mut transformed = syntax.clone();
    transformed.items = transform_items(
        project,
        reduced,
        render_plan,
        package,
        module_path,
        &syntax.items,
        retain_test_items,
    );
    transformed
}

fn transform_items(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
    items: &[Item],
    retain_test_items: bool,
) -> Vec<Item> {
    let preserve_uniffi_surface = package_preserves_uniffi_surface(project, reduced, package);
    let retained_macro_definitions =
        retained_macro_definitions_for_generated_items(project, reduced, package, items);

    let mut transformed = Vec::new();
    for item in items {
        let item = match item {
            _ if item_is_test(item) && !retain_test_items => None,
            Item::Use(item_use) if use_mentions_opensourced(&item_use.tree) => None,
            Item::Use(item_use) => {
                let mut item_use = item_use.clone();
                let is_public_use = use_is_reexport(&item_use.vis);
                let tree = prune_use_tree(
                    project,
                    reduced,
                    render_plan,
                    package,
                    module_path,
                    &item_use.tree,
                    Vec::new(),
                    is_public_use,
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
                    if !preserve_uniffi_surface {
                        strip_uniffi_attrs(&mut function.attrs);
                    }
                    allow_dead_code_if_not_public(&function.vis, &mut function.attrs);
                    Item::Fn(function)
                })
            }
            Item::Macro(item_macro) if is_automod_dir_macro(&item_macro.mac.path) => {
                automod_macro_has_reduced_modules(
                    project,
                    reduced,
                    render_plan,
                    package,
                    module_path,
                )
                .then(|| Item::Macro(item_macro.clone()))
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
            Item::Struct(item_struct) => item_id(package, module_path, item).and_then(|id| {
                render_plan.item_should_render(&id).then(|| {
                    let mut item_struct = item_struct.clone();
                    strip_opensourced_attrs(&mut item_struct.attrs);
                    if !preserve_uniffi_surface {
                        strip_uniffi_attrs_from_fields(&mut item_struct.fields);
                        strip_uniffi_attrs(&mut item_struct.attrs);
                    }
                    let preserve_private_fields = root_item_should_render(reduced, &id);
                    prune_private_struct_fields(
                        project,
                        reduced,
                        render_plan,
                        package,
                        &mut item_struct,
                        preserve_private_fields,
                    );
                    allow_dead_code_for_private_struct_fields(&mut item_struct);
                    allow_dead_code_if_not_public(&item_struct.vis, &mut item_struct.attrs);
                    Item::Struct(item_struct)
                })
            }),
            Item::Trait(item_trait) => item_id(package, module_path, item).and_then(|id| {
                render_plan.item_should_render(&id).then(|| {
                    let mut item_trait = item_trait.clone();
                    strip_opensourced_attrs(&mut item_trait.attrs);
                    if !preserve_uniffi_surface {
                        strip_uniffi_attrs(&mut item_trait.attrs);
                    }
                    prune_trait_items_if_only_type_surface(
                        project,
                        reduced,
                        package,
                        module_path,
                        &mut item_trait,
                    );
                    allow_dead_code_if_not_public(&item_trait.vis, &mut item_trait.attrs);
                    Item::Trait(item_trait)
                })
            }),
            Item::Enum(_)
            | Item::Union(_)
            | Item::Type(_)
            | Item::Const(_)
            | Item::Static(_)
            | Item::Macro(_) => item_id(package, module_path, item).and_then(|id| {
                render_plan.item_should_render(&id).then(|| {
                    let mut item = item.clone();
                    strip_opensourced_attrs_from_item(&mut item);
                    if !preserve_uniffi_surface {
                        strip_uniffi_attrs_from_item(&mut item);
                    }
                    allow_dead_code_for_non_public_item(&mut item);
                    item
                })
            }),
            Item::Impl(item_impl) => {
                let aliases = project
                    .module_aliases
                    .get(&(package.to_string(), module_path.to_vec()))
                    .cloned()
                    .unwrap_or_default();
                let Some(type_path) = resolved_local_type_path(
                    project,
                    package,
                    module_path,
                    &item_impl.self_ty,
                    &aliases,
                ) else {
                    continue;
                };
                let trait_path = item_impl
                    .trait_
                    .as_ref()
                    .map(|(_, path, _)| normalized_path(module_path, path, &aliases));
                let trait_input_type_paths = item_impl
                    .trait_
                    .as_ref()
                    .map(|(_, path, _)| trait_input_type_paths(module_path, path, &aliases))
                    .unwrap_or_default();
                let mut kept_impl_items = Vec::new();
                let mut kept_method = false;
                let trait_impl_is_required = trait_path.as_ref().is_some_and(|trait_path| {
                    trait_impl_items_are_reachable(reduced, package, &type_path, trait_path)
                });
                let marker_trait_impl_is_required = trait_path
                    .as_ref()
                    .and_then(|path| path.last())
                    .is_some_and(|trait_name| {
                        marker_trait_impl_should_remain(reduced, package, &type_path, trait_name)
                    })
                    && item_impl
                        .items
                        .iter()
                        .filter(|impl_item| retain_test_items || !impl_item_is_test(impl_item))
                        .all(|impl_item| !matches!(impl_item, ImplItem::Fn(_)));

                for impl_item in &item_impl.items {
                    if let ImplItem::Fn(method) = impl_item {
                        if attrs_are_test(&method.attrs) && !retain_test_items {
                            continue;
                        }
                        let id = CallableId::Method {
                            package: package.to_string(),
                            type_path: type_path.clone(),
                            trait_path: trait_path.clone(),
                            trait_input_type_paths: trait_input_type_paths.clone(),
                            method: method.sig.ident.to_string(),
                        };
                        if reduced.reachable.contains(&id) {
                            let mut method = method.clone();
                            strip_opensourced_attrs(&mut method.attrs);
                            if !preserve_uniffi_surface {
                                strip_uniffi_attrs(&mut method.attrs);
                            }
                            allow_dead_code_if_not_public(&method.vis, &mut method.attrs);
                            kept_impl_items.push(ImplItem::Fn(method));
                            kept_method = true;
                        }
                    }
                }

                if kept_method || trait_impl_is_required || marker_trait_impl_is_required {
                    if trait_path.is_some() {
                        kept_impl_items.clear();
                        for impl_item in &item_impl.items {
                            if retain_test_items || !impl_item_is_test(impl_item) {
                                let mut impl_item = impl_item.clone();
                                strip_opensourced_attrs_from_impl_item(&mut impl_item);
                                if !preserve_uniffi_surface {
                                    strip_uniffi_attrs_from_impl_item(&mut impl_item);
                                }
                                allow_dead_code_for_non_public_impl_item(&mut impl_item);
                                kept_impl_items.push(impl_item);
                            }
                        }
                    } else {
                        for impl_item in &item_impl.items {
                            if !matches!(impl_item, ImplItem::Fn(_))
                                && (retain_test_items || !impl_item_is_test(impl_item))
                            {
                                kept_impl_items.push(impl_item.clone());
                            }
                        }
                    }

                    let mut item_impl = item_impl.clone();
                    if !preserve_uniffi_surface {
                        strip_uniffi_attrs(&mut item_impl.attrs);
                        for impl_item in &mut kept_impl_items {
                            strip_uniffi_attrs_from_impl_item(impl_item);
                        }
                    }
                    item_impl.items = kept_impl_items;
                    downgrade_uniffi_async_runtime_if_no_async_methods(&mut item_impl);
                    Some(Item::Impl(item_impl))
                } else {
                    None
                }
            }
            Item::Mod(item_mod) => {
                let mut item_mod = item_mod.clone();
                strip_opensourced_attrs(&mut item_mod.attrs);
                if !preserve_uniffi_surface {
                    strip_uniffi_attrs(&mut item_mod.attrs);
                }
                let mut child_path = module_path.to_vec();
                child_path.push(item_mod.ident.to_string());
                let module_item_is_reachable = render_plan.item_should_render(&ItemId {
                    package: package.to_string(),
                    module_path: module_path.to_vec(),
                    name: item_mod.ident.to_string(),
                    kind: ItemKind::Mod,
                });
                if module_contains_root(project, reduced, package, &child_path) {
                    item_mod.vis = parse_quote!(pub);
                }

                if let Some((brace, child_items)) = &item_mod.content {
                    let mut child_items = transform_items(
                        project,
                        reduced,
                        render_plan,
                        package,
                        &child_path,
                        child_items,
                        retain_test_items,
                    );
                    if child_items.is_empty()
                        && (module_item_is_reachable
                            || reachable_package_mentions_ident(
                                project,
                                reduced,
                                package,
                                &item_mod.ident.to_string(),
                            ))
                    {
                        child_items = item_mod
                            .content
                            .as_ref()
                            .map(|(_, original_items)| {
                                original_items
                                    .iter()
                                    .filter(|item| {
                                        matches!(item, Item::Macro(item_macro) if item_macro.ident.is_none())
                                    })
                                    .cloned()
                                    .collect()
                            })
                            .unwrap_or_default();
                    }
                    if child_items.is_empty() {
                        continue;
                    }
                    item_mod.content = Some((*brace, child_items));
                } else if !module_should_render(project, reduced, render_plan, package, &child_path)
                {
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

fn prune_trait_items_if_only_type_surface(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
    item_trait: &mut syn::ItemTrait,
) {
    if trait_has_reachable_impl_methods(
        reduced,
        package,
        module_path,
        &item_trait.ident.to_string(),
    ) {
        return;
    }
    if trait_items_are_referenced_by_reachable_surfaces(project, reduced, package, item_trait) {
        return;
    }

    item_trait.items.clear();
    item_trait.attrs.retain(is_inert_type_surface_attr);
}

fn trait_items_are_referenced_by_reachable_surfaces(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    item_trait: &syn::ItemTrait,
) -> bool {
    item_trait.items.iter().any(|item| {
        let TraitItem::Fn(method) = item else {
            return false;
        };
        let name = method.sig.ident.to_string();
        reachable_reduced_callables_mention_ident(project, reduced, &name)
            || retained_impl_surfaces_mention_ident(project, reduced, package, &name)
    })
}

fn trait_has_reachable_impl_methods(
    reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
    trait_name: &str,
) -> bool {
    let mut trait_path = module_path.to_vec();
    trait_path.push(trait_name.to_string());
    reduced.reachable.iter().any(|callable| {
        matches!(
            callable,
            CallableId::Method {
                package: callable_package,
                trait_path: Some(callable_trait_path),
                ..
            } if callable_package == package && callable_trait_path == &trait_path
        )
    })
}

fn is_inert_type_surface_attr(attr: &syn::Attribute) -> bool {
    let path = attr.path();
    path.is_ident("cfg")
        || path.is_ident("allow")
        || path.is_ident("deny")
        || path.is_ident("doc")
        || path.is_ident("deprecated")
}

fn module_contains_root(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
) -> bool {
    reduced.roots.iter().any(|root| match root {
        RootId::Callable(callable) => match callable {
            CallableId::Free {
                package: root_package,
                module_path: root_module,
                ..
            } => root_package == package && path_has_prefix(root_module, module_path),
            CallableId::Method {
                package: root_package,
                type_path,
                ..
            } => {
                root_package == package
                    && project
                        .methods
                        .get(callable)
                        .map(|record| path_has_prefix(&record.module_path, module_path))
                        .unwrap_or_else(|| path_has_prefix(type_path, module_path))
            }
        },
        RootId::Item(item) => {
            item.package == package && path_has_prefix(&path_from_item(item), module_path)
        }
    })
}

fn module_should_render(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
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
    }) || render_plan.reachable_items.iter().any(|item| {
        item.package == package
            && (path_has_prefix(&item.module_path, module_path)
                || path_has_prefix(&path_from_item(item), module_path))
    })
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
        Item::Mod(item) => (item.ident.to_string(), ItemKind::Mod),
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

fn path_from_item(item: &ItemId) -> Vec<String> {
    let mut path = item.module_path.clone();
    path.push(item.name.clone());
    path
}

fn automod_macro_has_reduced_modules(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
) -> bool {
    project
        .files
        .values()
        .filter(|source| source.package == package)
        .filter(|source| source.module_path.len() == module_path.len() + 1)
        .filter(|source| path_has_prefix(&source.module_path, module_path))
        .any(|source| {
            module_should_render(project, reduced, render_plan, package, &source.module_path)
        })
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
        return package_preserves_uniffi_surface(project, reduced, package);
    }
    if is_uniffi && !package_preserves_uniffi_surface(project, reduced, package) {
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
        .any(|ident| {
            reachable_package_mentions_ident(project, reduced, package, ident)
                || reachable_reduced_packages_mention_ident(project, reduced, ident)
        })
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

fn attr_path_starts_with(path: &syn::Path, name: &str) -> bool {
    path_starts_with(path, name)
}

fn path_starts_with(path: &syn::Path, name: &str) -> bool {
    path.segments
        .first()
        .is_some_and(|segment| segment.ident == name)
}

fn package_preserves_uniffi_surface(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
) -> bool {
    reduced
        .reachable
        .iter()
        .filter(|callable| callable.package() == package)
        .any(|callable| {
            project
                .functions
                .get(callable)
                .is_some_and(|record| attrs_include_uniffi_export(&record.item.attrs))
                || project
                    .methods
                    .get(callable)
                    .is_some_and(|record| attrs_include_uniffi_export(&record.item.attrs))
        })
        || reduced
            .reachable_items
            .iter()
            .filter(|item| item.package() == package)
            .any(|item| {
                project.items.get(item).is_some_and(|record| {
                    item_attrs(&record.item).is_some_and(attrs_include_uniffi_export)
                })
            })
}

fn item_attrs(item: &Item) -> Option<&[syn::Attribute]> {
    match item {
        Item::Const(item) => Some(&item.attrs),
        Item::Enum(item) => Some(&item.attrs),
        Item::Fn(item) => Some(&item.attrs),
        Item::Impl(item) => Some(&item.attrs),
        Item::Macro(item) => Some(&item.attrs),
        Item::Mod(item) => Some(&item.attrs),
        Item::Static(item) => Some(&item.attrs),
        Item::Struct(item) => Some(&item.attrs),
        Item::Trait(item) => Some(&item.attrs),
        Item::Type(item) => Some(&item.attrs),
        Item::Union(item) => Some(&item.attrs),
        _ => None,
    }
}

fn attrs_include_uniffi_export(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        let path = attr.path();
        (path.segments.len() == 2
            && path.segments[0].ident == "uniffi"
            && path.segments[1].ident == "export")
            || (path.is_ident("cfg_attr")
                && token_stream_mentions_ident(&attr.to_token_stream(), "uniffi")
                && token_stream_mentions_ident(&attr.to_token_stream(), "export"))
    })
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
                    reachable_item_mentions_ident(project, reduced, package, item, record, ident)
                })
            })
}

fn reachable_reduced_packages_mention_ident(
    project: &Project,
    reduced: &ReducedProject,
    ident: &str,
) -> bool {
    reduced
        .packages
        .iter()
        .any(|package| reachable_package_mentions_ident(project, reduced, package, ident))
}

fn retained_impl_attrs_mention_ident(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
    ident: &str,
) -> bool {
    let Some(source) = project
        .files
        .values()
        .find(|source| source.package == package && source.module_path == module_path)
    else {
        return false;
    };
    let aliases = project
        .module_aliases
        .get(&(package.to_string(), module_path.to_vec()))
        .cloned()
        .unwrap_or_default();

    source.syntax.items.iter().any(|item| {
        let Item::Impl(item_impl) = item else {
            return false;
        };
        if !impl_has_reachable_method(project, reduced, package, module_path, item_impl, &aliases) {
            return false;
        }
        item_impl
            .attrs
            .iter()
            .any(|attr| token_stream_mentions_ident(&attr.to_token_stream(), ident))
    })
}

fn retained_impl_non_fn_items_mention_ident(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
    ident: &str,
) -> bool {
    let Some(source) = project
        .files
        .values()
        .find(|source| source.package == package && source.module_path == module_path)
    else {
        return false;
    };
    let aliases = project
        .module_aliases
        .get(&(package.to_string(), module_path.to_vec()))
        .cloned()
        .unwrap_or_default();

    source.syntax.items.iter().any(|item| {
        let Item::Impl(item_impl) = item else {
            return false;
        };
        if !impl_has_reachable_method(project, reduced, package, module_path, item_impl, &aliases) {
            return false;
        }
        item_impl.items.iter().any(|impl_item| {
            !matches!(impl_item, ImplItem::Fn(_))
                && !impl_item_is_test(impl_item)
                && token_stream_mentions_ident(&impl_item.to_token_stream(), ident)
        })
    })
}

fn retained_impl_items_mention_ident(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
    ident: &str,
) -> bool {
    let Some(source) = project
        .files
        .values()
        .find(|source| source.package == package && source.module_path == module_path)
    else {
        return false;
    };
    let aliases = project
        .module_aliases
        .get(&(package.to_string(), module_path.to_vec()))
        .cloned()
        .unwrap_or_default();

    source.syntax.items.iter().any(|item| {
        let Item::Impl(item_impl) = item else {
            return false;
        };
        if item_impl.trait_.is_none() {
            return false;
        }
        if !impl_should_render(project, reduced, package, module_path, item_impl, &aliases) {
            return false;
        }
        item_impl.items.iter().any(|impl_item| {
            !impl_item_is_test(impl_item)
                && token_stream_mentions_ident(&impl_item.to_token_stream(), ident)
        })
    })
}

fn retained_macro_invocations_mention_ident(
    project: &Project,
    _reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
    ident: &str,
) -> bool {
    let Some(source) = project
        .files
        .values()
        .find(|source| source.package == package && source.module_path == module_path)
    else {
        return false;
    };

    source.syntax.items.iter().any(|item| match item {
        Item::Macro(item_macro) if item_macro.ident.is_none() => {
            token_stream_mentions_ident(&item_macro.mac.tokens, ident)
        }
        _ => false,
    })
}

fn collect_retained_module_surface_idents(
    project: &Project,
    reduced: &ReducedProject,
    source: &SourceFile,
    idents: &mut BTreeSet<String>,
) {
    let aliases = project
        .module_aliases
        .get(&(source.package.clone(), source.module_path.clone()))
        .cloned()
        .unwrap_or_default();

    for item in &source.syntax.items {
        match item {
            Item::Impl(item_impl) => {
                let has_reachable_method = impl_has_reachable_method(
                    project,
                    reduced,
                    &source.package,
                    &source.module_path,
                    item_impl,
                    &aliases,
                );
                if has_reachable_method {
                    for attr in &item_impl.attrs {
                        collect_token_idents(&attr.to_token_stream(), idents);
                    }
                    for impl_item in &item_impl.items {
                        if !matches!(impl_item, ImplItem::Fn(_)) && !impl_item_is_test(impl_item) {
                            collect_token_idents(&impl_item.to_token_stream(), idents);
                        }
                    }
                }
                if item_impl.trait_.is_some()
                    && impl_should_render(
                        project,
                        reduced,
                        &source.package,
                        &source.module_path,
                        item_impl,
                        &aliases,
                    )
                {
                    for impl_item in &item_impl.items {
                        if !impl_item_is_test(impl_item) {
                            collect_token_idents(&impl_item.to_token_stream(), idents);
                        }
                    }
                }
            }
            Item::Macro(item_macro) if item_macro.ident.is_none() => {
                collect_token_idents(&item_macro.mac.tokens, idents);
            }
            _ => {}
        }
    }
}

fn impl_has_reachable_method(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
    item_impl: &syn::ItemImpl,
    aliases: &std::collections::HashMap<String, Vec<String>>,
) -> bool {
    let Some(type_path) =
        resolved_local_type_path(project, package, module_path, &item_impl.self_ty, aliases)
    else {
        return false;
    };
    let trait_path = item_impl
        .trait_
        .as_ref()
        .map(|(_, path, _)| normalized_path(module_path, path, aliases));
    let trait_input_type_paths = item_impl
        .trait_
        .as_ref()
        .map(|(_, path, _)| trait_input_type_paths(module_path, path, aliases))
        .unwrap_or_default();

    item_impl.items.iter().any(|impl_item| {
        let ImplItem::Fn(method) = impl_item else {
            return false;
        };
        let id = CallableId::Method {
            package: package.to_string(),
            type_path: type_path.clone(),
            trait_path: trait_path.clone(),
            trait_input_type_paths: trait_input_type_paths.clone(),
            method: method.sig.ident.to_string(),
        };
        reduced.reachable.contains(&id)
    })
}

fn impl_should_render(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
    item_impl: &syn::ItemImpl,
    aliases: &std::collections::HashMap<String, Vec<String>>,
) -> bool {
    if impl_has_reachable_method(project, reduced, package, module_path, item_impl, aliases) {
        return true;
    }
    let Some(type_path) =
        resolved_local_type_path(project, package, module_path, &item_impl.self_ty, aliases)
    else {
        return false;
    };
    let trait_path = item_impl
        .trait_
        .as_ref()
        .map(|(_, path, _)| normalized_path(module_path, path, aliases));
    let trait_impl_is_required = trait_path.as_ref().is_some_and(|trait_path| {
        trait_impl_items_are_reachable(reduced, package, &type_path, trait_path)
    });
    let marker_trait_impl_is_required = trait_path
        .as_ref()
        .and_then(|path| path.last())
        .is_some_and(|trait_name| {
            marker_trait_impl_should_remain(reduced, package, &type_path, trait_name)
        })
        && item_impl
            .items
            .iter()
            .filter(|impl_item| !impl_item_is_test(impl_item))
            .all(|impl_item| !matches!(impl_item, ImplItem::Fn(_)));

    trait_impl_is_required || marker_trait_impl_is_required
}

fn trait_impl_items_are_reachable(
    reduced: &ReducedProject,
    package: &str,
    type_path: &[String],
    trait_path: &[String],
) -> bool {
    path_item_is_reachable(
        reduced,
        package,
        type_path,
        &[
            ItemKind::Struct,
            ItemKind::Enum,
            ItemKind::Union,
            ItemKind::Type,
        ],
    ) && path_item_is_reachable(reduced, package, trait_path, &[ItemKind::Trait])
}

fn reachable_trait_impl_method_for_type_named(
    reduced: &ReducedProject,
    package: &str,
    type_path: &[String],
    trait_name: &str,
) -> bool {
    reduced.reachable.iter().any(|callable| {
        let CallableId::Method {
            package: callable_package,
            type_path: callable_type,
            trait_path: Some(trait_path),
            ..
        } = callable
        else {
            return false;
        };
        callable_package == package
            && callable_type == type_path
            && trait_path.last().is_some_and(|name| name == trait_name)
    })
}

fn marker_trait_impl_should_remain(
    reduced: &ReducedProject,
    package: &str,
    type_path: &[String],
    trait_name: &str,
) -> bool {
    if !path_item_is_reachable(
        reduced,
        package,
        type_path,
        &[
            ItemKind::Struct,
            ItemKind::Enum,
            ItemKind::Union,
            ItemKind::Type,
        ],
    ) {
        return false;
    }
    match trait_name {
        "Eq" => reachable_trait_impl_method_for_type_named(reduced, package, type_path, "Ord"),
        "Error" => true,
        "RefUnwindSafe" | "Send" | "Sync" | "Unpin" | "UnwindSafe" => true,
        _ => false,
    }
}

fn path_item_is_reachable(
    reduced: &ReducedProject,
    package: &str,
    path: &[String],
    kinds: &[ItemKind],
) -> bool {
    let Some((name, module_path)) = path.split_last() else {
        return false;
    };
    kinds.iter().any(|kind| {
        reduced.reachable_items.contains(&ItemId {
            package: package.to_string(),
            module_path: module_path.to_vec(),
            name: name.clone(),
            kind: *kind,
        })
    })
}

fn reachable_module_mentions_ident(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
    ident: &str,
) -> bool {
    reduced
        .reachable
        .iter()
        .filter(|callable| callable.package() == package)
        .any(|callable| {
            project.functions.get(callable).is_some_and(|record| {
                record.module_path == module_path
                    && (callable_mentions_ident(callable, ident)
                        || token_stream_mentions_ident(&record.item.to_token_stream(), ident))
            }) || project.methods.get(callable).is_some_and(|record| {
                record.module_path == module_path
                    && (callable_mentions_ident(callable, ident)
                        || token_stream_mentions_ident(&record.item.to_token_stream(), ident))
            })
        })
        || reduced
            .reachable_items
            .iter()
            .filter(|item| item.package() == package && item.module_path == module_path)
            .any(|item| {
                project.items.get(item).is_some_and(|record| {
                    reachable_item_mentions_ident(project, reduced, package, item, record, ident)
                })
            })
        || retained_impl_attrs_mention_ident(project, reduced, package, module_path, ident)
        || retained_impl_non_fn_items_mention_ident(project, reduced, package, module_path, ident)
        || retained_macro_invocations_mention_ident(project, reduced, package, module_path, ident)
}

fn reachable_module_has_method_call(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
    method: &str,
) -> bool {
    reduced
        .reachable
        .iter()
        .filter(|callable| callable.package() == package)
        .any(|callable| {
            project.functions.get(callable).is_some_and(|record| {
                record.module_path == module_path && item_fn_has_method_call(&record.item, method)
            }) || project.methods.get(callable).is_some_and(|record| {
                record.module_path == module_path
                    && impl_item_fn_has_method_call(&record.item, method)
            })
        })
        || reduced
            .reachable_items
            .iter()
            .filter(|item| item.package() == package && item.module_path == module_path)
            .any(|item| {
                project
                    .items
                    .get(item)
                    .is_some_and(|record| item_has_method_call(&record.item, method))
            })
}

fn reachable_module_has_associated_function_call(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
    function: &str,
) -> bool {
    reduced
        .reachable
        .iter()
        .filter(|callable| callable.package() == package)
        .any(|callable| {
            project.functions.get(callable).is_some_and(|record| {
                record.module_path == module_path
                    && item_fn_has_associated_function_call(&record.item, function)
            }) || project.methods.get(callable).is_some_and(|record| {
                record.module_path == module_path
                    && impl_item_fn_has_associated_function_call(&record.item, function)
            })
        })
        || reduced
            .reachable_items
            .iter()
            .filter(|item| item.package() == package && item.module_path == module_path)
            .any(|item| {
                project
                    .items
                    .get(item)
                    .is_some_and(|record| item_has_associated_function_call(&record.item, function))
            })
}

fn item_fn_has_method_call(item: &syn::ItemFn, method: &str) -> bool {
    let mut visitor = MethodCallVisitor::new(method);
    visitor.visit_item_fn(item);
    visitor.found
}

fn impl_item_fn_has_method_call(item: &syn::ImplItemFn, method: &str) -> bool {
    let mut visitor = MethodCallVisitor::new(method);
    visitor.visit_impl_item_fn(item);
    visitor.found
}

fn item_has_method_call(item: &Item, method: &str) -> bool {
    let mut visitor = MethodCallVisitor::new(method);
    visitor.visit_item(item);
    visitor.found
}

fn item_fn_has_associated_function_call(item: &syn::ItemFn, function: &str) -> bool {
    let mut visitor = AssociatedFunctionCallVisitor::new(function);
    visitor.visit_item_fn(item);
    visitor.found
}

fn impl_item_fn_has_associated_function_call(item: &syn::ImplItemFn, function: &str) -> bool {
    let mut visitor = AssociatedFunctionCallVisitor::new(function);
    visitor.visit_impl_item_fn(item);
    visitor.found
}

fn item_has_associated_function_call(item: &Item, function: &str) -> bool {
    let mut visitor = AssociatedFunctionCallVisitor::new(function);
    visitor.visit_item(item);
    visitor.found
}

struct MethodCallVisitor<'a> {
    method: &'a str,
    found: bool,
}

impl<'a> MethodCallVisitor<'a> {
    fn new(method: &'a str) -> Self {
        Self {
            method,
            found: false,
        }
    }
}

impl Visit<'_> for MethodCallVisitor<'_> {
    fn visit_expr_method_call(&mut self, node: &syn::ExprMethodCall) {
        if node.method == self.method {
            self.found = true;
            return;
        }
        visit::visit_expr_method_call(self, node);
    }
}

struct AssociatedFunctionCallVisitor<'a> {
    function: &'a str,
    found: bool,
}

impl<'a> AssociatedFunctionCallVisitor<'a> {
    fn new(function: &'a str) -> Self {
        Self {
            function,
            found: false,
        }
    }
}

impl Visit<'_> for AssociatedFunctionCallVisitor<'_> {
    fn visit_expr_call(&mut self, node: &syn::ExprCall) {
        if let syn::Expr::Path(path) = node.func.as_ref() {
            if path.path.segments.len() > 1
                && path
                    .path
                    .segments
                    .last()
                    .is_some_and(|segment| segment.ident == self.function)
            {
                self.found = true;
                return;
            }
        }
        visit::visit_expr_call(self, node);
    }
}

fn reachable_item_mentions_ident(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    item_id: &ItemId,
    record: &crate::model::ItemRecord,
    ident: &str,
) -> bool {
    if let Item::Trait(item_trait) = &record.item {
        if trait_has_reachable_impl_methods(reduced, package, &item_id.module_path, &item_id.name) {
            return token_stream_mentions_ident(&record.item.to_token_stream(), ident);
        }
        return trait_type_surface_mentions_ident(item_id, item_trait, ident);
    }

    let Item::Struct(item_struct) = &record.item else {
        return token_stream_mentions_ident(&record.item.to_token_stream(), ident);
    };
    if item_id.name == ident || item_id.module_path.iter().any(|segment| segment == ident) {
        return true;
    }
    if item_struct
        .attrs
        .iter()
        .any(|attr| token_stream_mentions_ident(&attr.to_token_stream(), ident))
    {
        return true;
    }
    if root_item_should_render(reduced, item_id) {
        return token_stream_mentions_ident(&record.item.to_token_stream(), ident);
    }

    let syn::Fields::Named(fields) = &item_struct.fields else {
        return token_stream_mentions_ident(&record.item.to_token_stream(), ident);
    };
    fields.named.iter().any(|field| {
        struct_field_should_remain(project, reduced, None, package, item_struct, field)
            && token_stream_mentions_ident(&field.to_token_stream(), ident)
    })
}

fn trait_type_surface_mentions_ident(
    item_id: &ItemId,
    item_trait: &syn::ItemTrait,
    ident: &str,
) -> bool {
    if item_id.name == ident || item_id.module_path.iter().any(|segment| segment == ident) {
        return true;
    }
    if item_trait
        .supertraits
        .iter()
        .any(|bound| token_stream_mentions_ident(&bound.to_token_stream(), ident))
    {
        return true;
    }
    item_trait
        .attrs
        .iter()
        .filter(|attr| is_inert_type_surface_attr(attr))
        .any(|attr| token_stream_mentions_ident(&attr.to_token_stream(), ident))
}

fn reachable_callables_mention_ident(
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
}

fn reachable_module_import_scope_mentions_ident(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
    ident: &str,
) -> bool {
    reachable_module_import_scope_mentions_ident_excluding(
        project,
        reduced,
        render_plan,
        package,
        module_path,
        ident,
        None,
    )
}

fn reachable_module_import_scope_mentions_ident_excluding(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
    ident: &str,
    excluded_module_path: Option<&[String]>,
) -> bool {
    let key = ImportScopeMentionKey::new(package, module_path, ident, excluded_module_path);
    if let Some(value) = render_plan
        .import_scope_mentions
        .borrow()
        .get(&key)
        .copied()
    {
        return value;
    }
    let value = reachable_module_import_scope_mentions_ident_excluding_uncached(
        project,
        reduced,
        render_plan,
        package,
        module_path,
        ident,
        excluded_module_path,
    );
    render_plan
        .import_scope_mentions
        .borrow_mut()
        .insert(key, value);
    value
}

fn reachable_module_import_scope_mentions_ident_excluding_uncached(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
    ident: &str,
    excluded_module_path: Option<&[String]>,
) -> bool {
    if render_plan.module_mentions_ident(package, module_path, ident) {
        return true;
    }

    if project
        .files
        .values()
        .find(|source| source.package == package && source.module_path == module_path)
        .is_some_and(|source| {
            inline_child_modules_import_scope_mentions_ident(
                project,
                reduced,
                render_plan,
                package,
                module_path,
                &source.syntax.items,
                ident,
                excluded_module_path,
            )
        })
    {
        return true;
    }

    project
        .files
        .values()
        .filter(|source| source.package == package)
        .filter(|source| source.module_path.len() == module_path.len() + 1)
        .filter(|source| path_has_prefix(&source.module_path, module_path))
        .filter(|source| excluded_module_path != Some(source.module_path.as_slice()))
        .filter(|source| {
            module_should_render(project, reduced, render_plan, package, &source.module_path)
        })
        .any(|source| {
            child_module_import_scope_mentions_parent_ident(
                project,
                reduced,
                render_plan,
                package,
                source.module_path.as_slice(),
                &source.syntax.items,
                ident,
                excluded_module_path,
            )
        })
}

fn items_have_super_glob_import(items: &[Item]) -> bool {
    items.iter().any(|item| {
        let Item::Use(item_use) = item else {
            return false;
        };
        use_tree_has_super_glob_import(&item_use.tree)
    })
}

#[allow(clippy::too_many_arguments)]
fn inline_child_modules_import_scope_mentions_ident(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
    items: &[Item],
    ident: &str,
    excluded_module_path: Option<&[String]>,
) -> bool {
    items.iter().any(|item| {
        let Item::Mod(item_mod) = item else {
            return false;
        };
        let Some((_, child_items)) = &item_mod.content else {
            return false;
        };

        let mut child_path = module_path.to_vec();
        child_path.push(item_mod.ident.to_string());
        if excluded_module_path.is_some_and(|excluded| child_path == excluded) {
            return false;
        }
        if !module_should_render(project, reduced, render_plan, package, &child_path) {
            return false;
        }

        child_module_import_scope_mentions_parent_ident(
            project,
            reduced,
            render_plan,
            package,
            &child_path,
            child_items,
            ident,
            excluded_module_path,
        )
    })
}

#[allow(clippy::too_many_arguments)]
fn child_module_import_scope_mentions_parent_ident(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    child_module_path: &[String],
    child_items: &[Item],
    ident: &str,
    excluded_module_path: Option<&[String]>,
) -> bool {
    let visible_names = super_import_visible_names_for_parent_ident(child_items, ident);
    if visible_names.iter().any(|visible_name| {
        reachable_module_import_scope_mentions_ident_excluding(
            project,
            reduced,
            render_plan,
            package,
            child_module_path,
            visible_name,
            excluded_module_path,
        )
    }) {
        return true;
    }

    items_have_super_glob_import(child_items)
        && reachable_module_import_scope_mentions_ident_excluding(
            project,
            reduced,
            render_plan,
            package,
            child_module_path,
            ident,
            excluded_module_path,
        )
}

fn super_import_visible_names_for_parent_ident(items: &[Item], ident: &str) -> BTreeSet<String> {
    let mut visible_names = BTreeSet::new();
    for item in items {
        let Item::Use(item_use) = item else {
            continue;
        };
        collect_super_import_visible_names(&item_use.tree, Vec::new(), ident, &mut visible_names);
    }
    visible_names
}

fn collect_super_import_visible_names(
    tree: &UseTree,
    mut prefix: Vec<String>,
    ident: &str,
    visible_names: &mut BTreeSet<String>,
) {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            collect_super_import_visible_names(&path.tree, prefix, ident, visible_names);
        }
        UseTree::Name(name) => {
            let visible_name = if name.ident == "self" {
                prefix
                    .last()
                    .cloned()
                    .unwrap_or_else(|| name.ident.to_string())
            } else {
                name.ident.to_string()
            };
            prefix.push(name.ident.to_string());
            if use_path_imports_parent_ident(&prefix, ident) {
                visible_names.insert(visible_name);
            }
        }
        UseTree::Rename(rename) => {
            prefix.push(rename.ident.to_string());
            if use_path_imports_parent_ident(&prefix, ident) {
                visible_names.insert(rename.rename.to_string());
            }
        }
        UseTree::Group(group) => {
            for nested in &group.items {
                collect_super_import_visible_names(nested, prefix.clone(), ident, visible_names);
            }
        }
        UseTree::Glob(_) => {}
    }
}

fn use_path_imports_parent_ident(path: &[String], ident: &str) -> bool {
    path.first().is_some_and(|first| first == "super")
        && path.get(1).is_some_and(|candidate| candidate == ident)
}

fn use_tree_has_super_glob_import(tree: &UseTree) -> bool {
    match tree {
        UseTree::Path(path) if path.ident == "super" => use_tree_contains_glob(&path.tree),
        UseTree::Path(path) => use_tree_has_super_glob_import(&path.tree),
        UseTree::Group(group) => group.items.iter().any(use_tree_has_super_glob_import),
        UseTree::Name(_) | UseTree::Rename(_) | UseTree::Glob(_) => false,
    }
}

fn use_tree_contains_glob(tree: &UseTree) -> bool {
    match tree {
        UseTree::Glob(_) => true,
        UseTree::Path(path) => use_tree_contains_glob(&path.tree),
        UseTree::Group(group) => group.items.iter().any(use_tree_contains_glob),
        UseTree::Name(_) | UseTree::Rename(_) => false,
    }
}

fn module_glob_is_used_in_module(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
    target_package: &str,
    target_path: &[String],
) -> bool {
    project.functions.keys().any(|callable| {
        let CallableId::Free {
            package: callable_package,
            module_path: callable_module,
            name,
        } = callable
        else {
            return false;
        };
        callable_package == target_package
            && callable_module == target_path
            && reduced.reachable.contains(callable)
            && reachable_module_import_scope_mentions_ident_excluding(
                project,
                reduced,
                render_plan,
                package,
                module_path,
                name,
                Some(target_path),
            )
    }) || project.items.keys().any(|item| {
        item.package == target_package
            && item.module_path == target_path
            && render_plan.item_should_render(item)
            && reachable_module_import_scope_mentions_ident_excluding(
                project,
                reduced,
                render_plan,
                package,
                module_path,
                &item.name,
                Some(target_path),
            )
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

fn strip_uniffi_attrs(attrs: &mut Vec<syn::Attribute>) {
    let mut retained = Vec::new();
    for mut attr in std::mem::take(attrs) {
        if attr_path_starts_with(attr.path(), "uniffi") {
            continue;
        }
        if attr.path().is_ident("cfg_attr")
            && token_stream_mentions_ident(&attr.to_token_stream(), "uniffi")
        {
            continue;
        }
        if attr.path().is_ident("derive") {
            if let Ok(paths) =
                attr.parse_args_with(Punctuated::<syn::Path, syn::Token![,]>::parse_terminated)
            {
                let paths = paths
                    .into_iter()
                    .filter(|path| !path_starts_with(path, "uniffi"))
                    .collect::<Vec<_>>();
                if paths.is_empty() {
                    continue;
                }
                attr = parse_quote!(#[derive(#(#paths),*)]);
            }
        }
        retained.push(attr);
    }
    *attrs = retained;
}

fn strip_opensourced_attrs_from_impl_item(item: &mut ImplItem) {
    if let ImplItem::Fn(method) = item {
        strip_opensourced_attrs(&mut method.attrs);
    }
}

fn strip_uniffi_attrs_from_impl_item(item: &mut ImplItem) {
    match item {
        ImplItem::Const(item) => strip_uniffi_attrs(&mut item.attrs),
        ImplItem::Fn(item) => strip_uniffi_attrs(&mut item.attrs),
        ImplItem::Macro(item) => strip_uniffi_attrs(&mut item.attrs),
        ImplItem::Type(item) => strip_uniffi_attrs(&mut item.attrs),
        ImplItem::Verbatim(_) => {}
        _ => {}
    }
}

fn strip_opensourced_attrs_from_item(item: &mut Item) {
    match item {
        Item::Const(item) => strip_opensourced_attrs(&mut item.attrs),
        Item::Enum(item) => strip_opensourced_attrs(&mut item.attrs),
        Item::Macro(item) => strip_opensourced_attrs(&mut item.attrs),
        Item::Mod(item) => strip_opensourced_attrs(&mut item.attrs),
        Item::Static(item) => strip_opensourced_attrs(&mut item.attrs),
        Item::Struct(item) => strip_opensourced_attrs(&mut item.attrs),
        Item::Trait(item) => {
            strip_opensourced_attrs(&mut item.attrs);
            for trait_item in &mut item.items {
                strip_opensourced_attrs_from_trait_item(trait_item);
            }
        }
        Item::Type(item) => strip_opensourced_attrs(&mut item.attrs),
        Item::Union(item) => strip_opensourced_attrs(&mut item.attrs),
        _ => {}
    }
}

fn strip_uniffi_attrs_from_item(item: &mut Item) {
    match item {
        Item::Const(item) => strip_uniffi_attrs(&mut item.attrs),
        Item::Enum(item) => {
            strip_uniffi_attrs(&mut item.attrs);
            for variant in &mut item.variants {
                strip_uniffi_attrs_from_variant(variant);
            }
        }
        Item::Fn(item) => strip_uniffi_attrs(&mut item.attrs),
        Item::Impl(item) => {
            strip_uniffi_attrs(&mut item.attrs);
            for impl_item in &mut item.items {
                strip_uniffi_attrs_from_impl_item(impl_item);
            }
        }
        Item::Macro(item) => strip_uniffi_attrs(&mut item.attrs),
        Item::Mod(item) => strip_uniffi_attrs(&mut item.attrs),
        Item::Static(item) => strip_uniffi_attrs(&mut item.attrs),
        Item::Struct(item) => {
            strip_uniffi_attrs(&mut item.attrs);
            strip_uniffi_attrs_from_fields(&mut item.fields);
        }
        Item::Trait(item) => {
            strip_uniffi_attrs(&mut item.attrs);
            for trait_item in &mut item.items {
                strip_uniffi_attrs_from_trait_item(trait_item);
            }
        }
        Item::Type(item) => strip_uniffi_attrs(&mut item.attrs),
        Item::Union(item) => {
            strip_uniffi_attrs(&mut item.attrs);
            for field in &mut item.fields.named {
                strip_uniffi_attrs_from_field(field);
            }
        }
        _ => {}
    }
}

fn strip_uniffi_attrs_from_fields(fields: &mut syn::Fields) {
    match fields {
        syn::Fields::Named(fields) => {
            for field in &mut fields.named {
                strip_uniffi_attrs_from_field(field);
            }
        }
        syn::Fields::Unnamed(fields) => {
            for field in &mut fields.unnamed {
                strip_uniffi_attrs_from_field(field);
            }
        }
        syn::Fields::Unit => {}
    }
}

fn strip_uniffi_attrs_from_field(field: &mut Field) {
    strip_uniffi_attrs(&mut field.attrs);
}

fn strip_uniffi_attrs_from_variant(variant: &mut Variant) {
    strip_uniffi_attrs(&mut variant.attrs);
    strip_uniffi_attrs_from_fields(&mut variant.fields);
}

fn strip_opensourced_attrs_from_trait_item(item: &mut TraitItem) {
    match item {
        TraitItem::Const(item) => strip_opensourced_attrs(&mut item.attrs),
        TraitItem::Fn(item) => strip_opensourced_attrs(&mut item.attrs),
        TraitItem::Macro(item) => strip_opensourced_attrs(&mut item.attrs),
        TraitItem::Type(item) => strip_opensourced_attrs(&mut item.attrs),
        TraitItem::Verbatim(_) => {}
        _ => {}
    }
}

fn strip_uniffi_attrs_from_trait_item(item: &mut TraitItem) {
    match item {
        TraitItem::Const(item) => strip_uniffi_attrs(&mut item.attrs),
        TraitItem::Fn(item) => strip_uniffi_attrs(&mut item.attrs),
        TraitItem::Macro(item) => strip_uniffi_attrs(&mut item.attrs),
        TraitItem::Type(item) => strip_uniffi_attrs(&mut item.attrs),
        TraitItem::Verbatim(_) => {}
        _ => {}
    }
}

fn downgrade_uniffi_async_runtime_if_no_async_methods(item_impl: &mut syn::ItemImpl) {
    if impl_items_contain_async_method(&item_impl.items) {
        return;
    }

    for attr in &mut item_impl.attrs {
        if is_uniffi_export_async_runtime_attr(attr) {
            *attr = parse_quote!(#[uniffi::export]);
        }
    }
}

fn impl_items_contain_async_method(items: &[ImplItem]) -> bool {
    items
        .iter()
        .any(|item| matches!(item, ImplItem::Fn(method) if method.sig.asyncness.is_some()))
}

fn is_uniffi_export_async_runtime_attr(attr: &syn::Attribute) -> bool {
    attr.path().segments.len() == 2
        && attr.path().segments[0].ident == "uniffi"
        && attr.path().segments[1].ident == "export"
        && attr.to_token_stream().to_string().contains("async_runtime")
}

fn allow_dead_code_if_not_public(vis: &syn::Visibility, attrs: &mut Vec<syn::Attribute>) {
    if matches!(vis, syn::Visibility::Public(_)) {
        return;
    }
    allow_dead_code(attrs);
}

fn allow_dead_code_for_private_struct_fields(item_struct: &mut syn::ItemStruct) {
    if struct_has_private_fields(item_struct) {
        allow_dead_code(&mut item_struct.attrs);
    }
}

fn allow_dead_code(attrs: &mut Vec<syn::Attribute>) {
    if attrs.iter().any(|attr| {
        attr.path().is_ident("allow")
            && token_stream_mentions_ident(&attr.to_token_stream(), "dead_code")
    }) {
        return;
    }
    attrs.push(parse_quote!(#[allow(dead_code)]));
}

fn struct_has_private_fields(item_struct: &syn::ItemStruct) -> bool {
    match &item_struct.fields {
        syn::Fields::Named(fields) => fields
            .named
            .iter()
            .any(|field| !matches!(field.vis, syn::Visibility::Public(_))),
        syn::Fields::Unnamed(fields) => fields
            .unnamed
            .iter()
            .any(|field| !matches!(field.vis, syn::Visibility::Public(_))),
        syn::Fields::Unit => false,
    }
}

fn allow_dead_code_for_non_public_item(item: &mut Item) {
    match item {
        Item::Const(item) => allow_dead_code_if_not_public(&item.vis, &mut item.attrs),
        Item::Enum(item) => allow_dead_code_if_not_public(&item.vis, &mut item.attrs),
        Item::Static(item) => allow_dead_code_if_not_public(&item.vis, &mut item.attrs),
        Item::Trait(item) => allow_dead_code_if_not_public(&item.vis, &mut item.attrs),
        Item::Type(item) => allow_dead_code_if_not_public(&item.vis, &mut item.attrs),
        Item::Union(item) => allow_dead_code_if_not_public(&item.vis, &mut item.attrs),
        _ => {}
    }
}

fn allow_dead_code_for_non_public_impl_item(item: &mut ImplItem) {
    if let ImplItem::Fn(method) = item {
        allow_dead_code_if_not_public(&method.vis, &mut method.attrs);
    }
}

fn prune_private_struct_fields(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    item_struct: &mut syn::ItemStruct,
    preserve_private_fields: bool,
) {
    if preserve_private_fields {
        return;
    }
    let original_struct = item_struct.clone();
    let syn::Fields::Named(fields) = &mut item_struct.fields else {
        return;
    };

    fields.named = fields
        .named
        .iter()
        .filter(|field| {
            struct_field_should_remain(
                project,
                reduced,
                Some(render_plan),
                package,
                &original_struct,
                field,
            )
        })
        .cloned()
        .collect();
}

fn struct_field_should_remain(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: Option<&RenderPlan>,
    package: &str,
    item_struct: &syn::ItemStruct,
    field: &Field,
) -> bool {
    if matches!(item_struct.vis, syn::Visibility::Public(_))
        && matches!(field.vis, syn::Visibility::Public(_))
    {
        return true;
    }
    if field_mentions_struct_type_params(item_struct, field) {
        return true;
    }
    if field_attrs_require_field(field) {
        return true;
    }
    let Some(name) = field.ident.as_ref() else {
        return true;
    };
    let name = name.to_string();
    if let Some(render_plan) = render_plan {
        render_plan.package_callable_mentions_ident(package, &name)
    } else {
        reachable_callables_mention_ident(project, reduced, package, &name)
    }
}

fn field_attrs_require_field(field: &Field) -> bool {
    field.attrs.iter().any(|attr| {
        let path = attr.path();
        !(path.is_ident("cfg")
            || path.is_ident("allow")
            || path.is_ident("deny")
            || path.is_ident("doc")
            || path.is_ident("deprecated"))
    })
}

fn field_mentions_struct_type_params(item_struct: &syn::ItemStruct, field: &Field) -> bool {
    item_struct.generics.type_params().any(|param| {
        token_stream_mentions_ident(&field.to_token_stream(), &param.ident.to_string())
    })
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

fn use_is_reexport(vis: &syn::Visibility) -> bool {
    !matches!(vis, syn::Visibility::Inherited)
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

#[allow(clippy::too_many_arguments)]
fn prune_use_tree(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
    tree: &UseTree,
    mut prefix: Vec<String>,
    is_public_use: bool,
) -> Option<UseTree> {
    match tree {
        UseTree::Path(path) => {
            if prefix.is_empty()
                && use_ident_is_pruned_local_dependency(project, reduced, package, &path.ident)
                && !known_macro_dependency_package_mentions(
                    project,
                    reduced,
                    package,
                    &dependency_code_name(&path.ident.to_string()),
                )
            {
                return None;
            }
            prefix.push(path.ident.to_string());
            let mut path = path.clone();
            path.tree = Box::new(prune_use_tree(
                project,
                reduced,
                render_plan,
                package,
                module_path,
                &path.tree,
                prefix,
                is_public_use,
            )?);
            Some(UseTree::Path(path))
        }
        UseTree::Name(name) => {
            let visible_name = if name.ident == "self" {
                prefix
                    .last()
                    .cloned()
                    .unwrap_or_else(|| name.ident.to_string())
            } else {
                name.ident.to_string()
            };
            prefix.push(name.ident.to_string());
            (!use_target_should_drop(
                project,
                reduced,
                render_plan,
                package,
                module_path,
                &prefix,
                is_public_use,
            ) || (is_public_use
                && public_reexport_name_is_referenced_by_reduced_package(
                    project,
                    reduced,
                    package,
                    &visible_name,
                )))
            .then(|| UseTree::Name(name.clone()))
        }
        UseTree::Rename(rename) => {
            prefix.push(rename.ident.to_string());
            let alias = rename.rename.to_string();
            (!use_target_should_drop(
                project,
                reduced,
                render_plan,
                package,
                module_path,
                &prefix,
                is_public_use,
            ) || renamed_use_alias_is_reachable(
                project,
                reduced,
                render_plan,
                package,
                module_path,
                &alias,
                is_public_use,
            ) || (is_public_use
                && (reachable_reduced_packages_mention_ident(project, reduced, &alias)
                    || public_reexport_name_is_referenced_by_reduced_package(
                        project, reduced, package, &alias,
                    ))))
            .then(|| UseTree::Rename(rename.clone()))
        }
        UseTree::Group(group) => {
            let mut group = group.clone();
            group.items = group
                .items
                .iter()
                .filter_map(|item| {
                    prune_use_tree(
                        project,
                        reduced,
                        render_plan,
                        package,
                        module_path,
                        item,
                        prefix.clone(),
                        is_public_use,
                    )
                })
                .collect();
            (!group.items.is_empty()).then_some(UseTree::Group(group))
        }
        UseTree::Glob(glob) => (!use_prefix_should_drop(
            project,
            reduced,
            render_plan,
            package,
            module_path,
            &prefix,
            is_public_use,
        ))
        .then(|| UseTree::Glob(glob.clone())),
    }
}

fn renamed_use_alias_is_reachable(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
    alias: &str,
    is_public_use: bool,
) -> bool {
    if reachable_module_import_scope_mentions_ident(
        project,
        reduced,
        render_plan,
        package,
        module_path,
        alias,
    ) {
        return true;
    }

    is_public_use
        && (render_plan.package_mentions_ident(package, alias)
            || reachable_package_mentions_ident(project, reduced, package, alias))
}

fn public_reexport_name_is_referenced_by_reduced_package(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    visible_name: &str,
) -> bool {
    reduced
        .reachable
        .iter()
        .filter(|callable| callable.package() != package)
        .any(|callable| {
            project.functions.get(callable).is_some_and(|record| {
                token_stream_mentions_dependency_public_name(
                    project,
                    &record.package,
                    package,
                    visible_name,
                    &record.item.to_token_stream(),
                )
            }) || project.methods.get(callable).is_some_and(|record| {
                token_stream_mentions_dependency_public_name(
                    project,
                    callable.package(),
                    package,
                    visible_name,
                    &record.item.to_token_stream(),
                )
            })
        })
        || reduced
            .reachable_items
            .iter()
            .filter(|item| item.package() != package)
            .any(|item| {
                project.items.get(item).is_some_and(|record| {
                    token_stream_mentions_dependency_public_name(
                        project,
                        &record.package,
                        package,
                        visible_name,
                        &record.item.to_token_stream(),
                    )
                })
            })
}

fn use_target_should_drop(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
    target: &[String],
    is_public_use: bool,
) -> bool {
    if known_macro_dependency_target_should_remain(project, reduced, package, target) {
        return false;
    }

    if target
        .first()
        .is_some_and(|first| use_name_is_pruned_local_dependency(project, reduced, package, first))
    {
        return !known_macro_dependency_target_should_remain(project, reduced, package, target);
    }

    if target
        .first()
        .is_some_and(|first| use_name_is_external_dependency(project, package, first))
    {
        return external_use_target_should_drop(
            project,
            reduced,
            render_plan,
            package,
            module_path,
            target,
            is_public_use,
        );
    }

    if target
        .first()
        .is_some_and(|first| matches!(first.as_str(), "std" | "core" | "alloc"))
    {
        return external_use_target_should_drop(
            project,
            reduced,
            render_plan,
            package,
            module_path,
            target,
            is_public_use,
        );
    }

    let Some((target_package, target_path)) =
        resolve_use_target_path(project, package, module_path, target)
    else {
        return false;
    };

    let leaf_is_used_in_module = target.last().is_some_and(|leaf| {
        reachable_module_import_scope_mentions_ident(
            project,
            reduced,
            render_plan,
            package,
            module_path,
            leaf,
        )
    });

    if let Some(callable) = find_use_function(project, &target_package, &target_path) {
        if !is_public_use && !leaf_is_used_in_module {
            return true;
        }
        return !reduced.reachable.contains(&callable);
    }
    if let Some(item) = find_use_item(project, &target_package, &target_path) {
        if !is_public_use && item.kind != ItemKind::Trait && !leaf_is_used_in_module {
            return true;
        }
        if (is_public_use || leaf_is_used_in_module) && item.kind == ItemKind::Mod {
            return false;
        }
        return !render_plan.item_should_render(&item);
    }
    if let Some((alias_package, alias_path)) =
        resolve_reexported_use_path(project, &target_package, &target_path)
    {
        if let Some(callable) = find_use_function(project, &alias_package, &alias_path) {
            if !is_public_use && !leaf_is_used_in_module {
                return true;
            }
            return !reduced.reachable.contains(&callable);
        }
        if let Some(item) = find_use_item(project, &alias_package, &alias_path) {
            if !is_public_use && item.kind != ItemKind::Trait && !leaf_is_used_in_module {
                return true;
            }
            if (is_public_use || leaf_is_used_in_module) && item.kind == ItemKind::Mod {
                return false;
            }
            return !render_plan.item_should_render(&item);
        }
        return project_has_module(project, &alias_package, &alias_path)
            && !module_should_render(project, reduced, render_plan, &alias_package, &alias_path);
    }

    if project_has_module(project, &target_package, &target_path) {
        return !module_should_render(project, reduced, render_plan, &target_package, &target_path);
    }

    if target.last().is_some_and(|leaf| {
        render_plan.reachable_items.iter().any(|item| {
            item.package == package && item.kind == ItemKind::Trait && item.name == *leaf
        })
    }) {
        return false;
    }

    target.last().is_some_and(|leaf| {
        if is_public_use {
            !reachable_package_mentions_ident(project, reduced, package, leaf)
        } else {
            !reachable_module_import_scope_mentions_ident(
                project,
                reduced,
                render_plan,
                package,
                module_path,
                leaf,
            )
        }
    })
}

fn use_prefix_should_drop(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
    prefix: &[String],
    is_public_use: bool,
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
        return external_use_target_should_drop(
            project,
            reduced,
            render_plan,
            package,
            module_path,
            prefix,
            is_public_use,
        );
    }

    if prefix
        .first()
        .is_some_and(|first| matches!(first.as_str(), "std" | "core" | "alloc"))
    {
        return external_use_target_should_drop(
            project,
            reduced,
            render_plan,
            package,
            module_path,
            prefix,
            is_public_use,
        );
    }

    resolve_use_target_path(project, package, module_path, prefix).is_some_and(
        |(target_package, target_path)| {
            if !project_has_module(project, &target_package, &target_path) {
                return false;
            }
            if !module_should_render(project, reduced, render_plan, &target_package, &target_path) {
                return true;
            }
            !is_public_use
                && prefix.first().is_none_or(|first| first != "super")
                && !module_glob_is_used_in_module(
                    project,
                    reduced,
                    render_plan,
                    package,
                    module_path,
                    &target_package,
                    &target_path,
                )
        },
    )
}

fn external_use_target_should_drop(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    _package: &str,
    module_path: &[String],
    target: &[String],
    is_public_use: bool,
) -> bool {
    let Some(leaf) = external_use_leaf(target) else {
        return false;
    };

    if is_derive_only_external_trait_import(leaf) {
        return !external_trait_import_should_remain(
            project,
            reduced,
            render_plan,
            _package,
            module_path,
            target,
            is_public_use,
        );
    }

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

    if external_trait_import_should_remain(
        project,
        reduced,
        render_plan,
        _package,
        module_path,
        target,
        is_public_use,
    ) {
        return false;
    }

    if is_public_use {
        !render_plan.package_mentions_ident(_package, leaf)
    } else {
        !reachable_module_import_scope_mentions_ident(
            project,
            reduced,
            render_plan,
            _package,
            module_path,
            leaf,
        )
    }
}

fn external_use_leaf(target: &[String]) -> Option<&str> {
    let leaf = target.last()?;
    if leaf == "self" && target.len() >= 2 {
        return target.get(target.len() - 2).map(String::as_str);
    }
    Some(leaf)
}

fn is_external_trait_import_candidate(target: &[String], leaf: &str) -> bool {
    if leaf.ends_with("Ext") {
        return true;
    }

    if target
        .first()
        .is_some_and(|first| matches!(first.as_str(), "std" | "core" | "alloc"))
    {
        return is_known_std_trait_import(leaf);
    }

    is_known_external_trait_import(leaf) || is_probable_external_trait_import(target, leaf)
}

fn is_probable_external_trait_import(target: &[String], leaf: &str) -> bool {
    target
        .first()
        .is_some_and(|first| !matches!(first.as_str(), "crate" | "self" | "super"))
        && leaf
            .chars()
            .next()
            .is_some_and(|first| first.is_ascii_uppercase())
}

fn is_known_std_trait_import(leaf: &str) -> bool {
    matches!(
        leaf,
        "Add"
            | "AddAssign"
            | "AsMut"
            | "AsRef"
            | "Borrow"
            | "BorrowMut"
            | "BufRead"
            | "Clone"
            | "Debug"
            | "Default"
            | "Deref"
            | "DerefMut"
            | "Display"
            | "Div"
            | "DivAssign"
            | "Drop"
            | "Eq"
            | "Error"
            | "Extend"
            | "Fn"
            | "FnMut"
            | "FnOnce"
            | "From"
            | "FromIterator"
            | "FromStr"
            | "Future"
            | "Hash"
            | "Hasher"
            | "Index"
            | "IndexMut"
            | "Into"
            | "IntoIterator"
            | "Iterator"
            | "Mul"
            | "MulAssign"
            | "Neg"
            | "Not"
            | "Ord"
            | "PartialEq"
            | "PartialOrd"
            | "Product"
            | "Read"
            | "Rem"
            | "RemAssign"
            | "Seek"
            | "Send"
            | "Shr"
            | "ShrAssign"
            | "Sub"
            | "SubAssign"
            | "Sum"
            | "Sync"
            | "TryFrom"
            | "TryInto"
            | "Write"
    )
}

fn is_known_external_trait_import(leaf: &str) -> bool {
    matches!(leaf, "Deserialize" | "Digest" | "Engine" | "Serialize")
}

fn external_trait_import_should_remain(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
    target: &[String],
    is_public_use: bool,
) -> bool {
    let Some(leaf) = target.last().map(String::as_str) else {
        return false;
    };
    if leaf.ends_with("Ext") {
        if is_public_use {
            return true;
        }
        if render_plan.module_mentions_ident(package, module_path, leaf)
            || reachable_module_mentions_ident(project, reduced, package, module_path, leaf)
        {
            return true;
        }
        if known_trait_associated_function_idents(target, leaf).is_some_and(|functions| {
            functions.iter().any(|function| {
                reachable_module_has_associated_function_call(
                    project,
                    reduced,
                    package,
                    module_path,
                    function,
                )
            })
        }) {
            return true;
        }
        if target
            .first()
            .is_some_and(|first| matches!(first.as_str(), "std" | "core" | "alloc"))
        {
            return true;
        }
        let method_candidates;
        let methods = if let Some(methods) = known_trait_method_idents(target, leaf) {
            methods.to_vec()
        } else {
            method_candidates = generic_trait_method_name_candidates(leaf);
            method_candidates.iter().map(String::as_str).collect()
        };
        return methods.iter().any(|method| {
            reachable_module_has_method_call(project, reduced, package, module_path, method)
        });
    }
    if !is_external_trait_import_candidate(target, leaf) {
        return false;
    }
    if is_derive_only_external_trait_import(leaf) {
        if render_plan.module_mentions_ident(package, module_path, leaf)
            || reachable_module_mentions_ident(project, reduced, package, module_path, leaf)
        {
            return true;
        }
        let Some(functions) = known_trait_associated_function_idents(target, leaf) else {
            return false;
        };
        if is_public_use {
            return functions.iter().any(|function| {
                render_plan.package_mentions_ident(package, function)
                    || reachable_package_mentions_ident(project, reduced, package, function)
            });
        }
        return functions.iter().any(|function| {
            reachable_module_has_associated_function_call(
                project,
                reduced,
                package,
                module_path,
                function,
            )
        });
    }
    if !is_public_use {
        if render_plan.module_mentions_ident(package, module_path, leaf)
            || reachable_module_mentions_ident(project, reduced, package, module_path, leaf)
        {
            return true;
        }
        let receiver_is_reachable =
            known_trait_receiver_idents(target, leaf).is_none_or(|receivers| {
                receivers.iter().any(|receiver| {
                    render_plan.module_mentions_ident(package, module_path, receiver)
                        || reachable_module_mentions_ident(
                            project,
                            reduced,
                            package,
                            module_path,
                            receiver,
                        )
                })
            });
        if !receiver_is_reachable {
            return false;
        }
        if known_trait_associated_function_idents(target, leaf).is_some_and(|functions| {
            functions.iter().any(|function| {
                reachable_module_has_associated_function_call(
                    project,
                    reduced,
                    package,
                    module_path,
                    function,
                )
            })
        }) {
            return true;
        }
        let method_candidates;
        let methods = if let Some(methods) = known_trait_method_idents(target, leaf) {
            methods.to_vec()
        } else {
            method_candidates = generic_trait_method_name_candidates(leaf);
            method_candidates.iter().map(String::as_str).collect()
        };
        return methods.iter().any(|method| {
            reachable_module_has_method_call(project, reduced, package, module_path, method)
        });
    }
    target
        .iter()
        .take(target.len().saturating_sub(1))
        .any(|segment| render_plan.package_mentions_ident(package, segment))
}

fn known_trait_receiver_idents(target: &[String], leaf: &str) -> Option<&'static [&'static str]> {
    match (target.first().map(String::as_str), leaf) {
        (Some("sha1"), "Digest") => Some(&["Sha1"]),
        _ => None,
    }
}

fn known_trait_method_idents(target: &[String], leaf: &str) -> Option<&'static [&'static str]> {
    match (target.first().map(String::as_str), leaf) {
        (Some("std" | "core" | "alloc"), "Hash") => Some(&["hash", "hash_slice"]),
        (Some("std" | "core" | "alloc"), "Hasher") => Some(&[
            "finish",
            "write",
            "write_u8",
            "write_u16",
            "write_u32",
            "write_u64",
        ]),
        (Some("std" | "core" | "alloc"), "Future") => Some(&["poll"]),
        (Some("std" | "core" | "alloc"), "Read") => {
            Some(&["read", "read_exact", "read_to_end", "read_to_string"])
        }
        (Some("std" | "core" | "alloc"), "Seek") => Some(&["seek", "rewind", "stream_position"]),
        (Some("std" | "core" | "alloc"), "Write") => {
            Some(&["write", "write_all", "write_fmt", "flush"])
        }
        (Some("tokio"), "AsyncReadExt") => Some(&[
            "read",
            "read_buf",
            "read_exact",
            "read_i8",
            "read_i16",
            "read_i32",
            "read_i64",
            "read_i128",
            "read_to_end",
            "read_to_string",
            "read_u8",
            "read_u16",
            "read_u32",
            "read_u64",
            "read_u128",
            "chain",
            "take",
        ]),
        (Some("tokio"), "AsyncWriteExt") => Some(&[
            "flush",
            "shutdown",
            "write",
            "write_all",
            "write_all_buf",
            "write_buf",
            "write_i8",
            "write_i16",
            "write_i32",
            "write_i64",
            "write_i128",
            "write_u8",
            "write_u16",
            "write_u32",
            "write_u64",
            "write_u128",
        ]),
        (Some("serde"), "Serialize") => Some(&["serialize"]),
        (Some("sha1"), "Digest") => Some(&["chain_update", "finalize", "reset", "update"]),
        (Some("base64"), "Engine") => Some(&[
            "decode",
            "decode_slice",
            "encode",
            "encode_slice",
            "encode_string",
        ]),
        _ => None,
    }
}

fn generic_trait_method_name_candidates(leaf: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    let snake_case = camel_case_to_snake_case(leaf);
    if !snake_case.is_empty() {
        candidates.push(snake_case);
    }
    for suffix in ["Ext", "Trait"] {
        if let Some(stem) = leaf.strip_suffix(suffix) {
            let snake_case = camel_case_to_snake_case(stem);
            if !snake_case.is_empty() {
                candidates.push(snake_case);
            }
        }
    }
    candidates.sort();
    candidates.dedup();
    candidates
}

fn camel_case_to_snake_case(value: &str) -> String {
    let mut output = String::new();
    let mut previous_was_lowercase_or_digit = false;
    for character in value.chars() {
        if character.is_ascii_uppercase() {
            if previous_was_lowercase_or_digit {
                output.push('_');
            }
            output.push(character.to_ascii_lowercase());
            previous_was_lowercase_or_digit = false;
        } else {
            previous_was_lowercase_or_digit =
                character.is_ascii_lowercase() || character.is_ascii_digit();
            output.push(character);
        }
    }
    output
}

fn known_trait_associated_function_idents(
    target: &[String],
    leaf: &str,
) -> Option<&'static [&'static str]> {
    match (target.first().map(String::as_str), leaf) {
        (Some("serde"), "Deserialize") => Some(&["deserialize"]),
        (Some("serde"), "Serialize") => Some(&["serialize"]),
        (Some("sha1"), "Digest") => Some(&["digest", "new", "new_with_prefix"]),
        (Some("std" | "core" | "alloc"), "ExitStatusExt") => Some(&["from_raw"]),
        _ => None,
    }
}

fn is_derive_only_external_trait_import(leaf: &str) -> bool {
    matches!(leaf, "Deserialize" | "Serialize")
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
    resolve_reexported_use_path_inner(project, package, path, &mut BTreeSet::new())
}

fn resolve_reexported_use_path_inner(
    project: &Project,
    package: &str,
    path: &[String],
    visited: &mut BTreeSet<(String, Vec<String>)>,
) -> Option<(String, Vec<String>)> {
    if !visited.insert((package.to_string(), path.to_vec())) {
        return None;
    }
    let name = path.last()?;
    let module_path = &path[..path.len() - 1];
    let aliases = project
        .module_aliases
        .get(&(package.to_string(), module_path.to_vec()))?;
    let target = aliases.get(name)?;
    let (target_package, target_path) =
        resolve_use_target_path(project, package, module_path, target)?;
    resolve_reexported_use_path_inner(project, &target_package, &target_path, visited)
        .or(Some((target_package, target_path)))
}

fn find_use_function(project: &Project, package: &str, path: &[String]) -> Option<CallableId> {
    find_use_function_direct(project, package, path).or_else(|| {
        let name = path.last()?;
        let module_path = path[..path.len() - 1].to_vec();
        find_glob_reexport_function(project, package, &module_path, name, &mut BTreeSet::new())
    })
}

fn find_use_function_direct(
    project: &Project,
    package: &str,
    path: &[String],
) -> Option<CallableId> {
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
    find_use_item_direct(project, package, path).or_else(|| {
        let name = path.last()?;
        let module_path = path[..path.len() - 1].to_vec();
        find_glob_reexport_item(project, package, &module_path, name, &mut BTreeSet::new())
    })
}

fn find_use_item_direct(project: &Project, package: &str, path: &[String]) -> Option<ItemId> {
    let name = path.last()?.clone();
    let module_path = path[..path.len() - 1].to_vec();
    [
        ItemKind::Struct,
        ItemKind::Enum,
        ItemKind::Union,
        ItemKind::Type,
        ItemKind::Trait,
        ItemKind::Mod,
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

fn find_glob_reexport_function(
    project: &Project,
    package: &str,
    module_path: &[String],
    name: &str,
    visited: &mut BTreeSet<(String, Vec<String>, String)>,
) -> Option<CallableId> {
    if !visited.insert((package.to_string(), module_path.to_vec(), name.to_string())) {
        return None;
    }
    let source = project
        .files
        .values()
        .find(|source| source.package == package && source.module_path == module_path)?;
    for glob_path in visible_glob_use_paths(&source.syntax.items) {
        let Some((target_package, target_module_path)) =
            resolve_use_target_path(project, package, module_path, &glob_path)
        else {
            continue;
        };
        let mut target_path = target_module_path.clone();
        target_path.push(name.to_string());
        if let Some(callable) = find_use_function_direct(project, &target_package, &target_path) {
            return Some(callable);
        }
        if let Some(callable) = find_glob_reexport_function(
            project,
            &target_package,
            &target_module_path,
            name,
            visited,
        ) {
            return Some(callable);
        }
    }
    None
}

fn find_glob_reexport_item(
    project: &Project,
    package: &str,
    module_path: &[String],
    name: &str,
    visited: &mut BTreeSet<(String, Vec<String>, String)>,
) -> Option<ItemId> {
    if !visited.insert((package.to_string(), module_path.to_vec(), name.to_string())) {
        return None;
    }
    let source = project
        .files
        .values()
        .find(|source| source.package == package && source.module_path == module_path)?;
    for glob_path in visible_glob_use_paths(&source.syntax.items) {
        let Some((target_package, target_module_path)) =
            resolve_use_target_path(project, package, module_path, &glob_path)
        else {
            continue;
        };
        let mut target_path = target_module_path.clone();
        target_path.push(name.to_string());
        if let Some(item) = find_use_item_direct(project, &target_package, &target_path) {
            return Some(item);
        }
        if let Some(item) =
            find_glob_reexport_item(project, &target_package, &target_module_path, name, visited)
        {
            return Some(item);
        }
    }
    None
}

fn visible_glob_use_paths(items: &[Item]) -> Vec<Vec<String>> {
    let mut paths = Vec::new();
    for item in items {
        let Item::Use(item_use) = item else {
            continue;
        };
        collect_glob_use_paths(&item_use.tree, Vec::new(), &mut paths);
    }
    paths
}

fn collect_glob_use_paths(tree: &UseTree, mut prefix: Vec<String>, paths: &mut Vec<Vec<String>>) {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            collect_glob_use_paths(&path.tree, prefix, paths);
        }
        UseTree::Group(group) => {
            for nested in &group.items {
                collect_glob_use_paths(nested, prefix.clone(), paths);
            }
        }
        UseTree::Glob(_) => paths.push(prefix),
        UseTree::Name(_) | UseTree::Rename(_) => {}
    }
}

fn project_has_module(project: &Project, package: &str, module_path: &[String]) -> bool {
    module_path.is_empty()
        || project
            .files
            .values()
            .any(|source| source.package == package && source.module_path == module_path)
}

fn local_type_path(
    module_path: &[String],
    self_ty: &Type,
    aliases: &std::collections::HashMap<String, Vec<String>>,
) -> Option<Vec<String>> {
    if let Type::Reference(reference) = self_ty {
        return local_type_path(module_path, &reference.elem, aliases);
    }
    let Type::Path(type_path) = self_ty else {
        return None;
    };
    let segments = apply_alias(
        type_path
            .path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect(),
        aliases,
    );
    normalize_segments(module_path, segments)
}

fn raw_local_type_path(module_path: &[String], self_ty: &Type) -> Option<Vec<String>> {
    if let Type::Reference(reference) = self_ty {
        return raw_local_type_path(module_path, &reference.elem);
    }
    let Type::Path(type_path) = self_ty else {
        return None;
    };
    normalize_segments(
        module_path,
        type_path
            .path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect(),
    )
}

fn resolved_local_type_path(
    project: &Project,
    package: &str,
    module_path: &[String],
    self_ty: &Type,
    aliases: &std::collections::HashMap<String, Vec<String>>,
) -> Option<Vec<String>> {
    if let Some(path) = raw_local_type_path(module_path, self_ty) {
        let canonical = canonical_type_path(project, package, module_path, path);
        if find_type_like_item(project, package, &canonical).is_some() {
            return Some(canonical);
        }
    }
    let path = local_type_path(module_path, self_ty, aliases)?;
    Some(canonical_type_path(project, package, module_path, path))
}

fn canonical_type_path(
    project: &Project,
    package: &str,
    module_path: &[String],
    path: Vec<String>,
) -> Vec<String> {
    if find_type_like_item(project, package, &path).is_some() {
        return path;
    }

    if let Some(path) = resolve_reexported_type_path(project, package, &path, &mut Vec::new()) {
        return path;
    }

    if path.len() == module_path.len() + 1 && path.starts_with(module_path) {
        if let Some(name) = path.last() {
            for depth in (0..module_path.len()).rev() {
                let mut candidate = module_path[..depth].to_vec();
                candidate.push(name.clone());
                if find_type_like_item(project, package, &candidate).is_some() {
                    return candidate;
                }
                if let Some(path) =
                    resolve_reexported_type_path(project, package, &candidate, &mut Vec::new())
                {
                    return path;
                }
            }
        }
    }

    path
}

fn resolve_reexported_type_path(
    project: &Project,
    package: &str,
    path: &[String],
    visited: &mut Vec<Vec<String>>,
) -> Option<Vec<String>> {
    if path.is_empty() || visited.iter().any(|seen| seen == path) {
        return None;
    }
    visited.push(path.to_vec());

    let (target_package, target_path) = resolve_reexported_use_path(project, package, path)?;
    if target_package != package {
        return None;
    }
    if find_type_like_item(project, package, &target_path).is_some() {
        return Some(target_path);
    }
    resolve_reexported_type_path(project, package, &target_path, visited)
}

fn find_type_like_item(project: &Project, package: &str, path: &[String]) -> Option<ItemId> {
    let name = path.last()?.clone();
    let module_path = path[..path.len() - 1].to_vec();
    [
        ItemKind::Struct,
        ItemKind::Enum,
        ItemKind::Union,
        ItemKind::Type,
        ItemKind::Trait,
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

fn normalized_path(
    module_path: &[String],
    path: &syn::Path,
    aliases: &std::collections::HashMap<String, Vec<String>>,
) -> Vec<String> {
    let segments = apply_alias(
        path.segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect(),
        aliases,
    );
    normalize_segments(module_path, segments).unwrap_or_default()
}

fn trait_input_type_paths(
    module_path: &[String],
    path: &syn::Path,
    aliases: &std::collections::HashMap<String, Vec<String>>,
) -> Vec<Vec<String>> {
    let mut type_paths = Vec::new();
    for segment in &path.segments {
        if let PathArguments::AngleBracketed(arguments) = &segment.arguments {
            for argument in &arguments.args {
                if let GenericArgument::Type(ty) = argument {
                    collect_type_paths(module_path, ty, aliases, &mut type_paths);
                }
            }
        }
    }
    type_paths.sort();
    type_paths.dedup();
    type_paths
}

fn collect_type_paths(
    module_path: &[String],
    ty: &Type,
    aliases: &std::collections::HashMap<String, Vec<String>>,
    type_paths: &mut Vec<Vec<String>>,
) {
    match ty {
        Type::Path(type_path) => {
            if let Some(path) = local_type_path(module_path, ty, aliases) {
                type_paths.push(path);
            }
            for segment in &type_path.path.segments {
                if let PathArguments::AngleBracketed(arguments) = &segment.arguments {
                    for argument in &arguments.args {
                        if let GenericArgument::Type(ty) = argument {
                            collect_type_paths(module_path, ty, aliases, type_paths);
                        }
                    }
                }
            }
        }
        Type::Reference(reference) => {
            collect_type_paths(module_path, &reference.elem, aliases, type_paths);
        }
        Type::Group(group) => collect_type_paths(module_path, &group.elem, aliases, type_paths),
        Type::Paren(paren) => collect_type_paths(module_path, &paren.elem, aliases, type_paths),
        _ => {}
    }
}

fn normalize_segments(module_path: &[String], segments: Vec<String>) -> Option<Vec<String>> {
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

fn apply_alias(
    mut segments: Vec<String>,
    aliases: &std::collections::HashMap<String, Vec<String>>,
) -> Vec<String> {
    let Some(first) = segments.first() else {
        return segments;
    };
    let Some(target) = aliases.get(first) else {
        return segments;
    };
    let mut resolved = target.clone();
    resolved.extend(segments.drain(1..));
    resolved
}
