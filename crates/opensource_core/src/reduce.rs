use std::collections::{BTreeSet, HashMap, VecDeque};

use syn::{
    visit::{self, Visit},
    Expr, ExprCall, ExprMethodCall, ExprPath, Local, Pat, Path, Type,
};

use crate::model::{CallableId, Project, ReducedProject};

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

    let packages = package_closure(project, root.package());
    let mut reachable = BTreeSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(root.clone());

    while let Some(callable) = queue.pop_front() {
        if !packages.contains(callable.package()) || !reachable.insert(callable.clone()) {
            continue;
        }

        for dependency in callable_dependencies(project, &callable) {
            if packages.contains(dependency.package()) && !reachable.contains(&dependency) {
                queue.push_back(dependency);
            }
        }
    }

    Ok(ReducedProject {
        root: root.clone(),
        packages,
        reachable,
    })
}

pub fn is_opensourced_attr(path: &Path) -> bool {
    path.segments
        .last()
        .is_some_and(|segment| segment.ident == "opensourced")
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

fn callable_dependencies(project: &Project, callable: &CallableId) -> BTreeSet<CallableId> {
    match callable {
        CallableId::Free { .. } => {
            let Some(record) = project.functions.get(callable) else {
                return BTreeSet::new();
            };
            let resolver = Resolver {
                project,
                package: &record.package,
                module_path: &record.module_path,
                aliases: &record.aliases,
                self_type: None,
            };
            let mut visitor = DependencyVisitor::new(resolver);
            visitor.visit_block(&record.item.block);
            visitor.dependencies
        }
        CallableId::Method {
            package, type_path, ..
        } => {
            let Some(record) = project.methods.get(callable) else {
                return BTreeSet::new();
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
            visitor.visit_block(&record.item.block);
            visitor.dependencies
        }
    }
}

struct DependencyVisitor<'a> {
    resolver: Resolver<'a>,
    dependencies: BTreeSet<CallableId>,
    variables: HashMap<String, TypeRef>,
}

impl<'a> DependencyVisitor<'a> {
    fn new(resolver: Resolver<'a>) -> Self {
        Self {
            resolver,
            dependencies: BTreeSet::new(),
            variables: HashMap::new(),
        }
    }

    fn add_path_call(&mut self, path: &Path) {
        if let Some(callable) = self.resolver.resolve_call_path(path) {
            self.dependencies.insert(callable);
        }
    }

    fn receiver_type(&self, expression: &Expr) -> Option<TypeRef> {
        match expression {
            Expr::Path(path) if path.path.segments.len() == 1 => {
                let name = path.path.segments.first()?.ident.to_string();
                self.variables.get(&name).cloned()
            }
            Expr::Call(call) => {
                if let Expr::Path(path) = call.func.as_ref() {
                    self.resolver.type_from_associated_call(&path.path)
                } else {
                    None
                }
            }
            Expr::Struct(expr) => self.resolver.resolve_type_path(&expr.path),
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
                    self.resolver.type_from_associated_call(&path.path)
                } else {
                    None
                }
            }
            Expr::Struct(expr) => self.resolver.resolve_type_path(&expr.path),
            _ => None,
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
            self.add_path_call(&path.path);
        }
        visit::visit_expr_call(self, call);
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        if let Some(receiver) = self.receiver_type(&call.receiver) {
            if let Some(callable) = self
                .resolver
                .resolve_method(&receiver, &call.method.to_string())
            {
                self.dependencies.insert(callable);
            }
        }
        visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_path(&mut self, path: &'ast ExprPath) {
        self.add_path_call(&path.path);
        visit::visit_expr_path(self, path);
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
    fn resolve_call_path(&self, path: &Path) -> Option<CallableId> {
        self.resolve_free_function(path)
            .or_else(|| self.resolve_associated_method(path))
    }

    fn resolve_free_function(&self, path: &Path) -> Option<CallableId> {
        let segments = self.apply_alias(path_segments(path));
        if segments.is_empty() {
            return None;
        }

        let name = segments.last()?.clone();
        let (package, module_path) = self.resolve_value_prefix(&segments[..segments.len() - 1])?;
        let id = CallableId::Free {
            package,
            module_path,
            name,
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
            let (package, type_path) = self.resolve_type_prefix(receiver_segments)?;
            TypeRef { package, type_path }
        };

        self.resolve_method(&receiver, &method)
    }

    fn resolve_method(&self, receiver: &TypeRef, method: &str) -> Option<CallableId> {
        let id = CallableId::Method {
            package: receiver.package.clone(),
            type_path: receiver.type_path.clone(),
            method: method.to_string(),
        };

        self.project.methods.contains_key(&id).then_some(id)
    }

    fn type_from_associated_call(&self, path: &Path) -> Option<TypeRef> {
        match self.resolve_associated_method(path)? {
            CallableId::Method {
                package, type_path, ..
            } => Some(TypeRef { package, type_path }),
            CallableId::Free { .. } => None,
        }
    }

    fn resolve_type(&self, ty: &Type) -> Option<TypeRef> {
        let Type::Path(type_path) = ty else {
            return None;
        };
        self.resolve_type_path(&type_path.path)
    }

    fn resolve_type_path(&self, path: &Path) -> Option<TypeRef> {
        let segments = self.apply_alias(path_segments(path));
        let (package, type_path) = self.resolve_type_prefix(&segments)?;
        Some(TypeRef { package, type_path })
    }

    fn resolve_value_prefix(&self, prefix: &[String]) -> Option<(String, Vec<String>)> {
        if prefix.is_empty() {
            return Some((self.package.to_string(), self.module_path.to_vec()));
        }

        self.resolve_prefix(prefix)
    }

    fn resolve_type_prefix(&self, prefix: &[String]) -> Option<(String, Vec<String>)> {
        if prefix.is_empty() {
            return None;
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
}

fn path_segments(path: &Path) -> Vec<String> {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect()
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
