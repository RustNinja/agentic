use std::{
    collections::{BTreeSet, HashMap, VecDeque},
    fs,
    path::{Path, PathBuf},
};

use proc_macro2::{Span, TokenTree};
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{
    parse::Parser as SynParser, punctuated::Punctuated, Attribute, Expr, GenericArgument, ImplItem,
    Item, ItemImpl, ItemMod, ItemUse, Lit, Meta, PathArguments, Type, UseTree,
};

use crate::{
    manifest::Workspace,
    model::{
        CallableId, FunctionRecord, ItemId, ItemKind, ItemRecord, MethodRecord, Project,
        SourceFile, SourceSpan,
    },
    reduce::is_cfg_test_attr,
};

pub fn parse_workspace(workspace: Workspace) -> Result<Project, Box<dyn std::error::Error>> {
    parse_workspace_with_package_filter(workspace, None, false)
}

pub fn parse_workspace_package_closure(
    workspace: Workspace,
    root_packages: &BTreeSet<String>,
) -> Result<Project, Box<dyn std::error::Error>> {
    let package_filter = workspace_package_closure(&workspace, root_packages);
    parse_workspace_with_package_filter(workspace, Some(&package_filter), true)
}

pub fn parse_workspace_packages(
    workspace: Workspace,
    packages: &BTreeSet<String>,
) -> Result<Project, Box<dyn std::error::Error>> {
    parse_workspace_with_package_filter(workspace, Some(packages), true)
}

fn parse_workspace_with_package_filter(
    workspace: Workspace,
    package_filter: Option<&BTreeSet<String>>,
    ignore_missing_modules: bool,
) -> Result<Project, Box<dyn std::error::Error>> {
    let mut parser = Parser {
        files: HashMap::new(),
        functions: HashMap::new(),
        methods: HashMap::new(),
        items: HashMap::new(),
        module_aliases: HashMap::new(),
        glob_use_paths_by_module: HashMap::new(),
        ignore_missing_modules,
    };

    for package in workspace.packages.values() {
        if package_filter.is_some_and(|filter| !filter.contains(&package.name)) {
            continue;
        }
        if package.lib_path.exists() {
            let module_dir = package
                .lib_path
                .parent()
                .ok_or_else(|| format!("package {} source path has no parent", package.name))?
                .to_path_buf();
            parser.parse_file(
                &package.name,
                Vec::new(),
                &package.lib_path,
                &module_dir,
                &package.root,
            )?;
        }
    }

    let source_files_by_module = parser
        .files
        .values()
        .map(|source| {
            (
                (source.package.clone(), source.module_path.clone()),
                source.path.clone(),
            )
        })
        .collect();
    let mut methods_by_receiver = HashMap::<(String, Vec<String>, String), Vec<CallableId>>::new();
    for callable in parser.methods.keys() {
        let CallableId::Method {
            package,
            type_path,
            method,
            ..
        } = callable
        else {
            continue;
        };
        methods_by_receiver
            .entry((package.clone(), type_path.clone(), method.clone()))
            .or_default()
            .push(callable.clone());
    }
    for methods in methods_by_receiver.values_mut() {
        methods.sort();
        methods.dedup();
    }
    let receivers_with_methods = methods_by_receiver
        .keys()
        .map(|(package, type_path, _)| (package.clone(), type_path.clone()))
        .collect();

    Ok(Project {
        workspace,
        files: parser.files,
        functions: parser.functions,
        methods: parser.methods,
        items: parser.items,
        module_aliases: parser.module_aliases,
        glob_use_paths_by_module: parser.glob_use_paths_by_module,
        source_files_by_module,
        methods_by_receiver,
        receivers_with_methods,
    })
}

fn workspace_package_closure(
    workspace: &Workspace,
    root_packages: &BTreeSet<String>,
) -> BTreeSet<String> {
    let mut packages = BTreeSet::new();
    let mut queue = VecDeque::from_iter(root_packages.iter().cloned());
    while let Some(package) = queue.pop_front() {
        if !packages.insert(package.clone()) {
            continue;
        }
        let Some(record) = workspace.packages.get(&package) else {
            continue;
        };
        for dependency in &record.dependencies {
            if workspace.packages.contains_key(&dependency.package)
                && !packages.contains(&dependency.package)
            {
                queue.push_back(dependency.package.clone());
            }
        }
    }
    packages
}

struct Parser {
    files: HashMap<std::path::PathBuf, SourceFile>,
    functions: HashMap<CallableId, FunctionRecord>,
    methods: HashMap<CallableId, MethodRecord>,
    items: HashMap<ItemId, ItemRecord>,
    module_aliases: HashMap<(String, Vec<String>), HashMap<String, Vec<String>>>,
    glob_use_paths_by_module: HashMap<(String, Vec<String>), Vec<Vec<String>>>,
    ignore_missing_modules: bool,
}

impl Parser {
    fn parse_file(
        &mut self,
        package: &str,
        module_path: Vec<String>,
        file_path: &Path,
        module_dir: &Path,
        package_root: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = file_path.canonicalize()?;
        if self.files.contains_key(&file_path) {
            return Ok(());
        }

        let text = fs::read_to_string(&file_path)?;
        let syntax = syn::parse_file(&text)
            .map_err(|error| format!("failed to parse {}: {error}", file_path.display()))?;
        let parent_aliases = module_path.split_last().and_then(|_| {
            let parent_path = &module_path[..module_path.len().saturating_sub(1)];
            self.module_aliases
                .get(&(package.to_string(), parent_path.to_vec()))
        });
        let aliases = collect_aliases(&syntax.items, parent_aliases);
        self.module_aliases
            .insert((package.to_string(), module_path.clone()), aliases.clone());
        self.glob_use_paths_by_module.insert(
            (package.to_string(), module_path.clone()),
            collect_glob_use_paths(&syntax.items),
        );

        self.collect_items(package, &module_path, &file_path, &syntax.items, &aliases)?;

        self.files.insert(
            file_path.clone(),
            SourceFile {
                package: package.to_string(),
                module_path: module_path.clone(),
                path: file_path.clone(),
                syntax,
            },
        );

        let items = self
            .files
            .get(&file_path)
            .expect("file was just inserted")
            .syntax
            .items
            .clone();
        for item in items {
            match item {
                Item::Mod(item_mod) => {
                    self.parse_external_module(
                        package,
                        &module_path,
                        module_dir,
                        package_root,
                        &item_mod,
                    )?;
                }
                Item::Macro(item_macro) => {
                    self.parse_automod_dir(package, &module_path, package_root, &item_macro)?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn collect_items(
        &mut self,
        package: &str,
        module_path: &[String],
        file_path: &Path,
        items: &[Item],
        aliases: &HashMap<String, Vec<String>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for item in items {
            if item_attrs_exclude_current_target(item) {
                continue;
            }
            match item {
                Item::Fn(function) => {
                    let id = CallableId::Free {
                        package: package.to_string(),
                        module_path: module_path.to_vec(),
                        name: function.sig.ident.to_string(),
                    };
                    self.functions.insert(
                        id.clone(),
                        FunctionRecord {
                            id,
                            package: package.to_string(),
                            module_path: module_path.to_vec(),
                            span: source_span(file_path, function.span()),
                            item: function.clone(),
                            aliases: aliases.clone(),
                        },
                    );
                }
                Item::Impl(item_impl) => {
                    self.collect_impl(package, module_path, file_path, item_impl, aliases)?;
                }
                Item::Struct(_)
                | Item::Enum(_)
                | Item::Union(_)
                | Item::Type(_)
                | Item::Trait(_)
                | Item::Const(_)
                | Item::Static(_)
                | Item::Macro(_) => {
                    if let Some((name, kind)) = item_name_and_kind(item).or_else(|| {
                        let Item::Macro(item_macro) = item else {
                            return None;
                        };
                        bitflags_struct_name(item_macro).map(|name| (name, ItemKind::Struct))
                    }) {
                        let id = ItemId {
                            package: package.to_string(),
                            module_path: module_path.to_vec(),
                            name,
                            kind,
                        };
                        self.items.insert(
                            id.clone(),
                            ItemRecord {
                                package: package.to_string(),
                                module_path: module_path.to_vec(),
                                span: source_span(file_path, item.span()),
                                item: item.clone(),
                                aliases: aliases.clone(),
                            },
                        );
                    }
                }
                Item::Mod(item_mod) => {
                    if item_mod.attrs.iter().any(is_cfg_test_attr) {
                        continue;
                    }
                    if item_mod.content.is_none()
                        && module_attrs_exclude_current_target(&item_mod.attrs)
                    {
                        continue;
                    }
                    let id = ItemId {
                        package: package.to_string(),
                        module_path: module_path.to_vec(),
                        name: item_mod.ident.to_string(),
                        kind: ItemKind::Mod,
                    };
                    self.items.insert(
                        id,
                        ItemRecord {
                            package: package.to_string(),
                            module_path: module_path.to_vec(),
                            span: source_span(file_path, item.span()),
                            item: item.clone(),
                            aliases: aliases.clone(),
                        },
                    );
                    if let Some((_, items)) = &item_mod.content {
                        let mut child_path = module_path.to_vec();
                        child_path.push(item_mod.ident.to_string());
                        let child_aliases = collect_aliases(items, Some(aliases));
                        self.module_aliases.insert(
                            (package.to_string(), child_path.clone()),
                            child_aliases.clone(),
                        );
                        self.glob_use_paths_by_module.insert(
                            (package.to_string(), child_path.clone()),
                            collect_glob_use_paths(items),
                        );
                        self.collect_items(package, &child_path, file_path, items, &child_aliases)?;
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn collect_impl(
        &mut self,
        package: &str,
        module_path: &[String],
        file_path: &Path,
        item_impl: &ItemImpl,
        aliases: &HashMap<String, Vec<String>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Some(type_path) =
            self.canonical_impl_type_path(package, module_path, &item_impl.self_ty, aliases)
        else {
            return Ok(());
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

        for impl_item in &item_impl.items {
            if impl_item_attrs_exclude_current_target(impl_item) {
                continue;
            }
            if let ImplItem::Fn(method) = impl_item {
                let id = CallableId::Method {
                    package: package.to_string(),
                    type_path: type_path.clone(),
                    trait_path: trait_path.clone(),
                    trait_input_type_paths: trait_input_type_paths.clone(),
                    method: method.sig.ident.to_string(),
                };
                self.methods.insert(
                    id.clone(),
                    MethodRecord {
                        module_path: module_path.to_vec(),
                        span: source_span(file_path, method.span()),
                        impl_span: source_span(file_path, item_impl.span()),
                        impl_attrs: item_impl.attrs.clone(),
                        item: method.clone(),
                        impl_generics: item_impl.generics.clone(),
                        impl_items: item_impl.items.clone(),
                        trait_input_type_paths: trait_input_type_paths.clone(),
                        aliases: aliases.clone(),
                    },
                );
            }
        }

        Ok(())
    }

    fn parse_external_module(
        &mut self,
        package: &str,
        module_path: &[String],
        module_dir: &Path,
        package_root: &Path,
        item_mod: &ItemMod,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if item_mod.content.is_some() {
            return Ok(());
        }
        if module_attrs_exclude_current_target(&item_mod.attrs) {
            return Ok(());
        }

        let name = item_mod.ident.to_string();
        let source_name = module_source_name(&name);
        let path_attr = path_attr(item_mod);
        let file_path = module_dir.join(format!("{source_name}.rs"));
        let mod_path = module_dir.join(source_name).join("mod.rs");
        let (next_file, next_dir) = if let Some(path_attr) = path_attr {
            let path = if path_attr.is_absolute() {
                path_attr
            } else {
                module_dir.join(path_attr)
            };
            if !path.exists() {
                if self.ignore_missing_modules
                    || module_attrs_exclude_current_target(&item_mod.attrs)
                {
                    return Ok(());
                }
                return Err(format!(
                    "module {name} path attribute points to missing source file in package {package}: {}",
                    path.display()
                )
                .into());
            }
            let next_dir = path.parent().unwrap_or(module_dir).to_path_buf();
            (path, next_dir)
        } else if file_path.exists() {
            (file_path, module_dir.join(source_name))
        } else if mod_path.exists() {
            (mod_path, module_dir.join(source_name))
        } else {
            if self.ignore_missing_modules || module_attrs_exclude_current_target(&item_mod.attrs) {
                return Ok(());
            }
            return Err(format!(
                "module {name} has no matching source file in package {package} at {} (looked for {} and {})",
                module_dir.display(),
                file_path.display(),
                mod_path.display()
            )
            .into());
        };

        let mut child_path = module_path.to_vec();
        child_path.push(name);
        self.parse_file(package, child_path, &next_file, &next_dir, package_root)
    }

    fn parse_automod_dir(
        &mut self,
        package: &str,
        module_path: &[String],
        package_root: &Path,
        item_macro: &syn::ItemMacro,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Some(relative_dir) = automod_dir_path(item_macro) else {
            return Ok(());
        };
        let dir = package_root.join(relative_dir);
        if !dir.exists() {
            return Ok(());
        }

        let mut entries = fs::read_dir(&dir)?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "rs"))
            .filter(|path| {
                path.file_name()
                    .is_some_and(|file_name| file_name != "mod.rs")
            })
            .collect::<Vec<_>>();
        entries.sort();

        for path in entries {
            let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
                continue;
            };
            let mut child_path = module_path.to_vec();
            child_path.push(stem.to_string());
            let module_dir = path
                .parent()
                .expect("automod child file should have a parent")
                .to_path_buf();
            self.parse_file(package, child_path, &path, &module_dir, package_root)?;
        }

        Ok(())
    }

    fn canonical_impl_type_path(
        &self,
        package: &str,
        module_path: &[String],
        self_ty: &Type,
        aliases: &HashMap<String, Vec<String>>,
    ) -> Option<Vec<String>> {
        if let Some(path) = raw_local_type_path(module_path, self_ty) {
            let canonical = self.canonical_type_path(package, module_path, path);
            if self.has_type_like_item(package, &canonical) {
                return Some(canonical);
            }
        }
        let path = local_type_path(module_path, self_ty, aliases)?;
        Some(self.canonical_type_path(package, module_path, path))
    }

    fn canonical_type_path(
        &self,
        package: &str,
        module_path: &[String],
        path: Vec<String>,
    ) -> Vec<String> {
        if self.has_type_like_item(package, &path) {
            return path;
        }

        if let Some(path) = self.resolve_reexported_type_path(package, &path, &mut Vec::new()) {
            return path;
        }

        if path.len() == module_path.len() + 1 && path.starts_with(module_path) {
            if let Some(name) = path.last() {
                for depth in (0..module_path.len()).rev() {
                    let mut candidate = module_path[..depth].to_vec();
                    candidate.push(name.clone());
                    if self.has_type_like_item(package, &candidate) {
                        return candidate;
                    }
                    if let Some(path) =
                        self.resolve_reexported_type_path(package, &candidate, &mut Vec::new())
                    {
                        return path;
                    }
                }
            }
        }

        path
    }

    fn resolve_reexported_type_path(
        &self,
        package: &str,
        path: &[String],
        visited: &mut Vec<Vec<String>>,
    ) -> Option<Vec<String>> {
        if path.is_empty() || visited.iter().any(|seen| seen == path) {
            return None;
        }
        visited.push(path.to_vec());

        let name = path.last()?;
        let module_path = &path[..path.len() - 1];
        let aliases = self
            .module_aliases
            .get(&(package.to_string(), module_path.to_vec()))?;
        let target = aliases.get(name)?;
        let resolved = normalize_segments(module_path, target.clone())?;
        if self.has_type_like_item(package, &resolved) {
            return Some(resolved);
        }
        self.resolve_reexported_type_path(package, &resolved, visited)
    }

    fn has_type_like_item(&self, package: &str, path: &[String]) -> bool {
        if path.is_empty() {
            return false;
        }
        let name = path.last().expect("path is not empty");
        let module_path = &path[..path.len() - 1];
        [
            ItemKind::Struct,
            ItemKind::Enum,
            ItemKind::Union,
            ItemKind::Type,
            ItemKind::Trait,
        ]
        .iter()
        .any(|kind| {
            let id = ItemId {
                package: package.to_string(),
                module_path: module_path.to_vec(),
                name: name.clone(),
                kind: *kind,
            };
            self.items.contains_key(&id)
        })
    }
}

fn automod_dir_path(item_macro: &syn::ItemMacro) -> Option<PathBuf> {
    if !is_automod_dir_macro(&item_macro.mac.path) {
        return None;
    }

    item_macro.mac.tokens.clone().into_iter().find_map(|token| {
        let TokenTree::Literal(literal) = token else {
            return None;
        };
        syn::parse2::<syn::LitStr>(literal.to_token_stream())
            .ok()
            .map(|literal| PathBuf::from(literal.value()))
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

fn item_name_and_kind(item: &Item) -> Option<(String, ItemKind)> {
    match item {
        Item::Struct(item) => Some((item.ident.to_string(), ItemKind::Struct)),
        Item::Enum(item) => Some((item.ident.to_string(), ItemKind::Enum)),
        Item::Union(item) => Some((item.ident.to_string(), ItemKind::Union)),
        Item::Type(item) => Some((item.ident.to_string(), ItemKind::Type)),
        Item::Trait(item) => Some((item.ident.to_string(), ItemKind::Trait)),
        Item::Mod(item) => Some((item.ident.to_string(), ItemKind::Mod)),
        Item::Const(item) => Some((item.ident.to_string(), ItemKind::Const)),
        Item::Static(item) => Some((item.ident.to_string(), ItemKind::Static)),
        Item::Macro(item) => item
            .ident
            .as_ref()
            .map(|ident| (ident.to_string(), ItemKind::Macro)),
        _ => None,
    }
}

fn bitflags_struct_name(item: &syn::ItemMacro) -> Option<String> {
    if item
        .mac
        .path
        .segments
        .last()
        .is_none_or(|segment| segment.ident != "bitflags")
    {
        return None;
    }
    let mut saw_struct = false;
    for token in item.mac.tokens.clone() {
        let TokenTree::Ident(ident) = token else {
            continue;
        };
        if saw_struct {
            return Some(ident.to_string());
        }
        saw_struct = ident == "struct";
    }
    None
}

fn source_span(file_path: &Path, span: Span) -> SourceSpan {
    let start = span.start();
    let end = span.end();
    SourceSpan {
        file: file_path.to_path_buf(),
        start_line: start.line,
        start_column: start.column,
        end_line: end.line,
        end_column: end.column,
    }
}

fn collect_aliases(
    items: &[Item],
    parent_aliases: Option<&HashMap<String, Vec<String>>>,
) -> HashMap<String, Vec<String>> {
    let mut aliases = if items_have_super_glob_import(items) {
        parent_aliases.cloned().unwrap_or_default()
    } else {
        HashMap::new()
    };
    for item in items {
        if let Item::Use(ItemUse { tree, .. }) = item {
            collect_use_tree(tree, Vec::new(), &mut aliases);
        }
    }
    aliases
}

fn collect_glob_use_paths(items: &[Item]) -> Vec<Vec<String>> {
    let mut paths = Vec::new();
    for item in items {
        if let Item::Use(ItemUse { tree, .. }) = item {
            collect_glob_use_tree(tree, Vec::new(), &mut paths);
        }
    }
    paths
}

fn collect_glob_use_tree(tree: &UseTree, mut prefix: Vec<String>, paths: &mut Vec<Vec<String>>) {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            collect_glob_use_tree(&path.tree, prefix, paths);
        }
        UseTree::Group(group) => {
            for nested in &group.items {
                collect_glob_use_tree(nested, prefix.clone(), paths);
            }
        }
        UseTree::Glob(_) => paths.push(prefix),
        UseTree::Name(_) | UseTree::Rename(_) => {}
    }
}

fn items_have_super_glob_import(items: &[Item]) -> bool {
    items.iter().any(|item| {
        let Item::Use(ItemUse { tree, .. }) = item else {
            return false;
        };
        use_tree_has_super_glob_import(tree)
    })
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

fn collect_use_tree(
    tree: &UseTree,
    mut prefix: Vec<String>,
    aliases: &mut HashMap<String, Vec<String>>,
) {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            collect_use_tree(&path.tree, prefix, aliases);
        }
        UseTree::Name(name) => {
            let ident = name.ident.to_string();
            let mut target = prefix;
            target.push(ident.clone());
            aliases.insert(ident, target);
        }
        UseTree::Rename(rename) => {
            let mut target = prefix;
            target.push(rename.ident.to_string());
            aliases.insert(rename.rename.to_string(), target);
        }
        UseTree::Group(group) => {
            for nested in &group.items {
                collect_use_tree(nested, prefix.clone(), aliases);
            }
        }
        UseTree::Glob(_) => {}
    }
}

fn local_type_path(
    module_path: &[String],
    self_ty: &Type,
    aliases: &HashMap<String, Vec<String>>,
) -> Option<Vec<String>> {
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

fn normalized_path(
    module_path: &[String],
    path: &syn::Path,
    aliases: &HashMap<String, Vec<String>>,
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

fn apply_alias(mut segments: Vec<String>, aliases: &HashMap<String, Vec<String>>) -> Vec<String> {
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

fn trait_input_type_paths(
    module_path: &[String],
    path: &syn::Path,
    aliases: &HashMap<String, Vec<String>>,
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
    aliases: &HashMap<String, Vec<String>>,
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
            collect_type_paths(module_path, &reference.elem, aliases, type_paths)
        }
        _ => {}
    }
}

fn module_source_name(name: &str) -> &str {
    name.strip_prefix("r#").unwrap_or(name)
}

fn module_attrs_exclude_current_target(attrs: &[Attribute]) -> bool {
    attrs
        .iter()
        .any(|attr| is_cfg_test_attr(attr) || cfg_attr_is_definitely_false_for_current_target(attr))
}

fn item_attrs_exclude_current_target(item: &Item) -> bool {
    let attrs = match item {
        Item::Const(item) => &item.attrs,
        Item::Enum(item) => &item.attrs,
        Item::ExternCrate(item) => &item.attrs,
        Item::Fn(item) => &item.attrs,
        Item::ForeignMod(item) => &item.attrs,
        Item::Impl(item) => &item.attrs,
        Item::Macro(item) => &item.attrs,
        Item::Mod(item) => &item.attrs,
        Item::Static(item) => &item.attrs,
        Item::Struct(item) => &item.attrs,
        Item::Trait(item) => &item.attrs,
        Item::TraitAlias(item) => &item.attrs,
        Item::Type(item) => &item.attrs,
        Item::Union(item) => &item.attrs,
        Item::Use(item) => &item.attrs,
        Item::Verbatim(_) => return false,
        _ => return false,
    };
    module_attrs_exclude_current_target(attrs)
}

fn impl_item_attrs_exclude_current_target(item: &ImplItem) -> bool {
    let attrs = match item {
        ImplItem::Const(item) => &item.attrs,
        ImplItem::Fn(item) => &item.attrs,
        ImplItem::Type(item) => &item.attrs,
        ImplItem::Macro(item) => &item.attrs,
        ImplItem::Verbatim(_) => return false,
        _ => return false,
    };
    module_attrs_exclude_current_target(attrs)
}

fn cfg_attr_is_definitely_false_for_current_target(attr: &Attribute) -> bool {
    if !attr.path().is_ident("cfg") {
        return false;
    }
    attr.parse_args::<Meta>()
        .ok()
        .and_then(|meta| cfg_meta_eval_current_target(&meta))
        .is_some_and(|active| !active)
}

fn cfg_meta_eval_current_target(meta: &Meta) -> Option<bool> {
    match meta {
        Meta::Path(path) => cfg_path_eval_current_target(path),
        Meta::NameValue(name_value) => {
            let key = name_value.path.segments.last()?.ident.to_string();
            let value = expr_string_literal(&name_value.value)?;
            cfg_key_value_eval_current_target(&key, &value)
        }
        Meta::List(list) => {
            let key = list.path.segments.last()?.ident.to_string();
            let args = SynParser::parse2(
                Punctuated::<Meta, syn::Token![,]>::parse_terminated,
                list.tokens.clone(),
            )
            .ok()?
            .into_iter()
            .collect::<Vec<_>>();
            match key.as_str() {
                "all" => {
                    let mut has_unknown = false;
                    for arg in &args {
                        match cfg_meta_eval_current_target(arg) {
                            Some(true) => {}
                            Some(false) => return Some(false),
                            None => has_unknown = true,
                        }
                    }
                    (!has_unknown).then_some(true)
                }
                "any" => {
                    let mut has_unknown = false;
                    for arg in &args {
                        match cfg_meta_eval_current_target(arg) {
                            Some(true) => return Some(true),
                            Some(false) => {}
                            None => has_unknown = true,
                        }
                    }
                    (!has_unknown).then_some(false)
                }
                "not" if args.len() == 1 => cfg_meta_eval_current_target(&args[0]).map(|v| !v),
                _ => None,
            }
        }
    }
}

fn cfg_path_eval_current_target(path: &syn::Path) -> Option<bool> {
    let key = path.segments.last()?.ident.to_string();
    match key.as_str() {
        "test" => Some(false),
        "unix" => Some(cfg!(unix)),
        "windows" => Some(cfg!(windows)),
        "debug_assertions" => Some(cfg!(debug_assertions)),
        _ => None,
    }
}

fn cfg_key_value_eval_current_target(key: &str, value: &str) -> Option<bool> {
    match key {
        "target_arch" => Some(value == std::env::consts::ARCH),
        "target_family" => Some(value == std::env::consts::FAMILY),
        "target_os" => Some(value == std::env::consts::OS),
        "target_pointer_width" => Some(value == (std::mem::size_of::<usize>() * 8).to_string()),
        _ => None,
    }
}

fn expr_string_literal(expr: &Expr) -> Option<String> {
    let Expr::Lit(expr_lit) = expr else {
        return None;
    };
    let Lit::Str(lit) = &expr_lit.lit else {
        return None;
    };
    Some(lit.value())
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn parse_skips_missing_module_behind_inactive_target_cfg() {
        let root = temp_workspace("parse-inactive-cfg-module");
        fs::create_dir_all(root.join("app/src")).expect("fixture dirs should create");
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        )
        .expect("workspace manifest should write");
        fs::write(
            root.join("app/Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
        )
        .expect("package manifest should write");
        fs::write(
            root.join("app/src/lib.rs"),
            "#[cfg(windows)]\n#[path = \"missing_windows.rs\"]\nmod windows_only;\n\npub fn selected() -> usize { 1 }\n",
        )
        .expect("lib source should write");

        let workspace = crate::manifest::load_workspace_without_marker_targets(&root)
            .expect("workspace should load");
        let project = parse_workspace(workspace).expect("inactive cfg module should be skipped");
        assert!(project
            .functions
            .keys()
            .any(|id| id.to_string() == "app::selected"));

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn parse_skips_existing_module_behind_inactive_target_cfg() {
        let root = temp_workspace("parse-existing-inactive-cfg-module");
        let inactive_os = inactive_target_os();
        fs::create_dir_all(root.join("app/src")).expect("fixture dirs should create");
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        )
        .expect("workspace manifest should write");
        fs::write(
            root.join("app/Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
        )
        .expect("package manifest should write");
        fs::write(
            root.join("app/src/lib.rs"),
            format!(
                "#[cfg(target_os = \"{inactive_os}\")]\nmod inactive;\n\npub fn selected() -> usize {{ 1 }}\n"
            ),
        )
        .expect("lib source should write");
        fs::write(
            root.join("app/src/inactive.rs"),
            "pub fn should_not_parse() -> usize { missing_dependency::value() }\n",
        )
        .expect("inactive module source should write");

        let workspace = crate::manifest::load_workspace_without_marker_targets(&root)
            .expect("workspace should load");
        let project =
            parse_workspace(workspace).expect("inactive existing module should be skipped");
        assert!(project
            .functions
            .keys()
            .any(|id| id.to_string() == "app::selected"));
        assert!(!project
            .functions
            .keys()
            .any(|id| id.to_string().contains("should_not_parse")));

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn parse_skips_inline_items_behind_inactive_target_cfg() {
        let root = temp_workspace("parse-inline-inactive-cfg-items");
        let inactive_os = inactive_target_os();
        fs::create_dir_all(root.join("app/src")).expect("fixture dirs should create");
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        )
        .expect("workspace manifest should write");
        fs::write(
            root.join("app/Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
        )
        .expect("package manifest should write");
        fs::write(
            root.join("app/src/lib.rs"),
            format!(
                r#"#[cfg(target_os = "{inactive_os}")]
pub fn inactive_free() -> usize {{ 1 }}

pub struct Active;

impl Active {{
    #[cfg(target_os = "{inactive_os}")]
    pub fn inactive_method(&self) -> usize {{ 1 }}

    pub fn active_method(&self) -> usize {{ 2 }}
}}
"#
            ),
        )
        .expect("lib source should write");

        let workspace = crate::manifest::load_workspace_without_marker_targets(&root)
            .expect("workspace should load");
        let project = parse_workspace(workspace).expect("inactive inline items should be skipped");
        assert!(!project
            .functions
            .keys()
            .any(|id| id.to_string() == "app::inactive_free"));
        assert!(project
            .methods
            .keys()
            .any(|id| id.to_string() == "app::Active::active_method"));
        assert!(!project
            .methods
            .keys()
            .any(|id| id.to_string() == "app::Active::inactive_method"));

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn explicit_root_parse_ignores_unrelated_broken_workspace_packages() {
        let root = temp_workspace("parse-root-package-closure");
        fs::create_dir_all(root.join("app/src")).expect("app dirs should create");
        fs::create_dir_all(root.join("broken/src")).expect("broken dirs should create");
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\", \"broken\"]\nresolver = \"2\"\n",
        )
        .expect("workspace manifest should write");
        fs::write(
            root.join("app/Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n\n[dependencies]\nbroken = { path = \"../broken\" }\n",
        )
        .expect("app manifest should write");
        fs::write(
            root.join("broken/Cargo.toml"),
            "[package]\nname = \"broken\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
        )
        .expect("broken manifest should write");
        fs::write(
            root.join("app/src/lib.rs"),
            "pub fn selected() -> usize { 1 }\n",
        )
        .expect("app source should write");
        fs::write(
            root.join("broken/src/lib.rs"),
            "#[path = \"missing.rs\"]\nmod missing;\npub fn broken() {}\n",
        )
        .expect("broken source should write");

        let workspace = crate::manifest::load_workspace_without_marker_targets(&root)
            .expect("workspace should load");
        let packages = BTreeSet::from(["app".to_string()]);
        let project = parse_workspace_package_closure(workspace, &packages)
            .expect("unrelated broken package should not block explicit root parsing");
        assert!(project
            .functions
            .keys()
            .any(|id| id.to_string() == "app::selected"));
        assert!(project
            .functions
            .keys()
            .any(|id| id.to_string().starts_with("broken::")));

        let workspace = crate::manifest::load_workspace_without_marker_targets(&root)
            .expect("workspace should load");
        let project = parse_workspace_packages(workspace, &packages)
            .expect("root package parse should not parse dependency package");
        assert!(project
            .functions
            .keys()
            .any(|id| id.to_string() == "app::selected"));
        assert!(!project
            .functions
            .keys()
            .any(|id| id.to_string().starts_with("broken::")));

        fs::remove_dir_all(root).ok();
    }

    fn temp_workspace(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("slicer-{label}-{}-{nanos}", std::process::id()))
    }

    fn inactive_target_os() -> &'static str {
        match std::env::consts::OS {
            "windows" => "macos",
            _ => "windows",
        }
    }
}
