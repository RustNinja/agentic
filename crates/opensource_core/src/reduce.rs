use std::{
    collections::{BTreeMap, BTreeSet, HashMap, VecDeque},
    path::{Path as FsPath, PathBuf},
};

use proc_macro2::{Delimiter, Literal, TokenStream, TokenTree};
use quote::ToTokens;
use syn::{
    parse::Parser,
    visit::{self, Visit},
    Expr, ExprCall, ExprIndex, ExprMacro, ExprMatch, ExprMethodCall, ExprPath, ExprStruct, Field,
    FnArg, GenericArgument, ImplItem, Item, ItemMacro, Local, Macro, Member, Meta, Pat,
    PatTupleStruct, Path, PathArguments, ReturnType, Stmt, Type, TypePath, UseTree,
};
use toml::Value;

use crate::model::{
    CallableId, CappedMethodFallbackEvidence, ItemId, ItemKind, Project, ReducedProject,
    ReductionEvidence, RootId, SemanticDependencies, SemanticReductionHints,
};

const MAX_UNRESOLVED_METHOD_NAME_CANDIDATES: usize = 1;
const MAX_UNRESOLVED_CONVERSION_CANDIDATES: usize = 24;

pub fn reduce_with_extra_roots(
    project: &Project,
    extra_roots: &[RootId],
) -> Result<ReducedProject, Box<dyn std::error::Error>> {
    reduce_with_extra_roots_and_semantics(project, extra_roots, &SemanticReductionHints::default())
}

pub fn reduce_with_extra_roots_and_semantics(
    project: &Project,
    extra_roots: &[RootId],
    semantic_hints: &SemanticReductionHints,
) -> Result<ReducedProject, Box<dyn std::error::Error>> {
    let mut callable_roots = project
        .functions
        .values()
        .filter(|record| {
            record
                .item
                .attrs
                .iter()
                .any(|attribute| is_opensourced_attr(attribute.path()))
        })
        .map(|record| record.id.clone())
        .chain(
            project
                .methods
                .iter()
                .filter(|(_, record)| {
                    record
                        .item
                        .attrs
                        .iter()
                        .any(|attribute| is_opensourced_attr(attribute.path()))
                })
                .map(|(id, _)| id.clone()),
        )
        .collect::<Vec<_>>();
    callable_roots.sort();
    callable_roots.dedup();

    let mut item_roots = project
        .items
        .iter()
        .filter(|(_, record)| item_has_opensourced_attr(&record.item))
        .map(|(id, _)| id.clone())
        .collect::<Vec<_>>();
    item_roots.sort();
    item_roots.dedup();

    for root in extra_roots {
        match root {
            RootId::Callable(callable) => {
                if project.functions.contains_key(callable)
                    || project.methods.contains_key(callable)
                {
                    callable_roots.push(callable.clone());
                }
            }
            RootId::Item(item) => {
                if project.items.contains_key(item) {
                    item_roots.push(item.clone());
                }
            }
        }
    }
    callable_roots.sort();
    callable_roots.dedup();
    item_roots.sort();
    item_roots.dedup();

    let mut roots = callable_roots
        .iter()
        .cloned()
        .map(RootId::Callable)
        .chain(item_roots.iter().cloned().map(RootId::Item))
        .collect::<Vec<_>>();
    roots.sort();
    roots.dedup();

    if roots.is_empty() {
        return Err("expected at least one #[opensourced] function or item, found 0".into());
    }
    let root = roots
        .first()
        .expect("roots were checked as non-empty")
        .clone();

    let mut candidate_packages = BTreeSet::new();
    for root in &roots {
        candidate_packages.extend(package_closure(project, root.package()));
    }
    let mut reachable = BTreeSet::new();
    let mut reachable_items = BTreeSet::new();
    let mut evidence = ReductionEvidence::default();
    let mut callable_queue = VecDeque::from(callable_roots);
    let mut item_queue = VecDeque::from(item_roots);

    while !callable_queue.is_empty() || !item_queue.is_empty() {
        while let Some(callable) = callable_queue.pop_front() {
            if !candidate_packages.contains(callable.package())
                || !reachable.insert(callable.clone())
            {
                continue;
            }

            let mut dependencies = callable_dependencies(project, &callable);
            dependencies.extend(semantic_callable_dependencies(semantic_hints, &callable));
            evidence.add(&dependencies.evidence);
            for dependency in dependencies.callables {
                if candidate_packages.contains(dependency.package())
                    && !reachable.contains(&dependency)
                {
                    callable_queue.push_back(dependency);
                }
            }
            for item in dependencies.items {
                if candidate_packages.contains(item.package()) && !reachable_items.contains(&item) {
                    item_queue.push_back(item);
                }
            }
        }

        while let Some(item) = item_queue.pop_front() {
            if !candidate_packages.contains(item.package()) || !reachable_items.insert(item.clone())
            {
                continue;
            }

            let mut dependencies = item_dependencies(project, &item);
            dependencies.extend(semantic_item_dependencies(semantic_hints, &item));
            evidence.add(&dependencies.evidence);
            for dependency in dependencies.callables {
                if candidate_packages.contains(dependency.package())
                    && !reachable.contains(&dependency)
                {
                    callable_queue.push_back(dependency);
                }
            }
            for item in dependencies.items {
                if candidate_packages.contains(item.package()) && !reachable_items.contains(&item) {
                    item_queue.push_back(item);
                }
            }
        }

        add_source_mentioned_dependency_packages(
            project,
            &mut candidate_packages,
            &reachable,
            &reachable_items,
        );

        let dependencies = externally_referenced_public_reexport_dependencies(
            project,
            &candidate_packages,
            &reachable,
            &reachable_items,
        );
        evidence.add(&dependencies.evidence);
        for dependency in dependencies.callables {
            if candidate_packages.contains(dependency.package()) && !reachable.contains(&dependency)
            {
                callable_queue.push_back(dependency);
            }
        }
        for item in dependencies.items {
            if candidate_packages.contains(item.package()) && !reachable_items.contains(&item) {
                item_queue.push_back(item);
            }
        }

        let dependencies = retained_item_macro_dependencies(
            project,
            &candidate_packages,
            &reachable,
            &reachable_items,
        );
        evidence.add(&dependencies.evidence);
        for dependency in dependencies.callables {
            if candidate_packages.contains(dependency.package()) && !reachable.contains(&dependency)
            {
                callable_queue.push_back(dependency);
            }
        }
        for item in dependencies.items {
            if candidate_packages.contains(item.package()) && !reachable_items.contains(&item) {
                item_queue.push_back(item);
            }
        }

        let dependencies = retained_rendered_use_dependencies(
            project,
            &candidate_packages,
            &reachable,
            &reachable_items,
        );
        evidence.add(&dependencies.evidence);
        for dependency in dependencies.callables {
            if candidate_packages.contains(dependency.package()) && !reachable.contains(&dependency)
            {
                callable_queue.push_back(dependency);
            }
        }
        for item in dependencies.items {
            if candidate_packages.contains(item.package()) && !reachable_items.contains(&item) {
                item_queue.push_back(item);
            }
        }

        let dependencies = reachable_struct_field_dependencies(
            project,
            &candidate_packages,
            &reachable,
            &reachable_items,
        );
        evidence.add(&dependencies.evidence);
        for dependency in dependencies.callables {
            if candidate_packages.contains(dependency.package()) && !reachable.contains(&dependency)
            {
                callable_queue.push_back(dependency);
            }
        }
        for item in dependencies.items {
            if candidate_packages.contains(item.package()) && !reachable_items.contains(&item) {
                item_queue.push_back(item);
            }
        }

        let dependencies = reachable_trait_impl_surface_dependencies(
            project,
            &candidate_packages,
            &reachable,
            &reachable_items,
        );
        evidence.add(&dependencies.evidence);
        for dependency in dependencies.callables {
            if candidate_packages.contains(dependency.package()) && !reachable.contains(&dependency)
            {
                callable_queue.push_back(dependency);
            }
        }
        for item in dependencies.items {
            if candidate_packages.contains(item.package()) && !reachable_items.contains(&item) {
                item_queue.push_back(item);
            }
        }

        let dependencies = reachable_macro_impl_surface_dependencies(
            project,
            &candidate_packages,
            &roots,
            &reachable,
            &reachable_items,
        );
        evidence.add(&dependencies.evidence);
        for dependency in dependencies.callables {
            if candidate_packages.contains(dependency.package()) && !reachable.contains(&dependency)
            {
                callable_queue.push_back(dependency);
            }
        }
        for item in dependencies.items {
            if candidate_packages.contains(item.package()) && !reachable_items.contains(&item) {
                item_queue.push_back(item);
            }
        }
    }

    let mut packages = reachable_packages(&roots, &reachable, &reachable_items);
    add_source_mentioned_dependency_packages(project, &mut packages, &reachable, &reachable_items);
    retain_referenced_public_reexport_dependencies(
        project,
        &mut packages,
        &mut reachable,
        &mut reachable_items,
        &mut evidence,
    );
    let proc_macro_dependency_packages = local_proc_macro_dependency_packages(project, &packages);
    retain_proc_macro_exports(
        project,
        &proc_macro_dependency_packages,
        &packages,
        &mut reachable,
        &mut reachable_items,
        &mut evidence,
    );
    let build_dependency_packages = add_local_build_dependency_packages(project, &mut packages);
    retain_entire_packages(
        project,
        &build_dependency_packages,
        &mut reachable,
        &mut reachable_items,
    );
    let library_support_dependency_packages =
        add_local_library_support_dependency_packages(project, &mut packages);
    retain_entire_packages(
        project,
        &library_support_dependency_packages,
        &mut reachable,
        &mut reachable_items,
    );
    retain_referenced_public_reexport_dependencies(
        project,
        &mut packages,
        &mut reachable,
        &mut reachable_items,
        &mut evidence,
    );

    Ok(ReducedProject {
        root,
        roots,
        packages,
        reachable,
        reachable_items,
        evidence,
    })
}

pub fn is_opensourced_attr(path: &Path) -> bool {
    path.segments
        .last()
        .is_some_and(|segment| segment.ident == "opensourced")
}

fn item_has_opensourced_attr(item: &Item) -> bool {
    match item {
        Item::Const(item) => &item.attrs,
        Item::Enum(item) => &item.attrs,
        Item::Macro(item) => &item.attrs,
        Item::Mod(item) => &item.attrs,
        Item::Static(item) => &item.attrs,
        Item::Struct(item) => &item.attrs,
        Item::Trait(item) => &item.attrs,
        Item::Type(item) => &item.attrs,
        Item::Union(item) => &item.attrs,
        _ => return false,
    }
    .iter()
    .any(|attribute| is_opensourced_attr(attribute.path()))
}

pub fn is_test_attr(path: &Path) -> bool {
    path.segments
        .last()
        .is_some_and(|segment| segment.ident == "test")
}

pub fn is_cfg_test_attr(attribute: &syn::Attribute) -> bool {
    if !attribute.path().is_ident("cfg") {
        return false;
    }

    attribute
        .parse_args::<Meta>()
        .is_ok_and(|meta| cfg_meta_is_excluded(&meta))
}

fn cfg_meta_is_excluded(meta: &Meta) -> bool {
    matches!(meta, Meta::Path(path) if path.is_ident("test"))
}

fn package_closure(project: &Project, root: &str) -> BTreeSet<String> {
    let mut packages = BTreeSet::new();
    let mut queue = VecDeque::from([root.to_string()]);

    while let Some(package) = queue.pop_front() {
        if !packages.insert(package.clone()) {
            continue;
        }

        let Some(package_record) = project.workspace.packages.get(&package) else {
            continue;
        };

        for dependency in &package_record.dependencies {
            if dependency.package == "opensourced" {
                continue;
            }
            if project.workspace.packages.contains_key(&dependency.package) {
                queue.push_back(dependency.package.clone());
            }
        }
    }

    packages
}

fn reachable_packages(
    roots: &[RootId],
    reachable: &BTreeSet<CallableId>,
    reachable_items: &BTreeSet<ItemId>,
) -> BTreeSet<String> {
    let mut packages = roots
        .iter()
        .map(|root| root.package().to_string())
        .collect::<BTreeSet<_>>();
    packages.extend(
        reachable
            .iter()
            .map(|callable| callable.package().to_string()),
    );
    packages.extend(
        reachable_items
            .iter()
            .map(|item| item.package().to_string()),
    );
    packages
}

fn retained_item_macro_dependencies(
    project: &Project,
    candidate_packages: &BTreeSet<String>,
    reachable: &BTreeSet<CallableId>,
    reachable_items: &BTreeSet<ItemId>,
) -> DependencySet {
    let mut dependencies = DependencySet::default();
    for source in project
        .files
        .values()
        .filter(|source| candidate_packages.contains(&source.package))
    {
        collect_retained_item_macro_dependencies(
            project,
            reachable,
            reachable_items,
            &source.package,
            &source.module_path,
            &source.syntax.items,
            &mut dependencies,
        );
    }
    dependencies
}

fn collect_retained_item_macro_dependencies(
    project: &Project,
    reachable: &BTreeSet<CallableId>,
    reachable_items: &BTreeSet<ItemId>,
    package: &str,
    module_path: &[String],
    items: &[Item],
    dependencies: &mut DependencySet,
) {
    let aliases = project
        .module_aliases
        .get(&(package.to_string(), module_path.to_vec()))
        .cloned()
        .unwrap_or_default();
    let resolver = Resolver {
        project,
        package,
        module_path,
        aliases: &aliases,
        self_type: None,
    };

    for item in items {
        match item {
            Item::Macro(item_macro)
                if item_macro.ident.is_none()
                    && should_scan_item_macro_dependencies(
                        project,
                        reachable,
                        reachable_items,
                        package,
                        module_path,
                        item_macro,
                    ) =>
            {
                let mut visitor = DependencyVisitor::new(resolver.clone());
                visitor.add_macro_path(&item_macro.mac.path);
                visitor.add_macro_token_dependencies(&item_macro.mac.tokens);
                dependencies.extend(visitor.dependencies);
            }
            Item::Mod(item_mod) => {
                if let Some((_, child_items)) = &item_mod.content {
                    let mut child_path = module_path.to_vec();
                    child_path.push(item_mod.ident.to_string());
                    collect_retained_item_macro_dependencies(
                        project,
                        reachable,
                        reachable_items,
                        package,
                        &child_path,
                        child_items,
                        dependencies,
                    );
                }
            }
            _ => {}
        }
    }
}

fn retained_rendered_use_dependencies(
    project: &Project,
    candidate_packages: &BTreeSet<String>,
    reachable: &BTreeSet<CallableId>,
    reachable_items: &BTreeSet<ItemId>,
) -> DependencySet {
    let mut dependencies = DependencySet::default();
    let mut module_ident_cache = HashMap::new();
    let mut package_ident_cache = HashMap::new();
    let mut public_reexport_ident_cache = HashMap::new();
    let mut visible_name_usage_cache = HashMap::new();
    let callable_idents_by_package = reachable_callable_idents_by_package(project, reachable);
    for source in project
        .files
        .values()
        .filter(|source| candidate_packages.contains(&source.package))
        .filter(|source| {
            module_has_reachable_code(project, reachable, reachable_items, source)
                || (source.module_path.is_empty()
                    && package_public_api_is_referenced_by_reachable_packages(
                        project,
                        reachable,
                        reachable_items,
                        &source.package,
                    ))
        })
    {
        let aliases = project
            .module_aliases
            .get(&(source.package.clone(), source.module_path.clone()))
            .cloned()
            .unwrap_or_default();
        let resolver = Resolver {
            project,
            package: &source.package,
            module_path: &source.module_path,
            aliases: &aliases,
            self_type: None,
        };
        let module_idents = reachable_import_scope_source_idents_cached(
            project,
            reachable,
            reachable_items,
            &source.package,
            &source.module_path,
            &callable_idents_by_package,
            &mut module_ident_cache,
        );
        let mut package_idents = package_ident_cache
            .entry(source.package.clone())
            .or_insert_with(|| {
                reachable_package_source_idents(
                    project,
                    reachable,
                    reachable_items,
                    &source.package,
                    &callable_idents_by_package,
                )
            })
            .clone();
        let public_reexport_idents = public_reexport_ident_cache
            .entry(source.package.clone())
            .or_insert_with(|| {
                public_reexport_idents_referenced_by_reachable_packages(
                    project,
                    reachable,
                    reachable_items,
                    &source.package,
                )
            });
        package_idents.extend(public_reexport_idents.iter().cloned());

        for item in &source.syntax.items {
            let syn::Item::Use(item_use) = item else {
                continue;
            };
            let reachable_idents = if matches!(item_use.vis, syn::Visibility::Public(_)) {
                &package_idents
            } else {
                &module_idents
            };
            let mut named_paths = Vec::new();
            let mut glob_paths = Vec::new();
            collect_use_dependency_paths(
                &item_use.tree,
                Vec::new(),
                &mut named_paths,
                &mut glob_paths,
            );

            for path in named_paths {
                let visible_name_is_used = if path.renamed {
                    reachable_import_scope_uses_visible_name_cached(
                        project,
                        reachable,
                        reachable_items,
                        &source.package,
                        &source.module_path,
                        &path.visible_name,
                        &mut visible_name_usage_cache,
                    )
                } else {
                    reachable_idents.contains(&path.visible_name)
                };
                if visible_name_is_used {
                    dependencies.extend(resolve_use_named_dependency(&resolver, &path.segments));
                }
            }
            for path in glob_paths {
                dependencies.extend(resolve_use_glob_dependencies(
                    &resolver,
                    &path,
                    reachable_idents,
                ));
            }
        }
    }
    dependencies
}

fn reachable_trait_impl_surface_dependencies(
    project: &Project,
    candidate_packages: &BTreeSet<String>,
    reachable: &BTreeSet<CallableId>,
    reachable_items: &BTreeSet<ItemId>,
) -> DependencySet {
    let mut dependencies = DependencySet::default();
    for package in candidate_packages {
        for module_path in project_module_paths(project, package) {
            let Some(items) = module_items_for_path(project, package, &module_path) else {
                continue;
            };
            let aliases = project
                .module_aliases
                .get(&(package.clone(), module_path.clone()))
                .cloned()
                .unwrap_or_default();
            let resolver = Resolver {
                project,
                package,
                module_path: &module_path,
                aliases: &aliases,
                self_type: None,
            };
            for item in items {
                let Item::Impl(item_impl) = item else {
                    continue;
                };
                if item_impl.trait_.is_none() {
                    continue;
                }
                let Some(self_type) = resolver.resolve_receiver_type(&item_impl.self_ty) else {
                    continue;
                };
                if !resolver
                    .type_ref_candidates(&self_type)
                    .iter()
                    .any(|type_ref| type_ref_item_is_reachable(reachable_items, type_ref))
                {
                    continue;
                }
                let Some(trait_item) = trait_item_for_impl(&resolver, item_impl) else {
                    continue;
                };
                if trait_item_has_public_surface(project, &trait_item) {
                    dependencies.items.insert(trait_item.clone());
                    let impl_resolver = Resolver {
                        project,
                        package,
                        module_path: &module_path,
                        aliases: &aliases,
                        self_type: Some(self_type.clone()),
                    };
                    let mut visitor = DependencyVisitor::new(impl_resolver);
                    visitor.add_generic_trait_bounds(&item_impl.generics);
                    for attr in &item_impl.attrs {
                        visitor.visit_attribute(attr);
                    }
                    visitor.visit_type(&item_impl.self_ty);
                    if let Some((_, trait_path, _)) = &item_impl.trait_ {
                        visitor.add_item_path(trait_path);
                    }
                    let trait_path = path_from_item(&trait_item);
                    for impl_item in &item_impl.items {
                        if trait_impl_item_should_scan_surface_dependencies(
                            project,
                            reachable,
                            package,
                            &self_type.type_path,
                            &trait_path,
                            &trait_item,
                            item_impl,
                            impl_item,
                        ) {
                            visitor.visit_impl_item(impl_item);
                        }
                    }
                    dependencies.extend(visitor.dependencies);
                }
            }
        }
    }
    dependencies
}

#[allow(clippy::too_many_arguments)]
fn trait_impl_item_should_scan_surface_dependencies(
    project: &Project,
    reachable: &BTreeSet<CallableId>,
    package: &str,
    type_path: &[String],
    trait_path: &[String],
    trait_item: &ItemId,
    item_impl: &syn::ItemImpl,
    impl_item: &ImplItem,
) -> bool {
    if impl_item_is_test(impl_item) {
        return false;
    }
    if item_impl
        .attrs
        .iter()
        .any(attr_requires_impl_surface_retention)
    {
        return true;
    }
    if impl_item_attrs_require_surface_retention(impl_item) {
        return true;
    }
    if trait_impl_item_is_required_by_trait(project, trait_item, impl_item) {
        return true;
    }
    let ImplItem::Fn(method) = impl_item else {
        return false;
    };
    reachable.iter().any(|callable| {
        matches!(
            callable,
            CallableId::Method {
                package: callable_package,
                type_path: callable_type_path,
                trait_path: Some(callable_trait_path),
                method: callable_method,
                ..
            } if callable_package == package
                && callable_type_path == type_path
                && callable_trait_path == trait_path
                && callable_method == &method.sig.ident.to_string()
        )
    })
}

fn trait_impl_item_is_required_by_trait(
    project: &Project,
    trait_item: &ItemId,
    impl_item: &ImplItem,
) -> bool {
    let Some(record) = project.items.get(trait_item) else {
        return false;
    };
    let Item::Trait(item_trait) = &record.item else {
        return false;
    };
    match impl_item {
        ImplItem::Fn(method) => item_trait.items.iter().any(|trait_item| {
            let syn::TraitItem::Fn(function) = trait_item else {
                return false;
            };
            function.sig.ident == method.sig.ident && function.default.is_none()
        }),
        ImplItem::Const(item) => item_trait.items.iter().any(|trait_item| {
            let syn::TraitItem::Const(constant) = trait_item else {
                return false;
            };
            constant.ident == item.ident && constant.default.is_none()
        }),
        ImplItem::Type(item) => item_trait.items.iter().any(|trait_item| {
            let syn::TraitItem::Type(associated_type) = trait_item else {
                return false;
            };
            associated_type.ident == item.ident && associated_type.default.is_none()
        }),
        _ => false,
    }
}

fn impl_item_attrs_require_surface_retention(impl_item: &ImplItem) -> bool {
    match impl_item {
        ImplItem::Const(item) => item.attrs.iter().any(attr_requires_impl_surface_retention),
        ImplItem::Fn(item) => item.attrs.iter().any(attr_requires_impl_surface_retention),
        ImplItem::Macro(item) => item.attrs.iter().any(attr_requires_impl_surface_retention),
        ImplItem::Type(item) => item.attrs.iter().any(attr_requires_impl_surface_retention),
        ImplItem::Verbatim(_) => false,
        _ => false,
    }
}

fn trait_item_type_surface_dependency_should_remain(item: &syn::TraitItem) -> bool {
    if trait_item_attrs_require_surface_retention(item) {
        return true;
    }
    match item {
        syn::TraitItem::Fn(method) => method.default.is_none(),
        syn::TraitItem::Const(item) => item.default.is_none(),
        syn::TraitItem::Type(item) => item.default.is_none(),
        syn::TraitItem::Macro(_) | syn::TraitItem::Verbatim(_) => true,
        _ => true,
    }
}

fn trait_item_attrs_require_surface_retention(item: &syn::TraitItem) -> bool {
    let attrs = match item {
        syn::TraitItem::Const(item) => &item.attrs,
        syn::TraitItem::Fn(item) => &item.attrs,
        syn::TraitItem::Macro(item) => &item.attrs,
        syn::TraitItem::Type(item) => &item.attrs,
        syn::TraitItem::Verbatim(_) => return false,
        _ => return false,
    };
    attrs.iter().any(attr_requires_impl_surface_retention)
}

fn externally_referenced_public_reexport_dependencies(
    project: &Project,
    candidate_packages: &BTreeSet<String>,
    reachable: &BTreeSet<CallableId>,
    reachable_items: &BTreeSet<ItemId>,
) -> DependencySet {
    let mut dependencies = DependencySet::default();
    let mut public_ident_cache = HashMap::new();
    for source in project
        .files
        .values()
        .filter(|source| candidate_packages.contains(&source.package))
    {
        let public_idents = public_ident_cache
            .entry(source.package.clone())
            .or_insert_with(|| {
                public_reexport_idents_referenced_by_reachable_packages(
                    project,
                    reachable,
                    reachable_items,
                    &source.package,
                )
            });
        if public_idents.is_empty() {
            continue;
        }

        let aliases = project
            .module_aliases
            .get(&(source.package.clone(), source.module_path.clone()))
            .cloned()
            .unwrap_or_default();
        let resolver = Resolver {
            project,
            package: &source.package,
            module_path: &source.module_path,
            aliases: &aliases,
            self_type: None,
        };

        for item in &source.syntax.items {
            let syn::Item::Use(item_use) = item else {
                continue;
            };
            if !matches!(item_use.vis, syn::Visibility::Public(_)) {
                continue;
            }

            let mut named_paths = Vec::new();
            let mut glob_paths = Vec::new();
            collect_use_dependency_paths(
                &item_use.tree,
                Vec::new(),
                &mut named_paths,
                &mut glob_paths,
            );
            for path in named_paths {
                if public_idents.contains(&path.visible_name) {
                    let mut path_dependencies =
                        resolve_use_named_dependency(&resolver, &path.segments);
                    if path_dependencies.is_empty() {
                        path_dependencies.extend(unresolved_public_reexport_macro_dependencies(
                            project,
                            &resolver,
                            &path.segments,
                        ));
                    }
                    dependencies.extend(path_dependencies);
                }
            }
        }
    }
    dependencies
}

fn unresolved_public_reexport_macro_dependencies(
    project: &Project,
    resolver: &Resolver<'_>,
    path: &[String],
) -> DependencySet {
    let mut dependencies = DependencySet::default();
    if path.len() < 2 {
        return dependencies;
    }
    let Some((package, module_path)) = resolver.resolve_value_prefix(&path[..path.len() - 1])
    else {
        return dependencies;
    };
    let Some(source) = project
        .files
        .values()
        .find(|source| source.package == package && source.module_path == module_path)
    else {
        return dependencies;
    };
    let aliases = project
        .module_aliases
        .get(&(source.package.clone(), source.module_path.clone()))
        .cloned()
        .unwrap_or_default();
    let resolver = Resolver {
        project,
        package: &source.package,
        module_path: &source.module_path,
        aliases: &aliases,
        self_type: None,
    };
    for item in &source.syntax.items {
        let syn::Item::Macro(item_macro) = item else {
            continue;
        };
        if item_macro.ident.is_some() {
            continue;
        }
        let mut visitor = DependencyVisitor::new(resolver.clone());
        visitor.add_macro_path(&item_macro.mac.path);
        visitor.add_macro_token_dependencies(&item_macro.mac.tokens);
        dependencies.extend(visitor.dependencies);
    }
    dependencies
}

fn retain_referenced_public_reexport_dependencies(
    project: &Project,
    packages: &mut BTreeSet<String>,
    reachable: &mut BTreeSet<CallableId>,
    reachable_items: &mut BTreeSet<ItemId>,
    evidence: &mut ReductionEvidence,
) {
    loop {
        let package_count = packages.len();
        add_source_mentioned_dependency_packages(project, packages, reachable, reachable_items);
        let packages_changed = packages.len() != package_count;
        let dependencies = externally_referenced_public_reexport_dependencies(
            project,
            packages,
            reachable,
            reachable_items,
        );
        let changed = retain_dependency_set(
            project,
            packages,
            reachable,
            reachable_items,
            dependencies,
            evidence,
        );
        if !changed && !packages_changed {
            break;
        }
    }
}

fn retain_dependency_set(
    project: &Project,
    candidate_packages: &BTreeSet<String>,
    reachable: &mut BTreeSet<CallableId>,
    reachable_items: &mut BTreeSet<ItemId>,
    dependencies: DependencySet,
    evidence: &mut ReductionEvidence,
) -> bool {
    let mut changed = false;
    evidence.add(&dependencies.evidence);
    let mut callable_queue = dependencies
        .callables
        .into_iter()
        .filter(|callable| {
            candidate_packages.contains(callable.package()) && !reachable.contains(callable)
        })
        .collect::<VecDeque<_>>();
    let mut item_queue = dependencies
        .items
        .into_iter()
        .filter(|item| {
            candidate_packages.contains(item.package()) && !reachable_items.contains(item)
        })
        .collect::<VecDeque<_>>();

    while !callable_queue.is_empty() || !item_queue.is_empty() {
        while let Some(callable) = callable_queue.pop_front() {
            if !candidate_packages.contains(callable.package())
                || !reachable.insert(callable.clone())
            {
                continue;
            }
            changed = true;
            let dependencies = callable_dependencies(project, &callable);
            evidence.add(&dependencies.evidence);
            for dependency in dependencies.callables {
                if candidate_packages.contains(dependency.package())
                    && !reachable.contains(&dependency)
                {
                    callable_queue.push_back(dependency);
                }
            }
            for item in dependencies.items {
                if candidate_packages.contains(item.package()) && !reachable_items.contains(&item) {
                    item_queue.push_back(item);
                }
            }
        }

        while let Some(item) = item_queue.pop_front() {
            if !candidate_packages.contains(item.package()) || !reachable_items.insert(item.clone())
            {
                continue;
            }
            changed = true;
            let dependencies = item_dependencies(project, &item);
            evidence.add(&dependencies.evidence);
            for dependency in dependencies.callables {
                if candidate_packages.contains(dependency.package())
                    && !reachable.contains(&dependency)
                {
                    callable_queue.push_back(dependency);
                }
            }
            for item in dependencies.items {
                if candidate_packages.contains(item.package()) && !reachable_items.contains(&item) {
                    item_queue.push_back(item);
                }
            }
        }
    }

    changed
}

fn reachable_source_idents(
    project: &Project,
    reachable: &BTreeSet<CallableId>,
    reachable_items: &BTreeSet<ItemId>,
    package: &str,
    module_path: &[String],
    callable_idents_by_package: &BTreeMap<String, BTreeSet<String>>,
) -> BTreeSet<String> {
    let mut idents = BTreeSet::new();
    for callable in reachable
        .iter()
        .filter(|callable| callable.package() == package)
    {
        if let Some(record) = project.functions.get(callable) {
            if record.module_path == module_path {
                collect_token_idents(&record.item.to_token_stream(), &mut idents);
            }
        }
        if let Some(record) = project.methods.get(callable) {
            if record.module_path == module_path {
                collect_token_idents(&record.item.to_token_stream(), &mut idents);
            }
        }
    }
    for item in reachable_items
        .iter()
        .filter(|item| item.package == package && item.module_path == module_path)
    {
        if let Some(record) = project.items.get(item) {
            collect_reachable_package_item_idents(
                project,
                reachable,
                item,
                record,
                &callable_idents_by_package,
                &mut idents,
            );
        }
    }
    idents
}

fn reachable_package_source_idents(
    project: &Project,
    reachable: &BTreeSet<CallableId>,
    reachable_items: &BTreeSet<ItemId>,
    package: &str,
    callable_idents_by_package: &BTreeMap<String, BTreeSet<String>>,
) -> BTreeSet<String> {
    let mut idents = BTreeSet::new();
    for callable in reachable
        .iter()
        .filter(|callable| callable.package() == package)
    {
        if let Some(record) = project.functions.get(callable) {
            collect_token_idents(&record.item.to_token_stream(), &mut idents);
        }
        if let Some(record) = project.methods.get(callable) {
            collect_token_idents(&record.item.to_token_stream(), &mut idents);
        }
    }
    for item in reachable_items
        .iter()
        .filter(|item| item.package == package)
    {
        if let Some(record) = project.items.get(item) {
            collect_reachable_package_item_idents(
                project,
                reachable,
                item,
                record,
                &callable_idents_by_package,
                &mut idents,
            );
        }
    }
    idents
}

fn reachable_import_scope_source_idents_cached(
    project: &Project,
    reachable: &BTreeSet<CallableId>,
    reachable_items: &BTreeSet<ItemId>,
    package: &str,
    module_path: &[String],
    callable_idents_by_package: &BTreeMap<String, BTreeSet<String>>,
    cache: &mut HashMap<(String, Vec<String>), BTreeSet<String>>,
) -> BTreeSet<String> {
    let key = (package.to_string(), module_path.to_vec());
    if let Some(idents) = cache.get(&key) {
        return idents.clone();
    }

    let mut idents = reachable_source_idents(
        project,
        reachable,
        reachable_items,
        package,
        module_path,
        callable_idents_by_package,
    );
    for source in project
        .files
        .values()
        .filter(|source| source.package == package)
        .filter(|source| source.module_path.len() == module_path.len() + 1)
        .filter(|source| path_has_prefix(&source.module_path, module_path))
        .filter(|source| module_has_reachable_code(project, reachable, reachable_items, source))
        .filter(|source| file_has_super_glob_import(&source.syntax))
    {
        idents.extend(reachable_import_scope_source_idents_cached(
            project,
            reachable,
            reachable_items,
            package,
            &source.module_path,
            callable_idents_by_package,
            cache,
        ));
    }

    cache.insert(key, idents.clone());
    idents
}

fn reachable_import_scope_uses_visible_name_cached(
    project: &Project,
    reachable: &BTreeSet<CallableId>,
    reachable_items: &BTreeSet<ItemId>,
    package: &str,
    module_path: &[String],
    visible_name: &str,
    cache: &mut HashMap<(String, Vec<String>, String), bool>,
) -> bool {
    let key = (
        package.to_string(),
        module_path.to_vec(),
        visible_name.to_string(),
    );
    if let Some(used) = cache.get(&key) {
        return *used;
    }

    let used = reachable_source_uses_visible_name(
        project,
        reachable,
        reachable_items,
        package,
        module_path,
        visible_name,
    ) || project
        .files
        .values()
        .filter(|source| source.package == package)
        .filter(|source| source.module_path.len() == module_path.len() + 1)
        .filter(|source| path_has_prefix(&source.module_path, module_path))
        .filter(|source| module_has_reachable_code(project, reachable, reachable_items, source))
        .filter(|source| file_has_super_glob_import(&source.syntax))
        .any(|source| {
            reachable_import_scope_uses_visible_name_cached(
                project,
                reachable,
                reachable_items,
                package,
                &source.module_path,
                visible_name,
                cache,
            )
        });

    cache.insert(key, used);
    used
}

fn reachable_source_uses_visible_name(
    project: &Project,
    reachable: &BTreeSet<CallableId>,
    reachable_items: &BTreeSet<ItemId>,
    package: &str,
    module_path: &[String],
    visible_name: &str,
) -> bool {
    for callable in reachable
        .iter()
        .filter(|callable| callable.package() == package)
    {
        if let Some(record) = project.functions.get(callable) {
            if record.module_path == module_path
                && item_fn_uses_import_visible_name(&record.item, visible_name)
            {
                return true;
            }
        }
        if let Some(record) = project.methods.get(callable) {
            if record.module_path == module_path
                && impl_item_fn_uses_import_visible_name(&record.item, visible_name)
            {
                return true;
            }
        }
    }
    for item in reachable_items
        .iter()
        .filter(|item| item.package == package && item.module_path == module_path)
    {
        if let Some(record) = project.items.get(item) {
            if item_uses_import_visible_name(&record.item, visible_name) {
                return true;
            }
        }
    }
    false
}

fn item_fn_uses_import_visible_name(item: &syn::ItemFn, visible_name: &str) -> bool {
    let mut visitor = ImportVisibleNameUseVisitor::new(visible_name);
    visitor.visit_item_fn(item);
    visitor.found
}

fn impl_item_fn_uses_import_visible_name(item: &syn::ImplItemFn, visible_name: &str) -> bool {
    let mut visitor = ImportVisibleNameUseVisitor::new(visible_name);
    visitor.visit_impl_item_fn(item);
    visitor.found
}

fn item_uses_import_visible_name(item: &Item, visible_name: &str) -> bool {
    let mut visitor = ImportVisibleNameUseVisitor::new(visible_name);
    visitor.visit_item(item);
    visitor.found
}

struct ImportVisibleNameUseVisitor<'a> {
    visible_name: &'a str,
    local_scopes: Vec<BTreeSet<String>>,
    found: bool,
}

impl<'a> ImportVisibleNameUseVisitor<'a> {
    fn new(visible_name: &'a str) -> Self {
        Self {
            visible_name,
            local_scopes: vec![BTreeSet::new()],
            found: false,
        }
    }

    fn push_scope(&mut self) {
        self.local_scopes.push(BTreeSet::new());
    }

    fn pop_scope(&mut self) {
        self.local_scopes.pop();
        if self.local_scopes.is_empty() {
            self.local_scopes.push(BTreeSet::new());
        }
    }

    fn insert_local_binding(&mut self, name: String) {
        self.local_scopes
            .last_mut()
            .expect("visitor should always have a local scope")
            .insert(name);
    }

    fn bind_local_pattern_names(&mut self, pattern: &Pat) {
        let scope = self
            .local_scopes
            .last_mut()
            .expect("visitor should always have a local scope");
        collect_pattern_ident_names(pattern, scope);
    }

    fn local_binding_visible(&self) -> bool {
        self.local_scopes
            .iter()
            .rev()
            .any(|scope| scope.contains(self.visible_name))
    }

    fn path_uses_visible_name(&self, path: &Path) -> bool {
        path.segments
            .first()
            .is_some_and(|segment| segment.ident == self.visible_name)
    }

    fn macro_tokens_use_visible_name(&self, tokens: &TokenStream) -> bool {
        if self.local_binding_visible() {
            return token_path_candidates(tokens).iter().any(|segments| {
                segments.len() > 1
                    && segments
                        .first()
                        .is_some_and(|segment| segment == self.visible_name)
            });
        }
        token_stream_mentions_ident(tokens, self.visible_name)
    }
}

impl<'ast> Visit<'ast> for ImportVisibleNameUseVisitor<'_> {
    fn visit_block(&mut self, block: &'ast syn::Block) {
        self.push_scope();
        visit::visit_block(self, block);
        self.pop_scope();
    }

    fn visit_local(&mut self, local: &'ast Local) {
        for attr in &local.attrs {
            self.visit_attribute(attr);
        }
        if let Some(init) = &local.init {
            self.visit_expr(&init.expr);
            if let Some((_, diverge)) = &init.diverge {
                self.visit_expr(diverge);
            }
        }
        self.bind_local_pattern_names(&local.pat);
        self.visit_pat(&local.pat);
    }

    fn visit_arm(&mut self, arm: &'ast syn::Arm) {
        self.push_scope();
        visit::visit_arm(self, arm);
        self.pop_scope();
    }

    fn visit_expr_if(&mut self, expr_if: &'ast syn::ExprIf) {
        if let Expr::Let(expr_let) = expr_if.cond.as_ref() {
            self.visit_expr(&expr_let.expr);
            self.push_scope();
            self.bind_local_pattern_names(&expr_let.pat);
            self.visit_pat(&expr_let.pat);
            self.visit_block(&expr_if.then_branch);
            self.pop_scope();
        } else {
            self.visit_expr(&expr_if.cond);
            self.visit_block(&expr_if.then_branch);
        }

        if let Some((_, else_branch)) = &expr_if.else_branch {
            self.visit_expr(else_branch);
        }
    }

    fn visit_expr_while(&mut self, expr_while: &'ast syn::ExprWhile) {
        if let Expr::Let(expr_let) = expr_while.cond.as_ref() {
            self.visit_expr(&expr_let.expr);
            self.push_scope();
            self.bind_local_pattern_names(&expr_let.pat);
            self.visit_pat(&expr_let.pat);
            self.visit_block(&expr_while.body);
            self.pop_scope();
        } else {
            self.visit_expr(&expr_while.cond);
            self.visit_block(&expr_while.body);
        }
    }

    fn visit_expr_for_loop(&mut self, expr_for_loop: &'ast syn::ExprForLoop) {
        self.visit_expr(&expr_for_loop.expr);
        self.push_scope();
        self.bind_local_pattern_names(&expr_for_loop.pat);
        self.visit_pat(&expr_for_loop.pat);
        self.visit_block(&expr_for_loop.body);
        self.pop_scope();
    }

    fn visit_expr_closure(&mut self, closure: &'ast syn::ExprClosure) {
        self.push_scope();
        for input in &closure.inputs {
            self.bind_local_pattern_names(input);
        }
        visit::visit_expr_closure(self, closure);
        self.pop_scope();
    }

    fn visit_pat_ident(&mut self, pat: &'ast syn::PatIdent) {
        self.insert_local_binding(pat.ident.to_string());
        visit::visit_pat_ident(self, pat);
    }

    fn visit_expr_path(&mut self, expr: &'ast ExprPath) {
        if self.path_uses_visible_name(&expr.path)
            && (expr.path.segments.len() > 1 || !self.local_binding_visible())
        {
            self.found = true;
        }
        visit::visit_expr_path(self, expr);
    }

    fn visit_type_path(&mut self, ty: &'ast TypePath) {
        if self.path_uses_visible_name(&ty.path) {
            self.found = true;
        }
        visit::visit_type_path(self, ty);
    }

    fn visit_expr_struct(&mut self, expr: &'ast ExprStruct) {
        if self.path_uses_visible_name(&expr.path) {
            self.found = true;
        }
        visit::visit_expr_struct(self, expr);
    }

    fn visit_expr_macro(&mut self, expr: &'ast ExprMacro) {
        if self.macro_tokens_use_visible_name(&expr.mac.tokens) {
            self.found = true;
        }
        visit::visit_expr_macro(self, expr);
    }

    fn visit_item_macro(&mut self, item: &'ast ItemMacro) {
        if self.macro_tokens_use_visible_name(&item.mac.tokens) {
            self.found = true;
        }
        visit::visit_item_macro(self, item);
    }

    fn visit_macro(&mut self, mac: &'ast Macro) {
        if self.macro_tokens_use_visible_name(&mac.tokens) {
            self.found = true;
        }
        visit::visit_macro(self, mac);
    }
}

fn file_has_super_glob_import(file: &syn::File) -> bool {
    file.items.iter().any(|item| {
        let Item::Use(item_use) = item else {
            return false;
        };
        use_tree_has_super_glob_import(&item_use.tree)
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

fn module_has_reachable_code(
    project: &Project,
    reachable: &BTreeSet<CallableId>,
    reachable_items: &BTreeSet<ItemId>,
    source: &crate::model::SourceFile,
) -> bool {
    source.module_path.is_empty()
        || reachable.iter().any(|callable| match callable {
            CallableId::Free {
                package,
                module_path,
                ..
            } => package == &source.package && path_has_prefix(module_path, &source.module_path),
            CallableId::Method {
                package, type_path, ..
            } => {
                package == &source.package
                    && project
                        .methods
                        .get(callable)
                        .map(|record| path_has_prefix(&record.module_path, &source.module_path))
                        .unwrap_or_else(|| path_has_prefix(type_path, &source.module_path))
            }
        })
        || reachable_items.iter().any(|item| {
            item.package == source.package
                && path_has_prefix(&item.module_path, &source.module_path)
        })
}

#[derive(Debug)]
struct UseDependencyPath {
    segments: Vec<String>,
    visible_name: String,
    renamed: bool,
}

fn collect_use_dependency_paths(
    tree: &UseTree,
    mut prefix: Vec<String>,
    named_paths: &mut Vec<UseDependencyPath>,
    glob_paths: &mut Vec<Vec<String>>,
) {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            collect_use_dependency_paths(&path.tree, prefix, named_paths, glob_paths);
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
            named_paths.push(UseDependencyPath {
                segments: prefix,
                visible_name,
                renamed: false,
            });
        }
        UseTree::Rename(rename) => {
            prefix.push(rename.ident.to_string());
            named_paths.push(UseDependencyPath {
                segments: prefix,
                visible_name: rename.rename.to_string(),
                renamed: true,
            });
        }
        UseTree::Group(group) => {
            for nested in &group.items {
                collect_use_dependency_paths(nested, prefix.clone(), named_paths, glob_paths);
            }
        }
        UseTree::Glob(_) => {
            glob_paths.push(prefix);
        }
    }
}

fn visible_glob_use_paths(items: &[syn::Item]) -> Vec<Vec<String>> {
    let mut paths = Vec::new();
    for item in items {
        let syn::Item::Use(item_use) = item else {
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

fn resolve_use_named_dependency(resolver: &Resolver<'_>, path: &[String]) -> DependencySet {
    let mut dependencies = DependencySet::default();
    if let Some(item) = resolver.resolve_item_segments(path) {
        dependencies.items.insert(item);
        return dependencies;
    }
    if let Some(callable) = resolver.resolve_free_function_segments(path) {
        dependencies.callables.insert(callable);
        return dependencies;
    }

    let Some((package, resolved_path)) = resolver.resolve_value_prefix(path) else {
        return dependencies;
    };
    if resolved_path.is_empty() {
        return dependencies;
    }
    let Some(name) = resolved_path.last() else {
        return dependencies;
    };
    let parent = &resolved_path[..resolved_path.len() - 1];
    dependencies.items.extend(
        resolver
            .project
            .items
            .keys()
            .filter(|item| {
                item.package == package
                    && item.name == *name
                    && path_has_prefix(&item.module_path, parent)
            })
            .cloned(),
    );
    dependencies.callables.extend(
        resolver
            .project
            .functions
            .keys()
            .filter(|callable| match callable {
                CallableId::Free {
                    package: callable_package,
                    module_path,
                    name: callable_name,
                } => {
                    callable_package == &package
                        && callable_name == name
                        && path_has_prefix(module_path, parent)
                }
                CallableId::Method { .. } => false,
            })
            .cloned(),
    );
    dependencies
}

fn resolve_use_glob_dependencies(
    resolver: &Resolver<'_>,
    path: &[String],
    reachable_idents: &BTreeSet<String>,
) -> DependencySet {
    let mut dependencies = DependencySet::default();
    let Some((package, module_path)) = resolver.resolve_prefix(path) else {
        return dependencies;
    };
    dependencies.items.extend(
        resolver
            .project
            .items
            .keys()
            .filter(|item| {
                item.package == package
                    && item.module_path == module_path
                    && reachable_idents.contains(&item.name)
            })
            .cloned(),
    );
    dependencies.callables.extend(
        resolver
            .project
            .functions
            .keys()
            .filter(|callable| match callable {
                CallableId::Free {
                    package: callable_package,
                    module_path: callable_module,
                    name,
                } => {
                    callable_package == &package
                        && callable_module == &module_path
                        && reachable_idents.contains(name)
                }
                CallableId::Method { .. } => false,
            })
            .cloned(),
    );
    dependencies
}

fn path_has_prefix(path: &[String], prefix: &[String]) -> bool {
    path.len() >= prefix.len() && path.iter().zip(prefix).all(|(left, right)| left == right)
}

fn path_ends_with(path: &[String], suffix: &[String]) -> bool {
    path.len() >= suffix.len()
        && path[path.len() - suffix.len()..]
            .iter()
            .zip(suffix)
            .all(|(left, right)| left == right)
}

fn item_macro_feeds_reachable_code(
    project: &Project,
    reachable: &BTreeSet<CallableId>,
    reachable_items: &BTreeSet<ItemId>,
    package: &str,
    module_path: &[String],
    item_macro: &ItemMacro,
) -> bool {
    let token_idents = macro_token_idents(&item_macro.mac.tokens);
    if token_idents.iter().any(|ident| {
        reachable_macro_generated_ident(
            project,
            reachable,
            reachable_items,
            package,
            module_path,
            ident,
        )
    }) {
        return true;
    }

    macro_definition_tokens_feed_reachable_code(
        project,
        reachable,
        reachable_items,
        package,
        module_path,
        item_macro,
    )
}

fn macro_definition_tokens_feed_reachable_code(
    project: &Project,
    reachable: &BTreeSet<CallableId>,
    reachable_items: &BTreeSet<ItemId>,
    package: &str,
    module_path: &[String],
    item_macro: &ItemMacro,
) -> bool {
    let aliases = project
        .module_aliases
        .get(&(package.to_string(), module_path.to_vec()))
        .cloned()
        .unwrap_or_default();
    let resolver = Resolver {
        project,
        package,
        module_path,
        aliases: &aliases,
        self_type: None,
    };
    let Some(item) = resolver.resolve_macro_path(&item_macro.mac.path) else {
        return false;
    };
    let Some(record) = project.items.get(&item) else {
        return false;
    };
    let Item::Macro(definition) = &record.item else {
        return false;
    };
    macro_definition_generated_item_idents(&definition.mac.tokens, &item_macro.mac.tokens)
        .iter()
        .any(|ident| {
            reachable_macro_generated_ident(
                project,
                reachable,
                reachable_items,
                package,
                module_path,
                ident,
            )
        })
}

fn reachable_macro_generated_ident(
    project: &Project,
    reachable: &BTreeSet<CallableId>,
    reachable_items: &BTreeSet<ItemId>,
    package: &str,
    module_path: &[String],
    ident: &str,
) -> bool {
    if module_path.is_empty() {
        return reachable_non_macro_package_mentions_ident(
            project,
            reachable,
            reachable_items,
            package,
            ident,
        );
    }

    reachable.iter().any(|callable| {
        if callable.package() != package {
            return false;
        }
        project.functions.get(callable).is_some_and(|record| {
            record.module_path == module_path
                && token_stream_mentions_ident(&record.item.to_token_stream(), ident)
                || token_stream_mentions_module_path_ident(
                    &record.item.to_token_stream(),
                    module_path,
                    ident,
                )
        }) || project.methods.get(callable).is_some_and(|record| {
            record.module_path == module_path
                && token_stream_mentions_ident(&record.item.to_token_stream(), ident)
                || token_stream_mentions_module_path_ident(
                    &record.item.to_token_stream(),
                    module_path,
                    ident,
                )
        })
    }) || reachable_items.iter().any(|item| {
        item.package == package
            && project.items.get(item).is_some_and(|record| {
                !matches!(record.item, Item::Macro(_))
                    && ((record.module_path == module_path
                        && token_stream_mentions_ident(&record.item.to_token_stream(), ident))
                        || token_stream_mentions_module_path_ident(
                            &record.item.to_token_stream(),
                            module_path,
                            ident,
                        ))
            })
    })
}

fn reachable_non_macro_package_mentions_ident(
    project: &Project,
    reachable: &BTreeSet<CallableId>,
    reachable_items: &BTreeSet<ItemId>,
    package: &str,
    ident: &str,
) -> bool {
    reachable.iter().any(|callable| {
        if callable.package() != package {
            return false;
        }
        callable_id_mentions_ident(callable, ident)
            || project.functions.get(callable).is_some_and(|record| {
                token_stream_mentions_ident(&record.item.to_token_stream(), ident)
            })
            || project.methods.get(callable).is_some_and(|record| {
                token_stream_mentions_ident(&record.item.to_token_stream(), ident)
            })
    }) || reachable_items.iter().any(|item| {
        item.package == package
            && item.kind != ItemKind::Macro
            && (item.name == ident
                || item.module_path.iter().any(|segment| segment == ident)
                || project.items.get(item).is_some_and(|record| {
                    token_stream_mentions_ident(&record.item.to_token_stream(), ident)
                }))
    })
}

fn callable_id_mentions_ident(callable: &CallableId, ident: &str) -> bool {
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

fn token_stream_mentions_module_path_ident(
    tokens: &TokenStream,
    module_path: &[String],
    ident: &str,
) -> bool {
    let mut expected = module_path.to_vec();
    expected.push(ident.to_string());
    token_path_candidates(tokens)
        .into_iter()
        .any(|candidate| path_ends_with(&candidate, &expected))
}

fn should_scan_item_macro_dependencies(
    project: &Project,
    reachable: &BTreeSet<CallableId>,
    reachable_items: &BTreeSet<ItemId>,
    package: &str,
    module_path: &[String],
    item_macro: &ItemMacro,
) -> bool {
    if macro_path_starts_with(&item_macro.mac.path, "uniffi")
        && !reachable_package_mentions_ident(project, reachable, reachable_items, package, "uniffi")
    {
        return false;
    }

    item_macro_feeds_reachable_code(
        project,
        reachable,
        reachable_items,
        package,
        module_path,
        item_macro,
    )
}

fn add_source_mentioned_dependency_packages(
    project: &Project,
    packages: &mut BTreeSet<String>,
    reachable: &BTreeSet<CallableId>,
    reachable_items: &BTreeSet<ItemId>,
) {
    let callable_idents_by_package = reachable_callable_idents_by_package(project, reachable);
    let mut idents_cache = HashMap::<String, BTreeSet<String>>::new();
    let mut path_prefix_cache = HashMap::<String, BTreeSet<String>>::new();
    let mut changed = true;
    while changed {
        changed = false;
        for package_name in packages.clone() {
            let Some(package) = project.workspace.packages.get(&package_name) else {
                continue;
            };
            if package_is_proc_macro(package) {
                continue;
            }
            let idents = idents_cache.entry(package_name.clone()).or_insert_with(|| {
                reachable_package_idents_with_callable_index(
                    project,
                    reachable,
                    reachable_items,
                    &package_name,
                    &callable_idents_by_package,
                )
            });
            let path_prefixes = path_prefix_cache
                .entry(package_name.clone())
                .or_insert_with(|| {
                    reachable_package_dependency_path_prefixes(
                        project,
                        reachable,
                        reachable_items,
                        &package_name,
                        &callable_idents_by_package,
                    )
                });
            if idents.is_empty() && path_prefixes.is_empty() {
                continue;
            }

            for dependency in &package.dependencies {
                if dependency.package != "opensourced"
                    && project.workspace.packages.contains_key(&dependency.package)
                    && source_mentions_dependency(
                        project,
                        &package_name,
                        &idents,
                        &path_prefixes,
                        dependency,
                    )
                {
                    changed |= packages.insert(dependency.package.clone());
                }
            }
        }
    }
}

fn add_local_build_dependency_packages(
    project: &Project,
    packages: &mut BTreeSet<String>,
) -> BTreeSet<String> {
    let mut retained_build_dependencies = BTreeSet::new();
    let mut changed = true;
    while changed {
        changed = false;
        for package_name in packages.clone() {
            let Some(package) = project.workspace.packages.get(&package_name) else {
                continue;
            };
            if manifest_build_script_path(&package.root, &package.manifest).is_none() {
                continue;
            }

            for dependency_package in local_build_dependency_packages(project, &package.manifest) {
                for package in package_closure(project, &dependency_package) {
                    retained_build_dependencies.insert(package.clone());
                    changed |= packages.insert(package);
                }
            }
        }
    }
    retained_build_dependencies
}

fn add_local_library_support_dependency_packages(
    project: &Project,
    packages: &mut BTreeSet<String>,
) -> BTreeSet<String> {
    let mut retained_library_dependencies = BTreeSet::new();
    let mut changed = true;
    while changed {
        changed = false;
        for package_name in packages.clone() {
            let Some(package) = project.workspace.packages.get(&package_name) else {
                continue;
            };
            if !package_should_copy_library_support_source(package) {
                continue;
            }

            for dependency_package in local_library_dependency_packages(project, &package.manifest)
            {
                for package in package_closure(project, &dependency_package) {
                    retained_library_dependencies.insert(package.clone());
                    changed |= packages.insert(package);
                }
            }
        }
    }
    retained_library_dependencies
}

fn local_proc_macro_dependency_packages(
    project: &Project,
    packages: &BTreeSet<String>,
) -> BTreeSet<String> {
    let mut retained_proc_macro_dependencies = BTreeSet::new();
    for package_name in packages.clone() {
        let Some(package) = project.workspace.packages.get(&package_name) else {
            continue;
        };
        if package_is_proc_macro(package) {
            retained_proc_macro_dependencies.insert(package_name);
        }
    }
    retained_proc_macro_dependencies
}

fn retain_proc_macro_exports(
    project: &Project,
    proc_macro_packages: &BTreeSet<String>,
    candidate_packages: &BTreeSet<String>,
    reachable: &mut BTreeSet<CallableId>,
    reachable_items: &mut BTreeSet<ItemId>,
    evidence: &mut ReductionEvidence,
) {
    if proc_macro_packages.is_empty() {
        return;
    }

    let mut callable_queue = VecDeque::new();
    let mut item_queue = VecDeque::new();
    for callable in project.functions.keys() {
        let Some(record) = project.functions.get(callable) else {
            continue;
        };
        if !proc_macro_packages.contains(callable.package()) {
            continue;
        }
        if !proc_macro_export_names(&record.item).iter().any(|name| {
            proc_macro_export_is_referenced(
                project,
                candidate_packages,
                reachable,
                reachable_items,
                &record.package,
                name,
            )
        }) {
            continue;
        }
        if !reachable.contains(callable) {
            callable_queue.push_back(callable.clone());
        }
    }

    while !callable_queue.is_empty() || !item_queue.is_empty() {
        while let Some(callable) = callable_queue.pop_front() {
            if !candidate_packages.contains(callable.package())
                || !reachable.insert(callable.clone())
            {
                continue;
            }
            let dependencies = callable_dependencies(project, &callable);
            evidence.add(&dependencies.evidence);
            for dependency in dependencies.callables {
                if candidate_packages.contains(dependency.package())
                    && !reachable.contains(&dependency)
                {
                    callable_queue.push_back(dependency);
                }
            }
            for item in dependencies.items {
                if candidate_packages.contains(item.package()) && !reachable_items.contains(&item) {
                    item_queue.push_back(item);
                }
            }
        }

        while let Some(item) = item_queue.pop_front() {
            if !candidate_packages.contains(item.package()) || !reachable_items.insert(item.clone())
            {
                continue;
            }
            let dependencies = item_dependencies(project, &item);
            evidence.add(&dependencies.evidence);
            for dependency in dependencies.callables {
                if candidate_packages.contains(dependency.package())
                    && !reachable.contains(&dependency)
                {
                    callable_queue.push_back(dependency);
                }
            }
            for item in dependencies.items {
                if candidate_packages.contains(item.package()) && !reachable_items.contains(&item) {
                    item_queue.push_back(item);
                }
            }
        }
    }
}

fn proc_macro_export_names(item: &syn::ItemFn) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for attr in &item.attrs {
        let path = attr.path();
        if path.is_ident("proc_macro") || path.is_ident("proc_macro_attribute") {
            names.insert(item.sig.ident.to_string());
            continue;
        }
        if !path.is_ident("proc_macro_derive") {
            continue;
        }
        let parser = syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated;
        if let Ok(args) = attr.parse_args_with(parser) {
            if let Some(name) = args.iter().find_map(|meta| match meta {
                Meta::Path(path) => path
                    .segments
                    .last()
                    .map(|segment| segment.ident.to_string()),
                _ => None,
            }) {
                names.insert(name);
            }
        }
    }
    names
}

fn proc_macro_export_is_referenced(
    project: &Project,
    candidate_packages: &BTreeSet<String>,
    reachable: &BTreeSet<CallableId>,
    reachable_items: &BTreeSet<ItemId>,
    proc_macro_package: &str,
    export_name: &str,
) -> bool {
    if public_reexport_idents_referenced_by_reachable_packages(
        project,
        reachable,
        reachable_items,
        proc_macro_package,
    )
    .contains(export_name)
    {
        return true;
    }

    if proc_macro_export_is_used_by_reachable_attrs(
        project,
        candidate_packages,
        reachable,
        reachable_items,
        proc_macro_package,
        export_name,
    ) {
        return true;
    }

    if proc_macro_export_is_used_by_reachable_macro_invocation(
        project,
        candidate_packages,
        reachable,
        reachable_items,
        proc_macro_package,
        export_name,
    ) {
        return true;
    }

    for callable in reachable {
        if callable.package() == proc_macro_package {
            continue;
        }
        if let Some(record) = project.functions.get(callable) {
            if proc_macro_export_alias_is_used(
                project,
                &record.package,
                &record.aliases,
                &record.item.to_token_stream(),
                proc_macro_package,
                export_name,
            ) {
                return true;
            }
        }
        if let Some(record) = project.methods.get(callable) {
            if proc_macro_export_alias_is_used(
                project,
                callable.package(),
                &record.aliases,
                &record.item.to_token_stream(),
                proc_macro_package,
                export_name,
            ) {
                return true;
            }
        }
    }

    reachable_items.iter().any(|item| {
        item.package != proc_macro_package
            && project.items.get(item).is_some_and(|record| {
                proc_macro_export_alias_is_used(
                    project,
                    &record.package,
                    &record.aliases,
                    &record.item.to_token_stream(),
                    proc_macro_package,
                    export_name,
                )
            })
    })
}

fn token_paths_reference_dependency_export(
    project: &Project,
    caller_package: &str,
    aliases: &HashMap<String, Vec<String>>,
    tokens: &TokenStream,
    dependency_package: &str,
    export_name: &str,
) -> bool {
    token_path_candidates(tokens).iter().any(|segments| {
        let (Some(first), Some(last)) = (segments.first(), segments.last()) else {
            return false;
        };
        if segments.len() < 2 || last != export_name {
            return false;
        }
        first_segment_targets_dependency(project, caller_package, dependency_package, first)
            || aliases.get(first).is_some_and(|target| {
                target.first().is_some_and(|target_first| {
                    first_segment_targets_dependency(
                        project,
                        caller_package,
                        dependency_package,
                        target_first,
                    )
                })
            })
    })
}

fn proc_macro_export_alias_is_used(
    project: &Project,
    caller_package: &str,
    aliases: &HashMap<String, Vec<String>>,
    tokens: &TokenStream,
    proc_macro_package: &str,
    export_name: &str,
) -> bool {
    if token_paths_reference_dependency_export(
        project,
        caller_package,
        aliases,
        tokens,
        proc_macro_package,
        export_name,
    ) {
        return true;
    }

    let mut idents = BTreeSet::new();
    collect_token_idents(tokens, &mut idents);
    aliases.iter().any(|(alias, target)| {
        idents.contains(alias)
            && target.last().is_some_and(|last| last == export_name)
            && target.first().is_some_and(|first| {
                first_segment_targets_dependency(project, caller_package, proc_macro_package, first)
            })
    })
}

fn proc_macro_export_is_used_by_reachable_attrs(
    project: &Project,
    candidate_packages: &BTreeSet<String>,
    reachable: &BTreeSet<CallableId>,
    reachable_items: &BTreeSet<ItemId>,
    proc_macro_package: &str,
    export_name: &str,
) -> bool {
    project
        .files
        .values()
        .filter(|source| {
            candidate_packages.contains(&source.package) && source.package != proc_macro_package
        })
        .any(|source| {
            let aliases = project
                .module_aliases
                .get(&(source.package.clone(), source.module_path.clone()))
                .cloned()
                .unwrap_or_default();
            proc_macro_export_is_used_by_reachable_attrs_in_items(
                project,
                reachable,
                reachable_items,
                &source.package,
                &source.module_path,
                &aliases,
                &source.syntax.items,
                proc_macro_package,
                export_name,
            )
        })
}

#[allow(clippy::too_many_arguments)]
fn proc_macro_export_is_used_by_reachable_attrs_in_items(
    project: &Project,
    reachable: &BTreeSet<CallableId>,
    reachable_items: &BTreeSet<ItemId>,
    package: &str,
    module_path: &[String],
    aliases: &HashMap<String, Vec<String>>,
    items: &[Item],
    proc_macro_package: &str,
    export_name: &str,
) -> bool {
    items.iter().any(|item| {
        let attrs = item_attrs(item);
        let attrs_reference_export = !attrs.is_empty()
            && item_attrs_feed_reachable_code(
                project,
                reachable,
                reachable_items,
                package,
                module_path,
                item,
            )
            && attrs.iter().any(|attr| {
                proc_macro_export_alias_is_used(
                    project,
                    package,
                    aliases,
                    &attr.to_token_stream(),
                    proc_macro_package,
                    export_name,
                )
            });
        if attrs_reference_export {
            return true;
        }

        let Item::Mod(item_mod) = item else {
            return false;
        };
        let Some((_, nested_items)) = &item_mod.content else {
            return false;
        };
        let mut nested_module_path = module_path.to_vec();
        nested_module_path.push(item_mod.ident.to_string());
        let nested_aliases = project
            .module_aliases
            .get(&(package.to_string(), nested_module_path.clone()))
            .unwrap_or(aliases);
        proc_macro_export_is_used_by_reachable_attrs_in_items(
            project,
            reachable,
            reachable_items,
            package,
            &nested_module_path,
            nested_aliases,
            nested_items,
            proc_macro_package,
            export_name,
        )
    })
}

fn item_attrs(item: &Item) -> &[syn::Attribute] {
    match item {
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
        _ => &[],
    }
}

fn item_attrs_feed_reachable_code(
    project: &Project,
    reachable: &BTreeSet<CallableId>,
    reachable_items: &BTreeSet<ItemId>,
    package: &str,
    module_path: &[String],
    item: &Item,
) -> bool {
    match item {
        Item::Fn(item_fn) => reachable.contains(&CallableId::Free {
            package: package.to_string(),
            module_path: module_path.to_vec(),
            name: item_fn.sig.ident.to_string(),
        }),
        Item::Mod(item_mod) => {
            let mut child_path = module_path.to_vec();
            child_path.push(item_mod.ident.to_string());
            reachable.iter().any(|callable| match callable {
                CallableId::Free {
                    package: callable_package,
                    module_path: callable_module_path,
                    ..
                } => {
                    callable_package == package
                        && path_has_prefix(callable_module_path, &child_path)
                }
                CallableId::Method {
                    package: callable_package,
                    ..
                } => {
                    callable_package == package
                        && project
                            .methods
                            .get(callable)
                            .is_some_and(|record| path_has_prefix(&record.module_path, &child_path))
                }
            }) || reachable_items.iter().any(|item| {
                item.package == package && path_has_prefix(&item.module_path, &child_path)
            })
        }
        Item::Impl(_) => reachable.iter().any(|callable| {
            callable.package() == package
                && project.methods.get(callable).is_some_and(|record| {
                    record.module_path == module_path
                        && item_impl_attrs_match_reachable_method(item, &record.item)
                })
        }),
        _ => item_id(package, module_path, item)
            .is_some_and(|item_id| reachable_items.contains(&item_id)),
    }
}

fn item_impl_attrs_match_reachable_method(item: &Item, method: &syn::ImplItemFn) -> bool {
    let Item::Impl(item_impl) = item else {
        return false;
    };
    item_impl.items.iter().any(|impl_item| {
        matches!(impl_item, ImplItem::Fn(impl_fn) if impl_fn.sig.ident == method.sig.ident)
    })
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

fn proc_macro_export_is_used_by_reachable_macro_invocation(
    project: &Project,
    candidate_packages: &BTreeSet<String>,
    reachable: &BTreeSet<CallableId>,
    reachable_items: &BTreeSet<ItemId>,
    proc_macro_package: &str,
    export_name: &str,
) -> bool {
    let reachable_symbols = reachable_symbol_names_by_package(project, reachable, reachable_items);
    project
        .files
        .values()
        .filter(|source| {
            candidate_packages.contains(&source.package) && source.package != proc_macro_package
        })
        .any(|source| {
            let aliases = project
                .module_aliases
                .get(&(source.package.clone(), source.module_path.clone()))
                .cloned()
                .unwrap_or_default();
            let Some(symbols) = reachable_symbols.get(&source.package) else {
                return false;
            };
            source.syntax.items.iter().any(|item| {
                let Item::Macro(item_macro) = item else {
                    return false;
                };
                proc_macro_item_invocation_references_export(
                    project,
                    &source.package,
                    &aliases,
                    item_macro,
                    proc_macro_package,
                    export_name,
                ) && item_macro_mentions_any_symbol(item_macro, symbols)
            })
        })
}

fn reachable_symbol_names_by_package(
    project: &Project,
    reachable: &BTreeSet<CallableId>,
    reachable_items: &BTreeSet<ItemId>,
) -> HashMap<String, BTreeSet<String>> {
    let mut symbols: HashMap<String, BTreeSet<String>> = HashMap::new();
    for callable in reachable {
        symbols
            .entry(callable.package().to_string())
            .or_default()
            .insert(callable_name(callable).to_string());
    }
    for item in reachable_items {
        if project.items.contains_key(item) {
            symbols
                .entry(item.package.clone())
                .or_default()
                .insert(item.name.clone());
        }
    }
    symbols
}

fn callable_name(callable: &CallableId) -> &str {
    match callable {
        CallableId::Free { name, .. } => name,
        CallableId::Method { method, .. } => method,
    }
}

fn proc_macro_item_invocation_references_export(
    project: &Project,
    caller_package: &str,
    aliases: &HashMap<String, Vec<String>>,
    item_macro: &ItemMacro,
    proc_macro_package: &str,
    export_name: &str,
) -> bool {
    let segments = item_macro
        .mac
        .path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>();
    let (Some(first), Some(last)) = (segments.first(), segments.last()) else {
        return false;
    };
    if last != export_name {
        return false;
    }
    if first_segment_targets_dependency(project, caller_package, proc_macro_package, first) {
        return true;
    }
    aliases.get(first).is_some_and(|target| {
        target
            .last()
            .is_some_and(|target_last| target_last == export_name)
            && target.first().is_some_and(|target_first| {
                first_segment_targets_dependency(
                    project,
                    caller_package,
                    proc_macro_package,
                    target_first,
                )
            })
    })
}

fn item_macro_mentions_any_symbol(item_macro: &ItemMacro, symbols: &BTreeSet<String>) -> bool {
    let mut idents = BTreeSet::new();
    collect_token_idents(&item_macro.mac.tokens, &mut idents);
    idents.iter().any(|ident| symbols.contains(ident))
}

fn manifest_build_script_path(root: &FsPath, manifest: &Value) -> Option<PathBuf> {
    let package_table = manifest.get("package").and_then(Value::as_table);
    match package_table.and_then(|table| table.get("build")) {
        Some(Value::Boolean(false)) => None,
        Some(Value::String(path)) => {
            let path = root.join(path);
            path.exists().then_some(path)
        }
        _ => {
            let path = root.join("build.rs");
            path.exists().then_some(path)
        }
    }
}

fn retain_entire_packages(
    project: &Project,
    packages: &BTreeSet<String>,
    reachable: &mut BTreeSet<CallableId>,
    reachable_items: &mut BTreeSet<ItemId>,
) {
    if packages.is_empty() {
        return;
    }

    reachable.extend(
        project
            .functions
            .keys()
            .filter(|callable| packages.contains(callable.package()))
            .cloned(),
    );
    reachable.extend(
        project
            .methods
            .keys()
            .filter(|callable| packages.contains(callable.package()))
            .cloned(),
    );
    reachable_items.extend(
        project
            .items
            .keys()
            .filter(|item| packages.contains(item.package()))
            .cloned(),
    );
}

fn local_build_dependency_packages(project: &Project, manifest: &Value) -> BTreeSet<String> {
    let mut packages = BTreeSet::new();
    if let Some(table) = manifest.get("build-dependencies").and_then(Value::as_table) {
        collect_local_dependency_packages(project, table, &mut packages);
    }

    if let Some(targets) = manifest.get("target").and_then(Value::as_table) {
        for target in targets.values() {
            let Some(target) = target.as_table() else {
                continue;
            };
            if let Some(table) = target.get("build-dependencies").and_then(Value::as_table) {
                collect_local_dependency_packages(project, table, &mut packages);
            }
        }
    }

    packages
}

fn collect_local_dependency_packages(
    project: &Project,
    table: &toml::value::Table,
    packages: &mut BTreeSet<String>,
) {
    for (alias, value) in table {
        let package = dependency_package_name(alias, value);
        if package != "opensourced" && project.workspace.packages.contains_key(&package) {
            packages.insert(package);
        }
    }
}

fn local_library_dependency_packages(project: &Project, manifest: &Value) -> BTreeSet<String> {
    let mut packages = BTreeSet::new();

    if let Some(table) = manifest.get("dependencies").and_then(Value::as_table) {
        collect_local_dependency_packages(project, table, &mut packages);
    }

    if let Some(targets) = manifest.get("target").and_then(Value::as_table) {
        for target in targets.values() {
            let Some(target) = target.as_table() else {
                continue;
            };
            if let Some(table) = target.get("dependencies").and_then(Value::as_table) {
                collect_local_dependency_packages(project, table, &mut packages);
            }
        }
    }

    packages
}

fn package_should_copy_library_support_source(package: &crate::manifest::Package) -> bool {
    package
        .entry_target
        .kind
        .iter()
        .any(|kind| matches!(kind.as_str(), "example" | "test" | "bench"))
        && package_library_source_path(package).is_some()
}

fn package_is_proc_macro(package: &crate::manifest::Package) -> bool {
    package
        .manifest
        .get("lib")
        .and_then(Value::as_table)
        .and_then(|lib| lib.get("proc-macro"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
        || package
            .entry_target
            .kind
            .iter()
            .any(|kind| kind == "proc-macro")
}

fn package_library_source_path(package: &crate::manifest::Package) -> Option<PathBuf> {
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
        return existing_package_path(&package.root, path);
    }
    existing_package_path(&package.root, "src/lib.rs")
}

fn existing_package_path(root: &FsPath, path: &str) -> Option<PathBuf> {
    let path = root.join(path);
    path.exists().then_some(path)
}

fn dependency_package_name(alias: &str, value: &Value) -> String {
    match value {
        Value::Table(table) => table
            .get("package")
            .and_then(Value::as_str)
            .unwrap_or(alias)
            .to_string(),
        _ => alias.to_string(),
    }
}

fn source_mentions_dependency(
    project: &Project,
    package: &str,
    idents: &BTreeSet<String>,
    path_prefixes: &BTreeSet<String>,
    dependency: &crate::manifest::Dependency,
) -> bool {
    let alias_code_name = crate_code_name(&dependency.alias);
    let package_code_name = crate_code_name(&dependency.package);
    let names = [
        dependency.alias.as_str(),
        dependency.package.as_str(),
        alias_code_name.as_str(),
        package_code_name.as_str(),
    ];
    if names.iter().any(|name| path_prefixes.contains(*name)) {
        return true;
    }

    project
        .module_aliases
        .iter()
        .filter(|((alias_package, _), _)| alias_package == package)
        .flat_map(|(_, aliases)| aliases.iter())
        .any(|(alias, target)| {
            (path_prefixes.contains(alias) || idents.contains(alias))
                && target
                    .first()
                    .is_some_and(|first| names.iter().any(|name| first == name))
        })
}

fn reachable_package_dependency_path_prefixes(
    project: &Project,
    reachable: &BTreeSet<CallableId>,
    reachable_items: &BTreeSet<ItemId>,
    package: &str,
    callable_idents_by_package: &BTreeMap<String, BTreeSet<String>>,
) -> BTreeSet<String> {
    let mut prefixes = BTreeSet::new();
    for callable in reachable
        .iter()
        .filter(|callable| callable.package() == package)
    {
        if let Some(record) = project.functions.get(callable) {
            collect_dependency_path_prefixes(&record.item.to_token_stream(), &mut prefixes);
        }
        if let Some(record) = project.methods.get(callable) {
            collect_dependency_path_prefixes(&record.item.to_token_stream(), &mut prefixes);
        }
    }
    for item in reachable_items
        .iter()
        .filter(|item| item.package() == package)
    {
        if let Some(record) = project.items.get(item) {
            collect_reachable_package_item_dependency_path_prefixes(
                project,
                reachable,
                record,
                callable_idents_by_package,
                &mut prefixes,
            );
        }
    }
    prefixes
}

fn collect_reachable_package_item_dependency_path_prefixes(
    project: &Project,
    reachable: &BTreeSet<CallableId>,
    record: &crate::model::ItemRecord,
    callable_idents_by_package: &BTreeMap<String, BTreeSet<String>>,
    prefixes: &mut BTreeSet<String>,
) {
    let Item::Struct(item_struct) = &record.item else {
        collect_dependency_path_prefixes(&record.item.to_token_stream(), prefixes);
        return;
    };

    collect_dependency_path_prefixes(&item_struct.vis.to_token_stream(), prefixes);
    collect_dependency_path_prefixes(&item_struct.generics.to_token_stream(), prefixes);
    for attr in &item_struct.attrs {
        collect_dependency_path_prefixes(&attr.to_token_stream(), prefixes);
    }

    for field in &item_struct.fields {
        if struct_field_source_mentions_should_remain(
            project,
            reachable,
            &record.package,
            field,
            callable_idents_by_package,
        ) {
            collect_dependency_path_prefixes(&field.to_token_stream(), prefixes);
        }
    }
}

fn collect_dependency_path_prefixes(tokens: &TokenStream, prefixes: &mut BTreeSet<String>) {
    for segments in token_path_candidates(tokens) {
        if segments.len() >= 2 {
            prefixes.insert(segments[0].clone());
        }
    }
}

fn reachable_package_idents(
    project: &Project,
    reachable: &BTreeSet<CallableId>,
    reachable_items: &BTreeSet<ItemId>,
    package: &str,
) -> BTreeSet<String> {
    let callable_idents_by_package = reachable_callable_idents_by_package(project, reachable);
    reachable_package_idents_with_callable_index(
        project,
        reachable,
        reachable_items,
        package,
        &callable_idents_by_package,
    )
}

fn reachable_package_idents_with_callable_index(
    project: &Project,
    reachable: &BTreeSet<CallableId>,
    reachable_items: &BTreeSet<ItemId>,
    package: &str,
    callable_idents_by_package: &BTreeMap<String, BTreeSet<String>>,
) -> BTreeSet<String> {
    let mut idents = BTreeSet::new();
    if let Some(callable_idents) = callable_idents_by_package.get(package) {
        idents.extend(callable_idents.iter().cloned());
    }
    for item in reachable_items
        .iter()
        .filter(|item| item.package() == package)
    {
        if let Some(record) = project.items.get(item) {
            collect_reachable_package_item_idents(
                project,
                reachable,
                item,
                record,
                callable_idents_by_package,
                &mut idents,
            );
        }
    }
    idents
}

fn reachable_callable_idents_by_package(
    project: &Project,
    reachable: &BTreeSet<CallableId>,
) -> BTreeMap<String, BTreeSet<String>> {
    let mut by_package = BTreeMap::<String, BTreeSet<String>>::new();
    for callable in reachable {
        let idents = by_package
            .entry(callable.package().to_string())
            .or_default();
        collect_callable_id_idents(callable, idents);
        if let Some(record) = project.functions.get(callable) {
            collect_token_idents(&record.item.to_token_stream(), idents);
        }
        if let Some(record) = project.methods.get(callable) {
            collect_token_idents(&record.item.to_token_stream(), idents);
        }
    }
    by_package
}

fn collect_callable_id_idents(callable: &CallableId, idents: &mut BTreeSet<String>) {
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

fn collect_reachable_package_item_idents(
    project: &Project,
    reachable: &BTreeSet<CallableId>,
    item: &ItemId,
    record: &crate::model::ItemRecord,
    callable_idents_by_package: &BTreeMap<String, BTreeSet<String>>,
    idents: &mut BTreeSet<String>,
) {
    let Item::Struct(item_struct) = &record.item else {
        collect_token_idents(&record.item.to_token_stream(), idents);
        return;
    };

    idents.insert(item.name.clone());
    idents.extend(item.module_path.iter().cloned());
    collect_token_idents(&item_struct.vis.to_token_stream(), idents);
    collect_token_idents(&item_struct.generics.to_token_stream(), idents);
    for attr in &item_struct.attrs {
        collect_token_idents(&attr.to_token_stream(), idents);
    }

    for field in &item_struct.fields {
        if struct_field_source_mentions_should_remain(
            project,
            reachable,
            &record.package,
            field,
            callable_idents_by_package,
        ) {
            collect_token_idents(&field.to_token_stream(), idents);
        }
    }
}

fn struct_field_source_mentions_should_remain(
    project: &Project,
    reachable: &BTreeSet<CallableId>,
    package: &str,
    field: &Field,
    callable_idents_by_package: &BTreeMap<String, BTreeSet<String>>,
) -> bool {
    struct_field_reachable_dependency_should_remain_with_callable_index(
        project,
        reachable,
        package,
        field,
        callable_idents_by_package,
    )
}

fn struct_field_static_surface_dependency_should_remain(field: &Field) -> bool {
    matches!(field.vis, syn::Visibility::Public(_))
        || field_attrs_require_field(field)
        || field.ident.is_none()
}

fn struct_field_reachable_dependency_should_remain_with_callable_index(
    _project: &Project,
    _reachable: &BTreeSet<CallableId>,
    package: &str,
    field: &Field,
    callable_idents_by_package: &BTreeMap<String, BTreeSet<String>>,
) -> bool {
    if matches!(field.vis, syn::Visibility::Public(_)) || field_attrs_require_field(field) {
        return true;
    }
    let Some(name) = field.ident.as_ref() else {
        return true;
    };
    callable_idents_by_package
        .get(package)
        .is_some_and(|idents| idents.contains(&name.to_string()))
}

fn field_attrs_require_field(field: &Field) -> bool {
    field.attrs.iter().any(|attr| {
        let path = attr.path();
        if path.is_ident("cfg") {
            return !is_cfg_test_attr(attr);
        }
        !(path.is_ident("allow")
            || path.is_ident("deny")
            || path.is_ident("doc")
            || path.is_ident("deprecated"))
    })
}

fn public_reexport_idents_referenced_by_reachable_packages(
    project: &Project,
    reachable: &BTreeSet<CallableId>,
    reachable_items: &BTreeSet<ItemId>,
    dependency_package: &str,
) -> BTreeSet<String> {
    let mut idents = BTreeSet::new();
    for callable in reachable
        .iter()
        .filter(|callable| callable.package() != dependency_package)
    {
        if let Some(record) = project.functions.get(callable) {
            collect_dependency_public_path_idents(
                project,
                &record.package,
                dependency_package,
                &record.item.to_token_stream(),
                &mut idents,
            );
        }
        if let Some(record) = project.methods.get(callable) {
            collect_dependency_public_path_idents(
                project,
                callable.package(),
                dependency_package,
                &record.item.to_token_stream(),
                &mut idents,
            );
        }
    }
    for item in reachable_items
        .iter()
        .filter(|item| item.package() != dependency_package)
    {
        if let Some(record) = project.items.get(item) {
            collect_dependency_public_path_idents(
                project,
                &record.package,
                dependency_package,
                &record.item.to_token_stream(),
                &mut idents,
            );
        }
    }
    idents
}

fn package_public_api_is_referenced_by_reachable_packages(
    project: &Project,
    reachable: &BTreeSet<CallableId>,
    reachable_items: &BTreeSet<ItemId>,
    dependency_package: &str,
) -> bool {
    !public_reexport_idents_referenced_by_reachable_packages(
        project,
        reachable,
        reachable_items,
        dependency_package,
    )
    .is_empty()
}

fn collect_dependency_public_path_idents(
    project: &Project,
    caller_package: &str,
    dependency_package: &str,
    tokens: &TokenStream,
    idents: &mut BTreeSet<String>,
) {
    for segments in token_path_candidates(tokens) {
        if segments.len() < 2 {
            continue;
        }
        if first_segment_targets_dependency(
            project,
            caller_package,
            dependency_package,
            &segments[0],
        ) {
            idents.insert(segments[1].clone());
        }
    }
}

fn first_segment_targets_dependency(
    project: &Project,
    caller_package: &str,
    dependency_package: &str,
    first: &str,
) -> bool {
    if first == dependency_package || first == crate_code_name(dependency_package) {
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

fn reachable_package_mentions_ident(
    project: &Project,
    reachable: &BTreeSet<CallableId>,
    reachable_items: &BTreeSet<ItemId>,
    package: &str,
    ident: &str,
) -> bool {
    reachable_package_idents(project, reachable, reachable_items, package).contains(ident)
}

fn callable_dependencies(project: &Project, callable: &CallableId) -> DependencySet {
    match callable {
        CallableId::Free { .. } => {
            let Some(record) = project.functions.get(callable) else {
                return DependencySet::default();
            };
            let resolver = Resolver {
                project,
                package: &record.package,
                module_path: &record.module_path,
                aliases: &record.aliases,
                self_type: None,
            };
            let mut visitor = DependencyVisitor::new(resolver);
            visitor.add_generic_trait_bounds(&record.item.sig.generics);
            for attr in &record.item.attrs {
                visitor.visit_attribute(attr);
            }
            visitor.visit_signature(&record.item.sig);
            visitor.add_expected_parse_types_from_return(&record.item.sig.output);
            visitor.add_expected_error_types_from_return(&record.item.sig.output);
            visitor.add_expected_collect_types_from_return(&record.item.sig.output);
            visitor.add_impl_trait_return_dependencies(&record.item.sig.output, &record.item.block);
            visitor.add_fn_inputs(&record.item.sig.inputs);
            visitor.visit_block(&record.item.block);
            let mut dependencies = visitor.dependencies;
            annotate_capped_method_fallback_evidence(
                &mut dependencies.evidence,
                &record.package,
                &record.module_path,
                callable.to_string(),
                &record.span,
            );
            dependencies
        }
        CallableId::Method {
            package,
            type_path,
            trait_path,
            ..
        } => {
            let Some(record) = project.methods.get(callable) else {
                return DependencySet::default();
            };
            let resolver = Resolver {
                project,
                package,
                module_path: &record.module_path,
                aliases: &record.aliases,
                self_type: Some(TypeRef {
                    package: package.clone(),
                    type_path: type_path.clone(),
                }),
            };
            let mut visitor = DependencyVisitor::new(resolver);
            visitor.add_generic_trait_bounds(&record.impl_generics);
            visitor.add_generic_trait_bounds(&record.item.sig.generics);
            if let Some(item) = visitor.resolver.resolve_local_type_item(type_path) {
                visitor.dependencies.items.insert(item);
            }
            if let Some(trait_path) = trait_path {
                if let Some(item) = visitor.resolver.resolve_trait_item(trait_path) {
                    visitor.dependencies.items.insert(item);
                }
            }
            for attr in &record.item.attrs {
                visitor.visit_attribute(attr);
            }
            visitor.visit_impl_peers(callable, record, trait_path.is_some());
            visitor.visit_signature(&record.item.sig);
            visitor.add_expected_parse_types_from_return(&record.item.sig.output);
            visitor.add_expected_error_types_from_return(&record.item.sig.output);
            visitor.add_expected_collect_types_from_return(&record.item.sig.output);
            visitor.add_impl_trait_return_dependencies(&record.item.sig.output, &record.item.block);
            visitor.add_fn_inputs(&record.item.sig.inputs);
            visitor.visit_block(&record.item.block);
            let mut dependencies = visitor.dependencies;
            annotate_capped_method_fallback_evidence(
                &mut dependencies.evidence,
                package,
                &record.module_path,
                callable.to_string(),
                &record.span,
            );
            dependencies
        }
    }
}

fn item_dependencies(project: &Project, item: &ItemId) -> DependencySet {
    let Some(record) = project.items.get(item) else {
        return DependencySet::default();
    };

    if item.kind == ItemKind::Mod {
        return module_root_dependencies(project, item);
    }

    let resolver = Resolver {
        project,
        package: &record.package,
        module_path: &record.module_path,
        aliases: &record.aliases,
        self_type: None,
    };
    let mut visitor = DependencyVisitor::new(resolver);
    if let Item::Struct(item_struct) = &record.item {
        if item_has_opensourced_attr(&record.item) {
            visitor.visit_item(&record.item);
        } else {
            visitor.visit_struct_static_surface_dependencies(item_struct);
        }
    } else if let Item::Trait(item_trait) = &record.item {
        if item_has_opensourced_attr(&record.item) {
            visitor.visit_item(&record.item);
        } else {
            visitor.visit_trait_type_surface_dependencies(item_trait);
        }
    } else {
        visitor.visit_item(&record.item);
    }
    let mut dependencies = visitor.dependencies;
    if item_has_opensourced_attr(&record.item) {
        dependencies.extend(item_root_macro_impl_dependencies(project, item));
    }
    annotate_capped_method_fallback_evidence(
        &mut dependencies.evidence,
        &record.package,
        &record.module_path,
        item.to_string(),
        &record.span,
    );
    dependencies.items.remove(item);
    dependencies
}

fn reachable_macro_impl_surface_dependencies(
    project: &Project,
    candidate_packages: &BTreeSet<String>,
    roots: &[RootId],
    _reachable: &BTreeSet<CallableId>,
    reachable_items: &BTreeSet<ItemId>,
) -> DependencySet {
    let mut dependencies = DependencySet::default();
    for item in reachable_items.iter().filter(|item| {
        candidate_packages.contains(item.package())
            && matches!(
                item.kind,
                ItemKind::Struct | ItemKind::Enum | ItemKind::Union | ItemKind::Type
            )
    }) {
        if !item_is_root(roots, item)
            && !item_is_root_callable_signature_surface(project, roots, item)
        {
            continue;
        }
        dependencies.extend(item_root_macro_impl_dependencies(project, item));
    }
    dependencies
}

fn item_is_root(roots: &[RootId], item: &ItemId) -> bool {
    roots
        .iter()
        .any(|root| matches!(root, RootId::Item(root_item) if root_item == item))
}

fn item_is_root_callable_signature_surface(
    project: &Project,
    roots: &[RootId],
    item: &ItemId,
) -> bool {
    roots.iter().any(|root| {
        let RootId::Callable(callable) = root else {
            return false;
        };
        callable_signature_references_item(project, callable, item)
    })
}

fn callable_signature_references_item(
    project: &Project,
    callable: &CallableId,
    item: &ItemId,
) -> bool {
    match callable {
        CallableId::Free { .. } => {
            let Some(record) = project.functions.get(callable) else {
                return false;
            };
            let resolver = Resolver {
                project,
                package: &record.package,
                module_path: &record.module_path,
                aliases: &record.aliases,
                self_type: None,
            };
            let mut visitor = DependencyVisitor::new(resolver);
            visitor.visit_generics(&record.item.sig.generics);
            for input in &record.item.sig.inputs {
                if let FnArg::Typed(input) = input {
                    visitor.visit_type(&input.ty);
                }
            }
            visitor.visit_return_type(&record.item.sig.output);
            visitor.dependencies.items.contains(item)
                || (callable.package() == item.package
                    && token_stream_mentions_ident(&record.item.sig.to_token_stream(), &item.name))
        }
        CallableId::Method {
            package, type_path, ..
        } => {
            let Some(record) = project.methods.get(callable) else {
                return false;
            };
            let resolver = Resolver {
                project,
                package,
                module_path: &record.module_path,
                aliases: &record.aliases,
                self_type: Some(TypeRef {
                    package: package.clone(),
                    type_path: type_path.clone(),
                }),
            };
            let mut visitor = DependencyVisitor::new(resolver);
            visitor.visit_signature(&record.item.sig);
            visitor.dependencies.items.contains(item)
                || (callable.package() == item.package
                    && token_stream_mentions_ident(&record.item.sig.to_token_stream(), &item.name))
        }
    }
}

fn reachable_struct_field_dependencies(
    project: &Project,
    candidate_packages: &BTreeSet<String>,
    reachable: &BTreeSet<CallableId>,
    reachable_items: &BTreeSet<ItemId>,
) -> DependencySet {
    let mut dependencies = DependencySet::default();
    let callable_idents_by_package = reachable_callable_idents_by_package(project, reachable);
    for item in reachable_items
        .iter()
        .filter(|item| candidate_packages.contains(item.package()) && item.kind == ItemKind::Struct)
    {
        let Some(record) = project.items.get(item) else {
            continue;
        };
        if item_has_opensourced_attr(&record.item) {
            continue;
        }
        let Item::Struct(item_struct) = &record.item else {
            continue;
        };
        let resolver = Resolver {
            project,
            package: &record.package,
            module_path: &record.module_path,
            aliases: &record.aliases,
            self_type: None,
        };
        let mut visitor = DependencyVisitor::new(resolver);
        visitor.visit_struct_reachable_field_dependencies(
            project,
            reachable,
            &record.package,
            item_struct,
            &callable_idents_by_package,
        );
        dependencies.extend(visitor.dependencies);
    }
    dependencies
}

fn item_root_macro_impl_dependencies(project: &Project, item: &ItemId) -> DependencySet {
    let mut dependencies = DependencySet::default();
    let item_path = path_from_item(item);
    let item_is_marked_root = project
        .items
        .get(item)
        .is_some_and(|record| item_has_opensourced_attr(&record.item));

    for module_path in project_module_paths(project, &item.package) {
        let Some(items) = module_items_for_path(project, &item.package, &module_path) else {
            continue;
        };
        let aliases = project
            .module_aliases
            .get(&(item.package.clone(), module_path.clone()))
            .cloned()
            .unwrap_or_default();
        let resolver = Resolver {
            project,
            package: &item.package,
            module_path: &module_path,
            aliases: &aliases,
            self_type: None,
        };

        for source_item in items {
            let Item::Impl(item_impl) = source_item else {
                continue;
            };
            if item_impl.trait_.is_some() || !impl_surface_has_macro_contract_attrs(item_impl) {
                continue;
            }
            let Some(self_type) = resolver.resolve_type(&item_impl.self_ty) else {
                continue;
            };
            if self_type.package != item.package || self_type.type_path != item_path {
                continue;
            }

            let impl_resolver = Resolver {
                project,
                package: &item.package,
                module_path: &module_path,
                aliases: &aliases,
                self_type: Some(self_type),
            };
            let mut visitor = DependencyVisitor::new(impl_resolver);
            visitor.add_generic_trait_bounds(&item_impl.generics);
            for attr in &item_impl.attrs {
                visitor.visit_attribute(attr);
            }
            let impl_has_surface_attr = item_impl
                .attrs
                .iter()
                .any(attr_requires_impl_surface_retention);
            for impl_item in &item_impl.items {
                let should_scan = if item_is_marked_root {
                    root_macro_impl_item_should_render(item_impl, impl_item)
                } else {
                    impl_has_surface_attr || impl_item_attrs_require_surface_retention(impl_item)
                };
                if should_scan {
                    if impl_has_surface_attr || impl_item_attrs_require_surface_retention(impl_item)
                    {
                        if let ImplItem::Fn(method) = impl_item {
                            let method_name = method.sig.ident.to_string();
                            let Some(self_type) = visitor.resolver.self_type.clone() else {
                                continue;
                            };
                            for callable in
                                visitor.resolver.resolve_methods(&self_type, &method_name)
                            {
                                dependencies.callables.insert(callable);
                            }
                        }
                    }
                    visitor.visit_impl_item(impl_item);
                }
            }
            dependencies.extend(visitor.dependencies);
        }
    }

    dependencies
}

fn impl_surface_has_macro_contract_attrs(item_impl: &syn::ItemImpl) -> bool {
    item_impl
        .attrs
        .iter()
        .any(attr_requires_impl_surface_retention)
        || item_impl.items.iter().any(|item| match item {
            ImplItem::Const(item) => item.attrs.iter().any(attr_requires_impl_surface_retention),
            ImplItem::Fn(item) => item.attrs.iter().any(attr_requires_impl_surface_retention),
            ImplItem::Macro(item) => item.attrs.iter().any(attr_requires_impl_surface_retention),
            ImplItem::Type(item) => item.attrs.iter().any(attr_requires_impl_surface_retention),
            ImplItem::Verbatim(_) => false,
            _ => false,
        })
}

fn root_macro_impl_item_should_render(item_impl: &syn::ItemImpl, impl_item: &ImplItem) -> bool {
    if item_impl
        .attrs
        .iter()
        .any(attr_requires_impl_surface_retention)
    {
        return true;
    }

    match impl_item {
        ImplItem::Const(item) => item.attrs.iter().any(attr_requires_impl_surface_retention),
        ImplItem::Fn(item) => {
            matches!(item.vis, syn::Visibility::Public(_))
                || item.attrs.iter().any(attr_requires_impl_surface_retention)
        }
        ImplItem::Macro(item) => item.attrs.iter().any(attr_requires_impl_surface_retention),
        ImplItem::Type(item) => item.attrs.iter().any(attr_requires_impl_surface_retention),
        ImplItem::Verbatim(_) => false,
        _ => false,
    }
}

fn attr_requires_impl_surface_retention(attribute: &syn::Attribute) -> bool {
    let Some(first) = attribute.path().segments.first() else {
        return false;
    };
    if first.ident == "cfg_attr" {
        let tokens = attribute.to_token_stream();
        return token_stream_mentions_ident(&tokens, "uniffi")
            && token_stream_mentions_ident(&tokens, "export");
    }
    !matches!(
        first.ident.to_string().as_str(),
        "allow"
            | "automatically_derived"
            | "cfg"
            | "cfg_attr"
            | "cold"
            | "deny"
            | "deprecated"
            | "doc"
            | "forbid"
            | "inline"
            | "must_use"
            | "repr"
            | "test"
            | "warn"
    )
}

fn module_root_dependencies(project: &Project, item: &ItemId) -> DependencySet {
    let mut module_path = item.module_path.clone();
    module_path.push(item.name.clone());
    let mut dependencies = DependencySet::default();

    dependencies.callables.extend(
        project
            .functions
            .iter()
            .filter(|(_, record)| {
                record.package == item.package
                    && path_has_prefix(&record.module_path, &module_path)
                    && !attrs_are_test(&record.item.attrs)
            })
            .map(|(id, _)| id.clone()),
    );
    dependencies.callables.extend(
        project
            .methods
            .iter()
            .filter(|(id, record)| {
                id.package() == item.package
                    && path_has_prefix(&record.module_path, &module_path)
                    && !attrs_are_test(&record.item.attrs)
            })
            .map(|(id, _)| id.clone()),
    );
    dependencies.items.extend(
        project
            .items
            .iter()
            .filter(|(id, record)| {
                id.package == item.package
                    && path_has_prefix(&id.module_path, &module_path)
                    && !item_has_test_attr(&record.item)
            })
            .map(|(id, _)| id.clone()),
    );

    dependencies
}

fn item_has_test_attr(item: &Item) -> bool {
    match item {
        Item::Const(item) => &item.attrs,
        Item::Enum(item) => &item.attrs,
        Item::Macro(item) => &item.attrs,
        Item::Mod(item) => &item.attrs,
        Item::Static(item) => &item.attrs,
        Item::Struct(item) => &item.attrs,
        Item::Trait(item) => &item.attrs,
        Item::Type(item) => &item.attrs,
        Item::Union(item) => &item.attrs,
        _ => return false,
    }
    .iter()
    .any(|attribute| is_cfg_test_attr(attribute) || is_test_attr(attribute.path()))
}

#[derive(Default)]
struct DependencySet {
    callables: BTreeSet<CallableId>,
    items: BTreeSet<ItemId>,
    evidence: ReductionEvidence,
}

impl DependencySet {
    fn is_empty(&self) -> bool {
        self.callables.is_empty() && self.items.is_empty()
    }

    fn extend(&mut self, other: Self) {
        self.callables.extend(other.callables);
        self.items.extend(other.items);
        self.evidence.add(&other.evidence);
    }
}

fn semantic_callable_dependencies(
    semantic_hints: &SemanticReductionHints,
    callable: &CallableId,
) -> DependencySet {
    semantic_hints
        .callable_edges
        .get(callable)
        .map(dependency_set_from_semantic_dependencies)
        .unwrap_or_default()
}

fn semantic_item_dependencies(
    semantic_hints: &SemanticReductionHints,
    item: &ItemId,
) -> DependencySet {
    semantic_hints
        .item_edges
        .get(item)
        .map(dependency_set_from_semantic_dependencies)
        .unwrap_or_default()
}

fn dependency_set_from_semantic_dependencies(dependencies: &SemanticDependencies) -> DependencySet {
    DependencySet {
        callables: dependencies.callables.clone(),
        items: dependencies.items.clone(),
        evidence: ReductionEvidence {
            semantic_edges_applied: dependencies.len(),
            ..ReductionEvidence::default()
        },
    }
}

fn annotate_capped_method_fallback_evidence(
    evidence: &mut ReductionEvidence,
    package: &str,
    module_path: &[String],
    owner: String,
    span: &crate::model::SourceSpan,
) {
    for detail in &mut evidence.capped_unresolved_method_details {
        if detail.owner.is_some() {
            continue;
        }
        detail.package = Some(package.to_string());
        detail.module_path = Some(module_path.to_vec());
        detail.owner = Some(owner.clone());
        detail.file = Some(span.file.clone());
        detail.start_line.get_or_insert(span.start_line);
    }
}

struct DependencyVisitor<'a> {
    resolver: Resolver<'a>,
    dependencies: DependencySet,
    generic_trait_bounds: HashMap<String, Vec<ItemId>>,
    generic_associated_type_bindings: HashMap<String, HashMap<String, TypeRef>>,
    generic_conversion_bounds: HashMap<String, Vec<GenericConversionBound>>,
    variable_trait_bounds: HashMap<String, Vec<ItemId>>,
    variable_associated_type_bindings: HashMap<String, HashMap<String, TypeRef>>,
    variable_conversion_bounds: HashMap<String, Vec<GenericConversionBound>>,
    variables: HashMap<String, TypeRef>,
    variable_candidates: HashMap<String, Vec<TypeRef>>,
    variable_type_arguments: HashMap<String, Vec<TypeRef>>,
    variable_map_key_types: HashMap<String, TypeRef>,
    variable_map_value_types: HashMap<String, TypeRef>,
    variable_entry_value_types: HashMap<String, TypeRef>,
    variable_result_ok_types: HashMap<String, TypeRef>,
    variable_result_error_types: HashMap<String, TypeRef>,
    local_value_scopes: Vec<BTreeSet<String>>,
    visible_packages: BTreeSet<String>,
    expected_parse_types: BTreeSet<TypeRef>,
    expected_error_types: BTreeSet<TypeRef>,
    expected_collect_types: BTreeSet<TypeRef>,
}

impl<'a> DependencyVisitor<'a> {
    fn new(resolver: Resolver<'a>) -> Self {
        let visible_packages = package_closure(resolver.project, resolver.package);
        Self {
            resolver,
            dependencies: DependencySet::default(),
            generic_trait_bounds: HashMap::new(),
            generic_associated_type_bindings: HashMap::new(),
            generic_conversion_bounds: HashMap::new(),
            variable_trait_bounds: HashMap::new(),
            variable_associated_type_bindings: HashMap::new(),
            variable_conversion_bounds: HashMap::new(),
            variables: HashMap::new(),
            variable_candidates: HashMap::new(),
            variable_type_arguments: HashMap::new(),
            variable_map_key_types: HashMap::new(),
            variable_map_value_types: HashMap::new(),
            variable_entry_value_types: HashMap::new(),
            variable_result_ok_types: HashMap::new(),
            variable_result_error_types: HashMap::new(),
            local_value_scopes: vec![BTreeSet::new()],
            visible_packages,
            expected_parse_types: BTreeSet::new(),
            expected_error_types: BTreeSet::new(),
            expected_collect_types: BTreeSet::new(),
        }
    }

    fn add_generic_trait_bounds(&mut self, generics: &syn::Generics) {
        for parameter in &generics.params {
            let syn::GenericParam::Type(type_parameter) = parameter else {
                continue;
            };
            let name = type_parameter.ident.to_string();
            let trait_items = self.trait_items_from_bounds(&type_parameter.bounds);
            let associated_type_bindings =
                self.associated_type_bindings_from_bounds(&type_parameter.bounds);
            let conversion_bounds = self.conversion_bounds_from_bounds(&type_parameter.bounds);
            self.insert_generic_trait_bounds(name.clone(), trait_items);
            self.insert_generic_associated_type_bindings(name.clone(), associated_type_bindings);
            self.insert_generic_conversion_bounds(name, conversion_bounds);
        }

        let Some(where_clause) = &generics.where_clause else {
            return;
        };
        for predicate in &where_clause.predicates {
            let syn::WherePredicate::Type(predicate) = predicate else {
                continue;
            };
            let Some(name) = generic_parameter_name_from_type(&predicate.bounded_ty) else {
                continue;
            };
            let trait_items = self.trait_items_from_bounds(&predicate.bounds);
            let associated_type_bindings =
                self.associated_type_bindings_from_bounds(&predicate.bounds);
            let conversion_bounds = self.conversion_bounds_from_bounds(&predicate.bounds);
            self.insert_generic_trait_bounds(name.clone(), trait_items);
            self.insert_generic_associated_type_bindings(name.clone(), associated_type_bindings);
            self.insert_generic_conversion_bounds(name, conversion_bounds);
        }
    }

    fn visit_struct_static_surface_dependencies(&mut self, item_struct: &syn::ItemStruct) {
        for attr in &item_struct.attrs {
            self.visit_attribute(attr);
        }
        self.visit_generics(&item_struct.generics);
        for field in &item_struct.fields {
            if struct_field_static_surface_dependency_should_remain(field) {
                self.visit_struct_field_surface_dependencies(item_struct, field);
            }
        }
    }

    fn visit_struct_reachable_field_dependencies(
        &mut self,
        project: &Project,
        reachable: &BTreeSet<CallableId>,
        package: &str,
        item_struct: &syn::ItemStruct,
        callable_idents_by_package: &BTreeMap<String, BTreeSet<String>>,
    ) {
        for field in &item_struct.fields {
            if struct_field_reachable_dependency_should_remain_with_callable_index(
                project,
                reachable,
                package,
                field,
                callable_idents_by_package,
            ) {
                self.visit_struct_field_surface_dependencies(item_struct, field);
            }
        }
    }

    fn visit_struct_field_surface_dependencies(
        &mut self,
        item_struct: &syn::ItemStruct,
        field: &Field,
    ) {
        self.add_derive_field_trait_dependencies_for_field(&item_struct.attrs, field);
        self.add_generic_field_type_trait_dependencies_for_field(field);
        self.add_serde_default_field_dependencies_for_field(field);
        self.visit_field(field);
    }

    fn visit_trait_type_surface_dependencies(&mut self, item_trait: &syn::ItemTrait) {
        for attr in &item_trait.attrs {
            self.visit_attribute(attr);
        }
        self.visit_generics(&item_trait.generics);
        for bound in &item_trait.supertraits {
            self.visit_type_param_bound(bound);
        }
        for item in &item_trait.items {
            if !trait_item_type_surface_dependency_should_remain(item) {
                continue;
            }
            self.visit_trait_item_type_surface_dependencies(item);
        }
    }

    fn visit_trait_item_type_surface_dependencies(&mut self, item: &syn::TraitItem) {
        match item {
            syn::TraitItem::Fn(method) => {
                for attr in &method.attrs {
                    self.visit_attribute(attr);
                }
                self.visit_signature(&method.sig);
                if method
                    .attrs
                    .iter()
                    .any(attr_requires_impl_surface_retention)
                {
                    if let Some(default) = &method.default {
                        self.visit_block(default);
                    }
                }
            }
            syn::TraitItem::Const(item) => {
                for attr in &item.attrs {
                    self.visit_attribute(attr);
                }
                self.visit_type(&item.ty);
                if item.attrs.iter().any(attr_requires_impl_surface_retention) {
                    if let Some((_, default)) = &item.default {
                        self.visit_expr(default);
                    }
                }
            }
            syn::TraitItem::Type(item) => {
                for attr in &item.attrs {
                    self.visit_attribute(attr);
                }
                self.visit_generics(&item.generics);
                for bound in &item.bounds {
                    self.visit_type_param_bound(bound);
                }
                if item.attrs.iter().any(attr_requires_impl_surface_retention) {
                    if let Some((_, default)) = &item.default {
                        self.visit_type(default);
                    }
                }
            }
            syn::TraitItem::Macro(item) => {
                for attr in &item.attrs {
                    self.visit_attribute(attr);
                }
                self.add_macro_path(&item.mac.path);
                self.add_macro_token_dependencies(&item.mac.tokens);
            }
            syn::TraitItem::Verbatim(tokens) => {
                self.add_macro_token_dependencies(tokens);
            }
            _ => self.visit_trait_item(item),
        }
    }

    fn insert_generic_trait_bounds(&mut self, name: String, mut trait_items: Vec<ItemId>) {
        if trait_items.is_empty() {
            return;
        }
        let bounds = self.generic_trait_bounds.entry(name).or_default();
        bounds.append(&mut trait_items);
        bounds.sort();
        bounds.dedup();
    }

    fn insert_generic_associated_type_bindings(
        &mut self,
        name: String,
        associated_type_bindings: HashMap<String, TypeRef>,
    ) {
        if associated_type_bindings.is_empty() {
            return;
        }
        self.generic_associated_type_bindings
            .entry(name)
            .or_default()
            .extend(associated_type_bindings);
    }

    fn insert_generic_conversion_bounds(
        &mut self,
        name: String,
        mut conversion_bounds: Vec<GenericConversionBound>,
    ) {
        if conversion_bounds.is_empty() {
            return;
        }
        let bounds = self.generic_conversion_bounds.entry(name).or_default();
        bounds.append(&mut conversion_bounds);
    }

    fn associated_type_bindings_from_bounds(
        &self,
        bounds: &syn::punctuated::Punctuated<syn::TypeParamBound, syn::Token![+]>,
    ) -> HashMap<String, TypeRef> {
        let mut bindings = HashMap::new();
        for bound in bounds {
            let syn::TypeParamBound::Trait(trait_bound) = bound else {
                continue;
            };
            for segment in &trait_bound.path.segments {
                let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
                    continue;
                };
                for argument in &arguments.args {
                    let GenericArgument::AssocType(associated_type) = argument else {
                        continue;
                    };
                    if let Some(type_ref) = self.resolver.resolve_receiver_type(&associated_type.ty)
                    {
                        bindings.insert(associated_type.ident.to_string(), type_ref);
                    }
                }
            }
        }
        bindings
    }

    fn conversion_bounds_from_bounds(
        &self,
        bounds: &syn::punctuated::Punctuated<syn::TypeParamBound, syn::Token![+]>,
    ) -> Vec<GenericConversionBound> {
        bounds
            .iter()
            .filter_map(|bound| {
                let syn::TypeParamBound::Trait(trait_bound) = bound else {
                    return None;
                };
                self.resolver
                    .conversion_bound_from_trait_path(&trait_bound.path)
            })
            .collect()
    }

    fn trait_items_from_bounds(
        &self,
        bounds: &syn::punctuated::Punctuated<syn::TypeParamBound, syn::Token![+]>,
    ) -> Vec<ItemId> {
        let mut trait_items = bounds
            .iter()
            .filter_map(|bound| {
                let syn::TypeParamBound::Trait(trait_bound) = bound else {
                    return None;
                };
                self.resolver.resolve_trait_bound_path(&trait_bound.path)
            })
            .collect::<Vec<_>>();
        trait_items.sort();
        trait_items.dedup();
        trait_items
    }

    fn add_fn_inputs(&mut self, inputs: &syn::punctuated::Punctuated<FnArg, syn::token::Comma>) {
        for input in inputs {
            let FnArg::Typed(input) = input else {
                continue;
            };
            self.bind_local_value_names(input.pat.as_ref());
            for (name, type_ref, candidates) in self.pattern_type_bindings(&input.pat, &input.ty) {
                self.insert_variable_candidates(name, type_ref, candidates);
            }
            let Pat::Ident(ident) = input.pat.as_ref() else {
                continue;
            };
            let trait_items = self.trait_items_from_type(&input.ty);
            self.insert_variable_trait_bounds(ident.ident.to_string(), trait_items);
            if let Some(name) = generic_parameter_name_from_type(&input.ty) {
                if let Some(bindings) = self.generic_associated_type_bindings.get(&name) {
                    self.insert_variable_associated_type_bindings(
                        ident.ident.to_string(),
                        bindings.clone(),
                    );
                }
                if let Some(bounds) = self.generic_conversion_bounds.get(&name) {
                    self.insert_variable_conversion_bounds(ident.ident.to_string(), bounds.clone());
                }
            }
            if let Some(type_ref) = self.resolver.resolve_receiver_type(&input.ty) {
                let candidates = self.resolver.receiver_type_candidates_from_type(&input.ty);
                self.insert_variable_candidates(ident.ident.to_string(), type_ref, candidates);
            }
        }
    }

    fn insert_variable_type(&mut self, name: String, type_ref: TypeRef) {
        let candidates = self.resolver.type_ref_candidates(&type_ref);
        self.insert_variable_candidates(name, type_ref, candidates);
    }

    fn bind_local_value_names(&mut self, pattern: &Pat) {
        let scope = self
            .local_value_scopes
            .last_mut()
            .expect("dependency visitor should always have a local value scope");
        collect_pattern_ident_names(pattern, scope);
    }

    fn push_local_value_scope(&mut self) {
        self.local_value_scopes.push(BTreeSet::new());
    }

    fn pop_local_value_scope(&mut self) {
        self.local_value_scopes.pop();
        if self.local_value_scopes.is_empty() {
            self.local_value_scopes.push(BTreeSet::new());
        }
    }

    fn local_value_binding_visible(&self, name: &str) -> bool {
        self.local_value_scopes
            .iter()
            .rev()
            .any(|scope| scope.contains(name))
    }

    fn expr_path_is_local_value(&self, path: &ExprPath) -> bool {
        path.qself.is_none()
            && path.path.segments.len() == 1
            && path
                .path
                .segments
                .first()
                .is_some_and(|segment| self.local_value_binding_visible(&segment.ident.to_string()))
    }

    fn local_value_type_candidates(&self, name: &str) -> Vec<TypeRef> {
        let mut candidates = self
            .variable_candidates
            .get(name)
            .cloned()
            .unwrap_or_default();
        if let Some(type_ref) = self.variables.get(name) {
            candidates.push(type_ref.clone());
        }
        candidates.sort();
        candidates.dedup();
        candidates
    }

    fn insert_variable_candidates(
        &mut self,
        name: String,
        type_ref: TypeRef,
        mut candidates: Vec<TypeRef>,
    ) {
        candidates.push(type_ref.clone());
        for candidate in self.resolver.type_ref_candidates(&type_ref) {
            candidates.push(candidate);
        }
        candidates.sort();
        candidates.dedup();
        self.variables.insert(name.clone(), type_ref);
        self.variable_candidates.insert(name, candidates);
    }

    fn insert_variable_type_data_from_type(&mut self, name: String, ty: &Type) {
        let type_arguments = self.resolver.project_type_argument_refs_in_type(ty);
        if !type_arguments.is_empty() {
            self.variable_type_arguments
                .insert(name.clone(), type_arguments);
        }
        if let Some(key_type) = self.resolver.map_key_type(ty) {
            self.variable_map_key_types.insert(name.clone(), key_type);
        }
        if let Some(value_type) = self.resolver.map_value_type(ty) {
            self.variable_map_value_types
                .insert(name.clone(), value_type);
        }
        if let Some(ok_type) = self.resolver.result_ok_type(ty) {
            self.variable_result_ok_types.insert(name.clone(), ok_type);
        }
        if let Some(error_type) = self.resolver.result_error_type(ty) {
            self.variable_result_error_types
                .insert(name.clone(), error_type);
        }
    }

    fn insert_variable_type_data_from_expr(&mut self, name: String, expression: &Expr) {
        let type_arguments = self.expression_type_arguments(expression);
        if !type_arguments.is_empty() {
            self.variable_type_arguments
                .insert(name.clone(), type_arguments);
        }
        if let Some(key_type) = self.expression_map_key_type(expression) {
            self.variable_map_key_types.insert(name.clone(), key_type);
        }
        if let Some(value_type) = self.expression_map_value_type(expression) {
            self.variable_map_value_types
                .insert(name.clone(), value_type);
        }
        if let Some(value_type) = self.expression_entry_value_type(expression) {
            self.variable_entry_value_types
                .insert(name.clone(), value_type);
        }
        if let Some(ok_type) = self.expression_result_ok_type(expression) {
            self.variable_result_ok_types.insert(name.clone(), ok_type);
        }
        if let Some(error_type) = self.expression_result_error_type(expression) {
            self.variable_result_error_types
                .insert(name.clone(), error_type);
        }
    }

    fn insert_variable_trait_bounds(&mut self, name: String, mut trait_items: Vec<ItemId>) {
        if trait_items.is_empty() {
            return;
        }
        let bounds = self.variable_trait_bounds.entry(name).or_default();
        bounds.append(&mut trait_items);
        bounds.sort();
        bounds.dedup();
    }

    fn insert_variable_associated_type_bindings(
        &mut self,
        name: String,
        associated_type_bindings: HashMap<String, TypeRef>,
    ) {
        if associated_type_bindings.is_empty() {
            return;
        }
        self.variable_associated_type_bindings
            .entry(name)
            .or_default()
            .extend(associated_type_bindings);
    }

    fn insert_variable_conversion_bounds(
        &mut self,
        name: String,
        mut conversion_bounds: Vec<GenericConversionBound>,
    ) {
        if conversion_bounds.is_empty() {
            return;
        }
        let bounds = self.variable_conversion_bounds.entry(name).or_default();
        bounds.append(&mut conversion_bounds);
    }

    fn trait_items_from_type(&self, ty: &Type) -> Vec<ItemId> {
        let mut trait_items = Vec::new();
        self.collect_trait_items_from_type(ty, &mut trait_items);
        trait_items.sort();
        trait_items.dedup();
        trait_items
    }

    fn collect_trait_items_from_type(&self, ty: &Type, trait_items: &mut Vec<ItemId>) {
        match ty {
            Type::Path(type_path) => {
                if let Some(name) = single_segment_type_name(&type_path.path) {
                    if let Some(bounds) = self.generic_trait_bounds.get(&name) {
                        trait_items.extend(bounds.iter().cloned());
                    }
                }
                self.collect_transparent_wrapper_trait_items(&type_path.path, trait_items);
            }
            Type::ImplTrait(impl_trait) => {
                trait_items.extend(self.trait_items_from_bounds(&impl_trait.bounds));
            }
            Type::TraitObject(trait_object) => {
                trait_items.extend(self.trait_items_from_bounds(&trait_object.bounds));
            }
            Type::Reference(reference) => {
                self.collect_trait_items_from_type(&reference.elem, trait_items)
            }
            Type::Group(group) => self.collect_trait_items_from_type(&group.elem, trait_items),
            Type::Paren(paren) => self.collect_trait_items_from_type(&paren.elem, trait_items),
            _ => {}
        }
    }

    fn collect_transparent_wrapper_trait_items(&self, path: &Path, trait_items: &mut Vec<ItemId>) {
        let Some(segment) = path.segments.last() else {
            return;
        };
        if !transparent_receiver_wrapper(&segment.ident.to_string()) {
            return;
        }
        let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
            return;
        };
        for argument in &arguments.args {
            let GenericArgument::Type(ty) = argument else {
                continue;
            };
            self.collect_trait_items_from_type(ty, trait_items);
        }
    }

    fn add_call_path(&mut self, path: &Path) {
        for callable in self.resolver.resolve_call_path(path) {
            self.dependencies.callables.insert(callable);
        }
    }

    fn add_expr_path_call(&mut self, path: &ExprPath) {
        if path.qself.is_some() {
            for callable in self.resolver.resolve_qself_call(path) {
                self.dependencies.callables.insert(callable);
            }
        } else {
            self.add_call_path(&path.path);
        }
    }

    fn add_item_path(&mut self, path: &Path) {
        if let Some(item) = self.resolver.resolve_item_path(path) {
            self.dependencies.items.insert(item);
        }
    }

    fn receiver_type(&self, expression: &Expr) -> Option<TypeRef> {
        match expression {
            Expr::Path(path) if path.path.segments.len() == 1 => {
                let name = path.path.segments.first()?.ident.to_string();
                if name == "self" {
                    return self.resolver.self_type.clone();
                }
                self.variables
                    .get(&name)
                    .cloned()
                    .or_else(|| self.resolver.type_from_value_path(&path.path))
                    .or_else(|| self.resolver.resolve_type_path(&path.path))
            }
            Expr::Path(path) => self
                .resolver
                .type_from_value_path(&path.path)
                .or_else(|| self.resolver.resolve_type_path(&path.path)),
            Expr::Call(call) => self
                .wrapper_constructor_arg_type(call)
                .or_else(|| self.std_mem_return_type(call))
                .or_else(|| {
                    if let Expr::Path(path) = call.func.as_ref() {
                        self.resolver.type_from_expr_path_call(path)
                    } else {
                        None
                    }
                }),
            Expr::MethodCall(call) => self.method_call_return_type(call),
            Expr::Try(expr) => self
                .expression_type_arguments(&expr.expr)
                .into_iter()
                .next()
                .or_else(|| self.receiver_type(&expr.expr)),
            Expr::Index(index) => self.index_output_type(index),
            Expr::Field(field) => {
                let receiver = self.receiver_type(&field.base)?;
                self.resolver.field_type(&receiver, &field.member)
            }
            Expr::Struct(expr) => self.resolver.resolve_type_path(&expr.path),
            Expr::Block(expr) => {
                final_block_expression(&expr.block).and_then(|expr| self.receiver_type(expr))
            }
            Expr::Unsafe(expr) => {
                final_block_expression(&expr.block).and_then(|expr| self.receiver_type(expr))
            }
            Expr::Reference(reference) => self.receiver_type(&reference.expr),
            Expr::Paren(paren) => self.receiver_type(&paren.expr),
            _ => None,
        }
    }

    fn receiver_type_candidates(&self, expression: &Expr) -> Vec<TypeRef> {
        let mut candidates = Vec::new();
        match expression {
            Expr::Path(path) if path.path.segments.len() == 1 => {
                let name = path.path.segments.first().unwrap().ident.to_string();
                if name == "self" {
                    if let Some(self_type) = &self.resolver.self_type {
                        candidates.extend(self.resolver.type_ref_candidates(self_type));
                    }
                } else if let Some(variable_candidates) = self.variable_candidates.get(&name) {
                    candidates.extend(variable_candidates.clone());
                } else {
                    candidates.extend(self.expression_type_arguments(expression));
                }
            }
            Expr::Call(call) => {
                if let Some(type_ref) = self.wrapper_constructor_arg_type(call).or_else(|| {
                    self.std_mem_return_type(call).or_else(|| {
                        if let Expr::Path(path) = call.func.as_ref() {
                            self.resolver.type_from_expr_path_call(path)
                        } else {
                            None
                        }
                    })
                }) {
                    candidates.extend(self.resolver.type_ref_candidates(&type_ref));
                }
                candidates.extend(self.call_return_type_candidates(call));
            }
            Expr::MethodCall(call) => {
                if matches!(call.method.to_string().as_str(), "as_ref" | "clone") {
                    candidates.extend(self.receiver_type_candidates(&call.receiver));
                } else if call.method == "lock" && call.args.is_empty() {
                    candidates.extend(
                        self.receiver_type(&call.receiver)
                            .into_iter()
                            .flat_map(|type_ref| self.resolver.type_ref_candidates(&type_ref)),
                    );
                }

                for receiver in self.receiver_type_candidates(&call.receiver) {
                    for callable in self
                        .resolver
                        .resolve_methods(&receiver, &call.method.to_string())
                    {
                        if let Some(return_type) =
                            self.resolver.return_type_from_callable(&callable)
                        {
                            candidates.extend(self.resolver.type_ref_candidates(&return_type));
                        }
                    }
                }
            }
            Expr::Try(expr) => {
                candidates.extend(self.expression_type_arguments(&expr.expr));
                candidates.extend(self.receiver_type_candidates(&expr.expr));
            }
            Expr::Index(index) => {
                candidates.extend(self.index_output_type_candidates(index));
            }
            Expr::Field(field) => {
                for receiver in self.receiver_type_candidates(&field.base) {
                    candidates.extend(
                        self.resolver
                            .field_type_candidates(&receiver, &field.member),
                    );
                }
            }
            Expr::Struct(expr) => {
                if let Some(type_ref) = self.resolver.resolve_type_path(&expr.path) {
                    candidates.extend(self.resolver.type_ref_candidates(&type_ref));
                }
            }
            Expr::Block(expr) => {
                if let Some(tail) = final_block_expression(&expr.block) {
                    candidates.extend(self.receiver_type_candidates(tail));
                }
            }
            Expr::Unsafe(expr) => {
                if let Some(tail) = final_block_expression(&expr.block) {
                    candidates.extend(self.receiver_type_candidates(tail));
                }
            }
            Expr::Reference(reference) => {
                candidates.extend(self.receiver_type_candidates(&reference.expr))
            }
            Expr::Paren(paren) => candidates.extend(self.receiver_type_candidates(&paren.expr)),
            _ => {}
        }

        if candidates.is_empty() {
            if let Some(type_ref) = self.receiver_type(expression) {
                candidates.extend(self.resolver.type_ref_candidates(&type_ref));
            }
        }
        candidates.sort();
        candidates.dedup();
        candidates
    }

    fn call_return_type_candidates(&self, call: &ExprCall) -> Vec<TypeRef> {
        let Expr::Path(path) = call.func.as_ref() else {
            return Vec::new();
        };
        let callables = if path.qself.is_some() {
            self.resolver.resolve_qself_call(path)
        } else {
            self.resolver.resolve_call_path(&path.path)
        };
        callables
            .iter()
            .filter_map(|callable| self.resolver.return_type_from_callable(callable))
            .flat_map(|type_ref| self.resolver.type_ref_candidates(&type_ref))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    fn index_output_type(&self, expression: &ExprIndex) -> Option<TypeRef> {
        self.index_output_type_candidates(expression)
            .into_iter()
            .next()
    }

    fn index_output_type_candidates(&self, expression: &ExprIndex) -> Vec<TypeRef> {
        let mut outputs = Vec::new();
        for receiver in self.receiver_type_candidates(&expression.expr) {
            outputs.extend(self.resolver.index_output_types(&receiver));
        }
        if let Some(receiver) = self.receiver_type(&expression.expr) {
            outputs.extend(self.resolver.index_output_types(&receiver));
        }
        outputs.sort();
        outputs.dedup();
        outputs
    }

    fn local_binding_type(&self, local: &Local) -> Option<(String, TypeRef, Vec<TypeRef>)> {
        let (name, explicit_type) = binding_name_and_type(&local.pat)?;
        if let Some(ty) = explicit_type {
            if let Some(type_ref) = self.resolver.resolve_receiver_type(ty) {
                let candidates = self.resolver.receiver_type_candidates_from_type(ty);
                return Some((name, type_ref, candidates));
            }
        }

        let init = local.init.as_ref()?;
        let type_ref = self.infer_expr_type(&init.expr)?;
        let candidates = self.receiver_type_candidates(&init.expr);
        Some((name, type_ref, candidates))
    }

    fn local_destructured_binding_types(
        &self,
        local: &Local,
    ) -> Vec<(String, TypeRef, Vec<TypeRef>)> {
        if binding_name_and_type(&local.pat).is_some() {
            return Vec::new();
        }

        let mut bindings = Vec::new();
        if let Pat::Type(pat_type) = &local.pat {
            self.collect_pattern_type_bindings(
                &pat_type.pat,
                &pat_type.ty,
                &self.resolver,
                &mut bindings,
            );
            return bindings;
        }

        let Some(init) = &local.init else {
            return bindings;
        };
        self.collect_pattern_bindings_from_expr(&local.pat, &init.expr, &mut bindings);
        bindings.sort();
        bindings.dedup();
        bindings
    }

    fn collect_pattern_bindings_from_expr(
        &self,
        pattern: &Pat,
        expression: &Expr,
        bindings: &mut Vec<(String, TypeRef, Vec<TypeRef>)>,
    ) {
        match expression {
            Expr::Call(call) => {
                let Expr::Path(path) = call.func.as_ref() else {
                    return;
                };
                for callable in self.resolver.resolve_call_path(&path.path) {
                    self.collect_pattern_bindings_from_callable_return(
                        pattern, &callable, bindings,
                    );
                }
            }
            Expr::MethodCall(call) => {
                for receiver in self.receiver_type_candidates(&call.receiver) {
                    for callable in self
                        .resolver
                        .resolve_methods(&receiver, &call.method.to_string())
                    {
                        self.collect_pattern_bindings_from_callable_return(
                            pattern, &callable, bindings,
                        );
                    }
                }
            }
            Expr::Try(expr) => {
                self.collect_pattern_bindings_from_expr(pattern, &expr.expr, bindings)
            }
            Expr::Block(expr) => {
                if let Some(tail) = final_block_expression(&expr.block) {
                    self.collect_pattern_bindings_from_expr(pattern, tail, bindings);
                }
            }
            Expr::Unsafe(expr) => {
                if let Some(tail) = final_block_expression(&expr.block) {
                    self.collect_pattern_bindings_from_expr(pattern, tail, bindings);
                }
            }
            Expr::Reference(reference) => {
                self.collect_pattern_bindings_from_expr(pattern, &reference.expr, bindings)
            }
            Expr::Paren(paren) => {
                self.collect_pattern_bindings_from_expr(pattern, &paren.expr, bindings)
            }
            _ => {}
        }
    }

    fn collect_pattern_bindings_from_callable_return(
        &self,
        pattern: &Pat,
        callable: &CallableId,
        bindings: &mut Vec<(String, TypeRef, Vec<TypeRef>)>,
    ) {
        match callable {
            CallableId::Free { .. } => {
                let Some(record) = self.resolver.project.functions.get(callable) else {
                    return;
                };
                let ReturnType::Type(_, ty) = &record.item.sig.output else {
                    return;
                };
                let resolver = Resolver {
                    project: self.resolver.project,
                    package: &record.package,
                    module_path: &record.module_path,
                    aliases: &record.aliases,
                    self_type: None,
                };
                self.collect_pattern_type_bindings(pattern, ty, &resolver, bindings);
            }
            CallableId::Method {
                package, type_path, ..
            } => {
                let Some(record) = self.resolver.project.methods.get(callable) else {
                    return;
                };
                let ReturnType::Type(_, ty) = &record.item.sig.output else {
                    return;
                };
                let resolver = Resolver {
                    project: self.resolver.project,
                    package,
                    module_path: &record.module_path,
                    aliases: &record.aliases,
                    self_type: Some(TypeRef {
                        package: package.clone(),
                        type_path: type_path.clone(),
                    }),
                };
                self.collect_pattern_type_bindings(pattern, ty, &resolver, bindings);
            }
        }
    }

    fn pattern_type_bindings(
        &self,
        pattern: &Pat,
        ty: &Type,
    ) -> Vec<(String, TypeRef, Vec<TypeRef>)> {
        let mut bindings = Vec::new();
        self.collect_pattern_type_bindings(pattern, ty, &self.resolver, &mut bindings);
        bindings.sort();
        bindings.dedup();
        bindings
    }

    fn collect_pattern_type_bindings(
        &self,
        pattern: &Pat,
        ty: &Type,
        resolver: &Resolver<'_>,
        bindings: &mut Vec<(String, TypeRef, Vec<TypeRef>)>,
    ) {
        match (pattern, ty) {
            (Pat::Ident(ident), _) => {
                if let Some(type_ref) = resolver.resolve_receiver_type(ty) {
                    bindings.push((
                        ident.ident.to_string(),
                        type_ref,
                        resolver.receiver_type_candidates_from_type(ty),
                    ));
                }
            }
            (Pat::Type(pat_type), _) => {
                self.collect_pattern_type_bindings(&pat_type.pat, &pat_type.ty, resolver, bindings);
            }
            (Pat::Reference(pattern), Type::Reference(ty)) => {
                self.collect_pattern_type_bindings(&pattern.pat, &ty.elem, resolver, bindings);
            }
            (Pat::Tuple(pattern), Type::Tuple(ty)) => {
                for (pattern, ty) in pattern.elems.iter().zip(&ty.elems) {
                    self.collect_pattern_type_bindings(pattern, ty, resolver, bindings);
                }
            }
            (Pat::TupleStruct(pattern), Type::Path(type_path)) => {
                let mut matched_generic_variant = false;
                for (pattern, ty) in generic_tuple_variant_field_types(pattern, &type_path.path) {
                    matched_generic_variant = true;
                    self.collect_pattern_type_bindings(pattern, ty, resolver, bindings);
                }
                if matched_generic_variant {
                    return;
                }
                if let Some(type_ref) = resolver.resolve_receiver_type(ty) {
                    self.collect_tuple_struct_pattern_type_bindings(
                        pattern, &type_ref, resolver, bindings,
                    );
                }
            }
            (Pat::Struct(pattern), _) => {
                let Some(type_ref) = resolver.resolve_receiver_type(ty) else {
                    return;
                };
                self.collect_named_pattern_type_bindings(pattern, &type_ref, resolver, bindings);
            }
            (_, Type::Group(group)) => {
                self.collect_pattern_type_bindings(pattern, &group.elem, resolver, bindings);
            }
            (_, Type::Paren(paren)) => {
                self.collect_pattern_type_bindings(pattern, &paren.elem, resolver, bindings);
            }
            _ => {}
        }
    }

    fn collect_tuple_struct_pattern_type_bindings(
        &self,
        pattern: &PatTupleStruct,
        type_ref: &TypeRef,
        resolver: &Resolver<'_>,
        bindings: &mut Vec<(String, TypeRef, Vec<TypeRef>)>,
    ) {
        let Some(constructor_name) = pattern
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
        else {
            return;
        };

        for candidate in resolver.type_ref_candidates(type_ref) {
            if let Some((record, variant)) =
                resolver.enum_variant_record(&candidate, &constructor_name)
            {
                let syn::Fields::Unnamed(fields) = &variant.fields else {
                    continue;
                };
                let field_resolver = Resolver {
                    project: self.resolver.project,
                    package: &record.package,
                    module_path: &record.module_path,
                    aliases: &record.aliases,
                    self_type: None,
                };
                for (pattern, field) in pattern.elems.iter().zip(fields.unnamed.iter()) {
                    self.collect_pattern_type_bindings(
                        pattern,
                        &field.ty,
                        &field_resolver,
                        bindings,
                    );
                }
                return;
            }
        }

        for candidate in resolver.type_ref_candidates(type_ref) {
            if candidate
                .type_path
                .last()
                .is_none_or(|name| name != &constructor_name)
            {
                continue;
            }
            let Some(item) = resolver.find_item(
                &candidate.package,
                &candidate.type_path,
                &[ItemKind::Struct],
            ) else {
                continue;
            };
            let Some(record) = self.resolver.project.items.get(&item) else {
                continue;
            };
            let Item::Struct(item_struct) = &record.item else {
                continue;
            };
            let syn::Fields::Unnamed(fields) = &item_struct.fields else {
                continue;
            };
            let field_resolver = Resolver {
                project: self.resolver.project,
                package: &record.package,
                module_path: &record.module_path,
                aliases: &record.aliases,
                self_type: None,
            };
            for (pattern, field) in pattern.elems.iter().zip(fields.unnamed.iter()) {
                self.collect_pattern_type_bindings(pattern, &field.ty, &field_resolver, bindings);
            }
            return;
        }
    }

    fn collect_named_pattern_type_bindings(
        &self,
        pattern: &syn::PatStruct,
        type_ref: &TypeRef,
        resolver: &Resolver<'_>,
        bindings: &mut Vec<(String, TypeRef, Vec<TypeRef>)>,
    ) {
        let Some(variant_name) = pattern
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
        else {
            return;
        };

        for candidate in resolver.type_ref_candidates(type_ref) {
            if let Some((record, variant)) = resolver.enum_variant_record(&candidate, &variant_name)
            {
                let field_resolver = Resolver {
                    project: self.resolver.project,
                    package: &record.package,
                    module_path: &record.module_path,
                    aliases: &record.aliases,
                    self_type: None,
                };
                for field in &pattern.fields {
                    let Some(ty) = field_member_type(&variant.fields, &field.member) else {
                        continue;
                    };
                    self.collect_pattern_type_bindings(&field.pat, ty, &field_resolver, bindings);
                }
                return;
            }
        }

        self.collect_struct_pattern_type_bindings(pattern, type_ref, resolver, bindings);
    }

    fn collect_struct_pattern_type_bindings(
        &self,
        pattern: &syn::PatStruct,
        type_ref: &TypeRef,
        resolver: &Resolver<'_>,
        bindings: &mut Vec<(String, TypeRef, Vec<TypeRef>)>,
    ) {
        let Some(item) =
            resolver.find_item(&type_ref.package, &type_ref.type_path, &[ItemKind::Struct])
        else {
            return;
        };
        let Some(record) = self.resolver.project.items.get(&item) else {
            return;
        };
        let Item::Struct(item_struct) = &record.item else {
            return;
        };
        let resolver = Resolver {
            project: self.resolver.project,
            package: &record.package,
            module_path: &record.module_path,
            aliases: &record.aliases,
            self_type: None,
        };
        for field in &pattern.fields {
            let Some(ty) = field_member_type(&item_struct.fields, &field.member) else {
                continue;
            };
            self.collect_pattern_type_bindings(&field.pat, ty, &resolver, bindings);
        }
    }

    fn local_binding_trait_bounds(&self, local: &Local) -> Option<(String, Vec<ItemId>)> {
        let (name, explicit_type) = binding_name_and_type(&local.pat)?;
        let trait_items = explicit_type
            .map(|ty| self.trait_items_from_type(ty))
            .unwrap_or_default();
        (!trait_items.is_empty()).then_some((name, trait_items))
    }

    fn infer_expr_type(&self, expression: &Expr) -> Option<TypeRef> {
        match expression {
            Expr::Call(call) => self
                .wrapper_constructor_arg_type(call)
                .or_else(|| self.std_mem_return_type(call))
                .or_else(|| {
                    if let Expr::Path(path) = call.func.as_ref() {
                        self.resolver.type_from_expr_path_call(path)
                    } else {
                        None
                    }
                }),
            Expr::MethodCall(call) => self.method_call_return_type(call),
            Expr::Try(expr) => self
                .expression_type_arguments(&expr.expr)
                .into_iter()
                .next()
                .or_else(|| self.infer_expr_type(&expr.expr)),
            Expr::Index(index) => self.index_output_type(index),
            Expr::Field(field) => {
                let receiver = self.receiver_type(&field.base)?;
                self.resolver.field_type(&receiver, &field.member)
            }
            Expr::Path(path) if path.path.segments.len() == 1 => {
                let name = path.path.segments.first()?.ident.to_string();
                self.variables
                    .get(&name)
                    .cloned()
                    .or_else(|| self.resolver.type_from_value_path(&path.path))
                    .or_else(|| self.resolver.resolve_type_path(&path.path))
            }
            Expr::Path(path) => self
                .resolver
                .type_from_value_path(&path.path)
                .or_else(|| self.resolver.resolve_type_path(&path.path)),
            Expr::Struct(expr) => self.resolver.resolve_type_path(&expr.path),
            Expr::Block(expr) => {
                final_block_expression(&expr.block).and_then(|expr| self.infer_expr_type(expr))
            }
            Expr::Unsafe(expr) => {
                final_block_expression(&expr.block).and_then(|expr| self.infer_expr_type(expr))
            }
            Expr::Reference(reference) => self.infer_expr_type(&reference.expr),
            Expr::Paren(paren) => self.infer_expr_type(&paren.expr),
            _ => None,
        }
    }

    fn method_call_return_type(&self, call: &ExprMethodCall) -> Option<TypeRef> {
        if call.method == "fold" {
            return call
                .args
                .first()
                .and_then(|initial| self.infer_expr_type(initial));
        }
        if matches!(call.method.to_string().as_str(), "remove" | "swap_remove") {
            return self.single_expression_type_argument(&call.receiver);
        }
        if matches!(
            call.method.to_string().as_str(),
            "or_insert" | "or_insert_with" | "or_insert_with_key" | "or_default"
        ) {
            return self.expression_entry_value_type(&call.receiver);
        }
        if matches!(call.method.to_string().as_str(), "as_ref" | "clone") {
            return self.receiver_type(&call.receiver);
        }
        if call.method == "lock" && call.args.is_empty() {
            return self.receiver_type(&call.receiver);
        }
        if matches!(
            call.method.to_string().as_str(),
            "expect" | "unwrap" | "unwrap_or" | "unwrap_or_else"
        ) {
            return self.expression_result_ok_type(&call.receiver).or_else(|| {
                self.expression_type_arguments(&call.receiver)
                    .first()
                    .cloned()
            });
        }
        self.receiver_type(&call.receiver)
            .and_then(|receiver| {
                self.resolver
                    .resolve_methods(&receiver, &call.method.to_string())
                    .into_iter()
                    .find_map(|callable| self.resolver.return_type_from_callable(&callable))
            })
            .or_else(|| {
                self.trait_bound_method_return_type(&call.receiver, &call.method.to_string())
            })
    }

    fn trait_bound_method_return_type(
        &self,
        receiver: &Expr,
        method_name: &str,
    ) -> Option<TypeRef> {
        let associated_type_bindings = self.receiver_associated_type_bindings(receiver);
        self.receiver_trait_bounds(receiver)
            .into_iter()
            .filter(|trait_item| {
                trait_item_contains_method(self.resolver.project, trait_item, method_name)
            })
            .find_map(|trait_item| {
                if let Some(associated_type) = self
                    .resolver
                    .trait_method_return_self_associated_name(&trait_item, method_name)
                {
                    if let Some(type_ref) = associated_type_bindings.get(&associated_type) {
                        return Some(type_ref.clone());
                    }
                }
                self.resolver
                    .trait_method_return_type(&trait_item, method_name)
            })
    }

    fn add_trait_bound_method_dependencies(&mut self, receiver: &Expr, method_name: &str) -> bool {
        let matching_traits = self
            .receiver_trait_bounds(receiver)
            .into_iter()
            .filter(|trait_item| {
                trait_item_contains_method(self.resolver.project, trait_item, method_name)
            })
            .collect::<Vec<_>>();
        if matching_traits.is_empty() {
            return false;
        }
        self.dependencies.items.extend(matching_traits);
        true
    }

    fn add_default_trait_method_dependencies(
        &mut self,
        receiver: &TypeRef,
        method_name: &str,
    ) -> bool {
        let matching_traits =
            default_trait_method_traits_for_receiver(self.resolver.project, receiver, method_name);
        if matching_traits.is_empty() {
            return false;
        }
        self.dependencies.items.extend(matching_traits);
        true
    }

    fn receiver_trait_bounds(&self, expression: &Expr) -> Vec<ItemId> {
        let mut trait_items = Vec::new();
        self.collect_receiver_trait_bounds(expression, &mut trait_items);
        trait_items.sort();
        trait_items.dedup();
        trait_items
    }

    fn collect_receiver_trait_bounds(&self, expression: &Expr, trait_items: &mut Vec<ItemId>) {
        match expression {
            Expr::Path(path) if path.path.segments.len() == 1 => {
                let name = path.path.segments.first().unwrap().ident.to_string();
                if let Some(bounds) = self.variable_trait_bounds.get(&name) {
                    trait_items.extend(bounds.iter().cloned());
                }
            }
            Expr::Reference(reference) => {
                self.collect_receiver_trait_bounds(&reference.expr, trait_items)
            }
            Expr::Paren(paren) => self.collect_receiver_trait_bounds(&paren.expr, trait_items),
            _ => {}
        }
    }

    fn receiver_associated_type_bindings(&self, expression: &Expr) -> HashMap<String, TypeRef> {
        match expression {
            Expr::Path(path) if path.path.segments.len() == 1 => {
                let name = path.path.segments.first().unwrap().ident.to_string();
                self.variable_associated_type_bindings
                    .get(&name)
                    .cloned()
                    .unwrap_or_default()
            }
            Expr::Reference(reference) => self.receiver_associated_type_bindings(&reference.expr),
            Expr::Paren(paren) => self.receiver_associated_type_bindings(&paren.expr),
            _ => HashMap::new(),
        }
    }

    fn wrapper_constructor_arg_type(&self, call: &ExprCall) -> Option<TypeRef> {
        let Expr::Path(path) = call.func.as_ref() else {
            return None;
        };
        let mut segments = path
            .path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string());
        let wrapper = segments.next()?;
        let function = segments.next()?;
        if segments.next().is_some()
            || function != "new"
            || !matches!(wrapper.as_str(), "Arc" | "Box" | "Rc")
        {
            return None;
        }

        call.args
            .first()
            .and_then(|argument| self.infer_expr_type(argument))
    }

    fn std_mem_return_type(&self, call: &ExprCall) -> Option<TypeRef> {
        let Expr::Path(path) = call.func.as_ref() else {
            return None;
        };
        match std_mem_return_function(&path.path)?.as_str() {
            "take" => call.args.first().and_then(|argument| {
                self.receiver_type(argument)
                    .or_else(|| self.infer_expr_type(argument))
            }),
            "replace" => call
                .args
                .first()
                .and_then(|argument| {
                    self.receiver_type(argument)
                        .or_else(|| self.infer_expr_type(argument))
                })
                .or_else(|| {
                    call.args
                        .iter()
                        .nth(1)
                        .and_then(|argument| self.infer_expr_type(argument))
                }),
            _ => None,
        }
    }

    fn add_expected_parse_types_from_return(&mut self, output: &ReturnType) {
        self.expected_parse_types
            .extend(self.resolver.type_arguments_from_return_type(output));
    }

    fn add_expected_error_types_from_return(&mut self, output: &ReturnType) {
        if let Some(type_ref) = self.resolver.error_type_from_return_type(output) {
            self.expected_error_types.insert(type_ref);
        }
    }

    fn add_expected_collect_types_from_return(&mut self, output: &ReturnType) {
        if let Some(type_ref) = self.resolver.type_from_return_type(output) {
            self.expected_collect_types.insert(type_ref);
        }
    }

    fn add_impl_trait_return_dependencies(&mut self, output: &ReturnType, block: &syn::Block) {
        let trait_names = impl_trait_return_bound_names(output);
        if trait_names.is_empty() {
            return;
        }
        let Some(expression) = final_block_expression(block) else {
            return;
        };
        let Some(type_ref) = self.infer_expr_type(expression) else {
            return;
        };
        for trait_name in trait_names {
            self.add_trait_impls_for_type_named(&type_ref, &trait_name);
        }
    }

    fn result_ok_type(&self, expression: &Expr) -> Option<TypeRef> {
        let Expr::Call(call) = expression else {
            return None;
        };
        let Expr::Path(path) = call.func.as_ref() else {
            return None;
        };
        self.resolver.type_from_path_turbofish(&path.path)
    }

    fn add_result_ok_binding_type(&mut self, pattern: &Pat, type_ref: &TypeRef) {
        self.add_result_variant_binding_type(pattern, "Ok", type_ref);
    }

    fn add_result_error_binding_type(&mut self, pattern: &Pat, type_ref: &TypeRef) {
        self.add_result_variant_binding_type(pattern, "Err", type_ref);
    }

    fn add_result_variant_binding_type(
        &mut self,
        pattern: &Pat,
        variant: &str,
        type_ref: &TypeRef,
    ) {
        let Pat::TupleStruct(tuple) = pattern else {
            return;
        };
        if tuple
            .path
            .segments
            .last()
            .is_none_or(|segment| segment.ident != variant)
        {
            return;
        }
        let Some(Pat::Ident(ident)) = tuple.elems.first() else {
            return;
        };
        self.insert_variable_type(ident.ident.to_string(), type_ref.clone());
    }

    fn add_pattern_bindings_for_type(&mut self, pattern: &Pat, type_ref: &TypeRef) {
        match pattern {
            Pat::Ident(ident) => {
                self.insert_variable_type(ident.ident.to_string(), type_ref.clone());
            }
            Pat::Reference(reference) => {
                self.add_pattern_bindings_for_type(&reference.pat, type_ref);
            }
            Pat::Type(pat_type) => {
                if let Some(explicit_type) = self.resolver.resolve_receiver_type(&pat_type.ty) {
                    self.add_pattern_bindings_for_type(&pat_type.pat, &explicit_type);
                } else {
                    self.add_pattern_bindings_for_type(&pat_type.pat, type_ref);
                }
            }
            Pat::TupleStruct(tuple) => {
                let Some(variant_name) = tuple
                    .path
                    .segments
                    .last()
                    .map(|segment| segment.ident.to_string())
                else {
                    return;
                };
                let mut field_types = self
                    .resolver
                    .enum_tuple_variant_field_types(type_ref, &variant_name);
                if field_types.is_empty() {
                    field_types = self
                        .resolver
                        .tuple_struct_field_types(type_ref, &variant_name);
                }
                for (pat, field_type) in tuple.elems.iter().zip(field_types) {
                    self.add_pattern_bindings_for_type(pat, &field_type);
                }
            }
            Pat::Struct(struct_pat) => {
                let Some(variant_name) = struct_pat
                    .path
                    .segments
                    .last()
                    .map(|segment| segment.ident.to_string())
                else {
                    return;
                };
                for field in &struct_pat.fields {
                    if let Some(field_type) = self
                        .resolver
                        .enum_named_variant_field_type(type_ref, &variant_name, &field.member)
                        .or_else(|| self.resolver.field_type(type_ref, &field.member))
                    {
                        self.add_pattern_bindings_for_type(&field.pat, &field_type);
                    }
                }
            }
            Pat::Or(or_pat) => {
                for case in &or_pat.cases {
                    self.add_pattern_bindings_for_type(case, type_ref);
                }
            }
            _ => {}
        }
    }

    fn visit_impl_peers(
        &mut self,
        callable: &CallableId,
        record: &crate::model::MethodRecord,
        keep_trait_peers: bool,
    ) {
        for impl_item in &record.impl_items {
            if impl_item_is_test(impl_item) {
                continue;
            }

            match impl_item {
                ImplItem::Fn(method) if keep_trait_peers => {
                    if method.sig.ident != callable_method(callable) {
                        let mut peer = callable.clone();
                        if let CallableId::Method { method: name, .. } = &mut peer {
                            *name = method.sig.ident.to_string();
                        }
                        self.dependencies.callables.insert(peer);
                    }
                }
                ImplItem::Fn(_) => {}
                _ => visit::visit_impl_item(self, impl_item),
            }
        }
    }

    fn add_external_call_arg_trait_impls(&mut self, call: &ExprCall) {
        for argument in &call.args {
            if let Some(type_ref) = self
                .receiver_type(argument)
                .or_else(|| self.infer_expr_type(argument))
            {
                self.add_non_conversion_trait_impls_for_type(&type_ref);
            }
        }
    }

    fn add_external_method_arg_trait_impls(&mut self, call: &ExprMethodCall) {
        for argument in &call.args {
            if let Some(type_ref) = self
                .receiver_type(argument)
                .or_else(|| self.infer_expr_type(argument))
            {
                self.add_non_conversion_trait_impls_for_type(&type_ref);
            }
        }
    }

    fn add_non_conversion_trait_impls_for_type(&mut self, type_ref: &TypeRef) {
        let type_refs = self.resolver.type_ref_candidates(type_ref);
        for callable in self.resolver.project.methods.keys() {
            let CallableId::Method {
                package,
                type_path,
                trait_path: Some(trait_path),
                ..
            } = callable
            else {
                continue;
            };

            if trait_path_is_conversion_like(trait_path) {
                continue;
            }

            if type_refs
                .iter()
                .any(|candidate| package == &candidate.package && type_path == &candidate.type_path)
            {
                self.dependencies.callables.insert(callable.clone());
            }
        }
    }

    fn add_trait_impls_for_type(&mut self, type_ref: &TypeRef) {
        let type_refs = self.resolver.type_ref_candidates(type_ref);
        for callable in self.resolver.project.methods.keys() {
            let CallableId::Method {
                package,
                type_path,
                trait_path: Some(_),
                ..
            } = callable
            else {
                continue;
            };

            if type_refs
                .iter()
                .any(|candidate| package == &candidate.package && type_path == &candidate.type_path)
            {
                self.dependencies.callables.insert(callable.clone());
            }
        }
    }

    fn add_trait_impls_for_type_named(&mut self, type_ref: &TypeRef, trait_name: &str) {
        let type_refs = self.resolver.type_ref_candidates(type_ref);
        for callable in self.resolver.project.methods.keys() {
            let CallableId::Method {
                package,
                type_path,
                trait_path: Some(trait_path),
                ..
            } = callable
            else {
                continue;
            };

            if type_refs
                .iter()
                .any(|candidate| package == &candidate.package && type_path == &candidate.type_path)
                && trait_path.last().is_some_and(|name| name == trait_name)
            {
                self.dependencies.callables.insert(callable.clone());
            }
        }
    }

    fn add_extension_trait_dependencies_for_method(
        &mut self,
        type_ref: &TypeRef,
        method_name: &str,
    ) {
        let type_refs = self.resolver.type_ref_candidates(type_ref);
        for callable in self.resolver.project.methods.keys() {
            let CallableId::Method {
                package,
                type_path,
                trait_path: Some(trait_path),
                ..
            } = callable
            else {
                continue;
            };
            if !type_refs.iter().any(|candidate| {
                package == &candidate.package
                    && conversion_input_matches_receiver(type_path, candidate)
            }) {
                continue;
            }
            let Some(trait_item) = self
                .resolver
                .resolve_trait_item_in_package(package, trait_path)
                .or_else(|| self.resolver.resolve_trait_item(trait_path))
            else {
                continue;
            };
            if !trait_item_contains_method(self.resolver.project, &trait_item, method_name) {
                continue;
            }
            self.dependencies.items.insert(trait_item);
            self.dependencies.callables.insert(callable.clone());
        }
    }

    fn add_deref_target_method_dependencies(&mut self, type_ref: &TypeRef, method_name: &str) {
        for target in self.resolver.deref_target_types(type_ref) {
            for callable in self.resolver.resolve_methods(&target, method_name) {
                self.add_method_dependency(&callable);
            }
        }
    }

    fn add_conversion_impls_by_trait(&mut self, trait_name: &str, method_name: &str) {
        let matches = self
            .resolver
            .project
            .methods
            .keys()
            .filter_map(|callable| {
                let CallableId::Method {
                    package,
                    type_path,
                    trait_path: Some(trait_path),
                    trait_input_type_paths,
                    method,
                    ..
                } = callable
                else {
                    return None;
                };

                (package == self.resolver.package
                    && method == method_name
                    && trait_path
                        .last()
                        .is_some_and(|candidate| candidate == trait_name)
                    && (self.resolver.resolve_local_type_item(type_path).is_some()
                        || trait_input_type_paths.iter().any(|type_path| {
                            self.resolver.resolve_local_type_item(type_path).is_some()
                        })))
                .then(|| callable.clone())
            })
            .collect::<Vec<_>>();

        if matches.len() > MAX_UNRESOLVED_CONVERSION_CANDIDATES {
            self.dependencies.evidence.unresolved_method_fallbacks += 1;
            self.dependencies
                .evidence
                .capped_unresolved_method_fallbacks += 1;
            return;
        }

        for callable in matches {
            self.dependencies.callables.insert(callable);
        }
    }

    fn add_conversion_impls_to_expected_type(
        &mut self,
        expression: &Expr,
        target: &TypeRef,
        trait_name: &str,
        trait_method_name: &str,
        expression_method_name: &str,
    ) {
        let receiver = match expression {
            Expr::MethodCall(call) if call.method == expression_method_name => self
                .receiver_type(&call.receiver)
                .or_else(|| self.infer_expr_type(&call.receiver)),
            Expr::Call(call) => {
                let Expr::Path(path) = call.func.as_ref() else {
                    return;
                };
                let segments = path_segments(&path.path);
                let Some(method) = segments.last() else {
                    return;
                };
                if method != expression_method_name || call.args.is_empty() {
                    return;
                }
                call.args.first().and_then(|argument| {
                    self.receiver_type(argument)
                        .or_else(|| self.infer_expr_type(argument))
                })
            }
            _ => None,
        };

        if let Some(receiver) = receiver {
            for callable in
                self.resolver
                    .resolve_conversion_impls(&receiver, trait_name, trait_method_name)
            {
                self.dependencies.callables.insert(callable);
            }
            return;
        }

        for callable in
            self.resolver
                .resolve_conversion_impls_to_target(target, trait_name, trait_method_name)
        {
            self.dependencies.callables.insert(callable);
        }
    }

    fn add_conversion_impls_for_associated_call_arg(&mut self, path: &Path, call: &ExprCall) {
        let segments = path_segments(path);
        let Some(method) = segments.last().map(String::as_str) else {
            return;
        };
        let Some((trait_name, method_name)) = (match method {
            "from" => Some(("From", "from")),
            "try_from" => Some(("TryFrom", "try_from")),
            _ => None,
        }) else {
            return;
        };
        if segments.len() < 2 {
            return;
        }

        if let Some(target) = self
            .resolver
            .resolve_type_segments(&segments[..segments.len() - 1])
            .or_else(|| {
                segments.get(segments.len() - 2).map(|name| TypeRef {
                    package: self.resolver.package.to_string(),
                    type_path: vec![name.clone()],
                })
            })
        {
            for callable in
                self.resolver
                    .resolve_conversion_impls_to_target(&target, trait_name, method_name)
            {
                self.dependencies.callables.insert(callable);
            }
        }

        let Some(first_arg) = call.args.first() else {
            return;
        };
        let Some(receiver) = self
            .receiver_type(first_arg)
            .or_else(|| self.infer_expr_type(first_arg))
        else {
            return;
        };

        for callable in self
            .resolver
            .resolve_conversion_impls(&receiver, trait_name, method_name)
        {
            self.dependencies.callables.insert(callable);
        }
    }

    fn add_method_dependency(&mut self, callable: &CallableId) {
        if let CallableId::Method {
            package,
            trait_path: Some(trait_path),
            ..
        } = callable
        {
            if let Some(trait_item) = self
                .resolver
                .resolve_trait_item_in_package(package, trait_path)
                .or_else(|| self.resolver.resolve_trait_item(trait_path))
            {
                self.dependencies.items.insert(trait_item);
            }
        }
        self.dependencies.callables.insert(callable.clone());
    }

    fn add_unresolved_method_fallback(&mut self, method_name: &str, start_line: Option<usize>) {
        self.add_unresolved_method_candidates(method_name, &[], start_line);
    }

    fn add_unresolved_method_candidates(
        &mut self,
        method_name: &str,
        receiver_candidates: &[TypeRef],
        start_line: Option<usize>,
    ) {
        let receiver_candidates = receiver_candidates
            .iter()
            .flat_map(|type_ref| self.resolver.type_ref_candidates(type_ref))
            .collect::<BTreeSet<_>>();
        let mut matches = Vec::new();
        for callable in self.resolver.project.methods.keys() {
            let CallableId::Method {
                package,
                type_path,
                method,
                ..
            } = callable
            else {
                continue;
            };

            if method != method_name || !self.visible_packages.contains(package) {
                continue;
            }

            if !receiver_candidates.is_empty()
                && !receiver_candidates.iter().any(|candidate| {
                    package == &candidate.package
                        && conversion_input_matches_receiver(type_path, candidate)
                })
            {
                continue;
            }

            matches.push(callable.clone());
        }

        if receiver_candidates.is_empty() && matches.len() > MAX_UNRESOLVED_METHOD_NAME_CANDIDATES {
            self.dependencies.evidence.unresolved_method_fallbacks += 1;
            self.dependencies
                .evidence
                .capped_unresolved_method_fallbacks += 1;
            self.dependencies
                .evidence
                .capped_unresolved_method_details
                .push(CappedMethodFallbackEvidence {
                    method_name: method_name.to_string(),
                    candidate_count: matches.len(),
                    receiver_candidate_count: receiver_candidates.len(),
                    package: None,
                    module_path: None,
                    owner: None,
                    file: None,
                    start_line,
                });
            return;
        }

        self.dependencies.evidence.unresolved_method_fallbacks += 1;
        self.dependencies
            .evidence
            .unresolved_method_candidate_matches += matches.len();
        for callable in matches {
            self.add_method_dependency(&callable);
        }
    }

    fn add_peer_trait_impls_for_resolved_methods(&mut self, resolved_methods: &[CallableId]) {
        let resolved_traits = resolved_methods
            .iter()
            .filter_map(|callable| {
                let CallableId::Method {
                    package,
                    type_path,
                    trait_path: Some(trait_path),
                    trait_input_type_paths,
                    method,
                    ..
                } = callable
                else {
                    return None;
                };
                Some((
                    package.clone(),
                    type_path.clone(),
                    trait_path.clone(),
                    trait_input_type_paths.clone(),
                    method.clone(),
                ))
            })
            .collect::<BTreeSet<_>>();

        if resolved_traits.is_empty() {
            return;
        }

        for callable in self.resolver.project.methods.keys() {
            let CallableId::Method {
                package,
                type_path,
                trait_path: Some(trait_path),
                trait_input_type_paths,
                method,
                ..
            } = callable
            else {
                continue;
            };
            if resolved_traits.contains(&(
                package.clone(),
                type_path.clone(),
                trait_path.clone(),
                trait_input_type_paths.clone(),
                method.clone(),
            )) {
                self.add_method_dependency(callable);
            }
        }
    }

    fn add_format_macro_trait_dependencies(&mut self, mac: &Macro) {
        if !macro_path_ends_with(&mac.path, "format")
            && !macro_path_ends_with(&mac.path, "format_args")
            && !macro_path_ends_with(&mac.path, "print")
            && !macro_path_ends_with(&mac.path, "println")
            && !macro_path_ends_with(&mac.path, "eprint")
            && !macro_path_ends_with(&mac.path, "eprintln")
            && !macro_path_ends_with(&mac.path, "write")
            && !macro_path_ends_with(&mac.path, "writeln")
        {
            return;
        }

        let parser = syn::punctuated::Punctuated::<Expr, syn::Token![,]>::parse_terminated;
        let Ok(arguments) = parser.parse2(mac.tokens.clone()) else {
            return;
        };

        if let Some(format_string) = arguments.first().and_then(format_literal_value) {
            for capture in format_string_capture_idents(&format_string) {
                let Ok(path) = syn::parse_str::<Path>(&capture) else {
                    continue;
                };
                self.add_call_path(&path);
                self.add_item_path(&path);
                for type_ref in self.local_value_type_candidates(&capture) {
                    self.add_trait_impls_for_type_named(&type_ref, "Display");
                }
            }
        }

        for argument in arguments.iter().skip(1) {
            self.visit_expr(argument);
            if let Some(type_ref) = self.infer_expr_type(argument) {
                self.add_trait_impls_for_type_named(&type_ref, "Display");
            }
        }
    }

    fn add_expression_macro_dependencies(&mut self, mac: &Macro) {
        let parser = syn::punctuated::Punctuated::<Expr, syn::Token![,]>::parse_terminated;
        let Ok(arguments) = parser.parse2(mac.tokens.clone()) else {
            return;
        };
        for argument in arguments {
            self.visit_expr(&argument);
        }
    }

    fn add_matches_macro_dependencies(&mut self, mac: &Macro) {
        if !macro_path_ends_with(&mac.path, "matches") {
            return;
        }
        let Some((subject_tokens, pattern_tokens)) =
            split_macro_tokens_at_first_top_level_comma(&mac.tokens)
        else {
            return;
        };
        let Ok(subject) = syn::parse2::<Expr>(subject_tokens) else {
            return;
        };

        self.visit_expr(&subject);
        let payload_types = self.expression_type_arguments(&subject);
        let ok_type = self.expression_result_ok_type(&subject);
        let error_type = self.expression_result_error_type(&subject);
        let bindings = matches_macro_pattern_bindings(
            &pattern_tokens,
            payload_types.first(),
            ok_type.as_ref().or_else(|| payload_types.first()),
            error_type
                .as_ref()
                .or_else(|| payload_types.get(1))
                .or_else(|| payload_types.last()),
        );
        if bindings.is_empty() {
            return;
        }

        let variables = self.variables.clone();
        let variable_candidates = self.variable_candidates.clone();
        let variable_type_arguments = self.variable_type_arguments.clone();
        let variable_result_ok_types = self.variable_result_ok_types.clone();
        let variable_result_error_types = self.variable_result_error_types.clone();
        self.push_local_value_scope();
        for (name, type_ref) in bindings {
            self.insert_variable_type(name.clone(), type_ref);
            self.local_value_scopes
                .last_mut()
                .expect("macro binding scope should exist")
                .insert(name);
        }
        self.add_macro_token_dependencies(&pattern_tokens);
        self.pop_local_value_scope();
        self.variables = variables;
        self.variable_candidates = variable_candidates;
        self.variable_type_arguments = variable_type_arguments;
        self.variable_result_ok_types = variable_result_ok_types;
        self.variable_result_error_types = variable_result_error_types;
    }

    fn add_local_initializer_trait_dependencies(&mut self, local: &Local, type_ref: &TypeRef) {
        if local
            .init
            .as_ref()
            .is_some_and(|init| self.expr_contains_unresolved_external_call(&init.expr))
        {
            self.add_trait_impls_for_type(type_ref);
        }
    }

    fn expr_contains_unresolved_external_call(&self, expression: &Expr) -> bool {
        let mut visitor = UnresolvedExternalCallVisitor {
            resolver: &self.resolver,
            found: false,
        };
        visitor.visit_expr(expression);
        visitor.found
    }

    fn add_derive_field_trait_dependencies(
        &mut self,
        attrs: &[syn::Attribute],
        fields: &syn::Fields,
    ) {
        for field in fields.iter() {
            self.add_derive_field_trait_dependencies_for_field(attrs, field);
        }
    }

    fn add_derive_field_trait_dependencies_for_field(
        &mut self,
        attrs: &[syn::Attribute],
        field: &Field,
    ) {
        let derive_traits = derive_trait_names(attrs);
        if derive_traits.is_empty() {
            return;
        }

        for type_ref in self.resolver.type_refs_in_type(&field.ty) {
            for trait_name in &derive_traits {
                self.add_trait_impls_for_type_named(&type_ref, trait_name);
                if trait_name == "Error" {
                    self.add_trait_impls_for_type_named(&type_ref, "Display");
                }
            }
        }
    }

    fn add_generic_field_type_trait_dependencies(&mut self, fields: &syn::Fields) {
        for field in fields.iter() {
            self.add_generic_field_type_trait_dependencies_for_field(field);
        }
    }

    fn add_generic_field_type_trait_dependencies_for_field(&mut self, field: &Field) {
        for type_ref in self.resolver.type_argument_refs_in_type(&field.ty) {
            self.add_non_conversion_trait_impls_for_type(&type_ref);
        }
    }

    fn add_serde_default_field_dependencies(&mut self, fields: &syn::Fields) {
        for field in fields.iter() {
            self.add_serde_default_field_dependencies_for_field(field);
        }
    }

    fn add_serde_default_field_dependencies_for_field(&mut self, field: &Field) {
        if !attrs_include_serde_default(&field.attrs) {
            return;
        }
        for type_ref in self.resolver.type_refs_in_type(&field.ty) {
            self.add_trait_impls_for_type_named(&type_ref, "Default");
        }
    }

    fn add_parse_method_trait_dependencies(&mut self) {
        for type_ref in self.expected_parse_types.clone() {
            self.add_trait_impls_for_type_named(&type_ref, "FromStr");
        }
    }

    fn add_parse_turbofish_trait_dependencies(&mut self, call: &ExprMethodCall) {
        if call.method != "parse" {
            return;
        }
        for type_ref in self.method_turbofish_type_refs(call) {
            self.add_trait_impls_for_type_named(&type_ref, "FromStr");
        }
    }

    fn add_collect_method_trait_dependencies(&mut self) {
        for type_ref in self.expected_collect_types.clone() {
            self.add_trait_impls_for_type_named(&type_ref, "FromIterator");
        }
    }

    fn add_method_turbofish_trait_dependencies(&mut self, call: &ExprMethodCall) {
        let trait_name = match call.method.to_string().as_str() {
            "get" => "FromSql",
            _ => return,
        };
        let Some(arguments) = &call.turbofish else {
            self.add_local_trait_impls_named(trait_name);
            return;
        };
        let type_refs = arguments
            .args
            .iter()
            .filter_map(|argument| {
                let GenericArgument::Type(ty) = argument else {
                    return None;
                };
                self.resolver.resolve_receiver_type(ty)
            })
            .collect::<BTreeSet<_>>();
        for type_ref in type_refs {
            self.add_trait_impls_for_type_named(&type_ref, trait_name);
        }
    }

    fn add_rusqlite_param_macro_trait_dependencies(&mut self, mac: &Macro) {
        if !macro_path_ends_with(&mac.path, "params")
            && !macro_path_ends_with(&mac.path, "named_params")
        {
            return;
        }

        for segments in token_path_candidates(&mac.tokens) {
            if segments.len() == 1 {
                if let Some(type_ref) = self.variables.get(&segments[0]).cloned() {
                    self.add_trait_impls_for_type_named(&type_ref, "ToSql");
                }
                continue;
            }
            if segments.len() < 2 {
                continue;
            }
            let type_segments = self
                .resolver
                .apply_alias(segments[..segments.len() - 1].to_vec());
            if let Some(type_ref) = self.resolver.resolve_type_segments(&type_segments) {
                self.add_trait_impls_for_type_named(&type_ref, "ToSql");
            }
        }
    }

    fn add_local_trait_impls_named(&mut self, trait_name: &str) {
        for callable in self.resolver.project.methods.keys() {
            let CallableId::Method {
                package,
                trait_path: Some(trait_path),
                ..
            } = callable
            else {
                continue;
            };
            if package == self.resolver.package
                && trait_path.last().is_some_and(|name| name == trait_name)
            {
                self.dependencies.callables.insert(callable.clone());
            }
        }
    }

    fn bind_pattern_type(&mut self, pattern: &Pat, type_ref: &TypeRef) {
        match pattern {
            Pat::Ident(ident) => {
                self.insert_variable_type(ident.ident.to_string(), type_ref.clone());
            }
            Pat::Reference(reference) => self.bind_pattern_type(&reference.pat, type_ref),
            Pat::Type(pat_type) => {
                if let Some(explicit_type) = self.resolver.resolve_receiver_type(&pat_type.ty) {
                    let candidates = self
                        .resolver
                        .receiver_type_candidates_from_type(&pat_type.ty);
                    self.bind_pattern_type_candidates(&pat_type.pat, &explicit_type, candidates);
                } else {
                    self.bind_pattern_type(&pat_type.pat, type_ref);
                }
            }
            _ => {}
        }
    }

    fn bind_pattern_type_candidates(
        &mut self,
        pattern: &Pat,
        type_ref: &TypeRef,
        candidates: Vec<TypeRef>,
    ) {
        match pattern {
            Pat::Ident(ident) => self.insert_variable_candidates(
                ident.ident.to_string(),
                type_ref.clone(),
                candidates,
            ),
            Pat::Reference(reference) => {
                self.bind_pattern_type_candidates(&reference.pat, type_ref, candidates)
            }
            Pat::Type(pat_type) => {
                if let Some(explicit_type) = self.resolver.resolve_receiver_type(&pat_type.ty) {
                    let explicit_candidates = self
                        .resolver
                        .receiver_type_candidates_from_type(&pat_type.ty);
                    self.bind_pattern_type_candidates(
                        &pat_type.pat,
                        &explicit_type,
                        explicit_candidates,
                    );
                } else {
                    self.bind_pattern_type_candidates(&pat_type.pat, type_ref, candidates);
                }
            }
            _ => self.bind_pattern_type(pattern, type_ref),
        }
    }

    fn bind_pattern_type_arguments(&mut self, pattern: &Pat, type_arguments: &[TypeRef]) {
        if type_arguments.is_empty() {
            return;
        }
        let mut type_arguments = type_arguments.to_vec();
        dedup_type_refs_preserve_order(&mut type_arguments);
        match pattern {
            Pat::Ident(ident) => {
                self.variable_type_arguments
                    .insert(ident.ident.to_string(), type_arguments);
            }
            Pat::Reference(reference) => {
                self.bind_pattern_type_arguments(&reference.pat, &type_arguments)
            }
            Pat::Type(pat_type) => self.bind_pattern_type_arguments(&pat_type.pat, &type_arguments),
            _ => {}
        }
    }

    fn bind_single_payload_pattern(&mut self, pattern: &Pat, type_ref: &TypeRef) {
        if let Pat::TupleStruct(tuple) = pattern {
            if tuple.elems.len() == 1
                && tuple.path.segments.last().is_some_and(|segment| {
                    matches!(segment.ident.to_string().as_str(), "Some" | "Ok" | "Err")
                })
            {
                if let Some(inner) = tuple.elems.first() {
                    self.add_pattern_bindings_for_type(inner, type_ref);
                    return;
                }
            }
        }

        self.add_pattern_bindings_for_type(pattern, type_ref);
    }

    fn expression_type_arguments(&self, expression: &Expr) -> Vec<TypeRef> {
        match expression {
            Expr::Path(path) if path.path.segments.len() == 1 => {
                let name = path.path.segments.first().unwrap().ident.to_string();
                self.variable_type_arguments
                    .get(&name)
                    .cloned()
                    .unwrap_or_else(|| self.resolver.type_arguments_from_value_path(&path.path))
            }
            Expr::Path(path) => self.resolver.type_arguments_from_value_path(&path.path),
            Expr::Field(field) => {
                let mut type_arguments = Vec::new();
                for receiver in self.receiver_type_candidates(&field.base) {
                    type_arguments
                        .extend(self.resolver.field_type_arguments(&receiver, &field.member));
                }
                type_arguments.sort();
                type_arguments.dedup();
                type_arguments
            }
            Expr::Call(call) => {
                let Expr::Path(path) = call.func.as_ref() else {
                    return Vec::new();
                };
                let callables = if path.qself.is_some() {
                    self.resolver.resolve_qself_call(path)
                } else {
                    self.resolver.resolve_call_path(&path.path)
                };
                callables
                    .iter()
                    .flat_map(|callable| {
                        self.resolver.return_type_arguments_from_callable(callable)
                    })
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect()
            }
            Expr::MethodCall(call) => {
                if call.method == "collect" {
                    return self.method_turbofish_type_argument_refs(call);
                }
                if call.method == "parse" {
                    return self.method_turbofish_type_refs(call);
                }
                if matches!(
                    call.method.to_string().as_str(),
                    "values" | "values_mut" | "into_values"
                ) {
                    return self
                        .expression_map_value_type(&call.receiver)
                        .into_iter()
                        .collect();
                }
                if call.method == "keys" && self.expression_map_value_type(&call.receiver).is_some()
                {
                    return self
                        .expression_type_arguments(&call.receiver)
                        .first()
                        .cloned()
                        .into_iter()
                        .collect();
                }
                if matches!(
                    call.method.to_string().as_str(),
                    "as_ref"
                        | "as_slice"
                        | "as_mut"
                        | "as_mut_slice"
                        | "by_ref"
                        | "chain"
                        | "chunks"
                        | "chunks_exact"
                        | "clone"
                        | "cloned"
                        | "copied"
                        | "cycle"
                        | "drain"
                        | "filter"
                        | "find"
                        | "flatten"
                        | "fuse"
                        | "inspect"
                        | "into_sorted_vec"
                        | "into_iter"
                        | "iter"
                        | "iter_mut"
                        | "make_contiguous"
                        | "max_by"
                        | "max_by_key"
                        | "min_by"
                        | "min_by_key"
                        | "ok_or"
                        | "ok_or_else"
                        | "or"
                        | "or_else"
                        | "peekable"
                        | "range"
                        | "range_mut"
                        | "reduce"
                        | "rev"
                        | "rchunks"
                        | "rchunks_exact"
                        | "rsplit"
                        | "rsplitn"
                        | "skip_while"
                        | "split"
                        | "split_inclusive"
                        | "splitn"
                        | "step_by"
                        | "take_while"
                        | "windows"
                ) {
                    return self.expression_type_arguments(&call.receiver);
                }
                if matches!(call.method.to_string().as_str(), "ok" | "err") {
                    return match call.method.to_string().as_str() {
                        "ok" => self
                            .expression_result_ok_type(&call.receiver)
                            .into_iter()
                            .collect(),
                        "err" => self
                            .expression_result_error_type(&call.receiver)
                            .into_iter()
                            .collect(),
                        _ => Vec::new(),
                    };
                }
                if matches!(call.method.to_string().as_str(), "then" | "then_some") {
                    return call
                        .args
                        .first()
                        .and_then(|argument| match argument {
                            Expr::Closure(closure) if call.method == "then" => {
                                self.infer_expr_type(&closure.body)
                            }
                            _ if call.method == "then_some" => self.infer_expr_type(argument),
                            _ => None,
                        })
                        .into_iter()
                        .collect();
                }
                if call.method == "xor" {
                    let mut type_arguments = self.expression_type_arguments(&call.receiver);
                    if let Some(argument) = call.args.first() {
                        type_arguments.extend(self.expression_type_arguments(argument));
                    }
                    type_arguments.sort();
                    type_arguments.dedup();
                    return type_arguments;
                }
                if call.method == "zip" {
                    let mut type_arguments = self.expression_type_arguments(&call.receiver);
                    if let Some(argument) = call.args.first() {
                        type_arguments.extend(self.expression_type_arguments(argument));
                    }
                    return type_arguments;
                }
                let Some(receiver) = self.receiver_type(&call.receiver) else {
                    return Vec::new();
                };
                self.resolver
                    .resolve_methods(&receiver, &call.method.to_string())
                    .iter()
                    .flat_map(|callable| {
                        self.resolver.return_type_arguments_from_callable(callable)
                    })
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect()
            }
            Expr::Block(expr) => final_block_expression(&expr.block)
                .map(|expr| self.expression_type_arguments(expr))
                .unwrap_or_default(),
            Expr::Unsafe(expr) => final_block_expression(&expr.block)
                .map(|expr| self.expression_type_arguments(expr))
                .unwrap_or_default(),
            Expr::Reference(reference) => self.expression_type_arguments(&reference.expr),
            Expr::Paren(paren) => self.expression_type_arguments(&paren.expr),
            _ => Vec::new(),
        }
    }

    fn receiver_expression_type_arguments(&self, expression: &Expr) -> Vec<TypeRef> {
        let mut type_arguments = self.expression_type_arguments(expression);
        if let Some(receiver) = self.receiver_type(expression) {
            let receiver_candidates = self.resolver.type_ref_candidates(&receiver);
            type_arguments.retain(|type_ref| !receiver_candidates.contains(type_ref));
        }
        type_arguments
    }

    fn expression_result_ok_type(&self, expression: &Expr) -> Option<TypeRef> {
        match expression {
            Expr::Path(path) if path.path.segments.len() == 1 => {
                let name = path.path.segments.first().unwrap().ident.to_string();
                self.variable_result_ok_types.get(&name).cloned()
            }
            Expr::Call(call) => {
                let Expr::Path(path) = call.func.as_ref() else {
                    return None;
                };
                let callables = if path.qself.is_some() {
                    self.resolver.resolve_qself_call(path)
                } else {
                    self.resolver.resolve_call_path(&path.path)
                };
                callables
                    .iter()
                    .find_map(|callable| self.resolver.return_ok_type_from_callable(callable))
            }
            Expr::MethodCall(call) => {
                if call.method == "collect" {
                    if let Some(ok_type) = self.collect_turbofish_result_ok_type(call) {
                        return Some(ok_type);
                    }
                }
                if matches!(
                    call.method.to_string().as_str(),
                    "max_by" | "max_by_key" | "min_by" | "min_by_key" | "reduce"
                ) {
                    if let Some(ok_type) = self.expression_type_arguments(&call.receiver).first() {
                        return Some(ok_type.clone());
                    }
                }
                if call.method == "find" {
                    if let Some(ok_type) = self.expression_type_arguments(&call.receiver).first() {
                        return Some(ok_type.clone());
                    }
                }
                if matches!(
                    call.method.to_string().as_str(),
                    "get" | "get_mut" | "remove"
                ) {
                    if let Some(value_type) = self.expression_map_value_type(&call.receiver) {
                        return Some(value_type);
                    }
                }
                if matches!(
                    call.method.to_string().as_str(),
                    "first"
                        | "get"
                        | "get_mut"
                        | "peek"
                        | "pop"
                        | "take"
                        | "replace"
                        | "front"
                        | "back"
                        | "pop_front"
                        | "pop_back"
                        | "pop_first"
                        | "pop_last"
                ) {
                    if let Some(ok_type) = self.single_expression_type_argument(&call.receiver) {
                        return Some(ok_type);
                    }
                }
                if matches!(call.method.to_string().as_str(), "last" | "next" | "nth") {
                    if let Some(ok_type) = self.expression_type_arguments(&call.receiver).first() {
                        return Some(ok_type.clone());
                    }
                }
                if call.method == "ok" {
                    return self.expression_result_ok_type(&call.receiver);
                }
                if call.method == "err" {
                    return self.expression_result_error_type(&call.receiver);
                }
                if call.method == "try_fold" {
                    if let Some(ok_type) = call
                        .args
                        .first()
                        .and_then(|initial| self.infer_expr_type(initial))
                    {
                        return Some(ok_type);
                    }
                }
                if matches!(
                    call.method.to_string().as_str(),
                    "map_err"
                        | "ok_or"
                        | "ok_or_else"
                        | "or"
                        | "or_else"
                        | "inspect_err"
                        | "as_ref"
                        | "as_mut"
                ) {
                    return self.expression_result_ok_type(&call.receiver);
                }
                self.parse_method_target_type(call).or_else(|| {
                    self.resolved_methods_for_call(call)
                        .iter()
                        .find_map(|callable| self.resolver.return_ok_type_from_callable(callable))
                })
            }
            Expr::Block(expr) => final_block_expression(&expr.block)
                .and_then(|expr| self.expression_result_ok_type(expr)),
            Expr::Unsafe(expr) => final_block_expression(&expr.block)
                .and_then(|expr| self.expression_result_ok_type(expr)),
            Expr::Reference(reference) => self.expression_result_ok_type(&reference.expr),
            Expr::Paren(paren) => self.expression_result_ok_type(&paren.expr),
            _ => None,
        }
    }

    fn single_expression_type_argument(&self, expression: &Expr) -> Option<TypeRef> {
        let mut type_arguments = self.expression_type_arguments(expression);
        (type_arguments.len() == 1).then(|| type_arguments.remove(0))
    }

    fn expression_map_key_type(&self, expression: &Expr) -> Option<TypeRef> {
        match expression {
            Expr::Path(path) if path.path.segments.len() == 1 => {
                let name = path.path.segments.first()?.ident.to_string();
                self.variable_map_key_types.get(&name).cloned()
            }
            Expr::Call(call) => {
                let Expr::Path(path) = call.func.as_ref() else {
                    return None;
                };
                let callables = if path.qself.is_some() {
                    self.resolver.resolve_qself_call(path)
                } else {
                    self.resolver.resolve_call_path(&path.path)
                };
                callables
                    .iter()
                    .find_map(|callable| self.resolver.return_map_key_type_from_callable(callable))
            }
            Expr::MethodCall(call) if call.method == "collect" => {
                self.collect_turbofish_map_key_type(call)
            }
            Expr::MethodCall(call)
                if matches!(
                    call.method.to_string().as_str(),
                    "as_ref" | "as_mut" | "clone"
                ) =>
            {
                self.expression_map_key_type(&call.receiver)
            }
            Expr::Block(expr) => final_block_expression(&expr.block)
                .and_then(|expr| self.expression_map_key_type(expr)),
            Expr::Unsafe(expr) => final_block_expression(&expr.block)
                .and_then(|expr| self.expression_map_key_type(expr)),
            Expr::Reference(reference) => self.expression_map_key_type(&reference.expr),
            Expr::Paren(paren) => self.expression_map_key_type(&paren.expr),
            _ => None,
        }
    }

    fn expression_map_value_type(&self, expression: &Expr) -> Option<TypeRef> {
        match expression {
            Expr::Path(path) if path.path.segments.len() == 1 => {
                let name = path.path.segments.first()?.ident.to_string();
                self.variable_map_value_types.get(&name).cloned()
            }
            Expr::Call(call) => {
                let Expr::Path(path) = call.func.as_ref() else {
                    return None;
                };
                let callables = if path.qself.is_some() {
                    self.resolver.resolve_qself_call(path)
                } else {
                    self.resolver.resolve_call_path(&path.path)
                };
                callables.iter().find_map(|callable| {
                    self.resolver.return_map_value_type_from_callable(callable)
                })
            }
            Expr::MethodCall(call) if call.method == "collect" => {
                self.collect_turbofish_map_value_type(call)
            }
            Expr::MethodCall(call)
                if matches!(
                    call.method.to_string().as_str(),
                    "as_ref" | "as_mut" | "clone"
                ) =>
            {
                self.expression_map_value_type(&call.receiver)
            }
            Expr::Block(expr) => final_block_expression(&expr.block)
                .and_then(|expr| self.expression_map_value_type(expr)),
            Expr::Unsafe(expr) => final_block_expression(&expr.block)
                .and_then(|expr| self.expression_map_value_type(expr)),
            Expr::Reference(reference) => self.expression_map_value_type(&reference.expr),
            Expr::Paren(paren) => self.expression_map_value_type(&paren.expr),
            _ => None,
        }
    }

    fn expression_entry_value_type(&self, expression: &Expr) -> Option<TypeRef> {
        match expression {
            Expr::Path(path) if path.path.segments.len() == 1 => {
                let name = path.path.segments.first()?.ident.to_string();
                self.variable_entry_value_types.get(&name).cloned()
            }
            Expr::MethodCall(call) => match call.method.to_string().as_str() {
                "entry" => self.expression_map_value_type(&call.receiver),
                "and_modify" => self.expression_entry_value_type(&call.receiver),
                "as_ref" | "as_mut" | "clone" => self.expression_entry_value_type(&call.receiver),
                _ => None,
            },
            Expr::Block(expr) => final_block_expression(&expr.block)
                .and_then(|expr| self.expression_entry_value_type(expr)),
            Expr::Unsafe(expr) => final_block_expression(&expr.block)
                .and_then(|expr| self.expression_entry_value_type(expr)),
            Expr::Reference(reference) => self.expression_entry_value_type(&reference.expr),
            Expr::Paren(paren) => self.expression_entry_value_type(&paren.expr),
            _ => None,
        }
    }

    fn expression_result_error_type(&self, expression: &Expr) -> Option<TypeRef> {
        match expression {
            Expr::Path(path) if path.path.segments.len() == 1 => {
                let name = path.path.segments.first().unwrap().ident.to_string();
                self.variable_result_error_types.get(&name).cloned()
            }
            Expr::Call(call) => {
                let Expr::Path(path) = call.func.as_ref() else {
                    return None;
                };
                let callables = if path.qself.is_some() {
                    self.resolver.resolve_qself_call(path)
                } else {
                    self.resolver.resolve_call_path(&path.path)
                };
                callables
                    .iter()
                    .find_map(|callable| self.resolver.return_error_type_from_callable(callable))
            }
            Expr::MethodCall(call) => {
                if call.method == "collect" {
                    if let Some(error_type) = self.collect_turbofish_result_error_type(call) {
                        return Some(error_type);
                    }
                }
                if call.method == "try_for_each" {
                    if let Some(error_type) = self.single_payload_closure_error_type(call) {
                        return Some(error_type);
                    }
                }
                if call.method == "try_fold" {
                    if let Some(error_type) = self.fold_closure_error_type(call) {
                        return Some(error_type);
                    }
                }
                if matches!(
                    call.method.to_string().as_str(),
                    "map" | "and_then" | "inspect" | "is_ok_and" | "as_ref" | "as_mut"
                ) {
                    return self.expression_result_error_type(&call.receiver);
                }
                self.parse_method_target_type(call)
                    .and_then(|target| {
                        self.resolver
                            .from_str_error_types(&target)
                            .into_iter()
                            .next()
                    })
                    .or_else(|| {
                        self.resolved_methods_for_call(call)
                            .iter()
                            .find_map(|callable| {
                                self.resolver.return_error_type_from_callable(callable)
                            })
                    })
            }
            Expr::Block(expr) => final_block_expression(&expr.block)
                .and_then(|expr| self.expression_result_error_type(expr)),
            Expr::Unsafe(expr) => final_block_expression(&expr.block)
                .and_then(|expr| self.expression_result_error_type(expr)),
            Expr::Reference(reference) => self.expression_result_error_type(&reference.expr),
            Expr::Paren(paren) => self.expression_result_error_type(&paren.expr),
            _ => None,
        }
    }

    fn resolved_methods_for_call(&self, call: &ExprMethodCall) -> Vec<CallableId> {
        let method = call.method.to_string();
        let mut receivers = self.receiver_type_candidates(&call.receiver);
        if let Some(receiver) = self.receiver_type(&call.receiver) {
            receivers.push(receiver);
        }
        receivers.sort();
        receivers.dedup();

        let mut resolved_methods = receivers
            .iter()
            .flat_map(|receiver| self.resolver.resolve_methods(receiver, &method))
            .collect::<Vec<_>>();
        resolved_methods.sort();
        resolved_methods.dedup();
        resolved_methods
    }

    fn parse_method_target_type(&self, call: &ExprMethodCall) -> Option<TypeRef> {
        (call.method == "parse")
            .then(|| self.method_turbofish_type_refs(call).into_iter().next())
            .flatten()
    }

    fn collect_turbofish_map_key_type(&self, call: &ExprMethodCall) -> Option<TypeRef> {
        self.collect_turbofish_target_type(call)
            .and_then(|target| self.resolver.map_key_type(target))
    }

    fn collect_turbofish_map_value_type(&self, call: &ExprMethodCall) -> Option<TypeRef> {
        self.collect_turbofish_target_type(call)
            .and_then(|target| self.resolver.map_value_type(target))
    }

    fn collect_turbofish_result_ok_type(&self, call: &ExprMethodCall) -> Option<TypeRef> {
        self.collect_turbofish_target_type(call)
            .and_then(|target| self.resolver.result_ok_type(target))
    }

    fn collect_turbofish_result_error_type(&self, call: &ExprMethodCall) -> Option<TypeRef> {
        self.collect_turbofish_target_type(call)
            .and_then(|target| self.resolver.result_error_type(target))
    }

    fn collect_turbofish_target_type<'b>(&self, call: &'b ExprMethodCall) -> Option<&'b Type> {
        if call.method != "collect" {
            return None;
        }
        let arguments = call.turbofish.as_ref()?;
        arguments.args.iter().find_map(|argument| {
            let GenericArgument::Type(ty) = argument else {
                return None;
            };
            Some(ty)
        })
    }

    fn method_turbofish_type_refs(&self, call: &ExprMethodCall) -> Vec<TypeRef> {
        let Some(arguments) = &call.turbofish else {
            return Vec::new();
        };
        let mut type_refs = arguments
            .args
            .iter()
            .filter_map(|argument| {
                let GenericArgument::Type(ty) = argument else {
                    return None;
                };
                self.resolver.resolve_receiver_type(ty)
            })
            .collect::<Vec<_>>();
        dedup_type_refs_preserve_order(&mut type_refs);
        type_refs
    }

    fn method_turbofish_type_argument_refs(&self, call: &ExprMethodCall) -> Vec<TypeRef> {
        let Some(arguments) = &call.turbofish else {
            return Vec::new();
        };
        let mut type_refs = Vec::new();
        for argument in &arguments.args {
            let GenericArgument::Type(ty) = argument else {
                continue;
            };
            type_refs.extend(self.resolver.project_type_argument_refs_in_type(ty));
        }
        dedup_type_refs_preserve_order(&mut type_refs);
        type_refs
    }

    fn add_closure_arg_dependencies(
        &mut self,
        call: &ExprMethodCall,
        resolved_methods: &[CallableId],
    ) {
        for (arg_index, argument) in call.args.iter().enumerate() {
            let Expr::Closure(closure) = argument else {
                continue;
            };
            let input_types = resolved_methods
                .iter()
                .flat_map(|callable| {
                    self.resolver
                        .closure_argument_input_types(callable, arg_index)
                })
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            let receiver_type_arguments = self.receiver_expression_type_arguments(&call.receiver);
            let receiver_input_types = resolved_methods
                .iter()
                .flat_map(|callable| {
                    self.resolver
                        .closure_argument_input_types_from_receiver_arguments(
                            callable,
                            arg_index,
                            &receiver_type_arguments,
                        )
                })
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            let input_types = if receiver_input_types.is_empty() {
                input_types
            } else {
                receiver_input_types
            };
            if input_types.is_empty() {
                continue;
            }

            self.visit_closure_with_input_types(closure, &input_types);
        }
    }

    fn add_call_closure_arg_dependencies(
        &mut self,
        call: &ExprCall,
        resolved_callables: &[CallableId],
    ) {
        for (arg_index, argument) in call.args.iter().enumerate() {
            let Expr::Closure(closure) = argument else {
                continue;
            };
            let input_types = resolved_callables
                .iter()
                .flat_map(|callable| {
                    self.resolver
                        .closure_argument_input_types(callable, arg_index)
                })
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            if input_types.is_empty() {
                continue;
            }

            self.visit_closure_with_input_types(closure, &input_types);
        }
    }

    fn add_generic_conversion_call_arg_dependencies(
        &mut self,
        call: &ExprCall,
        resolved_callables: &[CallableId],
    ) {
        for callable in resolved_callables {
            let return_error = self.resolver.return_error_type_from_callable(callable);
            for (arg_index, argument) in call.args.iter().enumerate() {
                let Some((input_type, _)) = self.resolver.callable_typed_input(callable, arg_index)
                else {
                    continue;
                };
                let Some(generic_name) = generic_parameter_name_from_type(input_type) else {
                    continue;
                };
                let Some(argument_type) = self
                    .receiver_type(argument)
                    .or_else(|| self.infer_expr_type(argument))
                else {
                    continue;
                };
                for bound in self
                    .resolver
                    .generic_conversion_bounds_for_callable(callable, &generic_name)
                {
                    for conversion in self.resolver.resolve_conversion_impls_from_to(
                        &argument_type,
                        &bound.target,
                        &bound.impl_trait_name,
                        &bound.method_name,
                    ) {
                        self.dependencies.callables.insert(conversion);
                    }
                    if let (Some(source_error), Some(target_error)) =
                        (bound.error.as_ref(), return_error.as_ref())
                    {
                        for conversion in self.resolver.resolve_conversion_impls_from_to(
                            source_error,
                            target_error,
                            "From",
                            "from",
                        ) {
                            self.dependencies.callables.insert(conversion);
                        }
                    }
                }
            }
        }
    }

    fn receiver_has_generic_conversion_bound(
        &self,
        expression: &Expr,
        impl_trait_name: &str,
        method_name: &str,
    ) -> bool {
        let Expr::Path(path) = expression else {
            return false;
        };
        if path.qself.is_some() || path.path.segments.len() != 1 {
            return false;
        }
        let Some(name) = path
            .path
            .segments
            .first()
            .map(|segment| segment.ident.to_string())
        else {
            return false;
        };
        self.variable_conversion_bounds
            .get(&name)
            .is_some_and(|bounds| {
                bounds.iter().any(|bound| {
                    bound.impl_trait_name == impl_trait_name && bound.method_name == method_name
                })
            })
    }

    fn visit_closure_with_input_types(
        &mut self,
        closure: &syn::ExprClosure,
        input_types: &[TypeRef],
    ) {
        let variables = self.variables.clone();
        let variable_candidates = self.variable_candidates.clone();
        let variable_type_arguments = self.variable_type_arguments.clone();
        let variable_result_ok_types = self.variable_result_ok_types.clone();
        let variable_result_error_types = self.variable_result_error_types.clone();
        for (pattern, type_ref) in closure.inputs.iter().zip(input_types.iter()) {
            self.bind_pattern_type(pattern, type_ref);
        }
        self.visit_expr(&closure.body);
        self.variables = variables;
        self.variable_candidates = variable_candidates;
        self.variable_type_arguments = variable_type_arguments;
        self.variable_result_ok_types = variable_result_ok_types;
        self.variable_result_error_types = variable_result_error_types;
    }

    fn visit_fold_method_call(&mut self, call: &ExprMethodCall) -> bool {
        if !matches!(call.method.to_string().as_str(), "fold" | "try_fold") || call.args.len() != 2
        {
            return false;
        }

        let mut args = call.args.iter();
        let Some(initial) = args.next() else {
            return false;
        };
        let Some(closure_expr) = args.next() else {
            return false;
        };
        let Some(accumulator_type) = self.infer_expr_type(initial) else {
            return false;
        };
        let Expr::Closure(closure) = closure_expr else {
            return false;
        };
        let Some(accumulator_pat) = closure.inputs.first() else {
            return false;
        };

        self.visit_expr(&call.receiver);
        self.visit_expr(initial);

        let item_types = self.expression_type_arguments(&call.receiver);
        let variables = self.variables.clone();
        let variable_candidates = self.variable_candidates.clone();
        let variable_type_arguments = self.variable_type_arguments.clone();
        let variable_result_ok_types = self.variable_result_ok_types.clone();
        let variable_result_error_types = self.variable_result_error_types.clone();
        self.bind_pattern_type(accumulator_pat, &accumulator_type);
        for (input, item_type) in closure.inputs.iter().skip(1).zip(item_types.iter()) {
            self.bind_pattern_type(input, item_type);
        }
        if closure.inputs.len() > item_types.len() + 1 {
            for input in closure.inputs.iter().skip(item_types.len() + 1) {
                self.visit_pat(input);
            }
        }
        self.visit_expr(&closure.body);
        self.variables = variables;
        self.variable_candidates = variable_candidates;
        self.variable_type_arguments = variable_type_arguments;
        self.variable_result_ok_types = variable_result_ok_types;
        self.variable_result_error_types = variable_result_error_types;
        true
    }

    fn visit_state_item_closure_method_call(&mut self, call: &ExprMethodCall) -> bool {
        if call.method != "scan" || call.args.len() != 2 {
            return false;
        }

        let mut args = call.args.iter();
        let Some(initial) = args.next() else {
            return false;
        };
        let Some(closure_expr) = args.next() else {
            return false;
        };
        let Some(state_type) = self.infer_expr_type(initial) else {
            return false;
        };
        let Expr::Closure(closure) = closure_expr else {
            return false;
        };
        let Some(state_pat) = closure.inputs.first() else {
            return false;
        };
        let Some(item_pat) = closure.inputs.iter().nth(1) else {
            return false;
        };
        let Some(item_type) = self
            .expression_type_arguments(&call.receiver)
            .first()
            .cloned()
        else {
            return false;
        };

        self.visit_expr(&call.receiver);
        self.visit_expr(initial);
        let variables = self.variables.clone();
        let variable_candidates = self.variable_candidates.clone();
        let variable_type_arguments = self.variable_type_arguments.clone();
        let variable_result_ok_types = self.variable_result_ok_types.clone();
        let variable_result_error_types = self.variable_result_error_types.clone();
        self.bind_pattern_type(state_pat, &state_type);
        self.bind_pattern_type(item_pat, &item_type);
        self.visit_expr(&closure.body);
        self.variables = variables;
        self.variable_candidates = variable_candidates;
        self.variable_type_arguments = variable_type_arguments;
        self.variable_result_ok_types = variable_result_ok_types;
        self.variable_result_error_types = variable_result_error_types;
        true
    }

    fn visit_two_payload_closure_method_call(&mut self, call: &ExprMethodCall) -> bool {
        let method = call.method.to_string();
        let closure_arg_index = match method.as_str() {
            "select_nth_unstable_by" if call.args.len() == 2 => 1,
            _ if call.args.len() == 1 => 0,
            _ => return false,
        };
        if !matches!(
            method.as_str(),
            "dedup_by"
                | "max_by"
                | "min_by"
                | "reduce"
                | "select_nth_unstable_by"
                | "sort_by"
                | "sort_unstable_by"
        ) {
            return false;
        }
        if !self.resolved_methods_for_call(call).is_empty() {
            return false;
        }
        let Some(item_type) = self
            .expression_type_arguments(&call.receiver)
            .first()
            .cloned()
        else {
            return false;
        };
        let Some(Expr::Closure(closure)) = call.args.iter().nth(closure_arg_index) else {
            return false;
        };

        self.visit_expr(&call.receiver);
        for (index, argument) in call.args.iter().enumerate() {
            if index != closure_arg_index {
                self.visit_expr(argument);
            }
        }
        let variables = self.variables.clone();
        let variable_candidates = self.variable_candidates.clone();
        let variable_type_arguments = self.variable_type_arguments.clone();
        let variable_result_ok_types = self.variable_result_ok_types.clone();
        let variable_result_error_types = self.variable_result_error_types.clone();
        for input in closure.inputs.iter().take(2) {
            self.bind_pattern_type(input, &item_type);
        }
        if closure.inputs.len() > 2 {
            for input in closure.inputs.iter().skip(2) {
                self.visit_pat(input);
            }
        }
        self.visit_expr(&closure.body);
        self.variables = variables;
        self.variable_candidates = variable_candidates;
        self.variable_type_arguments = variable_type_arguments;
        self.variable_result_ok_types = variable_result_ok_types;
        self.variable_result_error_types = variable_result_error_types;
        true
    }

    fn visit_key_value_payload_closure_method_call(&mut self, call: &ExprMethodCall) -> bool {
        if call.method != "retain" || call.args.len() != 1 {
            return false;
        }
        let Some(Expr::Closure(closure)) = call.args.first() else {
            return false;
        };
        if closure.inputs.len() < 2 {
            return false;
        }
        let type_arguments = self.expression_type_arguments(&call.receiver);
        let key_type = self
            .expression_map_key_type(&call.receiver)
            .or_else(|| type_arguments.first().cloned());
        let value_type = self
            .expression_map_value_type(&call.receiver)
            .or_else(|| type_arguments.get(1).cloned());
        let (Some(key_type), Some(value_type)) = (key_type, value_type) else {
            return false;
        };

        self.visit_expr(&call.receiver);
        let variables = self.variables.clone();
        let variable_candidates = self.variable_candidates.clone();
        let variable_type_arguments = self.variable_type_arguments.clone();
        let variable_result_ok_types = self.variable_result_ok_types.clone();
        let variable_result_error_types = self.variable_result_error_types.clone();
        if let Some(key_pat) = closure.inputs.first() {
            self.bind_pattern_type(key_pat, &key_type);
        }
        if let Some(value_pat) = closure.inputs.iter().nth(1) {
            self.bind_pattern_type(value_pat, &value_type);
        }
        for input in closure.inputs.iter().skip(2) {
            self.visit_pat(input);
        }
        self.visit_expr(&closure.body);
        self.variables = variables;
        self.variable_candidates = variable_candidates;
        self.variable_type_arguments = variable_type_arguments;
        self.variable_result_ok_types = variable_result_ok_types;
        self.variable_result_error_types = variable_result_error_types;
        true
    }

    fn visit_single_payload_closure_method_call(&mut self, call: &ExprMethodCall) -> bool {
        let closure_arg_index = match call.method.to_string().as_str() {
            "binary_search_by_key" | "select_nth_unstable_by_key" if call.args.len() == 2 => 1,
            "splitn" | "rsplitn" if call.args.len() == 2 => 1,
            _ if call.args.len() == 1 => 0,
            _ => return false,
        };
        let Some(Expr::Closure(closure)) = call.args.iter().nth(closure_arg_index) else {
            return false;
        };
        if !self.resolved_methods_for_call(call).is_empty() {
            return false;
        }
        let Some(payload_type) = self.single_payload_closure_method_type(call) else {
            return false;
        };
        let payload_type_arguments = self.expression_type_arguments(&call.receiver);
        let Some(payload_pat) = closure.inputs.first() else {
            return false;
        };

        self.visit_expr(&call.receiver);
        for (index, argument) in call.args.iter().enumerate() {
            if index != closure_arg_index {
                self.visit_expr(argument);
            }
        }
        let variables = self.variables.clone();
        let variable_candidates = self.variable_candidates.clone();
        let variable_type_arguments = self.variable_type_arguments.clone();
        let variable_result_ok_types = self.variable_result_ok_types.clone();
        let variable_result_error_types = self.variable_result_error_types.clone();
        self.bind_single_payload_closure_pattern(
            &call.receiver,
            payload_pat,
            &payload_type,
            &payload_type_arguments,
        );
        self.visit_expr(&closure.body);
        self.variables = variables;
        self.variable_candidates = variable_candidates;
        self.variable_type_arguments = variable_type_arguments;
        self.variable_result_ok_types = variable_result_ok_types;
        self.variable_result_error_types = variable_result_error_types;
        true
    }

    fn visit_map_or_else_method_call(&mut self, call: &ExprMethodCall) -> bool {
        if call.method != "map_or_else" || call.args.len() != 2 {
            return false;
        }
        if !self.resolved_methods_for_call(call).is_empty() {
            return false;
        }
        let mut args = call.args.iter();
        let Some(Expr::Closure(default_closure)) = args.next() else {
            return false;
        };
        let Some(Expr::Closure(payload_closure)) = args.next() else {
            return false;
        };
        let type_arguments = self.expression_type_arguments(&call.receiver);
        let Some(payload_type) = self
            .expression_result_ok_type(&call.receiver)
            .or_else(|| type_arguments.first().cloned())
            .or_else(|| self.receiver_type(&call.receiver))
        else {
            return false;
        };
        let error_type = self
            .expression_result_error_type(&call.receiver)
            .or_else(|| type_arguments.get(1).cloned());

        self.visit_expr(&call.receiver);
        if let Some(error_type) = error_type
            .as_ref()
            .filter(|_| !default_closure.inputs.is_empty())
        {
            self.visit_closure_with_input_types(default_closure, std::slice::from_ref(error_type));
        } else {
            self.visit_closure_with_input_types(default_closure, &[]);
        }
        self.visit_payload_closure_with_type(
            &call.receiver,
            payload_closure,
            &payload_type,
            &type_arguments,
        );
        true
    }

    fn visit_map_or_method_call(&mut self, call: &ExprMethodCall) -> bool {
        if call.method != "map_or" || call.args.len() != 2 {
            return false;
        }
        if !self.resolved_methods_for_call(call).is_empty() {
            return false;
        }
        let mut args = call.args.iter();
        let Some(default_expr) = args.next() else {
            return false;
        };
        let Some(Expr::Closure(payload_closure)) = args.next() else {
            return false;
        };
        let type_arguments = self.expression_type_arguments(&call.receiver);
        let Some(payload_type) = self
            .expression_result_ok_type(&call.receiver)
            .or_else(|| type_arguments.first().cloned())
            .or_else(|| self.receiver_type(&call.receiver))
        else {
            return false;
        };

        self.visit_expr(&call.receiver);
        self.visit_expr(default_expr);
        self.visit_payload_closure_with_type(
            &call.receiver,
            payload_closure,
            &payload_type,
            &type_arguments,
        );
        true
    }

    fn visit_payload_closure_with_type(
        &mut self,
        receiver: &Expr,
        closure: &syn::ExprClosure,
        payload_type: &TypeRef,
        payload_type_arguments: &[TypeRef],
    ) {
        let variables = self.variables.clone();
        let variable_candidates = self.variable_candidates.clone();
        let variable_type_arguments = self.variable_type_arguments.clone();
        let variable_result_ok_types = self.variable_result_ok_types.clone();
        let variable_result_error_types = self.variable_result_error_types.clone();
        if let Some(payload_pat) = closure.inputs.first() {
            self.bind_single_payload_closure_pattern(
                receiver,
                payload_pat,
                payload_type,
                payload_type_arguments,
            );
        }
        for input in closure.inputs.iter().skip(1) {
            self.visit_pat(input);
        }
        self.visit_expr(&closure.body);
        self.variables = variables;
        self.variable_candidates = variable_candidates;
        self.variable_type_arguments = variable_type_arguments;
        self.variable_result_ok_types = variable_result_ok_types;
        self.variable_result_error_types = variable_result_error_types;
    }

    fn bind_single_payload_closure_pattern(
        &mut self,
        receiver: &Expr,
        payload_pat: &Pat,
        payload_type: &TypeRef,
        payload_type_arguments: &[TypeRef],
    ) {
        if self.bind_adapter_tuple_payload_pattern(receiver, payload_pat) {
            return;
        }
        if self.bind_tuple_payload_pattern_from_type_arguments(payload_pat, payload_type_arguments)
        {
            return;
        }
        self.bind_single_payload_pattern(payload_pat, payload_type);
        self.bind_pattern_type_arguments(payload_pat, payload_type_arguments);
    }

    fn bind_tuple_payload_pattern_from_type_arguments(
        &mut self,
        payload_pat: &Pat,
        payload_type_arguments: &[TypeRef],
    ) -> bool {
        if !matches!(payload_pat, Pat::Tuple(_)) {
            return false;
        }

        let mut cursor = 0;
        self.bind_pattern_from_type_argument_cursor(
            payload_pat,
            payload_type_arguments,
            &mut cursor,
        )
    }

    fn bind_pattern_from_type_argument_cursor(
        &mut self,
        pattern: &Pat,
        type_arguments: &[TypeRef],
        cursor: &mut usize,
    ) -> bool {
        match pattern {
            Pat::Tuple(tuple) => tuple.elems.iter().all(|elem| {
                self.bind_pattern_from_type_argument_cursor(elem, type_arguments, cursor)
            }),
            Pat::Reference(reference) => {
                self.bind_pattern_from_type_argument_cursor(&reference.pat, type_arguments, cursor)
            }
            Pat::Type(pat_type) => {
                self.bind_pattern_from_type_argument_cursor(&pat_type.pat, type_arguments, cursor)
            }
            _ => {
                let Some(type_ref) = type_arguments.get(*cursor) else {
                    return false;
                };
                self.bind_single_payload_pattern(pattern, type_ref);
                *cursor += 1;
                true
            }
        }
    }

    fn bind_adapter_tuple_payload_pattern(&mut self, receiver: &Expr, payload_pat: &Pat) -> bool {
        let Pat::Tuple(tuple) = payload_pat else {
            return false;
        };
        let Expr::MethodCall(adapter) = receiver else {
            return false;
        };

        match adapter.method.to_string().as_str() {
            "enumerate" => {
                let Some(item_pat) = tuple.elems.get(1) else {
                    return false;
                };
                let Some(item_type) = self
                    .expression_type_arguments(&adapter.receiver)
                    .first()
                    .cloned()
                else {
                    return false;
                };
                self.bind_pattern_type(item_pat, &item_type);
                true
            }
            "zip" => {
                let Some(left_pat) = tuple.elems.first() else {
                    return false;
                };
                let Some(right_pat) = tuple.elems.get(1) else {
                    return false;
                };
                let Some(left_type) = self
                    .expression_type_arguments(&adapter.receiver)
                    .first()
                    .cloned()
                else {
                    return false;
                };
                let Some(right_type) = adapter
                    .args
                    .first()
                    .and_then(|arg| self.expression_type_arguments(arg).first().cloned())
                else {
                    return false;
                };
                self.bind_pattern_type(left_pat, &left_type);
                self.bind_pattern_type(right_pat, &right_type);
                true
            }
            _ => false,
        }
    }

    fn single_payload_closure_method_type(&self, call: &ExprMethodCall) -> Option<TypeRef> {
        let method = call.method.to_string();
        let type_arguments = self.expression_type_arguments(&call.receiver);
        match method.as_str() {
            "and_modify" => self.expression_entry_value_type(&call.receiver),
            "map"
            | "and_then"
            | "filter"
            | "filter_map"
            | "flat_map"
            | "find"
            | "find_map"
            | "for_each"
            | "inspect"
            | "is_some_and"
            | "is_ok_and"
            | "any"
            | "all"
            | "position"
            | "rposition"
            | "partition"
            | "partition_point"
            | "retain"
            | "retain_mut"
            | "skip_while"
            | "sort_by_cached_key"
            | "sort_by_key"
            | "sort_unstable_by_key"
            | "split"
            | "split_inclusive"
            | "splitn"
            | "rsplit"
            | "rsplitn"
            | "max_by_key"
            | "min_by_key"
            | "dedup_by_key"
            | "binary_search_by"
            | "binary_search_by_key"
            | "select_nth_unstable_by_key"
            | "take_while"
            | "map_while"
            | "try_for_each" => self
                .expression_result_ok_type(&call.receiver)
                .or_else(|| type_arguments.first().cloned())
                .or_else(|| self.receiver_type(&call.receiver)),
            "map_err" | "or_else" | "inspect_err" | "is_err_and" | "unwrap_or_else" => self
                .expression_result_error_type(&call.receiver)
                .or_else(|| type_arguments.get(1).cloned())
                .or_else(|| type_arguments.last().cloned())
                .or_else(|| self.receiver_type(&call.receiver)),
            _ => None,
        }
    }

    fn single_payload_closure_error_type(&self, call: &ExprMethodCall) -> Option<TypeRef> {
        let Some(Expr::Closure(closure)) = call.args.first() else {
            return None;
        };
        let first_input = closure.inputs.first()?;
        let (input_name, explicit_type) = binding_name_and_type(first_input)?;
        let payload_type = explicit_type
            .and_then(|ty| self.resolver.resolve_receiver_type(ty))
            .or_else(|| self.single_payload_closure_method_type(call))?;
        self.expression_error_type_with_bound_receiver(&closure.body, &input_name, &payload_type)
    }

    fn fold_closure_error_type(&self, call: &ExprMethodCall) -> Option<TypeRef> {
        if call.args.len() != 2 {
            return None;
        }
        let mut args = call.args.iter();
        let _initial = args.next()?;
        let Some(Expr::Closure(closure)) = args.next() else {
            return None;
        };
        let item_input = closure.inputs.iter().nth(1)?;
        let (item_name, explicit_type) = binding_name_and_type(item_input)?;
        let item_type = explicit_type
            .and_then(|ty| self.resolver.resolve_receiver_type(ty))
            .or_else(|| {
                self.expression_type_arguments(&call.receiver)
                    .first()
                    .cloned()
            })?;
        self.expression_error_type_with_bound_receiver(&closure.body, &item_name, &item_type)
    }

    fn expression_error_type_with_bound_receiver(
        &self,
        expression: &Expr,
        receiver_name: &str,
        receiver_type: &TypeRef,
    ) -> Option<TypeRef> {
        match expression {
            Expr::MethodCall(call) => {
                let Expr::Path(receiver) = call.receiver.as_ref() else {
                    return None;
                };
                if receiver.path.segments.len() != 1
                    || receiver
                        .path
                        .segments
                        .first()
                        .is_none_or(|segment| segment.ident != receiver_name)
                {
                    return None;
                }
                self.resolver
                    .resolve_methods(receiver_type, &call.method.to_string())
                    .iter()
                    .find_map(|callable| self.resolver.return_error_type_from_callable(callable))
            }
            Expr::Try(expr) => self.expression_error_type_with_bound_receiver(
                &expr.expr,
                receiver_name,
                receiver_type,
            ),
            Expr::Block(expr) => final_block_expression(&expr.block).and_then(|tail| {
                self.expression_error_type_with_bound_receiver(tail, receiver_name, receiver_type)
            }),
            Expr::Unsafe(expr) => final_block_expression(&expr.block).and_then(|tail| {
                self.expression_error_type_with_bound_receiver(tail, receiver_name, receiver_type)
            }),
            Expr::Reference(reference) => self.expression_error_type_with_bound_receiver(
                &reference.expr,
                receiver_name,
                receiver_type,
            ),
            Expr::Paren(paren) => self.expression_error_type_with_bound_receiver(
                &paren.expr,
                receiver_name,
                receiver_type,
            ),
            _ => None,
        }
    }
}

struct UnresolvedExternalCallVisitor<'a, 'project> {
    resolver: &'a Resolver<'project>,
    found: bool,
}

impl<'ast> Visit<'ast> for UnresolvedExternalCallVisitor<'_, '_> {
    fn visit_expr_call(&mut self, call: &'ast ExprCall) {
        if let Expr::Path(path) = call.func.as_ref() {
            if path.qself.is_none() && self.resolver.resolve_call_path(&path.path).is_empty() {
                self.found = true;
                return;
            }
        }
        visit::visit_expr_call(self, call);
    }
}

fn expr_contains_collect_call(expression: &Expr) -> bool {
    let mut visitor = CollectMethodVisitor { found: false };
    visitor.visit_expr(expression);
    visitor.found
}

struct CollectMethodVisitor {
    found: bool,
}

impl<'ast> Visit<'ast> for CollectMethodVisitor {
    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        if call.method == "collect" {
            self.found = true;
            return;
        }
        visit::visit_expr_method_call(self, call);
    }
}

impl<'ast> Visit<'ast> for DependencyVisitor<'_> {
    fn visit_block(&mut self, block: &'ast syn::Block) {
        self.push_local_value_scope();
        visit::visit_block(self, block);
        self.pop_local_value_scope();
    }

    fn visit_attribute(&mut self, attribute: &'ast syn::Attribute) {
        self.add_macro_path(attribute.path());
        self.add_item_path(attribute.path());
        if macro_path_ends_with(attribute.path(), "handle_error") {
            if let Ok(path) = syn::parse_str::<Path>("error_support::convert_log_report_error") {
                self.add_call_path(&path);
            }
        }
        if attribute_can_expand_to_code(attribute) {
            for segments in token_path_candidates(&attribute.meta.to_token_stream()) {
                let Ok(path) = syn::parse_str::<Path>(&segments.join("::")) else {
                    continue;
                };
                self.add_call_path(&path);
                self.add_item_path(&path);
                self.add_macro_path(&path);
            }
        }
        for segments in string_literal_path_candidates(&attribute.meta.to_token_stream()) {
            let Ok(path) = syn::parse_str::<Path>(&segments.join("::")) else {
                continue;
            };
            self.add_call_path(&path);
            self.add_item_path(&path);
            self.add_macro_path(&path);
            if attribute_uses_serde_helper_paths(attribute.path()) {
                for path in serde_module_helper_paths(&segments) {
                    self.add_call_path(&path);
                    self.add_item_path(&path);
                }
            }
        }
    }

    fn visit_local(&mut self, local: &'ast Local) {
        if let Some((name, trait_items)) = self.local_binding_trait_bounds(local) {
            self.insert_variable_trait_bounds(name, trait_items);
        }
        if let Some((name, explicit_type)) = binding_name_and_type(&local.pat) {
            if let Some(ty) = explicit_type {
                self.insert_variable_type_data_from_type(name, ty);
            } else if let Some(init) = &local.init {
                self.insert_variable_type_data_from_expr(name, &init.expr);
            }
        }
        for (name, type_ref, candidates) in self.local_destructured_binding_types(local) {
            self.insert_variable_candidates(name, type_ref, candidates);
        }
        if let Some((name, type_ref, candidates)) = self.local_binding_type(local) {
            if binding_name_and_type(&local.pat)
                .and_then(|(_, ty)| ty)
                .is_some()
            {
                if let Some(init) = &local.init {
                    self.add_conversion_impls_to_expected_type(
                        &init.expr, &type_ref, "From", "from", "into",
                    );
                    self.add_conversion_impls_to_expected_type(
                        &init.expr, &type_ref, "TryFrom", "try_from", "try_into",
                    );
                }
            }
            self.add_local_initializer_trait_dependencies(local, &type_ref);
            if local
                .init
                .as_ref()
                .is_some_and(|init| expr_contains_collect_call(&init.expr))
            {
                self.add_trait_impls_for_type_named(&type_ref, "FromIterator");
            }
            self.insert_variable_candidates(name, type_ref, candidates);
        }
        visit::visit_local(self, local);
        self.bind_local_value_names(&local.pat);
    }

    fn visit_expr_call(&mut self, call: &'ast ExprCall) {
        if let Expr::Path(path) = call.func.as_ref() {
            let resolved_callables = if self.expr_path_is_local_value(path) {
                Vec::new()
            } else if path.qself.is_some() {
                self.resolver.resolve_qself_call(path)
            } else {
                self.resolver.resolve_call_path(&path.path)
            };
            for callable in &resolved_callables {
                self.dependencies.callables.insert(callable.clone());
            }
            self.add_call_closure_arg_dependencies(call, &resolved_callables);
            self.add_generic_conversion_call_arg_dependencies(call, &resolved_callables);
            self.add_external_call_arg_trait_impls(call);
            if resolved_callables.is_empty()
                && path.path.segments.len() >= 2
                && self
                    .resolver
                    .resolve_associated_call_type(&path.path)
                    .is_some()
            {
                if let Some(method) = path.path.segments.last() {
                    self.add_unresolved_method_fallback(
                        &method.ident.to_string(),
                        Some(method.ident.span().start().line),
                    );
                }
            }
            if path.qself.is_none() {
                if let Some(first_arg) = call.args.first() {
                    if let Some(receiver) = self.receiver_type(first_arg) {
                        for callable in self.resolver.resolve_trait_call(&path.path, &receiver) {
                            self.dependencies.callables.insert(callable);
                        }
                    }
                }
                if is_conversion_adapter_path(&path.path, "Into", "into") {
                    self.add_conversion_impls_by_trait("From", "from");
                } else if is_conversion_adapter_path(&path.path, "TryInto", "try_into") {
                    self.add_conversion_impls_by_trait("TryFrom", "try_from");
                }
                self.add_conversion_impls_for_associated_call_arg(&path.path, call);
            }
        }
        visit::visit_expr_call(self, call);
    }

    fn visit_expr_match(&mut self, expr_match: &'ast ExprMatch) {
        self.visit_expr(&expr_match.expr);
        let ok_type = self
            .result_ok_type(&expr_match.expr)
            .or_else(|| self.expression_result_ok_type(&expr_match.expr));
        let error_type = self.expression_result_error_type(&expr_match.expr);
        let match_type = self
            .receiver_type(&expr_match.expr)
            .or_else(|| self.infer_expr_type(&expr_match.expr));

        for arm in &expr_match.arms {
            let variables = self.variables.clone();
            let variable_candidates = self.variable_candidates.clone();
            let variable_type_arguments = self.variable_type_arguments.clone();
            let variable_result_ok_types = self.variable_result_ok_types.clone();
            let variable_result_error_types = self.variable_result_error_types.clone();
            if let Some(ok_type) = &ok_type {
                self.add_result_ok_binding_type(&arm.pat, ok_type);
            }
            if let Some(error_type) = &error_type {
                self.add_result_error_binding_type(&arm.pat, error_type);
            }
            if let Some(match_type) = &match_type {
                self.add_pattern_bindings_for_type(&arm.pat, match_type);
            }
            let mut generic_bindings = Vec::new();
            self.collect_pattern_bindings_from_expr(
                &arm.pat,
                &expr_match.expr,
                &mut generic_bindings,
            );
            for (name, type_ref, candidates) in generic_bindings {
                self.insert_variable_candidates(name, type_ref, candidates);
            }
            self.push_local_value_scope();
            self.bind_local_value_names(&arm.pat);
            self.visit_pat(&arm.pat);
            if let Some((_, guard)) = &arm.guard {
                self.visit_expr(guard);
            }
            self.visit_expr(&arm.body);
            self.pop_local_value_scope();
            self.variables = variables;
            self.variable_candidates = variable_candidates;
            self.variable_type_arguments = variable_type_arguments;
            self.variable_result_ok_types = variable_result_ok_types;
            self.variable_result_error_types = variable_result_error_types;
        }
    }

    fn visit_expr_if(&mut self, expr_if: &'ast syn::ExprIf) {
        let outer_variables = self.variables.clone();
        let outer_variable_candidates = self.variable_candidates.clone();
        let outer_variable_type_arguments = self.variable_type_arguments.clone();
        let outer_variable_result_ok_types = self.variable_result_ok_types.clone();
        let outer_variable_result_error_types = self.variable_result_error_types.clone();
        if let Expr::Let(expr_let) = expr_if.cond.as_ref() {
            let type_arguments = self.expression_type_arguments(&expr_let.expr);
            if let [type_ref] = type_arguments.as_slice() {
                self.bind_single_payload_pattern(&expr_let.pat, type_ref);
            }
            if let Some(type_ref) = self
                .receiver_type(&expr_let.expr)
                .or_else(|| self.infer_expr_type(&expr_let.expr))
            {
                self.add_pattern_bindings_for_type(&expr_let.pat, &type_ref);
            }
            let mut generic_bindings = Vec::new();
            self.collect_pattern_bindings_from_expr(
                &expr_let.pat,
                &expr_let.expr,
                &mut generic_bindings,
            );
            for (name, type_ref, candidates) in generic_bindings {
                self.insert_variable_candidates(name, type_ref, candidates);
            }
            self.visit_expr(&expr_let.expr);
            self.push_local_value_scope();
            self.bind_local_value_names(&expr_let.pat);
            self.visit_pat(&expr_let.pat);
            self.visit_block(&expr_if.then_branch);
            self.pop_local_value_scope();
        } else {
            self.visit_expr(&expr_if.cond);
            self.visit_block(&expr_if.then_branch);
        }

        self.variables = outer_variables.clone();
        self.variable_candidates = outer_variable_candidates.clone();
        self.variable_type_arguments = outer_variable_type_arguments.clone();
        self.variable_result_ok_types = outer_variable_result_ok_types.clone();
        self.variable_result_error_types = outer_variable_result_error_types.clone();

        if let Some((_, else_branch)) = &expr_if.else_branch {
            self.visit_expr(else_branch);
        }
        self.variables = outer_variables;
        self.variable_candidates = outer_variable_candidates;
        self.variable_type_arguments = outer_variable_type_arguments;
        self.variable_result_ok_types = outer_variable_result_ok_types;
        self.variable_result_error_types = outer_variable_result_error_types;
    }

    fn visit_expr_while(&mut self, expr_while: &'ast syn::ExprWhile) {
        if let Expr::Let(expr_let) = expr_while.cond.as_ref() {
            let type_arguments = self.expression_type_arguments(&expr_let.expr);
            if let [type_ref] = type_arguments.as_slice() {
                self.bind_single_payload_pattern(&expr_let.pat, type_ref);
            }
            if let Some(type_ref) = self
                .receiver_type(&expr_let.expr)
                .or_else(|| self.infer_expr_type(&expr_let.expr))
            {
                self.add_pattern_bindings_for_type(&expr_let.pat, &type_ref);
            }
            let mut generic_bindings = Vec::new();
            self.collect_pattern_bindings_from_expr(
                &expr_let.pat,
                &expr_let.expr,
                &mut generic_bindings,
            );
            for (name, type_ref, candidates) in generic_bindings {
                self.insert_variable_candidates(name, type_ref, candidates);
            }
            self.visit_expr(&expr_let.expr);
            self.push_local_value_scope();
            self.bind_local_value_names(&expr_let.pat);
            self.visit_pat(&expr_let.pat);
            self.visit_block(&expr_while.body);
            self.pop_local_value_scope();
        } else {
            self.visit_expr(&expr_while.cond);
            self.visit_block(&expr_while.body);
        }
    }

    fn visit_expr_for_loop(&mut self, expr_for_loop: &'ast syn::ExprForLoop) {
        self.visit_expr(&expr_for_loop.expr);
        self.push_local_value_scope();
        let item_types = self.expression_type_arguments(&expr_for_loop.expr);
        if let [type_ref] = item_types.as_slice() {
            self.add_pattern_bindings_for_type(&expr_for_loop.pat, type_ref);
        }
        self.bind_local_value_names(&expr_for_loop.pat);
        self.visit_pat(&expr_for_loop.pat);
        self.visit_block(&expr_for_loop.body);
        self.pop_local_value_scope();
    }

    fn visit_expr_closure(&mut self, closure: &'ast syn::ExprClosure) {
        self.push_local_value_scope();
        for input in &closure.inputs {
            self.bind_local_value_names(input);
        }
        visit::visit_expr_closure(self, closure);
        self.pop_local_value_scope();
    }

    fn visit_expr_macro(&mut self, expr: &'ast ExprMacro) {
        self.add_macro_path(&expr.mac.path);
        visit::visit_expr_macro(self, expr);
    }

    fn visit_item_macro(&mut self, item: &'ast ItemMacro) {
        self.add_macro_token_dependencies(&item.mac.tokens);
        if let Some(ident) = &item.ident {
            let id = ItemId {
                package: self.resolver.package.to_string(),
                module_path: self.resolver.module_path.to_vec(),
                name: ident.to_string(),
                kind: ItemKind::Macro,
            };
            self.dependencies.items.insert(id);
        }
    }

    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        self.add_derive_field_trait_dependencies(&item.attrs, &item.fields);
        self.add_generic_field_type_trait_dependencies(&item.fields);
        self.add_serde_default_field_dependencies(&item.fields);
        visit::visit_item_struct(self, item);
    }

    fn visit_item_enum(&mut self, item: &'ast syn::ItemEnum) {
        let derive_traits = derive_trait_names(&item.attrs);
        for variant in &item.variants {
            self.add_derive_field_trait_dependencies(&variant.attrs, &variant.fields);
            self.add_generic_field_type_trait_dependencies(&variant.fields);
            self.add_serde_default_field_dependencies(&variant.fields);
            for field in variant.fields.iter() {
                for type_ref in self.resolver.type_refs_in_type(&field.ty) {
                    for trait_name in &derive_traits {
                        self.add_trait_impls_for_type_named(&type_ref, trait_name);
                        if trait_name == "Error" {
                            self.add_trait_impls_for_type_named(&type_ref, "Display");
                        }
                    }
                }
            }
        }
        visit::visit_item_enum(self, item);
    }

    fn visit_macro(&mut self, mac: &'ast Macro) {
        self.add_macro_path(&mac.path);
        self.add_macro_definition_receiver_dependencies(mac);
        self.add_format_macro_trait_dependencies(mac);
        self.add_expression_macro_dependencies(mac);
        self.add_matches_macro_dependencies(mac);
        self.add_rusqlite_param_macro_trait_dependencies(mac);
        self.add_macro_token_dependencies(&mac.tokens);
        visit::visit_macro(self, mac);
    }

    fn visit_expr_try(&mut self, expr: &'ast syn::ExprTry) {
        if let Some(source_error) = self.expression_result_error_type(&expr.expr) {
            for target_error in self.expected_error_types.clone() {
                if source_error == target_error {
                    continue;
                }
                for callable in self.resolver.resolve_conversion_impls_from_to(
                    &source_error,
                    &target_error,
                    "From",
                    "from",
                ) {
                    self.dependencies.callables.insert(callable);
                }
            }
        } else {
            for type_ref in self.expected_error_types.clone() {
                self.add_trait_impls_for_type_named(&type_ref, "From");
            }
        }
        visit::visit_expr_try(self, expr);
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        if call.method == "or_default" {
            if let Some(value_type) = self.expression_entry_value_type(&call.receiver) {
                self.add_trait_impls_for_type_named(&value_type, "Default");
            }
        }
        if self.visit_fold_method_call(call) {
            return;
        }
        if self.visit_state_item_closure_method_call(call) {
            return;
        }
        if self.visit_two_payload_closure_method_call(call) {
            return;
        }
        if self.visit_key_value_payload_closure_method_call(call) {
            return;
        }
        if self.visit_map_or_method_call(call) {
            return;
        }
        if self.visit_map_or_else_method_call(call) {
            return;
        }
        if self.visit_single_payload_closure_method_call(call) {
            return;
        }
        self.add_parse_turbofish_trait_dependencies(call);
        self.add_method_turbofish_trait_dependencies(call);
        let receiver_candidates = self.receiver_type_candidates(&call.receiver);
        if let Some(receiver) = self.receiver_type(&call.receiver) {
            let method = call.method.to_string();
            let mut resolved_methods = Vec::new();
            for candidate in &receiver_candidates {
                resolved_methods.extend(self.resolver.resolve_methods(candidate, &method));
            }
            resolved_methods.extend(self.resolver.resolve_methods(&receiver, &method));
            resolved_methods.sort();
            resolved_methods.dedup();
            let has_resolved_method = !resolved_methods.is_empty();
            for callable in &resolved_methods {
                self.add_method_dependency(callable);
            }
            let has_trait_bound_method =
                self.add_trait_bound_method_dependencies(&call.receiver, &method);
            let mut has_default_trait_method = false;
            let mut default_trait_receivers = receiver_candidates.clone();
            default_trait_receivers.push(receiver.clone());
            default_trait_receivers.sort();
            default_trait_receivers.dedup();
            for receiver in &default_trait_receivers {
                has_default_trait_method |=
                    self.add_default_trait_method_dependencies(receiver, &method);
            }
            if has_resolved_method {
                self.add_peer_trait_impls_for_resolved_methods(&resolved_methods);
                self.add_closure_arg_dependencies(call, &resolved_methods);
            }
            if !has_resolved_method && !has_trait_bound_method && !has_default_trait_method {
                self.add_unresolved_method_candidates(
                    &method,
                    &receiver_candidates,
                    Some(call.method.span().start().line),
                );
                self.add_trait_impls_for_type_named(&receiver, "Deref");
                self.add_trait_impls_for_type_named(&receiver, "DerefMut");
                self.add_extension_trait_dependencies_for_method(&receiver, &method);
                self.add_deref_target_method_dependencies(&receiver, &method);
                for candidate in &receiver_candidates {
                    self.add_trait_impls_for_type_named(candidate, "Deref");
                    self.add_trait_impls_for_type_named(candidate, "DerefMut");
                    self.add_extension_trait_dependencies_for_method(candidate, &method);
                    self.add_deref_target_method_dependencies(candidate, &method);
                }
                self.add_external_method_arg_trait_impls(call);
            }
            if call.method == "into" {
                for callable in self
                    .resolver
                    .resolve_conversion_impls(&receiver, "From", "from")
                {
                    self.dependencies.callables.insert(callable);
                }
            } else if call.method == "try_into" {
                for callable in self
                    .resolver
                    .resolve_conversion_impls(&receiver, "TryFrom", "try_from")
                {
                    self.dependencies.callables.insert(callable);
                }
            } else if call.method == "to_string" {
                self.add_trait_impls_for_type_named(&receiver, "Display");
            } else if call.method == "parse" {
                self.add_parse_method_trait_dependencies();
            } else if call.method == "collect" {
                self.add_collect_method_trait_dependencies();
            }
        } else if call.method == "into" {
            if self.receiver_has_generic_conversion_bound(&call.receiver, "From", "from") {
                visit::visit_expr_method_call(self, call);
                return;
            }
            self.add_conversion_impls_by_trait("From", "from");
        } else if call.method == "try_into" {
            if self.receiver_has_generic_conversion_bound(&call.receiver, "TryFrom", "try_from") {
                visit::visit_expr_method_call(self, call);
                return;
            }
            self.add_conversion_impls_by_trait("TryFrom", "try_from");
        } else if call.method == "parse" {
            self.add_parse_method_trait_dependencies();
        } else if call.method == "collect" {
            self.add_collect_method_trait_dependencies();
        } else {
            let method = call.method.to_string();
            if !self.add_trait_bound_method_dependencies(&call.receiver, &method) {
                self.add_unresolved_method_candidates(
                    &method,
                    &receiver_candidates,
                    Some(call.method.span().start().line),
                );
            }
            self.add_external_method_arg_trait_impls(call);
        }
        visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_index(&mut self, index: &'ast ExprIndex) {
        if let Some(receiver) = self.receiver_type(&index.expr) {
            self.add_trait_impls_for_type_named(&receiver, "Index");
        }
        for receiver in self.receiver_type_candidates(&index.expr) {
            self.add_trait_impls_for_type_named(&receiver, "Index");
        }
        visit::visit_expr_index(self, index);
    }

    fn visit_expr_struct(&mut self, expr: &'ast ExprStruct) {
        self.add_item_path(&expr.path);
        if let Some(struct_type) = self.resolver.resolve_type_path(&expr.path) {
            for field in &expr.fields {
                if let Some(target) = self.resolver.field_type(&struct_type, &field.member) {
                    self.add_conversion_impls_to_expected_type(
                        &field.expr,
                        &target,
                        "From",
                        "from",
                        "into",
                    );
                    self.add_conversion_impls_to_expected_type(
                        &field.expr,
                        &target,
                        "TryFrom",
                        "try_from",
                        "try_into",
                    );
                }
            }
        }
        visit::visit_expr_struct(self, expr);
    }

    fn visit_expr_path(&mut self, path: &'ast ExprPath) {
        if self.expr_path_is_local_value(path) {
            visit::visit_expr_path(self, path);
            return;
        }
        self.add_expr_path_call(path);
        if path.qself.is_none() {
            if is_conversion_adapter_path(&path.path, "Into", "into") {
                self.add_conversion_impls_by_trait("From", "from");
            } else if is_conversion_adapter_path(&path.path, "TryInto", "try_into") {
                self.add_conversion_impls_by_trait("TryFrom", "try_from");
            }
        }
        self.add_item_path(&path.path);
        visit::visit_expr_path(self, path);
    }

    fn visit_type_path(&mut self, ty: &'ast TypePath) {
        self.add_item_path(&ty.path);
        visit::visit_type_path(self, ty);
    }

    fn visit_trait_bound(&mut self, bound: &'ast syn::TraitBound) {
        self.add_item_path(&bound.path);
        visit::visit_trait_bound(self, bound);
    }

    fn visit_pat_tuple_struct(&mut self, pat: &'ast PatTupleStruct) {
        self.add_item_path(&pat.path);
        visit::visit_pat_tuple_struct(self, pat);
    }

    fn visit_pat(&mut self, pat: &'ast Pat) {
        match pat {
            Pat::Path(path) => self.add_item_path(&path.path),
            Pat::Ident(ident) => {
                if let Some(item) = self
                    .resolver
                    .resolve_pattern_ident_item(&ident.ident.to_string())
                {
                    self.dependencies.items.insert(item);
                }
            }
            _ => {}
        }
        visit::visit_pat(self, pat);
    }
}

impl DependencyVisitor<'_> {
    fn add_macro_path(&mut self, path: &Path) {
        if let Some(item) = self.resolver.resolve_macro_path(path) {
            self.dependencies.items.insert(item);
        }
    }

    fn add_macro_definition_receiver_dependencies(&mut self, mac: &Macro) {
        let Some(item) = self.resolver.resolve_macro_path(&mac.path) else {
            return;
        };
        let Some(record) = self.resolver.project.items.get(&item) else {
            return;
        };
        let Item::Macro(item_macro) = &record.item else {
            return;
        };
        let receiver_methods = macro_metavariable_method_names(&item_macro.mac.tokens);
        if receiver_methods.is_empty() {
            return;
        }

        let mut receiver_types = BTreeSet::new();
        for segments in token_path_candidates(&mac.tokens) {
            if segments.len() == 1 && self.local_value_binding_visible(&segments[0]) {
                receiver_types.extend(self.local_value_type_candidates(&segments[0]));
            }
        }

        for type_ref in receiver_types {
            for method in receiver_methods.values().flatten() {
                for callable in self.resolver.resolve_methods(&type_ref, method) {
                    self.add_method_dependency(&callable);
                }
            }
        }

        let metavariables = macro_metavariable_declaration_order(&item_macro.mac.tokens);
        if metavariables.is_empty() {
            return;
        }
        let local_method_flows = macro_local_method_flows(&item_macro.mac.tokens);
        let parser = syn::punctuated::Punctuated::<Expr, syn::Token![,]>::parse_terminated;
        let Ok(arguments) = parser.parse2(mac.tokens.clone()) else {
            return;
        };

        for (metavariable, argument) in metavariables.iter().zip(arguments.iter()) {
            let Some(methods) = receiver_methods.get(metavariable) else {
                continue;
            };
            let mut argument_types = self.receiver_type_candidates(argument);
            if let Some(type_ref) = self.receiver_type(argument) {
                argument_types.push(type_ref);
            }
            argument_types.sort();
            argument_types.dedup();

            for type_ref in &argument_types {
                for method in methods {
                    for callable in self.resolver.resolve_methods(&type_ref, method) {
                        self.add_method_dependency(&callable);
                    }
                }
            }

            for flow in local_method_flows
                .iter()
                .filter(|flow| &flow.metavariable == metavariable)
            {
                let mut return_types = Vec::new();
                for type_ref in &argument_types {
                    for callable in self.resolver.resolve_methods(type_ref, &flow.source_method) {
                        self.add_method_dependency(&callable);
                        if let Some(return_type) =
                            self.resolver.return_type_from_callable(&callable)
                        {
                            return_types.push(return_type);
                        }
                    }
                }
                return_types.sort();
                return_types.dedup();

                for return_type in &return_types {
                    for method in &flow.local_methods {
                        for callable in self.resolver.resolve_methods(return_type, method) {
                            self.add_method_dependency(&callable);
                        }
                    }
                }
            }
        }
    }

    fn add_macro_token_dependencies(&mut self, tokens: &TokenStream) {
        let mut referenced_types = BTreeSet::new();
        let mut candidate_method_names = BTreeSet::new();
        let self_method_names = macro_self_method_names(tokens);
        for segments in token_path_candidates(tokens) {
            let single_segment_local =
                segments.len() == 1 && self.local_value_binding_visible(&segments[0]);
            if single_segment_local {
                referenced_types.extend(self.local_value_type_candidates(&segments[0]));
                continue;
            }
            if segments.len() == 1 {
                candidate_method_names.insert(segments[0].clone());
            }
            let Ok(path) = syn::parse_str::<Path>(&segments.join("::")) else {
                continue;
            };
            for callable in self.resolver.resolve_call_path(&path) {
                if let Some(return_type) = self.resolver.return_type_from_callable(&callable) {
                    referenced_types.insert(return_type);
                }
                self.dependencies.callables.insert(callable);
            }
            self.add_item_path(&path);
            self.add_macro_path(&path);
            if let Some(type_ref) = self.resolver.resolve_type_path(&path) {
                referenced_types.insert(type_ref);
            }
        }

        if candidate_method_names.contains("eq_ignore_ascii_case") {
            for type_ref in &referenced_types {
                self.add_trait_impls_for_type_named(type_ref, "Deref");
            }
        }

        if let Some(self_type) = &self.resolver.self_type {
            for method in &self_method_names {
                for callable in self.resolver.resolve_methods(self_type, method) {
                    self.dependencies.callables.insert(callable);
                }
            }
        }

        for type_ref in referenced_types {
            for method in &candidate_method_names {
                for callable in self.resolver.resolve_methods(&type_ref, method) {
                    self.dependencies.callables.insert(callable);
                }
            }
        }
    }
}

fn macro_self_method_names(tokens: &TokenStream) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    collect_macro_self_method_names(tokens, &mut names);
    names
}

fn collect_macro_self_method_names(tokens: &TokenStream, names: &mut BTreeSet<String>) {
    let tokens = tokens.clone().into_iter().collect::<Vec<_>>();
    for token in &tokens {
        if let TokenTree::Group(group) = token {
            collect_macro_self_method_names(&group.stream(), names);
        }
    }
    for window in tokens.windows(3) {
        let [TokenTree::Ident(receiver), TokenTree::Punct(dot), TokenTree::Ident(method)] = window
        else {
            continue;
        };
        if receiver == "self" && dot.as_char() == '.' {
            names.insert(method.to_string());
        }
    }
}

fn macro_metavariable_method_names(tokens: &TokenStream) -> BTreeMap<String, BTreeSet<String>> {
    let mut names = BTreeMap::new();
    collect_macro_metavariable_method_names(tokens, &mut names);
    names
}

fn collect_macro_metavariable_method_names(
    tokens: &TokenStream,
    names: &mut BTreeMap<String, BTreeSet<String>>,
) {
    let tokens = tokens.clone().into_iter().collect::<Vec<_>>();
    for token in &tokens {
        if let TokenTree::Group(group) = token {
            collect_macro_metavariable_method_names(&group.stream(), names);
        }
    }

    for window in tokens.windows(4) {
        let [TokenTree::Punct(dollar), TokenTree::Ident(receiver), TokenTree::Punct(dot), TokenTree::Ident(method)] =
            window
        else {
            continue;
        };
        if dollar.as_char() == '$' && dot.as_char() == '.' {
            names
                .entry(receiver.to_string())
                .or_default()
                .insert(method.to_string());
        }
    }
}

fn macro_metavariable_declaration_order(tokens: &TokenStream) -> Vec<String> {
    let mut order = Vec::new();
    collect_macro_metavariable_declaration_order(tokens, &mut order);
    order
}

fn collect_macro_metavariable_declaration_order(tokens: &TokenStream, order: &mut Vec<String>) {
    let tokens = tokens.clone().into_iter().collect::<Vec<_>>();
    for token in &tokens {
        if let TokenTree::Group(group) = token {
            collect_macro_metavariable_declaration_order(&group.stream(), order);
        }
    }

    for window in tokens.windows(3) {
        let [TokenTree::Punct(dollar), TokenTree::Ident(name), TokenTree::Punct(colon)] = window
        else {
            continue;
        };
        if dollar.as_char() == '$' && colon.as_char() == ':' {
            let name = name.to_string();
            if !order.contains(&name) {
                order.push(name);
            }
        }
    }
}

#[derive(Debug)]
struct MacroLocalMethodFlow {
    metavariable: String,
    source_method: String,
    local_methods: BTreeSet<String>,
}

fn macro_local_method_flows(tokens: &TokenStream) -> Vec<MacroLocalMethodFlow> {
    let mut flows = Vec::new();
    collect_macro_local_method_flows(tokens, &mut flows);
    flows
}

fn collect_macro_local_method_flows(tokens: &TokenStream, flows: &mut Vec<MacroLocalMethodFlow>) {
    let tokens = tokens.clone().into_iter().collect::<Vec<_>>();
    for token in &tokens {
        if let TokenTree::Group(group) = token {
            collect_macro_local_method_flows(&group.stream(), flows);
        }
    }

    let mut local_sources = BTreeMap::<String, (String, String)>::new();
    for window in tokens.windows(7) {
        let [TokenTree::Ident(let_ident), TokenTree::Ident(local), TokenTree::Punct(eq), TokenTree::Punct(dollar), TokenTree::Ident(metavariable), TokenTree::Punct(dot), TokenTree::Ident(source_method)] =
            window
        else {
            continue;
        };
        if let_ident == "let"
            && eq.as_char() == '='
            && dollar.as_char() == '$'
            && dot.as_char() == '.'
        {
            local_sources.insert(
                local.to_string(),
                (metavariable.to_string(), source_method.to_string()),
            );
        }
    }

    let mut local_methods = BTreeMap::<String, BTreeSet<String>>::new();
    for window in tokens.windows(3) {
        let [TokenTree::Ident(local), TokenTree::Punct(dot), TokenTree::Ident(method)] = window
        else {
            continue;
        };
        if dot.as_char() == '.' && local_sources.contains_key(&local.to_string()) {
            local_methods
                .entry(local.to_string())
                .or_default()
                .insert(method.to_string());
        }
    }

    for (local, methods) in local_methods {
        let Some((metavariable, source_method)) = local_sources.remove(&local) else {
            continue;
        };
        flows.push(MacroLocalMethodFlow {
            metavariable,
            source_method,
            local_methods: methods,
        });
    }
}

fn split_macro_tokens_at_first_top_level_comma(
    tokens: &TokenStream,
) -> Option<(TokenStream, TokenStream)> {
    let mut before = Vec::new();
    let mut after = Vec::new();
    let mut found = false;
    for token in tokens.clone() {
        if !found && matches!(&token, TokenTree::Punct(punct) if punct.as_char() == ',') {
            found = true;
            continue;
        }
        if found {
            after.push(token);
        } else {
            before.push(token);
        }
    }
    found.then(|| (before.into_iter().collect(), after.into_iter().collect()))
}

fn matches_macro_pattern_bindings(
    tokens: &TokenStream,
    some_type: Option<&TypeRef>,
    ok_type: Option<&TypeRef>,
    error_type: Option<&TypeRef>,
) -> Vec<(String, TypeRef)> {
    let mut bindings = Vec::new();
    collect_matches_macro_pattern_bindings(tokens, some_type, ok_type, error_type, &mut bindings);
    bindings.sort();
    bindings.dedup();
    bindings
}

fn collect_matches_macro_pattern_bindings(
    tokens: &TokenStream,
    some_type: Option<&TypeRef>,
    ok_type: Option<&TypeRef>,
    error_type: Option<&TypeRef>,
    bindings: &mut Vec<(String, TypeRef)>,
) {
    let token_trees = tokens.clone().into_iter().collect::<Vec<_>>();
    for token in &token_trees {
        if let TokenTree::Group(group) = token {
            collect_matches_macro_pattern_bindings(
                &group.stream(),
                some_type,
                ok_type,
                error_type,
                bindings,
            );
        }
    }

    for window in token_trees.windows(2) {
        let [TokenTree::Ident(variant), TokenTree::Group(group)] = window else {
            continue;
        };
        if group.delimiter() != Delimiter::Parenthesis {
            continue;
        }
        let type_ref = match variant.to_string().as_str() {
            "Some" => some_type,
            "Ok" => ok_type,
            "Err" => error_type,
            _ => None,
        };
        let Some(type_ref) = type_ref else {
            continue;
        };
        let mut names = BTreeSet::new();
        collect_macro_pattern_binding_idents(&group.stream(), &mut names);
        bindings.extend(names.into_iter().map(|name| (name, type_ref.clone())));
    }
}

fn collect_macro_pattern_binding_idents(tokens: &TokenStream, names: &mut BTreeSet<String>) {
    for token in tokens.clone() {
        match token {
            TokenTree::Ident(ident) => {
                let ident = ident.to_string();
                if macro_pattern_binding_ident(&ident) {
                    names.insert(ident);
                }
            }
            TokenTree::Group(group) => collect_macro_pattern_binding_idents(&group.stream(), names),
            TokenTree::Punct(_) | TokenTree::Literal(_) => {}
        }
    }
}

fn macro_pattern_binding_ident(ident: &str) -> bool {
    ident != "_"
        && ident
            .chars()
            .next()
            .is_some_and(|ch| ch == '_' || ch.is_ascii_lowercase())
        && !matches!(
            ident,
            "as" | "box"
                | "false"
                | "if"
                | "in"
                | "let"
                | "match"
                | "mut"
                | "ref"
                | "self"
                | "true"
        )
}

fn generic_tuple_variant_field_types<'a>(
    pattern: &'a PatTupleStruct,
    ty: &'a Path,
) -> Vec<(&'a Pat, &'a Type)> {
    let Some(variant) = pattern
        .path
        .segments
        .last()
        .map(|segment| segment.ident.to_string())
    else {
        return Vec::new();
    };
    let Some(container) = ty.segments.last() else {
        return Vec::new();
    };
    let PathArguments::AngleBracketed(arguments) = &container.arguments else {
        return Vec::new();
    };
    match (container.ident.to_string().as_str(), variant.as_str()) {
        ("Option", "Some") => pattern
            .elems
            .first()
            .zip(generic_type_argument(arguments, 0))
            .into_iter()
            .collect(),
        ("Result", "Ok") => pattern
            .elems
            .first()
            .zip(generic_type_argument(arguments, 0))
            .into_iter()
            .collect(),
        ("Result", "Err") => pattern
            .elems
            .first()
            .zip(generic_type_argument(arguments, 1))
            .into_iter()
            .collect(),
        _ => Vec::new(),
    }
}

fn generic_type_argument<'a>(
    arguments: &'a syn::AngleBracketedGenericArguments,
    target_index: usize,
) -> Option<&'a Type> {
    arguments
        .args
        .iter()
        .filter_map(|argument| {
            let GenericArgument::Type(ty) = argument else {
                return None;
            };
            Some(ty)
        })
        .nth(target_index)
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct TypeRef {
    package: String,
    type_path: Vec<String>,
}

fn dedup_type_refs_preserve_order(type_refs: &mut Vec<TypeRef>) {
    let mut seen = BTreeSet::new();
    type_refs.retain(|type_ref| seen.insert(type_ref.clone()));
}

#[derive(Clone, Debug)]
struct GenericConversionBound {
    impl_trait_name: String,
    method_name: String,
    target: TypeRef,
    error: Option<TypeRef>,
}

#[derive(Clone)]
struct Resolver<'a> {
    project: &'a Project,
    package: &'a str,
    module_path: &'a [String],
    aliases: &'a HashMap<String, Vec<String>>,
    self_type: Option<TypeRef>,
}

impl Resolver<'_> {
    fn resolve_call_path(&self, path: &Path) -> Vec<CallableId> {
        let mut callables = Vec::new();
        if let Some(callable) = self.resolve_free_function(path) {
            callables.push(callable);
        }
        if let Some(callable) = self.resolve_associated_method(path) {
            callables.push(callable);
        }
        callables
    }

    fn resolve_free_function(&self, path: &Path) -> Option<CallableId> {
        let segments = self.apply_alias(path_segments(path));
        self.resolve_free_function_segments(&segments)
    }

    fn resolve_free_function_segments(&self, segments: &[String]) -> Option<CallableId> {
        if segments.is_empty() {
            return None;
        }

        let name = segments.last()?.clone();
        let (package, module_path) = self.resolve_value_prefix(&segments[..segments.len() - 1])?;
        let mut path = module_path;
        path.push(name);
        self.find_function(&package, &path, &mut BTreeSet::new())
    }

    fn resolve_associated_method(&self, path: &Path) -> Option<CallableId> {
        let segments = self.apply_alias(path_segments(path));
        if segments.len() < 2 {
            return None;
        }

        let method = segments.last()?.clone();
        let receiver_segments = &segments[..segments.len() - 1];
        let receiver = if receiver_segments.len() == 1 && receiver_segments[0] == "Self" {
            self.self_type.clone()?
        } else {
            self.resolve_type_segments(receiver_segments)?
        };

        self.resolve_methods(&receiver, &method).into_iter().next()
    }

    fn resolve_qself_call(&self, path: &ExprPath) -> Vec<CallableId> {
        let Some(qself) = &path.qself else {
            return Vec::new();
        };
        let Some(method) = path
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
        else {
            return Vec::new();
        };
        let Some(receiver) = self.resolve_type(&qself.ty) else {
            return Vec::new();
        };

        if qself.position == 0 {
            return self.resolve_methods(&receiver, &method);
        }

        let trait_segments = path
            .path
            .segments
            .iter()
            .take(qself.position)
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>();
        let Some(trait_item) = self.resolve_item_segments(&self.apply_alias(trait_segments)) else {
            return Vec::new();
        };
        if trait_item.kind != ItemKind::Trait {
            return Vec::new();
        }

        self.resolve_trait_item_method(&trait_item, &receiver, &method)
    }

    fn resolve_methods(&self, receiver: &TypeRef, method: &str) -> Vec<CallableId> {
        let mut matches = Vec::new();
        for candidate in self.type_ref_candidates(receiver) {
            if let Some(methods) = self.project.methods_by_receiver.get(&(
                candidate.package.clone(),
                candidate.type_path.clone(),
                method.to_string(),
            )) {
                matches.extend(methods.iter().cloned());
            }
        }

        matches.sort();
        matches.dedup();

        matches
    }

    fn resolve_trait_call(&self, path: &Path, receiver: &TypeRef) -> Vec<CallableId> {
        let segments = self.apply_alias(path_segments(path));
        if segments.len() < 2 {
            return Vec::new();
        }

        let method = segments.last().expect("segments length checked");
        let Some(trait_item) = self.resolve_item_segments(&segments[..segments.len() - 1]) else {
            return Vec::new();
        };
        if trait_item.kind != ItemKind::Trait {
            return Vec::new();
        }

        self.resolve_trait_item_method(&trait_item, receiver, method)
    }

    fn resolve_trait_bound_path(&self, path: &Path) -> Option<ItemId> {
        self.resolve_item_path(path)
            .filter(|item| item.kind == ItemKind::Trait)
            .or_else(|| {
                let segments = self.apply_alias(path_segments(path));
                self.resolve_trait_item(&segments)
            })
    }

    fn conversion_bound_from_trait_path(&self, path: &Path) -> Option<GenericConversionBound> {
        let segment = path.segments.last()?;
        let (impl_trait_name, method_name) = match segment.ident.to_string().as_str() {
            "Into" => ("From", "from"),
            "TryInto" => ("TryFrom", "try_from"),
            _ => return None,
        };
        let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
            return None;
        };
        let target = arguments.args.iter().find_map(|argument| {
            let GenericArgument::Type(ty) = argument else {
                return None;
            };
            self.resolve_receiver_type(ty)
        })?;
        let error = arguments.args.iter().find_map(|argument| {
            let GenericArgument::AssocType(associated_type) = argument else {
                return None;
            };
            (associated_type.ident == "Error")
                .then(|| self.resolve_receiver_type(&associated_type.ty))
                .flatten()
        });
        Some(GenericConversionBound {
            impl_trait_name: impl_trait_name.to_string(),
            method_name: method_name.to_string(),
            target,
            error,
        })
    }

    fn generic_conversion_bounds_for_callable(
        &self,
        callable: &CallableId,
        generic_name: &str,
    ) -> Vec<GenericConversionBound> {
        match callable {
            CallableId::Free { .. } => {
                let Some(record) = self.project.functions.get(callable) else {
                    return Vec::new();
                };
                let resolver = Resolver {
                    project: self.project,
                    package: &record.package,
                    module_path: &record.module_path,
                    aliases: &record.aliases,
                    self_type: None,
                };
                resolver.generic_conversion_bounds_from_generics(
                    &record.item.sig.generics,
                    generic_name,
                )
            }
            CallableId::Method {
                package, type_path, ..
            } => {
                let Some(record) = self.project.methods.get(callable) else {
                    return Vec::new();
                };
                let resolver = Resolver {
                    project: self.project,
                    package,
                    module_path: &record.module_path,
                    aliases: &record.aliases,
                    self_type: Some(TypeRef {
                        package: package.clone(),
                        type_path: type_path.clone(),
                    }),
                };
                let mut bounds = resolver
                    .generic_conversion_bounds_from_generics(&record.impl_generics, generic_name);
                bounds.extend(resolver.generic_conversion_bounds_from_generics(
                    &record.item.sig.generics,
                    generic_name,
                ));
                bounds
            }
        }
    }

    fn generic_conversion_bounds_from_generics(
        &self,
        generics: &syn::Generics,
        generic_name: &str,
    ) -> Vec<GenericConversionBound> {
        let mut bounds = Vec::new();
        for parameter in &generics.params {
            let syn::GenericParam::Type(type_parameter) = parameter else {
                continue;
            };
            if type_parameter.ident != generic_name {
                continue;
            }
            bounds.extend(type_parameter.bounds.iter().filter_map(|bound| {
                let syn::TypeParamBound::Trait(trait_bound) = bound else {
                    return None;
                };
                self.conversion_bound_from_trait_path(&trait_bound.path)
            }));
        }
        if let Some(where_clause) = &generics.where_clause {
            for predicate in &where_clause.predicates {
                let syn::WherePredicate::Type(predicate) = predicate else {
                    continue;
                };
                if generic_parameter_name_from_type(&predicate.bounded_ty).as_deref()
                    != Some(generic_name)
                {
                    continue;
                }
                bounds.extend(predicate.bounds.iter().filter_map(|bound| {
                    let syn::TypeParamBound::Trait(trait_bound) = bound else {
                        return None;
                    };
                    self.conversion_bound_from_trait_path(&trait_bound.path)
                }));
            }
        }
        bounds
    }

    fn resolve_trait_item_method(
        &self,
        trait_item: &ItemId,
        receiver: &TypeRef,
        method: &str,
    ) -> Vec<CallableId> {
        self.project
            .methods
            .keys()
            .filter_map(|id| {
                let CallableId::Method {
                    package,
                    type_path,
                    trait_path: Some(trait_path),
                    method: candidate_method,
                    ..
                } = id
                else {
                    return None;
                };

                (package == &receiver.package
                    && type_path == &receiver.type_path
                    && candidate_method == method
                    && trait_path == &path_from_item(trait_item))
                    .then(|| id.clone())
            })
            .collect()
    }

    fn trait_method_return_type(&self, trait_item: &ItemId, method_name: &str) -> Option<TypeRef> {
        let record = self.project.items.get(trait_item)?;
        let Item::Trait(item_trait) = &record.item else {
            return None;
        };
        let method = item_trait.items.iter().find_map(|trait_item| {
            let syn::TraitItem::Fn(function) = trait_item else {
                return None;
            };
            (function.sig.ident == method_name).then_some(function)
        })?;
        let resolver = Resolver {
            project: self.project,
            package: &record.package,
            module_path: &record.module_path,
            aliases: &record.aliases,
            self_type: None,
        };
        resolver.type_from_return_type(&method.sig.output)
    }

    fn trait_method_return_self_associated_name(
        &self,
        trait_item: &ItemId,
        method_name: &str,
    ) -> Option<String> {
        let record = self.project.items.get(trait_item)?;
        let Item::Trait(item_trait) = &record.item else {
            return None;
        };
        let method = item_trait.items.iter().find_map(|trait_item| {
            let syn::TraitItem::Fn(function) = trait_item else {
                return None;
            };
            (function.sig.ident == method_name).then_some(function)
        })?;
        return_type_self_associated_name(&method.sig.output)
    }

    fn type_from_associated_call(&self, path: &Path) -> Option<TypeRef> {
        if let Some(callable) = self.resolve_associated_method(path) {
            return match callable {
                CallableId::Method {
                    package, type_path, ..
                } => Some(TypeRef { package, type_path }),
                CallableId::Free { .. } => None,
            };
        }

        let segments = self.apply_alias(path_segments(path));
        if segments.len() < 2 {
            return None;
        }
        self.resolve_type_segments(&segments[..segments.len() - 1])
    }

    fn type_from_expr_path_call(&self, path: &ExprPath) -> Option<TypeRef> {
        if let Some(qself) = &path.qself {
            return self.resolve_type(&qself.ty);
        }
        self.type_from_associated_call(&path.path).or_else(|| {
            self.resolve_free_function(&path.path)
                .and_then(|callable| self.return_type_from_callable(&callable))
        })
    }

    fn type_from_value_path(&self, path: &Path) -> Option<TypeRef> {
        let item = self.resolve_item_path(path)?;
        let record = self.project.items.get(&item)?;
        let resolver = Resolver {
            project: self.project,
            package: &record.package,
            module_path: &record.module_path,
            aliases: &record.aliases,
            self_type: None,
        };
        match &record.item {
            Item::Const(item_const) => resolver.resolve_receiver_type(&item_const.ty),
            Item::Static(item_static) => resolver.resolve_receiver_type(&item_static.ty),
            _ => None,
        }
    }

    fn type_arguments_from_value_path(&self, path: &Path) -> Vec<TypeRef> {
        let Some(item) = self.resolve_item_path(path) else {
            return Vec::new();
        };
        let Some(record) = self.project.items.get(&item) else {
            return Vec::new();
        };
        let resolver = Resolver {
            project: self.project,
            package: &record.package,
            module_path: &record.module_path,
            aliases: &record.aliases,
            self_type: None,
        };
        match &record.item {
            Item::Const(item_const) => resolver.type_argument_refs_in_type(&item_const.ty),
            Item::Static(item_static) => resolver.type_argument_refs_in_type(&item_static.ty),
            _ => Vec::new(),
        }
    }

    fn return_type_from_callable(&self, callable: &CallableId) -> Option<TypeRef> {
        match callable {
            CallableId::Free { .. } => {
                let record = self.project.functions.get(callable)?;
                let resolver = Resolver {
                    project: self.project,
                    package: &record.package,
                    module_path: &record.module_path,
                    aliases: &record.aliases,
                    self_type: None,
                };
                resolver.type_from_return_type(&record.item.sig.output)
            }
            CallableId::Method {
                package, type_path, ..
            } => {
                let record = self.project.methods.get(callable)?;
                let resolver = Resolver {
                    project: self.project,
                    package,
                    module_path: &record.module_path,
                    aliases: &record.aliases,
                    self_type: Some(TypeRef {
                        package: package.clone(),
                        type_path: type_path.clone(),
                    }),
                };
                resolver.type_from_return_type(&record.item.sig.output)
            }
        }
    }

    fn return_type_arguments_from_callable(&self, callable: &CallableId) -> Vec<TypeRef> {
        match callable {
            CallableId::Free { .. } => {
                let Some(record) = self.project.functions.get(callable) else {
                    return Vec::new();
                };
                let resolver = Resolver {
                    project: self.project,
                    package: &record.package,
                    module_path: &record.module_path,
                    aliases: &record.aliases,
                    self_type: None,
                };
                resolver.type_arguments_from_return_type(&record.item.sig.output)
            }
            CallableId::Method {
                package, type_path, ..
            } => {
                let Some(record) = self.project.methods.get(callable) else {
                    return Vec::new();
                };
                let resolver = Resolver {
                    project: self.project,
                    package,
                    module_path: &record.module_path,
                    aliases: &record.aliases,
                    self_type: Some(TypeRef {
                        package: package.clone(),
                        type_path: type_path.clone(),
                    }),
                };
                resolver.type_arguments_from_return_type(&record.item.sig.output)
            }
        }
    }

    fn return_ok_type_from_callable(&self, callable: &CallableId) -> Option<TypeRef> {
        match callable {
            CallableId::Free { .. } => {
                let record = self.project.functions.get(callable)?;
                let resolver = Resolver {
                    project: self.project,
                    package: &record.package,
                    module_path: &record.module_path,
                    aliases: &record.aliases,
                    self_type: None,
                };
                resolver.ok_type_from_return_type(&record.item.sig.output)
            }
            CallableId::Method {
                package, type_path, ..
            } => {
                let record = self.project.methods.get(callable)?;
                let resolver = Resolver {
                    project: self.project,
                    package,
                    module_path: &record.module_path,
                    aliases: &record.aliases,
                    self_type: Some(TypeRef {
                        package: package.clone(),
                        type_path: type_path.clone(),
                    }),
                };
                resolver.ok_type_from_return_type(&record.item.sig.output)
            }
        }
    }

    fn return_error_type_from_callable(&self, callable: &CallableId) -> Option<TypeRef> {
        match callable {
            CallableId::Free { .. } => {
                let record = self.project.functions.get(callable)?;
                let resolver = Resolver {
                    project: self.project,
                    package: &record.package,
                    module_path: &record.module_path,
                    aliases: &record.aliases,
                    self_type: None,
                };
                resolver.error_type_from_return_type(&record.item.sig.output)
            }
            CallableId::Method {
                package, type_path, ..
            } => {
                let record = self.project.methods.get(callable)?;
                let resolver = Resolver {
                    project: self.project,
                    package,
                    module_path: &record.module_path,
                    aliases: &record.aliases,
                    self_type: Some(TypeRef {
                        package: package.clone(),
                        type_path: type_path.clone(),
                    }),
                };
                resolver.error_type_from_return_type(&record.item.sig.output)
            }
        }
    }

    fn return_map_value_type_from_callable(&self, callable: &CallableId) -> Option<TypeRef> {
        match callable {
            CallableId::Free { .. } => {
                let record = self.project.functions.get(callable)?;
                let resolver = Resolver {
                    project: self.project,
                    package: &record.package,
                    module_path: &record.module_path,
                    aliases: &record.aliases,
                    self_type: None,
                };
                resolver.map_value_type_from_return_type(&record.item.sig.output)
            }
            CallableId::Method {
                package, type_path, ..
            } => {
                let record = self.project.methods.get(callable)?;
                let resolver = Resolver {
                    project: self.project,
                    package,
                    module_path: &record.module_path,
                    aliases: &record.aliases,
                    self_type: Some(TypeRef {
                        package: package.clone(),
                        type_path: type_path.clone(),
                    }),
                };
                resolver.map_value_type_from_return_type(&record.item.sig.output)
            }
        }
    }

    fn return_map_key_type_from_callable(&self, callable: &CallableId) -> Option<TypeRef> {
        match callable {
            CallableId::Free { .. } => {
                let record = self.project.functions.get(callable)?;
                let resolver = Resolver {
                    project: self.project,
                    package: &record.package,
                    module_path: &record.module_path,
                    aliases: &record.aliases,
                    self_type: None,
                };
                resolver.map_key_type_from_return_type(&record.item.sig.output)
            }
            CallableId::Method {
                package, type_path, ..
            } => {
                let record = self.project.methods.get(callable)?;
                let resolver = Resolver {
                    project: self.project,
                    package,
                    module_path: &record.module_path,
                    aliases: &record.aliases,
                    self_type: Some(TypeRef {
                        package: package.clone(),
                        type_path: type_path.clone(),
                    }),
                };
                resolver.map_key_type_from_return_type(&record.item.sig.output)
            }
        }
    }

    fn closure_argument_input_types(
        &self,
        callable: &CallableId,
        argument_index: usize,
    ) -> Vec<TypeRef> {
        let Some((input_type, resolver)) = self.callable_typed_input(callable, argument_index)
        else {
            return Vec::new();
        };
        resolver.closure_input_types_from_type(input_type)
    }

    fn closure_argument_input_types_from_receiver_arguments(
        &self,
        callable: &CallableId,
        argument_index: usize,
        receiver_type_arguments: &[TypeRef],
    ) -> Vec<TypeRef> {
        if receiver_type_arguments.is_empty() {
            return Vec::new();
        }
        let Some((input_type, resolver)) = self.callable_typed_input(callable, argument_index)
        else {
            return Vec::new();
        };
        let generic_names = self.callable_impl_type_parameter_names(callable);
        if generic_names.is_empty() {
            return Vec::new();
        }
        let mut input_generic_names = Vec::new();
        resolver.collect_closure_input_generic_names(
            input_type,
            &generic_names,
            &mut input_generic_names,
        );
        input_generic_names
            .iter()
            .filter_map(|name| {
                generic_names
                    .iter()
                    .position(|generic| generic == name)
                    .and_then(|index| receiver_type_arguments.get(index))
                    .cloned()
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    fn callable_impl_type_parameter_names(&self, callable: &CallableId) -> Vec<String> {
        let CallableId::Method { .. } = callable else {
            return Vec::new();
        };
        let Some(record) = self.project.methods.get(callable) else {
            return Vec::new();
        };
        record
            .impl_generics
            .params
            .iter()
            .filter_map(|parameter| {
                let syn::GenericParam::Type(type_parameter) = parameter else {
                    return None;
                };
                Some(type_parameter.ident.to_string())
            })
            .collect()
    }

    fn callable_typed_input<'a>(
        &'a self,
        callable: &'a CallableId,
        argument_index: usize,
    ) -> Option<(&'a Type, Resolver<'a>)> {
        match callable {
            CallableId::Free { .. } => {
                let record = self.project.functions.get(callable)?;
                let input_type = record
                    .item
                    .sig
                    .inputs
                    .iter()
                    .filter_map(|input| match input {
                        FnArg::Typed(input) => Some(input.ty.as_ref()),
                        FnArg::Receiver(_) => None,
                    })
                    .nth(argument_index)?;
                Some((
                    input_type,
                    Resolver {
                        project: self.project,
                        package: &record.package,
                        module_path: &record.module_path,
                        aliases: &record.aliases,
                        self_type: None,
                    },
                ))
            }
            CallableId::Method {
                package, type_path, ..
            } => {
                let record = self.project.methods.get(callable)?;
                let input_type = record
                    .item
                    .sig
                    .inputs
                    .iter()
                    .filter_map(|input| match input {
                        FnArg::Typed(input) => Some(input.ty.as_ref()),
                        FnArg::Receiver(_) => None,
                    })
                    .nth(argument_index)?;
                Some((
                    input_type,
                    Resolver {
                        project: self.project,
                        package,
                        module_path: &record.module_path,
                        aliases: &record.aliases,
                        self_type: Some(TypeRef {
                            package: package.clone(),
                            type_path: type_path.clone(),
                        }),
                    },
                ))
            }
        }
    }

    fn closure_input_types_from_type(&self, ty: &Type) -> Vec<TypeRef> {
        let mut type_refs = Vec::new();
        self.collect_closure_input_types(ty, &mut type_refs);
        dedup_type_refs_preserve_order(&mut type_refs);
        type_refs
    }

    fn collect_closure_input_types(&self, ty: &Type, type_refs: &mut Vec<TypeRef>) {
        match ty {
            Type::ImplTrait(impl_trait) => {
                for bound in &impl_trait.bounds {
                    let syn::TypeParamBound::Trait(trait_bound) = bound else {
                        continue;
                    };
                    self.collect_closure_input_types_from_path(&trait_bound.path, type_refs);
                }
            }
            Type::TraitObject(trait_object) => {
                for bound in &trait_object.bounds {
                    let syn::TypeParamBound::Trait(trait_bound) = bound else {
                        continue;
                    };
                    self.collect_closure_input_types_from_path(&trait_bound.path, type_refs);
                }
            }
            Type::Path(type_path) => {
                self.collect_closure_input_types_from_path(&type_path.path, type_refs);
                for segment in &type_path.path.segments {
                    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
                        continue;
                    };
                    for argument in &arguments.args {
                        let GenericArgument::Type(ty) = argument else {
                            continue;
                        };
                        self.collect_closure_input_types(ty, type_refs);
                    }
                }
            }
            Type::Reference(reference) => {
                self.collect_closure_input_types(&reference.elem, type_refs)
            }
            Type::Group(group) => self.collect_closure_input_types(&group.elem, type_refs),
            Type::Paren(paren) => self.collect_closure_input_types(&paren.elem, type_refs),
            _ => {}
        }
    }

    fn collect_closure_input_types_from_path(&self, path: &Path, type_refs: &mut Vec<TypeRef>) {
        let Some(segment) = path.segments.last() else {
            return;
        };
        if !matches!(
            segment.ident.to_string().as_str(),
            "Fn" | "FnMut" | "FnOnce"
        ) {
            return;
        }
        let PathArguments::Parenthesized(arguments) = &segment.arguments else {
            return;
        };
        for input in &arguments.inputs {
            type_refs.extend(self.receiver_type_candidates_from_type(input));
        }
    }

    fn collect_closure_input_generic_names(
        &self,
        ty: &Type,
        generic_names: &[String],
        input_generic_names: &mut Vec<String>,
    ) {
        match ty {
            Type::ImplTrait(impl_trait) => {
                for bound in &impl_trait.bounds {
                    let syn::TypeParamBound::Trait(trait_bound) = bound else {
                        continue;
                    };
                    self.collect_closure_input_generic_names_from_path(
                        &trait_bound.path,
                        generic_names,
                        input_generic_names,
                    );
                }
            }
            Type::TraitObject(trait_object) => {
                for bound in &trait_object.bounds {
                    let syn::TypeParamBound::Trait(trait_bound) = bound else {
                        continue;
                    };
                    self.collect_closure_input_generic_names_from_path(
                        &trait_bound.path,
                        generic_names,
                        input_generic_names,
                    );
                }
            }
            Type::Path(type_path) => {
                self.collect_closure_input_generic_names_from_path(
                    &type_path.path,
                    generic_names,
                    input_generic_names,
                );
                for segment in &type_path.path.segments {
                    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
                        continue;
                    };
                    for argument in &arguments.args {
                        let GenericArgument::Type(ty) = argument else {
                            continue;
                        };
                        self.collect_closure_input_generic_names(
                            ty,
                            generic_names,
                            input_generic_names,
                        );
                    }
                }
            }
            Type::Reference(reference) => self.collect_closure_input_generic_names(
                &reference.elem,
                generic_names,
                input_generic_names,
            ),
            Type::Ptr(pointer) => self.collect_closure_input_generic_names(
                &pointer.elem,
                generic_names,
                input_generic_names,
            ),
            Type::Slice(slice) => self.collect_closure_input_generic_names(
                &slice.elem,
                generic_names,
                input_generic_names,
            ),
            Type::Array(array) => self.collect_closure_input_generic_names(
                &array.elem,
                generic_names,
                input_generic_names,
            ),
            Type::Group(group) => self.collect_closure_input_generic_names(
                &group.elem,
                generic_names,
                input_generic_names,
            ),
            Type::Paren(paren) => self.collect_closure_input_generic_names(
                &paren.elem,
                generic_names,
                input_generic_names,
            ),
            Type::Tuple(tuple) => {
                for elem in &tuple.elems {
                    self.collect_closure_input_generic_names(
                        elem,
                        generic_names,
                        input_generic_names,
                    );
                }
            }
            _ => {}
        }
    }

    fn collect_closure_input_generic_names_from_path(
        &self,
        path: &Path,
        generic_names: &[String],
        input_generic_names: &mut Vec<String>,
    ) {
        let Some(segment) = path.segments.last() else {
            return;
        };
        if !matches!(
            segment.ident.to_string().as_str(),
            "Fn" | "FnMut" | "FnOnce"
        ) {
            return;
        }
        let PathArguments::Parenthesized(arguments) = &segment.arguments else {
            return;
        };
        for input in &arguments.inputs {
            collect_type_generic_names(input, generic_names, input_generic_names);
        }
    }

    fn type_from_return_type(&self, output: &ReturnType) -> Option<TypeRef> {
        let ReturnType::Type(_, ty) = output else {
            return None;
        };
        self.resolve_receiver_type(ty)
    }

    fn ok_type_from_return_type(&self, output: &ReturnType) -> Option<TypeRef> {
        let ReturnType::Type(_, ty) = output else {
            return None;
        };
        self.result_ok_type(ty)
    }

    fn error_type_from_return_type(&self, output: &ReturnType) -> Option<TypeRef> {
        let ReturnType::Type(_, ty) = output else {
            return None;
        };
        self.result_error_type(ty)
    }

    fn map_value_type_from_return_type(&self, output: &ReturnType) -> Option<TypeRef> {
        let ReturnType::Type(_, ty) = output else {
            return None;
        };
        self.map_value_type(ty)
    }

    fn map_key_type_from_return_type(&self, output: &ReturnType) -> Option<TypeRef> {
        let ReturnType::Type(_, ty) = output else {
            return None;
        };
        self.map_key_type(ty)
    }

    fn result_ok_type(&self, ty: &Type) -> Option<TypeRef> {
        match ty {
            Type::Path(type_path) => {
                if let Some(ok_type) = self.result_ok_type_from_path(&type_path.path) {
                    return Some(ok_type);
                }
                let alias = self.resolve_type_path(&type_path.path)?;
                let item = self.resolver_item_for_type(&alias)?;
                let record = self.project.items.get(&item)?;
                let Item::Type(type_alias) = &record.item else {
                    return None;
                };
                let resolver = Resolver {
                    project: self.project,
                    package: &record.package,
                    module_path: &record.module_path,
                    aliases: &record.aliases,
                    self_type: None,
                };
                resolver.result_ok_type(&type_alias.ty)
            }
            Type::Reference(reference) => self.result_ok_type(&reference.elem),
            Type::Group(group) => self.result_ok_type(&group.elem),
            Type::Paren(paren) => self.result_ok_type(&paren.elem),
            _ => None,
        }
    }

    fn result_ok_type_from_path(&self, path: &Path) -> Option<TypeRef> {
        let last = path.segments.last()?;
        if last.ident != "Result" {
            return None;
        }
        let PathArguments::AngleBracketed(arguments) = &last.arguments else {
            return None;
        };
        let ok_type = arguments.args.iter().find_map(|argument| {
            let GenericArgument::Type(ty) = argument else {
                return None;
            };
            Some(ty)
        })?;
        self.resolve_receiver_type(ok_type)
    }

    fn result_error_type(&self, ty: &Type) -> Option<TypeRef> {
        match ty {
            Type::Path(type_path) => {
                if let Some(error_type) = self.result_error_type_from_path(&type_path.path) {
                    return Some(error_type);
                }
                let alias = self.resolve_type_path(&type_path.path)?;
                let item = self.resolver_item_for_type(&alias)?;
                let record = self.project.items.get(&item)?;
                let Item::Type(type_alias) = &record.item else {
                    return None;
                };
                let resolver = Resolver {
                    project: self.project,
                    package: &record.package,
                    module_path: &record.module_path,
                    aliases: &record.aliases,
                    self_type: None,
                };
                resolver.result_error_type(&type_alias.ty)
            }
            Type::Reference(reference) => self.result_error_type(&reference.elem),
            Type::Group(group) => self.result_error_type(&group.elem),
            Type::Paren(paren) => self.result_error_type(&paren.elem),
            _ => None,
        }
    }

    fn result_error_type_from_path(&self, path: &Path) -> Option<TypeRef> {
        let last = path.segments.last()?;
        if last.ident != "Result" {
            return None;
        }
        let PathArguments::AngleBracketed(arguments) = &last.arguments else {
            return None;
        };
        let mut type_arguments = arguments.args.iter().filter_map(|argument| {
            let GenericArgument::Type(ty) = argument else {
                return None;
            };
            Some(ty)
        });
        type_arguments.next()?;
        let error_type = type_arguments.next()?;
        self.resolve_receiver_type(error_type)
    }

    fn map_value_type(&self, ty: &Type) -> Option<TypeRef> {
        match ty {
            Type::Path(type_path) => {
                if let Some(value_type) = self.map_value_type_from_path(&type_path.path) {
                    return Some(value_type);
                }
                let alias = self.resolve_type_path(&type_path.path)?;
                let item = self.resolver_item_for_type(&alias)?;
                let record = self.project.items.get(&item)?;
                let Item::Type(type_alias) = &record.item else {
                    return None;
                };
                let resolver = Resolver {
                    project: self.project,
                    package: &record.package,
                    module_path: &record.module_path,
                    aliases: &record.aliases,
                    self_type: None,
                };
                resolver.map_value_type(&type_alias.ty)
            }
            Type::Reference(reference) => self.map_value_type(&reference.elem),
            Type::Group(group) => self.map_value_type(&group.elem),
            Type::Paren(paren) => self.map_value_type(&paren.elem),
            _ => None,
        }
    }

    fn map_key_type(&self, ty: &Type) -> Option<TypeRef> {
        match ty {
            Type::Path(type_path) => {
                if let Some(key_type) = self.map_key_type_from_path(&type_path.path) {
                    return Some(key_type);
                }
                let alias = self.resolve_type_path(&type_path.path)?;
                let item = self.resolver_item_for_type(&alias)?;
                let record = self.project.items.get(&item)?;
                let Item::Type(type_alias) = &record.item else {
                    return None;
                };
                let resolver = Resolver {
                    project: self.project,
                    package: &record.package,
                    module_path: &record.module_path,
                    aliases: &record.aliases,
                    self_type: None,
                };
                resolver.map_key_type(&type_alias.ty)
            }
            Type::Reference(reference) => self.map_key_type(&reference.elem),
            Type::Group(group) => self.map_key_type(&group.elem),
            Type::Paren(paren) => self.map_key_type(&paren.elem),
            _ => None,
        }
    }

    fn map_value_type_from_path(&self, path: &Path) -> Option<TypeRef> {
        let last = path.segments.last()?;
        if !matches!(
            last.ident.to_string().as_str(),
            "BTreeMap" | "HashMap" | "IndexMap"
        ) {
            return None;
        }
        let PathArguments::AngleBracketed(arguments) = &last.arguments else {
            return None;
        };
        let mut type_arguments = arguments.args.iter().filter_map(|argument| {
            let GenericArgument::Type(ty) = argument else {
                return None;
            };
            Some(ty)
        });
        type_arguments.next()?;
        let value_type = type_arguments.next()?;
        self.resolve_receiver_type(value_type)
    }

    fn map_key_type_from_path(&self, path: &Path) -> Option<TypeRef> {
        let last = path.segments.last()?;
        if !matches!(
            last.ident.to_string().as_str(),
            "BTreeMap" | "HashMap" | "IndexMap"
        ) {
            return None;
        }
        let PathArguments::AngleBracketed(arguments) = &last.arguments else {
            return None;
        };
        let key_type = arguments.args.iter().find_map(|argument| {
            let GenericArgument::Type(ty) = argument else {
                return None;
            };
            Some(ty)
        })?;
        self.resolve_receiver_type(key_type)
    }

    fn resolver_item_for_type(&self, type_ref: &TypeRef) -> Option<ItemId> {
        self.find_item(&type_ref.package, &type_ref.type_path, &type_like_kinds())
    }

    fn resolve_type(&self, ty: &Type) -> Option<TypeRef> {
        match ty {
            Type::Path(type_path) => self.resolve_type_path(&type_path.path),
            Type::Reference(reference) => self.resolve_type(&reference.elem),
            _ => None,
        }
    }

    fn resolve_receiver_type(&self, ty: &Type) -> Option<TypeRef> {
        match ty {
            Type::Reference(reference) => return self.resolve_receiver_type(&reference.elem),
            Type::Group(group) => return self.resolve_receiver_type(&group.elem),
            Type::Paren(paren) => return self.resolve_receiver_type(&paren.elem),
            _ => {}
        }
        if let Type::Path(type_path) = ty {
            if let Some(type_ref) = self.resolve_single_type_argument(&type_path.path) {
                return Some(type_ref);
            }
        }
        if let Some(type_ref) = self.resolve_type(ty) {
            return Some(type_ref);
        }

        match ty {
            Type::Path(_) => None,
            _ => None,
        }
    }

    fn resolve_single_type_argument(&self, path: &Path) -> Option<TypeRef> {
        let last = path.segments.last()?;
        let wrapper = last.ident.to_string();
        if !matches!(
            wrapper.as_str(),
            "Box" | "Rc" | "Arc" | "Cow" | "Pin" | "Ref" | "RefMut" | "Mutex" | "RwLock"
        ) {
            return None;
        }

        let PathArguments::AngleBracketed(arguments) = &last.arguments else {
            return None;
        };
        arguments.args.iter().find_map(|argument| {
            let GenericArgument::Type(ty) = argument else {
                return None;
            };
            self.resolve_receiver_type(ty)
        })
    }

    fn receiver_type_candidates_from_type(&self, ty: &Type) -> Vec<TypeRef> {
        let mut type_refs = Vec::new();
        if let Some(type_ref) = self.resolve_type(ty) {
            type_refs.push(type_ref);
        }
        if let Some(type_ref) = self.resolve_receiver_type(ty) {
            type_refs.push(type_ref);
        }
        if let Type::Path(type_path) = ty {
            if let Some(type_ref) = self.syntactic_type_ref(&type_path.path) {
                type_refs.push(type_ref);
            }
        }
        self.collect_type_arguments(ty, &mut type_refs);
        let expanded = type_refs
            .iter()
            .flat_map(|type_ref| self.type_ref_candidates(type_ref))
            .collect::<Vec<_>>();
        type_refs.extend(expanded);
        type_refs.sort();
        type_refs.dedup();
        type_refs
    }

    fn syntactic_type_ref(&self, path: &Path) -> Option<TypeRef> {
        let segments = self.apply_alias(path_segments(path));
        let (package, type_path) = self.resolve_prefix(&segments)?;
        Some(TypeRef { package, type_path })
    }

    fn type_arguments_from_return_type(&self, output: &ReturnType) -> Vec<TypeRef> {
        let ReturnType::Type(_, ty) = output else {
            return Vec::new();
        };
        self.project_type_argument_refs_in_type(ty)
    }

    fn type_refs_in_type(&self, ty: &Type) -> Vec<TypeRef> {
        self.receiver_type_candidates_from_type(ty)
    }

    fn type_argument_refs_in_type(&self, ty: &Type) -> Vec<TypeRef> {
        self.project_type_argument_refs_in_type(ty)
    }

    fn project_type_argument_refs_in_type(&self, ty: &Type) -> Vec<TypeRef> {
        let mut type_refs = Vec::new();
        self.collect_project_type_arguments(ty, &mut type_refs);
        dedup_type_refs_preserve_order(&mut type_refs);
        type_refs
    }

    fn collect_project_type_arguments(&self, ty: &Type, type_refs: &mut Vec<TypeRef>) {
        match ty {
            Type::Path(type_path) => {
                for segment in &type_path.path.segments {
                    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
                        continue;
                    };
                    for argument in &arguments.args {
                        let GenericArgument::Type(ty) = argument else {
                            continue;
                        };
                        if let Some(type_ref) = self.resolve_receiver_type(ty) {
                            if self.resolver_item_for_type(&type_ref).is_some() {
                                type_refs.push(type_ref);
                            }
                        }
                        self.collect_project_type_arguments(ty, type_refs);
                    }
                }
            }
            Type::ImplTrait(impl_trait) => {
                for bound in &impl_trait.bounds {
                    let syn::TypeParamBound::Trait(trait_bound) = bound else {
                        continue;
                    };
                    self.collect_project_path_argument_types(&trait_bound.path, type_refs);
                }
            }
            Type::TraitObject(trait_object) => {
                for bound in &trait_object.bounds {
                    let syn::TypeParamBound::Trait(trait_bound) = bound else {
                        continue;
                    };
                    self.collect_project_path_argument_types(&trait_bound.path, type_refs);
                }
            }
            Type::Reference(reference) => {
                self.collect_project_type_arguments(&reference.elem, type_refs)
            }
            Type::Ptr(pointer) => self.collect_project_type_arguments(&pointer.elem, type_refs),
            Type::Slice(slice) => self.collect_project_type_arguments(&slice.elem, type_refs),
            Type::Array(array) => self.collect_project_type_arguments(&array.elem, type_refs),
            Type::Group(group) => self.collect_project_type_arguments(&group.elem, type_refs),
            Type::Paren(paren) => self.collect_project_type_arguments(&paren.elem, type_refs),
            Type::Tuple(tuple) => {
                for elem in &tuple.elems {
                    if let Some(type_ref) = self.resolve_receiver_type(elem) {
                        if self.resolver_item_for_type(&type_ref).is_some() {
                            type_refs.push(type_ref);
                        }
                    }
                    self.collect_project_type_arguments(elem, type_refs);
                }
            }
            _ => {}
        }
    }

    fn collect_project_path_argument_types(&self, path: &Path, type_refs: &mut Vec<TypeRef>) {
        for segment in &path.segments {
            let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
                continue;
            };
            for argument in &arguments.args {
                let GenericArgument::Type(ty) = argument else {
                    continue;
                };
                if let Some(type_ref) = self.resolve_receiver_type(ty) {
                    if self.resolver_item_for_type(&type_ref).is_some() {
                        type_refs.push(type_ref);
                    }
                }
                self.collect_project_type_arguments(ty, type_refs);
            }
        }
    }

    fn collect_type_arguments(&self, ty: &Type, type_refs: &mut Vec<TypeRef>) {
        match ty {
            Type::Path(type_path) => {
                if let Some(type_ref) = self.resolve_receiver_type(ty) {
                    type_refs.push(type_ref);
                }
                for segment in &type_path.path.segments {
                    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
                        continue;
                    };
                    for argument in &arguments.args {
                        let GenericArgument::Type(ty) = argument else {
                            continue;
                        };
                        if let Some(type_ref) = self.resolve_receiver_type(ty) {
                            type_refs.push(type_ref);
                        }
                        self.collect_type_arguments(ty, type_refs);
                    }
                }
            }
            Type::ImplTrait(impl_trait) => {
                for bound in &impl_trait.bounds {
                    let syn::TypeParamBound::Trait(trait_bound) = bound else {
                        continue;
                    };
                    self.collect_path_argument_types(&trait_bound.path, type_refs);
                }
            }
            Type::TraitObject(trait_object) => {
                for bound in &trait_object.bounds {
                    let syn::TypeParamBound::Trait(trait_bound) = bound else {
                        continue;
                    };
                    self.collect_path_argument_types(&trait_bound.path, type_refs);
                }
            }
            Type::Reference(reference) => self.collect_type_arguments(&reference.elem, type_refs),
            Type::Ptr(pointer) => self.collect_type_arguments(&pointer.elem, type_refs),
            Type::Slice(slice) => self.collect_type_arguments(&slice.elem, type_refs),
            Type::Array(array) => self.collect_type_arguments(&array.elem, type_refs),
            Type::Group(group) => self.collect_type_arguments(&group.elem, type_refs),
            Type::Paren(paren) => self.collect_type_arguments(&paren.elem, type_refs),
            Type::Tuple(tuple) => {
                for elem in &tuple.elems {
                    self.collect_type_arguments(elem, type_refs);
                }
            }
            _ => {}
        }
    }

    fn collect_path_argument_types(&self, path: &Path, type_refs: &mut Vec<TypeRef>) {
        for segment in &path.segments {
            let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
                continue;
            };
            for argument in &arguments.args {
                let GenericArgument::Type(ty) = argument else {
                    continue;
                };
                if let Some(type_ref) = self.resolve_receiver_type(ty) {
                    type_refs.push(type_ref);
                }
                self.collect_type_arguments(ty, type_refs);
            }
        }
    }

    fn type_from_path_turbofish(&self, path: &Path) -> Option<TypeRef> {
        path.segments.iter().find_map(|segment| {
            let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
                return None;
            };
            arguments.args.iter().find_map(|argument| {
                let GenericArgument::Type(ty) = argument else {
                    return None;
                };
                self.resolve_type(ty)
            })
        })
    }

    fn resolve_conversion_impls(
        &self,
        receiver: &TypeRef,
        trait_name: &str,
        method_name: &str,
    ) -> Vec<CallableId> {
        self.project
            .methods
            .iter()
            .filter_map(|(id, record)| {
                let CallableId::Method {
                    package,
                    trait_path: Some(trait_path),
                    method,
                    ..
                } = id
                else {
                    return None;
                };
                (method == method_name
                    && trait_path
                        .last()
                        .is_some_and(|candidate| candidate == trait_name)
                    && record
                        .trait_input_type_paths
                        .iter()
                        .any(|type_path| conversion_input_matches_receiver(type_path, receiver))
                    && (package == &receiver.package || package == self.package))
                    .then(|| id.clone())
            })
            .collect()
    }

    fn resolve_conversion_impls_from_to(
        &self,
        source: &TypeRef,
        target: &TypeRef,
        trait_name: &str,
        method_name: &str,
    ) -> Vec<CallableId> {
        let source_candidates = self.type_ref_candidates(source);
        let target_candidates = self.type_ref_candidates(target);
        self.project
            .methods
            .iter()
            .filter_map(|(id, record)| {
                let CallableId::Method {
                    package,
                    type_path,
                    trait_path: Some(trait_path),
                    method,
                    ..
                } = id
                else {
                    return None;
                };
                (method == method_name
                    && trait_path
                        .last()
                        .is_some_and(|candidate| candidate == trait_name)
                    && target_candidates.iter().any(|candidate| {
                        package == &candidate.package && type_path == &candidate.type_path
                    })
                    && source_candidates.iter().any(|candidate| {
                        record.trait_input_type_paths.iter().any(|type_path| {
                            conversion_input_matches_receiver(type_path, candidate)
                        })
                    })
                    && (package == &source.package
                        || package == &target.package
                        || package == self.package))
                    .then(|| id.clone())
            })
            .collect()
    }

    fn resolve_conversion_impls_to_target(
        &self,
        target: &TypeRef,
        trait_name: &str,
        method_name: &str,
    ) -> Vec<CallableId> {
        let target_name = target.type_path.last();
        self.project
            .methods
            .keys()
            .filter_map(|id| {
                let CallableId::Method {
                    package,
                    type_path,
                    trait_path: Some(trait_path),
                    method,
                    ..
                } = id
                else {
                    return None;
                };
                (method == method_name
                    && trait_path
                        .last()
                        .is_some_and(|candidate| candidate == trait_name)
                    && (package == &target.package || package == self.package)
                    && (type_path == &target.type_path
                        || target_name.is_some_and(|name| type_path.last() == Some(name))))
                .then(|| id.clone())
            })
            .collect()
    }

    fn resolve_type_path(&self, path: &Path) -> Option<TypeRef> {
        let raw_segments = path_segments(path);
        if let Some(type_ref) = self.resolve_type_segments(&raw_segments) {
            return Some(type_ref);
        }
        let segments = self.apply_alias(raw_segments);
        self.resolve_type_segments(&segments)
    }

    fn resolve_associated_call_type(&self, path: &Path) -> Option<TypeRef> {
        let mut raw_segments = path_segments(path);
        if raw_segments.len() < 2 {
            return None;
        }
        raw_segments.pop();
        if let Some(type_ref) = self.resolve_type_segments(&raw_segments) {
            return Some(type_ref);
        }
        let segments = self.apply_alias(raw_segments);
        self.resolve_type_segments(&segments)
    }

    fn type_ref_candidates(&self, type_ref: &TypeRef) -> Vec<TypeRef> {
        let mut candidates = Vec::new();
        let mut queue = VecDeque::from([type_ref.clone()]);
        while let Some(candidate) = queue.pop_front() {
            if candidates.contains(&candidate) {
                continue;
            }
            if let Some(alias_target) = self.type_alias_target(&candidate) {
                queue.push_back(alias_target);
            }
            if let Some(reexport_target) = self.crate_root_reexport_method_candidate(&candidate) {
                queue.push_back(reexport_target);
            }
            if let Some(reexport_target) = self.reexported_type_method_candidate(&candidate) {
                queue.push_back(reexport_target);
            }
            candidates.push(candidate);
        }
        candidates
    }

    fn deref_target_types(&self, receiver: &TypeRef) -> Vec<TypeRef> {
        let mut targets = Vec::new();
        for candidate in self.type_ref_candidates(receiver) {
            for (callable, record) in &self.project.methods {
                let CallableId::Method {
                    package,
                    type_path,
                    trait_path: Some(trait_path),
                    method,
                    ..
                } = callable
                else {
                    continue;
                };
                if package != &candidate.package
                    || type_path != &candidate.type_path
                    || !matches!(method.as_str(), "deref" | "deref_mut")
                    || !trait_path
                        .last()
                        .is_some_and(|name| matches!(name.as_str(), "Deref" | "DerefMut"))
                {
                    continue;
                }

                let resolver = Resolver {
                    project: self.project,
                    package,
                    module_path: &record.module_path,
                    aliases: &record.aliases,
                    self_type: Some(candidate.clone()),
                };
                for impl_item in &record.impl_items {
                    let ImplItem::Type(item_type) = impl_item else {
                        continue;
                    };
                    if item_type.ident != "Target" {
                        continue;
                    }
                    if let Some(target) = resolver.resolve_receiver_type(&item_type.ty) {
                        targets.push(target);
                    }
                }
            }
        }
        targets.sort();
        targets.dedup();
        targets
    }

    fn index_output_types(&self, receiver: &TypeRef) -> Vec<TypeRef> {
        self.trait_associated_type_targets(receiver, "Index", "Output")
    }

    fn from_str_error_types(&self, receiver: &TypeRef) -> Vec<TypeRef> {
        self.trait_associated_type_targets(receiver, "FromStr", "Err")
    }

    fn trait_associated_type_targets(
        &self,
        receiver: &TypeRef,
        trait_name: &str,
        associated_type: &str,
    ) -> Vec<TypeRef> {
        let mut targets = Vec::new();
        for candidate in self.type_ref_candidates(receiver) {
            for (callable, record) in &self.project.methods {
                let CallableId::Method {
                    package,
                    type_path,
                    trait_path: Some(trait_path),
                    ..
                } = callable
                else {
                    continue;
                };
                if package != &candidate.package
                    || type_path != &candidate.type_path
                    || !trait_path.last().is_some_and(|name| name == trait_name)
                {
                    continue;
                }

                let resolver = Resolver {
                    project: self.project,
                    package,
                    module_path: &record.module_path,
                    aliases: &record.aliases,
                    self_type: Some(candidate.clone()),
                };
                for impl_item in &record.impl_items {
                    let ImplItem::Type(item_type) = impl_item else {
                        continue;
                    };
                    if item_type.ident != associated_type {
                        continue;
                    }
                    if let Some(target) = resolver.resolve_receiver_type(&item_type.ty) {
                        targets.push(target);
                    }
                }
            }
        }
        targets.sort();
        targets.dedup();
        targets
    }

    fn crate_root_reexport_method_candidate(&self, type_ref: &TypeRef) -> Option<TypeRef> {
        if type_ref.type_path.len() <= 1 {
            return None;
        }
        let leaf = type_ref.type_path.last()?.clone();
        let candidate = TypeRef {
            package: type_ref.package.clone(),
            type_path: vec![leaf],
        };
        self.project
            .receivers_with_methods
            .contains(&(candidate.package.clone(), candidate.type_path.clone()))
            .then_some(candidate)
    }

    fn reexported_type_method_candidate(&self, type_ref: &TypeRef) -> Option<TypeRef> {
        let item = self.find_item(&type_ref.package, &type_ref.type_path, &type_like_kinds())?;
        let item_path = path_from_item(&item);
        if item_path == type_ref.type_path {
            return None;
        }
        let candidate = TypeRef {
            package: item.package,
            type_path: item_path,
        };
        self.project
            .receivers_with_methods
            .contains(&(candidate.package.clone(), candidate.type_path.clone()))
            .then_some(candidate)
    }

    fn type_alias_target(&self, type_ref: &TypeRef) -> Option<TypeRef> {
        let item = self.find_item(&type_ref.package, &type_ref.type_path, &[ItemKind::Type])?;
        let record = self.project.items.get(&item)?;
        let syn::Item::Type(item_type) = &record.item else {
            return None;
        };
        let resolver = Resolver {
            project: self.project,
            package: &record.package,
            module_path: &record.module_path,
            aliases: &record.aliases,
            self_type: None,
        };
        resolver.resolve_receiver_type(&item_type.ty)
    }

    fn field_type(&self, receiver: &TypeRef, member: &Member) -> Option<TypeRef> {
        for candidate in self.type_ref_candidates(receiver) {
            let item = self.find_item(
                &candidate.package,
                &candidate.type_path,
                &[ItemKind::Struct],
            )?;
            let record = self.project.items.get(&item)?;
            let syn::Item::Struct(item_struct) = &record.item else {
                continue;
            };
            let Some(ty) = field_member_type(&item_struct.fields, member) else {
                continue;
            };
            let resolver = Resolver {
                project: self.project,
                package: &record.package,
                module_path: &record.module_path,
                aliases: &record.aliases,
                self_type: None,
            };
            if let Some(type_ref) = resolver.resolve_receiver_type(ty) {
                return Some(type_ref);
            }
        }

        None
    }

    fn field_type_candidates(&self, receiver: &TypeRef, member: &Member) -> Vec<TypeRef> {
        let mut type_refs = Vec::new();
        for candidate in self.type_ref_candidates(receiver) {
            let Some(item) = self.find_item(
                &candidate.package,
                &candidate.type_path,
                &[ItemKind::Struct],
            ) else {
                continue;
            };
            let Some(record) = self.project.items.get(&item) else {
                continue;
            };
            let syn::Item::Struct(item_struct) = &record.item else {
                continue;
            };
            let Some(ty) = field_member_type(&item_struct.fields, member) else {
                continue;
            };
            let resolver = Resolver {
                project: self.project,
                package: &record.package,
                module_path: &record.module_path,
                aliases: &record.aliases,
                self_type: None,
            };
            type_refs.extend(resolver.receiver_type_candidates_from_type(ty));
        }

        type_refs.sort();
        type_refs.dedup();
        type_refs
    }

    fn field_type_arguments(&self, receiver: &TypeRef, member: &Member) -> Vec<TypeRef> {
        let mut type_refs = Vec::new();
        for candidate in self.type_ref_candidates(receiver) {
            let Some(item) = self.find_item(
                &candidate.package,
                &candidate.type_path,
                &[ItemKind::Struct],
            ) else {
                continue;
            };
            let Some(record) = self.project.items.get(&item) else {
                continue;
            };
            let syn::Item::Struct(item_struct) = &record.item else {
                continue;
            };
            let Some(ty) = field_member_type(&item_struct.fields, member) else {
                continue;
            };
            let resolver = Resolver {
                project: self.project,
                package: &record.package,
                module_path: &record.module_path,
                aliases: &record.aliases,
                self_type: None,
            };
            type_refs.extend(resolver.type_argument_refs_in_type(ty));
        }

        type_refs.sort();
        type_refs.dedup();
        type_refs
    }

    fn enum_tuple_variant_field_types(
        &self,
        receiver: &TypeRef,
        variant_name: &str,
    ) -> Vec<TypeRef> {
        for candidate in self.type_ref_candidates(receiver) {
            let Some((record, variant)) = self.enum_variant_record(&candidate, variant_name) else {
                continue;
            };
            let syn::Fields::Unnamed(fields) = &variant.fields else {
                continue;
            };
            let resolver = Resolver {
                project: self.project,
                package: &record.package,
                module_path: &record.module_path,
                aliases: &record.aliases,
                self_type: None,
            };
            return fields
                .unnamed
                .iter()
                .filter_map(|field| resolver.resolve_receiver_type(&field.ty))
                .collect();
        }

        Vec::new()
    }

    fn tuple_struct_field_types(&self, receiver: &TypeRef, constructor_name: &str) -> Vec<TypeRef> {
        for candidate in self.type_ref_candidates(receiver) {
            if candidate
                .type_path
                .last()
                .is_none_or(|name| name != constructor_name)
            {
                continue;
            }
            let Some(item) = self.find_item(
                &candidate.package,
                &candidate.type_path,
                &[ItemKind::Struct],
            ) else {
                continue;
            };
            let Some(record) = self.project.items.get(&item) else {
                continue;
            };
            let syn::Item::Struct(item_struct) = &record.item else {
                continue;
            };
            let syn::Fields::Unnamed(fields) = &item_struct.fields else {
                continue;
            };
            let resolver = Resolver {
                project: self.project,
                package: &record.package,
                module_path: &record.module_path,
                aliases: &record.aliases,
                self_type: None,
            };
            return fields
                .unnamed
                .iter()
                .filter_map(|field| resolver.resolve_receiver_type(&field.ty))
                .collect();
        }

        Vec::new()
    }

    fn enum_named_variant_field_type(
        &self,
        receiver: &TypeRef,
        variant_name: &str,
        member: &Member,
    ) -> Option<TypeRef> {
        for candidate in self.type_ref_candidates(receiver) {
            let Some((record, variant)) = self.enum_variant_record(&candidate, variant_name) else {
                continue;
            };
            let Some(ty) = field_member_type(&variant.fields, member) else {
                continue;
            };
            let resolver = Resolver {
                project: self.project,
                package: &record.package,
                module_path: &record.module_path,
                aliases: &record.aliases,
                self_type: None,
            };
            if let Some(type_ref) = resolver.resolve_receiver_type(ty) {
                return Some(type_ref);
            }
        }

        None
    }

    fn enum_variant_record<'a>(
        &'a self,
        receiver: &TypeRef,
        variant_name: &str,
    ) -> Option<(&'a crate::model::ItemRecord, &'a syn::Variant)> {
        let item = self.find_item(&receiver.package, &receiver.type_path, &[ItemKind::Enum])?;
        let record = self.project.items.get(&item)?;
        let syn::Item::Enum(item_enum) = &record.item else {
            return None;
        };
        let variant = item_enum
            .variants
            .iter()
            .find(|variant| variant.ident == variant_name)?;
        Some((record, variant))
    }

    fn resolve_type_segments(&self, segments: &[String]) -> Option<TypeRef> {
        let Some(item) = self.resolve_item_segments(segments) else {
            return self.external_type_segments(segments);
        };
        matches!(
            item.kind,
            ItemKind::Struct | ItemKind::Enum | ItemKind::Union | ItemKind::Type | ItemKind::Trait
        )
        .then(|| TypeRef {
            package: item.package.clone(),
            type_path: path_from_item(&item),
        })
    }

    fn external_type_segments(&self, segments: &[String]) -> Option<TypeRef> {
        let first = segments.first()?;
        if !matches!(first.as_str(), "std" | "core" | "alloc")
            && !self.package_has_dependency_named(first)
        {
            return None;
        }
        Some(TypeRef {
            package: self.package.to_string(),
            type_path: segments.to_vec(),
        })
    }

    fn package_has_dependency_named(&self, name: &str) -> bool {
        let Some(package) = self.project.workspace.packages.get(self.package) else {
            return false;
        };
        package
            .dependencies
            .iter()
            .any(|dependency| dependency_name_matches(dependency, name))
    }

    fn resolve_local_type_item(&self, type_path: &[String]) -> Option<ItemId> {
        self.find_item(self.package, type_path, &type_like_kinds())
    }

    fn resolve_local_trait_item(&self, trait_path: &[String]) -> Option<ItemId> {
        self.find_item(self.package, trait_path, &[ItemKind::Trait])
            .or_else(|| {
                let segments = self.apply_alias(trait_path.to_vec());
                self.resolve_item_segments(&segments)
                    .filter(|item| item.kind == ItemKind::Trait)
            })
    }

    fn resolve_trait_item_in_package(
        &self,
        package: &str,
        trait_path: &[String],
    ) -> Option<ItemId> {
        self.find_item(package, trait_path, &[ItemKind::Trait])
            .or_else(|| {
                let leaf = trait_path.last()?;
                let mut matches = self
                    .project
                    .items
                    .keys()
                    .filter(|item| {
                        item.package == package
                            && item.kind == ItemKind::Trait
                            && item.name == *leaf
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                matches.sort();
                matches.into_iter().next()
            })
    }

    fn resolve_trait_item(&self, trait_path: &[String]) -> Option<ItemId> {
        if let Some(item) = self.resolve_local_trait_item(trait_path) {
            return Some(item);
        }
        let name = trait_path.last()?;
        let mut matches = self
            .project
            .items
            .keys()
            .filter(|item| item.kind == ItemKind::Trait && item.name == *name)
            .cloned()
            .collect::<Vec<_>>();
        matches.sort();
        matches.into_iter().next()
    }

    fn resolve_item_path(&self, path: &Path) -> Option<ItemId> {
        let segments = self.apply_alias(path_segments(path));
        self.resolve_item_segments(&segments)
    }

    fn resolve_pattern_ident_item(&self, ident: &str) -> Option<ItemId> {
        let segments = self.apply_alias(vec![ident.to_string()]);
        self.resolve_item_segments(&segments)
            .filter(|item| matches!(item.kind, ItemKind::Const | ItemKind::Static))
    }

    fn resolve_macro_path(&self, path: &Path) -> Option<ItemId> {
        let segments = self.apply_alias(path_segments(path));
        self.resolve_macro_segments(&segments)
    }

    fn resolve_item_segments(&self, segments: &[String]) -> Option<ItemId> {
        for split in (1..=segments.len()).rev() {
            let candidate = &segments[..split];
            let Some((package, path)) = self.resolve_prefix(candidate) else {
                continue;
            };
            if let Some(item) = self.find_item(&package, &path, &all_item_kinds()) {
                return Some(item);
            }
        }
        None
    }

    fn resolve_macro_segments(&self, segments: &[String]) -> Option<ItemId> {
        for split in (1..=segments.len()).rev() {
            let candidate = &segments[..split];
            let Some((package, path)) = self.resolve_prefix(candidate) else {
                continue;
            };
            if let Some(item) = self.find_item(&package, &path, &[ItemKind::Macro]) {
                return Some(item);
            }
        }
        let name = segments.last()?;
        let package = segments
            .first()
            .and_then(|first| self.resolve_dependency(first))
            .unwrap_or_else(|| self.package.to_string());
        let mut matches = self
            .project
            .items
            .keys()
            .filter(|item| {
                item.package == package && item.kind == ItemKind::Macro && item.name == *name
            })
            .cloned()
            .collect::<Vec<_>>();
        matches.sort();
        (matches.len() == 1).then(|| matches.remove(0))
    }

    fn find_item(&self, package: &str, path: &[String], kinds: &[ItemKind]) -> Option<ItemId> {
        if path.is_empty() {
            return None;
        }
        let name = path.last()?.clone();
        let module_path = path[..path.len() - 1].to_vec();
        if let Some(item) = self.find_item_in_module(package, path, kinds) {
            return Some(item);
        }

        if let Some(alias) = self.resolve_alias_target(package, &module_path, &name) {
            if let Some(item) = self.find_item(&alias.package, &alias.full_path(), kinds) {
                return Some(item);
            }
        }

        self.find_glob_reexport_item(package, &module_path, &name, kinds, &mut BTreeSet::new())
    }

    fn find_function(
        &self,
        package: &str,
        path: &[String],
        visited: &mut BTreeSet<(String, Vec<String>)>,
    ) -> Option<CallableId> {
        if path.is_empty() || !visited.insert((package.to_string(), path.to_vec())) {
            return None;
        }

        let name = path.last()?.clone();
        let module_path = path[..path.len() - 1].to_vec();
        let id = CallableId::Free {
            package: package.to_string(),
            module_path: module_path.clone(),
            name: name.clone(),
        };
        if self.project.functions.contains_key(&id) {
            return Some(id);
        }

        if let Some(alias) = self.resolve_alias_target(package, &module_path, &name) {
            if let Some(callable) = self.find_function(&alias.package, &alias.full_path(), visited)
            {
                return Some(callable);
            }
        }

        self.find_glob_reexport_function(package, &module_path, &name, &mut BTreeSet::new())
    }

    fn find_item_in_module(
        &self,
        package: &str,
        path: &[String],
        kinds: &[ItemKind],
    ) -> Option<ItemId> {
        if path.is_empty() {
            return None;
        }
        let name = path.last()?.clone();
        let module_path = path[..path.len() - 1].to_vec();
        kinds.iter().find_map(|kind| {
            let id = ItemId {
                package: package.to_string(),
                module_path: module_path.clone(),
                name: name.clone(),
                kind: *kind,
            };
            self.project.items.contains_key(&id).then_some(id)
        })
    }

    fn find_glob_reexport_item(
        &self,
        package: &str,
        module_path: &[String],
        name: &str,
        kinds: &[ItemKind],
        visited: &mut BTreeSet<(String, Vec<String>, String)>,
    ) -> Option<ItemId> {
        if !visited.insert((package.to_string(), module_path.to_vec(), name.to_string())) {
            return None;
        }

        let items = self.items_for_module(package, module_path)?;
        for glob_path in visible_glob_use_paths(items) {
            let aliases = self
                .project
                .module_aliases
                .get(&(package.to_string(), module_path.to_vec()))
                .cloned()
                .unwrap_or_default();
            let resolver = Resolver {
                project: self.project,
                package,
                module_path,
                aliases: &aliases,
                self_type: None,
            };
            let glob_path = resolver.apply_alias(glob_path);
            let Some((target_package, target_module_path)) = resolver.resolve_prefix(&glob_path)
            else {
                continue;
            };

            let mut target_path = target_module_path.clone();
            target_path.push(name.to_string());
            if let Some(item) = self.find_item_in_module(&target_package, &target_path, kinds) {
                return Some(item);
            }
            if let Some(alias) =
                self.resolve_alias_target(&target_package, &target_module_path, name)
            {
                let alias_path = alias.full_path();
                if let Some(item) = self.find_item_in_module(&alias.package, &alias_path, kinds) {
                    return Some(item);
                }
                if let Some((alias_name, alias_module_path)) = alias_path.split_last() {
                    if let Some(item) = self.find_glob_reexport_item(
                        &alias.package,
                        alias_module_path,
                        alias_name,
                        kinds,
                        visited,
                    ) {
                        return Some(item);
                    }
                }
            }
            if let Some(item) = self.find_glob_reexport_item(
                &target_package,
                &target_module_path,
                name,
                kinds,
                visited,
            ) {
                return Some(item);
            }
        }

        None
    }

    fn find_glob_reexport_function(
        &self,
        package: &str,
        module_path: &[String],
        name: &str,
        visited: &mut BTreeSet<(String, Vec<String>, String)>,
    ) -> Option<CallableId> {
        if !visited.insert((package.to_string(), module_path.to_vec(), name.to_string())) {
            return None;
        }

        let items = self.items_for_module(package, module_path)?;
        for glob_path in visible_glob_use_paths(items) {
            let aliases = self
                .project
                .module_aliases
                .get(&(package.to_string(), module_path.to_vec()))
                .cloned()
                .unwrap_or_default();
            let resolver = Resolver {
                project: self.project,
                package,
                module_path,
                aliases: &aliases,
                self_type: None,
            };
            let glob_path = resolver.apply_alias(glob_path);
            let Some((target_package, target_module_path)) = resolver.resolve_prefix(&glob_path)
            else {
                continue;
            };

            let id = CallableId::Free {
                package: target_package.clone(),
                module_path: target_module_path.clone(),
                name: name.to_string(),
            };
            if self.project.functions.contains_key(&id) {
                return Some(id);
            }
            if let Some(alias) =
                self.resolve_alias_target(&target_package, &target_module_path, name)
            {
                let alias_path = alias.full_path();
                if let Some((alias_name, alias_module_path)) = alias_path.split_last() {
                    let id = CallableId::Free {
                        package: alias.package.clone(),
                        module_path: alias_module_path.to_vec(),
                        name: alias_name.clone(),
                    };
                    if self.project.functions.contains_key(&id) {
                        return Some(id);
                    }
                    if let Some(callable) = self.find_glob_reexport_function(
                        &alias.package,
                        alias_module_path,
                        alias_name,
                        visited,
                    ) {
                        return Some(callable);
                    }
                }
            }
            if let Some(callable) = self.find_glob_reexport_function(
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

    fn resolve_value_prefix(&self, prefix: &[String]) -> Option<(String, Vec<String>)> {
        if prefix.is_empty() {
            return Some((self.package.to_string(), self.module_path.to_vec()));
        }

        self.resolve_prefix(prefix)
    }

    fn resolve_prefix(&self, prefix: &[String]) -> Option<(String, Vec<String>)> {
        let first = prefix.first()?;
        match first.as_str() {
            "crate" => Some((self.package.to_string(), prefix[1..].to_vec())),
            "self" => {
                let mut path = self.module_path.to_vec();
                path.extend_from_slice(&prefix[1..]);
                Some((self.package.to_string(), path))
            }
            "super" => {
                let mut path = self.module_path.to_vec();
                path.pop();
                path.extend_from_slice(&prefix[1..]);
                Some((self.package.to_string(), path))
            }
            "Self" => self
                .self_type
                .as_ref()
                .map(|self_type| (self_type.package.clone(), self_type.type_path.clone())),
            package if package == self.package => {
                Some((self.package.to_string(), prefix[1..].to_vec()))
            }
            dependency => {
                if let Some(package) = self.resolve_dependency(dependency) {
                    return Some((package, prefix[1..].to_vec()));
                }

                let mut path = self.module_path.to_vec();
                path.extend_from_slice(prefix);
                Some((self.package.to_string(), path))
            }
        }
    }

    fn source_for_module(
        &self,
        package: &str,
        module_path: &[String],
    ) -> Option<&crate::model::SourceFile> {
        let path = self
            .project
            .source_files_by_module
            .get(&(package.to_string(), module_path.to_vec()))?;
        self.project.files.get(path)
    }

    fn items_for_module(&self, package: &str, module_path: &[String]) -> Option<&[Item]> {
        if let Some(source) = self.source_for_module(package, module_path) {
            return Some(&source.syntax.items);
        }

        for split in (0..module_path.len()).rev() {
            let prefix = &module_path[..split];
            let Some(source) = self.source_for_module(package, prefix) else {
                continue;
            };
            if let Some(items) = inline_module_items(&source.syntax.items, &module_path[split..]) {
                return Some(items);
            }
        }

        None
    }

    fn resolve_dependency(&self, first: &str) -> Option<String> {
        let package = self.project.workspace.packages.get(self.package)?;
        package
            .dependencies
            .iter()
            .find(|dependency| dependency_name_matches(dependency, first))
            .and_then(|dependency| {
                self.project
                    .workspace
                    .packages
                    .contains_key(&dependency.package)
                    .then(|| dependency.package.clone())
            })
    }

    fn apply_alias(&self, segments: Vec<String>) -> Vec<String> {
        let Some(first) = segments.first() else {
            return segments;
        };
        let Some(target) = self.aliases.get(first) else {
            return segments;
        };

        let mut resolved = target.clone();
        resolved.extend_from_slice(&segments[1..]);
        resolved
    }

    fn resolve_alias_target(
        &self,
        package: &str,
        module_path: &[String],
        name: &str,
    ) -> Option<ResolvedAlias> {
        let aliases = self
            .project
            .module_aliases
            .get(&(package.to_string(), module_path.to_vec()))?;
        let target = aliases.get(name)?;
        let target_name = target.last()?.clone();
        let alias_resolver = Resolver {
            project: self.project,
            package,
            module_path,
            aliases,
            self_type: None,
        };
        let (target_package, target_module_path) =
            alias_resolver.resolve_value_prefix(&target[..target.len() - 1])?;
        if target_package == package && target_module_path == module_path && target_name == name {
            return None;
        }
        Some(ResolvedAlias {
            package: target_package,
            module_path: target_module_path,
            name: target_name,
        })
    }
}

struct ResolvedAlias {
    package: String,
    module_path: Vec<String>,
    name: String,
}

impl ResolvedAlias {
    fn full_path(&self) -> Vec<String> {
        let mut path = self.module_path.clone();
        path.push(self.name.clone());
        path
    }
}

fn path_segments(path: &Path) -> Vec<String> {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect()
}

fn std_mem_return_function(path: &Path) -> Option<String> {
    let segments = path_segments(path);
    let function = segments.last()?;
    if !matches!(function.as_str(), "take" | "replace") {
        return None;
    }
    let prefix = &segments[..segments.len().saturating_sub(1)];
    if matches!(prefix, [module] if module == "mem")
        || matches!(prefix, [root, module] if matches!(root.as_str(), "std" | "core") && module == "mem")
    {
        Some(function.clone())
    } else {
        None
    }
}

fn generic_parameter_name_from_type(ty: &Type) -> Option<String> {
    match ty {
        Type::Path(type_path) => single_segment_type_name(&type_path.path),
        Type::Reference(reference) => generic_parameter_name_from_type(&reference.elem),
        Type::Group(group) => generic_parameter_name_from_type(&group.elem),
        Type::Paren(paren) => generic_parameter_name_from_type(&paren.elem),
        _ => None,
    }
}

fn collect_type_generic_names(
    ty: &Type,
    generic_names: &[String],
    input_generic_names: &mut Vec<String>,
) {
    match ty {
        Type::Path(type_path) => {
            if let Some(name) = single_segment_type_name(&type_path.path) {
                if generic_names.contains(&name) {
                    input_generic_names.push(name);
                }
            }
            for segment in &type_path.path.segments {
                let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
                    continue;
                };
                for argument in &arguments.args {
                    let GenericArgument::Type(ty) = argument else {
                        continue;
                    };
                    collect_type_generic_names(ty, generic_names, input_generic_names);
                }
            }
        }
        Type::Reference(reference) => {
            collect_type_generic_names(&reference.elem, generic_names, input_generic_names)
        }
        Type::Ptr(pointer) => {
            collect_type_generic_names(&pointer.elem, generic_names, input_generic_names)
        }
        Type::Slice(slice) => {
            collect_type_generic_names(&slice.elem, generic_names, input_generic_names)
        }
        Type::Array(array) => {
            collect_type_generic_names(&array.elem, generic_names, input_generic_names)
        }
        Type::Group(group) => {
            collect_type_generic_names(&group.elem, generic_names, input_generic_names)
        }
        Type::Paren(paren) => {
            collect_type_generic_names(&paren.elem, generic_names, input_generic_names)
        }
        Type::Tuple(tuple) => {
            for elem in &tuple.elems {
                collect_type_generic_names(elem, generic_names, input_generic_names);
            }
        }
        _ => {}
    }
}

fn single_segment_type_name(path: &Path) -> Option<String> {
    (path.leading_colon.is_none() && path.segments.len() == 1)
        .then(|| path.segments.first().unwrap().ident.to_string())
}

fn transparent_receiver_wrapper(name: &str) -> bool {
    matches!(
        name,
        "Box" | "Rc" | "Arc" | "Cow" | "Pin" | "Ref" | "RefMut" | "Mutex" | "RwLock"
    )
}

fn path_from_item(item: &ItemId) -> Vec<String> {
    let mut path = item.module_path.clone();
    path.push(item.name.clone());
    path
}

fn project_module_paths(project: &Project, package: &str) -> BTreeSet<Vec<String>> {
    let mut paths = project
        .files
        .values()
        .filter(|source| source.package == package)
        .map(|source| source.module_path.clone())
        .collect::<BTreeSet<_>>();
    paths.extend(
        project
            .module_aliases
            .keys()
            .filter(|(candidate, _)| candidate == package)
            .map(|(_, module_path)| module_path.clone()),
    );
    paths.extend(
        project
            .items
            .keys()
            .filter(|item| item.package == package)
            .map(|item| item.module_path.clone()),
    );
    paths.extend(
        project
            .functions
            .values()
            .filter(|record| record.package == package)
            .map(|record| record.module_path.clone()),
    );
    paths.extend(
        project
            .methods
            .iter()
            .filter(|(id, _)| id.package() == package)
            .map(|(_, record)| record.module_path.clone()),
    );
    paths
}

fn module_items_for_path<'a>(
    project: &'a Project,
    package: &str,
    module_path: &[String],
) -> Option<&'a [Item]> {
    if let Some(source) = project
        .files
        .values()
        .find(|source| source.package == package && source.module_path == module_path)
    {
        return Some(&source.syntax.items);
    }

    for split in (0..module_path.len()).rev() {
        let prefix = &module_path[..split];
        let Some(source) = project
            .files
            .values()
            .find(|source| source.package == package && source.module_path == prefix)
        else {
            continue;
        };
        if let Some(items) = inline_module_items(&source.syntax.items, &module_path[split..]) {
            return Some(items);
        }
    }

    None
}

fn inline_module_items<'a>(items: &'a [Item], module_path: &[String]) -> Option<&'a [Item]> {
    let (name, rest) = module_path.split_first()?;
    let child = items.iter().find_map(|item| {
        let Item::Mod(item_mod) = item else {
            return None;
        };
        (item_mod.ident == name.as_str()).then_some(item_mod)
    })?;
    let (_, child_items) = child.content.as_ref()?;
    if rest.is_empty() {
        Some(child_items.as_slice())
    } else {
        inline_module_items(child_items, rest)
    }
}

fn type_like_kinds() -> [ItemKind; 5] {
    [
        ItemKind::Struct,
        ItemKind::Enum,
        ItemKind::Union,
        ItemKind::Type,
        ItemKind::Trait,
    ]
}

fn all_item_kinds() -> [ItemKind; 8] {
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
}

fn field_member_type<'a>(fields: &'a syn::Fields, member: &Member) -> Option<&'a Type> {
    match (fields, member) {
        (syn::Fields::Named(fields), Member::Named(name)) => fields
            .named
            .iter()
            .find(|field| field.ident.as_ref().is_some_and(|ident| ident == name))
            .map(|field| &field.ty),
        (syn::Fields::Unnamed(fields), Member::Unnamed(index)) => fields
            .unnamed
            .iter()
            .nth(index.index as usize)
            .map(|field| &field.ty),
        _ => None,
    }
}

fn derive_trait_names(attrs: &[syn::Attribute]) -> BTreeSet<String> {
    let mut traits = BTreeSet::new();
    for attr in attrs {
        if !attr.path().is_ident("derive") {
            continue;
        }
        let parser = syn::punctuated::Punctuated::<Path, syn::Token![,]>::parse_terminated;
        let Ok(paths) = attr.parse_args_with(parser) else {
            continue;
        };
        traits.extend(
            paths
                .iter()
                .filter_map(|path| path.segments.last())
                .map(|segment| segment.ident.to_string()),
        );
    }
    traits
}

fn attrs_include_serde_default(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if !attr.path().is_ident("serde") {
            return false;
        }
        let mut idents = BTreeSet::new();
        collect_token_idents(&attr.to_token_stream(), &mut idents);
        idents.contains("default")
    })
}

fn attribute_can_expand_to_code(attr: &syn::Attribute) -> bool {
    let path = attr.path();
    !(path.is_ident("allow")
        || path.is_ident("cfg")
        || path.is_ident("deny")
        || path.is_ident("deprecated")
        || path.is_ident("doc")
        || path.is_ident("expect")
        || path.is_ident("must_use")
        || path.is_ident("repr"))
}

fn trait_path_is_conversion_like(trait_path: &[String]) -> bool {
    trait_path.last().is_some_and(|name| {
        matches!(
            name.as_str(),
            "From" | "Into" | "TryFrom" | "TryInto" | "FromStr"
        )
    })
}

fn trait_item_contains_method(project: &Project, item: &ItemId, method_name: &str) -> bool {
    let Some(record) = project.items.get(item) else {
        return false;
    };
    let Item::Trait(item_trait) = &record.item else {
        return false;
    };
    item_trait.items.iter().any(|trait_item| {
        let syn::TraitItem::Fn(function) = trait_item else {
            return false;
        };
        function.sig.ident == method_name
    })
}

fn trait_item_contains_default_method(project: &Project, item: &ItemId, method_name: &str) -> bool {
    let Some(record) = project.items.get(item) else {
        return false;
    };
    let Item::Trait(item_trait) = &record.item else {
        return false;
    };
    item_trait.items.iter().any(|trait_item| {
        let syn::TraitItem::Fn(function) = trait_item else {
            return false;
        };
        function.sig.ident == method_name && function.default.is_some()
    })
}

fn trait_item_has_public_surface(project: &Project, item: &ItemId) -> bool {
    let Some(record) = project.items.get(item) else {
        return false;
    };
    let Item::Trait(item_trait) = &record.item else {
        return false;
    };
    matches!(item_trait.vis, syn::Visibility::Public(_))
}

fn trait_item_for_impl(resolver: &Resolver<'_>, item_impl: &syn::ItemImpl) -> Option<ItemId> {
    let (_, trait_path, _) = item_impl.trait_.as_ref()?;
    let segments = resolver.apply_alias(path_segments(trait_path));
    resolver.resolve_trait_item(&segments)
}

fn type_ref_item_is_reachable(reachable_items: &BTreeSet<ItemId>, type_ref: &TypeRef) -> bool {
    let Some((name, module_path)) = type_ref.type_path.split_last() else {
        return false;
    };
    type_like_kinds().iter().any(|kind| {
        reachable_items.contains(&ItemId {
            package: type_ref.package.clone(),
            module_path: module_path.to_vec(),
            name: name.clone(),
            kind: *kind,
        })
    })
}

fn default_trait_method_traits_for_receiver(
    project: &Project,
    receiver: &TypeRef,
    method_name: &str,
) -> Vec<ItemId> {
    let mut matching_traits = Vec::new();
    for module_path in project_module_paths(project, &receiver.package) {
        let Some(items) = module_items_for_path(project, &receiver.package, &module_path) else {
            continue;
        };
        let aliases = project
            .module_aliases
            .get(&(receiver.package.clone(), module_path.clone()))
            .cloned()
            .unwrap_or_default();
        let resolver = Resolver {
            project,
            package: &receiver.package,
            module_path: &module_path,
            aliases: &aliases,
            self_type: None,
        };
        for item in items {
            let Item::Impl(item_impl) = item else {
                continue;
            };
            if item_impl.trait_.is_none() {
                continue;
            }
            let Some(self_type) = resolver.resolve_receiver_type(&item_impl.self_ty) else {
                continue;
            };
            if !resolver
                .type_ref_candidates(&self_type)
                .iter()
                .any(|candidate| candidate == receiver)
            {
                continue;
            }
            let Some(trait_item) = trait_item_for_impl(&resolver, item_impl) else {
                continue;
            };
            if trait_item_contains_default_method(project, &trait_item, method_name) {
                matching_traits.push(trait_item);
            }
        }
    }
    matching_traits.sort();
    matching_traits.dedup();
    matching_traits
}

fn impl_trait_return_bound_names(output: &ReturnType) -> BTreeSet<String> {
    let ReturnType::Type(_, ty) = output else {
        return BTreeSet::new();
    };
    let mut names = BTreeSet::new();
    collect_impl_trait_bound_names(ty, &mut names);
    names
}

fn collect_impl_trait_bound_names(ty: &Type, names: &mut BTreeSet<String>) {
    match ty {
        Type::ImplTrait(impl_trait) => {
            for bound in &impl_trait.bounds {
                let syn::TypeParamBound::Trait(trait_bound) = bound else {
                    continue;
                };
                if let Some(segment) = trait_bound.path.segments.last() {
                    names.insert(segment.ident.to_string());
                }
            }
        }
        Type::Path(type_path) => {
            for segment in &type_path.path.segments {
                let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
                    continue;
                };
                for argument in &arguments.args {
                    let GenericArgument::Type(ty) = argument else {
                        continue;
                    };
                    collect_impl_trait_bound_names(ty, names);
                }
            }
        }
        Type::Reference(reference) => collect_impl_trait_bound_names(&reference.elem, names),
        Type::Group(group) => collect_impl_trait_bound_names(&group.elem, names),
        Type::Paren(paren) => collect_impl_trait_bound_names(&paren.elem, names),
        _ => {}
    }
}

fn final_block_expression(block: &syn::Block) -> Option<&Expr> {
    match block.stmts.last()? {
        Stmt::Expr(expr, None) => Some(expr),
        _ => None,
    }
}

fn conversion_input_matches_receiver(input_path: &[String], receiver: &TypeRef) -> bool {
    input_path == receiver.type_path
        || input_path.ends_with(&receiver.type_path)
        || input_path.last().is_some_and(|input| {
            receiver
                .type_path
                .last()
                .is_some_and(|receiver| input == receiver)
        })
}

fn is_conversion_adapter_path(path: &Path, trait_name: &str, method_name: &str) -> bool {
    let segments = path_segments(path);
    matches!(
        segments.as_slice(),
        [trait_segment, method_segment]
            if trait_segment == trait_name && method_segment == method_name
    )
}

fn callable_method(callable: &CallableId) -> &str {
    match callable {
        CallableId::Method { method, .. } => method,
        CallableId::Free { .. } => "",
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

fn return_type_self_associated_name(output: &ReturnType) -> Option<String> {
    let ReturnType::Type(_, ty) = output else {
        return None;
    };
    type_self_associated_name(ty)
}

fn type_self_associated_name(ty: &Type) -> Option<String> {
    match ty {
        Type::Path(type_path) => {
            if let Some(qself) = &type_path.qself {
                if type_is_self(&qself.ty) {
                    return type_path
                        .path
                        .segments
                        .last()
                        .map(|segment| segment.ident.to_string());
                }
                return None;
            }
            let mut segments = type_path.path.segments.iter();
            let first = segments.next()?;
            let second = segments.next()?;
            if segments.next().is_none() && first.ident == "Self" {
                return Some(second.ident.to_string());
            }
            None
        }
        Type::Reference(reference) => type_self_associated_name(&reference.elem),
        Type::Group(group) => type_self_associated_name(&group.elem),
        Type::Paren(paren) => type_self_associated_name(&paren.elem),
        _ => None,
    }
}

fn type_is_self(ty: &Type) -> bool {
    let Type::Path(type_path) = ty else {
        return false;
    };
    type_path.qself.is_none()
        && type_path.path.segments.len() == 1
        && type_path
            .path
            .segments
            .first()
            .is_some_and(|segment| segment.ident == "Self")
}

fn binding_name_and_type(pattern: &Pat) -> Option<(String, Option<&Type>)> {
    match pattern {
        Pat::Ident(ident) => Some((ident.ident.to_string(), None)),
        Pat::Type(pat_type) => match pat_type.pat.as_ref() {
            Pat::Ident(ident) => Some((ident.ident.to_string(), Some(pat_type.ty.as_ref()))),
            _ => None,
        },
        _ => None,
    }
}

fn collect_pattern_ident_names(pattern: &Pat, names: &mut BTreeSet<String>) {
    match pattern {
        Pat::Ident(ident) => {
            names.insert(ident.ident.to_string());
        }
        Pat::Reference(reference) => collect_pattern_ident_names(&reference.pat, names),
        Pat::Type(pat_type) => collect_pattern_ident_names(&pat_type.pat, names),
        Pat::Tuple(tuple) => {
            for element in &tuple.elems {
                collect_pattern_ident_names(element, names);
            }
        }
        Pat::TupleStruct(tuple) => {
            for element in &tuple.elems {
                collect_pattern_ident_names(element, names);
            }
        }
        Pat::Struct(item_struct) => {
            for field in &item_struct.fields {
                collect_pattern_ident_names(&field.pat, names);
            }
        }
        Pat::Slice(slice) => {
            for element in &slice.elems {
                collect_pattern_ident_names(element, names);
            }
        }
        Pat::Or(or) => {
            for case in &or.cases {
                collect_pattern_ident_names(case, names);
            }
        }
        _ => {}
    }
}

fn token_path_candidates(tokens: &TokenStream) -> Vec<Vec<String>> {
    let mut candidates = Vec::new();
    collect_token_path_candidates(tokens, &mut candidates);
    candidates
}

fn string_literal_path_candidates(tokens: &TokenStream) -> Vec<Vec<String>> {
    let mut candidates = Vec::new();
    collect_string_literal_path_candidates(tokens, &mut candidates);
    candidates
}

fn serde_module_helper_paths(segments: &[String]) -> Vec<Path> {
    ["serialize", "deserialize"]
        .into_iter()
        .filter_map(|helper| {
            syn::parse_str::<Path>(&format!("{}::{helper}", segments.join("::"))).ok()
        })
        .collect()
}

fn collect_string_literal_path_candidates(tokens: &TokenStream, candidates: &mut Vec<Vec<String>>) {
    for token in tokens.clone() {
        match token {
            TokenTree::Literal(literal) => {
                if let Some(segments) = literal_path_segments(&literal) {
                    candidates.push(segments);
                }
            }
            TokenTree::Group(group) => {
                collect_string_literal_path_candidates(&group.stream(), candidates);
            }
            TokenTree::Ident(_) | TokenTree::Punct(_) => {}
        }
    }
}

fn literal_path_segments(literal: &Literal) -> Option<Vec<String>> {
    let literal = syn::parse2::<syn::LitStr>(literal.to_token_stream()).ok()?;
    let value = literal.value();
    if value.is_empty() || value.contains('/') {
        return None;
    }
    let path = syn::parse_str::<Path>(&value).ok()?;
    let segments = path_segments(&path);
    (!segments.is_empty()).then_some(segments)
}

fn format_literal_value(expression: &Expr) -> Option<String> {
    let Expr::Lit(literal) = expression else {
        return None;
    };
    let syn::Lit::Str(literal) = &literal.lit else {
        return None;
    };
    Some(literal.value())
}

fn format_string_capture_idents(format: &str) -> BTreeSet<String> {
    let mut captures = BTreeSet::new();
    let mut chars = format.char_indices().peekable();
    while let Some((_, ch)) = chars.next() {
        if ch != '{' {
            continue;
        }
        if chars.peek().is_some_and(|(_, next)| *next == '{') {
            chars.next();
            continue;
        }

        let mut capture = String::new();
        while let Some((_, next)) = chars.peek().copied() {
            if next == '}' || next == ':' || next == '!' || next == '.' || next == '[' {
                break;
            }
            capture.push(next);
            chars.next();
        }

        if syn::parse_str::<syn::Ident>(&capture).is_ok() {
            captures.insert(capture);
        }
    }
    captures
}

fn macro_token_idents(tokens: &TokenStream) -> BTreeSet<String> {
    let mut idents = BTreeSet::new();
    collect_macro_token_idents(tokens, &mut idents);
    idents
}

fn macro_definition_generated_item_idents(
    definition_tokens: &TokenStream,
    invocation_tokens: &TokenStream,
) -> BTreeSet<String> {
    let invocation_args = macro_invocation_arg_map(definition_tokens, invocation_tokens);
    let mut idents = BTreeSet::new();
    collect_macro_definition_generated_item_idents(
        definition_tokens,
        &invocation_args,
        &mut idents,
    );
    idents
}

fn macro_invocation_arg_map(
    definition_tokens: &TokenStream,
    invocation_tokens: &TokenStream,
) -> HashMap<String, String> {
    macro_definition_metavariables(definition_tokens)
        .into_iter()
        .zip(macro_invocation_arg_idents(invocation_tokens))
        .collect()
}

fn macro_definition_metavariables(tokens: &TokenStream) -> Vec<String> {
    let mut names = Vec::new();
    collect_macro_definition_metavariables(tokens, &mut names);
    names
}

fn collect_macro_definition_metavariables(tokens: &TokenStream, names: &mut Vec<String>) {
    let mut skip_metavariable = false;
    for token in tokens.clone() {
        match token {
            TokenTree::Ident(ident) if skip_metavariable => {
                let ident = ident.to_string();
                if !names.contains(&ident) {
                    names.push(ident);
                }
                skip_metavariable = false;
            }
            TokenTree::Punct(punct) if punct.as_char() == '$' => {
                skip_metavariable = true;
            }
            TokenTree::Group(group) => {
                skip_metavariable = false;
                collect_macro_definition_metavariables(&group.stream(), names);
            }
            TokenTree::Ident(_) | TokenTree::Punct(_) | TokenTree::Literal(_) => {
                skip_metavariable = false;
            }
        }
    }
}

fn macro_invocation_arg_idents(tokens: &TokenStream) -> Vec<String> {
    tokens
        .clone()
        .into_iter()
        .filter_map(|token| match token {
            TokenTree::Ident(ident) => Some(ident.to_string()),
            _ => None,
        })
        .collect()
}

fn collect_macro_definition_generated_item_idents(
    tokens: &TokenStream,
    invocation_args: &HashMap<String, String>,
    idents: &mut BTreeSet<String>,
) {
    let mut item_name_expected = false;
    let mut metavariable_item_name = false;
    let mut impl_header = false;
    for token in tokens.clone() {
        match token {
            TokenTree::Ident(ident) if metavariable_item_name => {
                if let Some(arg) = invocation_args.get(&ident.to_string()) {
                    idents.insert(arg.clone());
                }
                item_name_expected = false;
                metavariable_item_name = false;
            }
            TokenTree::Ident(ident) => {
                let ident = ident.to_string();
                let starts_item_name = macro_generated_item_keyword(&ident);
                let starts_impl_header = ident == "impl";
                if item_name_expected && !macro_definition_noise_ident(&ident) {
                    idents.insert(ident);
                }
                if starts_impl_header {
                    impl_header = true;
                }
                item_name_expected = starts_item_name;
                metavariable_item_name = false;
            }
            TokenTree::Punct(punct) if punct.as_char() == '$' && item_name_expected => {
                metavariable_item_name = true;
            }
            TokenTree::Punct(_) | TokenTree::Literal(_) => {
                metavariable_item_name = false;
            }
            TokenTree::Group(group) => {
                item_name_expected = false;
                metavariable_item_name = false;
                if impl_header {
                    impl_header = false;
                } else {
                    collect_macro_definition_generated_item_idents(
                        &group.stream(),
                        invocation_args,
                        idents,
                    );
                }
            }
        }
    }
}

fn macro_generated_item_keyword(ident: &str) -> bool {
    matches!(
        ident,
        "const" | "enum" | "fn" | "mod" | "static" | "struct" | "trait" | "type" | "union"
    )
}

fn macro_definition_noise_ident(ident: &str) -> bool {
    macro_pattern_keyword(&[ident.to_string()])
        || matches!(
            ident,
            "block"
                | "expr"
                | "ident"
                | "item"
                | "lifetime"
                | "literal"
                | "meta"
                | "pat"
                | "pat_param"
                | "path"
                | "stmt"
                | "tt"
                | "ty"
                | "vis"
        )
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

fn token_stream_mentions_ident(tokens: &TokenStream, ident: &str) -> bool {
    let mut idents = BTreeSet::new();
    collect_token_idents(tokens, &mut idents);
    idents.contains(ident)
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
    )
}

fn macro_path_starts_with(path: &Path, name: &str) -> bool {
    path.segments
        .first()
        .is_some_and(|segment| segment.ident == name)
}

fn macro_path_ends_with(path: &Path, name: &str) -> bool {
    path.segments
        .last()
        .is_some_and(|segment| segment.ident == name)
}

fn attribute_uses_serde_helper_paths(path: &Path) -> bool {
    path.segments.last().is_some_and(|segment| {
        let ident = segment.ident.to_string();
        ident == "serde" || ident.ends_with("_serde")
    })
}

fn collect_token_path_candidates(tokens: &TokenStream, candidates: &mut Vec<Vec<String>>) {
    let token_trees = tokens.clone().into_iter().collect::<Vec<_>>();
    for token in &token_trees {
        if let TokenTree::Group(group) = token {
            collect_token_path_candidates(&group.stream(), candidates);
        }
    }

    let mut index = 0;
    while index < token_trees.len() {
        let TokenTree::Ident(ident) = &token_trees[index] else {
            index += 1;
            continue;
        };

        let mut segments = vec![ident.to_string()];
        let mut cursor = index + 1;
        while has_path_separator(&token_trees, cursor) {
            let Some(TokenTree::Ident(next)) = token_trees.get(cursor + 2) else {
                break;
            };
            segments.push(next.to_string());
            cursor += 3;
        }

        if !macro_pattern_keyword(&segments) {
            candidates.push(segments.clone());
            if let Some(last) = segments.last() {
                candidates.push(vec![last.clone()]);
            }
        }

        index = cursor.max(index + 1);
    }
}

fn has_path_separator(tokens: &[TokenTree], index: usize) -> bool {
    matches!(tokens.get(index), Some(TokenTree::Punct(punct)) if punct.as_char() == ':')
        && matches!(tokens.get(index + 1), Some(TokenTree::Punct(punct)) if punct.as_char() == ':')
}

fn macro_pattern_keyword(segments: &[String]) -> bool {
    matches!(
        segments,
        [single]
            if matches!(
                single.as_str(),
                "block"
                    | "expr"
                    | "ident"
                    | "item"
                    | "literal"
                    | "meta"
                    | "pat"
                    | "path"
                    | "stmt"
                    | "tt"
                    | "ty"
                    | "vis"
            )
    )
}

fn dependency_name_matches(dependency: &crate::manifest::Dependency, name: &str) -> bool {
    dependency.alias == name
        || dependency.package == name
        || crate_code_name(&dependency.alias) == name
        || crate_code_name(&dependency.package) == name
}

fn crate_code_name(name: &str) -> String {
    name.replace('-', "_")
}
