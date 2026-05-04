use std::path::PathBuf;

use proc_macro2::{TokenStream, TokenTree};
use quote::ToTokens;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum StaticIncludePath {
    SourceRelative(PathBuf),
    PackageRelative(PathBuf),
    Absolute(PathBuf),
}

enum IncludePathPart {
    Text(String),
    CargoManifestDir,
}

pub fn static_include_path(tokens: &TokenStream) -> Option<StaticIncludePath> {
    if let Some(value) = single_string_literal(tokens) {
        return static_path_from_value(&value);
    }

    let parts = concat_macro_parts(tokens)?;
    static_path_from_parts(&parts)
}

fn static_path_from_value(value: &str) -> Option<StaticIncludePath> {
    if value.is_empty() || value.contains('\0') {
        return None;
    }
    let path = PathBuf::from(value);
    if path.is_absolute() {
        Some(StaticIncludePath::Absolute(path))
    } else {
        Some(StaticIncludePath::SourceRelative(path))
    }
}

fn static_path_from_parts(parts: &[IncludePathPart]) -> Option<StaticIncludePath> {
    let cargo_manifest_dir_count = parts
        .iter()
        .filter(|part| matches!(part, IncludePathPart::CargoManifestDir))
        .count();
    if cargo_manifest_dir_count == 0 {
        let mut value = String::new();
        for part in parts {
            let IncludePathPart::Text(text) = part else {
                return None;
            };
            value.push_str(text);
        }
        return static_path_from_value(&value);
    }
    if cargo_manifest_dir_count != 1 {
        return None;
    }

    let mut seen_manifest_dir = false;
    let mut suffix = String::new();
    for part in parts {
        match part {
            IncludePathPart::Text(text) if seen_manifest_dir => suffix.push_str(text),
            IncludePathPart::Text(text) if text.is_empty() => {}
            IncludePathPart::Text(_) => return None,
            IncludePathPart::CargoManifestDir if seen_manifest_dir => return None,
            IncludePathPart::CargoManifestDir => seen_manifest_dir = true,
        }
    }

    let suffix = suffix
        .strip_prefix('/')
        .or_else(|| suffix.strip_prefix('\\'))
        .unwrap_or(&suffix);
    if suffix.is_empty() || suffix.contains('\0') {
        return None;
    }
    Some(StaticIncludePath::PackageRelative(PathBuf::from(suffix)))
}

fn concat_macro_parts(tokens: &TokenStream) -> Option<Vec<IncludePathPart>> {
    let mut tokens = tokens.clone().into_iter();
    let Some(TokenTree::Ident(ident)) = tokens.next() else {
        return None;
    };
    if ident != "concat" {
        return None;
    }
    let Some(TokenTree::Punct(punct)) = tokens.next() else {
        return None;
    };
    if punct.as_char() != '!' {
        return None;
    }
    let Some(TokenTree::Group(group)) = tokens.next() else {
        return None;
    };
    if tokens.next().is_some() {
        return None;
    }

    split_comma_separated(&group.stream())
        .into_iter()
        .map(|part| include_path_part(&part))
        .collect()
}

fn split_comma_separated(tokens: &TokenStream) -> Vec<TokenStream> {
    let mut parts = Vec::new();
    let mut current = Vec::new();
    for token in tokens.clone() {
        if matches!(&token, TokenTree::Punct(punct) if punct.as_char() == ',') {
            parts.push(current.into_iter().collect());
            current = Vec::new();
            continue;
        }
        current.push(token);
    }
    parts.push(current.into_iter().collect());
    parts
}

fn include_path_part(tokens: &TokenStream) -> Option<IncludePathPart> {
    if let Some(value) = single_string_literal(tokens) {
        return Some(IncludePathPart::Text(value));
    }
    let env_var = env_macro_argument(tokens)?;
    (env_var == "CARGO_MANIFEST_DIR").then_some(IncludePathPart::CargoManifestDir)
}

fn env_macro_argument(tokens: &TokenStream) -> Option<String> {
    let mut tokens = tokens.clone().into_iter();
    let Some(TokenTree::Ident(ident)) = tokens.next() else {
        return None;
    };
    if ident != "env" {
        return None;
    }
    let Some(TokenTree::Punct(punct)) = tokens.next() else {
        return None;
    };
    if punct.as_char() != '!' {
        return None;
    }
    let Some(TokenTree::Group(group)) = tokens.next() else {
        return None;
    };
    if tokens.next().is_some() {
        return None;
    }
    single_string_literal(&group.stream())
}

fn single_string_literal(tokens: &TokenStream) -> Option<String> {
    let mut tokens = tokens.clone().into_iter();
    let Some(TokenTree::Literal(literal)) = tokens.next() else {
        return None;
    };
    if tokens.next().is_some() {
        return None;
    }
    syn::parse2::<syn::LitStr>(literal.to_token_stream())
        .ok()
        .map(|literal| literal.value())
}
