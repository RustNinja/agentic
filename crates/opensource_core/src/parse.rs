use std::{collections::HashMap, fs, path::Path};

use syn::{
    GenericArgument, ImplItem, Item, ItemImpl, ItemMod, ItemUse, PathArguments, Type, UseTree,
};

use crate::{
    manifest::Workspace,
    model::{
        CallableId, FunctionRecord, ItemId, ItemKind, ItemRecord, MethodRecord, Project, SourceFile,
    },
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
            parser.parse_file(
                &package.name,
                Vec::new(),
                &package.lib_path,
                &package.root.join("src"),
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
    ) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = file_path.canonicalize()?;
        if self.files.contains_key(&file_path) {
            return Ok(());
        }

        let text = fs::read_to_string(&file_path)?;
        let syntax = syn::parse_file(&text)
            .map_err(|error| format!("failed to parse {}: {error}", file_path.display()))?;
        let aliases = collect_aliases(&syntax.items);
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
            if let Item::Mod(item_mod) = item {
                self.parse_external_module(package, &module_path, module_dir, &item_mod)?;
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
                    if let Some((_, items)) = &item_mod.content {
                        let mut child_path = module_path.to_vec();
                        child_path.push(item_mod.ident.to_string());
                        let child_aliases = collect_aliases(items);
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
        let Some(type_path) = local_type_path(module_path, &item_impl.self_ty) else {
            return Ok(());
        };
        let trait_path = item_impl
            .trait_
            .as_ref()
            .map(|(_, path, _)| normalized_path(module_path, path));
        let trait_input_type_paths = item_impl
            .trait_
            .as_ref()
            .map(|(_, path, _)| trait_input_type_paths(module_path, path))
            .unwrap_or_default();

        for impl_item in &item_impl.items {
            if let ImplItem::Fn(method) = impl_item {
                let id = CallableId::Method {
                    package: package.to_string(),
                    type_path: type_path.clone(),
                    trait_path: trait_path.clone(),
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
        item_mod: &ItemMod,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if item_mod.content.is_some() {
            return Ok(());
        }
        if item_mod.attrs.iter().any(is_cfg_test_attr) {
            return Ok(());
        }

        let name = item_mod.ident.to_string();
        let file_path = module_dir.join(format!("{name}.rs"));
        let mod_path = module_dir.join(&name).join("mod.rs");
        let (next_file, next_dir) = if file_path.exists() {
            let dir = file_path
                .parent()
                .expect("module file should have a parent")
                .to_path_buf();
            (file_path, dir)
        } else if mod_path.exists() {
            (mod_path, module_dir.join(&name))
        } else {
            return Err(format!("module {name} has no matching source file").into());
        };

        let mut child_path = module_path.to_vec();
        child_path.push(name);
        self.parse_file(package, child_path, &next_file, &next_dir)
    }
}

fn is_cfg_test_attr(attribute: &syn::Attribute) -> bool {
    if !attribute.path().is_ident("cfg") {
        return false;
    }

    attribute
        .parse_args::<syn::Ident>()
        .is_ok_and(|ident| ident == "test")
}

fn item_name_and_kind(item: &Item) -> Option<(String, ItemKind)> {
    match item {
        Item::Struct(item) => Some((item.ident.to_string(), ItemKind::Struct)),
        Item::Enum(item) => Some((item.ident.to_string(), ItemKind::Enum)),
        Item::Union(item) => Some((item.ident.to_string(), ItemKind::Union)),
        Item::Type(item) => Some((item.ident.to_string(), ItemKind::Type)),
        Item::Trait(item) => Some((item.ident.to_string(), ItemKind::Trait)),
        Item::Const(item) => Some((item.ident.to_string(), ItemKind::Const)),
        Item::Static(item) => Some((item.ident.to_string(), ItemKind::Static)),
        Item::Macro(item) => item
            .ident
            .as_ref()
            .map(|ident| (ident.to_string(), ItemKind::Macro)),
        _ => None,
    }
}

fn collect_aliases(items: &[Item]) -> HashMap<String, Vec<String>> {
    let mut aliases = HashMap::new();
    for item in items {
        if let Item::Use(ItemUse { tree, .. }) = item {
            collect_use_tree(tree, Vec::new(), &mut aliases);
        }
    }
    aliases
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

fn trait_input_type_paths(module_path: &[String], path: &syn::Path) -> Vec<Vec<String>> {
    let mut type_paths = Vec::new();
    for segment in &path.segments {
        if let PathArguments::AngleBracketed(arguments) = &segment.arguments {
            for argument in &arguments.args {
                if let GenericArgument::Type(ty) = argument {
                    collect_type_paths(module_path, ty, &mut type_paths);
                }
            }
        }
    }
    type_paths.sort();
    type_paths.dedup();
    type_paths
}

fn collect_type_paths(module_path: &[String], ty: &Type, type_paths: &mut Vec<Vec<String>>) {
    match ty {
        Type::Path(type_path) => {
            if let Some(path) = local_type_path(module_path, ty) {
                type_paths.push(path);
            }
            for segment in &type_path.path.segments {
                if let PathArguments::AngleBracketed(arguments) = &segment.arguments {
                    for argument in &arguments.args {
                        if let GenericArgument::Type(ty) = argument {
                            collect_type_paths(module_path, ty, type_paths);
                        }
                    }
                }
            }
        }
        Type::Reference(reference) => collect_type_paths(module_path, &reference.elem, type_paths),
        _ => {}
    }
}
