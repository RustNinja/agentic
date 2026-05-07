use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::feedback::{CheckDiagnostic, CheckSpan, CheckSuggestion};

#[derive(Debug, Clone)]
pub struct RepairOptions {
    pub output_root: PathBuf,
    pub diagnostics: Vec<CheckDiagnostic>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct RepairReport {
    pub removed_items: usize,
    pub removed_imports: usize,
    #[serde(default)]
    pub normalized_paths: usize,
    #[serde(default)]
    pub applied_suggestions: usize,
    pub added_dead_code_allows: usize,
    #[serde(default)]
    pub added_lint_allows: usize,
    pub deferred_dead_code_allows: usize,
    #[serde(default)]
    pub deferred_lint_allows: usize,
    pub skipped_diagnostics: usize,
    #[serde(default)]
    pub changed_files: Vec<RepairFileChange>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct RepairFileChange {
    pub path: PathBuf,
    pub removed_items: usize,
    pub removed_imports: usize,
    #[serde(default)]
    pub normalized_paths: usize,
    #[serde(default)]
    pub applied_suggestions: usize,
    pub added_dead_code_allows: usize,
    #[serde(default)]
    pub added_lint_allows: usize,
}

impl RepairReport {
    pub fn total_changes(&self) -> usize {
        self.removed_items
            + self.removed_imports
            + self.normalized_paths
            + self.applied_suggestions
            + self.added_dead_code_allows
            + self.added_lint_allows
    }
}

pub fn repair_workspace(
    options: RepairOptions,
) -> Result<RepairReport, Box<dyn std::error::Error>> {
    let mut item_candidates_by_file = BTreeMap::<PathBuf, Vec<DeadItemCandidate>>::new();
    let mut import_candidates_by_file = BTreeMap::<PathBuf, Vec<ImportSpanCandidate>>::new();
    let mut syntax_candidates_by_file = BTreeMap::<PathBuf, Vec<PathSyntaxCandidate>>::new();
    let mut suggestion_candidates_by_file = BTreeMap::<PathBuf, Vec<SuggestionCandidate>>::new();
    let mut allow_candidates_by_file = BTreeMap::<PathBuf, Vec<AllowDeadCodeCandidate>>::new();
    let mut lint_allow_candidates_by_file = BTreeMap::<PathBuf, Vec<AllowLintCandidate>>::new();
    let mut skipped_diagnostics = 0;

    for diagnostic in &options.diagnostics {
        if diagnostic.level == "error" {
            let mut repaired = false;
            for suggestion in &diagnostic.suggestions {
                let Some(path) = diagnostic_path(&options.output_root, &suggestion.file_name)
                else {
                    continue;
                };
                if let Some(candidate) = suggestion_candidate(suggestion) {
                    suggestion_candidates_by_file
                        .entry(path)
                        .or_default()
                        .push(candidate);
                    repaired = true;
                }
            }
            for span in diagnostic.spans.iter().filter(|span| span.is_primary) {
                let Some(path) = diagnostic_path(&options.output_root, &span.file_name) else {
                    continue;
                };
                if let Some(candidate) = path_syntax_candidate(diagnostic, span) {
                    syntax_candidates_by_file
                        .entry(path)
                        .or_default()
                        .push(candidate);
                    repaired = true;
                }
            }
            if !repaired {
                skipped_diagnostics += 1;
            }
            continue;
        }

        if diagnostic.level != "warning" {
            skipped_diagnostics += 1;
            continue;
        }

        let code = diagnostic.code.as_deref();
        if !matches!(
            code,
            Some("dead_code" | "unused_imports" | "unused_macros" | "private_interfaces")
        ) {
            skipped_diagnostics += 1;
            continue;
        }

        let mut repaired = false;
        if code == Some("unused_imports") {
            for suggestion in diagnostic
                .suggestions
                .iter()
                .filter(|suggestion| unused_import_suggestion_removes_whole_use(suggestion))
            {
                let Some(path) = diagnostic_path(&options.output_root, &suggestion.file_name)
                else {
                    continue;
                };
                import_candidates_by_file
                    .entry(path)
                    .or_default()
                    .push(ImportSpanCandidate {
                        line_start: suggestion.line_start as usize,
                        column_start: suggestion.column_start as usize,
                        column_end: suggestion.column_end as usize,
                        remove_line: true,
                        message: diagnostic.message.clone(),
                    });
                repaired = true;
            }
        }
        for span in diagnostic.spans.iter().filter(|span| span.is_primary) {
            let Some(path) = diagnostic_path(&options.output_root, &span.file_name) else {
                continue;
            };

            match code {
                Some("unused_imports") => {
                    import_candidates_by_file
                        .entry(path)
                        .or_default()
                        .push(ImportSpanCandidate {
                            line_start: span.line_start as usize,
                            column_start: span.column_start as usize,
                            column_end: span.column_end as usize,
                            remove_line: unused_import_span_removes_whole_use(
                                span_text(span).unwrap_or_default(),
                                &diagnostic.message,
                            ),
                            message: diagnostic.message.clone(),
                        });
                    repaired = true;
                }
                Some("dead_code") => {
                    if let Some(name) = dead_candidate_name(&diagnostic.message, span) {
                        item_candidates_by_file
                            .entry(path)
                            .or_default()
                            .push(DeadItemCandidate {
                                name,
                                line_start: span.line_start as usize,
                            });
                        repaired = true;
                    } else if dead_code_message_is_type_member(&diagnostic.message) {
                        allow_candidates_by_file.entry(path).or_default().push(
                            AllowDeadCodeCandidate {
                                line_start: span.line_start as usize,
                            },
                        );
                        repaired = true;
                    }
                }
                Some("unused_macros") => {
                    if let Some(name) = unused_macro_candidate_name(&diagnostic.message) {
                        item_candidates_by_file
                            .entry(path)
                            .or_default()
                            .push(DeadItemCandidate {
                                name,
                                line_start: span.line_start as usize,
                            });
                        repaired = true;
                    }
                }
                Some("private_interfaces") => {
                    lint_allow_candidates_by_file.entry(path).or_default().push(
                        AllowLintCandidate {
                            line_start: span.line_start as usize,
                            lint: "private_interfaces".to_string(),
                        },
                    );
                    repaired = true;
                }
                _ => {}
            }
        }

        if !repaired {
            skipped_diagnostics += 1;
        }
    }

    let mut report = RepairReport {
        skipped_diagnostics,
        ..RepairReport::default()
    };
    let has_structural_repairs = !syntax_candidates_by_file.is_empty()
        || !suggestion_candidates_by_file.is_empty()
        || !item_candidates_by_file.is_empty()
        || !import_candidates_by_file.is_empty();
    let mut suggestion_changed_files = BTreeSet::new();
    for (path, mut candidates) in suggestion_candidates_by_file {
        if !path.exists() {
            continue;
        }
        candidates.sort_by(|left, right| {
            right
                .byte_start
                .cmp(&left.byte_start)
                .then_with(|| right.line_start.cmp(&left.line_start))
                .then_with(|| right.column_start.cmp(&left.column_start))
                .then_with(|| right.replacement.cmp(&left.replacement))
        });
        candidates.dedup();
        let mut source = fs::read_to_string(&path)?;
        let applied_suggestions = apply_suggestions(&mut source, &candidates);
        if applied_suggestions > 0 {
            report.applied_suggestions += applied_suggestions;
            suggestion_changed_files.insert(path.clone());
            record_file_change(&mut report, &path, 0, 0, 0, applied_suggestions, 0, 0);
            fs::write(path, source)?;
        }
    }
    for (path, mut candidates) in syntax_candidates_by_file {
        if !path.exists() {
            continue;
        }
        if suggestion_changed_files.contains(&path) {
            continue;
        }
        candidates.sort_by(|left, right| {
            right
                .line_start
                .cmp(&left.line_start)
                .then_with(|| right.column_start.cmp(&left.column_start))
                .then_with(|| right.kind.cmp(&left.kind))
        });
        candidates.dedup();
        drop_candidates_covered_by_malformed_use_removals(&mut candidates);
        let mut source = fs::read_to_string(&path)?;
        let mut normalized_paths = 0;
        for candidate in candidates {
            if repair_path_syntax(&mut source, &candidate) {
                report.normalized_paths += 1;
                normalized_paths += 1;
            }
        }
        if normalized_paths > 0 {
            record_file_change(&mut report, &path, 0, 0, normalized_paths, 0, 0, 0);
        }
        fs::write(path, source)?;
    }
    for (path, mut candidates) in import_candidates_by_file {
        if !path.exists() {
            continue;
        }
        if suggestion_changed_files.contains(&path) {
            continue;
        }
        candidates.sort_by(|left, right| {
            right
                .line_start
                .cmp(&left.line_start)
                .then_with(|| right.column_start.cmp(&left.column_start))
        });
        candidates.dedup();
        let mut source = fs::read_to_string(&path)?;
        candidates = promote_fully_pruned_multiline_use_groups(&source, candidates);
        candidates = filter_import_candidates_covered_by_whole_use_removals(&source, candidates);
        candidates = dedup_whole_use_removal_candidates(&source, candidates);
        candidates.sort_by(|left, right| {
            right
                .line_start
                .cmp(&left.line_start)
                .then_with(|| right.column_start.cmp(&left.column_start))
        });
        candidates.dedup();
        let mut removed_imports = 0;
        for candidate in candidates {
            if remove_import_span(&mut source, &candidate) {
                report.removed_imports += 1;
                removed_imports += 1;
            }
        }
        if removed_imports > 0 {
            record_file_change(&mut report, &path, 0, removed_imports, 0, 0, 0, 0);
        }
        fs::write(path, source)?;
    }
    for (path, mut candidates) in item_candidates_by_file {
        if !path.exists() {
            continue;
        }
        if suggestion_changed_files.contains(&path) {
            continue;
        }
        candidates.sort_by(|left, right| right.line_start.cmp(&left.line_start));
        candidates.dedup();
        let mut source = fs::read_to_string(&path)?;
        let mut removed_items = 0;
        for candidate in candidates {
            if remove_item_at_line(&mut source, &candidate) {
                report.removed_items += 1;
                removed_items += 1;
            }
        }
        if removed_items > 0 {
            record_file_change(&mut report, &path, removed_items, 0, 0, 0, 0, 0);
        }
        fs::write(path, source)?;
    }

    if has_structural_repairs {
        report.deferred_dead_code_allows = allow_candidates_by_file
            .values()
            .map(std::vec::Vec::len)
            .sum();
        report.deferred_lint_allows = lint_allow_candidates_by_file
            .values()
            .map(std::vec::Vec::len)
            .sum();
    } else {
        for (path, mut candidates) in allow_candidates_by_file {
            if !path.exists() {
                continue;
            }
            candidates.sort_by(|left, right| right.line_start.cmp(&left.line_start));
            candidates.dedup();
            let mut source = fs::read_to_string(&path)?;
            let mut added_allows = 0;
            for candidate in candidates {
                if add_dead_code_allow_for_enclosing_type(&mut source, &candidate) {
                    report.added_dead_code_allows += 1;
                    added_allows += 1;
                }
            }
            if added_allows > 0 {
                record_file_change(&mut report, &path, 0, 0, 0, 0, added_allows, 0);
            }
            fs::write(path, source)?;
        }
        for (path, mut candidates) in lint_allow_candidates_by_file {
            if !path.exists() {
                continue;
            }
            candidates.sort_by(|left, right| {
                right
                    .line_start
                    .cmp(&left.line_start)
                    .then_with(|| right.lint.cmp(&left.lint))
            });
            candidates.dedup();
            let mut source = fs::read_to_string(&path)?;
            let mut added_allows = 0;
            for candidate in candidates {
                if add_lint_allow_before_line(&mut source, &candidate) {
                    report.added_lint_allows += 1;
                    added_allows += 1;
                }
            }
            if added_allows > 0 {
                record_file_change(&mut report, &path, 0, 0, 0, 0, 0, added_allows);
            }
            fs::write(path, source)?;
        }
    }

    Ok(report)
}

fn record_file_change(
    report: &mut RepairReport,
    path: &Path,
    removed_items: usize,
    removed_imports: usize,
    normalized_paths: usize,
    applied_suggestions: usize,
    added_dead_code_allows: usize,
    added_lint_allows: usize,
) {
    if let Some(change) = report
        .changed_files
        .iter_mut()
        .find(|change| change.path == path)
    {
        change.removed_items += removed_items;
        change.removed_imports += removed_imports;
        change.normalized_paths += normalized_paths;
        change.applied_suggestions += applied_suggestions;
        change.added_dead_code_allows += added_dead_code_allows;
        change.added_lint_allows += added_lint_allows;
        return;
    }
    report.changed_files.push(RepairFileChange {
        path: path.to_path_buf(),
        removed_items,
        removed_imports,
        normalized_paths,
        applied_suggestions,
        added_dead_code_allows,
        added_lint_allows,
    });
}

pub fn write_repair_report(
    report: &RepairReport,
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(report)?)?;
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DeadItemCandidate {
    name: String,
    line_start: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ImportSpanCandidate {
    line_start: usize,
    column_start: usize,
    column_end: usize,
    remove_line: bool,
    message: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AllowDeadCodeCandidate {
    line_start: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AllowLintCandidate {
    line_start: usize,
    lint: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PathSyntaxCandidate {
    kind: PathSyntaxRepairKind,
    line_start: usize,
    column_start: usize,
    column_end: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SuggestionCandidate {
    line_start: usize,
    line_end: usize,
    column_start: usize,
    column_end: usize,
    byte_start: Option<usize>,
    byte_end: Option<usize>,
    replacement: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SuggestionEdit {
    start: usize,
    end: usize,
    replacement: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum PathSyntaxRepairKind {
    MalformedUseRoot,
    RepeatedSeparator,
    AbsoluteTypeRoot,
}

fn diagnostic_path(output_root: &Path, file_name: &str) -> Option<PathBuf> {
    let path = PathBuf::from(file_name);
    if path.is_absolute() {
        path_is_inside_output_root(&path, output_root).then_some(path)
    } else {
        Some(output_root.join(path))
    }
}

fn path_is_inside_output_root(path: &Path, output_root: &Path) -> bool {
    let root = canonicalize_existing_prefix(output_root);
    let candidate = canonicalize_existing_prefix(path);
    candidate.starts_with(root)
}

fn canonicalize_existing_prefix(path: &Path) -> PathBuf {
    if let Ok(canonical) = path.canonicalize() {
        return canonical;
    }
    path.parent()
        .map(canonicalize_existing_prefix)
        .map(|parent| {
            path.file_name()
                .map(|name| parent.join(name))
                .unwrap_or(parent)
        })
        .unwrap_or_else(|| path.to_path_buf())
}

fn suggestion_candidate(suggestion: &CheckSuggestion) -> Option<SuggestionCandidate> {
    if suggestion.suggestion_applicability.as_deref() != Some("MachineApplicable") {
        return None;
    }
    if suggestion.suggested_replacement.len() > 256 * 1024 {
        return None;
    }
    if suggestion.suggested_replacement.contains('\0') {
        return None;
    }
    Some(SuggestionCandidate {
        line_start: suggestion.line_start.try_into().ok()?,
        line_end: suggestion.line_end.try_into().ok()?,
        column_start: suggestion.column_start.try_into().ok()?,
        column_end: suggestion.column_end.try_into().ok()?,
        byte_start: suggestion
            .byte_start
            .and_then(|value| value.try_into().ok()),
        byte_end: suggestion.byte_end.and_then(|value| value.try_into().ok()),
        replacement: suggestion.suggested_replacement.clone(),
    })
}

fn apply_suggestions(source: &mut String, candidates: &[SuggestionCandidate]) -> usize {
    let mut edits = candidates
        .iter()
        .filter_map(|candidate| suggestion_edit(source, candidate))
        .filter(|edit| source.get(edit.start..edit.end) != Some(edit.replacement.as_str()))
        .collect::<Vec<_>>();
    edits.sort_by(|left, right| {
        left.start
            .cmp(&right.start)
            .then_with(|| left.end.cmp(&right.end))
            .then_with(|| left.replacement.cmp(&right.replacement))
    });
    edits.dedup();

    let mut accepted = Vec::new();
    let mut previous_range = None::<(usize, usize)>;
    for edit in edits {
        if previous_range.is_some_and(|(start, end)| {
            edit.start < end || (edit.start == start && edit.end == end)
        }) {
            continue;
        }
        previous_range = Some((edit.start, edit.end));
        accepted.push(edit);
    }

    let applied = accepted.len();
    for edit in accepted.into_iter().rev() {
        source.replace_range(edit.start..edit.end, &edit.replacement);
    }
    applied
}

fn suggestion_edit(source: &str, candidate: &SuggestionCandidate) -> Option<SuggestionEdit> {
    let (start, end) = match (candidate.byte_start, candidate.byte_end) {
        (Some(start), Some(end)) => (start, end),
        _ => (
            source_location_to_byte_index(source, candidate.line_start, candidate.column_start)?,
            source_location_to_byte_index(source, candidate.line_end, candidate.column_end)?,
        ),
    };
    if start > end || end > source.len() {
        return None;
    }
    if !source.is_char_boundary(start) || !source.is_char_boundary(end) {
        return None;
    }
    Some(SuggestionEdit {
        start,
        end,
        replacement: candidate.replacement.clone(),
    })
}

fn source_location_to_byte_index(source: &str, line_number: usize, column: usize) -> Option<usize> {
    if line_number == 0 || column == 0 {
        return None;
    }
    let mut offset = 0usize;
    for (index, line) in source.split_inclusive('\n').enumerate() {
        if index + 1 == line_number {
            let line_without_newline = line.strip_suffix('\n').unwrap_or(line);
            let column_index = column_to_byte_index(line_without_newline, column);
            return (column_index <= line_without_newline.len()).then_some(offset + column_index);
        }
        offset += line.len();
    }
    if line_number == source.lines().count() + 1 && column == 1 {
        return Some(source.len());
    }
    None
}

fn dead_candidate_name(message: &str, span: &CheckSpan) -> Option<String> {
    if !dead_code_message_is_item(message) {
        return None;
    }
    name_from_span_text(span).or_else(|| name_from_backticks(message).map(str::to_string))
}

fn unused_macro_candidate_name(message: &str) -> Option<String> {
    message
        .starts_with("unused macro definition")
        .then(|| name_from_backticks(message).map(str::to_string))?
}

fn dead_code_message_is_item(message: &str) -> bool {
    message.contains("function `")
        || message.contains("method `")
        || message.contains("associated function `")
        || message.contains("associated items `")
        || message.contains("associated constant `")
        || message.contains("constant `")
        || message.contains("enum `")
        || message.contains("module `")
        || message.contains("static `")
        || message.contains("struct `")
        || message.contains("type alias `")
        || message.contains("union `")
        || message.contains("trait `")
}

fn dead_code_message_is_type_member(message: &str) -> bool {
    message.contains("field `")
        || message.contains("fields `")
        || message.contains("variant `")
        || message.contains("variants `")
}

fn name_from_backticks(message: &str) -> Option<&str> {
    let (_, rest) = message.split_once('`')?;
    let (name, _) = rest.split_once('`')?;
    Some(name)
}

fn name_from_span_text(span: &CheckSpan) -> Option<String> {
    let text = span_text(span)?.trim_start();
    let mut previous = "";
    for token in text.split_whitespace() {
        if matches!(
            previous,
            "const" | "enum" | "fn" | "mod" | "static" | "struct" | "trait" | "type" | "union"
        ) {
            return clean_ident_token(token);
        }
        previous = token;
    }
    None
}

fn clean_ident_token(token: &str) -> Option<String> {
    let token = token.trim_start_matches("r#");
    let ident = token
        .chars()
        .take_while(|character| character.is_ascii_alphanumeric() || *character == '_')
        .collect::<String>();
    (!ident.is_empty()).then_some(ident)
}

fn span_text(span: &CheckSpan) -> Option<&str> {
    span.text.first().map(String::as_str)
}

fn path_syntax_candidate(
    diagnostic: &CheckDiagnostic,
    span: &CheckSpan,
) -> Option<PathSyntaxCandidate> {
    let text = span_text(span).unwrap_or_default();
    let line_start = span.line_start as usize;
    if line_has_malformed_use_root(text) {
        return Some(PathSyntaxCandidate {
            kind: PathSyntaxRepairKind::MalformedUseRoot,
            line_start,
            column_start: 1,
            column_end: text.chars().count() + 1,
        });
    }

    let column_start = span.column_start as usize;
    let column_end = span.column_end as usize;
    if repeated_path_separator_is_repairable(&diagnostic.message)
        && repeated_path_separator_span(text, column_start).is_some()
    {
        return Some(PathSyntaxCandidate {
            kind: PathSyntaxRepairKind::RepeatedSeparator,
            line_start,
            column_start,
            column_end,
        });
    }

    if absolute_type_root_is_repairable(diagnostic)
        && line_has_absolute_type_root_at_span(text, span)
    {
        return Some(PathSyntaxCandidate {
            kind: PathSyntaxRepairKind::AbsoluteTypeRoot,
            line_start,
            column_start,
            column_end,
        });
    }

    None
}

fn repeated_path_separator_is_repairable(message: &str) -> bool {
    message.contains("path separator must be a double colon")
        || message.contains("expected identifier, found `::`")
}

fn absolute_type_root_is_repairable(diagnostic: &CheckDiagnostic) -> bool {
    matches!(diagnostic.code.as_deref(), Some("E0425" | "E0433"))
        && diagnostic
            .message
            .contains("in the list of imported crates")
}

fn line_has_malformed_use_root(line: &str) -> bool {
    let Some(rest) = use_path_after_prefix(line) else {
        return false;
    };
    let rest = rest.trim_start();
    rest.starts_with(":::")
}

fn use_path_after_prefix(text: &str) -> Option<&str> {
    let trimmed = text.trim_start();
    if let Some(rest) = trimmed.strip_prefix("use ") {
        return Some(rest);
    }
    if let Some(rest) = trimmed
        .strip_prefix("pub ")
        .and_then(|rest| rest.strip_prefix("use "))
    {
        return Some(rest);
    }
    if let Some(rest) = trimmed
        .strip_prefix("pub(crate) ")
        .or_else(|| trimmed.strip_prefix("pub(super) "))
        .and_then(|rest| rest.strip_prefix("use "))
    {
        return Some(rest);
    }
    trimmed
        .strip_prefix("pub(in ")
        .and_then(|rest| rest.split_once(") "))
        .and_then(|(_, rest)| rest.strip_prefix("use "))
}

fn line_has_absolute_type_root_at_span(line: &str, span: &CheckSpan) -> bool {
    let start = column_to_byte_index(line, span.column_start as usize);
    let end = column_to_byte_index(line, span.column_end as usize);
    let Some(identifier) = line.get(start..end).and_then(first_ident_prefix) else {
        return false;
    };
    let identifier = identifier.strip_prefix("r#").unwrap_or(identifier);
    if !identifier
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_uppercase())
    {
        return false;
    }
    if matches!(identifier, "Self" | "Super" | "Crate") {
        return false;
    }
    if start < 2 || line.get(start - 2..start) != Some("::") {
        return false;
    }
    if count_preceding_colons(line, start) != 2 {
        return false;
    }
    let Some(prefix) = line.get(..start - 2) else {
        return false;
    };
    match prefix.chars().next_back() {
        Some(character) => character_can_precede_absolute_root(character),
        None => true,
    }
}

fn first_ident_prefix(text: &str) -> Option<&str> {
    let end = text
        .char_indices()
        .find(|(_, character)| {
            !(character.is_ascii_alphanumeric() || *character == '_' || *character == '#')
        })
        .map(|(index, _)| index)
        .unwrap_or(text.len());
    (end > 0).then_some(&text[..end])
}

fn count_preceding_colons(line: &str, byte_index: usize) -> usize {
    line.as_bytes()[..byte_index]
        .iter()
        .rev()
        .take_while(|byte| **byte == b':')
        .count()
}

fn character_can_precede_absolute_root(character: char) -> bool {
    character.is_whitespace()
        || matches!(
            character,
            '(' | '[' | '{' | '<' | ',' | '=' | ':' | ';' | '&' | '|' | '!' | '?'
        )
}

fn repeated_path_separator_span(line: &str, one_based_column: usize) -> Option<(usize, usize)> {
    let bytes = line.as_bytes();
    if bytes.is_empty() {
        return None;
    }
    let mut cursor = column_to_byte_index(line, one_based_column).min(bytes.len());
    if cursor == bytes.len() || bytes[cursor] != b':' {
        if cursor == 0 || bytes[cursor - 1] != b':' {
            return None;
        }
        cursor -= 1;
    }

    let mut start = cursor;
    while start > 0 && bytes[start - 1] == b':' {
        start -= 1;
    }
    let mut end = cursor;
    while end < bytes.len() && bytes[end] == b':' {
        end += 1;
    }
    (end - start > 2).then_some((start, end))
}

fn drop_candidates_covered_by_malformed_use_removals(candidates: &mut Vec<PathSyntaxCandidate>) {
    let malformed_use_lines = candidates
        .iter()
        .filter(|candidate| candidate.kind == PathSyntaxRepairKind::MalformedUseRoot)
        .map(|candidate| candidate.line_start)
        .collect::<BTreeSet<_>>();
    if malformed_use_lines.is_empty() {
        return;
    }
    candidates.retain(|candidate| {
        candidate.kind == PathSyntaxRepairKind::MalformedUseRoot
            || !malformed_use_lines.contains(&candidate.line_start)
    });
}

fn is_use_statement_start(text: &str) -> bool {
    let trimmed = text.trim_start();
    if trimmed.starts_with("use ") {
        return true;
    }
    if trimmed
        .strip_prefix("pub ")
        .is_some_and(|rest| rest.starts_with("use "))
    {
        return true;
    }
    if trimmed
        .strip_prefix("pub(crate) ")
        .or_else(|| trimmed.strip_prefix("pub(super) "))
        .is_some_and(|rest| rest.starts_with("use "))
    {
        return true;
    }
    trimmed
        .strip_prefix("pub(in ")
        .and_then(|rest| rest.split_once(") "))
        .is_some_and(|(_, rest)| rest.starts_with("use "))
}

fn unused_import_span_removes_whole_use(span_text: &str, message: &str) -> bool {
    if !is_use_statement_start(span_text) {
        return false;
    }

    let import_names = use_statement_leaf_names_from_text(span_text);
    if import_names.len() <= 1 {
        return true;
    }

    let unused_names = unused_import_names_from_message(message);
    !unused_names.is_empty() && import_names.is_subset(&unused_names)
}

fn unused_import_suggestion_removes_whole_use(suggestion: &CheckSuggestion) -> bool {
    suggestion.suggestion_applicability.as_deref() == Some("MachineApplicable")
        && suggestion.suggested_replacement.is_empty()
        && suggestion.message == "remove the whole `use` item"
}

fn remove_item_at_line(source: &mut String, candidate: &DeadItemCandidate) -> bool {
    let mut lines = source.lines().map(str::to_string).collect::<Vec<_>>();
    let Some(mut start) = candidate.line_start.checked_sub(1) else {
        return false;
    };
    if start >= lines.len() {
        return false;
    }

    let needle = candidate.name.clone();
    let raw_needle = format!("r#{}", candidate.name);
    let search_floor = start.saturating_sub(8);
    while start > search_floor
        && !lines[start].contains(&needle)
        && !lines[start].contains(&raw_needle)
    {
        start -= 1;
    }
    if !lines[start].contains(&needle) && !lines[start].contains(&raw_needle) {
        return false;
    }

    while start > 0 {
        let previous = lines[start - 1].trim_start();
        if previous.starts_with("#[") || previous.starts_with("///") {
            start -= 1;
        } else {
            break;
        }
    }

    let Some(header_end) = (start..lines.len()).find(|index| {
        lines[*index].contains('{')
            || lines[*index].trim_end().ends_with(';')
            || lines[*index].trim_end().ends_with(',')
    }) else {
        return false;
    };
    if (lines[header_end].trim_end().ends_with(';') || lines[header_end].trim_end().ends_with(','))
        && !lines[header_end].contains('{')
    {
        lines.drain(start..=header_end);
        *source = join_lines(lines);
        return true;
    }

    let mut depth = 0isize;
    let mut saw_open = false;
    let mut end = None;
    for (index, line) in lines.iter().enumerate().skip(header_end) {
        for character in line.chars() {
            match character {
                '{' => {
                    saw_open = true;
                    depth += 1;
                }
                '}' if saw_open => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(index);
                        break;
                    }
                }
                _ => {}
            }
        }
        if end.is_some() {
            break;
        }
    }
    let Some(end) = end else {
        return false;
    };

    lines.drain(start..=end);
    *source = join_lines(lines);
    true
}

fn remove_import_span(source: &mut String, candidate: &ImportSpanCandidate) -> bool {
    let mut lines = source.lines().map(str::to_string).collect::<Vec<_>>();
    let Some(line_index) = candidate.line_start.checked_sub(1) else {
        return false;
    };

    if candidate.remove_line {
        if !remove_use_statement_at_line(&mut lines, line_index) {
            if line_index >= lines.len() {
                return false;
            }
            lines.remove(line_index);
        }
        *source = join_lines(lines);
        return true;
    }

    let Some(line) = lines.get_mut(line_index) else {
        return false;
    };
    if !remove_column_range(line, candidate.column_start, candidate.column_end) {
        return false;
    }
    *line = cleanup_import_line(line);
    if line.trim().is_empty() || line.contains("::{}") || line.contains("::::") {
        lines.remove(line_index);
    }

    *source = join_lines(lines);
    true
}

fn repair_path_syntax(source: &mut String, candidate: &PathSyntaxCandidate) -> bool {
    let mut lines = source.lines().map(str::to_string).collect::<Vec<_>>();
    let Some(line_index) = candidate.line_start.checked_sub(1) else {
        return false;
    };

    match candidate.kind {
        PathSyntaxRepairKind::MalformedUseRoot => {
            if !remove_malformed_use_statement_at_line(&mut lines, line_index) {
                return false;
            }
        }
        PathSyntaxRepairKind::RepeatedSeparator => {
            let Some(line) = lines.get_mut(line_index) else {
                return false;
            };
            if !collapse_repeated_path_separator_at_span(line, candidate.column_start) {
                return false;
            }
        }
        PathSyntaxRepairKind::AbsoluteTypeRoot => {
            let Some(line) = lines.get_mut(line_index) else {
                return false;
            };
            if !remove_absolute_type_root_at_span(line, candidate) {
                return false;
            }
        }
    }

    *source = join_lines(lines);
    true
}

fn collapse_repeated_path_separator_at_span(line: &mut String, one_based_column: usize) -> bool {
    let Some((start, end)) = repeated_path_separator_span(line, one_based_column) else {
        return false;
    };
    line.replace_range(start..end, "::");
    true
}

fn remove_absolute_type_root_at_span(line: &mut String, candidate: &PathSyntaxCandidate) -> bool {
    let start = column_to_byte_index(line, candidate.column_start);
    let end = column_to_byte_index(line, candidate.column_end);
    let Some(identifier) = line.get(start..end).and_then(first_ident_prefix) else {
        return false;
    };
    let identifier = identifier.strip_prefix("r#").unwrap_or(identifier);
    if !identifier
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_uppercase())
    {
        return false;
    }
    if start < 2 || line.get(start - 2..start) != Some("::") {
        return false;
    }
    if count_preceding_colons(line, start) != 2 {
        return false;
    }
    let Some(prefix) = line.get(..start - 2) else {
        return false;
    };
    let can_remove = match prefix.chars().next_back() {
        Some(character) => character_can_precede_absolute_root(character),
        None => true,
    };
    if !can_remove {
        return false;
    }
    line.replace_range(start - 2..start, "");
    true
}

fn add_dead_code_allow_for_enclosing_type(
    source: &mut String,
    candidate: &AllowDeadCodeCandidate,
) -> bool {
    let mut lines = source.lines().map(str::to_string).collect::<Vec<_>>();
    let Some(mut cursor) = candidate.line_start.checked_sub(1) else {
        return false;
    };
    if lines.is_empty() {
        return false;
    }
    if cursor >= lines.len() {
        cursor = lines.len() - 1;
    }

    let Some(item_line) = (0..=cursor)
        .rev()
        .find(|index| line_starts_type_item(&lines[*index]))
    else {
        return false;
    };

    let mut insert_at = item_line;
    while insert_at > 0 && line_is_outer_attr_or_doc(&lines[insert_at - 1]) {
        insert_at -= 1;
    }
    if lines[insert_at..item_line]
        .iter()
        .any(|line| line.contains("allow(dead_code)"))
    {
        return false;
    }

    lines.insert(insert_at, "#[allow(dead_code)]".to_string());
    *source = join_lines(lines);
    true
}

fn add_lint_allow_before_line(source: &mut String, candidate: &AllowLintCandidate) -> bool {
    let mut lines = source.lines().map(str::to_string).collect::<Vec<_>>();
    let Some(mut cursor) = candidate.line_start.checked_sub(1) else {
        return false;
    };
    if lines.is_empty() {
        return false;
    }
    if cursor >= lines.len() {
        cursor = lines.len() - 1;
    }

    let mut insert_at = cursor;
    while insert_at > 0 && line_is_outer_attr_or_doc(&lines[insert_at - 1]) {
        insert_at -= 1;
    }
    let lint_attr = format!("allow({})", candidate.lint);
    if lines[insert_at..=cursor]
        .iter()
        .any(|line| line.contains(&lint_attr))
    {
        return false;
    }

    let indent = lines
        .get(insert_at)
        .map(|line| line.chars().take_while(|ch| ch.is_whitespace()).collect())
        .unwrap_or_else(String::new);
    lines.insert(insert_at, format!("{indent}#[{lint_attr}]"));
    *source = join_lines(lines);
    true
}

fn line_starts_type_item(line: &str) -> bool {
    let trimmed = line.trim_start();
    let rest = strip_visibility_prefix(trimmed);
    rest.starts_with("struct ")
        || rest.starts_with("enum ")
        || rest.starts_with("union ")
        || rest.starts_with("type ")
}

fn strip_visibility_prefix(text: &str) -> &str {
    if let Some(rest) = text.strip_prefix("pub ") {
        return rest;
    }
    if let Some(rest) = text.strip_prefix("pub(crate) ") {
        return rest;
    }
    if let Some(rest) = text.strip_prefix("pub(super) ") {
        return rest;
    }
    if let Some(rest) = text.strip_prefix("pub(in ") {
        if let Some((_, rest)) = rest.split_once(") ") {
            return rest;
        }
    }
    text
}

fn line_is_outer_attr_or_doc(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("#[") || trimmed.starts_with("///")
}

fn remove_column_range(line: &mut String, column_start: usize, column_end: usize) -> bool {
    if column_start == 0 || column_end < column_start {
        return false;
    }
    let start = column_to_byte_index(line, column_start);
    let end = column_to_byte_index(line, column_end);
    if start >= end || end > line.len() {
        return false;
    }
    line.replace_range(start..end, "");
    true
}

fn column_to_byte_index(line: &str, one_based_column: usize) -> usize {
    if one_based_column <= 1 {
        return 0;
    }
    line.char_indices()
        .nth(one_based_column - 1)
        .map(|(index, _)| index)
        .unwrap_or(line.len())
}

fn cleanup_import_line(line: &str) -> String {
    let indentation_len = line.len() - line.trim_start().len();
    let indentation = &line[..indentation_len];
    let mut cleaned = line[indentation_len..].to_string();
    while cleaned.trim_start().starts_with(',') {
        cleaned = cleaned.trim_start()[1..].trim_start().to_string();
    }
    for _ in 0..4 {
        cleaned = cleaned
            .replace("{, ", "{")
            .replace("{,", "{")
            .replace(", }", "}")
            .replace(",}", "}")
            .replace(", ,", ",")
            .replace("{ }", "{}")
            .replace("::::", "::");
    }
    format!("{indentation}{cleaned}")
}

fn remove_use_statement_at_line(lines: &mut Vec<String>, line_index: usize) -> bool {
    let Some(line) = lines.get(line_index) else {
        return false;
    };
    if !is_use_statement_start(line) {
        return false;
    }

    let Some(end) = use_statement_end(lines, line_index) else {
        return false;
    };
    lines.drain(line_index..=end);
    true
}

fn remove_malformed_use_statement_at_line(lines: &mut Vec<String>, line_index: usize) -> bool {
    let Some(start) = (0..=line_index).rev().find(|index| {
        lines
            .get(*index)
            .is_some_and(|line| is_use_statement_start(line))
    }) else {
        return false;
    };
    let Some(end) = use_statement_end(lines, start) else {
        return false;
    };
    if line_index > end {
        return false;
    }
    let statement = lines[start..=end].join(" ");
    if !line_has_malformed_use_root(&statement) {
        return false;
    }
    lines.drain(start..=end);
    true
}

fn use_statement_end(lines: &[String], line_index: usize) -> Option<usize> {
    let mut depth = 0isize;
    for (index, line) in lines.iter().enumerate().skip(line_index) {
        for character in line.chars() {
            match character {
                '{' => depth += 1,
                '}' => depth -= 1,
                ';' if depth <= 0 => return Some(index),
                _ => {}
            }
        }
    }
    None
}

fn filter_import_candidates_covered_by_whole_use_removals(
    source: &str,
    candidates: Vec<ImportSpanCandidate>,
) -> Vec<ImportSpanCandidate> {
    let lines = source.lines().map(str::to_string).collect::<Vec<_>>();
    let whole_use_ranges = candidates
        .iter()
        .filter(|candidate| candidate.remove_line)
        .filter_map(|candidate| {
            let start = candidate.line_start.checked_sub(1)?;
            let line = lines.get(start)?;
            is_use_statement_start(line)
                .then(|| use_statement_end(&lines, start).map(|end| (start + 1, end + 1)))?
        })
        .collect::<Vec<_>>();

    if whole_use_ranges.is_empty() {
        return candidates;
    }

    candidates
        .into_iter()
        .filter(|candidate| {
            candidate.remove_line
                || !whole_use_ranges.iter().any(|(start, end)| {
                    *start <= candidate.line_start && candidate.line_start <= *end
                })
        })
        .collect()
}

fn dedup_whole_use_removal_candidates(
    source: &str,
    candidates: Vec<ImportSpanCandidate>,
) -> Vec<ImportSpanCandidate> {
    let lines = source.lines().map(str::to_string).collect::<Vec<_>>();
    let mut seen_whole_use_ranges = BTreeSet::<(usize, usize)>::new();
    candidates
        .into_iter()
        .filter(|candidate| {
            if !candidate.remove_line {
                return true;
            }
            let Some(start) = candidate.line_start.checked_sub(1) else {
                return true;
            };
            let range = lines
                .get(start)
                .filter(|line| is_use_statement_start(line))
                .and_then(|_| use_statement_end(&lines, start))
                .map(|end| (start, end))
                .unwrap_or((start, start));
            seen_whole_use_ranges.insert(range)
        })
        .collect()
}

fn promote_fully_pruned_multiline_use_groups(
    source: &str,
    mut candidates: Vec<ImportSpanCandidate>,
) -> Vec<ImportSpanCandidate> {
    let lines = source.lines().map(str::to_string).collect::<Vec<_>>();
    let mut groups = BTreeMap::<(usize, usize), Vec<ImportSpanCandidate>>::new();
    for candidate in &candidates {
        if candidate.remove_line {
            continue;
        }
        let Some(line_index) = candidate.line_start.checked_sub(1) else {
            continue;
        };
        let Some((start, end)) = enclosing_multiline_use_statement(&lines, line_index) else {
            continue;
        };
        if line_index == start || line_index >= end {
            continue;
        }
        groups
            .entry((start, end))
            .or_default()
            .push(candidate.clone());
    }

    for ((start, end), group_candidates) in groups {
        if multiline_use_group_all_imports_are_reported_unused(
            &lines,
            start,
            end,
            &group_candidates,
        ) || multiline_use_group_inner_lines_fully_pruned(&lines, start, end, &group_candidates)
        {
            candidates.push(ImportSpanCandidate {
                line_start: start + 1,
                column_start: 1,
                column_end: lines[start].len() + 1,
                remove_line: true,
                message: String::new(),
            });
        }
    }

    candidates
}

fn enclosing_multiline_use_statement(
    lines: &[String],
    line_index: usize,
) -> Option<(usize, usize)> {
    let mut start = line_index;
    loop {
        if is_use_statement_start(lines.get(start)?) {
            let end = use_statement_end(lines, start)?;
            return (end > start).then_some((start, end));
        }
        if start == 0 {
            return None;
        }
        start -= 1;
    }
}

fn multiline_use_group_inner_lines_fully_pruned(
    lines: &[String],
    start: usize,
    end: usize,
    candidates: &[ImportSpanCandidate],
) -> bool {
    if end <= start + 1 {
        return false;
    }
    for line_index in (start + 1)..end {
        let Some(line) = lines.get(line_index) else {
            return false;
        };
        if line.trim().is_empty() {
            continue;
        }
        let mut cleaned = line.clone();
        let mut line_candidates = candidates
            .iter()
            .filter(|candidate| candidate.line_start == line_index + 1)
            .collect::<Vec<_>>();
        line_candidates.sort_by(|left, right| right.column_start.cmp(&left.column_start));
        if line_candidates.is_empty() {
            return false;
        }
        for candidate in line_candidates {
            let _ = remove_column_range(&mut cleaned, candidate.column_start, candidate.column_end);
        }
        cleaned = cleanup_import_line(&cleaned);
        if !cleaned.trim().is_empty() {
            return false;
        }
    }
    true
}

fn multiline_use_group_all_imports_are_reported_unused(
    lines: &[String],
    start: usize,
    end: usize,
    candidates: &[ImportSpanCandidate],
) -> bool {
    let import_names = multiline_use_group_leaf_names(lines, start, end);
    if import_names.is_empty() {
        return false;
    }

    let unused_names = candidates
        .iter()
        .flat_map(|candidate| unused_import_names_from_message(&candidate.message))
        .collect::<BTreeSet<_>>();
    !unused_names.is_empty() && import_names.is_subset(&unused_names)
}

fn multiline_use_group_leaf_names(lines: &[String], start: usize, end: usize) -> BTreeSet<String> {
    use_statement_leaf_names_from_text(&lines[start..=end].join(" "))
}

fn use_statement_leaf_names_from_text(text: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let mut segment = String::new();
    for character in text.chars() {
        match character {
            '{' => segment.clear(),
            ',' | '}' | ';' => {
                if let Some(name) = last_ident_in_use_segment(&segment) {
                    names.insert(name);
                }
                segment.clear();
            }
            _ => segment.push(character),
        }
    }
    names
}

fn unused_import_names_from_message(message: &str) -> BTreeSet<String> {
    if !message.starts_with("unused import") {
        return BTreeSet::new();
    }

    let mut names = BTreeSet::new();
    let mut remaining = message;
    while let Some((_, after_open)) = remaining.split_once('`') {
        let Some((raw_name, after_close)) = after_open.split_once('`') else {
            break;
        };
        if let Some(name) = last_ident_in_use_segment(raw_name) {
            names.insert(name);
        }
        remaining = after_close;
    }
    names
}

fn last_ident_in_use_segment(segment: &str) -> Option<String> {
    segment
        .split(|character: char| {
            !(character.is_ascii_alphanumeric() || character == '_' || character == '#')
        })
        .rev()
        .filter_map(clean_ident_token)
        .find(|token| !matches!(token.as_str(), "use" | "crate" | "as"))
}

fn join_lines(lines: Vec<String>) -> String {
    let mut source = lines.join("\n");
    source.push('\n');
    source
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::{
        repair::{repair_workspace, RepairOptions},
        CheckDiagnostic, CheckSpan, CheckSuggestion,
    };

    #[test]
    fn removes_whole_unused_import_line() {
        let root = temp_output("repair-unused-import");
        let file = root.join("src/lib.rs");
        write(
            &file,
            "use std::fmt;\n\npub fn keep() -> usize {\n    1\n}\n",
        );

        let report = repair_workspace(RepairOptions {
            output_root: root.clone(),
            diagnostics: vec![warning(
                "unused_imports",
                "unused import: `std::fmt`",
                "src/lib.rs",
                1,
                1,
                14,
                "use std::fmt;",
            )],
        })
        .unwrap();

        assert_eq!(report.removed_imports, 1);
        assert_eq!(report.changed_files.len(), 1);
        assert_eq!(report.changed_files[0].removed_imports, 1);
        let source = fs::read_to_string(file).unwrap();
        assert!(!source.contains("std::fmt"));
        assert!(source.contains("pub fn keep"));
    }

    #[test]
    fn removes_unused_macro_definition() {
        let root = temp_output("repair-unused-macro");
        let file = root.join("src/lib.rs");
        write(
            &file,
            "macro_rules! req {\n    () => { 1 };\n}\n\npub(crate) use req;\n\npub fn keep() -> usize {\n    1\n}\n",
        );

        let report = repair_workspace(RepairOptions {
            output_root: root.clone(),
            diagnostics: vec![
                warning(
                    "unused_macros",
                    "unused macro definition: `req`",
                    "src/lib.rs",
                    1,
                    14,
                    17,
                    "macro_rules! req {",
                ),
                warning(
                    "unused_imports",
                    "unused import: `req`",
                    "src/lib.rs",
                    5,
                    16,
                    19,
                    "pub(crate) use req;",
                ),
            ],
        })
        .unwrap();

        assert_eq!(report.removed_items, 1);
        assert_eq!(report.removed_imports, 1);
        let source = fs::read_to_string(file).unwrap();
        assert!(!source.contains("macro_rules! req"), "{source}");
        assert!(!source.contains("use req"), "{source}");
        assert!(source.contains("pub fn keep"), "{source}");
    }

    #[test]
    fn removes_unused_import_from_machine_applicable_help() {
        let root = temp_output("repair-unused-import-suggestion");
        let file = root.join("src/lib.rs");
        write(
            &file,
            "use std::fmt;\nuse std::io;\n\npub fn keep() -> usize {\n    1\n}\n",
        );

        let report = repair_workspace(RepairOptions {
            output_root: root.clone(),
            diagnostics: vec![CheckDiagnostic {
                level: "warning".to_string(),
                message: "unused import: `std::fmt`".to_string(),
                code: Some("unused_imports".to_string()),
                package_id: None,
                target: None,
                rendered: None,
                spans: Vec::new(),
                suggestions: vec![CheckSuggestion {
                    level: "help".to_string(),
                    message: "remove the whole `use` item".to_string(),
                    file_name: "src/lib.rs".to_string(),
                    line_start: 1,
                    line_end: 2,
                    column_start: 1,
                    column_end: 1,
                    byte_start: Some(0),
                    byte_end: Some(14),
                    suggested_replacement: String::new(),
                    suggestion_applicability: Some("MachineApplicable".to_string()),
                }],
            }],
        })
        .unwrap();

        assert_eq!(report.removed_imports, 1);
        let source = fs::read_to_string(file).unwrap();
        assert!(!source.contains("std::fmt"));
        assert!(source.contains("use std::io;"));
        assert!(source.contains("pub fn keep"));
    }

    #[test]
    fn removes_dead_function_with_attributes() {
        let root = temp_output("repair-dead-function");
        let file = root.join("src/lib.rs");
        write(
            &file,
            "#[inline]\npub fn dead() -> usize {\n    1\n}\n\npub fn keep() -> usize {\n    2\n}\n",
        );

        let report = repair_workspace(RepairOptions {
            output_root: root.clone(),
            diagnostics: vec![warning(
                "dead_code",
                "function `dead` is never used",
                "src/lib.rs",
                2,
                8,
                12,
                "pub fn dead() -> usize {",
            )],
        })
        .unwrap();

        assert_eq!(report.removed_items, 1);
        assert_eq!(report.changed_files.len(), 1);
        assert_eq!(report.changed_files[0].removed_items, 1);
        let source = fs::read_to_string(file).unwrap();
        assert!(!source.contains("dead"));
        assert!(source.contains("pub fn keep"));
    }

    #[test]
    fn removes_fully_unused_multiline_import_group() {
        let root = temp_output("repair-multiline-import");
        let file = root.join("src/lib.rs");
        write(
            &file,
            "use crate::{\n    Alpha,\n    Beta,\n};\n\npub fn keep() {}\n",
        );

        let report = repair_workspace(RepairOptions {
            output_root: root.clone(),
            diagnostics: vec![
                warning(
                    "unused_imports",
                    "unused imports: `Alpha` and `Beta`",
                    "src/lib.rs",
                    2,
                    5,
                    10,
                    "    Alpha,",
                ),
                warning(
                    "unused_imports",
                    "unused imports: `Alpha` and `Beta`",
                    "src/lib.rs",
                    3,
                    5,
                    9,
                    "    Beta,",
                ),
            ],
        })
        .unwrap();

        assert_eq!(report.removed_imports, 1);
        let source = fs::read_to_string(file).unwrap();
        assert!(!source.contains("use crate"));
        assert!(source.contains("pub fn keep"));
    }

    #[test]
    fn allows_dead_struct_fields_after_structural_repairs_are_exhausted() {
        let root = temp_output("repair-dead-field");
        let file = root.join("src/lib.rs");
        write(
            &file,
            "#[derive(Debug)]\npub struct Config {\n    socket_path: String,\n    request_timeout: u64,\n}\n",
        );

        let report = repair_workspace(RepairOptions {
            output_root: root.clone(),
            diagnostics: vec![warning(
                "dead_code",
                "field `socket_path` is never read",
                "src/lib.rs",
                3,
                5,
                16,
                "    socket_path: String,",
            )],
        })
        .unwrap();

        assert_eq!(report.added_dead_code_allows, 1);
        assert_eq!(report.total_changes(), 1);
        assert_eq!(report.changed_files.len(), 1);
        assert_eq!(report.changed_files[0].added_dead_code_allows, 1);
        let source = fs::read_to_string(file).unwrap();
        assert!(source.contains("#[allow(dead_code)]\n#[derive(Debug)]\npub struct Config"));
        assert!(source.contains("socket_path: String"));
        assert!(source.contains("request_timeout: u64"));
    }

    #[test]
    fn allows_dead_enum_variants_without_removing_variants() {
        let root = temp_output("repair-dead-enum-variant");
        let file = root.join("src/lib.rs");
        write(
            &file,
            "pub enum Mode {\n    Fast,\n    Slow,\n}\n\npub fn keep() -> Mode {\n    Mode::Fast\n}\n",
        );

        let report = repair_workspace(RepairOptions {
            output_root: root.clone(),
            diagnostics: vec![warning(
                "dead_code",
                "variant `Slow` is never constructed",
                "src/lib.rs",
                3,
                5,
                9,
                "    Slow,",
            )],
        })
        .unwrap();

        assert_eq!(report.added_dead_code_allows, 1);
        let source = fs::read_to_string(file).unwrap();
        assert!(source.contains("#[allow(dead_code)]\npub enum Mode"));
        assert!(source.contains("Fast"));
        assert!(source.contains("Slow"));
        assert!(source.contains("Mode::Fast"));
    }

    #[test]
    fn allows_private_interface_lints_on_reported_item() {
        let root = temp_output("repair-private-interface");
        let file = root.join("src/lib.rs");
        write(
            &file,
            "mod connect {\n    pub(super) struct ClientHandler;\n}\n\npub struct SshClient {\n    pub(crate) handle: connect::ClientHandler,\n}\n",
        );

        let report = repair_workspace(RepairOptions {
            output_root: root.clone(),
            diagnostics: vec![warning(
                "private_interfaces",
                "type `ClientHandler` is more private than the item `SshClient::handle`",
                "src/lib.rs",
                6,
                5,
                47,
                "    pub(crate) handle: connect::ClientHandler,",
            )],
        })
        .unwrap();

        assert_eq!(report.added_lint_allows, 1);
        assert_eq!(report.total_changes(), 1);
        assert_eq!(report.changed_files.len(), 1);
        assert_eq!(report.changed_files[0].added_lint_allows, 1);
        let source = fs::read_to_string(file).unwrap();
        assert!(source.contains(
            "pub struct SshClient {\n    #[allow(private_interfaces)]\n    pub(crate) handle"
        ));
    }

    #[test]
    fn defers_dead_code_allows_when_structural_repairs_exist() {
        let root = temp_output("repair-defer-field-allow");
        let file = root.join("src/lib.rs");
        write(
            &file,
            "use std::fmt;\n\npub struct Config {\n    socket_path: String,\n}\n",
        );

        let report = repair_workspace(RepairOptions {
            output_root: root.clone(),
            diagnostics: vec![
                warning(
                    "unused_imports",
                    "unused import: `std::fmt`",
                    "src/lib.rs",
                    1,
                    1,
                    14,
                    "use std::fmt;",
                ),
                warning(
                    "dead_code",
                    "field `socket_path` is never read",
                    "src/lib.rs",
                    4,
                    5,
                    16,
                    "    socket_path: String,",
                ),
            ],
        })
        .unwrap();

        assert_eq!(report.removed_imports, 1);
        assert_eq!(report.added_dead_code_allows, 0);
        assert_eq!(report.deferred_dead_code_allows, 1);
        let source = fs::read_to_string(file).unwrap();
        assert!(!source.contains("std::fmt"));
        assert!(!source.contains("#[allow(dead_code)]"));
        assert!(source.contains("socket_path: String"));
    }

    #[test]
    fn removes_malformed_use_root_from_compiler_error() {
        let root = temp_output("repair-malformed-use-root");
        let file = root.join("src/lib.rs");
        write(
            &file,
            "use ::::{Removed};\n\npub fn keep() -> usize {\n    1\n}\n",
        );

        let report = repair_workspace(RepairOptions {
            output_root: root.clone(),
            diagnostics: vec![error(
                None,
                "expected identifier, found `::`",
                "src/lib.rs",
                1,
                7,
                9,
                "use ::::{Removed};",
            )],
        })
        .unwrap();

        assert_eq!(report.normalized_paths, 1);
        assert_eq!(report.total_changes(), 1);
        assert_eq!(report.skipped_diagnostics, 0);
        assert_eq!(report.changed_files.len(), 1);
        assert_eq!(report.changed_files[0].normalized_paths, 1);
        let source = fs::read_to_string(file).unwrap();
        assert!(!source.contains("Removed"));
        assert!(source.contains("pub fn keep"));
    }

    #[test]
    fn collapses_repeated_path_separator_at_compiler_span() {
        let root = temp_output("repair-repeated-path-separator");
        let file = root.join("src/lib.rs");
        write(&file, "pub fn run() {\n    module:::value();\n}\n");

        let report = repair_workspace(RepairOptions {
            output_root: root.clone(),
            diagnostics: vec![error(
                None,
                "path separator must be a double colon",
                "src/lib.rs",
                2,
                13,
                14,
                "    module:::value();",
            )],
        })
        .unwrap();

        assert_eq!(report.normalized_paths, 1);
        let source = fs::read_to_string(file).unwrap();
        assert!(source.contains("module::value();"));
        assert!(!source.contains("module:::value();"));
    }

    #[test]
    fn removes_uppercase_absolute_type_root_remnant() {
        let root = temp_output("repair-absolute-type-root");
        let file = root.join("src/lib.rs");
        write(
            &file,
            "pub struct Local;\npub fn build() {\n    let _ = ::Local;\n}\n",
        );

        let report = repair_workspace(RepairOptions {
            output_root: root.clone(),
            diagnostics: vec![error(
                Some("E0425"),
                "cannot find crate `Local` in the list of imported crates",
                "src/lib.rs",
                3,
                15,
                20,
                "    let _ = ::Local;",
            )],
        })
        .unwrap();

        assert_eq!(report.normalized_paths, 1);
        let source = fs::read_to_string(file).unwrap();
        assert!(source.contains("let _ = Local;"));
        assert!(!source.contains("::Local"));
    }

    #[test]
    fn keeps_lowercase_absolute_crate_roots_for_widening() {
        let root = temp_output("repair-lowercase-crate-root");
        let file = root.join("src/lib.rs");
        write(
            &file,
            "pub fn build() {\n    let _ = ::std::fmt::Error;\n}\n",
        );

        let report = repair_workspace(RepairOptions {
            output_root: root.clone(),
            diagnostics: vec![error(
                Some("E0433"),
                "failed to resolve: could not find `std` in the list of imported crates",
                "src/lib.rs",
                2,
                15,
                18,
                "    let _ = ::std::fmt::Error;",
            )],
        })
        .unwrap();

        assert_eq!(report.normalized_paths, 0);
        assert_eq!(report.skipped_diagnostics, 1);
        let source = fs::read_to_string(file).unwrap();
        assert!(source.contains("::std::fmt::Error"));
    }

    #[test]
    fn applies_machine_applicable_compiler_suggestions_inside_output() {
        let root = temp_output("repair-machine-suggestion");
        let file = root.join("src/lib.rs");
        let source = "pub fn run() {\n    let _: i32 = \"1\";\n}\n";
        let start = source.find("\"1\"").unwrap() as u64;
        let end = start + 3;
        write(&file, source);

        let report = repair_workspace(RepairOptions {
            output_root: root.clone(),
            diagnostics: vec![CheckDiagnostic {
                level: "error".to_string(),
                message: "mismatched types".to_string(),
                code: Some("E0308".to_string()),
                package_id: None,
                target: None,
                rendered: None,
                spans: Vec::new(),
                suggestions: vec![CheckSuggestion {
                    level: "help".to_string(),
                    message: "replace the string literal with an integer literal".to_string(),
                    file_name: "src/lib.rs".to_string(),
                    line_start: 2,
                    line_end: 2,
                    column_start: 18,
                    column_end: 21,
                    byte_start: Some(start),
                    byte_end: Some(end),
                    suggested_replacement: "1".to_string(),
                    suggestion_applicability: Some("MachineApplicable".to_string()),
                }],
            }],
        })
        .unwrap();

        assert_eq!(report.applied_suggestions, 1);
        assert_eq!(report.total_changes(), 1);
        assert_eq!(report.changed_files[0].applied_suggestions, 1);
        let source = fs::read_to_string(file).unwrap();
        assert!(source.contains("let _: i32 = 1;"));
        assert!(!source.contains("\"1\""));
    }

    fn warning(
        code: &str,
        message: &str,
        file_name: &str,
        line_start: u64,
        column_start: u64,
        column_end: u64,
        text: &str,
    ) -> CheckDiagnostic {
        CheckDiagnostic {
            level: "warning".to_string(),
            message: message.to_string(),
            code: Some(code.to_string()),
            package_id: None,
            target: None,
            rendered: None,
            spans: vec![CheckSpan {
                file_name: file_name.to_string(),
                line_start,
                line_end: line_start,
                column_start,
                column_end,
                is_primary: true,
                text: vec![text.to_string()],
            }],
            suggestions: Vec::new(),
        }
    }

    fn error(
        code: Option<&str>,
        message: &str,
        file_name: &str,
        line_start: u64,
        column_start: u64,
        column_end: u64,
        text: &str,
    ) -> CheckDiagnostic {
        CheckDiagnostic {
            level: "error".to_string(),
            message: message.to_string(),
            code: code.map(str::to_string),
            package_id: None,
            target: None,
            rendered: None,
            spans: vec![CheckSpan {
                file_name: file_name.to_string(),
                line_start,
                line_end: line_start,
                column_start,
                column_end,
                is_primary: true,
                text: vec![text.to_string()],
            }],
            suggestions: Vec::new(),
        }
    }

    fn temp_output(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("slicers-{label}-{unique}"));
        if path.exists() {
            fs::remove_dir_all(&path).unwrap();
        }
        path
    }

    fn write(path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, contents).unwrap();
    }
}
