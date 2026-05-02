use std::collections::{BTreeSet, HashMap, VecDeque};

use proc_macro2::{TokenStream, TokenTree};
use syn::{
    visit::{self, Visit},
    Expr, ExprCall, ExprMacro, ExprMatch, ExprMethodCall, ExprPath, FnArg, GenericArgument,
    ImplItem, ItemMacro, Local, Macro, Pat, PatTupleStruct, Path, PathArguments, Type, TypePath,
};

use crate::model::{CallableId, ItemId, ItemKind, Project, ReducedProject};

pub fn reduce(project: &Project) -> Result<ReducedProject, Box<dyn std::error::Error>> {
    let roots = project
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
        .collect::<Vec<_>>();

    let [root] = roots.as_slice() else {
        return Err(format!(
            "expected exactly one #[opensourced] function, found {}",
            roots.len()
        )
        .into());
    };

    let candidate_packages = package_closure(project, root.package());
    let mut reachable = BTreeSet::new();
    let mut reachable_items = BTreeSet::new();
    let mut callable_queue = VecDeque::from([root.clone()]);
    let mut item_queue = VecDeque::new();

    while !callable_queue.is_empty() || !item_queue.is_empty() {
        while let Some(callable) = callable_queue.pop_front() {
            if !candidate_packages.contains(callable.package())
                || !reachable.insert(callable.clone())
            {
                continue;
            }

            let dependencies = callable_dependencies(project, &callable);
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

    let packages = reachable_packages(root, &reachable, &reachable_items);

    Ok(ReducedProject {
        root: root.clone(),
        packages,
        reachable,
        reachable_items,
    })
}

pub fn is_opensourced_attr(path: &Path) -> bool {
    path.segments
        .last()
        .is_some_and(|segment| segment.ident == "opensourced")
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
        .parse_args::<syn::Ident>()
        .is_ok_and(|ident| ident == "test")
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
    root: &CallableId,
    reachable: &BTreeSet<CallableId>,
    reachable_items: &BTreeSet<ItemId>,
) -> BTreeSet<String> {
    let mut packages = BTreeSet::from([root.package().to_string()]);
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
            visitor.visit_signature(&record.item.sig);
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
            if let Some(item) = visitor.resolver.resolve_local_type_item(type_path) {
                visitor.dependencies.items.insert(item);
            }
            if let Some(trait_path) = trait_path {
                if let Some(item) = visitor.resolver.resolve_local_trait_item(trait_path) {
                    visitor.dependencies.items.insert(item);
                }
            }
            visitor.visit_impl_peers(callable, record, trait_path.is_some());
            visitor.visit_signature(&record.item.sig);
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

    let resolver = Resolver {
        project,
        package: &record.package,
        module_path: &record.module_path,
        aliases: &record.aliases,
        self_type: None,
    };
    let mut visitor = DependencyVisitor::new(resolver);
    visitor.visit_item(&record.item);
    visitor.dependencies.items.remove(item);
    visitor.dependencies
}

#[derive(Default)]
struct DependencySet {
    callables: BTreeSet<CallableId>,
    items: BTreeSet<ItemId>,
}

struct DependencyVisitor<'a> {
    resolver: Resolver<'a>,
    dependencies: DependencySet,
    variables: HashMap<String, TypeRef>,
}

impl<'a> DependencyVisitor<'a> {
    fn new(resolver: Resolver<'a>) -> Self {
        Self {
            resolver,
            dependencies: DependencySet::default(),
            variables: HashMap::new(),
        }
    }

    fn add_fn_inputs(&mut self, inputs: &syn::punctuated::Punctuated<FnArg, syn::token::Comma>) {
        for input in inputs {
            let FnArg::Typed(input) = input else {
                continue;
            };
            let Pat::Ident(ident) = input.pat.as_ref() else {
                continue;
            };
            if let Some(type_ref) = self.resolver.resolve_type(&input.ty) {
                self.variables.insert(ident.ident.to_string(), type_ref);
            }
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
                self.variables.get(&name).cloned()
            }
            Expr::Call(call) => {
                if let Expr::Path(path) = call.func.as_ref() {
                    self.resolver.type_from_expr_path_call(path)
                } else {
                    None
                }
            }
            Expr::Struct(expr) => self.resolver.resolve_type_path(&expr.path),
            Expr::Reference(reference) => self.receiver_type(&reference.expr),
            Expr::Paren(paren) => self.receiver_type(&paren.expr),
            _ => None,
        }
    }

    fn local_binding_type(&self, local: &Local) -> Option<(String, TypeRef)> {
        let (name, explicit_type) = binding_name_and_type(&local.pat)?;
        if let Some(ty) = explicit_type {
            if let Some(type_ref) = self.resolver.resolve_type(ty) {
                return Some((name, type_ref));
            }
        }

        let init = local.init.as_ref()?;
        let type_ref = self.infer_expr_type(&init.expr)?;
        Some((name, type_ref))
    }

    fn infer_expr_type(&self, expression: &Expr) -> Option<TypeRef> {
        match expression {
            Expr::Call(call) => {
                if let Expr::Path(path) = call.func.as_ref() {
                    self.resolver.type_from_expr_path_call(path)
                } else {
                    None
                }
            }
            Expr::Struct(expr) => self.resolver.resolve_type_path(&expr.path),
            _ => None,
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
        if !tuple
            .path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "Ok")
        {
            return;
        }
        let Some(Pat::Ident(ident)) = tuple.elems.first() else {
            return;
        };
        self.variables
            .insert(ident.ident.to_string(), type_ref.clone());
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
}

impl<'ast> Visit<'ast> for DependencyVisitor<'_> {
    fn visit_local(&mut self, local: &'ast Local) {
        if let Some((name, type_ref)) = self.local_binding_type(local) {
            self.variables.insert(name, type_ref);
        }
        visit::visit_local(self, local);
    }

    fn visit_expr_call(&mut self, call: &'ast ExprCall) {
        if let Expr::Path(path) = call.func.as_ref() {
            self.add_expr_path_call(path);
            if path.qself.is_none() {
                if let Some(first_arg) = call.args.first() {
                    if let Some(receiver) = self.receiver_type(first_arg) {
                        for callable in self.resolver.resolve_trait_call(&path.path, &receiver) {
                            self.dependencies.callables.insert(callable);
                        }
                    }
                }
            }
        }
        visit::visit_expr_call(self, call);
    }

    fn visit_expr_match(&mut self, expr_match: &'ast ExprMatch) {
        self.visit_expr(&expr_match.expr);
        let ok_type = self.result_ok_type(&expr_match.expr);

        for arm in &expr_match.arms {
            let variables = self.variables.clone();
            if let Some(ok_type) = &ok_type {
                self.add_result_ok_binding_type(&arm.pat, ok_type);
            }
            self.visit_pat(&arm.pat);
            if let Some((_, guard)) = &arm.guard {
                self.visit_expr(guard);
            }
            self.visit_expr(&arm.body);
            self.variables = variables;
        }
    }

    fn visit_expr_macro(&mut self, expr: &'ast ExprMacro) {
        self.add_macro_path(&expr.mac.path);
        visit::visit_expr_macro(self, expr);
    }

    fn visit_item_macro(&mut self, item: &'ast ItemMacro) {
        if let Some(ident) = &item.ident {
            let id = ItemId {
                package: self.resolver.package.to_string(),
                module_path: self.resolver.module_path.to_vec(),
                name: ident.to_string(),
                kind: ItemKind::Macro,
            };
            self.dependencies.items.insert(id);
            self.add_macro_token_dependencies(&item.mac.tokens);
        }
        visit::visit_item_macro(self, item);
    }

    fn visit_macro(&mut self, mac: &'ast Macro) {
        self.add_macro_path(&mac.path);
        visit::visit_macro(self, mac);
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        if let Some(receiver) = self.receiver_type(&call.receiver) {
            for callable in self
                .resolver
                .resolve_methods(&receiver, &call.method.to_string())
            {
                self.dependencies.callables.insert(callable);
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
            }
        }
        visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_path(&mut self, path: &'ast ExprPath) {
        self.add_expr_path_call(path);
        self.add_item_path(&path.path);
        visit::visit_expr_path(self, path);
    }

    fn visit_type_path(&mut self, ty: &'ast TypePath) {
        self.add_item_path(&ty.path);
        visit::visit_type_path(self, ty);
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

    fn add_macro_token_dependencies(&mut self, tokens: &TokenStream) {
        for segments in token_path_candidates(tokens) {
            let Ok(path) = syn::parse_str::<Path>(&segments.join("::")) else {
                continue;
            };
            self.add_call_path(&path);
            self.add_item_path(&path);
            self.add_macro_path(&path);
        }
    }
}

#[derive(Clone, Debug)]
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

        let alias = self.resolve_alias_target(&package, &module_path, &name)?;
        let id = CallableId::Free {
            package: alias.package,
            module_path: alias.module_path,
            name: alias.name,
        };
        self.project.functions.contains_key(&id).then_some(id)
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
        let inherent = CallableId::Method {
            package: receiver.package.clone(),
            type_path: receiver.type_path.clone(),
            trait_path: None,
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
            } = id
            else {
                return None;
            };

            (package == &receiver.package
                && type_path == &receiver.type_path
                && candidate_method == method)
                .then(|| id.clone())
        }));

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

    fn type_from_associated_call(&self, path: &Path) -> Option<TypeRef> {
        match self.resolve_associated_method(path)? {
            CallableId::Method {
                package, type_path, ..
            } => Some(TypeRef { package, type_path }),
            CallableId::Free { .. } => None,
        }
    }

    fn type_from_expr_path_call(&self, path: &ExprPath) -> Option<TypeRef> {
        if let Some(qself) = &path.qself {
            return self.resolve_type(&qself.ty);
        }
        self.type_from_associated_call(&path.path)
    }

    fn resolve_type(&self, ty: &Type) -> Option<TypeRef> {
        match ty {
            Type::Path(type_path) => self.resolve_type_path(&type_path.path),
            Type::Reference(reference) => self.resolve_type(&reference.elem),
            _ => None,
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
                (package == &receiver.package
                    && method == method_name
                    && trait_path
                        .last()
                        .is_some_and(|candidate| candidate == trait_name)
                    && record
                        .trait_input_type_paths
                        .iter()
                        .any(|type_path| type_path == &receiver.type_path))
                .then(|| id.clone())
            })
            .collect()
    }

    fn resolve_type_path(&self, path: &Path) -> Option<TypeRef> {
        let segments = self.apply_alias(path_segments(path));
        self.resolve_type_segments(&segments)
    }

    fn resolve_type_segments(&self, segments: &[String]) -> Option<TypeRef> {
        let item = self.resolve_item_segments(segments)?;
        matches!(
            item.kind,
            ItemKind::Struct | ItemKind::Enum | ItemKind::Union | ItemKind::Type | ItemKind::Trait
        )
        .then(|| TypeRef {
            package: item.package.clone(),
            type_path: path_from_item(&item),
        })
    }

    fn resolve_local_type_item(&self, type_path: &[String]) -> Option<ItemId> {
        self.find_item(self.package, type_path, &type_like_kinds())
    }

    fn resolve_local_trait_item(&self, trait_path: &[String]) -> Option<ItemId> {
        self.find_item(self.package, trait_path, &[ItemKind::Trait])
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
        None
    }

    fn find_item(&self, package: &str, path: &[String], kinds: &[ItemKind]) -> Option<ItemId> {
        if path.is_empty() {
            return None;
        }
        let name = path.last()?.clone();
        let module_path = path[..path.len() - 1].to_vec();
        if let Some(item) = kinds.iter().find_map(|kind| {
            let id = ItemId {
                package: package.to_string(),
                module_path: module_path.clone(),
                name: name.clone(),
                kind: *kind,
            };
            self.project.items.contains_key(&id).then_some(id)
        }) {
            return Some(item);
        }

        let alias = self.resolve_alias_target(package, &module_path, &name)?;
        self.find_item(&alias.package, &alias.full_path(), kinds)
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
            .find(|dependency| dependency.alias == first || dependency.package == first)
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

fn path_from_item(item: &ItemId) -> Vec<String> {
    let mut path = item.module_path.clone();
    path.push(item.name.clone());
    path
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
