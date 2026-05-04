use std::{
    collections::{BTreeMap, BTreeSet, HashMap, VecDeque},
    path::{Path as FsPath, PathBuf},
};

use proc_macro2::{Literal, TokenStream, TokenTree};
use quote::ToTokens;
use syn::{
    parse::Parser,
    visit::{self, Visit},
    Expr, ExprCall, ExprMacro, ExprMatch, ExprMethodCall, ExprPath, ExprStruct, FnArg,
    GenericArgument, ImplItem, Item, ItemMacro, Local, Macro, Member, Meta, Pat, PatTupleStruct,
    Path, PathArguments, ReturnType, Stmt, Type, TypePath, UseTree,
};
use toml::Value;

use crate::model::{
    CallableId, ItemId, ItemKind, Project, ReducedProject, ReductionEvidence, RootId,
    SemanticDependencies, SemanticReductionHints,
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
    let proc_macro_dependency_packages =
        add_local_proc_macro_dependency_packages(project, &mut packages);
    retain_entire_packages(
        project,
        &proc_macro_dependency_packages,
        &mut reachable,
        &mut reachable_items,
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
        for item in &source.syntax.items {
            let syn::Item::Macro(item_macro) = item else {
                continue;
            };
            if item_macro.ident.is_some()
                || !should_scan_item_macro_dependencies(
                    project,
                    reachable,
                    reachable_items,
                    &source.package,
                    item_macro,
                )
            {
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
            let mut visitor = DependencyVisitor::new(resolver);
            visitor.add_macro_path(&item_macro.mac.path);
            visitor.add_macro_token_dependencies(&item_macro.mac.tokens);
            dependencies.extend(visitor.dependencies);
        }
    }
    dependencies
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
            collect_token_idents(&record.item.to_token_stream(), &mut idents);
        }
    }
    idents
}

fn reachable_package_source_idents(
    project: &Project,
    reachable: &BTreeSet<CallableId>,
    reachable_items: &BTreeSet<ItemId>,
    package: &str,
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
            collect_token_idents(&record.item.to_token_stream(), &mut idents);
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
    cache: &mut HashMap<(String, Vec<String>), BTreeSet<String>>,
) -> BTreeSet<String> {
    let key = (package.to_string(), module_path.to_vec());
    if let Some(idents) = cache.get(&key) {
        return idents.clone();
    }

    let mut idents =
        reachable_source_idents(project, reachable, reachable_items, package, module_path);
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

fn item_macro_feeds_reachable_code(
    project: &Project,
    reachable: &BTreeSet<CallableId>,
    reachable_items: &BTreeSet<ItemId>,
    package: &str,
    item_macro: &ItemMacro,
) -> bool {
    macro_token_idents(&item_macro.mac.tokens)
        .iter()
        .any(|ident| {
            reachable_package_mentions_ident(project, reachable, reachable_items, package, ident)
        })
}

fn should_scan_item_macro_dependencies(
    project: &Project,
    reachable: &BTreeSet<CallableId>,
    reachable_items: &BTreeSet<ItemId>,
    package: &str,
    item_macro: &ItemMacro,
) -> bool {
    if macro_path_starts_with(&item_macro.mac.path, "uniffi")
        && !reachable_package_mentions_ident(project, reachable, reachable_items, package, "uniffi")
    {
        return false;
    }

    item_macro_feeds_reachable_code(project, reachable, reachable_items, package, item_macro)
}

fn add_source_mentioned_dependency_packages(
    project: &Project,
    packages: &mut BTreeSet<String>,
    reachable: &BTreeSet<CallableId>,
    reachable_items: &BTreeSet<ItemId>,
) {
    let mut changed = true;
    while changed {
        changed = false;
        for package_name in packages.clone() {
            let Some(package) = project.workspace.packages.get(&package_name) else {
                continue;
            };
            let idents =
                reachable_package_idents(project, reachable, reachable_items, &package_name);
            if idents.is_empty() {
                continue;
            }

            for dependency in &package.dependencies {
                if dependency.package != "opensourced"
                    && project.workspace.packages.contains_key(&dependency.package)
                    && source_mentions_dependency(project, &package_name, &idents, dependency)
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

fn add_local_proc_macro_dependency_packages(
    project: &Project,
    packages: &mut BTreeSet<String>,
) -> BTreeSet<String> {
    let mut retained_proc_macro_dependencies = BTreeSet::new();
    let mut changed = true;
    while changed {
        changed = false;
        for package_name in packages.clone() {
            let Some(package) = project.workspace.packages.get(&package_name) else {
                continue;
            };
            if !package_is_proc_macro(package) {
                continue;
            }

            for package in package_closure(project, &package_name) {
                retained_proc_macro_dependencies.insert(package.clone());
                changed |= packages.insert(package);
            }
        }
    }
    retained_proc_macro_dependencies
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
    if names.iter().any(|name| idents.contains(*name)) {
        return true;
    }

    project
        .module_aliases
        .iter()
        .filter(|((alias_package, _), _)| alias_package == package)
        .flat_map(|(_, aliases)| aliases.iter())
        .any(|(alias, target)| {
            idents.contains(alias)
                && target
                    .first()
                    .is_some_and(|first| names.iter().any(|name| first == name))
        })
}

fn reachable_package_idents(
    project: &Project,
    reachable: &BTreeSet<CallableId>,
    reachable_items: &BTreeSet<ItemId>,
    package: &str,
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
        .filter(|item| item.package() == package)
    {
        if let Some(record) = project.items.get(item) {
            collect_token_idents(&record.item.to_token_stream(), &mut idents);
        }
    }
    idents
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
            visitor.dependencies
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
            visitor.dependencies
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
    visitor.visit_item(&record.item);
    let mut dependencies = visitor.dependencies;
    if item_has_opensourced_attr(&record.item) {
        dependencies.extend(item_root_macro_impl_dependencies(project, item));
    }
    dependencies.items.remove(item);
    dependencies
}

fn item_root_macro_impl_dependencies(project: &Project, item: &ItemId) -> DependencySet {
    let mut dependencies = DependencySet::default();
    let item_path = path_from_item(item);

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
            for impl_item in &item_impl.items {
                if root_macro_impl_item_should_render(item_impl, impl_item) {
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

struct DependencyVisitor<'a> {
    resolver: Resolver<'a>,
    dependencies: DependencySet,
    generic_trait_bounds: HashMap<String, Vec<ItemId>>,
    variable_trait_bounds: HashMap<String, Vec<ItemId>>,
    variables: HashMap<String, TypeRef>,
    variable_candidates: HashMap<String, Vec<TypeRef>>,
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
            variable_trait_bounds: HashMap::new(),
            variables: HashMap::new(),
            variable_candidates: HashMap::new(),
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
            let trait_items = self.trait_items_from_bounds(&type_parameter.bounds);
            self.insert_generic_trait_bounds(type_parameter.ident.to_string(), trait_items);
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
            self.insert_generic_trait_bounds(name, trait_items);
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
            let Pat::Ident(ident) = input.pat.as_ref() else {
                continue;
            };
            let trait_items = self.trait_items_from_type(&input.ty);
            self.insert_variable_trait_bounds(ident.ident.to_string(), trait_items);
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

    fn insert_variable_trait_bounds(&mut self, name: String, mut trait_items: Vec<ItemId>) {
        if trait_items.is_empty() {
            return;
        }
        let bounds = self.variable_trait_bounds.entry(name).or_default();
        bounds.append(&mut trait_items);
        bounds.sort();
        bounds.dedup();
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
            Expr::Call(call) => self.wrapper_constructor_arg_type(call).or_else(|| {
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
            Expr::Field(field) => {
                let receiver = self.receiver_type(&field.base)?;
                self.resolver.field_type(&receiver, &field.member)
            }
            Expr::Struct(expr) => self.resolver.resolve_type_path(&expr.path),
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
                }
            }
            Expr::Call(call) => {
                if let Some(type_ref) = self.wrapper_constructor_arg_type(call).or_else(|| {
                    if let Expr::Path(path) = call.func.as_ref() {
                        self.resolver.type_from_expr_path_call(path)
                    } else {
                        None
                    }
                }) {
                    candidates.extend(self.resolver.type_ref_candidates(&type_ref));
                }
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

    fn local_binding_trait_bounds(&self, local: &Local) -> Option<(String, Vec<ItemId>)> {
        let (name, explicit_type) = binding_name_and_type(&local.pat)?;
        let trait_items = explicit_type
            .map(|ty| self.trait_items_from_type(ty))
            .unwrap_or_default();
        (!trait_items.is_empty()).then_some((name, trait_items))
    }

    fn infer_expr_type(&self, expression: &Expr) -> Option<TypeRef> {
        match expression {
            Expr::Call(call) => self.wrapper_constructor_arg_type(call).or_else(|| {
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
        if matches!(call.method.to_string().as_str(), "as_ref" | "clone") {
            return self.receiver_type(&call.receiver);
        }
        if call.method == "lock" && call.args.is_empty() {
            return self.receiver_type(&call.receiver);
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
        self.receiver_trait_bounds(receiver)
            .into_iter()
            .filter(|trait_item| {
                trait_item_contains_method(self.resolver.project, trait_item, method_name)
            })
            .find_map(|trait_item| {
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
        let Pat::TupleStruct(tuple) = pattern else {
            return;
        };
        if tuple
            .path
            .segments
            .last()
            .is_none_or(|segment| segment.ident != "Ok")
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
            Pat::TupleStruct(tuple) => {
                let Some(variant_name) = tuple
                    .path
                    .segments
                    .last()
                    .map(|segment| segment.ident.to_string())
                else {
                    return;
                };
                let field_types = self
                    .resolver
                    .enum_tuple_variant_field_types(type_ref, &variant_name);
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
                    if let Some(field_type) = self.resolver.enum_named_variant_field_type(
                        type_ref,
                        &variant_name,
                        &field.member,
                    ) {
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

    fn add_unresolved_method_fallback(&mut self, method_name: &str) {
        self.add_unresolved_method_candidates(method_name, &[]);
    }

    fn add_unresolved_method_candidates(
        &mut self,
        method_name: &str,
        receiver_candidates: &[TypeRef],
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
                    trait_path: Some(trait_path),
                    method,
                    ..
                } = callable
                else {
                    return None;
                };
                Some((package.clone(), trait_path.clone(), method.clone()))
            })
            .collect::<BTreeSet<_>>();

        if resolved_traits.is_empty() {
            return;
        }

        for callable in self.resolver.project.methods.keys() {
            let CallableId::Method {
                package,
                trait_path: Some(trait_path),
                method,
                ..
            } = callable
            else {
                continue;
            };
            if resolved_traits.contains(&(package.clone(), trait_path.clone(), method.clone())) {
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
        if !macro_path_ends_with(&mac.path, "assert")
            && !macro_path_ends_with(&mac.path, "assert_eq")
            && !macro_path_ends_with(&mac.path, "assert_ne")
            && !macro_path_ends_with(&mac.path, "debug_assert")
            && !macro_path_ends_with(&mac.path, "debug_assert_eq")
            && !macro_path_ends_with(&mac.path, "debug_assert_ne")
        {
            return;
        }

        let parser = syn::punctuated::Punctuated::<Expr, syn::Token![,]>::parse_terminated;
        let Ok(arguments) = parser.parse2(mac.tokens.clone()) else {
            return;
        };
        for argument in arguments {
            self.visit_expr(&argument);
        }
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
        let derive_traits = derive_trait_names(attrs);
        if derive_traits.is_empty() {
            return;
        }

        for field in fields.iter() {
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

    fn add_generic_field_type_trait_dependencies(&mut self, fields: &syn::Fields) {
        for field in fields.iter() {
            for type_ref in self.resolver.type_argument_refs_in_type(&field.ty) {
                self.add_non_conversion_trait_impls_for_type(&type_ref);
            }
        }
    }

    fn add_serde_default_field_dependencies(&mut self, fields: &syn::Fields) {
        for field in fields.iter() {
            if !attrs_include_serde_default(&field.attrs) {
                continue;
            }
            for type_ref in self.resolver.type_refs_in_type(&field.ty) {
                self.add_trait_impls_for_type_named(&type_ref, "Default");
            }
        }
    }

    fn add_parse_method_trait_dependencies(&mut self) {
        for type_ref in self.expected_parse_types.clone() {
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
            Expr::Reference(reference) => self.expression_type_arguments(&reference.expr),
            Expr::Paren(paren) => self.expression_type_arguments(&paren.expr),
            _ => Vec::new(),
        }
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
            if input_types.is_empty() {
                continue;
            }

            let variables = self.variables.clone();
            let variable_candidates = self.variable_candidates.clone();
            for (pattern, type_ref) in closure.inputs.iter().zip(input_types.iter()) {
                self.bind_pattern_type(pattern, type_ref);
            }
            self.visit_expr(&closure.body);
            self.variables = variables;
            self.variable_candidates = variable_candidates;
        }
    }

    fn visit_fold_method_call(&mut self, call: &ExprMethodCall) -> bool {
        if call.method != "fold" || call.args.len() != 2 {
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

        let variables = self.variables.clone();
        let variable_candidates = self.variable_candidates.clone();
        self.bind_pattern_type(accumulator_pat, &accumulator_type);
        for input in closure.inputs.iter().skip(1) {
            self.visit_pat(input);
        }
        self.visit_expr(&closure.body);
        self.variables = variables;
        self.variable_candidates = variable_candidates;
        true
    }

    fn visit_single_payload_closure_method_call(&mut self, call: &ExprMethodCall) -> bool {
        if !matches!(
            call.method.to_string().as_str(),
            "is_some_and" | "is_ok_and" | "is_err_and"
        ) || call.args.len() != 1
        {
            return false;
        }
        let Some(payload_type) = self.receiver_type(&call.receiver) else {
            return false;
        };
        let Some(Expr::Closure(closure)) = call.args.first() else {
            return false;
        };
        let Some(payload_pat) = closure.inputs.first() else {
            return false;
        };

        self.visit_expr(&call.receiver);
        let variables = self.variables.clone();
        let variable_candidates = self.variable_candidates.clone();
        self.bind_pattern_type(payload_pat, &payload_type);
        self.visit_expr(&closure.body);
        self.variables = variables;
        self.variable_candidates = variable_candidates;
        true
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
        }
    }

    fn visit_local(&mut self, local: &'ast Local) {
        if let Some((name, trait_items)) = self.local_binding_trait_bounds(local) {
            self.insert_variable_trait_bounds(name, trait_items);
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
            self.add_external_call_arg_trait_impls(call);
            if resolved_callables.is_empty() && path.path.segments.len() >= 2 {
                if let Some(method) = path.path.segments.last() {
                    self.add_unresolved_method_fallback(&method.ident.to_string());
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
        let ok_type = self.result_ok_type(&expr_match.expr);
        let match_type = self
            .receiver_type(&expr_match.expr)
            .or_else(|| self.infer_expr_type(&expr_match.expr));

        for arm in &expr_match.arms {
            let variables = self.variables.clone();
            let variable_candidates = self.variable_candidates.clone();
            if let Some(ok_type) = &ok_type {
                self.add_result_ok_binding_type(&arm.pat, ok_type);
            }
            if let Some(match_type) = &match_type {
                self.add_pattern_bindings_for_type(&arm.pat, match_type);
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
        }
    }

    fn visit_expr_if(&mut self, expr_if: &'ast syn::ExprIf) {
        let outer_variables = self.variables.clone();
        let outer_variable_candidates = self.variable_candidates.clone();
        if let Expr::Let(expr_let) = expr_if.cond.as_ref() {
            let type_arguments = self.expression_type_arguments(&expr_let.expr);
            if let [type_ref] = type_arguments.as_slice() {
                self.bind_single_payload_pattern(&expr_let.pat, type_ref);
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

        if let Some((_, else_branch)) = &expr_if.else_branch {
            self.visit_expr(else_branch);
        }
        self.variables = outer_variables;
        self.variable_candidates = outer_variable_candidates;
    }

    fn visit_expr_while(&mut self, expr_while: &'ast syn::ExprWhile) {
        if let Expr::Let(expr_let) = expr_while.cond.as_ref() {
            let type_arguments = self.expression_type_arguments(&expr_let.expr);
            if let [type_ref] = type_arguments.as_slice() {
                self.bind_single_payload_pattern(&expr_let.pat, type_ref);
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
        self.add_rusqlite_param_macro_trait_dependencies(mac);
        self.add_macro_token_dependencies(&mac.tokens);
        visit::visit_macro(self, mac);
    }

    fn visit_expr_try(&mut self, expr: &'ast syn::ExprTry) {
        for type_ref in self.expected_error_types.clone() {
            self.add_trait_impls_for_type_named(&type_ref, "From");
        }
        visit::visit_expr_try(self, expr);
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        if self.visit_fold_method_call(call) {
            return;
        }
        if self.visit_single_payload_closure_method_call(call) {
            return;
        }
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
            if has_resolved_method {
                self.add_peer_trait_impls_for_resolved_methods(&resolved_methods);
                self.add_closure_arg_dependencies(call, &resolved_methods);
            }
            if !has_resolved_method && !has_trait_bound_method {
                self.add_unresolved_method_candidates(&method, &receiver_candidates);
                self.add_trait_impls_for_type_named(&receiver, "Deref");
                self.add_trait_impls_for_type_named(&receiver, "DerefMut");
                self.add_extension_trait_dependencies_for_method(&receiver, &method);
                for candidate in &receiver_candidates {
                    self.add_trait_impls_for_type_named(candidate, "Deref");
                    self.add_trait_impls_for_type_named(candidate, "DerefMut");
                    self.add_extension_trait_dependencies_for_method(candidate, &method);
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
            self.add_conversion_impls_by_trait("From", "from");
        } else if call.method == "try_into" {
            self.add_conversion_impls_by_trait("TryFrom", "try_from");
        } else if call.method == "parse" {
            self.add_parse_method_trait_dependencies();
        } else if call.method == "collect" {
            self.add_collect_method_trait_dependencies();
        } else {
            let method = call.method.to_string();
            if !self.add_trait_bound_method_dependencies(&call.receiver, &method) {
                self.add_unresolved_method_candidates(&method, &receiver_candidates);
            }
            self.add_external_method_arg_trait_impls(call);
        }
        visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_struct(&mut self, expr: &'ast ExprStruct) {
        self.add_item_path(&expr.path);
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

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct TypeRef {
    package: String,
    type_path: Vec<String>,
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
        let id = CallableId::Free {
            package: package.clone(),
            module_path: module_path.clone(),
            name: name.clone(),
        };

        if self.project.functions.contains_key(&id) {
            return Some(id);
        }

        if let Some(alias) = self.resolve_alias_target(&package, &module_path, &name) {
            let id = CallableId::Free {
                package: alias.package,
                module_path: alias.module_path,
                name: alias.name,
            };
            if self.project.functions.contains_key(&id) {
                return Some(id);
            }
        }

        self.find_glob_reexport_function(&package, &module_path, &name, &mut BTreeSet::new())
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
            let inherent = CallableId::Method {
                package: candidate.package.clone(),
                type_path: candidate.type_path.clone(),
                trait_path: None,
                trait_input_type_paths: Vec::new(),
                method: method.to_string(),
            };
            if self.project.methods.contains_key(&inherent) {
                matches.push(inherent);
            }

            matches.extend(self.project.methods.keys().filter_map(|id| {
                let CallableId::Method {
                    package,
                    type_path,
                    trait_path: Some(_),
                    method: candidate_method,
                    ..
                } = id
                else {
                    return None;
                };

                (package == &candidate.package
                    && type_path == &candidate.type_path
                    && candidate_method == method)
                    .then(|| id.clone())
            }));
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
        type_refs.sort();
        type_refs.dedup();
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

    fn type_from_return_type(&self, output: &ReturnType) -> Option<TypeRef> {
        let ReturnType::Type(_, ty) = output else {
            return None;
        };
        self.resolve_receiver_type(ty)
    }

    fn error_type_from_return_type(&self, output: &ReturnType) -> Option<TypeRef> {
        let ReturnType::Type(_, ty) = output else {
            return None;
        };
        self.result_error_type(ty)
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
        if let Type::Path(type_path) = ty {
            if let Some(type_ref) = self.resolve_single_type_argument(&type_path.path) {
                return Some(type_ref);
            }
        }
        if let Some(type_ref) = self.resolve_type(ty) {
            return Some(type_ref);
        }

        match ty {
            Type::Reference(reference) => self.resolve_receiver_type(&reference.elem),
            Type::Group(group) => self.resolve_receiver_type(&group.elem),
            Type::Paren(paren) => self.resolve_receiver_type(&paren.elem),
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
        let mut type_refs = Vec::new();
        self.collect_type_arguments(ty, &mut type_refs);
        type_refs.sort();
        type_refs.dedup();
        type_refs
    }

    fn type_refs_in_type(&self, ty: &Type) -> Vec<TypeRef> {
        self.receiver_type_candidates_from_type(ty)
    }

    fn type_argument_refs_in_type(&self, ty: &Type) -> Vec<TypeRef> {
        let mut type_refs = Vec::new();
        self.collect_type_arguments(ty, &mut type_refs);
        type_refs.sort();
        type_refs.dedup();
        type_refs
    }

    fn collect_type_arguments(&self, ty: &Type, type_refs: &mut Vec<TypeRef>) {
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
            .methods
            .keys()
            .any(|id| {
                matches!(
                    id,
                    CallableId::Method {
                        package,
                        type_path,
                        ..
                    } if package == &candidate.package && type_path == &candidate.type_path
                )
            })
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
            .methods
            .keys()
            .any(|id| {
                matches!(
                    id,
                    CallableId::Method {
                        package,
                        type_path,
                        ..
                    } if package == &candidate.package && type_path == &candidate.type_path
                )
            })
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
        self.project
            .items
            .keys()
            .find(|item| {
                item.package == package && item.kind == ItemKind::Macro && item.name == *name
            })
            .cloned()
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

        let source = self
            .project
            .files
            .values()
            .find(|source| source.package == package && source.module_path == module_path)?;
        for glob_path in visible_glob_use_paths(&source.syntax.items) {
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

        let source = self
            .project
            .files
            .values()
            .find(|source| source.package == package && source.module_path == module_path)?;
        for glob_path in visible_glob_use_paths(&source.syntax.items) {
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

fn generic_parameter_name_from_type(ty: &Type) -> Option<String> {
    match ty {
        Type::Path(type_path) => single_segment_type_name(&type_path.path),
        Type::Reference(reference) => generic_parameter_name_from_type(&reference.elem),
        Type::Group(group) => generic_parameter_name_from_type(&group.elem),
        Type::Paren(paren) => generic_parameter_name_from_type(&paren.elem),
        _ => None,
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
            | "String"
            | "Vec"
            | "Result"
            | "Option"
            | "Box"
            | "Ok"
            | "Err"
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
