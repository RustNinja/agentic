use std::{fs, path::Path};

use syn::{ImplItem, Item, Type, UseTree};

use crate::{
    model::{CallableId, ItemId, ItemKind, Project, ReducedProject},
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

        for source in project
            .files
            .values()
            .filter(|source| &source.package == package_name)
        {
            let transformed = transform_file(
                reduced,
                &source.package,
                &source.module_path,
                &source.syntax,
            );
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

fn write_workspace_manifest(
    project: &Project,
    reduced: &ReducedProject,
    output_root: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut text = String::new();
    text.push_str("[workspace]\n");
    text.push_str("members = [\n");
    for member in &project.workspace.members {
        let Some(package_name) = project
            .workspace
            .packages
            .values()
            .find(|package| {
                package.root
                    == project
                        .workspace
                        .root
                        .join(member)
                        .canonicalize()
                        .unwrap_or_default()
            })
            .map(|package| package.name.as_str())
        else {
            continue;
        };
        if reduced.packages.contains(package_name) {
            text.push_str(&format!("    \"{package_name}\",\n"));
        }
    }
    text.push_str("]\n");
    text.push_str("resolver = \"2\"\n\n");
    text.push_str("[workspace.package]\n");
    text.push_str("edition = \"2021\"\n");
    text.push_str("version = \"0.1.0\"\n");
    text.push_str("license = \"MIT\"\n");

    fs::write(output_root.join("Cargo.toml"), text)?;
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

    let mut text = String::new();
    text.push_str("[package]\n");
    text.push_str(&format!("name = \"{}\"\n", package.name));
    text.push_str("version.workspace = true\n");
    text.push_str("edition.workspace = true\n");
    text.push_str("license.workspace = true\n");

    let dependencies = package
        .dependencies
        .iter()
        .filter(|dependency| {
            dependency.package != "opensourced" && reduced.packages.contains(&dependency.package)
        })
        .collect::<Vec<_>>();

    if !dependencies.is_empty() {
        text.push_str("\n[dependencies]\n");
        for dependency in dependencies {
            if dependency.alias == dependency.package {
                text.push_str(&format!(
                    "{} = {{ path = \"../{}\" }}\n",
                    dependency.alias, dependency.package
                ));
            } else {
                text.push_str(&format!(
                    "{} = {{ package = \"{}\", path = \"../{}\" }}\n",
                    dependency.alias, dependency.package, dependency.package
                ));
            }
        }
    }

    fs::write(output_path, text)?;
    Ok(())
}

fn transform_file(
    reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
    syntax: &syn::File,
) -> syn::File {
    let mut transformed = syntax.clone();
    transformed.items = transform_items(reduced, package, module_path, &syntax.items);
    transformed
}

fn transform_items(
    reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
    items: &[Item],
) -> Vec<Item> {
    items
        .iter()
        .filter_map(|item| match item {
            _ if item_is_test(item) => None,
            Item::Use(item_use) if use_mentions_opensourced(&item_use.tree) => None,
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
            Item::Struct(_)
            | Item::Enum(_)
            | Item::Union(_)
            | Item::Type(_)
            | Item::Trait(_)
            | Item::Const(_)
            | Item::Static(_) => {
                let id = item_id(package, module_path, item)?;
                reduced.reachable_items.contains(&id).then(|| item.clone())
            }
            Item::Impl(item_impl) => {
                let type_path = local_type_path(module_path, &item_impl.self_ty)?;
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
                if let Some((brace, child_items)) = &item_mod.content {
                    let mut child_path = module_path.to_vec();
                    child_path.push(item_mod.ident.to_string());
                    item_mod.content = Some((
                        *brace,
                        transform_items(reduced, package, &child_path, child_items),
                    ));
                }
                Some(Item::Mod(item_mod))
            }
            _ => Some(item.clone()),
        })
        .collect()
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
        _ => return None,
    };

    Some(ItemId {
        package: package.to_string(),
        module_path: module_path.to_vec(),
        name,
        kind,
    })
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
