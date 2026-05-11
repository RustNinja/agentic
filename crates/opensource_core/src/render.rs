use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, HashMap},
    env, fs,
    path::{Component, Path, PathBuf},
};

use proc_macro2::{Delimiter, TokenStream, TokenTree};
use quote::ToTokens;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::visit::{self, Visit};
use syn::visit_mut::{self, VisitMut};
use syn::{
    parse_quote, Block, Expr, ExprStruct, Field, FieldValue, Fields, ForeignItem, GenericArgument,
    ImplItem, Item, ItemMod, Lit, Member, Meta, Pat, PathArguments, TraitItem, Type, UseTree,
    Variant,
};
use toml::{value::Table, Value};

use crate::{
    include_path::{static_include_path, StaticIncludePath},
    manifest::Package,
    model::{CallableId, ItemId, ItemKind, Project, ReducedProject, RootId, SourceFile},
    reduce::{is_cfg_test_attr, is_opensourced_attr, is_test_attr},
    UsageDecisionIndex,
};

const OUTPUT_MARKER: &str = ".slicers-output";
const SUPPORT_PACKAGE_DIR: &str = "support";

#[derive(Debug, Clone, Default)]
pub(crate) struct RenderedMemberDecisionIndex {
    pub(crate) retained: BTreeSet<String>,
    pub(crate) blocked_by_unknown: BTreeSet<String>,
    pub(crate) prunable: BTreeSet<String>,
}

struct RenderPlan {
    usage: UsageDecisionIndex,
    reachable_items: BTreeSet<ItemId>,
    callable_idents_by_package: BTreeMap<String, BTreeSet<String>>,
    mentions: ReachableMentionIndex,
    import_scope_mentions: RefCell<BTreeMap<ImportScopeMentionKey, bool>>,
    import_scope_uses: RefCell<BTreeMap<ImportScopeMentionKey, bool>>,
}

impl RenderPlan {
    fn build(project: &Project, reduced: &ReducedProject, usage: &UsageDecisionIndex) -> Self {
        let mut reachable_items = BTreeSet::new();
        let mut rendered_item_idents = BTreeSet::new();
        let callable_idents = reachable_reduced_callable_ident_index(project, reduced);
        let reachable_token_idents = ReachableTokenIdentIndex::build(project, reduced);
        let retained_surface_idents =
            retained_surface_idents_by_package(project, reduced, &reachable_token_idents);
        let referenced_reexport_target_items =
            referenced_public_reexport_target_items(project, reduced, &reachable_token_idents);

        for item in &reduced.reachable_items {
            if root_item_should_render(reduced, item)
                || !package_has_reachable_callables(reduced, &item.package)
                || matches!(item.kind, ItemKind::Const | ItemKind::Static)
                || reachable_trait_surface_should_render(project, reduced, item)
                || referenced_reexport_target_items.contains(item)
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
        for item in &referenced_reexport_target_items {
            insert_render_plan_item(
                project,
                reduced,
                &mut reachable_items,
                &mut rendered_item_idents,
                item,
            );
        }
        for item in usage.blocked_by_unknown_items() {
            insert_render_plan_item(
                project,
                reduced,
                &mut reachable_items,
                &mut rendered_item_idents,
                &item,
            );
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
            usage: usage.clone(),
            mentions: ReachableMentionIndex::build(
                project,
                reduced,
                &reachable_items,
                &reachable_token_idents,
            ),
            reachable_items,
            callable_idents_by_package: callable_idents.by_package,
            import_scope_mentions: RefCell::new(BTreeMap::new()),
            import_scope_uses: RefCell::new(BTreeMap::new()),
        }
    }

    fn callable_should_render(&self, callable: &CallableId) -> bool {
        self.usage.should_render_callable(callable)
    }

    fn item_should_render(&self, item: &ItemId) -> bool {
        self.reachable_items.contains(item) || self.usage.is_blocked_by_unknown_item(item)
    }

    fn can_remove_callable(&self, callable: &CallableId) -> bool {
        self.usage.can_remove_callable(callable)
    }

    fn can_remove_item(&self, item: &ItemId) -> bool {
        !self.item_should_render(item)
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

pub(crate) fn rendered_member_decision_index(
    project: &Project,
    reduced: &ReducedProject,
    usage: &UsageDecisionIndex,
) -> RenderedMemberDecisionIndex {
    let render_plan = RenderPlan::build(project, reduced, usage);
    let mut decisions = RenderedMemberDecisionIndex::default();

    for (item_id, record) in &project.items {
        if !reduced.packages.contains(&item_id.package) || !render_plan.item_should_render(item_id)
        {
            continue;
        }
        match &record.item {
            Item::Struct(item_struct) => record_struct_member_decisions(
                project,
                reduced,
                &render_plan,
                usage,
                item_id,
                item_struct,
                &mut decisions,
            ),
            Item::Enum(item_enum) => record_enum_member_decisions(
                project,
                reduced,
                usage,
                item_id,
                item_enum,
                &mut decisions,
            ),
            _ => {}
        }
    }

    decisions
}

fn record_struct_member_decisions(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    usage: &UsageDecisionIndex,
    item_id: &ItemId,
    item_struct: &syn::ItemStruct,
    decisions: &mut RenderedMemberDecisionIndex,
) {
    let parent_is_blocked = usage.is_blocked_by_unknown_item(item_id);
    match &item_struct.fields {
        Fields::Named(fields) => {
            for field in &fields.named {
                let Some(name) = field.ident.as_ref().map(ToString::to_string) else {
                    continue;
                };
                let member = rendered_member_symbol_path(item_id, &name);
                let should_remain = root_item_should_render(reduced, item_id)
                    || struct_field_should_remain(
                        project,
                        reduced,
                        Some(render_plan),
                        &item_id.package,
                        &item_id.module_path,
                        item_struct,
                        field,
                    );
                record_rendered_member_decision(
                    decisions,
                    member,
                    should_remain,
                    parent_is_blocked,
                );
            }
        }
        Fields::Unnamed(fields) => {
            for (index, _field) in fields.unnamed.iter().enumerate() {
                let member = rendered_member_symbol_path(item_id, &index.to_string());
                record_rendered_member_decision(decisions, member, true, parent_is_blocked);
            }
        }
        Fields::Unit => {}
    }
}

fn record_enum_member_decisions(
    project: &Project,
    reduced: &ReducedProject,
    usage: &UsageDecisionIndex,
    item_id: &ItemId,
    item_enum: &syn::ItemEnum,
    decisions: &mut RenderedMemberDecisionIndex,
) {
    let parent_is_blocked = usage.is_blocked_by_unknown_item(item_id);
    let preserves_full_surface = enum_preserves_full_variant_surface(reduced, item_id, item_enum);
    for variant in &item_enum.variants {
        let name = variant.ident.to_string();
        let member = rendered_member_symbol_path(item_id, &name);
        let should_remain = preserves_full_surface
            || enum_variant_should_remain(project, reduced, &item_id.package, &name);
        record_rendered_member_decision(decisions, member, should_remain, parent_is_blocked);
    }
}

fn record_rendered_member_decision(
    decisions: &mut RenderedMemberDecisionIndex,
    member: String,
    should_remain: bool,
    parent_is_blocked: bool,
) {
    if should_remain {
        if parent_is_blocked {
            decisions.blocked_by_unknown.insert(member);
        } else {
            decisions.retained.insert(member);
        }
    } else {
        decisions.prunable.insert(member);
    }
}

fn rendered_member_symbol_path(item: &ItemId, member: &str) -> String {
    format!("{item}::{member}")
}

fn reachable_trait_surface_should_render(
    project: &Project,
    reduced: &ReducedProject,
    item: &ItemId,
) -> bool {
    if item.kind != ItemKind::Trait {
        return false;
    }
    let Some(record) = project.items.get(item) else {
        return false;
    };
    let Item::Trait(item_trait) = &record.item else {
        return false;
    };
    item_trait.items.is_empty()
        || item_trait.items.iter().any(|trait_item| {
            trait_item_should_remain_for_type_surface(
                project,
                reduced,
                &item.package,
                &item.module_path,
                &item.name,
                trait_item,
            )
        })
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
    fn build(
        project: &Project,
        reduced: &ReducedProject,
        rendered_items: &BTreeSet<ItemId>,
        reachable_token_idents: &ReachableTokenIdentIndex,
    ) -> Self {
        let mut index = Self::default();
        for callable in &reduced.reachable {
            let package = callable.package();
            let mut idents = BTreeSet::new();
            collect_callable_idents(callable, &mut idents);
            let module_path = if let Some(record) = project.functions.get(callable) {
                collect_token_idents(&record.item.to_token_stream(), &mut idents);
                expand_alias_surface_idents(&record.aliases, &mut idents);
                record.module_path.as_slice()
            } else if let Some(record) = project.methods.get(callable) {
                collect_token_idents(&record.item.to_token_stream(), &mut idents);
                expand_alias_surface_idents(&record.aliases, &mut idents);
                record.module_path.as_slice()
            } else {
                callable_module_path(callable)
            };
            index.add_package_idents(package, idents.iter().cloned());
            index.add_module_idents(package, module_path, idents);
        }

        for item in rendered_items {
            let idents = rendered_item_surface_idents(project, reduced, item);
            index.add_package_idents(&item.package, idents.iter().cloned());
            index.add_module_idents(&item.package, &item.module_path, idents);
        }

        for package in &reduced.packages {
            for module_path in project_module_paths(project, package) {
                let Some(items) = module_items_for_path(project, package, &module_path) else {
                    continue;
                };
                let mut idents = BTreeSet::new();
                collect_retained_module_surface_idents(
                    project,
                    reduced,
                    package,
                    &module_path,
                    items,
                    reachable_token_idents,
                    &mut idents,
                );
                index.add_module_idents(package, &module_path, idents);
            }
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

#[derive(Default)]
struct ReachableTokenIdentIndex {
    packages: BTreeMap<String, BTreeSet<String>>,
    modules: BTreeMap<ModuleMentionKey, BTreeSet<String>>,
}

impl ReachableTokenIdentIndex {
    fn build(project: &Project, reduced: &ReducedProject) -> Self {
        let mut index = Self::default();
        for callable in &reduced.reachable {
            let package = callable.package();
            let mut idents = BTreeSet::new();
            collect_callable_idents(callable, &mut idents);
            let module_path = if let Some(record) = project.functions.get(callable) {
                collect_token_idents(&record.item.to_token_stream(), &mut idents);
                expand_alias_surface_idents(&record.aliases, &mut idents);
                record.module_path.as_slice()
            } else if let Some(record) = project.methods.get(callable) {
                collect_token_idents(&record.item.to_token_stream(), &mut idents);
                expand_alias_surface_idents(&record.aliases, &mut idents);
                record.module_path.as_slice()
            } else {
                callable_module_path(callable)
            };
            index.add(package, module_path, idents);
        }

        for item in &reduced.reachable_items {
            let idents = rendered_item_surface_idents(project, reduced, item);
            index.add(&item.package, &item.module_path, idents);
        }

        index
    }

    fn add<I>(&mut self, package: &str, module_path: &[String], idents: I)
    where
        I: IntoIterator<Item = String>,
    {
        let idents = idents.into_iter().collect::<BTreeSet<_>>();
        self.packages
            .entry(package.to_string())
            .or_default()
            .extend(idents.iter().cloned());
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
    usage: &UsageDecisionIndex,
    output_root: &Path,
) -> Result<usize, Box<dyn std::error::Error>> {
    prepare_output_root(&project.workspace.root, output_root)?;
    write_output_marker(output_root)?;

    let render_plan = RenderPlan::build(project, reduced, usage);
    let package_usages = package_source_usages(project, reduced, &render_plan);
    let support_packages = SupportPackagePlan::build(project, reduced, &package_usages)?;

    write_workspace_manifest(
        project,
        reduced,
        output_root,
        &package_usages,
        &support_packages,
    )?;

    let mut files_written = 1
        + copy_workspace_lockfile(project, output_root)?
        + copy_workspace_cargo_config(project, output_root)?
        + copy_workspace_toolchain_files(project, output_root)?;
    files_written += support_packages.copy_to(output_root)?;
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
            &support_packages,
        )?;
        files_written += 1;

        if let Some(build_script) = build_script_to_render(package, package_usage) {
            let Some(resolved_build_script) = resolve_package_copy_source(package, &build_script)?
            else {
                return Err(format!(
                    "build script {} resolves outside package root {}",
                    build_script.display(),
                    package.root.display()
                )
                .into());
            };
            if !fs::metadata(&resolved_build_script)?.is_file() {
                return Err(format!(
                    "build script {} is not a regular file",
                    build_script.display()
                )
                .into());
            }
            let relative_path = package_relative_copy_path(package, &build_script)?;
            let output_path = package_output.join(relative_path);
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&resolved_build_script, output_path)?;
            files_written += 1;
            files_written +=
                copy_build_script_assets(package, &resolved_build_script, &package_output)?;
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
            let relative_path = package_relative_copy_path(package, &source.path)?;
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

fn copy_workspace_toolchain_files(
    project: &Project,
    output_root: &Path,
) -> Result<usize, Box<dyn std::error::Error>> {
    let mut copied = 0;
    for file_name in ["rust-toolchain.toml", "rust-toolchain"] {
        let source = project.workspace.root.join(file_name);
        if !source.is_file() {
            continue;
        }
        fs::copy(source, output_root.join(file_name))?;
        copied += 1;
    }
    Ok(copied)
}

#[derive(Default)]
struct SupportPackagePlan {
    packages: BTreeMap<PathBuf, SupportPackage>,
    workspace_manifests: BTreeMap<PathBuf, Value>,
    generated_workspace_roots: BTreeMap<PathBuf, PathBuf>,
}

struct SupportPackage {
    root: PathBuf,
    output_rel_dir: PathBuf,
    manifest: Value,
    workspace: Option<SupportWorkspace>,
    required_names: Option<BTreeSet<String>>,
    source_plan: SupportSourcePlan,
}

#[derive(Clone)]
struct SupportWorkspace {
    root: PathBuf,
    manifest: Value,
}

#[derive(Clone, Default)]
struct SupportSourcePlan {
    transformed_sources: Option<BTreeMap<PathBuf, syn::File>>,
    usage: Option<TokenUsage>,
}

impl SupportSourcePlan {
    fn is_restricted(&self) -> bool {
        self.usage.is_some()
    }

    fn dependency_public_names(&self, alias: &str) -> Option<BTreeSet<String>> {
        self.usage.as_ref()?.dependency_public_names(alias)
    }
}

impl SupportPackagePlan {
    fn build(
        project: &Project,
        reduced: &ReducedProject,
        package_usages: &HashMap<String, PackageSourceUsage>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut builder = SupportPackagePlanBuilder::new(project, reduced);
        builder.collect_retained_dependency_paths(reduced, package_usages)?;
        builder
            .collect_patch_replace_paths(&project.workspace.root, &project.workspace.manifest)?;
        builder.finish()
    }

    fn copy_to(&self, output_root: &Path) -> Result<usize, Box<dyn std::error::Error>> {
        let output_root = absolute_normalized_path(output_root)?;
        let mut copied = 0;
        for package in self.packages.values() {
            let package_output = output_root.join(&package.output_rel_dir);
            copied += copy_support_package_tree(package, &package_output)?;
            copied += copy_support_include_assets(package, &package_output, &output_root)?;
            let manifest = self.transformed_support_manifest(package)?;
            fs::write(
                package_output.join("Cargo.toml"),
                toml::to_string_pretty(&manifest)?,
            )?;
        }
        Ok(copied)
    }

    fn transformed_support_manifest(
        &self,
        package: &SupportPackage,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let mut manifest = package
            .manifest
            .as_table()
            .cloned()
            .unwrap_or_else(default_package_manifest_table);

        if let Some(package_table) = manifest.get_mut("package").and_then(Value::as_table_mut) {
            materialize_workspace_inherited_fields(
                package_table,
                package.workspace.as_ref(),
                &["package"],
            )?;
        }
        materialize_workspace_lints(&mut manifest, package.workspace.as_ref())?;

        for target_table in ["bin", "example", "test", "bench"] {
            manifest.remove(target_table);
        }

        let has_build_script = support_build_script_path(package).is_some();

        if let Some(table) = manifest
            .get_mut("dependencies")
            .and_then(Value::as_table_mut)
        {
            self.transform_support_dependency_table(package, table)?;
        }
        if has_build_script {
            if let Some(table) = manifest
                .get_mut("build-dependencies")
                .and_then(Value::as_table_mut)
            {
                self.transform_support_dependency_table(package, table)?;
            }
        } else {
            manifest.remove("build-dependencies");
        }
        manifest.remove("dev-dependencies");

        if let Some(targets) = manifest.get_mut("target").and_then(Value::as_table_mut) {
            for (_target_name, target) in targets.iter_mut() {
                let Some(target) = target.as_table_mut() else {
                    continue;
                };
                if let Some(table) = target.get_mut("dependencies").and_then(Value::as_table_mut) {
                    self.transform_support_dependency_table(package, table)?;
                }
                if has_build_script {
                    if let Some(table) = target
                        .get_mut("build-dependencies")
                        .and_then(Value::as_table_mut)
                    {
                        self.transform_support_dependency_table(package, table)?;
                    }
                } else {
                    target.remove("build-dependencies");
                }
                target.remove("dev-dependencies");
            }
            targets.retain(|_, target| target.as_table().is_some_and(|table| !table.is_empty()));
        }
        if manifest
            .get("target")
            .and_then(Value::as_table)
            .is_some_and(Table::is_empty)
        {
            manifest.remove("target");
        }

        manifest.remove("workspace");
        manifest.remove("patch");
        manifest.remove("replace");
        manifest.remove("profile");
        Ok(Value::Table(manifest))
    }

    fn transform_support_dependency_table(
        &self,
        package: &SupportPackage,
        dependencies: &mut Table,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let aliases = dependencies.keys().cloned().collect::<Vec<_>>();
        for alias in aliases {
            let Some(value) = dependencies.get(&alias).cloned() else {
                continue;
            };
            if package.source_plan.is_restricted()
                && package
                    .source_plan
                    .dependency_public_names(&alias)
                    .is_none()
            {
                dependencies.remove(&alias);
                continue;
            }
            let transformed = self.transformed_support_dependency_value(package, &alias, &value)?;
            dependencies.insert(alias, transformed);
        }
        Ok(())
    }

    fn transformed_support_dependency_value(
        &self,
        package: &SupportPackage,
        alias: &str,
        value: &Value,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let (mut value, manifest_dir) =
            materialized_support_dependency_value(package, alias, value)?;
        rewrite_dependency_path_value(&mut value, &manifest_dir, &package.output_rel_dir, self)?;
        remove_workspace_dependency_marker(&mut value);
        Ok(value)
    }

    fn transformed_workspace_dependency_value(&self, value: &Value, manifest_dir: &Path) -> Value {
        self.transformed_dependency_value(value, manifest_dir, Path::new(""))
    }

    fn transformed_dependency_value(
        &self,
        value: &Value,
        manifest_dir: &Path,
        output_manifest_dir: &Path,
    ) -> Value {
        let mut value = value.clone();
        let _ = rewrite_dependency_path_value(&mut value, manifest_dir, output_manifest_dir, self);
        value
    }

    fn support_output_for_root(&self, root: &Path) -> Option<&Path> {
        let root = root.canonicalize().ok()?;
        self.packages
            .get(&root)
            .map(|package| package.output_rel_dir.as_path())
            .or_else(|| {
                self.generated_workspace_roots
                    .get(&root)
                    .map(PathBuf::as_path)
            })
    }

    fn merged_patch_tables(
        &self,
        project: &Project,
        retained_patch_names: &BTreeSet<String>,
    ) -> Option<Value> {
        let mut patches = Table::new();
        if let Some(source_patches) = project
            .workspace
            .manifest
            .get("patch")
            .and_then(Value::as_table)
        {
            merge_transformed_patch_tables(
                &mut patches,
                source_patches,
                &project.workspace.root,
                self,
                retained_patch_names,
            );
        }
        for (workspace_root, manifest) in &self.workspace_manifests {
            if let Some(source_patches) = manifest.get("patch").and_then(Value::as_table) {
                merge_transformed_patch_tables(
                    &mut patches,
                    source_patches,
                    workspace_root,
                    self,
                    retained_patch_names,
                );
            }
        }
        patches.retain(|_, value| value.as_table().is_none_or(|table| !table.is_empty()));
        (!patches.is_empty()).then_some(Value::Table(patches))
    }

    fn merged_replace_table(&self, project: &Project) -> Option<Value> {
        let mut replacements = Table::new();
        if let Some(source_replacements) = project
            .workspace
            .manifest
            .get("replace")
            .and_then(Value::as_table)
        {
            merge_transformed_replace_table(
                &mut replacements,
                source_replacements,
                &project.workspace.root,
                self,
            );
        }
        for (workspace_root, manifest) in &self.workspace_manifests {
            if let Some(source_replacements) = manifest.get("replace").and_then(Value::as_table) {
                merge_transformed_replace_table(
                    &mut replacements,
                    source_replacements,
                    workspace_root,
                    self,
                );
            }
        }
        (!replacements.is_empty()).then_some(Value::Table(replacements))
    }
}

struct SupportPackagePlanBuilder<'a> {
    project: &'a Project,
    pending: BTreeSet<PathBuf>,
    pending_broad_roots: BTreeSet<PathBuf>,
    pending_required_names: BTreeMap<PathBuf, BTreeSet<String>>,
    packages: BTreeMap<PathBuf, SupportPackage>,
    workspace_manifests: BTreeMap<PathBuf, Value>,
    used_output_dirs: BTreeSet<PathBuf>,
    generated_workspace_roots: BTreeMap<PathBuf, PathBuf>,
}

impl<'a> SupportPackagePlanBuilder<'a> {
    fn new(project: &'a Project, reduced: &ReducedProject) -> Self {
        let generated_workspace_roots = reduced
            .packages
            .iter()
            .filter_map(|package_name| {
                let package = project.workspace.packages.get(package_name)?;
                Some((package.root.clone(), PathBuf::from(package_name)))
            })
            .collect();
        Self {
            project,
            pending: BTreeSet::new(),
            pending_broad_roots: BTreeSet::new(),
            pending_required_names: BTreeMap::new(),
            packages: BTreeMap::new(),
            workspace_manifests: BTreeMap::new(),
            used_output_dirs: BTreeSet::new(),
            generated_workspace_roots,
        }
    }

    fn collect_retained_dependency_paths(
        &mut self,
        reduced: &ReducedProject,
        package_usages: &HashMap<String, PackageSourceUsage>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for package_name in &reduced.packages {
            let Some(package) = self.project.workspace.packages.get(package_name) else {
                continue;
            };
            let Some(package_usage) = package_usages.get(package_name) else {
                continue;
            };
            let requested_features = requested_local_features(self.project, reduced, package_name);
            let feature_required_aliases =
                dependency_aliases_required_by_features(package, &requested_features);
            let retain_for_copied_support_source =
                package_should_copy_library_support_source(package);

            for (table_name, table) in package_dependency_tables(package) {
                let retention = if table_name == "build-dependencies"
                    && build_script_should_render_with_usage(package, package_usage)
                {
                    DependencyRetention::BuildScript
                } else {
                    DependencyRetention::SourceMentioned
                };

                for (alias, value) in table {
                    self.collect_retained_dependency_path(
                        reduced,
                        package,
                        package_name,
                        alias,
                        value,
                        retention,
                        package_usage,
                        feature_required_aliases.contains(alias),
                        retain_for_copied_support_source,
                    )?;
                }
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn collect_retained_dependency_path(
        &mut self,
        reduced: &ReducedProject,
        package: &Package,
        package_name: &str,
        alias: &str,
        value: &Value,
        retention: DependencyRetention,
        package_usage: &PackageSourceUsage,
        is_feature_required: bool,
        retain_for_copied_support_source: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (source, manifest_dir) =
            workspace_resolved_dependency_value_with_dir(self.project, alias, value, &package.root);
        let dependency_package = dependency_package_name(alias, source);
        if is_marker_dependency(alias, &dependency_package) {
            return Ok(());
        }

        if !(dependency_should_render(
            self.project,
            reduced,
            package_name,
            alias,
            retention,
            DependencyUsageScope::Any,
            package_usage,
            retain_for_copied_support_source,
        ) || is_feature_required)
        {
            return Ok(());
        }

        if self
            .project
            .workspace
            .packages
            .contains_key(&dependency_package)
        {
            let Some(root) = dependency_path_root(source, manifest_dir)? else {
                return Ok(());
            };
            if !self.generated_workspace_roots.contains_key(&root) {
                self.add_or_update_dependency_root(
                    root,
                    package_usage.dependency_public_names(alias),
                )?;
            }
            return Ok(());
        }

        self.add_dependency_path(
            source,
            manifest_dir,
            package_usage.dependency_public_names(alias),
        )
    }

    fn collect_patch_replace_paths(
        &mut self,
        manifest_dir: &Path,
        manifest: &Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(patches) = manifest.get("patch").and_then(Value::as_table) {
            for value in patches.values() {
                self.collect_manifest_path_dependencies(manifest_dir, value)?;
            }
        }
        if let Some(replacements) = manifest.get("replace").and_then(Value::as_table) {
            for value in replacements.values() {
                self.collect_manifest_path_dependencies(manifest_dir, value)?;
            }
        }
        Ok(())
    }

    fn collect_manifest_path_dependencies(
        &mut self,
        manifest_dir: &Path,
        value: &Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Some(table) = value.as_table() else {
            return Ok(());
        };
        if table.get("path").and_then(Value::as_str).is_some() {
            return self.add_dependency_path(value, manifest_dir, None);
        }
        for value in table.values() {
            self.collect_manifest_path_dependencies(manifest_dir, value)?;
        }
        Ok(())
    }

    fn add_dependency_path(
        &mut self,
        value: &Value,
        manifest_dir: &Path,
        required_names: Option<BTreeSet<String>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Some(root) = dependency_path_root(value, manifest_dir)? else {
            return Ok(());
        };
        if self.generated_workspace_roots.contains_key(&root) {
            return Ok(());
        }
        self.add_or_update_dependency_root(root, required_names)?;
        Ok(())
    }

    fn add_or_update_dependency_root(
        &mut self,
        root: PathBuf,
        required_names: Option<BTreeSet<String>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.packages.contains_key(&root) {
            return self.update_existing_dependency_root(root, required_names);
        }
        self.add_pending_dependency_root(root, required_names);
        Ok(())
    }

    fn add_pending_dependency_root(
        &mut self,
        root: PathBuf,
        required_names: Option<BTreeSet<String>>,
    ) {
        self.pending.insert(root.clone());
        if let Some(required_names) = required_names.filter(|names| !names.is_empty()) {
            if self.pending_broad_roots.contains(&root) {
                return;
            }
            self.pending_required_names
                .entry(root)
                .or_default()
                .extend(required_names);
        } else {
            self.pending_broad_roots.insert(root.clone());
            self.pending_required_names.remove(&root);
        }
    }

    fn update_existing_dependency_root(
        &mut self,
        root: PathBuf,
        required_names: Option<BTreeSet<String>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let requested_broad = required_names.as_ref().is_none_or(BTreeSet::is_empty);
        let Some(package) = self.packages.get(&root) else {
            return Ok(());
        };
        let Some(existing_names) = &package.required_names else {
            return Ok(());
        };

        let updated_required_names = if requested_broad {
            None
        } else {
            let mut names = existing_names.clone();
            names.extend(required_names.unwrap_or_default());
            if &names == existing_names {
                return Ok(());
            }
            Some(names)
        };

        let package_root = package.root.clone();
        let manifest = package.manifest.clone();
        let workspace = package.workspace.clone();
        let source_plan = if let Some(names) = &updated_required_names {
            build_support_source_plan(&package_root, &manifest, workspace.as_ref(), names)?
        } else {
            debug_support_prune(
                &package_root,
                "broad copy: widened by a later support dependency requirement",
            );
            SupportSourcePlan::default()
        };

        let package = self
            .packages
            .get_mut(&root)
            .expect("support package was checked as existing");
        package.required_names = updated_required_names;
        package.source_plan = source_plan.clone();

        self.collect_support_manifest_dependency_paths(
            &package_root,
            &manifest,
            workspace.as_ref(),
            &source_plan,
        )?;
        Ok(())
    }

    fn finish(mut self) -> Result<SupportPackagePlan, Box<dyn std::error::Error>> {
        while let Some(root) = self.pending.pop_first() {
            if self.packages.contains_key(&root) {
                continue;
            }
            let required_names = if self.pending_broad_roots.remove(&root) {
                None
            } else {
                Some(
                    self.pending_required_names
                        .remove(&root)
                        .unwrap_or_default(),
                )
            };
            let manifest = read_toml_value(&root.join("Cargo.toml"))?;
            let name = manifest_package_name(&manifest).unwrap_or_else(|| {
                root.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("package")
                    .to_string()
            });
            let workspace = support_workspace_for_package(&root)?;
            if let Some(workspace) = &workspace {
                self.workspace_manifests
                    .entry(workspace.root.clone())
                    .or_insert_with(|| workspace.manifest.clone());
                self.collect_patch_replace_paths(&workspace.root, &workspace.manifest)?;
            }
            let source_plan = if let Some(required_names) = &required_names {
                build_support_source_plan(&root, &manifest, workspace.as_ref(), required_names)?
            } else {
                SupportSourcePlan::default()
            };
            self.collect_support_manifest_dependency_paths(
                &root,
                &manifest,
                workspace.as_ref(),
                &source_plan,
            )?;
            let output_rel_dir = self.allocate_output_dir(&name);
            self.packages.insert(
                root.clone(),
                SupportPackage {
                    root,
                    output_rel_dir,
                    manifest,
                    workspace,
                    required_names,
                    source_plan,
                },
            );
        }

        Ok(SupportPackagePlan {
            packages: self.packages,
            workspace_manifests: self.workspace_manifests,
            generated_workspace_roots: self.generated_workspace_roots,
        })
    }

    fn collect_support_manifest_dependency_paths(
        &mut self,
        package_root: &Path,
        manifest: &Value,
        workspace: Option<&SupportWorkspace>,
        source_plan: &SupportSourcePlan,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let has_build_script = manifest_build_script_path(package_root, manifest).is_some();

        if let Some(table) = manifest.get("dependencies").and_then(Value::as_table) {
            self.collect_support_dependency_table_paths(
                package_root,
                table,
                workspace,
                source_plan,
            )?;
        }
        if has_build_script {
            if let Some(table) = manifest.get("build-dependencies").and_then(Value::as_table) {
                self.collect_support_dependency_table_paths(
                    package_root,
                    table,
                    workspace,
                    &SupportSourcePlan::default(),
                )?;
            }
        }

        if let Some(targets) = manifest.get("target").and_then(Value::as_table) {
            for target in targets.values() {
                let Some(target) = target.as_table() else {
                    continue;
                };
                if let Some(table) = target.get("dependencies").and_then(Value::as_table) {
                    self.collect_support_dependency_table_paths(
                        package_root,
                        table,
                        workspace,
                        source_plan,
                    )?;
                }
                if has_build_script {
                    if let Some(table) = target.get("build-dependencies").and_then(Value::as_table)
                    {
                        self.collect_support_dependency_table_paths(
                            package_root,
                            table,
                            workspace,
                            &SupportSourcePlan::default(),
                        )?;
                    }
                }
            }
        }
        Ok(())
    }

    fn collect_support_dependency_table_paths(
        &mut self,
        package_root: &Path,
        dependencies: &Table,
        workspace: Option<&SupportWorkspace>,
        source_plan: &SupportSourcePlan,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for (alias, value) in dependencies {
            let required_names = source_plan.dependency_public_names(alias);
            if source_plan.is_restricted() && required_names.is_none() {
                continue;
            }
            let (value, manifest_dir) =
                materialized_dependency_value_for_workspace(package_root, workspace, alias, value)?;
            self.add_dependency_path(&value, &manifest_dir, required_names)?;
        }
        Ok(())
    }

    fn allocate_output_dir(&mut self, package_name: &str) -> PathBuf {
        let base = sanitize_support_package_dir(package_name);
        for index in 0.. {
            let candidate = if index == 0 {
                PathBuf::from(SUPPORT_PACKAGE_DIR).join(&base)
            } else {
                PathBuf::from(SUPPORT_PACKAGE_DIR).join(format!("{base}-{index}"))
            };
            if self.used_output_dirs.insert(candidate.clone()) {
                return candidate;
            }
        }
        unreachable!("unbounded support package output suffix search should return")
    }
}

fn default_package_manifest_table() -> Table {
    let mut manifest = Table::new();
    manifest.insert("package".to_string(), default_package("package"));
    manifest
}

fn read_toml_value(path: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let text = fs::read_to_string(path)?;
    Ok(text.parse::<Value>()?)
}

fn manifest_package_name(manifest: &Value) -> Option<String> {
    manifest
        .get("package")
        .and_then(|package| package.get("name"))
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn support_workspace_for_package(
    package_root: &Path,
) -> Result<Option<SupportWorkspace>, Box<dyn std::error::Error>> {
    let package_root = package_root.canonicalize()?;
    let mut cursor = Some(package_root.as_path());
    while let Some(root) = cursor {
        let manifest_path = root.join("Cargo.toml");
        if manifest_path.is_file() {
            let manifest = read_toml_value(&manifest_path)?;
            if manifest
                .get("workspace")
                .and_then(Value::as_table)
                .is_some()
                && workspace_contains_package(root, &manifest, &package_root)
            {
                return Ok(Some(SupportWorkspace {
                    root: root.to_path_buf(),
                    manifest,
                }));
            }
        }
        cursor = root.parent();
    }
    Ok(None)
}

fn workspace_contains_package(
    workspace_root: &Path,
    manifest: &Value,
    package_root: &Path,
) -> bool {
    if workspace_root == package_root {
        return true;
    }
    let Some(members) = manifest
        .get("workspace")
        .and_then(|workspace| workspace.get("members"))
        .and_then(Value::as_array)
    else {
        return false;
    };

    members.iter().filter_map(Value::as_str).any(|member| {
        if member.contains('*') {
            let prefix = member.split('*').next().unwrap_or_default();
            let prefix = workspace_root.join(prefix);
            package_root.starts_with(normalize_path(&prefix))
        } else {
            workspace_root
                .join(member)
                .canonicalize()
                .is_ok_and(|member_root| member_root == package_root)
        }
    }) || package_root.starts_with(workspace_root)
}

fn materialize_workspace_inherited_fields(
    table: &mut Table,
    workspace: Option<&SupportWorkspace>,
    section_path: &[&str],
) -> Result<(), Box<dyn std::error::Error>> {
    let inherited_keys = table
        .iter()
        .filter(|(_, value)| value_uses_workspace_marker(value))
        .map(|(key, _)| key.clone())
        .collect::<Vec<_>>();
    for key in inherited_keys {
        let value = workspace_value(workspace, section_path, &key)?.clone();
        table.insert(key, value);
    }
    Ok(())
}

fn materialize_workspace_lints(
    manifest: &mut Table,
    workspace: Option<&SupportWorkspace>,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(lints) = manifest.get_mut("lints") else {
        return Ok(());
    };
    if !value_uses_workspace_marker(lints) {
        return Ok(());
    }

    let mut overlay = lints.as_table().cloned().unwrap_or_default();
    overlay.remove("workspace");
    let mut lints = workspace_value(workspace, &["lints"], "")?
        .as_table()
        .cloned()
        .ok_or("workspace lints inheritance resolved to a non-table value")?;
    lints.extend(overlay);
    manifest.insert("lints".to_string(), Value::Table(lints));
    Ok(())
}

fn workspace_value<'a>(
    workspace: Option<&'a SupportWorkspace>,
    section_path: &[&str],
    key: &str,
) -> Result<&'a Value, Box<dyn std::error::Error>> {
    let workspace = workspace.ok_or("manifest uses workspace inheritance outside a workspace")?;
    let mut value = workspace
        .manifest
        .get("workspace")
        .ok_or("workspace manifest has no [workspace] table")?;
    for section in section_path {
        if !section.is_empty() {
            value = value.get(*section).ok_or_else(|| {
                format!(
                    "workspace manifest {} has no workspace.{} table",
                    workspace.root.display(),
                    section_path.join(".")
                )
            })?;
        }
    }
    if key.is_empty() {
        return Ok(value);
    }
    value.get(key).ok_or_else(|| {
        format!(
            "workspace manifest {} has no inherited key {}.{}",
            workspace.root.display(),
            section_path.join("."),
            key
        )
        .into()
    })
}

fn value_uses_workspace_marker(value: &Value) -> bool {
    value
        .as_table()
        .and_then(|table| table.get("workspace"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn materialized_support_dependency_value(
    package: &SupportPackage,
    alias: &str,
    value: &Value,
) -> Result<(Value, PathBuf), Box<dyn std::error::Error>> {
    materialized_dependency_value_for_workspace(
        &package.root,
        package.workspace.as_ref(),
        alias,
        value,
    )
}

fn materialized_dependency_value_for_workspace(
    package_root: &Path,
    workspace: Option<&SupportWorkspace>,
    alias: &str,
    value: &Value,
) -> Result<(Value, PathBuf), Box<dyn std::error::Error>> {
    if !dependency_uses_workspace(value) {
        return Ok((value.clone(), package_root.to_path_buf()));
    }

    let workspace = workspace.ok_or_else(|| {
        format!(
            "dependency {alias} uses workspace inheritance outside a workspace at {}",
            package_root.display()
        )
    })?;
    let base = workspace
        .manifest
        .get("workspace")
        .and_then(|workspace| workspace.get("dependencies"))
        .and_then(Value::as_table)
        .and_then(|dependencies| dependencies.get(alias))
        .ok_or_else(|| {
            format!(
                "workspace dependency {alias} is not defined in {}",
                workspace.root.join("Cargo.toml").display()
            )
        })?;
    Ok((
        merge_workspace_dependency_value(base, value),
        workspace.root.clone(),
    ))
}

fn merge_workspace_dependency_value(base: &Value, overlay: &Value) -> Value {
    let Some(overlay_table) = overlay.as_table() else {
        return overlay.clone();
    };
    if !dependency_uses_workspace(overlay) {
        return overlay.clone();
    }

    let mut table = dependency_value_as_table(base);
    for (key, value) in overlay_table {
        if key == "workspace" {
            continue;
        }
        if key == "features" {
            merge_feature_values(&mut table, value);
        } else {
            table.insert(key.clone(), value.clone());
        }
    }
    Value::Table(table)
}

fn dependency_value_as_table(value: &Value) -> Table {
    match value {
        Value::Table(table) => table.clone(),
        Value::String(version) => {
            let mut table = Table::new();
            table.insert("version".to_string(), Value::String(version.clone()));
            table
        }
        _ => Table::new(),
    }
}

fn merge_feature_values(table: &mut Table, value: &Value) {
    let Some(new_features) = value.as_array() else {
        table.insert("features".to_string(), value.clone());
        return;
    };
    let mut features = table
        .get("features")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    features.extend(new_features.iter().cloned());
    features.sort_by_key(|value| value.as_str().unwrap_or_default().to_string());
    features.dedup();
    table.insert("features".to_string(), Value::Array(features));
}

fn rewrite_dependency_path_value(
    value: &mut Value,
    manifest_dir: &Path,
    output_manifest_dir: &Path,
    support_packages: &SupportPackagePlan,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(table) = value.as_table_mut() else {
        return Ok(());
    };
    let Some(path_value) = table.get("path").and_then(Value::as_str) else {
        return Ok(());
    };
    let path = PathBuf::from(path_value);
    let source_root = if path.is_absolute() {
        path
    } else {
        manifest_dir.join(path)
    };
    let source_root = source_root.canonicalize()?;
    if let Some(output_root) = support_packages.support_output_for_root(&source_root) {
        let relative = relative_path_between(output_manifest_dir, output_root);
        table.insert("path".to_string(), Value::String(toml_path(&relative)));
    } else {
        table.insert("path".to_string(), Value::String(toml_path(&source_root)));
    }
    Ok(())
}

fn remove_workspace_dependency_marker(value: &mut Value) {
    if let Some(table) = value.as_table_mut() {
        table.remove("workspace");
    }
}

fn workspace_resolved_dependency_value_with_dir<'a>(
    project: &'a Project,
    alias: &str,
    value: &'a Value,
    package_root: &'a Path,
) -> (&'a Value, &'a Path) {
    if !dependency_uses_workspace(value) {
        return (value, package_root);
    }
    project
        .workspace
        .manifest
        .get("workspace")
        .and_then(|workspace| workspace.get("dependencies"))
        .and_then(Value::as_table)
        .and_then(|dependencies| dependencies.get(alias))
        .map(|value| (value, project.workspace.root.as_path()))
        .unwrap_or((value, package_root))
}

fn dependency_path_root(
    value: &Value,
    manifest_dir: &Path,
) -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
    let Some(path) = value
        .as_table()
        .and_then(|table| table.get("path"))
        .and_then(Value::as_str)
    else {
        return Ok(None);
    };
    let path = PathBuf::from(path);
    let path = if path.is_absolute() {
        path
    } else {
        manifest_dir.join(path)
    };
    let Ok(root) = path.canonicalize() else {
        return Ok(None);
    };
    if !root.join("Cargo.toml").is_file() {
        return Ok(None);
    }
    Ok(Some(root))
}

fn sanitize_support_package_dir(package_name: &str) -> String {
    let sanitized = package_name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    if sanitized.is_empty() {
        "package".to_string()
    } else {
        sanitized
    }
}

fn copy_support_package_tree(
    package: &SupportPackage,
    package_output: &Path,
) -> Result<usize, Box<dyn std::error::Error>> {
    if support_build_script_path(package).is_some() {
        return copy_support_package_tree_broad(&package.root, package_output);
    }

    copy_support_package_library_source_tree(package, package_output)
}

fn copy_support_package_tree_broad(
    package_root: &Path,
    package_output: &Path,
) -> Result<usize, Box<dyn std::error::Error>> {
    let package_root = package_root.canonicalize()?;
    let mut visited = BTreeSet::new();
    copy_support_package_tree_inner(
        &package_root,
        &package_root,
        package_output,
        false,
        &mut visited,
    )
}

fn copy_support_package_library_source_tree(
    package: &SupportPackage,
    package_output: &Path,
) -> Result<usize, Box<dyn std::error::Error>> {
    let package_root = package.root.canonicalize()?;
    let mut copied = copy_support_package_file(
        &package_root,
        &package_root.join("Cargo.toml"),
        package_output,
    )?;
    if let Some(transformed_sources) = &package.source_plan.transformed_sources {
        for (source_file, syntax) in transformed_sources {
            copied += write_support_package_source_file(
                &package_root,
                source_file,
                syntax,
                package_output,
            )?;
        }
        return Ok(copied);
    }
    let Some(lib_path) = support_library_source_path(package) else {
        return Ok(copied);
    };
    if let Some(source_files) = support_library_module_source_files(package)? {
        for source_file in source_files {
            copied += copy_support_package_file(&package_root, &source_file, package_output)?;
        }
        return Ok(copied);
    }

    let src_dir = package_root.join("src");
    if lib_path.starts_with(&src_dir) {
        copied += copy_support_package_tree_inner(
            &package_root,
            &src_dir,
            package_output,
            true,
            &mut BTreeSet::new(),
        )?;
        return Ok(copied);
    }
    let source_root = lib_path.parent().unwrap_or(&lib_path);
    copied += copy_support_package_tree_inner(
        &package_root,
        source_root,
        package_output,
        true,
        &mut BTreeSet::new(),
    )?;
    Ok(copied)
}

fn build_support_source_plan(
    package_root: &Path,
    manifest: &Value,
    workspace: Option<&SupportWorkspace>,
    required_names: &BTreeSet<String>,
) -> Result<SupportSourcePlan, Box<dyn std::error::Error>> {
    if required_names.is_empty() {
        debug_support_prune(package_root, "broad copy: no required public names");
        return Ok(SupportSourcePlan::default());
    }
    if manifest_build_script_path(package_root, manifest).is_some() {
        debug_support_prune(package_root, "broad copy: build script present");
        return Ok(SupportSourcePlan::default());
    }

    let Some(lib_path) = support_library_source_path_from(package_root, manifest) else {
        debug_support_prune(package_root, "broad copy: no library source path");
        return Ok(SupportSourcePlan::default());
    };
    let Ok(package_root) = package_root.canonicalize() else {
        debug_support_prune(package_root, "broad copy: package root canonicalize failed");
        return Ok(SupportSourcePlan::default());
    };
    let Ok(lib_path) = lib_path.canonicalize() else {
        debug_support_prune(
            &package_root,
            "broad copy: library path canonicalize failed",
        );
        return Ok(SupportSourcePlan::default());
    };
    if !lib_path.starts_with(&package_root) {
        debug_support_prune(&package_root, "broad copy: library outside package root");
        return Ok(SupportSourcePlan::default());
    }

    let text = match fs::read_to_string(&lib_path) {
        Ok(text) => text,
        Err(_) => {
            debug_support_prune(&package_root, "broad copy: library read failed");
            return Ok(SupportSourcePlan::default());
        }
    };
    let syntax = match syn::parse_file(&text) {
        Ok(syntax) => syntax,
        Err(_) => {
            debug_support_prune(&package_root, "broad copy: library parse failed");
            return Ok(SupportSourcePlan::default());
        }
    };
    let dependency_roots = support_source_dependency_roots(&package_root, manifest, workspace)?;
    let macro_expansions =
        support_macro_expansions_for_manifest(&package_root, manifest, workspace)?;
    let Some(transformed_sources) = build_restricted_support_sources(
        &package_root,
        &lib_path,
        &syntax,
        &dependency_roots,
        required_names,
        &macro_expansions,
    )?
    else {
        debug_support_prune(
            &package_root,
            &format!("broad copy: restricted source closure failed for {required_names:?}"),
        );
        return Ok(SupportSourcePlan::default());
    };

    let mut usage = TokenUsage::default();
    for syntax in transformed_sources.values() {
        usage.record_file(syntax);
    }
    record_required_external_reexport_usage(&syntax, &dependency_roots, required_names, &mut usage);
    for expansion in &macro_expansions {
        if support_macro_expansion_is_live(&usage, expansion) {
            usage.merge(&expansion.usage);
        }
    }

    Ok(SupportSourcePlan {
        transformed_sources: Some(transformed_sources),
        usage: Some(usage),
    })
}

fn debug_support_prune(package_root: &Path, message: &str) {
    if env::var_os("SLICERS_DEBUG_SUPPORT_PRUNE").is_some() {
        eprintln!("support prune {}: {message}", package_root.display());
    }
}

fn support_source_dependency_roots(
    package_root: &Path,
    manifest: &Value,
    workspace: Option<&SupportWorkspace>,
) -> Result<BTreeSet<String>, Box<dyn std::error::Error>> {
    let mut roots = BTreeSet::new();
    if let Some(manifest) = manifest.as_table() {
        collect_support_source_dependency_roots(package_root, workspace, manifest, &mut roots)?;
    }

    if let Some(targets) = manifest.get("target").and_then(Value::as_table) {
        for target in targets.values() {
            let Some(target) = target.as_table() else {
                continue;
            };
            collect_support_source_dependency_roots(package_root, workspace, target, &mut roots)?;
        }
    }

    Ok(roots)
}

fn collect_support_source_dependency_roots(
    package_root: &Path,
    workspace: Option<&SupportWorkspace>,
    table_parent: &Table,
    roots: &mut BTreeSet<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    for table_name in ["dependencies", "build-dependencies", "dev-dependencies"] {
        let Some(dependencies) = table_parent.get(table_name).and_then(Value::as_table) else {
            continue;
        };
        for (alias, value) in dependencies {
            let (value, _) =
                materialized_dependency_value_for_workspace(package_root, workspace, alias, value)?;
            let package = dependency_package_name(alias, &value);
            if is_marker_dependency(alias, &package) {
                continue;
            }
            roots.insert(alias.clone());
            roots.insert(dependency_code_name(alias));
        }
    }
    Ok(())
}

fn support_macro_expansions_for_manifest(
    package_root: &Path,
    manifest: &Value,
    workspace: Option<&SupportWorkspace>,
) -> Result<Vec<SupportMacroExpansion>, Box<dyn std::error::Error>> {
    let mut expansions = BTreeMap::<(PathBuf, String), SupportMacroExpansion>::new();
    if let Some(manifest) = manifest.as_table() {
        collect_support_macro_expansions_from_manifest_table(
            package_root,
            workspace,
            manifest,
            &mut expansions,
        )?;
    }

    if let Some(targets) = manifest.get("target").and_then(Value::as_table) {
        for target in targets.values() {
            let Some(target) = target.as_table() else {
                continue;
            };
            collect_support_macro_expansions_from_manifest_table(
                package_root,
                workspace,
                target,
                &mut expansions,
            )?;
        }
    }

    Ok(expansions.into_values().collect())
}

fn collect_support_macro_expansions_from_manifest_table(
    package_root: &Path,
    workspace: Option<&SupportWorkspace>,
    table_parent: &Table,
    expansions: &mut BTreeMap<(PathBuf, String), SupportMacroExpansion>,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(dependencies) = table_parent.get("dependencies").and_then(Value::as_table) else {
        return Ok(());
    };
    for (alias, value) in dependencies {
        let (value, manifest_dir) =
            materialized_dependency_value_for_workspace(package_root, workspace, alias, value)?;
        let package = dependency_package_name(alias, &value);
        if is_marker_dependency(alias, &package) {
            continue;
        }
        let Some(root) = dependency_path_root(&value, &manifest_dir)? else {
            continue;
        };
        let key = (root.clone(), alias.clone());
        if expansions.contains_key(&key) {
            continue;
        }
        let Some(expansion) = support_proc_macro_expansion(alias, &root)? else {
            continue;
        };
        expansions.insert(key, expansion);
    }
    Ok(())
}

fn support_proc_macro_expansion(
    alias: &str,
    package_root: &Path,
) -> Result<Option<SupportMacroExpansion>, Box<dyn std::error::Error>> {
    let manifest = read_toml_value(&package_root.join("Cargo.toml"))?;
    if !manifest_is_proc_macro_crate(&manifest) {
        return Ok(None);
    }
    let Some(lib_path) = support_library_source_path_from(package_root, &manifest) else {
        return Ok(None);
    };
    let Ok(package_root) = package_root.canonicalize() else {
        return Ok(None);
    };
    let Ok(lib_path) = lib_path.canonicalize() else {
        return Ok(None);
    };
    if !lib_path.starts_with(&package_root) {
        return Ok(None);
    }
    let text = match fs::read_to_string(&lib_path) {
        Ok(text) => text,
        Err(_) => return Ok(None),
    };
    let syntax = match syn::parse_file(&text) {
        Ok(syntax) => syntax,
        Err(_) => return Ok(None),
    };

    let mut modules = BTreeMap::new();
    let root_module_dir = lib_path.parent().unwrap_or(&package_root).to_path_buf();
    if !collect_support_module_sources_with_syntax(
        &package_root,
        &lib_path,
        &root_module_dir,
        None,
        syntax,
        &mut modules,
    )? {
        return Ok(None);
    }

    let mut export_names = BTreeSet::new();
    let mut quote_bodies = Vec::new();
    for module in modules.values() {
        collect_proc_macro_export_names(&module.syntax, &mut export_names);
        collect_macro_expansion_quote_bodies(&module.syntax.to_token_stream(), &mut quote_bodies);
    }
    if export_names.is_empty() || quote_bodies.is_empty() {
        return Ok(None);
    }

    let mut usage = TokenUsage::default();
    let mut paths = Vec::new();
    for body in quote_bodies {
        collect_token_usage(&body, &mut usage);
        paths.extend(token_path_candidates(&body));
    }
    if usage.idents.is_empty() && usage.path_roots.is_empty() {
        return Ok(None);
    }

    Ok(Some(SupportMacroExpansion {
        alias: alias.to_string(),
        export_names,
        usage,
        paths,
    }))
}

fn manifest_is_proc_macro_crate(manifest: &Value) -> bool {
    manifest
        .get("lib")
        .and_then(|lib| lib.get("proc-macro"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn collect_proc_macro_export_names(syntax: &syn::File, export_names: &mut BTreeSet<String>) {
    for item in &syntax.items {
        let Item::Fn(item_fn) = item else {
            continue;
        };
        for attr in &item_fn.attrs {
            if let Some(export_name) = proc_macro_export_name(attr, &item_fn.sig.ident) {
                export_names.insert(export_name);
            }
        }
    }
}

fn proc_macro_export_name(attr: &syn::Attribute, fn_ident: &syn::Ident) -> Option<String> {
    if attr.path().is_ident("proc_macro") || attr.path().is_ident("proc_macro_attribute") {
        return Some(fn_ident.to_string());
    }

    if !attr.path().is_ident("proc_macro_derive") {
        return None;
    }
    let args = attr
        .parse_args_with(Punctuated::<Meta, syn::Token![,]>::parse_terminated)
        .ok()?;
    args.iter().find_map(|meta| match meta {
        Meta::Path(path) => path.get_ident().map(ToString::to_string),
        Meta::List(_) | Meta::NameValue(_) => None,
    })
}

fn collect_macro_expansion_quote_bodies(tokens: &TokenStream, bodies: &mut Vec<TokenStream>) {
    let token_trees = tokens.clone().into_iter().collect::<Vec<_>>();
    for token in &token_trees {
        if let TokenTree::Group(group) = token {
            collect_macro_expansion_quote_bodies(&group.stream(), bodies);
        }
    }

    let mut index = 0;
    while index + 2 < token_trees.len() {
        let TokenTree::Ident(ident) = &token_trees[index] else {
            index += 1;
            continue;
        };
        let macro_name = ident.to_string();
        let is_expansion_macro = matches!(
            macro_name.as_str(),
            "quote" | "quote_spanned" | "parse_quote"
        );
        if is_expansion_macro
            && matches!(token_trees.get(index + 1), Some(TokenTree::Punct(punct)) if punct.as_char() == '!')
        {
            if let Some(TokenTree::Group(group)) = token_trees.get(index + 2) {
                bodies.push(group.stream());
                index += 3;
                continue;
            }
        }
        index += 1;
    }
}

fn support_macro_expansion_is_live(
    live_usage: &TokenUsage,
    expansion: &SupportMacroExpansion,
) -> bool {
    if !live_usage.mentions_dependency(&expansion.alias) {
        return false;
    }
    let imported_names = live_usage
        .dependency_public_names(&expansion.alias)
        .unwrap_or_default();
    expansion.export_names.iter().any(|name| {
        live_usage.mentions_ident(name)
            || live_usage.use_idents.contains(name)
            || imported_names.contains(name)
    })
}

#[derive(Clone)]
struct SupportModuleSource {
    module_dir: PathBuf,
    parent_file: Option<PathBuf>,
    syntax: syn::File,
    inline: bool,
}

#[derive(Clone, Default)]
struct SupportLiveSet {
    item_names: BTreeSet<String>,
    public_exports: BTreeSet<String>,
    assoc_item_names: BTreeMap<String, BTreeSet<String>>,
}

struct SupportUseNeeds<'a> {
    live_idents: &'a BTreeSet<String>,
    live_exports: &'a BTreeSet<String>,
    live_assoc_items: &'a BTreeMap<String, BTreeSet<String>>,
    live_paths: &'a BTreeSet<Vec<String>>,
}

struct SupportMacroExpansion {
    alias: String,
    export_names: BTreeSet<String>,
    usage: TokenUsage,
    paths: Vec<Vec<String>>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct SupportMethodCandidate {
    source_file: PathBuf,
    self_name: String,
}

#[derive(Clone, Default)]
struct SupportEnumVariantPayloads {
    unnamed: Vec<Option<SupportMethodCandidate>>,
    named: BTreeMap<String, SupportMethodCandidate>,
}

struct SupportResolveContext<'a> {
    modules: &'a BTreeMap<PathBuf, SupportModuleSource>,
    root_file: &'a Path,
    dependency_roots: &'a BTreeSet<String>,
}

enum SupportReexportMark {
    NotMatched,
    Matched(bool),
    Unsupported,
}

enum SupportLocalTarget {
    Local,
    External,
    Unsupported,
}

enum SupportLocalGlobTarget {
    Module(PathBuf),
    Enum {
        source_file: PathBuf,
        enum_name: String,
        variants: BTreeSet<String>,
    },
    External,
    Unsupported,
}

fn build_restricted_support_sources(
    package_root: &Path,
    lib_path: &Path,
    syntax: &syn::File,
    dependency_roots: &BTreeSet<String>,
    required_names: &BTreeSet<String>,
    macro_expansions: &[SupportMacroExpansion],
) -> Result<Option<BTreeMap<PathBuf, syn::File>>, Box<dyn std::error::Error>> {
    let mut modules = BTreeMap::new();
    let root_module_dir = lib_path.parent().unwrap_or(package_root).to_path_buf();
    if !collect_support_module_sources_with_syntax(
        package_root,
        lib_path,
        &root_module_dir,
        None,
        syntax.clone(),
        &mut modules,
    )? {
        debug_support_prune(
            package_root,
            "restricted fail: module source collection failed",
        );
        return Ok(None);
    }

    let mut live = BTreeMap::<PathBuf, SupportLiveSet>::new();
    let root_file = lib_path.to_path_buf();
    let ctx = SupportResolveContext {
        modules: &modules,
        root_file: &root_file,
        dependency_roots,
    };
    let method_index = support_method_index(&modules);
    let variant_payload_index = support_enum_variant_payload_index(&modules);
    for required_name in required_names {
        if !seed_support_required_name(&ctx, required_name, &mut live) {
            debug_support_prune(
                package_root,
                &format!("restricted fail: could not seed required name {required_name}"),
            );
            return Ok(None);
        }
    }

    let mut live_usage_by_file = BTreeMap::<PathBuf, TokenUsage>::new();
    let mut changed = true;
    while changed {
        changed = false;
        for (source_file, live_set) in live.clone() {
            let Some(module) = modules.get(&source_file) else {
                debug_support_prune(
                    package_root,
                    &format!(
                        "restricted fail: live source missing {}",
                        source_file.display()
                    ),
                );
                return Ok(None);
            };
            let named_items = support_named_item_names(&module.syntax.items);
            let mut live_usage = TokenUsage::default();

            for name in &live_set.item_names {
                let Some(item) = named_items.get(name) else {
                    debug_support_prune(
                        package_root,
                        &format!(
                            "restricted fail: live item {name} missing in {}",
                            source_file.display()
                        ),
                    );
                    return Ok(None);
                };
                let Some(inserted) = mark_support_token_dependencies(
                    &ctx,
                    &source_file,
                    &support_item_dependency_tokens(item),
                    &named_items,
                    &mut live,
                    &mut live_usage,
                    true,
                ) else {
                    debug_support_prune(
                        package_root,
                        &format!(
                            "restricted fail: token dependency unsupported in {}::{name}",
                            source_file.display()
                        ),
                    );
                    return Ok(None);
                };
                changed |= inserted;
                let Some(inserted) = mark_support_item_assoc_usage_dependencies(
                    &ctx,
                    &source_file,
                    item,
                    &named_items,
                    &mut live,
                ) else {
                    debug_support_prune(
                        package_root,
                        &format!(
                            "restricted fail: item assoc dependency unsupported in {}::{name}",
                            source_file.display()
                        ),
                    );
                    return Ok(None);
                };
                changed |= inserted;
                let Some(inserted) =
                    mark_support_item_attribute_dependencies(&ctx, &source_file, item, &mut live)
                else {
                    debug_support_prune(
                        package_root,
                        &format!(
                            "restricted fail: item attribute dependency unsupported in {}::{name}",
                            source_file.display()
                        ),
                    );
                    return Ok(None);
                };
                changed |= inserted;
                changed |=
                    mark_support_method_fallback_dependencies(&ctx, &method_index, item, &mut live);
                changed |= mark_support_enum_variant_method_dependencies(
                    &ctx,
                    &method_index,
                    &variant_payload_index,
                    item,
                    &mut live,
                );
            }

            for item in &module.syntax.items {
                let Item::Impl(item_impl) = item else {
                    continue;
                };
                if !support_impl_should_render(item_impl, &named_items, &live_set) {
                    continue;
                }
                let tokens = support_impl_dependency_tokens(item_impl, &named_items, &live_set)
                    .unwrap_or_else(|| item.to_token_stream());
                let Some(inserted) = mark_support_token_dependencies(
                    &ctx,
                    &source_file,
                    &tokens,
                    &named_items,
                    &mut live,
                    &mut live_usage,
                    true,
                ) else {
                    debug_support_prune(
                        package_root,
                        &format!(
                            "restricted fail: impl dependency unsupported in {}",
                            source_file.display()
                        ),
                    );
                    return Ok(None);
                };
                changed |= inserted;
                let Some(inserted) = mark_support_impl_self_assoc_dependencies(
                    &ctx,
                    &source_file,
                    item_impl,
                    &named_items,
                    &live_set,
                    &mut live,
                ) else {
                    debug_support_prune(
                        package_root,
                        &format!(
                            "restricted fail: impl self dependency unsupported in {}",
                            source_file.display()
                        ),
                    );
                    return Ok(None);
                };
                changed |= inserted;
                let Some(inserted) = mark_support_impl_attribute_dependencies(
                    &ctx,
                    &source_file,
                    item_impl,
                    &mut live,
                ) else {
                    debug_support_prune(
                        package_root,
                        &format!(
                            "restricted fail: impl attribute dependency unsupported in {}",
                            source_file.display()
                        ),
                    );
                    return Ok(None);
                };
                changed |= inserted;
                if let Some(rendered_impl) =
                    transform_support_impl(item_impl, &named_items, &live_set)
                {
                    let rendered_item = Item::Impl(rendered_impl);
                    changed |= mark_support_method_fallback_dependencies(
                        &ctx,
                        &method_index,
                        &rendered_item,
                        &mut live,
                    );
                    changed |= mark_support_enum_variant_method_dependencies(
                        &ctx,
                        &method_index,
                        &variant_payload_index,
                        &rendered_item,
                        &mut live,
                    );
                }
            }

            for item in &module.syntax.items {
                if !matches!(item, Item::Macro(item_macro) if item_macro.ident.is_none()) {
                    continue;
                }
                let tokens = item.to_token_stream();
                let Some(inserted) = mark_support_token_dependencies(
                    &ctx,
                    &source_file,
                    &tokens,
                    &named_items,
                    &mut live,
                    &mut live_usage,
                    false,
                ) else {
                    return Ok(None);
                };
                changed |= inserted;
            }

            let mut macro_probe_usage = live_usage.clone();
            let live_assoc_import_items = support_combined_assoc_items(
                &live_set.assoc_item_names,
                &live_usage.dependency_public_names,
            );
            for item in &module.syntax.items {
                let Item::Use(item_use) = item else {
                    continue;
                };
                if support_use_tree_imports_live_name(
                    &item_use.tree,
                    Vec::new(),
                    &live_usage.idents,
                    &live_set.public_exports,
                ) {
                    macro_probe_usage.record_use(item_use);
                }
                let Some(inserted) = mark_support_live_use_imports(
                    &ctx,
                    &source_file,
                    &item_use.tree,
                    Vec::new(),
                    SupportUseNeeds {
                        live_idents: &live_usage.idents,
                        live_exports: &live_set.public_exports,
                        live_assoc_items: &live_assoc_import_items,
                        live_paths: &live_usage.path_candidates,
                    },
                    &mut live,
                ) else {
                    debug_support_prune(
                        package_root,
                        &format!(
                            "restricted fail: use dependency unsupported in {}",
                            source_file.display()
                        ),
                    );
                    return Ok(None);
                };
                changed |= inserted;
            }

            for expansion in macro_expansions {
                if !support_macro_expansion_is_live(&macro_probe_usage, expansion) {
                    continue;
                }
                live_usage.merge(&expansion.usage);
                macro_probe_usage.merge(&expansion.usage);
                for segments in &expansion.paths {
                    let Some(inserted) =
                        mark_support_path_target(&ctx, &source_file, segments, &mut live)
                    else {
                        debug_support_prune(
                            package_root,
                            &format!(
                                "restricted fail: proc-macro expansion dependency unsupported in {}",
                                source_file.display()
                            ),
                        );
                        return Ok(None);
                    };
                    changed |= inserted;
                }
            }

            let live_assoc_import_items = support_combined_assoc_items(
                &live_set.assoc_item_names,
                &live_usage.dependency_public_names,
            );
            for item in &module.syntax.items {
                let Item::Use(item_use) = item else {
                    continue;
                };
                let Some(inserted) = mark_support_live_use_imports(
                    &ctx,
                    &source_file,
                    &item_use.tree,
                    Vec::new(),
                    SupportUseNeeds {
                        live_idents: &live_usage.idents,
                        live_exports: &live_set.public_exports,
                        live_assoc_items: &live_assoc_import_items,
                        live_paths: &live_usage.path_candidates,
                    },
                    &mut live,
                ) else {
                    debug_support_prune(
                        package_root,
                        &format!(
                            "restricted fail: expanded use dependency unsupported in {}",
                            source_file.display()
                        ),
                    );
                    return Ok(None);
                };
                changed |= inserted;
            }
            live_usage_by_file.insert(source_file.clone(), live_usage);
        }
    }

    for (source_file, live_set) in &live {
        let Some(module) = modules.get(source_file) else {
            debug_support_prune(
                package_root,
                &format!(
                    "restricted fail: final live source missing {}",
                    source_file.display()
                ),
            );
            return Ok(None);
        };
        for item_name in &live_set.item_names {
            let Some(Item::Mod(item_mod)) = module
                .syntax
                .items
                .iter()
                .find(|item| support_item_name(item).as_ref() == Some(item_name))
            else {
                continue;
            };
            if item_mod.content.is_none()
                && support_external_module_source(package_root, &module.module_dir, item_mod)
                    .is_some_and(|(child_file, _)| !live.contains_key(&child_file))
            {
                debug_support_prune(
                    package_root,
                    &format!(
                        "restricted fail: external module {item_name} was retained without child source"
                    ),
                );
                return Ok(None);
            }
        }
    }

    let mut transformed_sources = BTreeMap::new();
    for (source_file, module) in &modules {
        if module.inline {
            continue;
        }
        if *source_file != root_file && !live.contains_key(source_file) {
            continue;
        }
        let live_set = live.get(source_file).cloned().unwrap_or_default();
        transformed_sources.insert(
            source_file.clone(),
            transform_restricted_support_file(
                &ctx,
                &live,
                source_file,
                &module.syntax,
                &live_set,
                live_usage_by_file.get(source_file),
            ),
        );
    }

    Ok(Some(transformed_sources))
}

fn support_item_dependency_tokens(item: &Item) -> TokenStream {
    let Item::Mod(item_mod) = item else {
        return item.to_token_stream();
    };
    if item_mod.content.is_none() {
        return item.to_token_stream();
    }
    let mut item_mod = item_mod.clone();
    item_mod.content = item_mod.content.map(|(brace, _items)| (brace, Vec::new()));
    item_mod.to_token_stream()
}

fn mark_support_token_dependencies(
    ctx: &SupportResolveContext<'_>,
    source_file: &Path,
    tokens: &TokenStream,
    named_items: &BTreeMap<String, Item>,
    live: &mut BTreeMap<PathBuf, SupportLiveSet>,
    live_usage: &mut TokenUsage,
    allow_bare_ident_dependencies: bool,
) -> Option<bool> {
    let mut changed = false;
    collect_token_usage(tokens, live_usage);
    let path_candidates = token_path_candidates(tokens);

    for candidate in named_items.keys() {
        if live
            .get(source_file)
            .is_some_and(|live_set| live_set.item_names.contains(candidate))
        {
            continue;
        }
        let Some(candidate_item) = named_items.get(candidate) else {
            continue;
        };
        if matches!(candidate_item, Item::Mod(item_mod) if item_mod.content.is_none()) {
            continue;
        }
        let macro_invocation_names_local_macro = matches!(candidate_item, Item::Macro(_))
            && path_candidates
                .iter()
                .any(|segments| matches!(segments.as_slice(), [first] if first == candidate));
        if macro_invocation_names_local_macro
            || (allow_bare_ident_dependencies && token_stream_mentions_ident(tokens, candidate))
            || path_candidates.iter().any(|segments| {
                segments.len() > 1 && segments.first().is_some_and(|first| first == candidate)
            })
        {
            changed |= live
                .entry(source_file.to_path_buf())
                .or_default()
                .item_names
                .insert(candidate.clone());
        }
    }

    for segments in token_path_candidates(tokens) {
        if let Some(inserted) =
            mark_support_local_assoc_dependencies(ctx, source_file, &segments, live)
        {
            changed |= inserted;
        }
        let inserted = mark_support_path_target(ctx, source_file, &segments, live)?;
        changed |= inserted;
    }

    Some(changed)
}

fn mark_support_local_assoc_dependencies(
    ctx: &SupportResolveContext<'_>,
    source_file: &Path,
    segments: &[String],
    live: &mut BTreeMap<PathBuf, SupportLiveSet>,
) -> Option<bool> {
    let Some((prefix, type_name, assoc_name)) = support_required_assoc_segments(segments) else {
        return Some(false);
    };
    if matches!(
        prefix.first().map(String::as_str),
        Some("crate" | "self" | "super")
    ) {
        return mark_support_path_target(ctx, source_file, segments, live);
    }
    let mut target_file = source_file.to_path_buf();
    for segment in &prefix {
        let child_file = support_child_module_file(ctx.modules, &target_file, segment)?;
        target_file = child_file;
    }
    let Some(module) = ctx.modules.get(&target_file) else {
        return None;
    };
    let named_items = support_named_item_names(&module.syntax.items);
    if !named_items.contains_key(&type_name) {
        return Some(false);
    }
    let live_set = live.entry(target_file).or_default();
    let inserted_type = live_set.item_names.insert(type_name.clone());
    let inserted_assoc = live_set
        .assoc_item_names
        .entry(type_name)
        .or_default()
        .insert(assoc_name);
    Some(inserted_type | inserted_assoc)
}

fn collect_support_module_sources_with_syntax(
    package_root: &Path,
    source_file: &Path,
    module_dir: &Path,
    parent_file: Option<PathBuf>,
    syntax: syn::File,
    modules: &mut BTreeMap<PathBuf, SupportModuleSource>,
) -> Result<bool, Box<dyn std::error::Error>> {
    let Ok(source_file) = source_file.canonicalize() else {
        return Ok(false);
    };
    if !source_file.starts_with(package_root) {
        return Ok(false);
    }
    if modules.contains_key(&source_file) {
        return Ok(true);
    }
    modules.insert(
        source_file.clone(),
        SupportModuleSource {
            module_dir: module_dir.to_path_buf(),
            parent_file,
            syntax: syntax.clone(),
            inline: false,
        },
    );

    for item in &syntax.items {
        let Item::Mod(item_mod) = item else {
            continue;
        };
        if attrs_are_test(&item_mod.attrs) {
            continue;
        }
        if let Some((_brace, items)) = &item_mod.content {
            let inline_file = support_inline_module_file(&source_file, &item_mod.ident.to_string());
            let inline_module_dir =
                module_dir.join(module_source_name(&item_mod.ident.to_string()));
            if !collect_support_inline_module_sources_with_syntax(
                package_root,
                &inline_file,
                &inline_module_dir,
                source_file.clone(),
                syn::File {
                    shebang: None,
                    attrs: Vec::new(),
                    items: items.clone(),
                },
                modules,
            )? {
                return Ok(false);
            }
            continue;
        }
        let Some((child_file, child_dir)) =
            support_external_module_source(package_root, module_dir, item_mod)
        else {
            return Ok(false);
        };
        let text = match fs::read_to_string(&child_file) {
            Ok(text) => text,
            Err(_) => return Ok(false),
        };
        let child_syntax = match syn::parse_file(&text) {
            Ok(syntax) => syntax,
            Err(_) => return Ok(false),
        };
        if !collect_support_module_sources_with_syntax(
            package_root,
            &child_file,
            &child_dir,
            Some(source_file.clone()),
            child_syntax,
            modules,
        )? {
            return Ok(false);
        }
    }

    Ok(true)
}

fn collect_support_inline_module_sources_with_syntax(
    package_root: &Path,
    source_file: &Path,
    module_dir: &Path,
    parent_file: PathBuf,
    syntax: syn::File,
    modules: &mut BTreeMap<PathBuf, SupportModuleSource>,
) -> Result<bool, Box<dyn std::error::Error>> {
    if modules.contains_key(source_file) {
        return Ok(true);
    }
    modules.insert(
        source_file.to_path_buf(),
        SupportModuleSource {
            module_dir: module_dir.to_path_buf(),
            parent_file: Some(parent_file.clone()),
            syntax: syntax.clone(),
            inline: true,
        },
    );

    for item in &syntax.items {
        let Item::Mod(item_mod) = item else {
            continue;
        };
        if attrs_are_test(&item_mod.attrs) {
            continue;
        }
        if let Some((_brace, items)) = &item_mod.content {
            let inline_file = support_inline_module_file(source_file, &item_mod.ident.to_string());
            let inline_module_dir =
                module_dir.join(module_source_name(&item_mod.ident.to_string()));
            if !collect_support_inline_module_sources_with_syntax(
                package_root,
                &inline_file,
                &inline_module_dir,
                source_file.to_path_buf(),
                syn::File {
                    shebang: None,
                    attrs: Vec::new(),
                    items: items.clone(),
                },
                modules,
            )? {
                return Ok(false);
            }
            continue;
        }
        let Some((child_file, child_dir)) =
            support_external_module_source(package_root, module_dir, item_mod)
        else {
            return Ok(false);
        };
        let text = match fs::read_to_string(&child_file) {
            Ok(text) => text,
            Err(_) => return Ok(false),
        };
        let child_syntax = match syn::parse_file(&text) {
            Ok(syntax) => syntax,
            Err(_) => return Ok(false),
        };
        if !collect_support_module_sources_with_syntax(
            package_root,
            &child_file,
            &child_dir,
            Some(source_file.to_path_buf()),
            child_syntax,
            modules,
        )? {
            return Ok(false);
        }
    }

    Ok(true)
}

fn seed_support_required_name(
    ctx: &SupportResolveContext<'_>,
    required_name: &str,
    live: &mut BTreeMap<PathBuf, SupportLiveSet>,
) -> bool {
    let Some(root_module) = ctx.modules.get(ctx.root_file) else {
        return false;
    };
    if let Some((prefix, type_name, assoc_name)) = support_required_assoc_path(required_name) {
        let mut visited = BTreeSet::new();
        return matches!(
            mark_support_use_target_with_assoc(
                ctx,
                ctx.root_file,
                &prefix,
                &type_name,
                Some(&assoc_name),
                live,
                &mut visited
            ),
            SupportReexportMark::Matched(_)
        );
    }
    if let Some((prefix, target_name)) = support_required_path(required_name) {
        let mut visited = BTreeSet::new();
        return matches!(
            mark_support_use_target(
                ctx,
                ctx.root_file,
                &prefix,
                &target_name,
                live,
                &mut visited
            ),
            SupportReexportMark::Matched(_)
        );
    }
    let named_items = support_named_item_names(&root_module.syntax.items);
    if named_items.contains_key(required_name) {
        live.entry(ctx.root_file.to_path_buf())
            .or_default()
            .item_names
            .insert(required_name.to_string());
        return true;
    }

    for item in &root_module.syntax.items {
        let Item::Use(item_use) = item else {
            continue;
        };
        if !use_is_reexport(&item_use.vis) {
            continue;
        }
        let mut visited = BTreeSet::new();
        match mark_support_required_reexport(
            ctx,
            ctx.root_file,
            &item_use.tree,
            Vec::new(),
            required_name,
            None,
            live,
            &mut visited,
        ) {
            SupportReexportMark::Matched(_) => {
                live.entry(ctx.root_file.to_path_buf())
                    .or_default()
                    .public_exports
                    .insert(required_name.to_string());
                return true;
            }
            SupportReexportMark::Unsupported => return false,
            SupportReexportMark::NotMatched => {}
        }
    }

    false
}

fn support_required_path(required_name: &str) -> Option<(Vec<String>, String)> {
    let segments = required_name
        .split("::")
        .filter(|segment| !segment.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    if segments.len() < 2 {
        return None;
    }
    let target_name = segments.last()?.clone();
    Some((segments[..segments.len() - 1].to_vec(), target_name))
}

fn support_required_assoc_path(required_name: &str) -> Option<(Vec<String>, String, String)> {
    let segments = required_name
        .split("::")
        .filter(|segment| !segment.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    support_required_assoc_segments(&segments)
}

fn support_required_assoc_segments(segments: &[String]) -> Option<(Vec<String>, String, String)> {
    if segments.len() < 2 {
        return None;
    }
    let assoc_name = segments.last()?;
    if !support_assoc_item_name_is_precise(assoc_name) {
        return None;
    }
    let type_name = segments.get(segments.len() - 2)?;
    if !support_type_like_ident(type_name) {
        return None;
    }
    Some((
        segments[..segments.len() - 2].to_vec(),
        type_name.clone(),
        assoc_name.clone(),
    ))
}

fn record_required_external_reexport_usage(
    root_syntax: &syn::File,
    dependency_roots: &BTreeSet<String>,
    required_names: &BTreeSet<String>,
    usage: &mut TokenUsage,
) {
    for required_name in required_names {
        let (visible_path, assoc_name) = if let Some((prefix, type_name, assoc_name)) =
            support_required_assoc_path(required_name)
        {
            let mut visible_path = prefix;
            visible_path.push(type_name);
            (visible_path, Some(assoc_name))
        } else {
            (
                required_name
                    .split("::")
                    .filter(|segment| !segment.is_empty())
                    .map(str::to_string)
                    .collect::<Vec<_>>(),
                None,
            )
        };
        let Some(visible_name) = visible_path.last() else {
            continue;
        };
        for item in &root_syntax.items {
            let Item::Use(item_use) = item else {
                continue;
            };
            if !use_is_reexport(&item_use.vis) {
                continue;
            }
            record_required_external_reexport_use_tree(
                &item_use.tree,
                Vec::new(),
                dependency_roots,
                visible_name,
                assoc_name.as_deref(),
                usage,
            );
        }
    }
}

fn record_required_external_reexport_use_tree(
    tree: &UseTree,
    mut prefix: Vec<String>,
    dependency_roots: &BTreeSet<String>,
    visible_name: &str,
    assoc_name: Option<&str>,
    usage: &mut TokenUsage,
) {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            record_required_external_reexport_use_tree(
                &path.tree,
                prefix,
                dependency_roots,
                visible_name,
                assoc_name,
                usage,
            );
        }
        UseTree::Name(name) => {
            if name.ident != visible_name {
                return;
            }
            record_required_external_reexport_path(
                &prefix,
                &name.ident.to_string(),
                dependency_roots,
                assoc_name,
                usage,
            );
        }
        UseTree::Rename(rename) => {
            if rename.rename != visible_name {
                return;
            }
            record_required_external_reexport_path(
                &prefix,
                &rename.ident.to_string(),
                dependency_roots,
                assoc_name,
                usage,
            );
        }
        UseTree::Group(group) => {
            for item in &group.items {
                record_required_external_reexport_use_tree(
                    item,
                    prefix.clone(),
                    dependency_roots,
                    visible_name,
                    assoc_name,
                    usage,
                );
            }
        }
        UseTree::Glob(_) => {}
    }
}

fn record_required_external_reexport_path(
    prefix: &[String],
    imported_name: &str,
    dependency_roots: &BTreeSet<String>,
    assoc_name: Option<&str>,
    usage: &mut TokenUsage,
) {
    let Some(root) = prefix.first() else {
        return;
    };
    if !dependency_roots.contains(root) {
        return;
    }
    let mut imported_path = prefix[1..].to_vec();
    imported_path.push(imported_name.to_string());
    if let Some(assoc_name) = assoc_name {
        imported_path.push(assoc_name.to_string());
    }
    if let Some(required_path) = dependency_required_path_string(&imported_path) {
        usage
            .dependency_public_names
            .entry(root.clone())
            .or_default()
            .insert(required_path);
    }
}

fn support_type_like_ident(ident: &str) -> bool {
    ident
        .chars()
        .next()
        .is_some_and(|ch| ch == '_' || ch.is_ascii_uppercase())
}

fn support_assoc_item_name_is_precise(ident: &str) -> bool {
    ident
        .chars()
        .next()
        .is_some_and(|ch| ch == '_' || ch.is_ascii_lowercase())
}

fn mark_support_live_assoc_item(
    live: &mut BTreeMap<PathBuf, SupportLiveSet>,
    source_file: &Path,
    type_name: &str,
    assoc_name: &str,
) -> bool {
    live.entry(source_file.to_path_buf())
        .or_default()
        .assoc_item_names
        .entry(type_name.to_string())
        .or_default()
        .insert(assoc_name.to_string())
}

fn mark_support_required_reexport(
    ctx: &SupportResolveContext<'_>,
    source_file: &Path,
    tree: &UseTree,
    mut prefix: Vec<String>,
    required_name: &str,
    assoc_name: Option<&str>,
    live: &mut BTreeMap<PathBuf, SupportLiveSet>,
    visited: &mut BTreeSet<(PathBuf, String)>,
) -> SupportReexportMark {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            mark_support_required_reexport(
                ctx,
                source_file,
                &path.tree,
                prefix,
                required_name,
                assoc_name,
                live,
                visited,
            )
        }
        UseTree::Name(name) => {
            if name.ident != required_name {
                return SupportReexportMark::NotMatched;
            }
            mark_support_use_target_with_assoc(
                ctx,
                source_file,
                &prefix,
                required_name,
                assoc_name,
                live,
                visited,
            )
        }
        UseTree::Rename(rename) => {
            if rename.rename != required_name {
                return SupportReexportMark::NotMatched;
            }
            mark_support_use_target_with_assoc(
                ctx,
                source_file,
                &prefix,
                &rename.ident.to_string(),
                assoc_name,
                live,
                visited,
            )
        }
        UseTree::Group(group) => {
            let mut matched = false;
            let mut inserted = false;
            for item in &group.items {
                match mark_support_required_reexport(
                    ctx,
                    source_file,
                    item,
                    prefix.clone(),
                    required_name,
                    assoc_name,
                    live,
                    visited,
                ) {
                    SupportReexportMark::Matched(item_inserted) => {
                        matched = true;
                        inserted |= item_inserted;
                    }
                    SupportReexportMark::Unsupported => return SupportReexportMark::Unsupported,
                    SupportReexportMark::NotMatched => {}
                }
            }
            if matched {
                SupportReexportMark::Matched(inserted)
            } else {
                SupportReexportMark::NotMatched
            }
        }
        UseTree::Glob(_) => {
            if prefix.is_empty() {
                return SupportReexportMark::NotMatched;
            }
            match support_local_use_prefix_target(ctx, source_file, &prefix) {
                SupportLocalTarget::Local => mark_support_use_target_with_assoc(
                    ctx,
                    source_file,
                    &prefix,
                    required_name,
                    assoc_name,
                    live,
                    visited,
                ),
                SupportLocalTarget::External => SupportReexportMark::Unsupported,
                SupportLocalTarget::Unsupported => SupportReexportMark::Unsupported,
            }
        }
    }
}

fn mark_support_live_use_imports(
    ctx: &SupportResolveContext<'_>,
    source_file: &Path,
    tree: &UseTree,
    mut prefix: Vec<String>,
    needs: SupportUseNeeds<'_>,
    live: &mut BTreeMap<PathBuf, SupportLiveSet>,
) -> Option<bool> {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            mark_support_live_use_imports(ctx, source_file, &path.tree, prefix, needs, live)
        }
        UseTree::Name(name) => {
            let imported_name = name.ident.to_string();
            let visible_name = if name.ident == "self" {
                prefix
                    .last()
                    .cloned()
                    .unwrap_or_else(|| imported_name.clone())
            } else {
                imported_name.clone()
            };
            let is_live_name = needs.live_idents.contains(&visible_name)
                || needs.live_exports.contains(&visible_name);
            let live_assoc_items = needs.live_assoc_items.get(&visible_name);
            if is_live_name || live_assoc_items.is_some() {
                let mut inserted = false;
                let mut visited = BTreeSet::new();
                if imported_name != "self" {
                    inserted |= support_reexport_mark_to_option(mark_support_use_target(
                        ctx,
                        source_file,
                        &prefix,
                        &imported_name,
                        live,
                        &mut visited,
                    ))?;
                }
                let target_prefix = support_import_target_prefix(&prefix, &imported_name);
                inserted |= mark_support_visible_path_dependencies(
                    ctx,
                    source_file,
                    &target_prefix,
                    &visible_name,
                    needs.live_paths,
                    live,
                )?;
                if let Some(assoc_items) = live_assoc_items {
                    inserted |= mark_support_import_assoc_requirements(
                        ctx,
                        source_file,
                        &target_prefix,
                        "",
                        assoc_items,
                        live,
                    )?;
                }
                return Some(inserted);
            }
            Some(false)
        }
        UseTree::Rename(rename) => {
            let local_name = rename.rename.to_string();
            let is_live_name =
                needs.live_idents.contains(&local_name) || needs.live_exports.contains(&local_name);
            let live_assoc_items = needs.live_assoc_items.get(&local_name);
            if is_live_name || live_assoc_items.is_some() {
                let mut inserted = false;
                let mut visited = BTreeSet::new();
                let imported_name = rename.ident.to_string();
                let target_prefix = support_import_target_prefix(&prefix, &imported_name);
                if imported_name != "self" {
                    inserted |= support_reexport_mark_to_option(mark_support_use_target(
                        ctx,
                        source_file,
                        &prefix,
                        &imported_name,
                        live,
                        &mut visited,
                    ))?;
                }
                inserted |= mark_support_visible_path_dependencies(
                    ctx,
                    source_file,
                    &target_prefix,
                    &local_name,
                    needs.live_paths,
                    live,
                )?;
                if let Some(assoc_items) = live_assoc_items {
                    inserted |= mark_support_import_assoc_requirements(
                        ctx,
                        source_file,
                        &target_prefix,
                        "",
                        assoc_items,
                        live,
                    )?;
                }
                return Some(inserted);
            }
            Some(false)
        }
        UseTree::Group(group) => {
            let mut inserted = false;
            for item in &group.items {
                inserted |= mark_support_live_use_imports(
                    ctx,
                    source_file,
                    item,
                    prefix.clone(),
                    SupportUseNeeds {
                        live_idents: needs.live_idents,
                        live_exports: needs.live_exports,
                        live_assoc_items: needs.live_assoc_items,
                        live_paths: needs.live_paths,
                    },
                    live,
                )?;
            }
            Some(inserted)
        }
        UseTree::Glob(_) => mark_support_live_glob_imports(ctx, source_file, &prefix, needs, live),
    }
}

fn support_import_target_prefix(prefix: &[String], imported_name: &str) -> Vec<String> {
    let mut target_prefix = prefix.to_vec();
    if imported_name != "self" {
        target_prefix.push(imported_name.to_string());
    }
    target_prefix
}

fn mark_support_visible_path_dependencies(
    ctx: &SupportResolveContext<'_>,
    source_file: &Path,
    target_prefix: &[String],
    visible_name: &str,
    live_paths: &BTreeSet<Vec<String>>,
    live: &mut BTreeMap<PathBuf, SupportLiveSet>,
) -> Option<bool> {
    let mut inserted = false;
    for path in live_paths {
        let Some((root, rest)) = path.split_first() else {
            continue;
        };
        if root != visible_name || rest.is_empty() {
            continue;
        }
        let mut target_path = target_prefix.to_vec();
        target_path.extend(rest.iter().cloned());
        inserted |= mark_support_path_target(ctx, source_file, &target_path, live)?;
    }
    Some(inserted)
}

fn support_combined_assoc_items(
    first: &BTreeMap<String, BTreeSet<String>>,
    second: &BTreeMap<String, BTreeSet<String>>,
) -> BTreeMap<String, BTreeSet<String>> {
    let mut combined = first.clone();
    for (name, assoc_items) in second {
        combined
            .entry(name.clone())
            .or_default()
            .extend(assoc_items.iter().cloned());
    }
    combined
}

fn support_reexport_mark_to_option(mark: SupportReexportMark) -> Option<bool> {
    match mark {
        SupportReexportMark::Matched(inserted) => Some(inserted),
        SupportReexportMark::NotMatched => Some(false),
        SupportReexportMark::Unsupported => None,
    }
}

fn support_use_tree_imports_live_name(
    tree: &UseTree,
    mut prefix: Vec<String>,
    live_idents: &BTreeSet<String>,
    live_exports: &BTreeSet<String>,
) -> bool {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            support_use_tree_imports_live_name(&path.tree, prefix, live_idents, live_exports)
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
            live_idents.contains(&visible_name) || live_exports.contains(&visible_name)
        }
        UseTree::Rename(rename) => {
            let visible_name = rename.rename.to_string();
            live_idents.contains(&visible_name) || live_exports.contains(&visible_name)
        }
        UseTree::Group(group) => group.items.iter().any(|item| {
            support_use_tree_imports_live_name(item, prefix.clone(), live_idents, live_exports)
        }),
        UseTree::Glob(_) => false,
    }
}

fn mark_support_live_glob_imports(
    ctx: &SupportResolveContext<'_>,
    source_file: &Path,
    prefix: &[String],
    needs: SupportUseNeeds<'_>,
    live: &mut BTreeMap<PathBuf, SupportLiveSet>,
) -> Option<bool> {
    match support_local_glob_prefix_target(ctx, source_file, prefix) {
        SupportLocalGlobTarget::External => {
            if needs.live_idents.is_empty()
                && needs.live_exports.is_empty()
                && needs.live_assoc_items.is_empty()
            {
                Some(false)
            } else {
                None
            }
        }
        SupportLocalGlobTarget::Unsupported => None,
        SupportLocalGlobTarget::Enum {
            source_file,
            enum_name,
            variants,
        } => {
            if needs.live_assoc_items.contains_key(&enum_name) {
                return None;
            }
            let variant_is_live = variants.iter().any(|variant| {
                needs.live_idents.contains(variant) || needs.live_exports.contains(variant)
            });
            if !variant_is_live {
                return Some(false);
            }
            Some(
                live.entry(source_file)
                    .or_default()
                    .item_names
                    .insert(enum_name),
            )
        }
        SupportLocalGlobTarget::Module(_) => {
            let mut inserted = false;
            for name in needs.live_idents.union(needs.live_exports) {
                let mut visited = BTreeSet::new();
                match mark_support_use_target(ctx, source_file, prefix, name, live, &mut visited) {
                    SupportReexportMark::Matched(name_inserted) => {
                        inserted |= name_inserted;
                    }
                    SupportReexportMark::NotMatched => {}
                    SupportReexportMark::Unsupported => return None,
                }
            }
            for (visible_name, assoc_items) in needs.live_assoc_items {
                inserted |= mark_support_import_assoc_requirements(
                    ctx,
                    source_file,
                    prefix,
                    visible_name,
                    assoc_items,
                    live,
                )?;
            }
            Some(inserted)
        }
    }
}

fn mark_support_import_assoc_requirements(
    ctx: &SupportResolveContext<'_>,
    source_file: &Path,
    prefix: &[String],
    imported_name: &str,
    assoc_items: &BTreeSet<String>,
    live: &mut BTreeMap<PathBuf, SupportLiveSet>,
) -> Option<bool> {
    let mut inserted = false;
    for assoc_item in assoc_items {
        let assoc_segments = assoc_item
            .split("::")
            .filter(|segment| !segment.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>();
        if assoc_segments.len() == 1
            && !assoc_segments
                .first()
                .is_some_and(|segment| support_assoc_item_name_is_precise(segment))
        {
            continue;
        }
        let mut segments = prefix.to_vec();
        if !imported_name.is_empty() {
            segments.push(imported_name.to_string());
        }
        segments.extend(assoc_segments);
        inserted |= mark_support_path_target(ctx, source_file, &segments, live)?;
    }
    Some(inserted)
}

fn mark_support_path_target(
    ctx: &SupportResolveContext<'_>,
    source_file: &Path,
    segments: &[String],
    live: &mut BTreeMap<PathBuf, SupportLiveSet>,
) -> Option<bool> {
    let (target_file, target_segments): (&Path, &[String]) = match segments {
        [first, rest @ ..] if first == "self" => (source_file, rest),
        [first, rest @ ..] if first == "crate" => (ctx.root_file, rest),
        [first, rest @ ..] if first == "super" => {
            let parent = support_parent_module_file(ctx.modules, source_file)?;
            (parent.as_path(), rest)
        }
        _ => (source_file, segments),
    };
    if target_segments.len() < 2 {
        return Some(false);
    }
    if let Some((prefix, type_name, assoc_name)) = support_required_assoc_segments(target_segments)
    {
        let mut visited = BTreeSet::new();
        return support_reexport_mark_to_option(mark_support_use_target_with_assoc(
            ctx,
            target_file,
            &prefix,
            &type_name,
            Some(&assoc_name),
            live,
            &mut visited,
        ));
    }
    let Some((target_name, prefix)) = target_segments.split_last() else {
        return Some(false);
    };
    let mut visited = BTreeSet::new();
    support_reexport_mark_to_option(mark_support_use_target(
        ctx,
        target_file,
        prefix,
        target_name,
        live,
        &mut visited,
    ))
}

fn mark_support_use_target(
    ctx: &SupportResolveContext<'_>,
    source_file: &Path,
    prefix: &[String],
    target_name: &str,
    live: &mut BTreeMap<PathBuf, SupportLiveSet>,
    visited: &mut BTreeSet<(PathBuf, String)>,
) -> SupportReexportMark {
    mark_support_use_target_with_assoc(ctx, source_file, prefix, target_name, None, live, visited)
}

fn mark_support_use_target_with_assoc(
    ctx: &SupportResolveContext<'_>,
    source_file: &Path,
    prefix: &[String],
    target_name: &str,
    assoc_name: Option<&str>,
    live: &mut BTreeMap<PathBuf, SupportLiveSet>,
    visited: &mut BTreeSet<(PathBuf, String)>,
) -> SupportReexportMark {
    let mut source_file = source_file;
    let mut prefix = prefix;
    if let Some(first) = prefix.first() {
        match first.as_str() {
            "self" => {
                prefix = &prefix[1..];
            }
            "crate" => {
                source_file = ctx.root_file;
                prefix = &prefix[1..];
            }
            "super" => {
                let Some(parent) = support_parent_module_file(ctx.modules, source_file) else {
                    return SupportReexportMark::Unsupported;
                };
                source_file = parent.as_path();
                prefix = &prefix[1..];
            }
            _ => {}
        }
    }

    let Some(first) = prefix.first() else {
        return seed_support_name_in_file(ctx, source_file, target_name, assoc_name, live, visited);
    };

    let Some(child_file) = support_child_module_file(ctx.modules, source_file, first) else {
        match mark_support_reexported_module_prefix_with_assoc(
            ctx,
            source_file,
            first,
            &prefix[1..],
            target_name,
            assoc_name,
            live,
            visited,
        ) {
            SupportReexportMark::NotMatched => {}
            matched => return matched,
        }
        if ctx.dependency_roots.contains(first) {
            return SupportReexportMark::Matched(false);
        }
        return SupportReexportMark::NotMatched;
    };
    if prefix.len() == 1 {
        return match seed_support_name_in_file(
            ctx,
            &child_file,
            target_name,
            assoc_name,
            live,
            visited,
        ) {
            SupportReexportMark::Matched(inserted_child) => {
                let inserted_module = live
                    .entry(source_file.to_path_buf())
                    .or_default()
                    .item_names
                    .insert(first.clone());
                SupportReexportMark::Matched(inserted_module | inserted_child)
            }
            other => other,
        };
    }
    match mark_support_use_target_with_assoc(
        ctx,
        &child_file,
        &prefix[1..],
        target_name,
        assoc_name,
        live,
        visited,
    ) {
        SupportReexportMark::Matched(inserted_child) => {
            let inserted_module = live
                .entry(source_file.to_path_buf())
                .or_default()
                .item_names
                .insert(first.clone());
            SupportReexportMark::Matched(inserted_module | inserted_child)
        }
        other => other,
    }
}

fn mark_support_reexported_module_prefix_with_assoc(
    ctx: &SupportResolveContext<'_>,
    source_file: &Path,
    visible_prefix: &str,
    remaining_prefix: &[String],
    target_name: &str,
    assoc_name: Option<&str>,
    live: &mut BTreeMap<PathBuf, SupportLiveSet>,
    visited: &mut BTreeSet<(PathBuf, String)>,
) -> SupportReexportMark {
    let Some(module) = ctx.modules.get(source_file) else {
        return SupportReexportMark::Unsupported;
    };
    let mut matched = false;
    let mut inserted = false;
    for item in &module.syntax.items {
        let Item::Use(item_use) = item else {
            continue;
        };
        if !use_is_reexport(&item_use.vis) {
            continue;
        }
        for mut target_prefix in support_reexported_module_prefixes(&item_use.tree, visible_prefix)
        {
            if !dependency_alias_target_is_module_like(visible_prefix, &target_prefix) {
                continue;
            }
            target_prefix.extend(remaining_prefix.iter().cloned());
            match mark_support_use_target_with_assoc(
                ctx,
                source_file,
                &target_prefix,
                target_name,
                assoc_name,
                live,
                visited,
            ) {
                SupportReexportMark::Matched(item_inserted) => {
                    matched = true;
                    inserted |= item_inserted;
                    inserted |= live
                        .entry(source_file.to_path_buf())
                        .or_default()
                        .public_exports
                        .insert(visible_prefix.to_string());
                }
                SupportReexportMark::Unsupported => return SupportReexportMark::Unsupported,
                SupportReexportMark::NotMatched => {}
            }
        }
    }
    if matched {
        SupportReexportMark::Matched(inserted)
    } else {
        SupportReexportMark::NotMatched
    }
}

fn support_reexported_module_prefixes(tree: &UseTree, visible_name: &str) -> Vec<Vec<String>> {
    fn collect(
        tree: &UseTree,
        mut prefix: Vec<String>,
        visible_name: &str,
        out: &mut Vec<Vec<String>>,
    ) {
        match tree {
            UseTree::Path(path) => {
                prefix.push(path.ident.to_string());
                collect(&path.tree, prefix, visible_name, out);
            }
            UseTree::Name(name) => {
                if name.ident == "self" && prefix.last().is_some_and(|name| name == visible_name) {
                    out.push(prefix);
                }
            }
            UseTree::Rename(rename) => {
                if rename.rename == visible_name {
                    let mut target = prefix;
                    if rename.ident != "self" {
                        target.push(rename.ident.to_string());
                    }
                    out.push(target);
                }
            }
            UseTree::Group(group) => {
                for item in &group.items {
                    collect(item, prefix.clone(), visible_name, out);
                }
            }
            UseTree::Glob(_) => {}
        }
    }

    let mut prefixes = Vec::new();
    collect(tree, Vec::new(), visible_name, &mut prefixes);
    prefixes
}

fn seed_support_name_in_file(
    ctx: &SupportResolveContext<'_>,
    source_file: &Path,
    required_name: &str,
    assoc_name: Option<&str>,
    live: &mut BTreeMap<PathBuf, SupportLiveSet>,
    visited: &mut BTreeSet<(PathBuf, String)>,
) -> SupportReexportMark {
    let key = (
        source_file.to_path_buf(),
        assoc_name
            .map(|assoc_name| format!("{required_name}::{assoc_name}"))
            .unwrap_or_else(|| required_name.to_string()),
    );
    if !visited.insert(key) {
        return SupportReexportMark::NotMatched;
    }
    let Some(module) = ctx.modules.get(source_file) else {
        return SupportReexportMark::Unsupported;
    };
    let named_items = support_named_item_names(&module.syntax.items);
    if named_items.contains_key(required_name) {
        let live_set = live.entry(source_file.to_path_buf()).or_default();
        let inserted_item = live_set.item_names.insert(required_name.to_string());
        let inserted_assoc = assoc_name.is_some_and(|assoc_name| {
            live_set
                .assoc_item_names
                .entry(required_name.to_string())
                .or_default()
                .insert(assoc_name.to_string())
        });
        return SupportReexportMark::Matched(inserted_item | inserted_assoc);
    }

    for item in &module.syntax.items {
        let Item::Use(item_use) = item else {
            continue;
        };
        if !use_is_reexport(&item_use.vis) {
            continue;
        }
        match mark_support_required_reexport(
            ctx,
            source_file,
            &item_use.tree,
            Vec::new(),
            required_name,
            assoc_name,
            live,
            visited,
        ) {
            SupportReexportMark::Matched(inserted) => {
                let inserted_export = live
                    .entry(source_file.to_path_buf())
                    .or_default()
                    .public_exports
                    .insert(required_name.to_string());
                return SupportReexportMark::Matched(inserted | inserted_export);
            }
            SupportReexportMark::Unsupported => return SupportReexportMark::Unsupported,
            SupportReexportMark::NotMatched => {}
        }
    }

    SupportReexportMark::NotMatched
}

fn support_child_module_file(
    modules: &BTreeMap<PathBuf, SupportModuleSource>,
    source_file: &Path,
    module_name: &str,
) -> Option<PathBuf> {
    let module = modules.get(source_file)?;
    module.syntax.items.iter().find_map(|item| {
        let Item::Mod(item_mod) = item else {
            return None;
        };
        if item_mod.ident != module_name || attrs_are_test(&item_mod.attrs) {
            return None;
        }
        if item_mod.content.is_some() {
            let inline_file = support_inline_module_file(source_file, module_name);
            return modules.contains_key(&inline_file).then_some(inline_file);
        }
        support_external_module_source(Path::new("/"), &module.module_dir, item_mod)
            .map(|(child_file, _)| child_file)
    })
}

fn support_inline_module_file(source_file: &Path, module_name: &str) -> PathBuf {
    let mut key = source_file.as_os_str().to_os_string();
    key.push(format!("#inline:{module_name}"));
    PathBuf::from(key)
}

fn support_parent_module_file<'a>(
    modules: &'a BTreeMap<PathBuf, SupportModuleSource>,
    source_file: &Path,
) -> Option<&'a PathBuf> {
    modules.get(source_file)?.parent_file.as_ref()
}

fn support_local_use_prefix_target(
    ctx: &SupportResolveContext<'_>,
    source_file: &Path,
    prefix: &[String],
) -> SupportLocalTarget {
    let mut source_file = source_file;
    let mut prefix = prefix;
    if let Some(first) = prefix.first() {
        match first.as_str() {
            "self" => {
                prefix = &prefix[1..];
            }
            "crate" => {
                source_file = ctx.root_file;
                prefix = &prefix[1..];
            }
            "super" => {
                let Some(parent) = support_parent_module_file(ctx.modules, source_file) else {
                    return SupportLocalTarget::Unsupported;
                };
                source_file = parent.as_path();
                prefix = &prefix[1..];
            }
            _ => {}
        }
    }
    let Some(first) = prefix.first() else {
        return SupportLocalTarget::Unsupported;
    };
    let Some(mut target_file) = support_child_module_file(ctx.modules, source_file, first) else {
        return SupportLocalTarget::External;
    };
    for segment in &prefix[1..] {
        let Some(child_file) = support_child_module_file(ctx.modules, &target_file, segment) else {
            return SupportLocalTarget::Unsupported;
        };
        target_file = child_file;
    }
    SupportLocalTarget::Local
}

fn support_local_glob_prefix_target(
    ctx: &SupportResolveContext<'_>,
    source_file: &Path,
    prefix: &[String],
) -> SupportLocalGlobTarget {
    let mut source_file = source_file;
    let mut prefix = prefix;
    if let Some(first) = prefix.first() {
        match first.as_str() {
            "self" => {
                prefix = &prefix[1..];
            }
            "crate" => {
                source_file = ctx.root_file;
                prefix = &prefix[1..];
            }
            "super" => {
                let Some(parent) = support_parent_module_file(ctx.modules, source_file) else {
                    return SupportLocalGlobTarget::Unsupported;
                };
                source_file = parent.as_path();
                prefix = &prefix[1..];
            }
            _ => {}
        }
    }

    let Some(first) = prefix.first() else {
        return SupportLocalGlobTarget::Unsupported;
    };
    let mut target_file = source_file.to_path_buf();
    for (index, segment) in prefix.iter().enumerate() {
        if let Some(child_file) = support_child_module_file(ctx.modules, &target_file, segment) {
            if index + 1 == prefix.len() {
                return SupportLocalGlobTarget::Module(child_file);
            }
            target_file = child_file;
            continue;
        }

        if index == 0
            && segment == first
            && support_enum_variants_in_file(ctx.modules, source_file, segment).is_none()
        {
            return SupportLocalGlobTarget::External;
        }
        if index + 1 == prefix.len() {
            if let Some(variants) =
                support_enum_variants_in_file(ctx.modules, &target_file, segment)
            {
                return SupportLocalGlobTarget::Enum {
                    source_file: target_file,
                    enum_name: segment.clone(),
                    variants,
                };
            }
        }
        return SupportLocalGlobTarget::Unsupported;
    }

    SupportLocalGlobTarget::Unsupported
}

fn support_enum_variants_in_file(
    modules: &BTreeMap<PathBuf, SupportModuleSource>,
    source_file: &Path,
    enum_name: &str,
) -> Option<BTreeSet<String>> {
    let module = modules.get(source_file)?;
    module.syntax.items.iter().find_map(|item| {
        let Item::Enum(item_enum) = item else {
            return None;
        };
        if item_enum.ident != enum_name {
            return None;
        }
        Some(
            item_enum
                .variants
                .iter()
                .map(|variant| variant.ident.to_string())
                .collect(),
        )
    })
}

fn transform_restricted_support_file(
    ctx: &SupportResolveContext<'_>,
    live_by_file: &BTreeMap<PathBuf, SupportLiveSet>,
    source_file: &Path,
    syntax: &syn::File,
    live_set: &SupportLiveSet,
    live_usage_override: Option<&TokenUsage>,
) -> syn::File {
    let named_items = support_named_item_names(&syntax.items);
    let live_usage = live_usage_override
        .cloned()
        .unwrap_or_else(|| support_live_item_usage(syntax, live_set));
    let mut live_import_names = support_live_import_names(&live_usage);
    let non_enum_usage = support_live_non_enum_item_usage(syntax, live_set);
    let enum_variant_type_usage = support_live_enum_variant_type_usage(syntax, live_set);
    for variant_name in support_live_enum_variant_names(syntax, live_set) {
        if !non_enum_usage.bare_idents.contains(&variant_name)
            && !non_enum_usage.path_roots.contains(&variant_name)
            && !non_enum_usage
                .local_path_leaf_idents
                .contains(&variant_name)
            && !enum_variant_type_usage.bare_idents.contains(&variant_name)
            && !enum_variant_type_usage.path_roots.contains(&variant_name)
            && !enum_variant_type_usage
                .local_path_leaf_idents
                .contains(&variant_name)
        {
            live_import_names.remove(&variant_name);
        }
    }
    let live_derive_idents = support_live_derive_idents(syntax, live_set);
    let public_use_names = live_set
        .public_exports
        .union(&live_usage.idents)
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut transformed = syntax.clone();
    transformed.items = syntax
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Mod(item_mod) if item_mod.content.is_some() => {
                transform_support_inline_module(ctx, live_by_file, source_file, item_mod, live_set)
            }
            Item::Use(item_use) if use_is_public_api_reexport(&item_use.vis) => {
                let mut item_use = item_use.clone();
                item_use.tree = prune_support_public_use_tree(
                    ctx,
                    live_by_file,
                    source_file,
                    &item_use.tree,
                    Vec::new(),
                    &public_use_names,
                )?;
                Some(Item::Use(item_use))
            }
            Item::Use(item_use) => {
                let mut item_use = item_use.clone();
                item_use.tree = prune_support_private_use_tree(
                    ctx,
                    live_by_file,
                    source_file,
                    &item_use.tree,
                    Vec::new(),
                    &live_import_names,
                    &live_derive_idents,
                )?;
                Some(Item::Use(item_use))
            }
            Item::ExternCrate(_) => Some(item.clone()),
            Item::Impl(item_impl) => {
                transform_support_impl(item_impl, &named_items, live_set).map(Item::Impl)
            }
            _ => {
                if support_item_name(item).is_none_or(|name| live_set.item_names.contains(&name)) {
                    Some(item.clone())
                } else {
                    None
                }
            }
        })
        .collect();
    transformed
}

fn transform_support_inline_module(
    ctx: &SupportResolveContext<'_>,
    live_by_file: &BTreeMap<PathBuf, SupportLiveSet>,
    source_file: &Path,
    item_mod: &syn::ItemMod,
    parent_live_set: &SupportLiveSet,
) -> Option<Item> {
    let name = item_mod.ident.to_string();
    if !parent_live_set.item_names.contains(&name) {
        return None;
    }
    let child_file = support_child_module_file(ctx.modules, source_file, &name)?;
    let child_module = ctx.modules.get(&child_file)?;
    let child_live_set = live_by_file.get(&child_file).cloned().unwrap_or_default();
    let transformed = transform_restricted_support_file(
        ctx,
        live_by_file,
        &child_file,
        &child_module.syntax,
        &child_live_set,
        None,
    );
    let mut item_mod = item_mod.clone();
    item_mod.content = item_mod
        .content
        .map(|(brace, _items)| (brace, transformed.items));
    Some(Item::Mod(item_mod))
}

fn support_live_import_names(usage: &TokenUsage) -> BTreeSet<String> {
    usage
        .bare_idents
        .union(&usage.path_roots)
        .cloned()
        .chain(usage.local_path_leaf_idents.iter().cloned())
        .chain(usage.use_idents.iter().cloned())
        .collect()
}

fn support_live_enum_variant_names(
    syntax: &syn::File,
    live_set: &SupportLiveSet,
) -> BTreeSet<String> {
    syntax
        .items
        .iter()
        .filter_map(|item| {
            let Item::Enum(item_enum) = item else {
                return None;
            };
            live_set
                .item_names
                .contains(&item_enum.ident.to_string())
                .then_some(item_enum)
        })
        .flat_map(|item_enum| {
            item_enum
                .variants
                .iter()
                .map(|variant| variant.ident.to_string())
        })
        .collect()
}

fn support_live_item_usage(syntax: &syn::File, live_set: &SupportLiveSet) -> TokenUsage {
    let mut usage = TokenUsage::default();
    let named_items = support_named_item_names(&syntax.items);
    for item in &syntax.items {
        if support_item_should_collect_live_usage(item, &named_items, live_set) {
            collect_token_usage(&item.to_token_stream(), &mut usage);
        }
    }
    usage
}

fn support_live_non_enum_item_usage(syntax: &syn::File, live_set: &SupportLiveSet) -> TokenUsage {
    let mut usage = TokenUsage::default();
    let named_items = support_named_item_names(&syntax.items);
    for item in &syntax.items {
        if matches!(item, Item::Enum(_)) {
            continue;
        }
        if support_item_should_collect_live_usage(item, &named_items, live_set) {
            collect_token_usage(&item.to_token_stream(), &mut usage);
        }
    }
    usage
}

fn support_live_enum_variant_type_usage(
    syntax: &syn::File,
    live_set: &SupportLiveSet,
) -> TokenUsage {
    let mut usage = TokenUsage::default();
    for item in &syntax.items {
        let Item::Enum(item_enum) = item else {
            continue;
        };
        if !live_set.item_names.contains(&item_enum.ident.to_string()) {
            continue;
        }
        for variant in &item_enum.variants {
            for field in &variant.fields {
                collect_token_usage(&field.ty.to_token_stream(), &mut usage);
                for attr in &field.attrs {
                    collect_token_usage(&attr.to_token_stream(), &mut usage);
                }
            }
            for attr in &variant.attrs {
                collect_token_usage(&attr.to_token_stream(), &mut usage);
            }
            if let Some((_eq, discriminant)) = &variant.discriminant {
                collect_token_usage(&discriminant.to_token_stream(), &mut usage);
            }
        }
    }
    usage
}

fn support_live_derive_idents(syntax: &syn::File, live_set: &SupportLiveSet) -> BTreeSet<String> {
    let mut derives = BTreeSet::new();
    let named_items = support_named_item_names(&syntax.items);
    for item in &syntax.items {
        if !support_item_should_collect_live_usage(item, &named_items, live_set) {
            continue;
        }
        let Some(attrs) = item_attrs(item) else {
            continue;
        };
        collect_derive_attr_idents(attrs, &mut derives);
    }
    derives
}

fn support_item_should_collect_live_usage(
    item: &Item,
    named_items: &BTreeMap<String, Item>,
    live_set: &SupportLiveSet,
) -> bool {
    support_item_name(item).is_some_and(|name| {
        live_set.item_names.contains(&name) || live_set.public_exports.contains(&name)
    }) || matches!(item, Item::Impl(item_impl) if support_impl_should_render(item_impl, named_items, live_set))
        || matches!(item, Item::Macro(item_macro) if item_macro.ident.is_none())
}

fn support_method_index(
    modules: &BTreeMap<PathBuf, SupportModuleSource>,
) -> BTreeMap<String, Vec<SupportMethodCandidate>> {
    let mut index = BTreeMap::<String, Vec<SupportMethodCandidate>>::new();
    for (source_file, module) in modules {
        let named_items = support_named_item_names(&module.syntax.items);
        for item in &module.syntax.items {
            let Item::Impl(item_impl) = item else {
                continue;
            };
            if item_impl.trait_.is_some() {
                continue;
            }
            let Some(self_name) = support_impl_self_named_item(item_impl) else {
                continue;
            };
            if !named_items.contains_key(&self_name) {
                continue;
            }
            for impl_item in &item_impl.items {
                let ImplItem::Fn(item_fn) = impl_item else {
                    continue;
                };
                index
                    .entry(item_fn.sig.ident.to_string())
                    .or_default()
                    .push(SupportMethodCandidate {
                        source_file: source_file.clone(),
                        self_name: self_name.clone(),
                    });
            }
        }
    }
    index
}

fn support_item_source_index(
    modules: &BTreeMap<PathBuf, SupportModuleSource>,
) -> BTreeMap<String, Vec<PathBuf>> {
    let mut index = BTreeMap::<String, Vec<PathBuf>>::new();
    for (source_file, module) in modules {
        for name in support_named_item_names(&module.syntax.items).keys() {
            index
                .entry(name.clone())
                .or_default()
                .push(source_file.clone());
        }
    }
    index
}

fn support_enum_variant_payload_index(
    modules: &BTreeMap<PathBuf, SupportModuleSource>,
) -> BTreeMap<(String, String), SupportEnumVariantPayloads> {
    let item_sources = support_item_source_index(modules);
    let mut index = BTreeMap::<(String, String), SupportEnumVariantPayloads>::new();
    for module in modules.values() {
        for item in &module.syntax.items {
            let Item::Enum(item_enum) = item else {
                continue;
            };
            let enum_name = item_enum.ident.to_string();
            for variant in &item_enum.variants {
                let mut payloads = SupportEnumVariantPayloads::default();
                match &variant.fields {
                    Fields::Unnamed(fields) => {
                        payloads.unnamed = fields
                            .unnamed
                            .iter()
                            .map(|field| support_field_candidate(field, &item_sources))
                            .collect();
                    }
                    Fields::Named(fields) => {
                        for field in &fields.named {
                            let Some(ident) = &field.ident else {
                                continue;
                            };
                            if let Some(candidate) = support_field_candidate(field, &item_sources) {
                                payloads.named.insert(ident.to_string(), candidate);
                            }
                        }
                    }
                    Fields::Unit => {}
                }
                if !payloads.unnamed.is_empty() || !payloads.named.is_empty() {
                    index.insert((enum_name.clone(), variant.ident.to_string()), payloads);
                }
            }
        }
    }
    index
}

fn support_field_candidate(
    field: &Field,
    item_sources: &BTreeMap<String, Vec<PathBuf>>,
) -> Option<SupportMethodCandidate> {
    let Type::Path(type_path) = &field.ty else {
        return None;
    };
    if type_path.qself.is_some() {
        return None;
    }
    let type_name = type_path.path.segments.last()?.ident.to_string();
    let source_files = item_sources.get(&type_name)?;
    if source_files.len() != 1 {
        return None;
    }
    Some(SupportMethodCandidate {
        source_file: source_files[0].clone(),
        self_name: type_name,
    })
}

fn support_impl_dependency_tokens(
    item_impl: &syn::ItemImpl,
    named_items: &BTreeMap<String, Item>,
    live_set: &SupportLiveSet,
) -> Option<TokenStream> {
    transform_support_impl(item_impl, named_items, live_set)
        .map(|item_impl| item_impl.to_token_stream())
}

fn transform_support_impl(
    item_impl: &syn::ItemImpl,
    named_items: &BTreeMap<String, Item>,
    live_set: &SupportLiveSet,
) -> Option<syn::ItemImpl> {
    if !support_impl_should_render(item_impl, named_items, live_set) {
        return None;
    }
    if item_impl.trait_.is_some() {
        return Some(item_impl.clone());
    }
    let Some(self_name) = support_impl_self_named_item(item_impl) else {
        return Some(item_impl.clone());
    };
    let assoc_items = live_set
        .assoc_item_names
        .get(&self_name)
        .filter(|assoc_items| !assoc_items.is_empty());

    let mut pruned = item_impl.clone();
    pruned.items = item_impl
        .items
        .iter()
        .filter(|item| support_impl_item_should_render(item, assoc_items))
        .cloned()
        .collect();
    (!pruned.items.is_empty()).then_some(pruned)
}

fn support_impl_item_should_render(
    item: &ImplItem,
    assoc_items: Option<&BTreeSet<String>>,
) -> bool {
    match item {
        ImplItem::Fn(item_fn) => assoc_items
            .is_some_and(|assoc_items| assoc_items.contains(&item_fn.sig.ident.to_string())),
        // Associated consts, types, and macros can affect method signatures or macro-expanded
        // impl bodies. Keep them unless a future semantic pass proves an item-level decision.
        _ => true,
    }
}

fn support_impl_self_named_item(item_impl: &syn::ItemImpl) -> Option<String> {
    let Type::Path(type_path) = item_impl.self_ty.as_ref() else {
        return None;
    };
    type_path
        .path
        .segments
        .last()
        .map(|segment| segment.ident.to_string())
}

fn mark_support_item_assoc_usage_dependencies(
    ctx: &SupportResolveContext<'_>,
    source_file: &Path,
    item: &Item,
    named_items: &BTreeMap<String, Item>,
    live: &mut BTreeMap<PathBuf, SupportLiveSet>,
) -> Option<bool> {
    let mut visitor = SupportAssocUsageVisitor::new(
        None,
        support_macro_metavariable_method_requirements_from_named_items(named_items),
    );
    visitor.visit_item(item);
    mark_support_assoc_paths(ctx, source_file, visitor.assoc_paths, live)
}

fn mark_support_item_attribute_dependencies(
    ctx: &SupportResolveContext<'_>,
    source_file: &Path,
    item: &Item,
    live: &mut BTreeMap<PathBuf, SupportLiveSet>,
) -> Option<bool> {
    let mut visitor = SupportAttributePathVisitor::default();
    visitor.visit_item(item);
    mark_support_attr_paths(ctx, source_file, visitor.paths, live)
}

fn mark_support_method_fallback_dependencies(
    ctx: &SupportResolveContext<'_>,
    method_index: &BTreeMap<String, Vec<SupportMethodCandidate>>,
    item: &Item,
    live: &mut BTreeMap<PathBuf, SupportLiveSet>,
) -> bool {
    const SUPPORT_METHOD_FALLBACK_CAP: usize = 3;

    let mut visitor = SupportMethodNameVisitor::default();
    visitor.visit_item(item);
    visitor
        .names
        .extend(token_method_call_names(&item.to_token_stream()));
    let mut changed = false;
    for method in visitor.names {
        let Some(candidates) = method_index.get(&method) else {
            continue;
        };
        if candidates.len() > SUPPORT_METHOD_FALLBACK_CAP {
            continue;
        }
        for candidate in candidates {
            changed |= mark_support_live_item_name(
                ctx,
                &candidate.source_file,
                &candidate.self_name,
                live,
            );
            changed |= mark_support_live_assoc_item(
                live,
                &candidate.source_file,
                &candidate.self_name,
                &method,
            );
        }
    }
    changed
}

fn mark_support_enum_variant_method_dependencies(
    ctx: &SupportResolveContext<'_>,
    method_index: &BTreeMap<String, Vec<SupportMethodCandidate>>,
    variant_payload_index: &BTreeMap<(String, String), SupportEnumVariantPayloads>,
    item: &Item,
    live: &mut BTreeMap<PathBuf, SupportLiveSet>,
) -> bool {
    let mut visitor = SupportEnumVariantMethodVisitor {
        method_index,
        variant_payload_index,
        current_impl_self: None,
        calls: BTreeSet::new(),
    };
    visitor.visit_item(item);

    let mut changed = false;
    for (candidate, method) in visitor.calls {
        changed |=
            mark_support_live_item_name(ctx, &candidate.source_file, &candidate.self_name, live);
        changed |= mark_support_live_assoc_item(
            live,
            &candidate.source_file,
            &candidate.self_name,
            &method,
        );
    }
    changed
}

fn mark_support_impl_attribute_dependencies(
    ctx: &SupportResolveContext<'_>,
    source_file: &Path,
    item_impl: &syn::ItemImpl,
    live: &mut BTreeMap<PathBuf, SupportLiveSet>,
) -> Option<bool> {
    let mut visitor = SupportAttributePathVisitor::default();
    visitor.visit_item_impl(item_impl);
    mark_support_attr_paths(ctx, source_file, visitor.paths, live)
}

fn mark_support_live_item_name(
    ctx: &SupportResolveContext<'_>,
    source_file: &Path,
    item_name: &str,
    live: &mut BTreeMap<PathBuf, SupportLiveSet>,
) -> bool {
    let mut changed = live
        .entry(source_file.to_path_buf())
        .or_default()
        .item_names
        .insert(item_name.to_string());
    let mut current_file = source_file.to_path_buf();
    while let Some(parent_file) = support_parent_module_file(ctx.modules, &current_file).cloned() {
        let Some(module_name) = support_module_name_for_child(ctx, &parent_file, &current_file)
        else {
            break;
        };
        changed |= live
            .entry(parent_file.clone())
            .or_default()
            .item_names
            .insert(module_name);
        current_file = parent_file;
    }
    changed
}

fn support_module_name_for_child(
    ctx: &SupportResolveContext<'_>,
    parent_file: &Path,
    child_file: &Path,
) -> Option<String> {
    let parent = ctx.modules.get(parent_file)?;
    parent.syntax.items.iter().find_map(|item| {
        let Item::Mod(item_mod) = item else {
            return None;
        };
        if item_mod.content.is_some() || attrs_are_test(&item_mod.attrs) {
            return None;
        }
        let (candidate, _) =
            support_external_module_source(Path::new("/"), &parent.module_dir, item_mod)?;
        (candidate == child_file).then(|| item_mod.ident.to_string())
    })
}

fn mark_support_impl_self_assoc_dependencies(
    ctx: &SupportResolveContext<'_>,
    source_file: &Path,
    item_impl: &syn::ItemImpl,
    named_items: &BTreeMap<String, Item>,
    live_set: &SupportLiveSet,
    live: &mut BTreeMap<PathBuf, SupportLiveSet>,
) -> Option<bool> {
    let Some(self_name) = support_impl_self_named_item(item_impl) else {
        return Some(false);
    };
    let Some(rendered_impl) = transform_support_impl(item_impl, named_items, live_set) else {
        return Some(false);
    };
    let mut changed = false;
    let mut assoc_usage = SupportAssocUsageVisitor::new(
        Some(self_name.clone()),
        support_macro_metavariable_method_requirements_from_named_items(named_items),
    );
    for impl_item in &rendered_impl.items {
        let ImplItem::Fn(item_fn) = impl_item else {
            continue;
        };
        let mut visitor = SelfAssocUsageVisitor::default();
        visitor.visit_impl_item_fn(item_fn);
        for assoc_name in visitor.assoc_items {
            changed |= mark_support_live_assoc_item(live, source_file, &self_name, &assoc_name);
        }
        assoc_usage.visit_impl_item_fn(item_fn);
    }
    changed |= mark_support_assoc_paths(ctx, source_file, assoc_usage.assoc_paths, live)?;
    Some(changed)
}

fn mark_support_attr_paths(
    ctx: &SupportResolveContext<'_>,
    source_file: &Path,
    paths: BTreeSet<Vec<String>>,
    live: &mut BTreeMap<PathBuf, SupportLiveSet>,
) -> Option<bool> {
    let mut changed = false;
    for path in paths {
        changed |= match path.as_slice() {
            [] => false,
            [name] => {
                let mut visited = BTreeSet::new();
                support_reexport_mark_to_option(mark_support_use_target(
                    ctx,
                    source_file,
                    &[],
                    name,
                    live,
                    &mut visited,
                ))?
            }
            _ => mark_support_path_target(ctx, source_file, &path, live)?,
        };
    }
    Some(changed)
}

fn mark_support_assoc_paths(
    ctx: &SupportResolveContext<'_>,
    source_file: &Path,
    assoc_paths: BTreeSet<Vec<String>>,
    live: &mut BTreeMap<PathBuf, SupportLiveSet>,
) -> Option<bool> {
    let mut changed = false;
    for assoc_path in assoc_paths {
        changed |= mark_support_path_target(ctx, source_file, &assoc_path, live)?;
    }
    Some(changed)
}

#[derive(Default)]
struct SelfAssocUsageVisitor {
    assoc_items: BTreeSet<String>,
}

impl Visit<'_> for SelfAssocUsageVisitor {
    fn visit_expr_method_call(&mut self, node: &syn::ExprMethodCall) {
        if method_call_receiver_is_self(&node.receiver) {
            self.assoc_items.insert(node.method.to_string());
        }
        visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_call(&mut self, node: &syn::ExprCall) {
        if let syn::Expr::Path(path) = node.func.as_ref() {
            let segments = path
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<_>>();
            if let [self_name, assoc_name] = segments.as_slice() {
                if self_name == "Self" && support_assoc_item_name_is_precise(assoc_name) {
                    self.assoc_items.insert(assoc_name.clone());
                }
            }
        }
        visit::visit_expr_call(self, node);
    }
}

fn method_call_receiver_is_self(receiver: &Expr) -> bool {
    match receiver {
        Expr::Path(path) => path.path.is_ident("self"),
        Expr::Reference(reference) => method_call_receiver_is_self(&reference.expr),
        Expr::Paren(paren) => method_call_receiver_is_self(&paren.expr),
        Expr::Group(group) => method_call_receiver_is_self(&group.expr),
        _ => false,
    }
}

#[derive(Default)]
struct SupportAttributePathVisitor {
    paths: BTreeSet<Vec<String>>,
}

impl Visit<'_> for SupportAttributePathVisitor {
    fn visit_attribute(&mut self, attribute: &syn::Attribute) {
        collect_support_attribute_helper_paths(&attribute.meta, &mut self.paths);
        visit::visit_attribute(self, attribute);
    }
}

fn collect_support_attribute_helper_paths(meta: &Meta, paths: &mut BTreeSet<Vec<String>>) {
    match meta {
        Meta::Path(_) => {}
        Meta::NameValue(name_value) => {
            if support_helper_path_meta_key(&support_meta_path_key(&name_value.path)) {
                if let syn::Expr::Lit(expr_lit) = &name_value.value {
                    if let syn::Lit::Str(literal) = &expr_lit.lit {
                        collect_support_path_like_string(&literal.value(), paths);
                    }
                }
            }
        }
        Meta::List(list) => {
            if support_helper_path_meta_key(&support_meta_path_key(&list.path)) {
                if let Ok(literal) = syn::parse2::<syn::LitStr>(list.tokens.clone()) {
                    collect_support_path_like_string(&literal.value(), paths);
                    return;
                }
            }
            let Ok(arguments) =
                Punctuated::<Meta, syn::Token![,]>::parse_terminated.parse2(list.tokens.clone())
            else {
                return;
            };
            for nested in arguments {
                collect_support_attribute_helper_paths(&nested, paths);
            }
        }
    }
}

fn support_helper_path_meta_key(key: &str) -> bool {
    matches!(
        key,
        "default"
            | "deserialize_with"
            | "serialize_with"
            | "skip_serializing_if"
            | "with"
            | "serde_as"
    )
}

fn support_meta_path_key(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

fn collect_support_path_like_string(value: &str, paths: &mut BTreeSet<Vec<String>>) {
    let segments = value
        .split("::")
        .filter(|segment| !segment.is_empty())
        .filter(|segment| {
            segment
                .chars()
                .next()
                .is_some_and(|ch| ch == '_' || ch.is_ascii_alphabetic())
        })
        .map(str::to_string)
        .collect::<Vec<_>>();
    if !segments.is_empty() {
        paths.insert(segments);
    }
}

#[derive(Default)]
struct SupportMethodNameVisitor {
    names: BTreeSet<String>,
}

impl Visit<'_> for SupportMethodNameVisitor {
    fn visit_expr_method_call(&mut self, node: &syn::ExprMethodCall) {
        self.names.insert(node.method.to_string());
        visit::visit_expr_method_call(self, node);
    }
}

fn token_method_call_names(tokens: &TokenStream) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    collect_token_method_call_names(tokens, &mut names);
    names
}

fn collect_token_method_call_names(tokens: &TokenStream, names: &mut BTreeSet<String>) {
    let token_trees = tokens.clone().into_iter().collect::<Vec<_>>();
    for token in &token_trees {
        if let TokenTree::Group(group) = token {
            collect_token_method_call_names(&group.stream(), names);
        }
    }

    for window in token_trees.windows(3) {
        let [TokenTree::Punct(dot), TokenTree::Ident(method), TokenTree::Group(args)] = window
        else {
            continue;
        };
        if dot.as_char() == '.'
            && args.delimiter() == Delimiter::Parenthesis
            && support_assoc_item_name_is_precise(&method.to_string())
        {
            names.insert(method.to_string());
        }
    }
}

#[derive(Clone, Default)]
struct MacroMetavariableMethodRequirement {
    metavariables: Vec<String>,
    methods: BTreeMap<String, BTreeSet<String>>,
}

fn support_macro_metavariable_method_requirements_from_named_items(
    named_items: &BTreeMap<String, Item>,
) -> BTreeMap<String, MacroMetavariableMethodRequirement> {
    let items = named_items.values().cloned().collect::<Vec<_>>();
    support_macro_metavariable_method_requirements_from_items(&items)
}

fn support_macro_metavariable_method_requirements_from_items(
    items: &[Item],
) -> BTreeMap<String, MacroMetavariableMethodRequirement> {
    let mut requirements = BTreeMap::new();
    for item in items {
        let Item::Macro(item_macro) = item else {
            continue;
        };
        let Some(name) = item_macro.ident.as_ref().map(ToString::to_string) else {
            continue;
        };
        let methods = macro_metavariable_method_names(&item_macro.mac.tokens);
        if methods.is_empty() {
            continue;
        }
        requirements.insert(
            name,
            MacroMetavariableMethodRequirement {
                metavariables: macro_definition_metavariables(&item_macro.mac.tokens),
                methods,
            },
        );
    }
    requirements
}

fn macro_invocation_single_ident(path: &syn::Path) -> Option<String> {
    (path.segments.len() == 1).then(|| path.segments[0].ident.to_string())
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
    let token_trees = tokens.clone().into_iter().collect::<Vec<_>>();
    for token in &token_trees {
        if let TokenTree::Group(group) = token {
            collect_macro_metavariable_method_names(&group.stream(), names);
        }
    }

    for window in token_trees.windows(4) {
        let [TokenTree::Punct(dollar), TokenTree::Ident(receiver), TokenTree::Punct(dot), TokenTree::Ident(method)] =
            window
        else {
            continue;
        };
        if dollar.as_char() != '$'
            || dot.as_char() != '.'
            || !support_assoc_item_name_is_precise(&method.to_string())
        {
            continue;
        }
        names
            .entry(receiver.to_string())
            .or_default()
            .insert(method.to_string());
    }
}

struct SupportEnumVariantMethodVisitor<'a> {
    method_index: &'a BTreeMap<String, Vec<SupportMethodCandidate>>,
    variant_payload_index: &'a BTreeMap<(String, String), SupportEnumVariantPayloads>,
    current_impl_self: Option<String>,
    calls: BTreeSet<(SupportMethodCandidate, String)>,
}

impl<'ast> Visit<'ast> for SupportEnumVariantMethodVisitor<'_> {
    fn visit_item_impl(&mut self, item_impl: &'ast syn::ItemImpl) {
        let previous = self.current_impl_self.clone();
        if let Some(self_name) = support_impl_self_named_item(item_impl) {
            self.current_impl_self = Some(self_name);
        }
        visit::visit_item_impl(self, item_impl);
        self.current_impl_self = previous;
    }

    fn visit_expr_match(&mut self, node: &'ast syn::ExprMatch) {
        for arm in &node.arms {
            let bindings = support_pat_variant_payload_bindings(
                &arm.pat,
                self.current_impl_self.as_deref(),
                self.variant_payload_index,
            );
            if !bindings.is_empty() {
                let mut collector = SupportBoundMethodCallVisitor {
                    bindings: &bindings,
                    method_index: self.method_index,
                    calls: &mut self.calls,
                };
                collector.visit_expr(&arm.body);
                if let Some((_if, guard)) = &arm.guard {
                    collector.visit_expr(guard);
                }
            }
        }
        visit::visit_expr_match(self, node);
    }
}

struct SupportBoundMethodCallVisitor<'a> {
    bindings: &'a BTreeMap<String, SupportMethodCandidate>,
    method_index: &'a BTreeMap<String, Vec<SupportMethodCandidate>>,
    calls: &'a mut BTreeSet<(SupportMethodCandidate, String)>,
}

impl Visit<'_> for SupportBoundMethodCallVisitor<'_> {
    fn visit_expr_method_call(&mut self, node: &syn::ExprMethodCall) {
        if let Some(binding) = support_receiver_binding_name(&node.receiver) {
            if let Some(candidate) = self.bindings.get(&binding) {
                let method = node.method.to_string();
                if self
                    .method_index
                    .get(&method)
                    .is_some_and(|candidates| candidates.iter().any(|indexed| indexed == candidate))
                {
                    self.calls.insert((candidate.clone(), method));
                }
            }
        }
        visit::visit_expr_method_call(self, node);
    }

    fn visit_macro(&mut self, node: &syn::Macro) {
        collect_bound_method_calls_from_tokens(
            node.tokens.clone(),
            self.bindings,
            self.method_index,
            self.calls,
        );
        visit::visit_macro(self, node);
    }
}

fn collect_bound_method_calls_from_tokens(
    tokens: TokenStream,
    bindings: &BTreeMap<String, SupportMethodCandidate>,
    method_index: &BTreeMap<String, Vec<SupportMethodCandidate>>,
    calls: &mut BTreeSet<(SupportMethodCandidate, String)>,
) {
    let mut previous_ident = None::<String>;
    let mut receiver_binding = None::<String>;
    for token in tokens {
        match token {
            TokenTree::Ident(ident) => {
                let ident = ident.to_string();
                if let Some(binding) = receiver_binding.take() {
                    if let Some(candidate) = bindings.get(&binding) {
                        if method_index.get(&ident).is_some_and(|candidates| {
                            candidates.iter().any(|indexed| indexed == candidate)
                        }) {
                            calls.insert((candidate.clone(), ident.clone()));
                        }
                    }
                }
                previous_ident = Some(ident);
            }
            TokenTree::Punct(punct) if punct.as_char() == '.' => {
                receiver_binding = previous_ident
                    .take()
                    .filter(|ident| bindings.contains_key(ident));
            }
            TokenTree::Group(group) => {
                collect_bound_method_calls_from_tokens(
                    group.stream(),
                    bindings,
                    method_index,
                    calls,
                );
                previous_ident = None;
                receiver_binding = None;
            }
            _ => {
                previous_ident = None;
                receiver_binding = None;
            }
        }
    }
}

fn support_pat_variant_payload_bindings(
    pat: &Pat,
    current_impl_self: Option<&str>,
    variant_payload_index: &BTreeMap<(String, String), SupportEnumVariantPayloads>,
) -> BTreeMap<String, SupportMethodCandidate> {
    let mut bindings = BTreeMap::new();
    collect_support_pat_variant_payload_bindings(
        pat,
        current_impl_self,
        variant_payload_index,
        &mut bindings,
    );
    bindings
}

fn collect_support_pat_variant_payload_bindings(
    pat: &Pat,
    current_impl_self: Option<&str>,
    variant_payload_index: &BTreeMap<(String, String), SupportEnumVariantPayloads>,
    bindings: &mut BTreeMap<String, SupportMethodCandidate>,
) {
    match pat {
        Pat::TupleStruct(tuple_struct) => {
            if let Some(payloads) = support_variant_payloads_for_path(
                &tuple_struct.path,
                current_impl_self,
                variant_payload_index,
            ) {
                for (index, elem) in tuple_struct.elems.iter().enumerate() {
                    let Some(candidate) = payloads.unnamed.get(index).and_then(Option::as_ref)
                    else {
                        continue;
                    };
                    collect_support_payload_binding(elem, candidate, bindings);
                }
            }
        }
        Pat::Struct(pat_struct) => {
            if let Some(payloads) = support_variant_payloads_for_path(
                &pat_struct.path,
                current_impl_self,
                variant_payload_index,
            ) {
                for field in &pat_struct.fields {
                    let member = match &field.member {
                        syn::Member::Named(ident) => ident.to_string(),
                        syn::Member::Unnamed(index) => index.index.to_string(),
                    };
                    let Some(candidate) = payloads.named.get(&member) else {
                        continue;
                    };
                    collect_support_payload_binding(&field.pat, candidate, bindings);
                }
            }
        }
        Pat::Or(pat_or) => {
            for case in &pat_or.cases {
                collect_support_pat_variant_payload_bindings(
                    case,
                    current_impl_self,
                    variant_payload_index,
                    bindings,
                );
            }
        }
        Pat::Reference(reference) => collect_support_pat_variant_payload_bindings(
            &reference.pat,
            current_impl_self,
            variant_payload_index,
            bindings,
        ),
        Pat::Paren(paren) => collect_support_pat_variant_payload_bindings(
            &paren.pat,
            current_impl_self,
            variant_payload_index,
            bindings,
        ),
        _ => {}
    }
}

fn collect_support_payload_binding(
    pat: &Pat,
    candidate: &SupportMethodCandidate,
    bindings: &mut BTreeMap<String, SupportMethodCandidate>,
) {
    match pat {
        Pat::Ident(ident) => {
            if ident.by_ref.is_none() && ident.subpat.is_none() {
                bindings.insert(ident.ident.to_string(), candidate.clone());
            }
        }
        Pat::Reference(reference) => {
            collect_support_payload_binding(&reference.pat, candidate, bindings);
        }
        Pat::Paren(paren) => collect_support_payload_binding(&paren.pat, candidate, bindings),
        _ => {}
    }
}

fn support_variant_payloads_for_path<'a>(
    path: &syn::Path,
    current_impl_self: Option<&str>,
    variant_payload_index: &'a BTreeMap<(String, String), SupportEnumVariantPayloads>,
) -> Option<&'a SupportEnumVariantPayloads> {
    let segments = path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>();
    let variant_name = segments.last()?;
    let enum_name = segments.iter().rev().nth(1).and_then(|name| {
        if name == "Self" {
            current_impl_self.map(str::to_string)
        } else {
            Some(name.clone())
        }
    })?;
    variant_payload_index.get(&(enum_name, variant_name.clone()))
}

fn support_receiver_binding_name(receiver: &Expr) -> Option<String> {
    match receiver {
        Expr::Path(path) if path.qself.is_none() && path.path.segments.len() == 1 => {
            Some(path.path.segments[0].ident.to_string())
        }
        Expr::Reference(reference) => support_receiver_binding_name(&reference.expr),
        Expr::Paren(paren) => support_receiver_binding_name(&paren.expr),
        _ => None,
    }
}

#[derive(Default)]
struct SupportAssocUsageVisitor {
    self_type: Option<String>,
    variable_type_scopes: Vec<BTreeMap<String, Vec<String>>>,
    assoc_paths: BTreeSet<Vec<String>>,
    macro_method_requirements: BTreeMap<String, MacroMetavariableMethodRequirement>,
}

impl SupportAssocUsageVisitor {
    fn new(
        self_type: Option<String>,
        macro_method_requirements: BTreeMap<String, MacroMetavariableMethodRequirement>,
    ) -> Self {
        Self {
            self_type,
            variable_type_scopes: Vec::new(),
            assoc_paths: BTreeSet::new(),
            macro_method_requirements,
        }
    }

    fn push_fn_arg_scope(&mut self, inputs: &Punctuated<syn::FnArg, syn::Token![,]>) {
        let mut scope = BTreeMap::new();
        for input in inputs {
            let syn::FnArg::Typed(input) = input else {
                continue;
            };
            let syn::Pat::Ident(pat_ident) = input.pat.as_ref() else {
                continue;
            };
            if let Some(type_path) = support_type_assoc_path(&input.ty) {
                scope.insert(pat_ident.ident.to_string(), type_path);
            }
        }
        self.variable_type_scopes.push(scope);
    }

    fn pop_fn_arg_scope(&mut self) {
        self.variable_type_scopes.pop();
    }

    fn record_local_type(&mut self, name: String, type_path: Vec<String>) {
        if let Some(scope) = self.variable_type_scopes.last_mut() {
            scope.insert(name, type_path);
        }
    }

    fn variable_type(&self, name: &str) -> Option<Vec<String>> {
        self.variable_type_scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).cloned())
    }

    fn macro_arg_type_path(&self, name: &str) -> Option<Vec<String>> {
        if name == "self" {
            return self
                .self_type
                .as_ref()
                .map(|self_type| vec![self_type.clone()]);
        }
        self.variable_type(name)
    }

    fn record_macro_metavariable_method_dependencies(&mut self, mac: &syn::Macro) {
        let Some(macro_name) = macro_invocation_single_ident(&mac.path) else {
            return;
        };
        let Some(requirement) = self.macro_method_requirements.get(&macro_name) else {
            return;
        };
        let invocation_args = macro_invocation_arg_idents(&mac.tokens);
        let mut assoc_paths = Vec::new();
        for (metavariable, arg) in requirement.metavariables.iter().zip(invocation_args) {
            let Some(methods) = requirement.methods.get(metavariable) else {
                continue;
            };
            let Some(type_path) = self.macro_arg_type_path(&arg) else {
                continue;
            };
            for method in methods {
                let mut assoc_path = type_path.clone();
                assoc_path.push(method.clone());
                if support_required_assoc_segments(&assoc_path).is_some() {
                    assoc_paths.push(assoc_path);
                }
            }
        }
        self.assoc_paths.extend(assoc_paths);
    }
}

impl Visit<'_> for SupportAssocUsageVisitor {
    fn visit_item_impl(&mut self, node: &syn::ItemImpl) {
        let previous = self.self_type.clone();
        if let Some(self_type) = support_impl_self_named_item(node) {
            self.self_type = Some(self_type);
        }
        visit::visit_item_impl(self, node);
        self.self_type = previous;
    }

    fn visit_item_fn(&mut self, node: &syn::ItemFn) {
        self.push_fn_arg_scope(&node.sig.inputs);
        visit::visit_block(self, &node.block);
        self.pop_fn_arg_scope();
    }

    fn visit_impl_item_fn(&mut self, node: &syn::ImplItemFn) {
        self.push_fn_arg_scope(&node.sig.inputs);
        visit::visit_block(self, &node.block);
        self.pop_fn_arg_scope();
    }

    fn visit_local(&mut self, node: &syn::Local) {
        if let syn::Pat::Type(pat_type) = &node.pat {
            if let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref() {
                if let Some(type_path) = support_type_assoc_path(&pat_type.ty) {
                    self.record_local_type(pat_ident.ident.to_string(), type_path);
                }
            }
        } else if let syn::Pat::Ident(pat_ident) = &node.pat {
            if let Some(init) = &node.init {
                if let Some(type_path) = support_assoc_receiver_type_path(&init.expr, self) {
                    self.record_local_type(pat_ident.ident.to_string(), type_path);
                }
            }
        }
        visit::visit_local(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &syn::ExprMethodCall) {
        let method = node.method.to_string();
        if support_assoc_item_name_is_precise(&method) {
            if let Some(mut type_path) = support_assoc_receiver_type_path(&node.receiver, self) {
                type_path.push(method);
                if support_required_assoc_segments(&type_path).is_some() {
                    self.assoc_paths.insert(type_path);
                }
            }
        }
        visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_call(&mut self, node: &syn::ExprCall) {
        if let syn::Expr::Path(path) = node.func.as_ref() {
            if let Some(assoc_path) =
                support_assoc_call_path_from_path(&path.path, self.self_type.as_deref())
            {
                if support_required_assoc_segments(&assoc_path).is_some() {
                    self.assoc_paths.insert(assoc_path);
                }
            }
        }
        visit::visit_expr_call(self, node);
    }

    fn visit_macro(&mut self, node: &syn::Macro) {
        self.record_macro_metavariable_method_dependencies(node);
        visit::visit_macro(self, node);
    }
}

fn support_assoc_receiver_type_path(
    expr: &Expr,
    visitor: &SupportAssocUsageVisitor,
) -> Option<Vec<String>> {
    match expr {
        Expr::Call(call) => match call.func.as_ref() {
            Expr::Path(path) => support_type_path_from_assoc_path(&path.path),
            _ => None,
        },
        Expr::MethodCall(method_call) => {
            support_assoc_receiver_type_path(&method_call.receiver, visitor)
        }
        Expr::Struct(expr_struct) => support_type_path_from_path(&expr_struct.path),
        Expr::Path(expr_path) => {
            if expr_path.path.is_ident("self") {
                return visitor
                    .self_type
                    .as_ref()
                    .map(|self_type| vec![self_type.clone()]);
            }
            let segments = expr_path
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<_>>();
            if segments.len() == 1 {
                let ident = segments.first()?;
                if let Some(type_path) = visitor.variable_type(ident) {
                    return Some(type_path);
                }
            }
            support_type_path_from_segments(&segments)
        }
        Expr::Reference(reference) => support_assoc_receiver_type_path(&reference.expr, visitor),
        Expr::Paren(paren) => support_assoc_receiver_type_path(&paren.expr, visitor),
        Expr::Group(group) => support_assoc_receiver_type_path(&group.expr, visitor),
        Expr::Try(expr_try) => support_assoc_receiver_type_path(&expr_try.expr, visitor),
        Expr::Await(expr_await) => support_assoc_receiver_type_path(&expr_await.base, visitor),
        _ => None,
    }
}

fn support_type_assoc_path(ty: &Type) -> Option<Vec<String>> {
    match ty {
        Type::Path(type_path) => support_type_path_from_path(&type_path.path),
        Type::Reference(reference) => support_type_assoc_path(&reference.elem),
        Type::Paren(paren) => support_type_assoc_path(&paren.elem),
        Type::Group(group) => support_type_assoc_path(&group.elem),
        _ => None,
    }
}

fn support_type_path_from_assoc_path(path: &syn::Path) -> Option<Vec<String>> {
    let segments = path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>();
    let (assoc_name, type_path) = segments.split_last()?;
    if !support_assoc_item_name_is_precise(assoc_name) {
        return None;
    }
    support_type_path_from_segments(type_path)
}

fn support_assoc_call_path_from_path(
    path: &syn::Path,
    self_type: Option<&str>,
) -> Option<Vec<String>> {
    let segments = path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>();
    let (assoc_name, type_segments) = segments.split_last()?;
    if !support_assoc_item_name_is_precise(assoc_name) {
        return None;
    }
    let mut type_path = support_type_path_from_segments(type_segments)?;
    if type_path.as_slice() == ["Self"] {
        type_path = vec![self_type?.to_string()];
    }
    type_path.push(assoc_name.clone());
    Some(type_path)
}

fn support_type_path_from_path(path: &syn::Path) -> Option<Vec<String>> {
    let segments = path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>();
    support_type_path_from_segments(&segments)
}

fn support_type_path_from_segments(segments: &[String]) -> Option<Vec<String>> {
    if segments.is_empty() {
        return None;
    }
    let type_index = segments
        .iter()
        .rposition(|segment| support_type_like_ident(segment))?;
    Some(segments[..=type_index].to_vec())
}

fn collect_derive_attr_idents(attrs: &[syn::Attribute], derives: &mut BTreeSet<String>) {
    for attr in attrs {
        if !attr.path().is_ident("derive") {
            continue;
        }
        let Ok(paths) =
            attr.parse_args_with(Punctuated::<syn::Path, syn::Token![,]>::parse_terminated)
        else {
            continue;
        };
        for path in paths {
            if let Some(ident) = path
                .segments
                .last()
                .map(|segment| segment.ident.to_string())
            {
                derives.insert(ident);
            }
        }
    }
}

fn prune_support_public_use_tree(
    ctx: &SupportResolveContext<'_>,
    live_by_file: &BTreeMap<PathBuf, SupportLiveSet>,
    source_file: &Path,
    tree: &UseTree,
    mut prefix: Vec<String>,
    live_names: &BTreeSet<String>,
) -> Option<UseTree> {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            let mut path = path.clone();
            path.tree = Box::new(prune_support_public_use_tree(
                ctx,
                live_by_file,
                source_file,
                &path.tree,
                prefix,
                live_names,
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
            live_names
                .contains(&visible_name)
                .then(|| UseTree::Name(name.clone()))
        }
        UseTree::Rename(rename) => (live_names.contains(&rename.ident.to_string())
            || live_names.contains(&rename.rename.to_string()))
        .then(|| UseTree::Rename(rename.clone())),
        UseTree::Group(group) => {
            let mut group = group.clone();
            group.items = group
                .items
                .iter()
                .filter_map(|item| {
                    prune_support_public_use_tree(
                        ctx,
                        live_by_file,
                        source_file,
                        item,
                        prefix.clone(),
                        live_names,
                    )
                })
                .collect::<Punctuated<_, syn::Token![,]>>();
            (!group.items.is_empty()).then_some(UseTree::Group(group))
        }
        UseTree::Glob(glob) => match support_local_glob_prefix_target(ctx, source_file, &prefix) {
            SupportLocalGlobTarget::Module(target_file) => {
                let target_live_set = live_by_file.get(&target_file);
                support_live_set_exposes_names(target_live_set, live_names)
                    .then(|| UseTree::Glob(glob.clone()))
            }
            SupportLocalGlobTarget::Enum {
                source_file,
                enum_name,
                variants,
            } => {
                let enum_is_live = live_by_file
                    .get(&source_file)
                    .is_some_and(|live_set| live_set.item_names.contains(&enum_name));
                (enum_is_live && variants.iter().any(|variant| live_names.contains(variant)))
                    .then(|| UseTree::Glob(glob.clone()))
            }
            SupportLocalGlobTarget::External | SupportLocalGlobTarget::Unsupported => None,
        },
    }
}

fn prune_support_private_use_tree(
    ctx: &SupportResolveContext<'_>,
    live_by_file: &BTreeMap<PathBuf, SupportLiveSet>,
    source_file: &Path,
    tree: &UseTree,
    mut prefix: Vec<String>,
    live_names: &BTreeSet<String>,
    live_derive_idents: &BTreeSet<String>,
) -> Option<UseTree> {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            let mut path = path.clone();
            path.tree = Box::new(prune_support_private_use_tree(
                ctx,
                live_by_file,
                source_file,
                &path.tree,
                prefix,
                live_names,
                live_derive_idents,
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
            support_private_import_name_should_render(
                &prefix,
                &name.ident.to_string(),
                &visible_name,
                live_names,
                live_derive_idents,
            )
            .then(|| UseTree::Name(name.clone()))
        }
        UseTree::Rename(rename) => support_private_import_name_should_render(
            &prefix,
            &rename.ident.to_string(),
            &rename.rename.to_string(),
            live_names,
            live_derive_idents,
        )
        .then(|| UseTree::Rename(rename.clone())),
        UseTree::Group(group) => {
            let mut group = group.clone();
            group.items = group
                .items
                .iter()
                .filter_map(|item| {
                    prune_support_private_use_tree(
                        ctx,
                        live_by_file,
                        source_file,
                        item,
                        prefix.clone(),
                        live_names,
                        live_derive_idents,
                    )
                })
                .collect::<Punctuated<_, syn::Token![,]>>();
            (!group.items.is_empty()).then_some(UseTree::Group(group))
        }
        UseTree::Glob(glob) => match support_local_glob_prefix_target(ctx, source_file, &prefix) {
            SupportLocalGlobTarget::Module(target_file) => {
                support_live_set_exposes_names(live_by_file.get(&target_file), live_names)
                    .then(|| UseTree::Glob(glob.clone()))
            }
            SupportLocalGlobTarget::Enum {
                source_file,
                enum_name,
                variants,
            } => {
                let enum_is_live = live_by_file
                    .get(&source_file)
                    .is_some_and(|live_set| live_set.item_names.contains(&enum_name));
                (enum_is_live && variants.iter().any(|variant| live_names.contains(variant)))
                    .then(|| UseTree::Glob(glob.clone()))
            }
            SupportLocalGlobTarget::External | SupportLocalGlobTarget::Unsupported => None,
        },
    }
}

fn support_private_import_name_should_render(
    prefix: &[String],
    imported_name: &str,
    visible_name: &str,
    live_names: &BTreeSet<String>,
    live_derive_idents: &BTreeSet<String>,
) -> bool {
    if support_import_is_derive_only(prefix, imported_name) {
        return live_derive_idents.contains(imported_name)
            || live_derive_idents.contains(visible_name);
    }
    live_names.contains(visible_name)
}

fn support_import_is_derive_only(prefix: &[String], imported_name: &str) -> bool {
    imported_name == "Error"
        && prefix
            .first()
            .is_some_and(|first| dependency_code_name(first) == "thiserror")
}

fn support_live_set_exposes_names(
    live_set: Option<&SupportLiveSet>,
    names: &BTreeSet<String>,
) -> bool {
    let Some(live_set) = live_set else {
        return false;
    };
    live_set.item_names.iter().any(|name| names.contains(name))
        || live_set
            .public_exports
            .iter()
            .any(|name| names.contains(name))
}

fn support_impl_should_render(
    item_impl: &syn::ItemImpl,
    named_items: &BTreeMap<String, Item>,
    live_set: &SupportLiveSet,
) -> bool {
    support_impl_header_mentions_live_named_item(item_impl, named_items, live_set)
}

fn support_impl_header_mentions_live_named_item(
    item_impl: &syn::ItemImpl,
    named_items: &BTreeMap<String, Item>,
    live_set: &SupportLiveSet,
) -> bool {
    let mut tokens = TokenStream::new();
    item_impl.generics.to_tokens(&mut tokens);
    item_impl.self_ty.to_tokens(&mut tokens);

    if let Some((_, trait_path, _)) = &item_impl.trait_ {
        trait_path.to_tokens(&mut tokens);
    }

    let mentioned_local_names = named_items
        .keys()
        .filter(|name| token_stream_mentions_ident(&tokens, name))
        .collect::<Vec<_>>();
    !mentioned_local_names.is_empty()
        && mentioned_local_names.iter().all(|name| {
            live_set.item_names.contains(*name) || live_set.public_exports.contains(*name)
        })
}

fn support_named_item_names(items: &[Item]) -> BTreeMap<String, Item> {
    items
        .iter()
        .filter_map(|item| support_item_name(item).map(|name| (name, item.clone())))
        .collect()
}

fn support_item_name(item: &Item) -> Option<String> {
    match item {
        Item::Const(item) => Some(item.ident.to_string()),
        Item::Enum(item) => Some(item.ident.to_string()),
        Item::Fn(item) => Some(item.sig.ident.to_string()),
        Item::Macro(item) => item.ident.as_ref().map(ToString::to_string),
        Item::Mod(item) => Some(item.ident.to_string()),
        Item::Static(item) => Some(item.ident.to_string()),
        Item::Struct(item) => Some(item.ident.to_string()),
        Item::Trait(item) => Some(item.ident.to_string()),
        Item::Type(item) => Some(item.ident.to_string()),
        Item::Union(item) => Some(item.ident.to_string()),
        _ => None,
    }
}

fn support_library_module_source_files(
    package: &SupportPackage,
) -> Result<Option<BTreeSet<PathBuf>>, Box<dyn std::error::Error>> {
    support_library_module_source_files_for(&package.root, &package.manifest)
}

fn support_library_module_source_files_for(
    package_root: &Path,
    manifest: &Value,
) -> Result<Option<BTreeSet<PathBuf>>, Box<dyn std::error::Error>> {
    let Some(lib_path) = support_library_source_path_from(package_root, manifest) else {
        return Ok(None);
    };
    let Ok(package_root) = package_root.canonicalize() else {
        return Ok(None);
    };
    let Ok(lib_path) = lib_path.canonicalize() else {
        return Ok(None);
    };
    if !lib_path.starts_with(&package_root) {
        return Ok(None);
    }

    let mut sources = BTreeSet::new();
    let module_dir = lib_path.parent().unwrap_or(&package_root).to_path_buf();
    if !collect_support_library_module_sources(&package_root, &lib_path, &module_dir, &mut sources)?
    {
        return Ok(None);
    }
    Ok(Some(sources))
}

fn write_support_package_source_file(
    package_root: &Path,
    path: &Path,
    syntax: &syn::File,
    package_output: &Path,
) -> Result<usize, Box<dyn std::error::Error>> {
    let relative = path.strip_prefix(package_root)?;
    let output_path = package_output.join(relative);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(output_path, prettyplease::unparse(syntax))?;
    Ok(1)
}

fn collect_support_library_module_sources(
    package_root: &Path,
    source_file: &Path,
    module_dir: &Path,
    sources: &mut BTreeSet<PathBuf>,
) -> Result<bool, Box<dyn std::error::Error>> {
    let Ok(source_file) = source_file.canonicalize() else {
        return Ok(false);
    };
    if !source_file.starts_with(package_root) {
        return Ok(false);
    }
    if !sources.insert(source_file.clone()) {
        return Ok(true);
    }

    let text = match fs::read_to_string(&source_file) {
        Ok(text) => text,
        Err(_) => return Ok(false),
    };
    let syntax = match syn::parse_file(&text) {
        Ok(syntax) => syntax,
        Err(_) => return Ok(false),
    };

    for item in syntax.items {
        let Item::Mod(item_mod) = item else {
            continue;
        };
        if item_mod.content.is_some() || attrs_are_test(&item_mod.attrs) {
            continue;
        }
        let Some((child_file, child_dir)) =
            support_external_module_source(package_root, module_dir, &item_mod)
        else {
            return Ok(false);
        };
        if !collect_support_library_module_sources(package_root, &child_file, &child_dir, sources)?
        {
            return Ok(false);
        }
    }

    Ok(true)
}

fn support_external_module_source(
    package_root: &Path,
    module_dir: &Path,
    item_mod: &ItemMod,
) -> Option<(PathBuf, PathBuf)> {
    let name = item_mod.ident.to_string();
    let source_name = module_source_name(&name);
    let path_attr = support_path_attr(item_mod);
    let file_path = module_dir.join(format!("{source_name}.rs"));
    let mod_path = module_dir.join(source_name).join("mod.rs");
    let (source_file, child_module_dir) = if let Some(path_attr) = path_attr {
        let source_file = if path_attr.is_absolute() {
            path_attr
        } else {
            module_dir.join(path_attr)
        };
        let child_module_dir = source_file.parent().unwrap_or(module_dir).to_path_buf();
        (source_file, child_module_dir)
    } else if file_path.exists() {
        (file_path, module_dir.join(source_name))
    } else if mod_path.exists() {
        (mod_path, module_dir.join(source_name))
    } else {
        return None;
    };
    let Ok(resolved) = source_file.canonicalize() else {
        return None;
    };
    if !resolved.starts_with(package_root) {
        return None;
    }
    Some((resolved, child_module_dir))
}

fn support_path_attr(item_mod: &ItemMod) -> Option<PathBuf> {
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

fn module_source_name(name: &str) -> &str {
    name.strip_prefix("r#").unwrap_or(name)
}

fn copy_support_package_file(
    package_root: &Path,
    path: &Path,
    package_output: &Path,
) -> Result<usize, Box<dyn std::error::Error>> {
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(error) => return Err(error.into()),
    };
    if !metadata.is_file() {
        return Ok(0);
    }
    let relative = path.strip_prefix(package_root)?;
    let output_path = package_output.join(relative);
    if output_path.exists() {
        return Ok(0);
    }
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(path, output_path)?;
    Ok(1)
}

fn copy_support_package_tree_inner(
    package_root: &Path,
    path: &Path,
    package_output: &Path,
    skip_auto_targets: bool,
    visited: &mut BTreeSet<PathBuf>,
) -> Result<usize, Box<dyn std::error::Error>> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Ok(0);
    }

    let resolved = path.canonicalize()?;
    if !resolved.starts_with(package_root) {
        return Ok(0);
    }

    if metadata.is_dir() {
        if skip_auto_targets && is_support_auto_target_dir(package_root, path) {
            return Ok(0);
        }
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| matches!(name, ".git" | ".hg" | ".svn" | "target"))
        {
            return Ok(0);
        }
        if !visited.insert(resolved) {
            return Ok(0);
        }

        let mut entries = fs::read_dir(path)?.collect::<Result<Vec<_>, _>>()?;
        entries.sort_by_key(|entry| entry.file_name());
        let mut copied = 0;
        for entry in entries {
            copied += copy_support_package_tree_inner(
                package_root,
                &entry.path(),
                package_output,
                skip_auto_targets,
                visited,
            )?;
        }
        return Ok(copied);
    }

    if !metadata.is_file() {
        return Ok(0);
    }
    let relative = path.strip_prefix(package_root)?;
    let output_path = package_output.join(relative);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(path, output_path)?;
    Ok(1)
}

fn is_support_auto_target_dir(package_root: &Path, path: &Path) -> bool {
    let Ok(relative) = path.strip_prefix(package_root) else {
        return false;
    };
    let components = relative
        .components()
        .filter_map(|component| match component {
            Component::Normal(component) => component.to_str(),
            Component::CurDir
            | Component::ParentDir
            | Component::RootDir
            | Component::Prefix(_) => None,
        })
        .collect::<Vec<_>>();
    matches!(
        components.as_slice(),
        ["examples" | "tests" | "benches", ..] | ["src", "bin", ..]
    )
}

fn copy_support_include_assets(
    package: &SupportPackage,
    package_output: &Path,
    output_root: &Path,
) -> Result<usize, Box<dyn std::error::Error>> {
    let package_root = package.root.canonicalize()?;
    let allowed_source_root = package
        .workspace
        .as_ref()
        .map(|workspace| workspace.root.as_path())
        .unwrap_or(package_root.as_path())
        .canonicalize()?;
    let mut rust_files = Vec::new();
    if let Some(transformed_sources) = &package.source_plan.transformed_sources {
        let mut copied = 0;
        for (source, syntax) in transformed_sources {
            copied += copy_include_assets_for_support_source(
                &package_root,
                &allowed_source_root,
                source,
                syntax,
                package_output,
                output_root,
            )?;
        }
        return Ok(copied);
    } else if support_build_script_path(package).is_some() {
        let mut visited = BTreeSet::new();
        collect_support_rust_files(&package_root, &package_root, &mut visited, &mut rust_files)?;
    } else if let Some(source_files) = support_library_module_source_files(package)? {
        rust_files.extend(source_files);
    } else {
        let mut visited = BTreeSet::new();
        collect_support_rust_files(&package_root, &package_root, &mut visited, &mut rust_files)?;
    }

    let mut copied = 0;
    for source in rust_files {
        let Ok(text) = fs::read_to_string(&source) else {
            continue;
        };
        let Ok(syntax) = syn::parse_file(&text) else {
            continue;
        };
        copied += copy_include_assets_for_support_source(
            &package_root,
            &allowed_source_root,
            &source,
            &syntax,
            package_output,
            output_root,
        )?;
    }
    Ok(copied)
}

fn collect_support_rust_files(
    root: &Path,
    path: &Path,
    visited: &mut BTreeSet<PathBuf>,
    files: &mut Vec<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Ok(());
    }

    let resolved = path.canonicalize()?;
    if !resolved.starts_with(root) {
        return Ok(());
    }

    if metadata.is_dir() {
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| matches!(name, ".git" | ".hg" | ".svn" | "target"))
        {
            return Ok(());
        }
        if !visited.insert(resolved) {
            return Ok(());
        }
        let mut entries = fs::read_dir(path)?.collect::<Result<Vec<_>, _>>()?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            collect_support_rust_files(root, &entry.path(), visited, files)?;
        }
        return Ok(());
    }

    if metadata.is_file() && path.extension().is_some_and(|extension| extension == "rs") {
        files.push(path.to_path_buf());
    }
    Ok(())
}

fn copy_include_assets_for_support_source(
    package_root: &Path,
    allowed_source_root: &Path,
    source: &Path,
    syntax: &syn::File,
    package_output: &Path,
    output_root: &Path,
) -> Result<usize, Box<dyn std::error::Error>> {
    let Some(source_dir) = source.parent() else {
        return Ok(0);
    };

    let mut candidates = BTreeSet::new();
    collect_include_macro_paths(&syntax.to_token_stream(), &mut candidates);

    let mut copied = 0;
    for candidate in candidates {
        let path = match candidate {
            StaticIncludePath::SourceRelative(path) => source_dir.join(path),
            StaticIncludePath::PackageRelative(path) => package_root.join(path),
            StaticIncludePath::Absolute(_) => continue,
        };
        copied += copy_support_include_asset(
            package_root,
            allowed_source_root,
            &path,
            package_output,
            output_root,
        )?;
    }
    Ok(copied)
}

fn copy_support_include_asset(
    package_root: &Path,
    allowed_source_root: &Path,
    source_path: &Path,
    package_output: &Path,
    output_root: &Path,
) -> Result<usize, Box<dyn std::error::Error>> {
    let metadata = match fs::symlink_metadata(source_path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(error) => return Err(error.into()),
    };
    if metadata.file_type().is_symlink() {
        return Ok(0);
    }
    let Ok(resolved) = source_path.canonicalize() else {
        return Ok(0);
    };
    if !resolved.starts_with(allowed_source_root) || !fs::metadata(&resolved)?.is_file() {
        return Ok(0);
    }

    let relative = relative_path_between(package_root, &resolved);
    let output_path = normalize_path(&package_output.join(relative));
    if !output_path.starts_with(output_root) || output_path.exists() {
        return Ok(0);
    }
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(resolved, output_path)?;
    Ok(1)
}

fn relative_path_between(from_dir: &Path, target: &Path) -> PathBuf {
    let from = normal_components(from_dir);
    let target = normal_components(target);
    let mut common = 0;
    while common < from.len() && common < target.len() && from[common] == target[common] {
        common += 1;
    }

    let mut relative = PathBuf::new();
    for _ in common..from.len() {
        relative.push("..");
    }
    for component in &target[common..] {
        relative.push(component);
    }
    if relative.as_os_str().is_empty() {
        relative.push(".");
    }
    relative
}

fn normal_components(path: &Path) -> Vec<String> {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(component) => Some(component.to_string_lossy().to_string()),
            Component::CurDir => None,
            Component::ParentDir => Some("..".to_string()),
            Component::RootDir | Component::Prefix(_) => None,
        })
        .collect()
}

fn merge_transformed_patch_tables(
    patches: &mut Table,
    source_patches: &Table,
    manifest_dir: &Path,
    support_packages: &SupportPackagePlan,
    retained_patch_names: &BTreeSet<String>,
) {
    for (source, value) in source_patches {
        let Some(source_table) = value.as_table() else {
            patches
                .entry(source.clone())
                .or_insert_with(|| value.clone());
            continue;
        };
        let target = patches
            .entry(source.clone())
            .or_insert_with(|| Value::Table(Table::new()));
        let Some(target_table) = target.as_table_mut() else {
            continue;
        };
        for (name, dependency) in source_table {
            if !retained_patch_names.contains(name) {
                continue;
            }
            target_table.entry(name.clone()).or_insert_with(|| {
                support_packages.transformed_workspace_dependency_value(dependency, manifest_dir)
            });
        }
    }
}

fn merge_transformed_replace_table(
    replacements: &mut Table,
    source_replacements: &Table,
    manifest_dir: &Path,
    support_packages: &SupportPackagePlan,
) {
    for (name, dependency) in source_replacements {
        replacements.entry(name.clone()).or_insert_with(|| {
            support_packages.transformed_workspace_dependency_value(dependency, manifest_dir)
        });
    }
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
        Item::Enum(item_enum) => {
            collect_enum_surface_idents(project, reduced, item, item_enum, &mut idents);
        }
        Item::Trait(item_trait) => {
            collect_trait_surface_idents(project, reduced, item, item_trait, &mut idents);
        }
        _ => collect_token_idents(&record.item.to_token_stream(), &mut idents),
    }
    expand_alias_surface_idents(&record.aliases, &mut idents);
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
    collect_token_idents(&item_struct.generics.to_token_stream(), idents);
    if root_item_should_render(reduced, item_id) {
        collect_token_idents(&item_struct.to_token_stream(), idents);
        return;
    }
    let syn::Fields::Named(fields) = &item_struct.fields else {
        collect_token_idents(&item_struct.to_token_stream(), idents);
        return;
    };
    for field in &fields.named {
        if struct_field_should_remain(
            project,
            reduced,
            None,
            &item_id.package,
            &item_id.module_path,
            item_struct,
            field,
        ) {
            collect_token_idents(&field.to_token_stream(), idents);
        }
    }
}

fn collect_enum_surface_idents(
    project: &Project,
    reduced: &ReducedProject,
    item_id: &ItemId,
    item_enum: &syn::ItemEnum,
    idents: &mut BTreeSet<String>,
) {
    if enum_preserves_full_variant_surface(reduced, item_id, item_enum) {
        collect_token_idents(&item_enum.to_token_stream(), idents);
        return;
    }

    idents.insert(item_id.name.clone());
    idents.extend(item_id.module_path.iter().cloned());
    for attr in &item_enum.attrs {
        collect_token_idents(&attr.to_token_stream(), idents);
    }
    collect_token_idents(&item_enum.generics.to_token_stream(), idents);
    for variant in &item_enum.variants {
        if enum_variant_should_remain(
            project,
            reduced,
            &item_id.package,
            &variant.ident.to_string(),
        ) {
            collect_token_idents(&variant.to_token_stream(), idents);
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
    if root_item_should_render(reduced, item_id) {
        collect_token_idents(&item_trait.to_token_stream(), idents);
        return;
    }
    idents.insert(item_id.name.clone());
    idents.extend(item_id.module_path.iter().cloned());
    collect_token_idents(&item_trait.generics.to_token_stream(), idents);
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
    for item in &item_trait.items {
        if trait_item_should_remain_for_type_surface(
            project,
            reduced,
            &item_id.package,
            &item_id.module_path,
            &item_id.name,
            item,
        ) {
            collect_token_idents(&item.to_token_stream(), idents);
        }
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
            expand_alias_surface_idents(&record.aliases, &mut idents);
        }
        if let Some(record) = project.methods.get(callable) {
            collect_token_idents(&record.item.to_token_stream(), &mut idents);
            expand_alias_surface_idents(&record.aliases, &mut idents);
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
            TokenTree::Literal(literal) => collect_literal_format_captures(&literal, idents),
            TokenTree::Punct(_) => {}
        }
    }
}

fn expand_alias_surface_idents(
    aliases: &HashMap<String, Vec<String>>,
    idents: &mut BTreeSet<String>,
) {
    let mut pending = idents.iter().cloned().collect::<Vec<_>>();
    let mut visited = BTreeSet::new();
    while let Some(ident) = pending.pop() {
        if !visited.insert(ident.clone()) {
            continue;
        }
        let Some(target) = aliases.get(&ident) else {
            continue;
        };
        let Some(target_ident) = target
            .iter()
            .rev()
            .find(|segment| !matches!(segment.as_str(), "crate" | "self" | "super"))
            .cloned()
        else {
            continue;
        };
        if idents.insert(target_ident.clone()) {
            pending.push(target_ident);
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

fn retained_surface_idents_by_package(
    project: &Project,
    reduced: &ReducedProject,
    reachable_token_idents: &ReachableTokenIdentIndex,
) -> BTreeMap<String, BTreeSet<String>> {
    let mut packages = BTreeMap::<String, BTreeSet<String>>::new();
    for package in &reduced.packages {
        for module_path in project_module_paths(project, package) {
            let Some(items) = module_items_for_path(project, package, &module_path) else {
                continue;
            };
            let mut idents = BTreeSet::new();
            collect_retained_module_surface_idents(
                project,
                reduced,
                package,
                &module_path,
                items,
                reachable_token_idents,
                &mut idents,
            );
            packages.entry(package.clone()).or_default().extend(idents);
        }
    }
    packages
}

fn referenced_public_reexport_target_items(
    project: &Project,
    reduced: &ReducedProject,
    _mentions: &ReachableTokenIdentIndex,
) -> BTreeSet<ItemId> {
    let mut target_items = BTreeSet::new();
    for package in &reduced.packages {
        for module_path in project_module_paths(project, package) {
            let Some(items) = module_items_for_path(project, package, &module_path) else {
                continue;
            };
            for item in items {
                let Item::Use(item_use) = item else {
                    continue;
                };
                if !use_is_reexport(&item_use.vis) {
                    continue;
                }
                let mut visible_targets = Vec::new();
                collect_use_tree_visible_targets(&item_use.tree, Vec::new(), &mut visible_targets);
                for (visible_name, target) in visible_targets {
                    let Some((target_package, target_path)) =
                        resolve_use_target_path(project, package, &module_path, &target)
                    else {
                        continue;
                    };
                    if !reduced.packages.contains(&target_package) {
                        continue;
                    }
                    if let Some(item) = find_use_item(project, &target_package, &target_path) {
                        if public_reexport_target_item_is_referenced(
                            project,
                            reduced,
                            package,
                            &module_path,
                            &visible_name,
                            &item,
                        ) {
                            target_items.insert(item);
                        }
                        continue;
                    }
                    if let Some((alias_package, alias_path)) =
                        resolve_reexported_use_path(project, &target_package, &target_path)
                    {
                        if reduced.packages.contains(&alias_package) {
                            if let Some(item) = find_use_item(project, &alias_package, &alias_path)
                            {
                                if public_reexport_target_item_is_referenced(
                                    project,
                                    reduced,
                                    package,
                                    &module_path,
                                    &visible_name,
                                    &item,
                                ) {
                                    target_items.insert(item);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    target_items
}

fn public_reexport_target_item_is_referenced(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
    visible_name: &str,
    target_item: &ItemId,
) -> bool {
    root_item_should_render(reduced, target_item)
        || public_reexport_name_is_referenced_by_reduced_package(
            project,
            reduced,
            package,
            visible_name,
        )
        || reduced_package_mentions_public_reexport_target(
            project,
            reduced,
            package,
            module_path,
            visible_name,
            target_item,
        )
}

fn reduced_package_mentions_public_reexport_target(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
    visible_name: &str,
    target_item: &ItemId,
) -> bool {
    reduced
        .reachable
        .iter()
        .filter(|callable| callable.package() == package)
        .any(|callable| {
            project.functions.get(callable).is_some_and(|record| {
                token_stream_mentions_public_reexport_target(
                    &record.item.to_token_stream(),
                    module_path,
                    visible_name,
                )
            }) || project.methods.get(callable).is_some_and(|record| {
                token_stream_mentions_public_reexport_target(
                    &record.item.to_token_stream(),
                    module_path,
                    visible_name,
                )
            })
        })
        || reduced
            .reachable_items
            .iter()
            .filter(|item| {
                item.package == package && *item != target_item && item.kind != ItemKind::Mod
            })
            .any(|item| {
                project.items.get(item).is_some_and(|record| {
                    token_stream_mentions_public_reexport_target(
                        &record.item.to_token_stream(),
                        module_path,
                        visible_name,
                    )
                })
            })
}

fn token_stream_mentions_public_reexport_target(
    tokens: &TokenStream,
    module_path: &[String],
    visible_name: &str,
) -> bool {
    if module_path.is_empty() {
        return token_stream_mentions_unqualified_ident(tokens, visible_name);
    }
    token_stream_mentions_module_path_ident(tokens, module_path, visible_name)
}

fn collect_use_tree_visible_targets(
    tree: &UseTree,
    mut prefix: Vec<String>,
    targets: &mut Vec<(String, Vec<String>)>,
) {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            collect_use_tree_visible_targets(&path.tree, prefix, targets);
        }
        UseTree::Name(name) => {
            let ident = name.ident.to_string();
            let visible_name = if ident == "self" {
                prefix.last().cloned().unwrap_or_else(|| ident.clone())
            } else {
                ident.clone()
            };
            let mut target = prefix;
            if ident != "self" {
                target.push(ident);
            }
            if !target.is_empty() {
                targets.push((visible_name, target));
            }
        }
        UseTree::Rename(rename) => {
            let ident = rename.ident.to_string();
            let mut target = prefix;
            if ident != "self" {
                target.push(ident);
            }
            if !target.is_empty() {
                targets.push((rename.rename.to_string(), target));
            }
        }
        UseTree::Group(group) => {
            for nested in &group.items {
                collect_use_tree_visible_targets(nested, prefix.clone(), targets);
            }
        }
        UseTree::Glob(_) => {}
    }
}

fn source_tree_rs_files(path: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    if !path.exists() {
        return Ok(files);
    }
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
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
    let mut visited_dirs = BTreeSet::new();
    copy_source_tree_path_inner(package, path, package_output, &mut visited_dirs)
}

fn copy_source_tree_path_inner(
    package: &Package,
    path: &Path,
    package_output: &Path,
    visited_dirs: &mut BTreeSet<PathBuf>,
) -> Result<usize, Box<dyn std::error::Error>> {
    let Some(resolved) = resolve_package_copy_source(package, path)? else {
        return Ok(0);
    };
    let metadata = fs::metadata(&resolved)?;
    if metadata.is_file() {
        copy_package_file(package, &resolved, path, package_output)?;
        return Ok(1);
    }
    if !metadata.is_dir() {
        return Ok(0);
    }
    if !visited_dirs.insert(resolved.clone()) {
        return Ok(0);
    }

    let mut copied = 0;
    for entry in fs::read_dir(&resolved)? {
        let entry = entry?;
        copied += copy_source_tree_path_inner(
            package,
            &path.join(entry.file_name()),
            package_output,
            visited_dirs,
        )?;
    }
    Ok(copied)
}

fn package_should_copy_library_support_source(package: &Package) -> bool {
    package_target_uses_dev_dependencies(package) && package_library_source_path(package).is_some()
}

fn package_is_proc_macro(project: &Project, package_name: &str) -> bool {
    let Some(package) = project.workspace.packages.get(package_name) else {
        return false;
    };
    package
        .manifest
        .get("lib")
        .and_then(|lib| lib.get("proc-macro"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
        || package
            .entry_target
            .kind
            .iter()
            .any(|kind| kind == "proc-macro")
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

fn support_library_source_path(package: &SupportPackage) -> Option<PathBuf> {
    support_library_source_path_from(&package.root, &package.manifest)
}

fn support_library_source_path_from(package_root: &Path, manifest: &Value) -> Option<PathBuf> {
    if let Some(path) = manifest
        .get("lib")
        .and_then(|lib| lib.get("path"))
        .and_then(Value::as_str)
    {
        let path = package_root.join(path);
        return path.exists().then_some(path);
    }
    let path = package_root.join("src/lib.rs");
    path.exists().then_some(path)
}

fn build_script_path(package: &Package) -> Option<PathBuf> {
    manifest_build_script_path(&package.root, &package.manifest)
}

fn support_build_script_path(package: &SupportPackage) -> Option<PathBuf> {
    manifest_build_script_path(&package.root, &package.manifest)
}

fn manifest_build_script_path(root: &Path, manifest: &Value) -> Option<PathBuf> {
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
    let mut visited_dirs = BTreeSet::new();
    copy_non_rust_path_assets_inner(package, path, package_output, &mut visited_dirs)
}

fn copy_non_rust_path_assets_inner(
    package: &Package,
    path: &Path,
    package_output: &Path,
    visited_dirs: &mut BTreeSet<PathBuf>,
) -> Result<usize, Box<dyn std::error::Error>> {
    if path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| matches!(name, "target" | ".git"))
    {
        return Ok(0);
    }

    let Some(resolved) = resolve_package_copy_source(package, path)? else {
        return Ok(0);
    };
    let metadata = fs::metadata(&resolved)?;
    if metadata.is_file() {
        if should_copy_asset(path) {
            copy_asset(package, &resolved, path, package_output)?;
            return Ok(1);
        }
        return Ok(0);
    }
    if !metadata.is_dir() {
        return Ok(0);
    }
    if !visited_dirs.insert(resolved.clone()) {
        return Ok(0);
    }

    let mut copied = 0;
    for entry in fs::read_dir(&resolved)? {
        let entry = entry?;
        copied += copy_non_rust_path_assets_inner(
            package,
            &path.join(entry.file_name()),
            package_output,
            visited_dirs,
        )?;
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
        let path = match candidate {
            StaticIncludePath::SourceRelative(path) => source_dir.join(path),
            StaticIncludePath::PackageRelative(path) => package.root.join(path),
            StaticIncludePath::Absolute(_) => continue,
        };
        let Some(resolved) = resolve_package_copy_source(package, &path)? else {
            continue;
        };
        if !fs::metadata(&resolved)?.is_file() {
            continue;
        }
        copy_asset(package, &resolved, &path, package_output)?;
        copied += 1;
    }

    Ok(copied)
}

fn collect_include_macro_paths(tokens: &TokenStream, candidates: &mut BTreeSet<StaticIncludePath>) {
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
                if let Some(path) = static_include_path(&group.stream()) {
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

fn copy_asset(
    package: &Package,
    source_path: &Path,
    package_path: &Path,
    package_output: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    copy_package_file(package, source_path, package_path, package_output)
}

fn copy_package_file(
    package: &Package,
    source_path: &Path,
    package_path: &Path,
    package_output: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let relative_path = package_relative_copy_path(package, package_path)?;
    let output_path = package_output.join(relative_path);
    if output_path.exists() {
        return Ok(());
    }
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(source_path, output_path)?;
    Ok(())
}

fn resolve_package_copy_source(
    package: &Package,
    path: &Path,
) -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
    match fs::symlink_metadata(path) {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    }
    let Ok(resolved) = path.canonicalize() else {
        return Ok(None);
    };
    if !resolved.starts_with(&package.root) {
        return Ok(None);
    }
    Ok(Some(resolved))
}

fn package_relative_copy_path(
    package: &Package,
    path: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let relative = path.strip_prefix(&package.root).map_err(|_| {
        format!(
            "copy path {} is not relative to package root {}",
            path.display(),
            package.root.display()
        )
    })?;
    let mut normalized = PathBuf::new();
    for component in relative.components() {
        match component {
            Component::Normal(component) => normalized.push(component),
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err(format!(
                        "copy path {} escapes package root {}",
                        path.display(),
                        package.root.display()
                    )
                    .into());
                }
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(format!(
                    "copy path {} is not relative to package root {}",
                    path.display(),
                    package.root.display()
                )
                .into());
            }
        }
    }
    if normalized.as_os_str().is_empty() {
        return Err(format!(
            "copy path {} resolves to package root {}",
            path.display(),
            package.root.display()
        )
        .into());
    }
    Ok(normalized)
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
    support_packages: &SupportPackagePlan,
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
    if !support_packages.packages.is_empty() {
        workspace.insert(
            "exclude".to_string(),
            Value::Array(
                support_packages
                    .packages
                    .values()
                    .map(|package| Value::String(toml_path(&package.output_rel_dir)))
                    .collect(),
            ),
        );
    }

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
    if let Some(lints) = project
        .workspace
        .manifest
        .get("workspace")
        .and_then(|workspace| workspace.get("lints"))
    {
        workspace.insert("lints".to_string(), lints.clone());
    }

    let workspace_dependencies =
        retained_workspace_dependencies(project, reduced, package_usages, support_packages);
    let retained_patch_names = retained_patch_dependency_names(
        project,
        reduced,
        package_usages,
        support_packages,
        &workspace_dependencies,
    )?;
    if !workspace_dependencies.is_empty() {
        workspace.insert(
            "dependencies".to_string(),
            Value::Table(workspace_dependencies),
        );
    }

    root.insert("workspace".to_string(), Value::Table(workspace));
    if let Some(profile) = project.workspace.manifest.get("profile") {
        root.insert("profile".to_string(), profile.clone());
    }
    if let Some(patch) = support_packages.merged_patch_tables(project, &retained_patch_names) {
        root.insert("patch".to_string(), patch);
    }
    if let Some(replace) = support_packages.merged_replace_table(project) {
        root.insert("replace".to_string(), replace);
    }
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
    package_usage: &PackageSourceUsage,
    support_packages: &SupportPackagePlan,
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
    if let Some(lints) = package.manifest.get("lints") {
        manifest.insert("lints".to_string(), lints.clone());
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
        support_packages,
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
            support_packages,
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
            support_packages,
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
        support_packages,
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
    support_packages: &SupportPackagePlan,
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
                        support_packages.transformed_workspace_dependency_value(
                            source,
                            &project.workspace.root,
                        ),
                    );
                }
            }
        }
    }

    dependencies
}

fn retained_patch_dependency_names(
    project: &Project,
    reduced: &ReducedProject,
    package_usages: &HashMap<String, PackageSourceUsage>,
    support_packages: &SupportPackagePlan,
    workspace_dependencies: &Table,
) -> Result<BTreeSet<String>, Box<dyn std::error::Error>> {
    let mut names = BTreeSet::new();
    for (alias, value) in workspace_dependencies {
        names.insert(dependency_patch_name(alias, value));
    }
    names.extend(support_package_patch_dependency_names(support_packages)?);

    for package_name in &reduced.packages {
        let Some(package) = project.workspace.packages.get(package_name) else {
            continue;
        };
        let Some(package_usage) = package_usages.get(package_name) else {
            continue;
        };
        let requested_features = requested_local_features(project, reduced, package_name);
        let feature_required_aliases =
            dependency_aliases_required_by_features(package, &requested_features);
        let retain_for_copied_support_source = package_should_copy_library_support_source(package);

        for (table_name, table) in package_dependency_tables(package) {
            let retention = if table_name == "build-dependencies"
                && build_script_should_render_with_usage(package, package_usage)
            {
                DependencyRetention::BuildScript
            } else {
                DependencyRetention::SourceMentioned
            };

            for (alias, value) in table {
                let dependency_package = dependency_package_name(alias, value);
                let is_feature_required = feature_required_aliases.contains(alias);
                if is_marker_dependency(alias, &dependency_package) {
                    continue;
                }
                if dependency_should_render(
                    project,
                    reduced,
                    package_name,
                    alias,
                    retention,
                    DependencyUsageScope::Any,
                    package_usage,
                    retain_for_copied_support_source,
                ) || is_feature_required
                {
                    names.insert(dependency_patch_name(alias, value));
                }
            }
        }
    }
    Ok(names)
}

fn support_package_patch_dependency_names(
    support_packages: &SupportPackagePlan,
) -> Result<BTreeSet<String>, Box<dyn std::error::Error>> {
    let mut names = BTreeSet::new();
    for package in support_packages.packages.values() {
        for table in support_manifest_dependency_tables(&package.manifest) {
            for (alias, value) in table {
                let (value, _) = materialized_support_dependency_value(package, alias, value)?;
                let dependency_package = dependency_package_name(alias, &value);
                if is_marker_dependency(alias, &dependency_package) {
                    continue;
                }
                names.insert(dependency_patch_name(alias, &value));
            }
        }
    }
    Ok(names)
}

fn support_manifest_dependency_tables(manifest: &Value) -> Vec<&Table> {
    let mut tables = Vec::new();
    for table_name in ["dependencies", "build-dependencies", "dev-dependencies"] {
        if let Some(table) = manifest.get(table_name).and_then(Value::as_table) {
            tables.push(table);
        }
    }
    if let Some(targets) = manifest.get("target").and_then(Value::as_table) {
        for target in targets.values() {
            let Some(target) = target.as_table() else {
                continue;
            };
            for table_name in ["dependencies", "build-dependencies", "dev-dependencies"] {
                if let Some(table) = target.get(table_name).and_then(Value::as_table) {
                    tables.push(table);
                }
            }
        }
    }
    tables
}

fn dependency_patch_name(alias: &str, value: &Value) -> String {
    value
        .as_table()
        .and_then(|table| table.get("package"))
        .and_then(Value::as_str)
        .unwrap_or(alias)
        .to_string()
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
    support_packages: &SupportPackagePlan,
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
                let value = if reduced.packages.contains(&dependency_package) {
                    local_dependency_value(alias, &dependency_package, value)
                } else {
                    support_packages.transformed_dependency_value(
                        value,
                        &package.root,
                        Path::new(package_name),
                    )
                };
                retained_aliases.insert(alias.clone());
                dependencies.insert(alias.clone(), value);
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
                support_packages.transformed_dependency_value(
                    value,
                    &package.root,
                    Path::new(package_name),
                ),
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
    support_packages: &SupportPackagePlan,
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
                support_packages,
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
                support_packages,
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
                    support_packages,
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
                    support_packages,
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
    support_packages: &SupportPackagePlan,
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
                let value = if reduced.packages.contains(&dependency_package) {
                    local_dependency_value(alias, &dependency_package, value)
                } else {
                    support_packages.transformed_dependency_value(
                        value,
                        &package.root,
                        Path::new(package_name),
                    )
                };
                retained_aliases.insert(alias.clone());
                dependencies.insert(alias.clone(), value);
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
                .map(|package| {
                    support_packages.transformed_dependency_value(
                        value,
                        &package.root,
                        Path::new(package_name),
                    )
                })
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

    fn dependency_public_names(&self, alias: &str) -> Option<BTreeSet<String>> {
        self.all.dependency_public_names(alias)
    }
}

#[derive(Clone, Default)]
struct TokenUsage {
    idents: BTreeSet<String>,
    bare_idents: BTreeSet<String>,
    path_roots: BTreeSet<String>,
    path_candidates: BTreeSet<Vec<String>>,
    local_path_leaf_idents: BTreeSet<String>,
    use_idents: BTreeSet<String>,
    dependency_root_aliases: BTreeMap<String, BTreeSet<Vec<String>>>,
    dependency_public_names: BTreeMap<String, BTreeSet<String>>,
}

impl TokenUsage {
    fn record_file(&mut self, file: &syn::File) {
        collect_token_usage(&file.to_token_stream(), self);
        self.record_dependency_method_calls(file);
        self.record_dependency_methods_through_typed_values(file);
        self.record_dependency_assoc_calls_through_imports(file);
        for item in &file.items {
            if let Item::Use(item_use) = item {
                self.record_use(item_use);
            }
        }
    }

    fn record_use(&mut self, item_use: &syn::ItemUse) {
        collect_token_usage(&item_use.to_token_stream(), self);
        collect_use_tree_idents(&item_use.tree, &mut self.use_idents);
        collect_use_tree_dependency_root_aliases(
            &item_use.tree,
            Vec::new(),
            &mut self.dependency_root_aliases,
        );
        collect_use_tree_dependency_public_names(
            &item_use.tree,
            Vec::new(),
            &mut self.dependency_public_names,
        );
    }

    fn record_dependency_method_calls(&mut self, file: &syn::File) {
        let mut visitor = DependencyMethodCallVisitor::default();
        visitor.visit_file(file);
        for (root, type_path, method) in visitor.calls {
            self.record_dependency_assoc_requirement(root, type_path, method);
        }
    }

    fn record_dependency_methods_through_typed_values(&mut self, file: &syn::File) {
        let mut visitor = DependencyTypedValueMethodVisitor {
            type_imports: dependency_type_imports(file),
            macro_method_requirements: support_macro_metavariable_method_requirements_from_items(
                &file.items,
            ),
            ..Default::default()
        };
        visitor.visit_file(file);
        for (root, type_path, method) in visitor.calls {
            self.record_dependency_assoc_requirement(root, type_path, method);
        }
    }

    fn record_dependency_assoc_requirement(
        &mut self,
        root: String,
        mut type_path: Vec<String>,
        assoc_name: String,
    ) {
        type_path.push(assoc_name);
        if let Some(required_path) = dependency_required_path_string(&type_path) {
            self.dependency_public_names
                .entry(root)
                .or_default()
                .insert(required_path);
        }
    }

    fn record_dependency_assoc_calls_through_imports(&mut self, file: &syn::File) {
        let imports = dependency_type_imports(file);
        if imports.is_empty() {
            return;
        }

        let mut visitor = SupportAssocUsageVisitor::default();
        visitor.visit_file(file);
        for assoc_path in visitor.assoc_paths {
            self.record_imported_assoc_path(&imports, &assoc_path);
        }

        for assoc_path in token_path_candidates(&file.to_token_stream()) {
            if support_required_assoc_segments(&assoc_path).is_some() {
                self.record_imported_assoc_path(&imports, &assoc_path);
            }
        }
    }

    fn record_imported_assoc_path(
        &mut self,
        imports: &BTreeMap<String, (String, Vec<String>)>,
        assoc_path: &[String],
    ) {
        let Some((visible_name, assoc_rest)) = assoc_path.split_first() else {
            return;
        };
        let Some((root, imported_type_path)) = imports.get(visible_name) else {
            return;
        };
        let mut required_path = imported_type_path.clone();
        required_path.extend(assoc_rest.iter().cloned());
        if let Some(required_path) = dependency_required_path_string(&required_path) {
            self.dependency_public_names
                .entry(root.clone())
                .or_default()
                .insert(required_path);
        }
    }

    fn mentions_ident(&self, ident: &str) -> bool {
        self.idents.contains(ident)
    }

    fn merge(&mut self, other: &TokenUsage) {
        self.idents.extend(other.idents.iter().cloned());
        self.bare_idents.extend(other.bare_idents.iter().cloned());
        self.path_roots.extend(other.path_roots.iter().cloned());
        self.path_candidates
            .extend(other.path_candidates.iter().cloned());
        self.local_path_leaf_idents
            .extend(other.local_path_leaf_idents.iter().cloned());
        self.use_idents.extend(other.use_idents.iter().cloned());
        for (local, targets) in &other.dependency_root_aliases {
            self.dependency_root_aliases
                .entry(local.clone())
                .or_default()
                .extend(targets.iter().cloned());
        }
        for (root, names) in &other.dependency_public_names {
            self.dependency_public_names
                .entry(root.clone())
                .or_default()
                .extend(names.iter().cloned());
        }
    }

    fn mentions_dependency(&self, alias: &str) -> bool {
        let code_name = dependency_code_name(alias);
        self.path_roots.contains(&code_name)
            || self.use_idents.contains(&code_name)
            || (alias != code_name
                && (self.path_roots.contains(alias) || self.use_idents.contains(alias)))
            || self.dependency_root_aliases.iter().any(|(local, targets)| {
                self.path_roots.contains(local)
                    && targets.iter().any(|target| {
                        target
                            .first()
                            .is_some_and(|root| root == alias || root == &code_name)
                    })
            })
            || known_macro_dependency_usage(self, alias, &code_name)
    }

    fn dependency_public_names(&self, alias: &str) -> Option<BTreeSet<String>> {
        if !self.mentions_dependency(alias) {
            return None;
        }
        let code_name = dependency_code_name(alias);
        let roots = BTreeSet::from([alias.to_string(), code_name.clone()]);

        let mut names = BTreeSet::new();
        for root in roots {
            if let Some(root_names) = self.dependency_public_names.get(&root) {
                names.extend(root_names.iter().cloned());
            }
        }
        for (local, targets) in &self.dependency_root_aliases {
            for target in targets {
                let Some(root) = target.first() else {
                    continue;
                };
                if root != alias && root != &code_name {
                    continue;
                }
                if target.len() == 1 {
                    if dependency_alias_target_is_module_like(local, target) {
                        if let Some(local_names) = self.dependency_public_names.get(local) {
                            for local_name in local_names {
                                let path = local_name
                                    .split("::")
                                    .map(str::to_string)
                                    .collect::<Vec<_>>();
                                if let Some(path) = dependency_required_path_string(&path) {
                                    names.insert(path);
                                }
                            }
                        }
                    }
                    continue;
                }
                if let Some(target_name) = dependency_required_path_string(&target[1..]) {
                    names.insert(target_name.clone());
                    if dependency_alias_target_is_module_like(local, target) {
                        if let Some(local_names) = self.dependency_public_names.get(local) {
                            for local_name in local_names {
                                let mut path = target[1..].to_vec();
                                path.extend(local_name.split("::").map(str::to_string));
                                if let Some(path) = dependency_required_path_string(&path) {
                                    names.insert(path);
                                }
                            }
                        }
                    }
                }
            }
        }
        Some(names)
    }
}

#[derive(Default)]
struct DependencyMethodCallVisitor {
    calls: Vec<(String, Vec<String>, String)>,
}

impl Visit<'_> for DependencyMethodCallVisitor {
    fn visit_expr_method_call(&mut self, node: &syn::ExprMethodCall) {
        if support_assoc_item_name_is_precise(&node.method.to_string()) {
            if let Some((root, type_path)) = dependency_receiver_type_path(&node.receiver) {
                self.calls.push((root, type_path, node.method.to_string()));
            }
        }
        visit::visit_expr_method_call(self, node);
    }
}

#[derive(Default)]
struct DependencyTypedValueMethodVisitor {
    scopes: Vec<BTreeMap<String, (String, Vec<String>)>>,
    calls: Vec<(String, Vec<String>, String)>,
    type_imports: BTreeMap<String, (String, Vec<String>)>,
    macro_method_requirements: BTreeMap<String, MacroMetavariableMethodRequirement>,
}

impl DependencyTypedValueMethodVisitor {
    fn push_scope(&mut self) {
        self.scopes.push(BTreeMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn insert_binding(&mut self, name: String, dependency_type: (String, Vec<String>)) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, dependency_type);
        }
    }

    fn lookup_binding(&self, name: &str) -> Option<(String, Vec<String>)> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).cloned())
    }

    fn record_fn_inputs<'ast, I>(&mut self, inputs: I)
    where
        I: IntoIterator<Item = &'ast syn::FnArg>,
    {
        for input in inputs {
            let syn::FnArg::Typed(typed) = input else {
                continue;
            };
            let Some(binding) = pat_single_ident(&typed.pat) else {
                continue;
            };
            let Some(dependency_type) = self.dependency_type_path_from_type(&typed.ty) else {
                continue;
            };
            self.insert_binding(binding, dependency_type);
        }
    }

    fn scoped_receiver_type_path(&self, expr: &Expr) -> Option<(String, Vec<String>)> {
        match expr {
            Expr::Path(path) if path.path.segments.len() == 1 => {
                let ident = path.path.segments.first()?.ident.to_string();
                self.lookup_binding(&ident)
            }
            Expr::MethodCall(method_call) => self.scoped_receiver_type_path(&method_call.receiver),
            Expr::Reference(reference) => self.scoped_receiver_type_path(&reference.expr),
            Expr::Paren(paren) => self.scoped_receiver_type_path(&paren.expr),
            Expr::Group(group) => self.scoped_receiver_type_path(&group.expr),
            Expr::Try(expr_try) => self.scoped_receiver_type_path(&expr_try.expr),
            Expr::Await(expr_await) => self.scoped_receiver_type_path(&expr_await.base),
            _ => self.dependency_receiver_type_path(expr),
        }
    }

    fn dependency_receiver_type_path(&self, expr: &Expr) -> Option<(String, Vec<String>)> {
        match expr {
            Expr::Call(call) => match call.func.as_ref() {
                Expr::Path(path) => self.dependency_type_path_from_associated_path(&path.path),
                _ => None,
            },
            Expr::MethodCall(method_call) => {
                self.dependency_receiver_type_path(&method_call.receiver)
            }
            Expr::Struct(expr_struct) => self.dependency_type_path_from_path(&expr_struct.path),
            Expr::Path(expr_path) => self.dependency_type_path_from_path(&expr_path.path),
            Expr::Reference(reference) => self.dependency_receiver_type_path(&reference.expr),
            Expr::Paren(paren) => self.dependency_receiver_type_path(&paren.expr),
            Expr::Group(group) => self.dependency_receiver_type_path(&group.expr),
            Expr::Try(expr_try) => self.dependency_receiver_type_path(&expr_try.expr),
            Expr::Await(expr_await) => self.dependency_receiver_type_path(&expr_await.base),
            _ => None,
        }
    }

    fn dependency_type_path_from_type(&self, ty: &Type) -> Option<(String, Vec<String>)> {
        match ty {
            Type::Path(type_path) => self.dependency_type_path_from_path(&type_path.path),
            Type::Reference(reference) => self.dependency_type_path_from_type(&reference.elem),
            Type::Paren(paren) => self.dependency_type_path_from_type(&paren.elem),
            Type::Group(group) => self.dependency_type_path_from_type(&group.elem),
            _ => None,
        }
    }

    fn dependency_type_path_from_associated_path(
        &self,
        path: &syn::Path,
    ) -> Option<(String, Vec<String>)> {
        dependency_type_path_from_associated_path(path).or_else(|| {
            let segments = path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<_>>();
            let (assoc_name, type_segments) = segments.split_last()?;
            if !support_assoc_item_name_is_precise(assoc_name) {
                return None;
            }
            self.dependency_type_path_from_segments(type_segments)
        })
    }

    fn dependency_type_path_from_path(&self, path: &syn::Path) -> Option<(String, Vec<String>)> {
        dependency_type_path_from_path(path).or_else(|| {
            let segments = path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<_>>();
            self.dependency_type_path_from_segments(&segments)
        })
    }

    fn dependency_type_path_from_segments(
        &self,
        segments: &[String],
    ) -> Option<(String, Vec<String>)> {
        let (first, rest) = segments.split_first()?;
        let (root, imported_type_path) = self.type_imports.get(first)?;
        let mut type_path = imported_type_path.clone();
        type_path.extend(rest.iter().cloned());
        Some((root.clone(), type_path))
    }

    fn scoped_token_method_calls(
        &self,
        tokens: &TokenStream,
    ) -> Vec<(String, Vec<String>, String)> {
        let mut calls = Vec::new();
        self.collect_scoped_token_method_calls(tokens, &mut calls);
        calls
    }

    fn collect_scoped_token_method_calls(
        &self,
        tokens: &TokenStream,
        calls: &mut Vec<(String, Vec<String>, String)>,
    ) {
        let token_trees = tokens.clone().into_iter().collect::<Vec<_>>();
        for token in &token_trees {
            if let TokenTree::Group(group) = token {
                self.collect_scoped_token_method_calls(&group.stream(), calls);
            }
        }

        for window in token_trees.windows(4) {
            let [TokenTree::Ident(receiver), TokenTree::Punct(dot), TokenTree::Ident(method), TokenTree::Group(args)] =
                window
            else {
                continue;
            };
            if dot.as_char() != '.'
                || args.delimiter() != Delimiter::Parenthesis
                || !support_assoc_item_name_is_precise(&method.to_string())
            {
                continue;
            }
            if let Some((root, type_path)) = self.lookup_binding(&receiver.to_string()) {
                calls.push((root, type_path, method.to_string()));
            }
        }
    }

    fn scoped_macro_metavariable_method_calls(
        &self,
        mac: &syn::Macro,
    ) -> Vec<(String, Vec<String>, String)> {
        let Some(macro_name) = macro_invocation_single_ident(&mac.path) else {
            return Vec::new();
        };
        let Some(requirement) = self.macro_method_requirements.get(&macro_name) else {
            return Vec::new();
        };
        let invocation_args = macro_invocation_arg_idents(&mac.tokens);
        let mut calls = Vec::new();
        for (metavariable, arg) in requirement.metavariables.iter().zip(invocation_args) {
            let Some(methods) = requirement.methods.get(metavariable) else {
                continue;
            };
            let Some((root, type_path)) = self.lookup_binding(&arg) else {
                continue;
            };
            for method in methods {
                calls.push((root.clone(), type_path.clone(), method.clone()));
            }
        }
        calls
    }
}

impl Visit<'_> for DependencyTypedValueMethodVisitor {
    fn visit_item_fn(&mut self, node: &syn::ItemFn) {
        self.push_scope();
        self.record_fn_inputs(&node.sig.inputs);
        self.visit_block(&node.block);
        self.pop_scope();
    }

    fn visit_impl_item_fn(&mut self, node: &syn::ImplItemFn) {
        self.push_scope();
        self.record_fn_inputs(&node.sig.inputs);
        self.visit_block(&node.block);
        self.pop_scope();
    }

    fn visit_local(&mut self, node: &syn::Local) {
        if let Some(init) = &node.init {
            self.visit_expr(&init.expr);
        }

        let explicit_type = pat_dependency_type_annotation(&node.pat)
            .or_else(|| pat_dependency_type_annotation_with_imports(&node.pat, &self.type_imports));
        let inferred_type = node
            .init
            .as_ref()
            .and_then(|init| self.scoped_receiver_type_path(&init.expr));
        if let (Some(binding), Some(dependency_type)) = (
            pat_single_ident_or_typed_ident(&node.pat),
            explicit_type.or(inferred_type),
        ) {
            self.insert_binding(binding, dependency_type);
        }
    }

    fn visit_expr_method_call(&mut self, node: &syn::ExprMethodCall) {
        if support_assoc_item_name_is_precise(&node.method.to_string()) {
            if let Some((root, type_path)) = self.scoped_receiver_type_path(&node.receiver) {
                self.calls.push((root, type_path, node.method.to_string()));
            }
        }
        visit::visit_expr_method_call(self, node);
    }

    fn visit_macro(&mut self, node: &syn::Macro) {
        self.calls
            .extend(self.scoped_macro_metavariable_method_calls(node));
        self.calls
            .extend(self.scoped_token_method_calls(&node.tokens));
        visit::visit_macro(self, node);
    }

    fn visit_block(&mut self, node: &syn::Block) {
        self.push_scope();
        visit::visit_block(self, node);
        self.pop_scope();
    }
}

fn dependency_receiver_type_path(expr: &Expr) -> Option<(String, Vec<String>)> {
    match expr {
        Expr::Call(call) => match call.func.as_ref() {
            Expr::Path(path) => dependency_type_path_from_associated_path(&path.path),
            _ => None,
        },
        Expr::MethodCall(method_call) => dependency_receiver_type_path(&method_call.receiver),
        Expr::Struct(expr_struct) => dependency_type_path_from_path(&expr_struct.path),
        Expr::Path(expr_path) => dependency_type_path_from_path(&expr_path.path),
        Expr::Reference(reference) => dependency_receiver_type_path(&reference.expr),
        Expr::Paren(paren) => dependency_receiver_type_path(&paren.expr),
        Expr::Group(group) => dependency_receiver_type_path(&group.expr),
        Expr::Try(expr_try) => dependency_receiver_type_path(&expr_try.expr),
        Expr::Await(expr_await) => dependency_receiver_type_path(&expr_await.base),
        _ => None,
    }
}

fn dependency_type_path_from_type(ty: &Type) -> Option<(String, Vec<String>)> {
    match ty {
        Type::Path(type_path) => dependency_type_path_from_path(&type_path.path),
        Type::Reference(reference) => dependency_type_path_from_type(&reference.elem),
        Type::Paren(paren) => dependency_type_path_from_type(&paren.elem),
        Type::Group(group) => dependency_type_path_from_type(&group.elem),
        _ => None,
    }
}

fn pat_single_ident_or_typed_ident(pat: &Pat) -> Option<String> {
    pat_single_ident(pat).or_else(|| match pat {
        Pat::Type(typed) => pat_single_ident(&typed.pat),
        _ => None,
    })
}

fn pat_single_ident(pat: &Pat) -> Option<String> {
    match pat {
        Pat::Ident(ident) => Some(ident.ident.to_string()),
        Pat::Reference(reference) => pat_single_ident(&reference.pat),
        Pat::Paren(paren) => pat_single_ident(&paren.pat),
        _ => None,
    }
}

fn pat_dependency_type_annotation(pat: &Pat) -> Option<(String, Vec<String>)> {
    match pat {
        Pat::Type(typed) => dependency_type_path_from_type(&typed.ty),
        Pat::Reference(reference) => pat_dependency_type_annotation(&reference.pat),
        Pat::Paren(paren) => pat_dependency_type_annotation(&paren.pat),
        _ => None,
    }
}

fn pat_dependency_type_annotation_with_imports(
    pat: &Pat,
    imports: &BTreeMap<String, (String, Vec<String>)>,
) -> Option<(String, Vec<String>)> {
    match pat {
        Pat::Type(typed) => dependency_type_path_from_type_with_imports(&typed.ty, imports),
        Pat::Reference(reference) => {
            pat_dependency_type_annotation_with_imports(&reference.pat, imports)
        }
        Pat::Paren(paren) => pat_dependency_type_annotation_with_imports(&paren.pat, imports),
        _ => None,
    }
}

fn dependency_type_path_from_type_with_imports(
    ty: &Type,
    imports: &BTreeMap<String, (String, Vec<String>)>,
) -> Option<(String, Vec<String>)> {
    match ty {
        Type::Path(type_path) => {
            let segments = type_path
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<_>>();
            dependency_type_path_from_segments_with_imports(&segments, imports)
        }
        Type::Reference(reference) => {
            dependency_type_path_from_type_with_imports(&reference.elem, imports)
        }
        Type::Paren(paren) => dependency_type_path_from_type_with_imports(&paren.elem, imports),
        Type::Group(group) => dependency_type_path_from_type_with_imports(&group.elem, imports),
        _ => None,
    }
}

fn dependency_type_path_from_segments_with_imports(
    segments: &[String],
    imports: &BTreeMap<String, (String, Vec<String>)>,
) -> Option<(String, Vec<String>)> {
    let (first, rest) = segments.split_first()?;
    let (root, imported_type_path) = imports.get(first)?;
    let mut type_path = imported_type_path.clone();
    type_path.extend(rest.iter().cloned());
    Some((root.clone(), type_path))
}

fn dependency_type_path_from_associated_path(path: &syn::Path) -> Option<(String, Vec<String>)> {
    let segments = path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>();
    let (assoc_name, type_path) = segments.split_last()?;
    if !support_assoc_item_name_is_precise(assoc_name) {
        return None;
    }
    dependency_type_path_from_segments(type_path)
}

fn dependency_type_path_from_path(path: &syn::Path) -> Option<(String, Vec<String>)> {
    let segments = path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>();
    dependency_type_path_from_segments(&segments)
}

fn dependency_type_path_from_segments(segments: &[String]) -> Option<(String, Vec<String>)> {
    if segments.len() < 2 {
        return None;
    }
    let root = segments.first()?;
    if matches!(root.as_str(), "crate" | "self" | "super" | "Self") {
        return None;
    }
    let type_index = segments
        .iter()
        .rposition(|segment| support_type_like_ident(segment))?;
    if type_index == 0 {
        return None;
    }
    Some((root.clone(), segments[1..=type_index].to_vec()))
}

fn dependency_type_imports(file: &syn::File) -> BTreeMap<String, (String, Vec<String>)> {
    let mut imports = BTreeMap::new();
    for item in &file.items {
        let Item::Use(item_use) = item else {
            continue;
        };
        collect_dependency_type_imports(&item_use.tree, Vec::new(), &mut imports);
    }
    imports
}

fn collect_dependency_type_imports(
    tree: &UseTree,
    mut prefix: Vec<String>,
    imports: &mut BTreeMap<String, (String, Vec<String>)>,
) {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            collect_dependency_type_imports(&path.tree, prefix, imports);
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
            record_dependency_type_import(&prefix, &name.ident.to_string(), &visible_name, imports);
        }
        UseTree::Rename(rename) => {
            record_dependency_type_import(
                &prefix,
                &rename.ident.to_string(),
                &rename.rename.to_string(),
                imports,
            );
        }
        UseTree::Group(group) => {
            for item in &group.items {
                collect_dependency_type_imports(item, prefix.clone(), imports);
            }
        }
        UseTree::Glob(_) => {}
    }
}

fn record_dependency_type_import(
    prefix: &[String],
    imported_name: &str,
    visible_name: &str,
    imports: &mut BTreeMap<String, (String, Vec<String>)>,
) {
    if !support_type_like_ident(visible_name) && !support_type_like_ident(imported_name) {
        return;
    }
    let Some(root) = prefix.first() else {
        return;
    };
    if matches!(root.as_str(), "crate" | "self" | "super") {
        return;
    }
    let mut imported_type_path = prefix[1..].to_vec();
    if imported_name != "self" {
        imported_type_path.push(imported_name.to_string());
    }
    if imported_type_path.is_empty()
        || !imported_type_path
            .iter()
            .any(|segment| support_type_like_ident(segment))
    {
        return;
    }
    imports.insert(visible_name.to_string(), (root.clone(), imported_type_path));
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
            TokenTree::Literal(literal) => {
                collect_literal_format_captures(literal, &mut usage.idents)
            }
            TokenTree::Punct(_) => {}
        }
    }

    for (index, token) in token_trees.iter().enumerate() {
        let TokenTree::Ident(ident) = token else {
            continue;
        };
        let follows_path_separator = index >= 2
            && matches!(
                (&token_trees[index - 2], &token_trees[index - 1]),
                (TokenTree::Punct(first), TokenTree::Punct(second))
                    if first.as_char() == ':' && second.as_char() == ':'
            );
        let precedes_path_separator = token_trees.get(index + 1..index + 3).is_some_and(|next| {
            matches!(
                next,
                [TokenTree::Punct(first), TokenTree::Punct(second)]
                    if first.as_char() == ':' && second.as_char() == ':'
            )
        });
        if !follows_path_separator && !precedes_path_separator {
            usage.bare_idents.insert(ident.to_string());
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

    for segments in token_path_candidates(tokens) {
        usage.path_candidates.insert(segments.clone());
        if let [root, rest @ ..] = segments.as_slice() {
            if matches!(root.as_str(), "crate" | "self" | "super") {
                if let Some(leaf) = local_required_leaf_from_path(rest) {
                    usage.local_path_leaf_idents.insert(leaf);
                }
            }
            if let Some(path) = dependency_required_path_string(rest) {
                usage
                    .dependency_public_names
                    .entry(root.clone())
                    .or_default()
                    .insert(path);
            }
        }
    }
}

fn local_required_leaf_from_path(path: &[String]) -> Option<String> {
    if path.is_empty() {
        return None;
    }
    path.iter()
        .find(|segment| {
            segment
                .chars()
                .next()
                .is_some_and(|ch| ch == '_' || ch.is_ascii_uppercase())
        })
        .cloned()
        .or_else(|| path.last().cloned())
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

fn collect_use_tree_dependency_root_aliases(
    tree: &UseTree,
    mut prefix: Vec<String>,
    aliases: &mut BTreeMap<String, BTreeSet<Vec<String>>>,
) {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            collect_use_tree_dependency_root_aliases(&path.tree, prefix, aliases);
        }
        UseTree::Rename(rename) => {
            let Some(target_root) =
                dependency_root_alias_target(&prefix, &rename.ident.to_string())
            else {
                return;
            };
            aliases
                .entry(rename.rename.to_string())
                .or_default()
                .insert(target_root);
        }
        UseTree::Group(group) => {
            for item in &group.items {
                collect_use_tree_dependency_root_aliases(item, prefix.clone(), aliases);
            }
        }
        UseTree::Name(_) | UseTree::Glob(_) => {}
    }
}

fn dependency_root_alias_target(prefix: &[String], ident: &str) -> Option<Vec<String>> {
    let mut target = prefix.to_vec();
    target.push(ident.to_string());
    let root = target.first()?;
    (!matches!(root.as_str(), "crate" | "self" | "super")).then_some(target)
}

fn collect_use_tree_dependency_public_names(
    tree: &UseTree,
    mut prefix: Vec<String>,
    names: &mut BTreeMap<String, BTreeSet<String>>,
) {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            collect_use_tree_dependency_public_names(&path.tree, prefix, names);
        }
        UseTree::Name(name) => {
            if let Some(root) = prefix.first() {
                let mut path = prefix[1..].to_vec();
                path.push(name.ident.to_string());
                if let Some(path) = dependency_required_path_string(&path) {
                    names.entry(root.clone()).or_default().insert(path);
                }
            }
        }
        UseTree::Rename(rename) => {
            if let Some(root) = prefix.first() {
                let mut path = prefix[1..].to_vec();
                path.push(rename.ident.to_string());
                if let Some(path) = dependency_required_path_string(&path) {
                    names.entry(root.clone()).or_default().insert(path);
                }
            }
        }
        UseTree::Group(group) => {
            for item in &group.items {
                collect_use_tree_dependency_public_names(item, prefix.clone(), names);
            }
        }
        UseTree::Glob(_) => {}
    }
}

fn dependency_required_path_string(path: &[String]) -> Option<String> {
    if path.is_empty() {
        return None;
    }
    let retained_len = path
        .iter()
        .position(|segment| support_type_like_ident(segment))
        .map_or(path.len(), |index| {
            let mut retained_len = index + 1;
            if path
                .get(retained_len)
                .is_some_and(|segment| support_assoc_item_name_is_precise(segment))
            {
                retained_len += 1;
            }
            retained_len
        });
    Some(path[..retained_len].join("::"))
}

fn dependency_alias_target_is_module_like(local: &str, target: &[String]) -> bool {
    local
        .chars()
        .next()
        .is_some_and(|ch| ch == '_' || ch.is_ascii_lowercase())
        || target
            .last()
            .and_then(|segment| segment.chars().next())
            .is_some_and(|ch| ch == '_' || ch.is_ascii_lowercase())
}

fn known_macro_dependency_usage(usage: &TokenUsage, alias: &str, code_name: &str) -> bool {
    match code_name {
        "serde" | "serde_derive" => {
            usage.mentions_ident("Serialize")
                || usage.mentions_ident("Deserialize")
                || usage.mentions_ident("serde")
        }
        "thiserror" => usage.mentions_ident("Error") && usage.mentions_ident("error"),
        "strum" => usage.mentions_ident("EnumIter"),
        "strum_macros" => {
            usage.mentions_ident("EnumIter")
                || usage.mentions_ident("Display")
                || usage.mentions_ident("EnumString")
                || usage.mentions_ident("AsRefStr")
        }
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
        "strum" => reachable_package_mentions_ident(project, reduced, package, "EnumIter"),
        "strum_macros" => {
            reachable_package_mentions_ident(project, reduced, package, "EnumIter")
                || reachable_package_mentions_ident(project, reduced, package, "Display")
                || reachable_package_mentions_ident(project, reduced, package, "EnumString")
                || reachable_package_mentions_ident(project, reduced, package, "AsRefStr")
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
        "thiserror" if leaf == "Error" => {
            reachable_package_mentions_ident(project, reduced, package, "Error")
                && reachable_package_mentions_ident(project, reduced, package, "error")
        }
        "thiserror" => false,
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
        TokenTree::Literal(literal) => literal_mentions_format_capture(&literal, ident),
        TokenTree::Punct(_) => false,
    })
}

fn token_stream_mentions_unqualified_ident(tokens: &TokenStream, ident: &str) -> bool {
    let tokens = tokens.clone().into_iter().collect::<Vec<_>>();
    for (index, token) in tokens.iter().enumerate() {
        match token {
            TokenTree::Ident(candidate) if candidate == ident => {
                if !previous_tokens_are_path_separator(&tokens, index) {
                    return true;
                }
            }
            TokenTree::Group(group)
                if token_stream_mentions_unqualified_ident(&group.stream(), ident) =>
            {
                return true;
            }
            _ => {}
        }
    }
    false
}

fn previous_tokens_are_path_separator(tokens: &[TokenTree], index: usize) -> bool {
    if index < 2 {
        return false;
    }
    matches!(
        (&tokens[index - 2], &tokens[index - 1]),
        (TokenTree::Punct(left), TokenTree::Punct(right))
            if left.as_char() == ':' && right.as_char() == ':'
    )
}

fn literal_mentions_format_capture(literal: &proc_macro2::Literal, ident: &str) -> bool {
    let Ok(literal) = syn::parse2::<syn::LitStr>(literal.to_token_stream()) else {
        return false;
    };
    format_string_mentions_capture(&literal.value(), ident)
}

fn collect_literal_format_captures(literal: &proc_macro2::Literal, idents: &mut BTreeSet<String>) {
    let Ok(literal) = syn::parse2::<syn::LitStr>(literal.to_token_stream()) else {
        return;
    };
    collect_format_string_captures(&literal.value(), idents);
}

fn format_string_mentions_capture(value: &str, ident: &str) -> bool {
    let mut captures = BTreeSet::new();
    collect_format_string_captures(value, &mut captures);
    captures.contains(ident)
}

fn collect_format_string_captures(value: &str, idents: &mut BTreeSet<String>) {
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != b'{' {
            index += 1;
            continue;
        }
        index += 1;
        if index < bytes.len() && bytes[index] == b'{' {
            index += 1;
            continue;
        }
        let start = index;
        if index >= bytes.len()
            || !(bytes[index] == b'_' || (bytes[index] as char).is_ascii_alphabetic())
        {
            continue;
        }
        index += 1;
        while index < bytes.len()
            && (bytes[index] == b'_' || (bytes[index] as char).is_ascii_alphanumeric())
        {
            index += 1;
        }
        if index >= bytes.len() || matches!(bytes[index], b'}' | b':' | b'?') {
            if let Some(capture) = value.get(start..index) {
                idents.insert(capture.to_string());
            }
        }
    }
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
    let retained_macro_definitions = retained_macro_definitions_for_generated_items(
        project,
        reduced,
        package,
        module_path,
        items,
    );

    let mut transformed = Vec::new();
    for item in items {
        let item = match item {
            _ if item_is_test(item) && !retain_test_items => None,
            Item::Use(item_use) if use_mentions_opensourced(&item_use.tree) => None,
            Item::Use(item_use)
                if local_macro_self_reexport_should_prune(
                    project,
                    reduced,
                    render_plan,
                    package,
                    module_path,
                    items,
                    item_use,
                ) =>
            {
                None
            }
            Item::Use(item_use)
                if local_macro_self_reexport_should_remain(
                    project,
                    reduced,
                    render_plan,
                    package,
                    module_path,
                    items,
                    item_use,
                ) =>
            {
                Some(Item::Use(item_use.clone()))
            }
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
                render_plan.callable_should_render(&id).then(|| {
                    let mut function = function.clone();
                    strip_opensourced_attrs(&mut function.attrs);
                    if !preserve_uniffi_surface {
                        strip_uniffi_attrs(&mut function.attrs);
                    }
                    prune_rendered_struct_literal_fields(
                        project,
                        reduced,
                        render_plan,
                        package,
                        module_path,
                        None,
                        &mut function.block,
                    );
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
                if should_retain_macro_invocation(
                    project,
                    reduced,
                    package,
                    module_path,
                    item_macro,
                ) =>
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
                        module_path,
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
                    for trait_item in &mut item_trait.items {
                        strip_opensourced_attrs_from_trait_item(trait_item);
                    }
                    if !preserve_uniffi_surface {
                        strip_uniffi_attrs(&mut item_trait.attrs);
                        for trait_item in &mut item_trait.items {
                            strip_uniffi_attrs_from_trait_item(trait_item);
                        }
                    }
                    if !root_item_should_render(reduced, &id) {
                        prune_trait_items_if_only_type_surface(
                            project,
                            reduced,
                            package,
                            module_path,
                            &mut item_trait,
                        );
                    }
                    allow_dead_code_if_not_public(&item_trait.vis, &mut item_trait.attrs);
                    Item::Trait(item_trait)
                })
            }),
            Item::Enum(item_enum) => item_id(package, module_path, item).and_then(|id| {
                render_plan.item_should_render(&id).then(|| {
                    let mut item_enum = item_enum.clone();
                    strip_opensourced_attrs(&mut item_enum.attrs);
                    if !preserve_uniffi_surface {
                        strip_uniffi_attrs(&mut item_enum.attrs);
                        for variant in &mut item_enum.variants {
                            strip_uniffi_attrs_from_variant(variant);
                        }
                    }
                    if !enum_preserves_full_variant_surface(reduced, &id, &item_enum) {
                        prune_private_enum_variants(
                            project,
                            reduced,
                            package,
                            module_path,
                            &mut item_enum,
                        );
                    }
                    allow_dead_code_if_not_public(&item_enum.vis, &mut item_enum.attrs);
                    Item::Enum(item_enum)
                })
            }),
            Item::Union(_) | Item::Type(_) | Item::Const(_) | Item::Static(_) | Item::Macro(_) => {
                item_id(package, module_path, item).and_then(|id| {
                    render_plan.item_should_render(&id).then(|| {
                        let mut item = item.clone();
                        strip_opensourced_attrs_from_item(&mut item);
                        if !preserve_uniffi_surface {
                            strip_uniffi_attrs_from_item(&mut item);
                        }
                        allow_dead_code_for_non_public_item(&mut item);
                        item
                    })
                })
            }
            Item::Impl(item_impl) => {
                let aliases = project
                    .module_aliases
                    .get(&(package.to_string(), module_path.to_vec()))
                    .cloned()
                    .unwrap_or_default();
                let trait_path = item_impl
                    .trait_
                    .as_ref()
                    .map(|(_, path, _)| normalized_path(module_path, path, &aliases));
                let trait_input_type_paths = item_impl
                    .trait_
                    .as_ref()
                    .map(|(_, path, _)| trait_input_type_paths(module_path, path, &aliases))
                    .unwrap_or_default();
                let Some(type_path) = resolved_local_type_path(
                    project,
                    package,
                    module_path,
                    &item_impl.self_ty,
                    &aliases,
                ) else {
                    let Some(trait_item) = trait_path
                        .as_ref()
                        .and_then(|trait_path| trait_item_for_path(project, package, trait_path))
                    else {
                        continue;
                    };
                    if !reduced.reachable_items.contains(&trait_item)
                        || !trait_default_method_is_referenced_by_reachable_callables(
                            project,
                            reduced,
                            &trait_item,
                        )
                        || !type_tokens_are_referenced_by_reachable_callables(
                            project,
                            reduced,
                            &item_impl.self_ty,
                        )
                    {
                        continue;
                    }
                    let mut kept_impl_items = Vec::new();
                    for impl_item in &item_impl.items {
                        if retain_test_items
                            || impl_item_should_render_for_trait_surface(
                                project,
                                reduced,
                                package,
                                &[],
                                trait_path.as_deref(),
                                trait_input_type_paths.as_slice(),
                                Some(&trait_item),
                                item_impl,
                                impl_item,
                            )
                        {
                            let mut impl_item = impl_item.clone();
                            strip_opensourced_attrs_from_impl_item(&mut impl_item);
                            if !preserve_uniffi_surface {
                                strip_uniffi_attrs_from_impl_item(&mut impl_item);
                            }
                            allow_dead_code_for_non_public_impl_item(&mut impl_item);
                            kept_impl_items.push(impl_item);
                        }
                    }
                    let mut item_impl = item_impl.clone();
                    if !preserve_uniffi_surface {
                        strip_uniffi_attrs(&mut item_impl.attrs);
                    }
                    item_impl.items = kept_impl_items;
                    transformed.push(Item::Impl(item_impl));
                    continue;
                };
                let mut kept_impl_items = Vec::new();
                let mut kept_method = false;
                let trait_impl_is_required = trait_path.as_ref().is_some_and(|trait_path| {
                    trait_impl_items_are_reachable(reduced, package, &type_path, trait_path)
                });
                let trait_item = trait_path
                    .as_ref()
                    .and_then(|trait_path| trait_item_for_path(project, package, trait_path));
                let default_trait_impl_is_required =
                    trait_item.as_ref().is_some_and(|trait_item| {
                        reduced.reachable_items.contains(trait_item)
                            && trait_default_method_is_referenced_by_reachable_callables(
                                project, reduced, trait_item,
                            )
                            && trait_impl_self_type_is_referenced_by_reachable_surface(
                                project, reduced, package, &type_path,
                            )
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
                let root_macro_impl_surface_is_required = root_item_impl_surface_should_render(
                    project, reduced, package, &type_path, item_impl,
                );

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
                        if render_plan.callable_should_render(&id)
                            || retained_impl_surfaces_call_inherent_associated_function(
                                project,
                                reduced,
                                package,
                                &type_path,
                                &method.sig.ident.to_string(),
                            )
                        {
                            let mut method = method.clone();
                            strip_opensourced_attrs(&mut method.attrs);
                            if !preserve_uniffi_surface {
                                strip_uniffi_attrs(&mut method.attrs);
                            }
                            prune_rendered_struct_literal_fields(
                                project,
                                reduced,
                                render_plan,
                                package,
                                module_path,
                                Some(&type_path),
                                &mut method.block,
                            );
                            allow_dead_code_if_not_public(&method.vis, &mut method.attrs);
                            kept_impl_items.push(ImplItem::Fn(method));
                            kept_method = true;
                        }
                    }
                }

                if kept_method
                    || trait_impl_is_required
                    || default_trait_impl_is_required
                    || marker_trait_impl_is_required
                    || root_macro_impl_surface_is_required
                {
                    if trait_path.is_some() {
                        kept_impl_items.clear();
                        for impl_item in &item_impl.items {
                            if retain_test_items
                                || impl_item_should_render_for_trait_surface(
                                    project,
                                    reduced,
                                    package,
                                    &type_path,
                                    trait_path.as_deref(),
                                    trait_input_type_paths.as_slice(),
                                    trait_item.as_ref(),
                                    item_impl,
                                    impl_item,
                                )
                            {
                                let mut impl_item = impl_item.clone();
                                strip_opensourced_attrs_from_impl_item(&mut impl_item);
                                if !preserve_uniffi_surface {
                                    strip_uniffi_attrs_from_impl_item(&mut impl_item);
                                }
                                allow_dead_code_for_non_public_impl_item(&mut impl_item);
                                kept_impl_items.push(impl_item);
                            }
                        }
                    } else if root_macro_impl_surface_is_required {
                        kept_impl_items.clear();
                        for impl_item in &item_impl.items {
                            if (retain_test_items || !impl_item_is_test(impl_item))
                                && root_macro_impl_item_should_render(item_impl, impl_item)
                            {
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
            Item::ForeignMod(foreign_mod) => {
                let mut foreign_mod = foreign_mod.clone();
                if !foreign_mod_attrs_require_intact_surface(&foreign_mod.attrs) {
                    foreign_mod.items.retain(|foreign_item| {
                        foreign_item_should_render(render_plan, package, module_path, foreign_item)
                    });
                }
                (!foreign_mod.items.is_empty()).then(|| Item::ForeignMod(foreign_mod))
            }
            _ => Some(item.clone()),
        };

        if let Some(item) = item {
            transformed.push(item);
        }
    }

    transformed
}

fn foreign_mod_attrs_require_intact_surface(attrs: &[syn::Attribute]) -> bool {
    attrs
        .iter()
        .any(|attr| attr.path().is_ident("cfg") || attr.path().is_ident("cfg_attr"))
}

fn foreign_item_should_render(
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
    foreign_item: &ForeignItem,
) -> bool {
    if let Some(name) = foreign_item_name(foreign_item) {
        return render_plan.module_mentions_ident(package, module_path, &name)
            || render_plan.package_callable_mentions_ident(package, &name);
    }

    let mut idents = BTreeSet::new();
    collect_token_idents(&foreign_item.to_token_stream(), &mut idents);
    idents.into_iter().any(|ident| {
        render_plan.module_mentions_ident(package, module_path, &ident)
            || render_plan.package_callable_mentions_ident(package, &ident)
    })
}

fn foreign_item_name(foreign_item: &ForeignItem) -> Option<String> {
    match foreign_item {
        ForeignItem::Fn(item) => Some(item.sig.ident.to_string()),
        ForeignItem::Static(item) => Some(item.ident.to_string()),
        ForeignItem::Type(item) => Some(item.ident.to_string()),
        ForeignItem::Macro(item) => item
            .mac
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string()),
        ForeignItem::Verbatim(_) => None,
        _ => None,
    }
}

fn prune_trait_items_if_only_type_surface(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
    item_trait: &mut syn::ItemTrait,
) {
    let trait_name = item_trait.ident.to_string();
    item_trait.items = item_trait
        .items
        .iter()
        .filter(|item| {
            trait_item_should_remain_for_type_surface(
                project,
                reduced,
                package,
                module_path,
                &trait_name,
                item,
            )
        })
        .cloned()
        .collect();
    if item_trait.items.is_empty() {
        item_trait.attrs.retain(is_inert_type_surface_attr);
    }
}

fn trait_item_should_remain_for_type_surface(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
    trait_name: &str,
    item: &TraitItem,
) -> bool {
    if trait_item_attrs_require_surface_retention(item) {
        return true;
    }
    match item {
        TraitItem::Fn(method) => {
            let name = method.sig.ident.to_string();
            reachable_reduced_callables_mention_ident(project, reduced, &name)
                || retained_root_macro_impl_items_mention_ident(
                    project,
                    reduced,
                    package,
                    module_path,
                    &name,
                )
                || (method.default.is_none()
                    && rendered_impl_for_trait_surface_exists(
                        project,
                        reduced,
                        package,
                        module_path,
                        trait_name,
                    ))
        }
        TraitItem::Const(item) => {
            let name = item.ident.to_string();
            reachable_reduced_callables_mention_ident(project, reduced, &name)
                || (item.default.is_none()
                    && rendered_impl_for_trait_surface_exists(
                        project,
                        reduced,
                        package,
                        module_path,
                        trait_name,
                    ))
        }
        TraitItem::Type(item) => {
            let name = item.ident.to_string();
            reachable_reduced_callables_mention_ident(project, reduced, &name)
                || (item.default.is_none()
                    && rendered_impl_for_trait_surface_exists(
                        project,
                        reduced,
                        package,
                        module_path,
                        trait_name,
                    ))
        }
        _ => true,
    }
}

fn trait_item_attrs_require_surface_retention(item: &TraitItem) -> bool {
    let attrs = match item {
        TraitItem::Const(item) => &item.attrs,
        TraitItem::Fn(item) => &item.attrs,
        TraitItem::Macro(item) => &item.attrs,
        TraitItem::Type(item) => &item.attrs,
        TraitItem::Verbatim(_) => return false,
        _ => return false,
    };
    attrs.iter().any(|attr| !is_inert_type_surface_attr(attr))
}

fn rendered_impl_for_trait_surface_exists(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
    trait_name: &str,
) -> bool {
    let mut expected_trait_path = module_path.to_vec();
    expected_trait_path.push(trait_name.to_string());
    project_module_paths(project, package)
        .into_iter()
        .any(|impl_module_path| {
            let Some(items) = module_items_for_path(project, package, &impl_module_path) else {
                return false;
            };
            let aliases = project
                .module_aliases
                .get(&(package.to_string(), impl_module_path.clone()))
                .cloned()
                .unwrap_or_default();
            items.iter().any(|item| {
                let Item::Impl(item_impl) = item else {
                    return false;
                };
                let Some((_, trait_path, _)) = &item_impl.trait_ else {
                    return false;
                };
                normalized_path(&impl_module_path, trait_path, &aliases) == expected_trait_path
                    && impl_should_render(
                        project,
                        reduced,
                        package,
                        &impl_module_path,
                        item_impl,
                        &aliases,
                    )
            })
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
    }) || inline_module_fallback_macro_should_render(project, render_plan, package, module_path)
}

fn inline_module_fallback_macro_should_render(
    project: &Project,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
) -> bool {
    let Some(module_name) = module_path.last() else {
        return false;
    };
    if !render_plan.package_mentions_ident(package, module_name) {
        return false;
    }

    inline_module_items_for_path(project, package, module_path).is_some_and(|items| {
        items
            .iter()
            .any(|item| matches!(item, Item::Macro(item_macro) if item_macro.ident.is_none()))
    })
}

fn inline_module_items_for_path<'a>(
    project: &'a Project,
    package: &str,
    module_path: &[String],
) -> Option<&'a [Item]> {
    project
        .files
        .values()
        .filter(|source| source.package == package)
        .filter(|source| source.module_path.len() < module_path.len())
        .filter(|source| path_has_prefix(module_path, &source.module_path))
        .max_by_key(|source| source.module_path.len())
        .and_then(|source| {
            find_inline_module_items(
                &source.syntax.items,
                &module_path[source.module_path.len()..],
            )
        })
}

fn find_inline_module_items<'a>(items: &'a [Item], module_path: &[String]) -> Option<&'a [Item]> {
    let (name, rest) = module_path.split_first()?;
    let child_items = items.iter().find_map(|item| {
        let Item::Mod(item_mod) = item else {
            return None;
        };
        if item_mod.ident != name {
            return None;
        }
        item_mod.content.as_ref().map(|(_, items)| items.as_slice())
    })?;

    if rest.is_empty() {
        Some(child_items)
    } else {
        find_inline_module_items(child_items, rest)
    }
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
    module_path: &[String],
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

    macro_invocation_feeds_reachable_code(project, reduced, package, module_path, item_macro)
}

fn retained_macro_definitions_for_generated_items(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
    items: &[Item],
) -> BTreeSet<String> {
    items
        .iter()
        .filter_map(|item| {
            let Item::Macro(item_macro) = item else {
                return None;
            };
            if !should_retain_macro_invocation(project, reduced, package, module_path, item_macro) {
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
    module_path: &[String],
    item_macro: &syn::ItemMacro,
) -> bool {
    let token_idents = macro_token_idents(&item_macro.mac.tokens);
    if token_idents
        .iter()
        .any(|ident| reachable_macro_generated_ident(project, reduced, package, module_path, ident))
    {
        return true;
    }

    macro_definition_tokens_feed_reachable_code(project, reduced, package, module_path, item_macro)
}

fn macro_definition_tokens_feed_reachable_code(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
    item_macro: &syn::ItemMacro,
) -> bool {
    let Some(item) =
        resolve_macro_item_for_invocation(project, package, module_path, &item_macro.mac.path)
    else {
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
        .any(|ident| reachable_macro_generated_ident(project, reduced, package, module_path, ident))
}

fn reachable_macro_generated_ident(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
    ident: &str,
) -> bool {
    if module_path.is_empty() {
        return reachable_non_macro_package_mentions_ident(project, reduced, package, ident);
    }

    reduced.reachable.iter().any(|callable| {
        if callable.package() != package {
            return false;
        }
        project.functions.get(callable).is_some_and(|record| {
            (record.module_path == module_path
                && token_stream_mentions_ident(&record.item.to_token_stream(), ident))
                || token_stream_mentions_module_path_ident(
                    &record.item.to_token_stream(),
                    module_path,
                    ident,
                )
        }) || project.methods.get(callable).is_some_and(|record| {
            (record.module_path == module_path
                && token_stream_mentions_ident(&record.item.to_token_stream(), ident))
                || token_stream_mentions_module_path_ident(
                    &record.item.to_token_stream(),
                    module_path,
                    ident,
                )
        })
    }) || reduced.reachable_items.iter().any(|item| {
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
    reduced: &ReducedProject,
    package: &str,
    ident: &str,
) -> bool {
    reduced.reachable.iter().any(|callable| {
        if callable.package() != package {
            return false;
        }
        callable_mentions_ident(callable, ident)
            || project.functions.get(callable).is_some_and(|record| {
                token_stream_mentions_ident(&record.item.to_token_stream(), ident)
            })
            || project.methods.get(callable).is_some_and(|record| {
                token_stream_mentions_ident(&record.item.to_token_stream(), ident)
            })
    }) || reduced.reachable_items.iter().any(|item| {
        item.package == package
            && item.kind != ItemKind::Macro
            && (item.name == ident
                || item.module_path.iter().any(|segment| segment == ident)
                || project.items.get(item).is_some_and(|record| {
                    token_stream_mentions_ident(&record.item.to_token_stream(), ident)
                }))
    })
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

fn resolve_macro_item_for_invocation(
    project: &Project,
    package: &str,
    module_path: &[String],
    path: &syn::Path,
) -> Option<ItemId> {
    let segments = path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>();
    let (target_package, target_path) =
        resolve_macro_invocation_segments(project, package, module_path, &segments)?;
    if let Some(item) = find_macro_item(project, &target_package, &target_path) {
        return Some(item);
    }

    let name = target_path.last()?;
    let mut matches = project
        .items
        .keys()
        .filter(|item| {
            item.package == target_package && item.kind == ItemKind::Macro && item.name == *name
        })
        .cloned()
        .collect::<Vec<_>>();
    matches.sort();
    (matches.len() == 1).then(|| matches.remove(0))
}

fn resolve_macro_invocation_segments(
    project: &Project,
    package: &str,
    module_path: &[String],
    segments: &[String],
) -> Option<(String, Vec<String>)> {
    let first = segments.first()?;
    match first.as_str() {
        "crate" => Some((package.to_string(), segments[1..].to_vec())),
        "self" => {
            let mut path = module_path.to_vec();
            path.extend_from_slice(&segments[1..]);
            Some((package.to_string(), path))
        }
        "super" => {
            let mut path = module_path.to_vec();
            path.pop();
            path.extend_from_slice(&segments[1..]);
            Some((package.to_string(), path))
        }
        name if name == package => Some((package.to_string(), segments[1..].to_vec())),
        dependency => {
            if let Some(target_package) = resolve_dependency_package(project, package, dependency) {
                return Some((target_package, segments[1..].to_vec()));
            }
            let mut path = module_path.to_vec();
            path.extend_from_slice(segments);
            Some((package.to_string(), path))
        }
    }
}

fn resolve_dependency_package(project: &Project, package: &str, name: &str) -> Option<String> {
    project
        .workspace
        .packages
        .get(package)?
        .dependencies
        .iter()
        .find(|dependency| dependency_name_matches(dependency, name))
        .and_then(|dependency| {
            project
                .workspace
                .packages
                .contains_key(&dependency.package)
                .then(|| dependency.package.clone())
        })
}

fn find_macro_item(project: &Project, package: &str, path: &[String]) -> Option<ItemId> {
    let name = path.last()?.clone();
    let module_path = path[..path.len() - 1].to_vec();
    project
        .items
        .keys()
        .find(|item| {
            item.package == package
                && item.kind == ItemKind::Macro
                && item.module_path == module_path
                && item.name == name
        })
        .cloned()
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

fn macro_pattern_keyword(segments: &[String]) -> bool {
    if segments.len() != 1 {
        return false;
    }

    matches!(
        segments[0].as_str(),
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

fn token_path_candidates(tokens: &TokenStream) -> Vec<Vec<String>> {
    let mut candidates = Vec::new();
    collect_token_path_candidates(tokens, &mut candidates);
    candidates
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

        candidates.push(segments.clone());
        if let Some(last) = segments.last() {
            candidates.push(vec![last.clone()]);
        }

        index = cursor.max(index + 1);
    }
}

fn has_path_separator(tokens: &[TokenTree], index: usize) -> bool {
    matches!(tokens.get(index), Some(TokenTree::Punct(punct)) if punct.as_char() == ':')
        && matches!(tokens.get(index + 1), Some(TokenTree::Punct(punct)) if punct.as_char() == ':')
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
    let Some(items) = module_items_for_path(project, package, module_path) else {
        return false;
    };
    let aliases = project
        .module_aliases
        .get(&(package.to_string(), module_path.to_vec()))
        .cloned()
        .unwrap_or_default();

    items.iter().any(|item| {
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

fn retained_impl_attrs_mention_unqualified_ident(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
    ident: &str,
) -> bool {
    let Some(items) = module_items_for_path(project, package, module_path) else {
        return false;
    };
    let aliases = project
        .module_aliases
        .get(&(package.to_string(), module_path.to_vec()))
        .cloned()
        .unwrap_or_default();

    items.iter().any(|item| {
        let Item::Impl(item_impl) = item else {
            return false;
        };
        if !impl_has_reachable_method(project, reduced, package, module_path, item_impl, &aliases) {
            return false;
        }
        item_impl
            .attrs
            .iter()
            .any(|attr| token_stream_mentions_unqualified_ident(&attr.to_token_stream(), ident))
    })
}

fn retained_impl_headers_mention_unqualified_ident(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
    ident: &str,
) -> bool {
    let Some(items) = module_items_for_path(project, package, module_path) else {
        return false;
    };
    let aliases = project
        .module_aliases
        .get(&(package.to_string(), module_path.to_vec()))
        .cloned()
        .unwrap_or_default();

    items.iter().any(|item| {
        let Item::Impl(item_impl) = item else {
            return false;
        };
        let header_should_render =
            impl_has_reachable_method(project, reduced, package, module_path, item_impl, &aliases)
                || (item_impl.trait_.is_some()
                    && impl_should_render(
                        project,
                        reduced,
                        package,
                        module_path,
                        item_impl,
                        &aliases,
                    ));
        (header_should_render
            || root_macro_impl_surface_should_render_for_module(
                project,
                reduced,
                package,
                module_path,
                item_impl,
                &aliases,
            ))
            && impl_header_mentions_unqualified_ident(item_impl, ident)
    })
}

fn retained_impl_non_fn_items_mention_ident(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
    ident: &str,
) -> bool {
    let Some(items) = module_items_for_path(project, package, module_path) else {
        return false;
    };
    let aliases = project
        .module_aliases
        .get(&(package.to_string(), module_path.to_vec()))
        .cloned()
        .unwrap_or_default();

    items.iter().any(|item| {
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

fn retained_impl_non_fn_items_mention_unqualified_ident(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
    ident: &str,
) -> bool {
    let Some(items) = module_items_for_path(project, package, module_path) else {
        return false;
    };
    let aliases = project
        .module_aliases
        .get(&(package.to_string(), module_path.to_vec()))
        .cloned()
        .unwrap_or_default();

    items.iter().any(|item| {
        let Item::Impl(item_impl) = item else {
            return false;
        };
        if !impl_has_reachable_method(project, reduced, package, module_path, item_impl, &aliases) {
            return false;
        }
        item_impl.items.iter().any(|impl_item| {
            !matches!(impl_item, ImplItem::Fn(_))
                && !impl_item_is_test(impl_item)
                && token_stream_mentions_unqualified_ident(&impl_item.to_token_stream(), ident)
        })
    })
}

fn retained_root_macro_impl_items_mention_ident(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
    ident: &str,
) -> bool {
    let Some(items) = module_items_for_path(project, package, module_path) else {
        return false;
    };
    let aliases = project
        .module_aliases
        .get(&(package.to_string(), module_path.to_vec()))
        .cloned()
        .unwrap_or_default();

    items.iter().any(|item| {
        let Item::Impl(item_impl) = item else {
            return false;
        };
        if !root_macro_impl_surface_should_render_for_module(
            project,
            reduced,
            package,
            module_path,
            item_impl,
            &aliases,
        ) {
            return false;
        }
        item_impl.items.iter().any(|impl_item| {
            !impl_item_is_test(impl_item)
                && root_macro_impl_item_should_render(item_impl, impl_item)
                && token_stream_mentions_ident(&impl_item.to_token_stream(), ident)
        })
    })
}

fn retained_root_macro_impl_items_mention_unqualified_ident(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
    ident: &str,
) -> bool {
    let Some(items) = module_items_for_path(project, package, module_path) else {
        return false;
    };
    let aliases = project
        .module_aliases
        .get(&(package.to_string(), module_path.to_vec()))
        .cloned()
        .unwrap_or_default();

    items.iter().any(|item| {
        let Item::Impl(item_impl) = item else {
            return false;
        };
        if !root_macro_impl_surface_should_render_for_module(
            project,
            reduced,
            package,
            module_path,
            item_impl,
            &aliases,
        ) {
            return false;
        }
        item_impl.items.iter().any(|impl_item| {
            !impl_item_is_test(impl_item)
                && root_macro_impl_item_should_render(item_impl, impl_item)
                && token_stream_mentions_unqualified_ident(&impl_item.to_token_stream(), ident)
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
    let Some(items) = module_items_for_path(project, package, module_path) else {
        return false;
    };

    items.iter().any(|item| match item {
        Item::Macro(item_macro) if item_macro.ident.is_none() => {
            token_stream_mentions_ident(&item_macro.mac.tokens, ident)
        }
        _ => false,
    })
}

fn retained_macro_invocations_mention_unqualified_ident(
    project: &Project,
    _reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
    ident: &str,
) -> bool {
    let Some(items) = module_items_for_path(project, package, module_path) else {
        return false;
    };

    items.iter().any(|item| match item {
        Item::Macro(item_macro) if item_macro.ident.is_none() => {
            token_stream_mentions_unqualified_ident(&item_macro.mac.tokens, ident)
        }
        _ => false,
    })
}

fn collect_retained_module_surface_idents(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
    items: &[Item],
    reachable_token_idents: &ReachableTokenIdentIndex,
    idents: &mut BTreeSet<String>,
) {
    let retained_macro_definitions = retained_macro_definitions_for_generated_items(
        project,
        reduced,
        package,
        module_path,
        items,
    );
    let aliases = project
        .module_aliases
        .get(&(package.to_string(), module_path.to_vec()))
        .cloned()
        .unwrap_or_default();

    for item in items {
        match item {
            Item::ForeignMod(_) => {}
            Item::Impl(item_impl) => {
                let has_reachable_method = impl_has_reachable_method(
                    project,
                    reduced,
                    package,
                    module_path,
                    item_impl,
                    &aliases,
                );
                if has_reachable_method {
                    collect_impl_header_idents(item_impl, idents);
                    for attr in &item_impl.attrs {
                        collect_token_idents(&attr.to_token_stream(), idents);
                    }
                    for impl_item in &item_impl.items {
                        if root_macro_impl_surface_should_render_for_module(
                            project,
                            reduced,
                            package,
                            module_path,
                            item_impl,
                            &aliases,
                        ) && root_macro_impl_item_should_render(item_impl, impl_item)
                        {
                            if !impl_item_is_test(impl_item) {
                                collect_token_idents(&impl_item.to_token_stream(), idents);
                            }
                        } else if !matches!(impl_item, ImplItem::Fn(_))
                            && !impl_item_is_test(impl_item)
                        {
                            collect_token_idents(&impl_item.to_token_stream(), idents);
                        }
                    }
                }
                if item_impl.trait_.is_some()
                    && impl_should_render(
                        project,
                        reduced,
                        package,
                        module_path,
                        item_impl,
                        &aliases,
                    )
                {
                    collect_impl_header_idents(item_impl, idents);
                    for attr in &item_impl.attrs {
                        collect_token_idents(&attr.to_token_stream(), idents);
                    }
                    for impl_item in &item_impl.items {
                        if !impl_item_is_test(impl_item) {
                            collect_token_idents(&impl_item.to_token_stream(), idents);
                        }
                    }
                }
            }
            Item::Macro(item_macro)
                if macro_definition_should_remain(item_macro, &retained_macro_definitions) =>
            {
                collect_token_idents(&item_macro.mac.tokens, idents);
            }
            Item::Macro(item_macro) if item_macro.ident.is_none() => {
                collect_token_idents(&item_macro.mac.tokens, idents);
            }
            _ => {}
        }
    }
    expand_alias_surface_idents(&aliases, idents);

    for item in items {
        let Item::ForeignMod(foreign_mod) = item else {
            continue;
        };
        for foreign_item in &foreign_mod.items {
            if foreign_item_feeds_reachable_code_from_index(
                reachable_token_idents,
                package,
                module_path,
                idents,
                foreign_item,
            ) {
                collect_token_idents(&foreign_item.to_token_stream(), idents);
            }
        }
    }
    expand_alias_surface_idents(&aliases, idents);
}

fn foreign_item_feeds_reachable_code_from_index(
    reachable_token_idents: &ReachableTokenIdentIndex,
    package: &str,
    module_path: &[String],
    module_surface_idents: &BTreeSet<String>,
    foreign_item: &ForeignItem,
) -> bool {
    let Some(name) = foreign_item_name(foreign_item) else {
        return false;
    };
    module_surface_idents.contains(&name)
        || reachable_token_idents.module_mentions_ident(package, module_path, &name)
        || reachable_token_idents.package_mentions_ident(package, &name)
}

fn foreign_item_feeds_reachable_code(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
    foreign_item: &ForeignItem,
) -> bool {
    let Some(name) = foreign_item_name(foreign_item) else {
        return false;
    };
    reachable_module_mentions_ident(project, reduced, package, module_path, &name)
        || reachable_package_mentions_ident(project, reduced, package, &name)
}

fn retained_foreign_items_mention_unqualified_ident(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
    ident: &str,
) -> bool {
    module_items_for_path(project, package, module_path).is_some_and(|items| {
        items.iter().any(|item| {
            let Item::ForeignMod(foreign_mod) = item else {
                return false;
            };
            foreign_mod.items.iter().any(|foreign_item| {
                foreign_item_feeds_reachable_code(
                    project,
                    reduced,
                    package,
                    module_path,
                    foreign_item,
                ) && token_stream_mentions_unqualified_ident(&foreign_item.to_token_stream(), ident)
            })
        })
    })
}

fn retained_foreign_items_mention_unqualified_ident_in_plan(
    project: &Project,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
    ident: &str,
) -> bool {
    module_items_for_path(project, package, module_path).is_some_and(|items| {
        items.iter().any(|item| {
            let Item::ForeignMod(foreign_mod) = item else {
                return false;
            };
            foreign_mod.items.iter().any(|foreign_item| {
                foreign_item_feeds_render_plan(render_plan, package, module_path, foreign_item)
                    && token_stream_mentions_unqualified_ident(
                        &foreign_item.to_token_stream(),
                        ident,
                    )
            })
        })
    })
}

fn foreign_item_feeds_render_plan(
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
    foreign_item: &ForeignItem,
) -> bool {
    let Some(name) = foreign_item_name(foreign_item) else {
        return false;
    };
    render_plan.module_mentions_ident(package, module_path, &name)
        || render_plan.package_mentions_ident(package, &name)
}

fn collect_impl_header_idents(item_impl: &syn::ItemImpl, idents: &mut BTreeSet<String>) {
    collect_token_idents(&item_impl.generics.to_token_stream(), idents);
    collect_token_idents(&item_impl.self_ty.to_token_stream(), idents);
    if let Some((_, trait_path, _)) = &item_impl.trait_ {
        collect_token_idents(&trait_path.to_token_stream(), idents);
    }
}

fn impl_header_mentions_unqualified_ident(item_impl: &syn::ItemImpl, ident: &str) -> bool {
    token_stream_mentions_unqualified_ident(&item_impl.generics.to_token_stream(), ident)
        || token_stream_mentions_unqualified_ident(&item_impl.self_ty.to_token_stream(), ident)
        || item_impl.trait_.as_ref().is_some_and(|(_, trait_path, _)| {
            token_stream_mentions_unqualified_ident(&trait_path.to_token_stream(), ident)
        })
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
    if root_item_impl_surface_should_render(project, reduced, package, &type_path, item_impl) {
        return true;
    }
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

#[allow(clippy::too_many_arguments)]
fn impl_item_should_render_for_trait_surface(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    type_path: &[String],
    trait_path: Option<&[String]>,
    trait_input_type_paths: &[Vec<String>],
    trait_item: Option<&ItemId>,
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
    if trait_item.is_some_and(|trait_item| {
        trait_impl_item_is_required_by_trait(project, trait_item, impl_item)
    }) {
        return true;
    }
    let (Some(trait_path), ImplItem::Fn(method)) = (trait_path, impl_item) else {
        return trait_item.is_none();
    };
    let id = CallableId::Method {
        package: package.to_string(),
        type_path: type_path.to_vec(),
        trait_path: Some(trait_path.to_vec()),
        trait_input_type_paths: trait_input_type_paths.to_vec(),
        method: method.sig.ident.to_string(),
    };
    reduced.reachable.contains(&id)
}

fn trait_item_for_path(project: &Project, package: &str, trait_path: &[String]) -> Option<ItemId> {
    let (name, module_path) = trait_path.split_last()?;
    let item = ItemId {
        package: package.to_string(),
        module_path: module_path.to_vec(),
        name: name.clone(),
        kind: ItemKind::Trait,
    };
    project.items.contains_key(&item).then_some(item)
}

fn trait_default_method_is_referenced_by_reachable_callables(
    project: &Project,
    reduced: &ReducedProject,
    trait_item: &ItemId,
) -> bool {
    let Some(record) = project.items.get(trait_item) else {
        return false;
    };
    let Item::Trait(item_trait) = &record.item else {
        return false;
    };
    item_trait.items.iter().any(|item| {
        let TraitItem::Fn(method) = item else {
            return false;
        };
        method.default.is_some()
            && reachable_reduced_callables_mention_ident(
                project,
                reduced,
                &method.sig.ident.to_string(),
            )
    })
}

fn trait_impl_self_type_is_referenced_by_reachable_surface(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    type_path: &[String],
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
    ) || type_path
        .last()
        .is_some_and(|name| reachable_reduced_callables_mention_ident(project, reduced, name))
}

fn type_tokens_are_referenced_by_reachable_callables(
    project: &Project,
    reduced: &ReducedProject,
    ty: &Type,
) -> bool {
    let mut idents = BTreeSet::new();
    collect_token_idents(&ty.to_token_stream(), &mut idents);
    idents
        .iter()
        .any(|ident| reachable_reduced_callables_mention_ident(project, reduced, ident))
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
            let TraitItem::Fn(function) = trait_item else {
                return false;
            };
            function.sig.ident == method.sig.ident && function.default.is_none()
        }),
        ImplItem::Const(item) => item_trait.items.iter().any(|trait_item| {
            let TraitItem::Const(constant) = trait_item else {
                return false;
            };
            constant.ident == item.ident && constant.default.is_none()
        }),
        ImplItem::Type(item) => item_trait.items.iter().any(|trait_item| {
            let TraitItem::Type(associated_type) = trait_item else {
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

fn root_item_impl_surface_should_render(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    type_path: &[String],
    item_impl: &syn::ItemImpl,
) -> bool {
    item_impl.trait_.is_none()
        && (path_item_is_root(
            reduced,
            package,
            type_path,
            &[
                ItemKind::Struct,
                ItemKind::Enum,
                ItemKind::Union,
                ItemKind::Type,
            ],
        ) || type_path_is_root_callable_signature_surface(project, reduced, package, type_path))
        && impl_surface_has_macro_contract_attrs(item_impl)
}

fn root_macro_impl_surface_should_render_for_module(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
    item_impl: &syn::ItemImpl,
    aliases: &HashMap<String, Vec<String>>,
) -> bool {
    resolved_local_type_path(project, package, module_path, &item_impl.self_ty, aliases)
        .is_some_and(|type_path| {
            root_item_impl_surface_should_render(project, reduced, package, &type_path, item_impl)
        })
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

fn path_item_is_root(
    reduced: &ReducedProject,
    package: &str,
    path: &[String],
    kinds: &[ItemKind],
) -> bool {
    let Some((name, module_path)) = path.split_last() else {
        return false;
    };
    kinds.iter().any(|kind| {
        reduced.roots.iter().any(|root| {
            matches!(
                root,
                RootId::Item(item)
                    if item.package == package
                        && item.module_path == module_path
                        && item.name == *name
                        && item.kind == *kind
            )
        })
    })
}

fn type_path_is_root_callable_signature_surface(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    type_path: &[String],
) -> bool {
    let Some(name) = type_path.last() else {
        return false;
    };
    reduced.roots.iter().any(|root| {
        let RootId::Callable(callable) = root else {
            return false;
        };
        callable.package() == package
            && (project.functions.get(callable).is_some_and(|record| {
                token_stream_mentions_ident(&record.item.sig.to_token_stream(), name)
            }) || project.methods.get(callable).is_some_and(|record| {
                token_stream_mentions_ident(&record.item.sig.to_token_stream(), name)
            }))
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
        || retained_root_macro_impl_items_mention_ident(
            project,
            reduced,
            package,
            module_path,
            ident,
        )
        || retained_macro_invocations_mention_ident(project, reduced, package, module_path, ident)
}

fn reachable_module_mentions_unqualified_ident(
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
                    && token_stream_mentions_unqualified_ident(
                        &record.item.to_token_stream(),
                        ident,
                    )
            }) || project.methods.get(callable).is_some_and(|record| {
                record.module_path == module_path
                    && token_stream_mentions_unqualified_ident(
                        &record.item.to_token_stream(),
                        ident,
                    )
            })
        })
        || reduced
            .reachable_items
            .iter()
            .filter(|item| item.package() == package && item.module_path == module_path)
            .any(|item| {
                project.items.get(item).is_some_and(|record| {
                    reachable_item_mentions_unqualified_ident(
                        project, reduced, package, item, record, ident,
                    )
                })
            })
        || retained_impl_attrs_mention_unqualified_ident(
            project,
            reduced,
            package,
            module_path,
            ident,
        )
        || retained_impl_non_fn_items_mention_unqualified_ident(
            project,
            reduced,
            package,
            module_path,
            ident,
        )
        || retained_root_macro_impl_items_mention_unqualified_ident(
            project,
            reduced,
            package,
            module_path,
            ident,
        )
        || retained_macro_invocations_mention_unqualified_ident(
            project,
            reduced,
            package,
            module_path,
            ident,
        )
        || retained_foreign_items_mention_unqualified_ident(
            project,
            reduced,
            package,
            module_path,
            ident,
        )
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

fn reachable_import_scope_has_method_call(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
    method: &str,
) -> bool {
    if reachable_module_has_method_call(project, reduced, package, module_path, method) {
        return true;
    }
    child_modules_with_super_glob_import(project, reduced, render_plan, package, module_path)
        .into_iter()
        .any(|child_path| {
            reachable_import_scope_has_method_call(
                project,
                reduced,
                render_plan,
                package,
                &child_path,
                method,
            )
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

fn reachable_import_scope_has_associated_function_call(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
    function: &str,
) -> bool {
    if reachable_module_has_associated_function_call(
        project,
        reduced,
        package,
        module_path,
        function,
    ) {
        return true;
    }
    child_modules_with_super_glob_import(project, reduced, render_plan, package, module_path)
        .into_iter()
        .any(|child_path| {
            reachable_import_scope_has_associated_function_call(
                project,
                reduced,
                render_plan,
                package,
                &child_path,
                function,
            )
        })
}

fn retained_impl_surfaces_call_inherent_associated_function(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    type_path: &[String],
    function: &str,
) -> bool {
    let Some(type_name) = type_path.last() else {
        return false;
    };

    project_module_paths(project, package)
        .into_iter()
        .any(|module_path| {
            if !reduced.packages.contains(package) {
                return false;
            }
            let Some(items) = module_items_for_path(project, package, &module_path) else {
                return false;
            };
            let aliases = project
                .module_aliases
                .get(&(package.to_string(), module_path.clone()))
                .cloned()
                .unwrap_or_default();

            items.iter().any(|item| {
                let Item::Impl(item_impl) = item else {
                    return false;
                };
                if !root_macro_impl_surface_should_render_for_module(
                    project,
                    reduced,
                    package,
                    &module_path,
                    item_impl,
                    &aliases,
                ) {
                    return false;
                }
                item_impl.items.iter().any(|impl_item| {
                    let ImplItem::Fn(method) = impl_item else {
                        return false;
                    };
                    !impl_item_is_test(impl_item)
                        && root_macro_impl_item_should_render(item_impl, impl_item)
                        && impl_item_fn_has_associated_function_call_on_type(
                            method, type_name, function,
                        )
                })
            })
        })
}

fn child_modules_with_super_glob_import(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
) -> Vec<Vec<String>> {
    let mut children = Vec::new();
    if let Some(source) = project
        .files
        .values()
        .find(|source| source.package == package && source.module_path == module_path)
    {
        for item in &source.syntax.items {
            let Item::Mod(item_mod) = item else {
                continue;
            };
            let Some((_, child_items)) = &item_mod.content else {
                continue;
            };
            if !items_have_super_glob_import(child_items) {
                continue;
            }
            let mut child_path = module_path.to_vec();
            child_path.push(item_mod.ident.to_string());
            if module_should_render(project, reduced, render_plan, package, &child_path) {
                children.push(child_path);
            }
        }
    }

    children.extend(
        project
            .files
            .values()
            .filter(|source| source.package == package)
            .filter(|source| source.module_path.len() == module_path.len() + 1)
            .filter(|source| path_has_prefix(&source.module_path, module_path))
            .filter(|source| {
                module_should_render(project, reduced, render_plan, package, &source.module_path)
                    && items_have_super_glob_import(&source.syntax.items)
            })
            .map(|source| source.module_path.clone()),
    );
    children.sort();
    children.dedup();
    children
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

fn impl_item_fn_has_associated_function_call_on_type(
    item: &syn::ImplItemFn,
    type_name: &str,
    function: &str,
) -> bool {
    let mut visitor = AssociatedFunctionCallOnTypeVisitor::new(type_name, function);
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
    generic_type_params: BTreeSet<String>,
    found: bool,
}

impl<'a> AssociatedFunctionCallVisitor<'a> {
    fn new(function: &'a str) -> Self {
        Self {
            function,
            generic_type_params: BTreeSet::new(),
            found: false,
        }
    }
}

impl Visit<'_> for AssociatedFunctionCallVisitor<'_> {
    fn visit_generics(&mut self, node: &syn::Generics) {
        self.generic_type_params
            .extend(node.type_params().map(|param| param.ident.to_string()));
        visit::visit_generics(self, node);
    }

    fn visit_expr_call(&mut self, node: &syn::ExprCall) {
        if let syn::Expr::Path(path) = node.func.as_ref() {
            if path.path.segments.len() > 1
                && path
                    .path
                    .segments
                    .last()
                    .is_some_and(|segment| segment.ident == self.function)
                && !associated_function_call_receiver_is_generic_param(
                    &path.path,
                    &self.generic_type_params,
                )
            {
                self.found = true;
                return;
            }
        }
        visit::visit_expr_call(self, node);
    }
}

fn associated_function_call_receiver_is_generic_param(
    path: &syn::Path,
    generic_type_params: &BTreeSet<String>,
) -> bool {
    if path.segments.len() != 2 {
        return false;
    }
    path.segments
        .first()
        .is_some_and(|segment| generic_type_params.contains(&segment.ident.to_string()))
}

struct AssociatedFunctionCallOnTypeVisitor<'a> {
    type_name: &'a str,
    function: &'a str,
    found: bool,
}

impl<'a> AssociatedFunctionCallOnTypeVisitor<'a> {
    fn new(type_name: &'a str, function: &'a str) -> Self {
        Self {
            type_name,
            function,
            found: false,
        }
    }
}

impl Visit<'_> for AssociatedFunctionCallOnTypeVisitor<'_> {
    fn visit_expr_call(&mut self, node: &syn::ExprCall) {
        if let syn::Expr::Path(path) = node.func.as_ref() {
            let segments = path.path.segments.iter().collect::<Vec<_>>();
            if segments.len() > 1
                && segments
                    .last()
                    .is_some_and(|segment| segment.ident == self.function)
                && segments[..segments.len() - 1]
                    .iter()
                    .any(|segment| segment.ident == self.type_name)
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
        if root_item_should_render(reduced, item_id) {
            return token_stream_mentions_ident(&record.item.to_token_stream(), ident);
        }
        return trait_type_surface_mentions_ident(
            project, reduced, package, item_id, item_trait, ident,
        );
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
    if token_stream_mentions_ident(&item_struct.generics.to_token_stream(), ident) {
        return true;
    }
    if root_item_should_render(reduced, item_id) {
        return token_stream_mentions_ident(&record.item.to_token_stream(), ident);
    }

    let syn::Fields::Named(fields) = &item_struct.fields else {
        return token_stream_mentions_ident(&record.item.to_token_stream(), ident);
    };
    fields.named.iter().any(|field| {
        struct_field_should_remain(
            project,
            reduced,
            None,
            package,
            &item_id.module_path,
            item_struct,
            field,
        ) && token_stream_mentions_ident(&field.to_token_stream(), ident)
    })
}

fn reachable_item_mentions_unqualified_ident(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    item_id: &ItemId,
    record: &crate::model::ItemRecord,
    ident: &str,
) -> bool {
    if let Item::Trait(item_trait) = &record.item {
        if root_item_should_render(reduced, item_id) {
            return token_stream_mentions_unqualified_ident(&record.item.to_token_stream(), ident);
        }
        return trait_type_surface_mentions_unqualified_ident(
            project, reduced, package, item_id, item_trait, ident,
        );
    }

    let Item::Struct(item_struct) = &record.item else {
        return token_stream_mentions_unqualified_ident(&record.item.to_token_stream(), ident);
    };
    if item_id.name == ident || item_id.module_path.iter().any(|segment| segment == ident) {
        return true;
    }
    if item_struct
        .attrs
        .iter()
        .any(|attr| token_stream_mentions_unqualified_ident(&attr.to_token_stream(), ident))
    {
        return true;
    }
    if token_stream_mentions_unqualified_ident(&item_struct.generics.to_token_stream(), ident) {
        return true;
    }
    if root_item_should_render(reduced, item_id) {
        return token_stream_mentions_unqualified_ident(&record.item.to_token_stream(), ident);
    }

    let syn::Fields::Named(fields) = &item_struct.fields else {
        return token_stream_mentions_unqualified_ident(&record.item.to_token_stream(), ident);
    };
    fields.named.iter().any(|field| {
        struct_field_should_remain(
            project,
            reduced,
            None,
            package,
            &item_id.module_path,
            item_struct,
            field,
        ) && token_stream_mentions_unqualified_ident(&field.to_token_stream(), ident)
    })
}

fn trait_type_surface_mentions_ident(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    item_id: &ItemId,
    item_trait: &syn::ItemTrait,
    ident: &str,
) -> bool {
    if item_id.name == ident || item_id.module_path.iter().any(|segment| segment == ident) {
        return true;
    }
    if token_stream_mentions_ident(&item_trait.generics.to_token_stream(), ident) {
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
        || item_trait.items.iter().any(|item| {
            trait_item_should_remain_for_type_surface(
                project,
                reduced,
                package,
                &item_id.module_path,
                &item_id.name,
                item,
            ) && token_stream_mentions_ident(&item.to_token_stream(), ident)
        })
}

fn trait_type_surface_mentions_unqualified_ident(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    item_id: &ItemId,
    item_trait: &syn::ItemTrait,
    ident: &str,
) -> bool {
    if item_id.name == ident || item_id.module_path.iter().any(|segment| segment == ident) {
        return true;
    }
    if token_stream_mentions_unqualified_ident(&item_trait.generics.to_token_stream(), ident) {
        return true;
    }
    if item_trait
        .supertraits
        .iter()
        .any(|bound| token_stream_mentions_unqualified_ident(&bound.to_token_stream(), ident))
    {
        return true;
    }
    item_trait
        .attrs
        .iter()
        .filter(|attr| is_inert_type_surface_attr(attr))
        .any(|attr| token_stream_mentions_unqualified_ident(&attr.to_token_stream(), ident))
        || item_trait.items.iter().any(|item| {
            trait_item_should_remain_for_type_surface(
                project,
                reduced,
                package,
                &item_id.module_path,
                &item_id.name,
                item,
            ) && token_stream_mentions_unqualified_ident(&item.to_token_stream(), ident)
        })
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

fn reachable_module_import_scope_mentions_unqualified_ident(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
    ident: &str,
) -> bool {
    if reachable_module_mentions_unqualified_ident(project, reduced, package, module_path, ident) {
        return true;
    }

    if project
        .files
        .values()
        .find(|source| source.package == package && source.module_path == module_path)
        .is_some_and(|source| {
            inline_child_modules_import_scope_uses_imported_ident(
                project,
                reduced,
                render_plan,
                package,
                module_path,
                &source.syntax.items,
                ident,
            )
        })
    {
        return true;
    }

    if project
        .files
        .values()
        .filter(|source| source.package == package)
        .filter(|source| source.module_path.len() == module_path.len() + 1)
        .filter(|source| path_has_prefix(&source.module_path, module_path))
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
                None,
            )
        })
    {
        return true;
    }

    child_modules_with_super_glob_import(project, reduced, render_plan, package, module_path)
        .into_iter()
        .any(|child_path| {
            reachable_module_import_scope_mentions_unqualified_ident(
                project,
                reduced,
                render_plan,
                package,
                &child_path,
                ident,
            )
        })
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

fn enum_preserves_full_variant_surface(
    reduced: &ReducedProject,
    item_id: &ItemId,
    item_enum: &syn::ItemEnum,
) -> bool {
    matches!(item_enum.vis, syn::Visibility::Public(_))
        || root_item_should_render(reduced, item_id)
        || enum_attrs_require_full_variant_surface(item_enum)
}

fn enum_attrs_require_full_variant_surface(item_enum: &syn::ItemEnum) -> bool {
    item_enum
        .attrs
        .iter()
        .any(enum_attr_requires_full_variant_surface)
        || item_enum.variants.iter().any(|variant| {
            variant
                .attrs
                .iter()
                .any(enum_attr_requires_full_variant_surface)
                || variant.fields.iter().any(|field| {
                    field
                        .attrs
                        .iter()
                        .any(enum_attr_requires_full_variant_surface)
                })
        })
}

fn enum_attr_requires_full_variant_surface(attr: &syn::Attribute) -> bool {
    let path = attr.path();
    path.is_ident("derive")
        || path.is_ident("serde")
        || path.is_ident("clap")
        || path.is_ident("command")
        || path.is_ident("arg")
        || path.is_ident("error")
        || path.is_ident("repr")
        || path.is_ident("non_exhaustive")
        || path_starts_with(path, "uniffi")
        || (path.is_ident("cfg_attr")
            && (token_stream_mentions_ident(&attr.to_token_stream(), "derive")
                || token_stream_mentions_ident(&attr.to_token_stream(), "serde")
                || token_stream_mentions_ident(&attr.to_token_stream(), "uniffi")
                || token_stream_mentions_ident(&attr.to_token_stream(), "clap")
                || token_stream_mentions_ident(&attr.to_token_stream(), "error")))
}

fn prune_private_enum_variants(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    _module_path: &[String],
    item_enum: &mut syn::ItemEnum,
) {
    item_enum.variants = item_enum
        .variants
        .iter()
        .filter(|variant| {
            enum_variant_should_remain(project, reduced, package, &variant.ident.to_string())
        })
        .cloned()
        .collect();
}

fn enum_variant_should_remain(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    variant_name: &str,
) -> bool {
    reachable_callables_mention_ident(project, reduced, package, variant_name)
}

fn prune_private_struct_fields(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
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
                module_path,
                &original_struct,
                field,
            )
        })
        .cloned()
        .collect();
}

fn prune_rendered_struct_literal_fields(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
    current_type_path: Option<&[String]>,
    block: &mut Block,
) {
    let aliases = project
        .module_aliases
        .get(&(package.to_string(), module_path.to_vec()))
        .cloned()
        .unwrap_or_default();
    let mut pruner = StructLiteralFieldPruner {
        project,
        reduced,
        render_plan,
        package,
        module_path,
        aliases,
        current_type_path: current_type_path.map(<[String]>::to_vec),
    };
    pruner.visit_block_mut(block);
}

struct StructLiteralFieldPruner<'a> {
    project: &'a Project,
    reduced: &'a ReducedProject,
    render_plan: &'a RenderPlan,
    package: &'a str,
    module_path: &'a [String],
    aliases: std::collections::HashMap<String, Vec<String>>,
    current_type_path: Option<Vec<String>>,
}

impl VisitMut for StructLiteralFieldPruner<'_> {
    fn visit_expr_struct_mut(&mut self, expr: &mut ExprStruct) {
        for field in &mut expr.fields {
            visit_mut::visit_expr_mut(self, &mut field.expr);
        }
        if let Some(rest) = &mut expr.rest {
            visit_mut::visit_expr_mut(self, rest);
        }

        let Some((item_id, item_struct)) = self.resolve_struct_literal_item(expr) else {
            return;
        };
        if root_item_should_render(self.reduced, &item_id) {
            return;
        }
        let struct_module_path = item_id.module_path.clone();

        expr.fields = expr
            .fields
            .iter()
            .filter(|field| {
                self.struct_literal_field_should_remain(&struct_module_path, item_struct, field)
            })
            .cloned()
            .collect();
    }
}

impl<'a> StructLiteralFieldPruner<'a> {
    fn resolve_struct_literal_item(
        &self,
        expr: &ExprStruct,
    ) -> Option<(ItemId, &'a syn::ItemStruct)> {
        let type_path = if path_is_self(&expr.path) {
            self.current_type_path.clone()?
        } else {
            let path = &expr.path;
            let ty: Type = parse_quote!(#path);
            resolved_local_type_path(
                self.project,
                self.package,
                self.module_path,
                &ty,
                &self.aliases,
            )?
        };
        let (name, module_path) = type_path.split_last()?;
        let item_id = ItemId {
            package: self.package.to_string(),
            module_path: module_path.to_vec(),
            name: name.clone(),
            kind: ItemKind::Struct,
        };
        let record = self.project.items.get(&item_id)?;
        let Item::Struct(item_struct) = &record.item else {
            return None;
        };
        Some((item_id, item_struct))
    }

    fn struct_literal_field_should_remain(
        &self,
        struct_module_path: &[String],
        item_struct: &syn::ItemStruct,
        field_value: &FieldValue,
    ) -> bool {
        let Some(field_name) = member_name(&field_value.member) else {
            return true;
        };
        let Some(field) = named_struct_field(item_struct, &field_name) else {
            return true;
        };
        if !struct_literal_initializer_can_be_pruned_with_aliases(
            &field_value.expr,
            Some(&self.aliases),
        ) {
            return true;
        }
        struct_field_should_remain(
            self.project,
            self.reduced,
            Some(self.render_plan),
            self.package,
            struct_module_path,
            item_struct,
            field,
        )
    }
}

fn struct_field_should_remain(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: Option<&RenderPlan>,
    package: &str,
    module_path: &[String],
    item_struct: &syn::ItemStruct,
    field: &Field,
) -> bool {
    if matches!(item_struct.vis, syn::Visibility::Public(_))
        && matches!(field.vis, syn::Visibility::Public(_))
    {
        return true;
    }
    if field_mentions_struct_type_params(item_struct, field)
        && field_provides_required_struct_type_param_usage(
            project,
            reduced,
            render_plan,
            package,
            module_path,
            item_struct,
            field,
        )
    {
        return true;
    }
    if field_attrs_require_field(field) {
        return true;
    }
    if struct_attrs_require_field_surface(item_struct)
        || (struct_type_is_referenced_by_reachable_signature(
            project,
            reduced,
            package,
            module_path,
            item_struct,
        ) && field_type_requires_signature_surface_retention(
            project,
            package,
            module_path,
            field,
        ))
    {
        return true;
    }
    let Some(name) = field.ident.as_ref() else {
        return true;
    };
    let name = name.to_string();
    reachable_callables_need_struct_field(project, reduced, render_plan, package, &name)
        || retained_impl_items_mention_struct_field(
            project,
            reduced,
            render_plan,
            package,
            module_path,
            item_struct,
            &name,
        )
}

fn field_provides_required_struct_type_param_usage(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: Option<&RenderPlan>,
    package: &str,
    module_path: &[String],
    item_struct: &syn::ItemStruct,
    field: &Field,
) -> bool {
    item_struct.generics.type_params().any(|param| {
        token_stream_mentions_ident(&field.to_token_stream(), &param.ident.to_string())
            && !struct_type_param_is_used_by_other_retained_field(
                project,
                reduced,
                render_plan,
                package,
                module_path,
                item_struct,
                field,
                &param.ident.to_string(),
            )
    })
}

#[allow(clippy::too_many_arguments)]
fn struct_type_param_is_used_by_other_retained_field(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: Option<&RenderPlan>,
    package: &str,
    module_path: &[String],
    item_struct: &syn::ItemStruct,
    field: &Field,
    type_param: &str,
) -> bool {
    let syn::Fields::Named(fields) = &item_struct.fields else {
        return false;
    };
    fields.named.iter().any(|candidate| {
        candidate.ident != field.ident
            && struct_field_should_remain_without_type_param_guard(
                project,
                reduced,
                render_plan,
                package,
                module_path,
                item_struct,
                candidate,
            )
            && token_stream_mentions_ident(&candidate.to_token_stream(), type_param)
    })
}

fn struct_field_should_remain_without_type_param_guard(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: Option<&RenderPlan>,
    package: &str,
    module_path: &[String],
    item_struct: &syn::ItemStruct,
    field: &Field,
) -> bool {
    if matches!(item_struct.vis, syn::Visibility::Public(_))
        && matches!(field.vis, syn::Visibility::Public(_))
    {
        return true;
    }
    if field_attrs_require_field(field) {
        return true;
    }
    if struct_attrs_require_field_surface(item_struct)
        || (struct_type_is_referenced_by_reachable_signature(
            project,
            reduced,
            package,
            module_path,
            item_struct,
        ) && field_type_requires_signature_surface_retention(
            project,
            package,
            module_path,
            field,
        ))
    {
        return true;
    }
    let Some(name) = field.ident.as_ref() else {
        return true;
    };
    let name = name.to_string();
    reachable_callables_need_struct_field(project, reduced, render_plan, package, &name)
        || retained_impl_items_mention_struct_field(
            project,
            reduced,
            render_plan,
            package,
            module_path,
            item_struct,
            &name,
        )
}

fn retained_impl_items_mention_struct_field(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: Option<&RenderPlan>,
    package: &str,
    module_path: &[String],
    item_struct: &syn::ItemStruct,
    field_name: &str,
) -> bool {
    let Some(items) = module_items_for_path(project, package, module_path) else {
        return false;
    };
    let aliases = project
        .module_aliases
        .get(&(package.to_string(), module_path.to_vec()))
        .cloned()
        .unwrap_or_default();
    let struct_name = item_struct.ident.to_string();

    items.iter().any(|item| {
        let Item::Impl(item_impl) = item else {
            return false;
        };
        let Some(type_path) =
            resolved_local_type_path(project, package, module_path, &item_impl.self_ty, &aliases)
        else {
            return false;
        };
        if type_path.last() != Some(&struct_name) {
            return false;
        }
        item_impl.items.iter().any(|impl_item| {
            impl_item_should_render_for_field_scan(
                project,
                reduced,
                render_plan,
                package,
                module_path,
                item_impl,
                impl_item,
                &type_path,
                &aliases,
            ) && impl_item_needs_struct_field(
                impl_item,
                field_name,
                item_impl.trait_.is_some(),
                &aliases,
            )
        })
    })
}

fn reachable_callables_need_struct_field(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: Option<&RenderPlan>,
    package: &str,
    field_name: &str,
) -> bool {
    reduced
        .reachable
        .iter()
        .filter(|callable| callable.package() == package)
        .filter(|callable| {
            render_plan.is_none_or(|render_plan| render_plan.callable_should_render(callable))
        })
        .any(|callable| {
            project.functions.get(callable).is_some_and(|record| {
                function_needs_struct_field(&record.item, &record.aliases, field_name)
            }) || project.methods.get(callable).is_some_and(|record| {
                method_needs_struct_field(&record.item, &record.aliases, field_name)
            })
        })
}

fn function_needs_struct_field(
    function: &syn::ItemFn,
    aliases: &HashMap<String, Vec<String>>,
    field_name: &str,
) -> bool {
    block_needs_struct_field(&function.block, aliases, field_name)
}

fn method_needs_struct_field(
    method: &syn::ImplItemFn,
    aliases: &HashMap<String, Vec<String>>,
    field_name: &str,
) -> bool {
    block_needs_struct_field(&method.block, aliases, field_name)
}

fn impl_item_needs_struct_field(
    impl_item: &ImplItem,
    field_name: &str,
    count_pure_struct_initializers: bool,
    aliases: &HashMap<String, Vec<String>>,
) -> bool {
    let ImplItem::Fn(method) = impl_item else {
        return token_stream_mentions_ident(&impl_item.to_token_stream(), field_name);
    };
    method_needs_struct_field_with_options(
        method,
        aliases,
        field_name,
        count_pure_struct_initializers,
    )
}

fn block_needs_struct_field(
    block: &Block,
    aliases: &HashMap<String, Vec<String>>,
    field_name: &str,
) -> bool {
    block_needs_struct_field_with_options(block, aliases, field_name, false)
}

fn method_needs_struct_field_with_options(
    method: &syn::ImplItemFn,
    aliases: &HashMap<String, Vec<String>>,
    field_name: &str,
    count_pure_struct_initializers: bool,
) -> bool {
    block_needs_struct_field_with_options(
        &method.block,
        aliases,
        field_name,
        count_pure_struct_initializers,
    )
}

fn block_needs_struct_field_with_options(
    block: &Block,
    aliases: &HashMap<String, Vec<String>>,
    field_name: &str,
    count_pure_struct_initializers: bool,
) -> bool {
    let mut visitor = StructFieldUseVisitor {
        field_name,
        aliases,
        count_pure_struct_initializers,
        needs_field: false,
    };
    visitor.visit_block(block);
    visitor.needs_field
}

struct StructFieldUseVisitor<'a> {
    field_name: &'a str,
    aliases: &'a HashMap<String, Vec<String>>,
    count_pure_struct_initializers: bool,
    needs_field: bool,
}

impl Visit<'_> for StructFieldUseVisitor<'_> {
    fn visit_expr_field(&mut self, field: &syn::ExprField) {
        if member_name(&field.member).as_deref() == Some(self.field_name) {
            self.needs_field = true;
        }
        visit::visit_expr_field(self, field);
    }

    fn visit_pat_struct(&mut self, pattern: &syn::PatStruct) {
        if pattern
            .fields
            .iter()
            .any(|field| member_name(&field.member).as_deref() == Some(self.field_name))
        {
            self.needs_field = true;
        }
        visit::visit_pat_struct(self, pattern);
    }

    fn visit_expr_struct(&mut self, expr: &ExprStruct) {
        for field in &expr.fields {
            if member_name(&field.member).as_deref() == Some(self.field_name)
                && (self.count_pure_struct_initializers
                    || !struct_literal_initializer_can_be_pruned_with_aliases(
                        &field.expr,
                        Some(self.aliases),
                    ))
            {
                self.needs_field = true;
            }
            self.visit_expr(&field.expr);
        }
        if let Some(rest) = &expr.rest {
            self.visit_expr(rest);
        }
    }

    fn visit_macro(&mut self, mac: &syn::Macro) {
        if token_stream_mentions_ident(&mac.tokens, self.field_name) {
            self.needs_field = true;
        }
        visit::visit_macro(self, mac);
    }
}

fn named_struct_field<'a>(item_struct: &'a syn::ItemStruct, name: &str) -> Option<&'a Field> {
    let Fields::Named(fields) = &item_struct.fields else {
        return None;
    };
    fields
        .named
        .iter()
        .find(|field| field.ident.as_ref().is_some_and(|ident| ident == name))
}

fn member_name(member: &Member) -> Option<String> {
    match member {
        Member::Named(ident) => Some(ident.to_string()),
        Member::Unnamed(_) => None,
    }
}

fn path_is_self(path: &syn::Path) -> bool {
    path.segments.len() == 1 && path.segments[0].ident == "Self"
}

fn struct_literal_initializer_can_be_pruned_with_aliases(
    expr: &Expr,
    aliases: Option<&HashMap<String, Vec<String>>>,
) -> bool {
    match expr {
        Expr::Path(path) => path.qself.is_none() && path.path.is_ident("None"),
        Expr::Lit(_) => true,
        Expr::Array(array) => array.elems.is_empty(),
        Expr::Tuple(tuple) => tuple.elems.is_empty(),
        Expr::Call(call) => {
            let Expr::Path(function) = call.func.as_ref() else {
                return false;
            };
            function.qself.is_none()
                && pure_std_struct_literal_initializer_path(&function.path, aliases)
                && call
                    .args
                    .iter()
                    .all(|arg| struct_literal_initializer_can_be_pruned_with_aliases(arg, aliases))
        }
        _ => false,
    }
}

fn pure_std_struct_literal_initializer_path(
    path: &syn::Path,
    aliases: Option<&HashMap<String, Vec<String>>>,
) -> bool {
    let segments = path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>();
    let segments = aliases
        .map(|aliases| apply_alias(segments.clone(), aliases))
        .unwrap_or(segments);
    let Some(last) = segments.last() else {
        return false;
    };
    (path_segments_match(&segments, &["std", "borrow", "Cow"])
        || path_segments_match(&segments, &["alloc", "borrow", "Cow"]))
        && matches!(last.as_str(), "Borrowed" | "Owned")
        || (path_segments_match(&segments, &["std", "time", "Duration"])
            || path_segments_match(&segments, &["core", "time", "Duration"]))
            && matches!(
                last.as_str(),
                "from_secs" | "from_millis" | "from_micros" | "from_nanos"
            )
}

fn path_segments_match(segments: &[String], prefix: &[&str]) -> bool {
    segments.len() == prefix.len() + 1
        && segments
            .iter()
            .zip(prefix.iter())
            .all(|(left, right)| left == right)
}

#[allow(clippy::too_many_arguments)]
fn impl_item_should_render_for_field_scan(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: Option<&RenderPlan>,
    package: &str,
    module_path: &[String],
    item_impl: &syn::ItemImpl,
    impl_item: &ImplItem,
    type_path: &[String],
    aliases: &std::collections::HashMap<String, Vec<String>>,
) -> bool {
    if impl_item_is_test(impl_item) {
        return false;
    }
    let trait_path = item_impl
        .trait_
        .as_ref()
        .map(|(_, path, _)| normalized_path(module_path, path, aliases));
    if trait_path.is_some()
        && impl_should_render(project, reduced, package, module_path, item_impl, aliases)
    {
        return true;
    }
    if root_macro_impl_surface_should_render_for_module(
        project,
        reduced,
        package,
        module_path,
        item_impl,
        aliases,
    ) {
        return root_macro_impl_item_should_render(item_impl, impl_item);
    }
    if let ImplItem::Fn(method) = impl_item {
        let trait_input_type_paths = item_impl
            .trait_
            .as_ref()
            .map(|(_, path, _)| trait_input_type_paths(module_path, path, aliases))
            .unwrap_or_default();
        let id = CallableId::Method {
            package: package.to_string(),
            type_path: type_path.to_vec(),
            trait_path,
            trait_input_type_paths,
            method: method.sig.ident.to_string(),
        };
        if render_plan.is_some_and(|render_plan| render_plan.callable_should_render(&id)) {
            return true;
        }
        return reduced.reachable.contains(&id)
            || retained_impl_surfaces_call_inherent_associated_function(
                project,
                reduced,
                package,
                type_path,
                &method.sig.ident.to_string(),
            );
    }
    impl_has_reachable_method(project, reduced, package, module_path, item_impl, aliases)
}

fn field_attrs_require_field(field: &Field) -> bool {
    field.attrs.iter().any(|attr| {
        let path = attr.path();
        if path.is_ident("cfg") {
            return !is_cfg_test_attr(attr);
        }
        if path.is_ident("derive") {
            return false;
        }
        !(path.is_ident("allow")
            || path.is_ident("deny")
            || path.is_ident("doc")
            || path.is_ident("deprecated"))
    })
}

fn struct_attrs_require_field_surface(item_struct: &syn::ItemStruct) -> bool {
    item_struct.attrs.iter().any(|attr| {
        let path = attr.path();
        if path.is_ident("cfg") {
            return !is_cfg_test_attr(attr);
        }
        if path.is_ident("derive") {
            return false;
        }
        !(path.is_ident("allow")
            || path.is_ident("deny")
            || path.is_ident("doc")
            || path.is_ident("deprecated"))
    })
}

fn struct_type_is_referenced_by_reachable_signature(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    module_path: &[String],
    item_struct: &syn::ItemStruct,
) -> bool {
    if !matches!(item_struct.vis, syn::Visibility::Public(_)) {
        return false;
    }
    let struct_name = item_struct.ident.to_string();
    reduced
        .reachable
        .iter()
        .filter(|callable| callable.package() == package)
        .any(|callable| {
            project.functions.get(callable).is_some_and(|record| {
                record.module_path == module_path
                    && signature_returns_ident(&record.item.sig, &struct_name)
            }) || project.methods.get(callable).is_some_and(|record| {
                record.module_path == module_path
                    && signature_returns_ident(&record.item.sig, &struct_name)
            })
        })
}

fn signature_returns_ident(signature: &syn::Signature, ident: &str) -> bool {
    let syn::ReturnType::Type(_, ty) = &signature.output else {
        return false;
    };
    token_stream_mentions_ident(&ty.to_token_stream(), ident)
}

fn field_type_requires_signature_surface_retention(
    project: &Project,
    package: &str,
    module_path: &[String],
    field: &Field,
) -> bool {
    type_contains_dyn_trait(&field.ty)
        || field_type_mentions_bounded_local_generic(project, package, module_path, &field.ty)
}

fn type_contains_dyn_trait(ty: &Type) -> bool {
    match ty {
        Type::TraitObject(_) => true,
        Type::Array(array) => type_contains_dyn_trait(&array.elem),
        Type::Group(group) => type_contains_dyn_trait(&group.elem),
        Type::Paren(paren) => type_contains_dyn_trait(&paren.elem),
        Type::Path(type_path) => type_path.path.segments.iter().any(|segment| {
            if let PathArguments::AngleBracketed(arguments) = &segment.arguments {
                arguments.args.iter().any(|argument| {
                    matches!(argument, GenericArgument::Type(ty) if type_contains_dyn_trait(ty))
                })
            } else {
                false
            }
        }),
        Type::Ptr(ptr) => type_contains_dyn_trait(&ptr.elem),
        Type::Reference(reference) => type_contains_dyn_trait(&reference.elem),
        Type::Slice(slice) => type_contains_dyn_trait(&slice.elem),
        Type::Tuple(tuple) => tuple.elems.iter().any(type_contains_dyn_trait),
        _ => false,
    }
}

fn field_type_mentions_bounded_local_generic(
    project: &Project,
    package: &str,
    module_path: &[String],
    ty: &Type,
) -> bool {
    let aliases = project
        .module_aliases
        .get(&(package.to_string(), module_path.to_vec()))
        .cloned()
        .unwrap_or_default();
    let mut type_paths = Vec::new();
    collect_type_paths(module_path, ty, &aliases, &mut type_paths);
    type_paths.into_iter().any(|path| {
        let canonical = canonical_type_path(project, package, module_path, path);
        let Some(item_id) = find_type_like_item(project, package, &canonical) else {
            return false;
        };
        let Some(record) = project.items.get(&item_id) else {
            return false;
        };
        let Item::Struct(item_struct) = &record.item else {
            return false;
        };
        item_struct
            .generics
            .type_params()
            .any(|param| !param.bounds.is_empty())
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

fn use_is_public_api_reexport(vis: &syn::Visibility) -> bool {
    matches!(vis, syn::Visibility::Public(_))
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

fn local_macro_self_reexport_should_prune(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
    items: &[Item],
    item_use: &syn::ItemUse,
) -> bool {
    if use_is_public_api_reexport(&item_use.vis) {
        return false;
    }
    let Some(name) = single_use_name(&item_use.tree) else {
        return false;
    };
    let macro_is_local = items.iter().any(|item| {
        matches!(
            item,
            Item::Macro(item_macro)
                if item_macro
                    .ident
                    .as_ref()
                    .is_some_and(|ident| ident == name)
        )
    });
    macro_is_local
        && !local_macro_self_reexport_is_used_by_rendered_module(
            project,
            reduced,
            render_plan,
            package,
            module_path,
            &name.to_string(),
        )
}

fn local_macro_self_reexport_should_remain(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
    items: &[Item],
    item_use: &syn::ItemUse,
) -> bool {
    if use_is_public_api_reexport(&item_use.vis) {
        return false;
    }
    let Some(name) = single_use_name(&item_use.tree) else {
        return false;
    };
    let macro_is_local = items.iter().any(|item| {
        matches!(
            item,
            Item::Macro(item_macro)
                if item_macro
                    .ident
                    .as_ref()
                    .is_some_and(|ident| ident == name)
        )
    });
    macro_is_local
        && local_macro_self_reexport_is_used_by_rendered_module(
            project,
            reduced,
            render_plan,
            package,
            module_path,
            &name.to_string(),
        )
}

fn local_macro_self_reexport_is_used_by_rendered_module(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
    name: &str,
) -> bool {
    reduced
        .reachable
        .iter()
        .filter(|callable| callable.package() == package)
        .any(|callable| {
            project.functions.get(callable).is_some_and(|record| {
                token_stream_mentions_module_path_ident(
                    &record.item.to_token_stream(),
                    module_path,
                    name,
                )
            }) || project.methods.get(callable).is_some_and(|record| {
                token_stream_mentions_module_path_ident(
                    &record.item.to_token_stream(),
                    module_path,
                    name,
                )
            })
        })
        || project_module_paths(project, package)
            .into_iter()
            .any(|candidate| {
                candidate != module_path
                    && module_should_render(project, reduced, render_plan, package, &candidate)
                    && module_items_for_path(project, package, &candidate).is_some_and(|items| {
                        items.iter().any(|item| {
                            let Item::Use(item_use) = item else {
                                return false;
                            };
                            use_tree_mentions_module_path_ident(
                                &item_use.tree,
                                Vec::new(),
                                module_path,
                                name,
                            )
                        })
                    })
            })
}

fn single_use_name(tree: &UseTree) -> Option<&syn::Ident> {
    match tree {
        UseTree::Name(name) => Some(&name.ident),
        _ => None,
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
                && !pruned_local_proc_macro_dependency_alias_should_remain(
                    project,
                    package,
                    &path.ident.to_string(),
                )
                && !pruned_proc_macro_helper_dependency_alias_should_remain(
                    project,
                    reduced,
                    package,
                    &path.ident.to_string(),
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
            (!use_target_resolves_to_removed_symbol(
                project,
                reduced,
                render_plan,
                package,
                module_path,
                &prefix,
            ) && (!use_target_should_drop(
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
                ))))
            .then(|| UseTree::Name(name.clone()))
        }
        UseTree::Rename(rename) => {
            prefix.push(rename.ident.to_string());
            let alias = rename.rename.to_string();
            let alias_is_reachable = renamed_use_alias_is_reachable(
                project,
                reduced,
                render_plan,
                package,
                module_path,
                &alias,
                is_public_use,
            );
            if !is_public_use
                && !alias_is_reachable
                && !renamed_use_target_has_implicit_scope_effect(
                    project,
                    reduced,
                    render_plan,
                    package,
                    module_path,
                    &prefix,
                )
            {
                return None;
            }
            (!use_target_resolves_to_removed_symbol(
                project,
                reduced,
                render_plan,
                package,
                module_path,
                &prefix,
            ) && (!use_target_should_drop(
                project,
                reduced,
                render_plan,
                package,
                module_path,
                &prefix,
                is_public_use,
            ) || alias_is_reachable
                || (is_public_use
                    && (reachable_reduced_packages_mention_ident(project, reduced, &alias)
                        || public_reexport_name_is_referenced_by_reduced_package(
                            project, reduced, package, &alias,
                        )))))
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

fn renamed_use_target_has_implicit_scope_effect(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
    target: &[String],
) -> bool {
    if target
        .first()
        .is_some_and(|first| use_name_is_external_dependency(project, package, first))
        || target
            .first()
            .is_some_and(|first| matches!(first.as_str(), "std" | "core" | "alloc"))
    {
        return external_trait_import_should_remain(
            project,
            reduced,
            render_plan,
            package,
            module_path,
            target,
            false,
        );
    }

    let Some((target_package, target_path)) =
        resolve_use_target_path(project, package, module_path, target)
    else {
        return false;
    };
    find_use_item(project, &target_package, &target_path).is_some_and(|item| {
        item.kind == ItemKind::Trait
            && local_trait_import_should_remain(
                project,
                reduced,
                render_plan,
                package,
                module_path,
                &item,
            )
    })
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
    if reachable_module_import_scope_uses_imported_ident(
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

fn use_target_resolves_to_removed_symbol(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
    target: &[String],
) -> bool {
    if pruned_local_proc_macro_dependency_target_should_remain(
        project,
        reduced,
        render_plan,
        package,
        module_path,
        target,
    ) || pruned_proc_macro_helper_dependency_target_should_remain(
        project,
        reduced,
        render_plan,
        package,
        module_path,
        target,
    ) {
        return false;
    }

    if target
        .first()
        .is_some_and(|first| use_name_is_external_dependency(project, package, first))
        || target
            .first()
            .is_some_and(|first| matches!(first.as_str(), "std" | "core" | "alloc"))
    {
        return false;
    }

    let Some((target_package, target_path)) =
        resolve_use_target_path(project, package, module_path, target)
    else {
        return false;
    };

    if let Some(item) = find_use_item(project, &target_package, &target_path) {
        if item.kind == ItemKind::Trait
            && local_trait_import_should_remain(
                project,
                reduced,
                render_plan,
                package,
                module_path,
                &item,
            )
        {
            return false;
        }
    }

    local_use_target_resolves_to_removed_symbol(
        project,
        reduced,
        render_plan,
        &target_package,
        &target_path,
    ) || resolve_reexported_use_path(project, &target_package, &target_path).is_some_and(
        |(alias_package, alias_path)| {
            local_use_target_resolves_to_removed_symbol(
                project,
                reduced,
                render_plan,
                &alias_package,
                &alias_path,
            )
        },
    )
}

fn local_use_target_resolves_to_removed_symbol(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    target_package: &str,
    target_path: &[String],
) -> bool {
    if let Some(callable) = find_use_function(project, target_package, target_path) {
        return render_plan.can_remove_callable(&callable);
    }
    if let Some(item) = find_use_item(project, target_package, target_path) {
        if item.kind == ItemKind::Mod {
            let mut module_path = item.module_path.clone();
            module_path.push(item.name.clone());
            return render_plan.can_remove_item(&item)
                && !module_should_render(
                    project,
                    reduced,
                    render_plan,
                    &item.package,
                    &module_path,
                );
        }
        return render_plan.can_remove_item(&item);
    }
    false
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
    if known_macro_dependency_target_should_remain(project, reduced, package, target)
        && !target.last().is_some_and(|leaf| {
            is_derive_only_external_trait_import(leaf)
                || is_thiserror_error_derive_import(target, leaf)
        })
    {
        return false;
    }

    if pruned_local_proc_macro_dependency_target_should_remain(
        project,
        reduced,
        render_plan,
        package,
        module_path,
        target,
    ) || pruned_proc_macro_helper_dependency_target_should_remain(
        project,
        reduced,
        render_plan,
        package,
        module_path,
        target,
    ) {
        return false;
    }

    if target
        .first()
        .is_some_and(|first| use_name_is_pruned_local_dependency(project, reduced, package, first))
    {
        if pruned_proc_macro_helper_dependency_target_should_remain(
            project,
            reduced,
            render_plan,
            package,
            module_path,
            target,
        ) {
            return false;
        }
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
    if target.last().is_some_and(|leaf| {
        reduced.packages.contains(&target_package)
            && package_is_proc_macro(project, &target_package)
            && proc_macro_export_is_reachable(project, reduced, &target_package, leaf)
            && rendered_attrs_mention_unqualified_ident(
                project,
                reduced,
                render_plan,
                package,
                module_path,
                leaf,
            )
    }) {
        return false;
    }

    let leaf_is_used_in_module = target.last().is_some_and(|leaf| {
        if is_public_use {
            reachable_module_import_scope_mentions_ident(
                project,
                reduced,
                render_plan,
                package,
                module_path,
                leaf,
            )
        } else {
            reachable_module_import_scope_uses_imported_ident(
                project,
                reduced,
                render_plan,
                package,
                module_path,
                leaf,
            )
        }
    });
    if let Some(callable) = find_use_function(project, &target_package, &target_path) {
        if !is_public_use && !leaf_is_used_in_module {
            return true;
        }
        return render_plan.can_remove_callable(&callable);
    }
    if let Some(item) = find_use_item(project, &target_package, &target_path) {
        if !is_public_use && item.kind == ItemKind::Trait && !leaf_is_used_in_module {
            return !local_trait_import_should_remain(
                project,
                reduced,
                render_plan,
                package,
                module_path,
                &item,
            );
        }
        if !is_public_use && item.kind != ItemKind::Trait && !leaf_is_used_in_module {
            return true;
        }
        if (is_public_use || leaf_is_used_in_module) && item.kind == ItemKind::Mod {
            return false;
        }
        return render_plan.can_remove_item(&item);
    }
    if let Some((alias_package, alias_path)) =
        resolve_reexported_use_path(project, &target_package, &target_path)
    {
        if let Some(callable) = find_use_function(project, &alias_package, &alias_path) {
            if !is_public_use && !leaf_is_used_in_module {
                return true;
            }
            return render_plan.can_remove_callable(&callable);
        }
        if let Some(item) = find_use_item(project, &alias_package, &alias_path) {
            if !is_public_use && item.kind == ItemKind::Trait && !leaf_is_used_in_module {
                return !local_trait_import_should_remain(
                    project,
                    reduced,
                    render_plan,
                    package,
                    module_path,
                    &item,
                );
            }
            if !is_public_use && item.kind != ItemKind::Trait && !leaf_is_used_in_module {
                return true;
            }
            if (is_public_use || leaf_is_used_in_module) && item.kind == ItemKind::Mod {
                return false;
            }
            return render_plan.can_remove_item(&item);
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
        } else if target
            .first()
            .is_some_and(|first| use_name_is_dependency(project, package, first))
        {
            !reachable_module_import_scope_mentions_unqualified_ident(
                project,
                reduced,
                render_plan,
                package,
                module_path,
                leaf,
            )
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

fn local_trait_import_should_remain(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
    trait_item: &ItemId,
) -> bool {
    let Some(record) = project.items.get(trait_item) else {
        return false;
    };
    let Item::Trait(item_trait) = &record.item else {
        return false;
    };
    item_trait.items.iter().any(|item| {
        let TraitItem::Fn(function) = item else {
            return false;
        };
        let method = function.sig.ident.to_string();
        reachable_import_scope_has_method_call(
            project,
            reduced,
            render_plan,
            package,
            module_path,
            &method,
        ) || render_plan.module_mentions_ident(package, module_path, &method)
            || reachable_module_mentions_ident(project, reduced, package, module_path, &method)
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
        if pruned_proc_macro_helper_dependency_target_should_remain(
            project,
            reduced,
            render_plan,
            package,
            module_path,
            prefix,
        ) {
            return false;
        }
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
            if !project_has_module_or_inline(project, &target_package, &target_path) {
                return false;
            }
            if is_public_use {
                let exposes_referenced_name = public_glob_prefix_exposes_referenced_name(
                    project,
                    reduced,
                    render_plan,
                    package,
                    module_path,
                    &target_package,
                    &target_path,
                );
                if !module_should_render(
                    project,
                    reduced,
                    render_plan,
                    &target_package,
                    &target_path,
                ) && !exposes_referenced_name
                {
                    return true;
                }
                return !exposes_referenced_name;
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

    if is_thiserror_error_derive_import(target, leaf) {
        return !rendered_attrs_mention_unqualified_ident(
            project,
            reduced,
            render_plan,
            _package,
            module_path,
            leaf,
        );
    }

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
        !reachable_module_import_scope_uses_imported_ident(
            project,
            reduced,
            render_plan,
            _package,
            module_path,
            leaf,
        )
    }
}

fn reachable_module_import_scope_uses_imported_ident(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
    ident: &str,
) -> bool {
    let key = ImportScopeMentionKey::new(package, module_path, ident, None);
    if let Some(value) = render_plan.import_scope_uses.borrow().get(&key).copied() {
        return value;
    }
    let value = reachable_module_import_scope_uses_imported_ident_uncached(
        project,
        reduced,
        render_plan,
        package,
        module_path,
        ident,
    );
    render_plan
        .import_scope_uses
        .borrow_mut()
        .insert(key, value);
    value
}

fn reachable_module_import_scope_uses_imported_ident_uncached(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
    ident: &str,
) -> bool {
    if reachable_module_uses_imported_ident(
        project,
        reduced,
        render_plan,
        package,
        module_path,
        ident,
    ) {
        return true;
    }

    if retained_impl_attrs_mention_unqualified_ident(project, reduced, package, module_path, ident)
        || retained_impl_headers_mention_unqualified_ident(
            project,
            reduced,
            package,
            module_path,
            ident,
        )
        || retained_impl_non_fn_items_mention_unqualified_ident(
            project,
            reduced,
            package,
            module_path,
            ident,
        )
        || retained_root_macro_impl_items_mention_unqualified_ident(
            project,
            reduced,
            package,
            module_path,
            ident,
        )
        || retained_macro_invocations_mention_unqualified_ident(
            project,
            reduced,
            package,
            module_path,
            ident,
        )
        || retained_foreign_items_mention_unqualified_ident_in_plan(
            project,
            render_plan,
            package,
            module_path,
            ident,
        )
        || rendered_attrs_mention_unqualified_ident(
            project,
            reduced,
            render_plan,
            package,
            module_path,
            ident,
        )
    {
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
                None,
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
        .filter(|source| {
            module_should_render(project, reduced, render_plan, package, &source.module_path)
        })
        .any(|source| {
            child_module_import_scope_uses_parent_ident(
                project,
                reduced,
                render_plan,
                package,
                source.module_path.as_slice(),
                &source.syntax.items,
                ident,
            )
        })
}

fn inline_child_modules_import_scope_uses_imported_ident(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
    items: &[Item],
    ident: &str,
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
        if !module_should_render(project, reduced, render_plan, package, &child_path) {
            return false;
        }

        child_module_import_scope_uses_parent_ident(
            project,
            reduced,
            render_plan,
            package,
            &child_path,
            child_items,
            ident,
        )
    })
}

fn child_module_import_scope_uses_parent_ident(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    child_module_path: &[String],
    child_items: &[Item],
    ident: &str,
) -> bool {
    let visible_names = super_import_visible_names_for_parent_ident(child_items, ident);
    if visible_names.iter().any(|visible_name| {
        reachable_module_import_scope_uses_imported_ident(
            project,
            reduced,
            render_plan,
            package,
            child_module_path,
            visible_name,
        )
    }) {
        return true;
    }

    items_have_super_glob_import(child_items)
        && reachable_module_import_scope_uses_imported_ident(
            project,
            reduced,
            render_plan,
            package,
            child_module_path,
            ident,
        )
}

fn reachable_module_uses_imported_ident(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
    ident: &str,
) -> bool {
    if reduced
        .reachable
        .iter()
        .filter(|callable| callable.package() == package)
        .any(|callable| {
            project.functions.get(callable).is_some_and(|record| {
                record.module_path == module_path
                    && function_uses_imported_ident(
                        project,
                        reduced,
                        render_plan,
                        package,
                        module_path,
                        &record.item,
                        ident,
                    )
            }) || project.methods.get(callable).is_some_and(|record| {
                let current_type_path = match callable {
                    CallableId::Method { type_path, .. } => Some(type_path.as_slice()),
                    CallableId::Free { .. } => None,
                };
                record.module_path == module_path
                    && method_uses_imported_ident(
                        project,
                        reduced,
                        render_plan,
                        package,
                        module_path,
                        current_type_path,
                        &record.item,
                        ident,
                    )
            })
        })
    {
        return true;
    }

    reachable_module_non_callable_mentions_imported_ident(
        project,
        reduced,
        render_plan,
        package,
        module_path,
        ident,
    )
}

fn reachable_module_non_callable_mentions_imported_ident(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
    ident: &str,
) -> bool {
    render_plan
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
        || retained_root_macro_impl_items_mention_ident(
            project,
            reduced,
            package,
            module_path,
            ident,
        )
        || retained_macro_invocations_mention_ident(project, reduced, package, module_path, ident)
}

fn function_uses_imported_ident(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
    function: &syn::ItemFn,
    ident: &str,
) -> bool {
    let mut block = function.block.clone();
    prune_rendered_struct_literal_fields(
        project,
        reduced,
        render_plan,
        package,
        module_path,
        None,
        &mut block,
    );
    let mut visitor = ImportUsageVisitor::new(ident);
    visitor.push_scope();
    visitor.visit_signature(&function.sig);
    visitor.visit_block(&block);
    visitor.found
}

fn method_uses_imported_ident(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
    current_type_path: Option<&[String]>,
    function: &syn::ImplItemFn,
    ident: &str,
) -> bool {
    let mut block = function.block.clone();
    prune_rendered_struct_literal_fields(
        project,
        reduced,
        render_plan,
        package,
        module_path,
        current_type_path,
        &mut block,
    );
    let mut visitor = ImportUsageVisitor::new(ident);
    visitor.push_scope();
    visitor.visit_signature(&function.sig);
    visitor.visit_block(&block);
    visitor.found
}

struct ImportUsageVisitor<'a> {
    ident: &'a str,
    scopes: Vec<BTreeSet<String>>,
    found: bool,
}

impl<'a> ImportUsageVisitor<'a> {
    fn new(ident: &'a str) -> Self {
        Self {
            ident,
            scopes: vec![BTreeSet::new()],
            found: false,
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(BTreeSet::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
        if self.scopes.is_empty() {
            self.scopes.push(BTreeSet::new());
        }
    }

    fn add_bindings_from_pat(&mut self, pat: &syn::Pat) {
        let mut bindings = BTreeSet::new();
        collect_pat_bindings(pat, &mut bindings);
        if let Some(scope) = self.scopes.last_mut() {
            scope.extend(bindings);
        }
    }

    fn is_value_bound(&self, ident: &str) -> bool {
        self.scopes.iter().rev().any(|scope| scope.contains(ident))
    }

    fn visit_signature(&mut self, signature: &syn::Signature) {
        for generic in &signature.generics.params {
            visit::visit_generic_param(self, generic);
        }
        if let Some(where_clause) = &signature.generics.where_clause {
            visit::visit_where_clause(self, where_clause);
        }
        for input in &signature.inputs {
            match input {
                syn::FnArg::Receiver(receiver) => visit::visit_receiver(self, receiver),
                syn::FnArg::Typed(input) => {
                    self.visit_pat(&input.pat);
                    self.visit_type(&input.ty);
                    self.add_bindings_from_pat(&input.pat);
                }
            }
        }
        visit::visit_return_type(self, &signature.output);
    }

    fn path_uses_import(&mut self, path: &syn::Path) {
        if path.leading_colon.is_some() {
            return;
        }
        let Some(first) = path.segments.first() else {
            return;
        };
        if first.ident != self.ident {
            return;
        }
        if path.segments.len() > 1 || !self.is_value_bound(self.ident) {
            self.found = true;
        }
    }
}

impl Visit<'_> for ImportUsageVisitor<'_> {
    fn visit_block(&mut self, block: &syn::Block) {
        self.push_scope();
        for statement in &block.stmts {
            self.visit_stmt(statement);
        }
        self.pop_scope();
    }

    fn visit_stmt(&mut self, statement: &syn::Stmt) {
        match statement {
            syn::Stmt::Local(local) => {
                self.visit_pat(&local.pat);
                if let Some(init) = &local.init {
                    self.visit_expr(&init.expr);
                    if let Some((_else_token, diverge)) = &init.diverge {
                        self.visit_expr(diverge);
                    }
                }
                self.add_bindings_from_pat(&local.pat);
            }
            syn::Stmt::Item(item) => self.visit_item(item),
            syn::Stmt::Expr(expr, _) => self.visit_expr(expr),
            syn::Stmt::Macro(item) => self.visit_macro(&item.mac),
        }
    }

    fn visit_arm(&mut self, arm: &syn::Arm) {
        self.visit_pat(&arm.pat);
        self.push_scope();
        self.add_bindings_from_pat(&arm.pat);
        if let Some((_if_token, guard)) = &arm.guard {
            self.visit_expr(guard);
        }
        self.visit_expr(&arm.body);
        self.pop_scope();
    }

    fn visit_expr_closure(&mut self, closure: &syn::ExprClosure) {
        self.push_scope();
        for input in &closure.inputs {
            self.visit_pat(input);
            self.add_bindings_from_pat(input);
        }
        visit::visit_return_type(self, &closure.output);
        self.visit_expr(&closure.body);
        self.pop_scope();
    }

    fn visit_expr_for_loop(&mut self, loop_expr: &syn::ExprForLoop) {
        self.visit_expr(&loop_expr.expr);
        self.visit_pat(&loop_expr.pat);
        self.push_scope();
        self.add_bindings_from_pat(&loop_expr.pat);
        self.visit_block(&loop_expr.body);
        self.pop_scope();
    }

    fn visit_path(&mut self, path: &syn::Path) {
        self.path_uses_import(path);
        visit::visit_path(self, path);
    }

    fn visit_macro(&mut self, item_macro: &syn::Macro) {
        if item_macro
            .path
            .segments
            .first()
            .is_some_and(|segment| segment.ident == self.ident)
            || token_stream_mentions_ident(&item_macro.tokens, self.ident)
        {
            self.found = true;
        }
        visit::visit_macro(self, item_macro);
    }
}

fn collect_pat_bindings(pat: &syn::Pat, bindings: &mut BTreeSet<String>) {
    match pat {
        syn::Pat::Ident(ident) => {
            bindings.insert(ident.ident.to_string());
            if let Some((_at, subpat)) = &ident.subpat {
                collect_pat_bindings(subpat, bindings);
            }
        }
        syn::Pat::Or(pat) => {
            for case in &pat.cases {
                collect_pat_bindings(case, bindings);
            }
        }
        syn::Pat::Paren(pat) => collect_pat_bindings(&pat.pat, bindings),
        syn::Pat::Reference(pat) => collect_pat_bindings(&pat.pat, bindings),
        syn::Pat::Rest(_)
        | syn::Pat::Lit(_)
        | syn::Pat::Macro(_)
        | syn::Pat::Path(_)
        | syn::Pat::Range(_)
        | syn::Pat::Verbatim(_)
        | syn::Pat::Wild(_) => {}
        syn::Pat::Slice(pat) => {
            for elem in &pat.elems {
                collect_pat_bindings(elem, bindings);
            }
        }
        syn::Pat::Struct(pat) => {
            for field in &pat.fields {
                collect_pat_bindings(&field.pat, bindings);
            }
        }
        syn::Pat::Tuple(pat) => {
            for elem in &pat.elems {
                collect_pat_bindings(elem, bindings);
            }
        }
        syn::Pat::TupleStruct(pat) => {
            for elem in &pat.elems {
                collect_pat_bindings(elem, bindings);
            }
        }
        syn::Pat::Type(pat) => collect_pat_bindings(&pat.pat, bindings),
        _ => {}
    }
}

fn public_glob_prefix_exposes_referenced_name(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    source_module_path: &[String],
    target_package: &str,
    target_module_path: &[String],
) -> bool {
    let Some(items) = module_items_for_path(project, target_package, target_module_path) else {
        return false;
    };
    module_rendered_public_visible_names(
        project,
        reduced,
        render_plan,
        target_package,
        target_module_path,
        items,
    )
    .iter()
    .any(|name| {
        public_glob_exposed_name_is_used(
            project,
            reduced,
            render_plan,
            package,
            source_module_path,
            target_package,
            target_module_path,
            name,
        )
    })
}

fn public_glob_exposed_name_is_used(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    source_module_path: &[String],
    target_package: &str,
    target_module_path: &[String],
    name: &str,
) -> bool {
    if !source_module_path.is_empty()
        && reachable_package_mentions_module_path_ident(
            project,
            reduced,
            package,
            source_module_path,
            name,
        )
    {
        return true;
    }

    if !source_module_path.is_empty()
        && rendered_imports_mention_module_path_ident(
            project,
            reduced,
            render_plan,
            package,
            source_module_path,
            name,
        )
    {
        return true;
    }

    if !source_module_path.is_empty()
        && reachable_module_import_scope_mentions_ident(
            project,
            reduced,
            render_plan,
            package,
            source_module_path,
            name,
        )
    {
        return true;
    }

    if !source_module_path.is_empty()
        && public_reexport_path_name_is_referenced_by_reduced_package(
            project,
            reduced,
            render_plan,
            package,
            source_module_path,
            name,
        )
    {
        return true;
    }

    if source_module_path.is_empty()
        && reachable_package_mentions_unqualified_ident(project, reduced, package, name)
    {
        return true;
    }

    public_glob_target_name_is_selected_root(reduced, target_package, target_module_path, name)
        || (source_module_path.is_empty()
            && public_reexport_name_is_referenced_by_reduced_package(
                project,
                reduced,
                target_package,
                name,
            ))
}

fn public_reexport_path_name_is_referenced_by_reduced_package(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
    visible_name: &str,
) -> bool {
    reduced
        .reachable
        .iter()
        .filter(|callable| callable.package() != package)
        .any(|callable| {
            project.functions.get(callable).is_some_and(|record| {
                token_stream_mentions_dependency_public_path_name(
                    project,
                    &record.package,
                    package,
                    module_path,
                    visible_name,
                    &record.item.to_token_stream(),
                )
            }) || project.methods.get(callable).is_some_and(|record| {
                token_stream_mentions_dependency_public_path_name(
                    project,
                    callable.package(),
                    package,
                    module_path,
                    visible_name,
                    &record.item.to_token_stream(),
                )
            })
        })
        || reduced
            .reachable_items
            .iter()
            .filter(|item| item.package != package)
            .any(|item| {
                project.items.get(item).is_some_and(|record| {
                    token_stream_mentions_dependency_public_path_name(
                        project,
                        &record.package,
                        package,
                        module_path,
                        visible_name,
                        &record.item.to_token_stream(),
                    )
                })
            })
        || rendered_imports_mention_dependency_module_path_ident(
            project,
            reduced,
            render_plan,
            package,
            module_path,
            visible_name,
        )
}

fn token_stream_mentions_dependency_public_path_name(
    project: &Project,
    caller_package: &str,
    dependency_package: &str,
    module_path: &[String],
    visible_name: &str,
    tokens: &TokenStream,
) -> bool {
    token_path_candidates(tokens).iter().any(|segments| {
        let Some((root, rest)) = segments.split_first() else {
            return false;
        };
        if !first_segment_targets_dependency(project, caller_package, dependency_package, root) {
            return false;
        }
        let mut expected = module_path.to_vec();
        expected.push(visible_name.to_string());
        rest == expected.as_slice()
    })
}

fn rendered_imports_mention_dependency_module_path_ident(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    dependency_package: &str,
    module_path: &[String],
    ident: &str,
) -> bool {
    project
        .files
        .values()
        .filter(|source| source.package != dependency_package)
        .filter(|source| {
            module_should_render(
                project,
                reduced,
                render_plan,
                &source.package,
                &source.module_path,
            )
        })
        .any(|source| {
            source.syntax.items.iter().any(|item| {
                let Item::Use(item_use) = item else {
                    return false;
                };
                use_tree_mentions_dependency_module_path_ident(
                    project,
                    reduced,
                    render_plan,
                    &source.package,
                    &source.module_path,
                    &item_use.tree,
                    Vec::new(),
                    dependency_package,
                    module_path,
                    ident,
                )
            })
        })
}

#[allow(clippy::too_many_arguments)]
fn use_tree_mentions_dependency_module_path_ident(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    caller_package: &str,
    caller_module_path: &[String],
    tree: &UseTree,
    mut prefix: Vec<String>,
    dependency_package: &str,
    module_path: &[String],
    ident: &str,
) -> bool {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            use_tree_mentions_dependency_module_path_ident(
                project,
                reduced,
                render_plan,
                caller_package,
                caller_module_path,
                &path.tree,
                prefix,
                dependency_package,
                module_path,
                ident,
            )
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
            if name.ident != "self" {
                prefix.push(name.ident.to_string());
            }
            dependency_use_target_matches_module_path_ident(
                project,
                caller_package,
                dependency_package,
                &prefix,
                module_path,
                ident,
            ) && reachable_module_import_scope_uses_imported_ident(
                project,
                reduced,
                render_plan,
                caller_package,
                caller_module_path,
                &visible_name,
            )
        }
        UseTree::Rename(rename) => {
            prefix.push(rename.ident.to_string());
            dependency_use_target_matches_module_path_ident(
                project,
                caller_package,
                dependency_package,
                &prefix,
                module_path,
                ident,
            ) && reachable_module_import_scope_uses_imported_ident(
                project,
                reduced,
                render_plan,
                caller_package,
                caller_module_path,
                &rename.rename.to_string(),
            )
        }
        UseTree::Group(group) => group.items.iter().any(|nested| {
            use_tree_mentions_dependency_module_path_ident(
                project,
                reduced,
                render_plan,
                caller_package,
                caller_module_path,
                nested,
                prefix.clone(),
                dependency_package,
                module_path,
                ident,
            )
        }),
        UseTree::Glob(_) => {
            dependency_use_prefix_matches_module_path(
                project,
                caller_package,
                dependency_package,
                &prefix,
                module_path,
            ) && reachable_module_import_scope_uses_imported_ident(
                project,
                reduced,
                render_plan,
                caller_package,
                caller_module_path,
                ident,
            )
        }
    }
}

fn dependency_use_target_matches_module_path_ident(
    project: &Project,
    caller_package: &str,
    dependency_package: &str,
    target: &[String],
    module_path: &[String],
    ident: &str,
) -> bool {
    let Some((root, rest)) = target.split_first() else {
        return false;
    };
    if !first_segment_targets_dependency(project, caller_package, dependency_package, root) {
        return false;
    }
    let mut expected = module_path.to_vec();
    expected.push(ident.to_string());
    rest == expected.as_slice()
}

fn dependency_use_prefix_matches_module_path(
    project: &Project,
    caller_package: &str,
    dependency_package: &str,
    target: &[String],
    module_path: &[String],
) -> bool {
    let Some((root, rest)) = target.split_first() else {
        return false;
    };
    first_segment_targets_dependency(project, caller_package, dependency_package, root)
        && rest == module_path
}

fn rendered_imports_mention_module_path_ident(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
    ident: &str,
) -> bool {
    project
        .files
        .values()
        .filter(|source| source.package == package)
        .filter(|source| {
            module_should_render(project, reduced, render_plan, package, &source.module_path)
        })
        .any(|source| {
            source.syntax.items.iter().any(|item| {
                let Item::Use(item_use) = item else {
                    return false;
                };
                use_tree_mentions_module_path_ident(&item_use.tree, Vec::new(), module_path, ident)
            })
        })
}

fn use_tree_mentions_module_path_ident(
    tree: &UseTree,
    mut prefix: Vec<String>,
    module_path: &[String],
    ident: &str,
) -> bool {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            use_tree_mentions_module_path_ident(&path.tree, prefix, module_path, ident)
        }
        UseTree::Name(name) => {
            if name.ident != "self" {
                prefix.push(name.ident.to_string());
            }
            use_target_mentions_module_path_ident(&prefix, module_path, ident)
        }
        UseTree::Rename(rename) => {
            prefix.push(rename.ident.to_string());
            use_target_mentions_module_path_ident(&prefix, module_path, ident)
        }
        UseTree::Group(group) => group.items.iter().any(|nested| {
            use_tree_mentions_module_path_ident(nested, prefix.clone(), module_path, ident)
        }),
        UseTree::Glob(_) => use_target_mentions_module_path_ident(&prefix, module_path, ident),
    }
}

fn use_target_mentions_module_path_ident(
    target: &[String],
    module_path: &[String],
    ident: &str,
) -> bool {
    let mut expected = module_path.to_vec();
    expected.push(ident.to_string());
    path_ends_with(target, &expected)
}

fn public_glob_target_name_is_selected_root(
    reduced: &ReducedProject,
    target_package: &str,
    target_module_path: &[String],
    name: &str,
) -> bool {
    reduced.roots.iter().any(|root| match root {
        RootId::Callable(CallableId::Free {
            package,
            module_path,
            name: callable_name,
        }) => {
            package == target_package && module_path == target_module_path && callable_name == name
        }
        RootId::Item(item) => {
            item.package == target_package
                && item.module_path == target_module_path
                && item.name == name
        }
        _ => false,
    })
}

fn reachable_package_mentions_unqualified_ident(
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
            project.functions.get(callable).is_some_and(|record| {
                token_stream_mentions_unqualified_ident(&record.item.to_token_stream(), ident)
            }) || project.methods.get(callable).is_some_and(|record| {
                token_stream_mentions_unqualified_ident(&record.item.to_token_stream(), ident)
            })
        })
        || reduced
            .reachable_items
            .iter()
            .filter(|item| {
                item.package == package && item.name != ident && item.kind != ItemKind::Mod
            })
            .any(|item| {
                project.items.get(item).is_some_and(|record| {
                    token_stream_mentions_unqualified_ident(&record.item.to_token_stream(), ident)
                })
            })
}

fn reachable_package_mentions_module_path_ident(
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
                token_stream_mentions_module_path_ident(
                    &record.item.to_token_stream(),
                    module_path,
                    ident,
                )
            }) || project.methods.get(callable).is_some_and(|record| {
                token_stream_mentions_module_path_ident(
                    &record.item.to_token_stream(),
                    module_path,
                    ident,
                )
            })
        })
        || reduced
            .reachable_items
            .iter()
            .filter(|item| item.package == package)
            .any(|item| {
                project.items.get(item).is_some_and(|record| {
                    token_stream_mentions_module_path_ident(
                        &record.item.to_token_stream(),
                        module_path,
                        ident,
                    )
                })
            })
}

fn module_rendered_public_visible_names(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
    items: &[Item],
) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for item in items {
        match item {
            Item::Use(item_use) if use_is_reexport(&item_use.vis) => {
                collect_use_tree_visible_names(&item_use.tree, Vec::new(), &mut names);
            }
            _ if item_is_public(item)
                && public_visible_item_should_render(
                    project,
                    reduced,
                    render_plan,
                    package,
                    module_path,
                    item,
                ) =>
            {
                if let Some(name) = support_item_name(item) {
                    names.insert(name);
                }
            }
            _ => {}
        }
    }
    names
}

fn public_visible_item_should_render(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
    item: &Item,
) -> bool {
    match item {
        Item::Fn(function) => {
            let id = CallableId::Free {
                package: package.to_string(),
                module_path: module_path.to_vec(),
                name: function.sig.ident.to_string(),
            };
            render_plan.callable_should_render(&id)
        }
        Item::Mod(item_mod) => {
            let mut child_path = module_path.to_vec();
            child_path.push(item_mod.ident.to_string());
            module_should_render(project, reduced, render_plan, package, &child_path)
        }
        _ => item_id(package, module_path, item)
            .is_some_and(|id| render_plan.item_should_render(&id)),
    }
}

fn collect_use_tree_visible_names(
    tree: &UseTree,
    mut prefix: Vec<String>,
    names: &mut BTreeSet<String>,
) {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            collect_use_tree_visible_names(&path.tree, prefix, names);
        }
        UseTree::Name(name) => {
            if name.ident == "self" {
                if let Some(prefix) = prefix.last() {
                    names.insert(prefix.clone());
                }
            } else {
                names.insert(name.ident.to_string());
            }
        }
        UseTree::Rename(rename) => {
            names.insert(rename.rename.to_string());
        }
        UseTree::Group(group) => {
            for item in &group.items {
                collect_use_tree_visible_names(item, prefix.clone(), names);
            }
        }
        UseTree::Glob(_) => {}
    }
}

fn item_is_public(item: &Item) -> bool {
    match item {
        Item::Const(item) => use_is_reexport(&item.vis),
        Item::Enum(item) => use_is_reexport(&item.vis),
        Item::ExternCrate(item) => use_is_reexport(&item.vis),
        Item::Fn(item) => use_is_reexport(&item.vis),
        Item::Macro(_) => false,
        Item::Mod(item) => use_is_reexport(&item.vis),
        Item::Static(item) => use_is_reexport(&item.vis),
        Item::Struct(item) => use_is_reexport(&item.vis),
        Item::Trait(item) => use_is_reexport(&item.vis),
        Item::TraitAlias(item) => use_is_reexport(&item.vis),
        Item::Type(item) => use_is_reexport(&item.vis),
        Item::Union(item) => use_is_reexport(&item.vis),
        _ => false,
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
    if source_trait_import_should_remain(
        project,
        reduced,
        render_plan,
        package,
        module_path,
        target,
    ) {
        return true;
    }
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
                reachable_import_scope_has_associated_function_call(
                    project,
                    reduced,
                    render_plan,
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
            reachable_import_scope_has_method_call(
                project,
                reduced,
                render_plan,
                package,
                module_path,
                method,
            )
        });
    }
    if !is_external_trait_import_candidate(target, leaf) {
        return false;
    }
    if is_derive_only_external_trait_import(leaf) {
        if render_plan.module_mentions_ident(package, module_path, leaf) {
            return true;
        }
        if known_trait_method_idents(target, leaf).is_some_and(|methods| {
            methods.iter().any(|method| {
                reachable_import_scope_has_method_call(
                    project,
                    reduced,
                    render_plan,
                    package,
                    module_path,
                    method,
                )
            })
        }) {
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
        let is_known_trait = is_known_external_trait_import(leaf)
            || target
                .first()
                .is_some_and(|first| matches!(first.as_str(), "std" | "core" | "alloc"))
                && is_known_std_trait_import(leaf);
        if is_known_trait
            && (render_plan.module_mentions_ident(package, module_path, leaf)
                || reachable_module_mentions_ident(project, reduced, package, module_path, leaf))
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
                reachable_import_scope_has_associated_function_call(
                    project,
                    reduced,
                    render_plan,
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
            reachable_import_scope_has_method_call(
                project,
                reduced,
                render_plan,
                package,
                module_path,
                method,
            )
        });
    }
    target
        .iter()
        .take(target.len().saturating_sub(1))
        .any(|segment| render_plan.package_mentions_ident(package, segment))
}

fn source_trait_import_should_remain(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
    target: &[String],
) -> bool {
    let Some((target_package, target_path)) =
        resolve_use_target_path(project, package, module_path, target)
    else {
        return external_source_trait_import_should_remain(
            project,
            reduced,
            render_plan,
            package,
            module_path,
            target,
        );
    };
    let Some(item) = find_use_item(project, &target_package, &target_path) else {
        return external_source_trait_import_should_remain(
            project,
            reduced,
            render_plan,
            package,
            module_path,
            target,
        );
    };
    item.kind == ItemKind::Trait
        && local_trait_import_should_remain(
            project,
            reduced,
            render_plan,
            package,
            module_path,
            &item,
        )
}

fn external_source_trait_import_should_remain(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
    target: &[String],
) -> bool {
    external_dependency_trait_method_names(project, package, target)
        .into_iter()
        .any(|method| {
            reachable_import_scope_has_method_call(
                project,
                reduced,
                render_plan,
                package,
                module_path,
                &method,
            ) || render_plan.module_mentions_ident(package, module_path, &method)
                || reachable_module_mentions_ident(project, reduced, package, module_path, &method)
        })
}

fn external_dependency_trait_method_names(
    project: &Project,
    package: &str,
    target: &[String],
) -> BTreeSet<String> {
    let Some(dependency_name) = target.first() else {
        return BTreeSet::new();
    };
    let Some(trait_name) = target.last() else {
        return BTreeSet::new();
    };
    let Some(package_record) = project.workspace.packages.get(package) else {
        return BTreeSet::new();
    };
    for (_table_name, table) in package_dependency_tables(package_record) {
        for (alias, value) in table {
            if !dependency_value_name_matches(alias, value, dependency_name) {
                continue;
            }
            let Ok(Some(root)) = dependency_path_root(value, &package_record.root) else {
                continue;
            };
            if let Some(methods) = trait_method_names_from_package_root(&root, trait_name) {
                return methods;
            }
        }
    }
    BTreeSet::new()
}

fn dependency_value_name_matches(alias: &str, value: &Value, name: &str) -> bool {
    alias == name
        || dependency_code_name(alias) == name
        || value
            .as_table()
            .and_then(|table| table.get("package"))
            .and_then(Value::as_str)
            .is_some_and(|package| package == name || dependency_code_name(package) == name)
}

fn trait_method_names_from_package_root(root: &Path, trait_name: &str) -> Option<BTreeSet<String>> {
    let manifest = read_toml_value(&root.join("Cargo.toml")).ok()?;
    let lib_path = manifest
        .get("lib")
        .and_then(Value::as_table)
        .and_then(|table| table.get("path"))
        .and_then(Value::as_str)
        .map(|path| root.join(path))
        .unwrap_or_else(|| root.join("src/lib.rs"));
    let source = fs::read_to_string(lib_path).ok()?;
    let syntax = syn::parse_file(&source).ok()?;
    syntax.items.iter().find_map(|item| {
        let Item::Trait(item_trait) = item else {
            return None;
        };
        if item_trait.ident != trait_name {
            return None;
        }
        Some(
            item_trait
                .items
                .iter()
                .filter_map(|item| {
                    let TraitItem::Fn(function) = item else {
                        return None;
                    };
                    Some(function.sig.ident.to_string())
                })
                .collect(),
        )
    })
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
        (Some("std" | "core" | "alloc"), "FromStr") => Some(&["from_str"]),
        _ => None,
    }
}

fn is_derive_only_external_trait_import(leaf: &str) -> bool {
    matches!(leaf, "Deserialize" | "Serialize")
}

fn is_thiserror_error_derive_import(target: &[String], leaf: &str) -> bool {
    leaf == "Error"
        && target
            .first()
            .is_some_and(|first| dependency_code_name(first) == "thiserror")
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

fn use_name_is_dependency(project: &Project, package: &str, name: &str) -> bool {
    let Some(package_record) = project.workspace.packages.get(package) else {
        return false;
    };
    package_record
        .dependencies
        .iter()
        .any(|dependency| dependency_name_matches(dependency, name))
}

fn pruned_local_proc_macro_dependency_alias_should_remain(
    project: &Project,
    package: &str,
    alias: &str,
) -> bool {
    local_proc_macro_dependency_package_for_alias(project, package, alias).is_some()
}

fn local_proc_macro_dependency_package_for_alias(
    project: &Project,
    package: &str,
    alias: &str,
) -> Option<String> {
    let package_record = project.workspace.packages.get(package)?;
    package_record.dependencies.iter().find_map(|dependency| {
        (dependency_name_matches(dependency, alias)
            && project.workspace.packages.contains_key(&dependency.package)
            && package_is_proc_macro(project, &dependency.package))
        .then(|| dependency.package.clone())
    })
}

fn local_dependency_package_for_alias(
    project: &Project,
    package: &str,
    alias: &str,
) -> Option<String> {
    let package_record = project.workspace.packages.get(package)?;
    package_record.dependencies.iter().find_map(|dependency| {
        (dependency_name_matches(dependency, alias)
            && project.workspace.packages.contains_key(&dependency.package))
        .then(|| dependency.package.clone())
    })
}

fn pruned_local_proc_macro_dependency_target_should_remain(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
    target: &[String],
) -> bool {
    let Some(first) = target.first() else {
        return false;
    };
    let Some(target_package) =
        local_proc_macro_dependency_package_for_alias(project, package, first)
    else {
        return false;
    };
    target.last().is_some_and(|leaf| {
        proc_macro_export_is_reachable(project, reduced, &target_package, leaf)
            && rendered_attrs_mention_unqualified_ident(
                project,
                reduced,
                render_plan,
                package,
                module_path,
                leaf,
            )
    })
}

fn pruned_proc_macro_helper_dependency_alias_should_remain(
    project: &Project,
    reduced: &ReducedProject,
    package: &str,
    alias: &str,
) -> bool {
    package_is_proc_macro(project, package)
        && reduced.packages.contains(package)
        && local_dependency_package_for_alias(project, package, alias)
            .is_some_and(|dependency| !reduced.packages.contains(&dependency))
}

fn pruned_proc_macro_helper_dependency_target_should_remain(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
    target: &[String],
) -> bool {
    let Some(first) = target.first() else {
        return false;
    };
    if !package_is_proc_macro(project, package) || !reduced.packages.contains(package) {
        return false;
    }
    if local_dependency_package_for_alias(project, package, first)
        .is_none_or(|dependency| reduced.packages.contains(&dependency))
    {
        return false;
    }
    target.last().is_some_and(|leaf| {
        reachable_module_import_scope_mentions_ident(
            project,
            reduced,
            render_plan,
            package,
            module_path,
            leaf,
        ) || reachable_package_mentions_ident(project, reduced, package, first)
            || reachable_package_mentions_ident(project, reduced, package, leaf)
    })
}

fn proc_macro_export_is_reachable(
    project: &Project,
    reduced: &ReducedProject,
    proc_macro_package: &str,
    export_name: &str,
) -> bool {
    project.functions.iter().any(|(callable, record)| {
        callable.package() == proc_macro_package
            && reduced.reachable.contains(callable)
            && proc_macro_export_names(&record.item).contains(export_name)
    })
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
        let parser = Punctuated::<Meta, syn::Token![,]>::parse_terminated;
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

fn rendered_attrs_mention_unqualified_ident(
    project: &Project,
    reduced: &ReducedProject,
    render_plan: &RenderPlan,
    package: &str,
    module_path: &[String],
    ident: &str,
) -> bool {
    let Some(items) = module_items_for_path(project, package, module_path) else {
        return false;
    };
    let aliases = project
        .module_aliases
        .get(&(package.to_string(), module_path.to_vec()))
        .cloned()
        .unwrap_or_default();
    items.iter().any(|item| match item {
        Item::Fn(function) => {
            let id = CallableId::Free {
                package: package.to_string(),
                module_path: module_path.to_vec(),
                name: function.sig.ident.to_string(),
            };
            reduced.reachable.contains(&id)
                && attrs_mention_unqualified_ident(&function.attrs, ident)
        }
        Item::Struct(_)
        | Item::Enum(_)
        | Item::Union(_)
        | Item::Type(_)
        | Item::Const(_)
        | Item::Static(_)
        | Item::Macro(_)
        | Item::Trait(_) => item_id(package, module_path, item).is_some_and(|id| {
            render_plan.item_should_render(&id) && item_attrs_mention_unqualified_ident(item, ident)
        }),
        Item::Impl(item_impl) => {
            impl_should_render(project, reduced, package, module_path, item_impl, &aliases)
                && (attrs_mention_unqualified_ident(&item_impl.attrs, ident)
                    || item_impl
                        .items
                        .iter()
                        .any(|item| impl_item_attrs_mention_unqualified_ident(item, ident)))
        }
        _ => false,
    })
}

fn item_attrs_mention_unqualified_ident(item: &Item, ident: &str) -> bool {
    match item {
        Item::Const(item) => attrs_mention_unqualified_ident(&item.attrs, ident),
        Item::Enum(item) => {
            attrs_mention_unqualified_ident(&item.attrs, ident)
                || item.variants.iter().any(|variant| {
                    attrs_mention_unqualified_ident(&variant.attrs, ident)
                        || variant
                            .fields
                            .iter()
                            .any(|field| attrs_mention_unqualified_ident(&field.attrs, ident))
                })
        }
        Item::Macro(item) => attrs_mention_unqualified_ident(&item.attrs, ident),
        Item::Mod(item) => attrs_mention_unqualified_ident(&item.attrs, ident),
        Item::Static(item) => attrs_mention_unqualified_ident(&item.attrs, ident),
        Item::Struct(item) => {
            attrs_mention_unqualified_ident(&item.attrs, ident)
                || item
                    .fields
                    .iter()
                    .any(|field| attrs_mention_unqualified_ident(&field.attrs, ident))
        }
        Item::Trait(item) => {
            attrs_mention_unqualified_ident(&item.attrs, ident)
                || item
                    .items
                    .iter()
                    .any(|item| trait_item_attrs_mention_unqualified_ident(item, ident))
        }
        Item::Type(item) => attrs_mention_unqualified_ident(&item.attrs, ident),
        Item::Union(item) => {
            attrs_mention_unqualified_ident(&item.attrs, ident)
                || item
                    .fields
                    .named
                    .iter()
                    .any(|field| attrs_mention_unqualified_ident(&field.attrs, ident))
        }
        _ => false,
    }
}

fn trait_item_attrs_mention_unqualified_ident(item: &TraitItem, ident: &str) -> bool {
    match item {
        TraitItem::Const(item) => attrs_mention_unqualified_ident(&item.attrs, ident),
        TraitItem::Fn(item) => attrs_mention_unqualified_ident(&item.attrs, ident),
        TraitItem::Macro(item) => attrs_mention_unqualified_ident(&item.attrs, ident),
        TraitItem::Type(item) => attrs_mention_unqualified_ident(&item.attrs, ident),
        TraitItem::Verbatim(_) => false,
        _ => false,
    }
}

fn impl_item_attrs_mention_unqualified_ident(item: &ImplItem, ident: &str) -> bool {
    match item {
        ImplItem::Const(item) => attrs_mention_unqualified_ident(&item.attrs, ident),
        ImplItem::Fn(item) => attrs_mention_unqualified_ident(&item.attrs, ident),
        ImplItem::Macro(item) => attrs_mention_unqualified_ident(&item.attrs, ident),
        ImplItem::Type(item) => attrs_mention_unqualified_ident(&item.attrs, ident),
        ImplItem::Verbatim(_) => false,
        _ => false,
    }
}

fn attrs_mention_unqualified_ident(attrs: &[syn::Attribute], ident: &str) -> bool {
    attrs
        .iter()
        .any(|attr| token_stream_mentions_unqualified_ident(&attr.to_token_stream(), ident))
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
    find_use_function_inner(project, package, path, &mut BTreeSet::new())
}

fn find_use_function_inner(
    project: &Project,
    package: &str,
    path: &[String],
    visited: &mut BTreeSet<(String, Vec<String>)>,
) -> Option<CallableId> {
    if path.is_empty() || !visited.insert((package.to_string(), path.to_vec())) {
        return None;
    }
    if let Some(callable) = find_use_function_direct(project, package, path) {
        return Some(callable);
    }

    let name = path.last()?;
    let module_path = path[..path.len() - 1].to_vec();
    if let Some((alias_package, alias_path)) = resolve_reexported_use_path(project, package, path) {
        if let Some(callable) =
            find_use_function_inner(project, &alias_package, &alias_path, visited)
        {
            return Some(callable);
        }
    }

    find_glob_reexport_function(project, package, &module_path, name, &mut BTreeSet::new())
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
    let items = module_items_for_path(project, package, module_path)?;
    for glob_path in visible_glob_use_paths(items) {
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
        if let Some((alias_package, alias_path)) =
            resolve_reexported_use_path(project, &target_package, &target_path)
        {
            if let Some(callable) = find_use_function_direct(project, &alias_package, &alias_path) {
                return Some(callable);
            }
            if let Some((alias_name, alias_module_path)) = alias_path.split_last() {
                if let Some(callable) = find_glob_reexport_function(
                    project,
                    &alias_package,
                    alias_module_path,
                    alias_name,
                    visited,
                ) {
                    return Some(callable);
                }
            }
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
    let items = module_items_for_path(project, package, module_path)?;
    for glob_path in visible_glob_use_paths(items) {
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
        if let Some((alias_package, alias_path)) =
            resolve_reexported_use_path(project, &target_package, &target_path)
        {
            if let Some(item) = find_use_item_direct(project, &alias_package, &alias_path) {
                return Some(item);
            }
            if let Some((alias_name, alias_module_path)) = alias_path.split_last() {
                if let Some(item) = find_glob_reexport_item(
                    project,
                    &alias_package,
                    alias_module_path,
                    alias_name,
                    visited,
                ) {
                    return Some(item);
                }
            }
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

fn project_has_module_or_inline(project: &Project, package: &str, module_path: &[String]) -> bool {
    project_has_module(project, package, module_path)
        || inline_module_items_for_path(project, package, module_path).is_some()
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
