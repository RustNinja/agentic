use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use proc_macro2::TokenTree;
use quote::ToTokens;
use syn::{
    GenericArgument, ImplItem, Item, ItemImpl, ItemMod, ItemUse, PathArguments, Type, UseTree,
};

use crate::{
    manifest::Workspace,
    model::{
        CallableId, FunctionRecord, ItemId, ItemKind, ItemRecord, MethodRecord, Project, SourceFile,
    },
    reduce::is_cfg_test_attr,
};

pub fn parse_workspace(workspace: Workspace) -> Result<Project, Box<dyn std::error::Error>> {
    let mut parser = Parser {
        files: HashMap::new(),
        functions: HashMap::new(),
        methods: HashMap::new(),
        items: HashMap::new(),
        module_aliases: HashMap::new(),
    };

    for package in workspace.packages.values() {
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

    Ok(Project {
        workspace,
        files: parser.files,
        functions: parser.functions,
        methods: parser.methods,
        items: parser.items,
        module_aliases: parser.module_aliases,
    })
}

struct Parser {
    files: HashMap<std::path::PathBuf, SourceFile>,
    functions: HashMap<CallableId, FunctionRecord>,
    methods: HashMap<CallableId, MethodRecord>,
    items: HashMap<ItemId, ItemRecord>,
    module_aliases: HashMap<(String, Vec<String>), HashMap<String, Vec<String>>>,
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

        self.collect_items(package, &module_path, &syntax.items, &aliases)?;

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
        items: &[Item],
        aliases: &HashMap<String, Vec<String>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for item in items {
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
                            item: function.clone(),
                            aliases: aliases.clone(),
                        },
                    );
                }
                Item::Impl(item_impl) => {
                    self.collect_impl(package, module_path, item_impl, aliases)?;
                }
                Item::Struct(_)
                | Item::Enum(_)
                | Item::Union(_)
                | Item::Type(_)
                | Item::Trait(_)
                | Item::Const(_)
                | Item::Static(_)
                | Item::Macro(_) => {
                    if let Some((name, kind)) = item_name_and_kind(item) {
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
                                item: item.clone(),
                                aliases: aliases.clone(),
                            },
                        );
                    }
                }
                Item::Mod(item_mod) => {
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
                        self.collect_items(package, &child_path, items, &child_aliases)?;
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
        item_impl: &ItemImpl,
        aliases: &HashMap<String, Vec<String>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Some(type_path) = local_type_path(module_path, &item_impl.self_ty, aliases) else {
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
                        item: method.clone(),
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
        if item_mod.attrs.iter().any(is_cfg_test_attr) {
            return Ok(());
        }

        let name = item_mod.ident.to_string();
        let source_name = module_source_name(&name);
        let file_path = module_dir.join(format!("{source_name}.rs"));
        let mod_path = module_dir.join(&source_name).join("mod.rs");
        let (next_file, next_dir) = if file_path.exists() {
            (file_path, module_dir.join(&source_name))
        } else if mod_path.exists() {
            (mod_path, module_dir.join(&source_name))
        } else {
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
