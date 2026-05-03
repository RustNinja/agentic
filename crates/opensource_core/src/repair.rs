use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::feedback::{CheckDiagnostic, CheckSpan};

#[derive(Debug, Clone)]
pub struct RepairOptions {
    pub output_root: PathBuf,
    pub diagnostics: Vec<CheckDiagnostic>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct RepairReport {
    pub removed_items: usize,
    pub removed_imports: usize,
    pub added_dead_code_allows: usize,
    pub deferred_dead_code_allows: usize,
    pub skipped_diagnostics: usize,
}

impl RepairReport {
    pub fn total_changes(&self) -> usize {
        self.removed_items + self.removed_imports + self.added_dead_code_allows
    }
}

pub fn repair_workspace(
    options: RepairOptions,
) -> Result<RepairReport, Box<dyn std::error::Error>> {
    let mut item_candidates_by_file = BTreeMap::<PathBuf, Vec<DeadItemCandidate>>::new();
    let mut import_candidates_by_file = BTreeMap::<PathBuf, Vec<ImportSpanCandidate>>::new();
    let mut allow_candidates_by_file = BTreeMap::<PathBuf, Vec<AllowDeadCodeCandidate>>::new();
    let mut skipped_diagnostics = 0;

    for diagnostic in &options.diagnostics {
        if diagnostic.level != "warning" {
            skipped_diagnostics += 1;
            continue;
        }

        let code = diagnostic.code.as_deref();
        if !matches!(code, Some("dead_code" | "unused_imports")) {
            skipped_diagnostics += 1;
            continue;
        }

        let mut repaired = false;
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
    let has_structural_repairs =
        !item_candidates_by_file.is_empty() || !import_candidates_by_file.is_empty();
    for (path, mut candidates) in item_candidates_by_file {
        if !path.exists() {
            continue;
        }
        candidates.sort_by(|left, right| right.line_start.cmp(&left.line_start));
        candidates.dedup();
        let mut source = fs::read_to_string(&path)?;
        for candidate in candidates {
            if remove_item_at_line(&mut source, &candidate) {
                report.removed_items += 1;
            }
        }
        fs::write(path, source)?;
    }
    for (path, mut candidates) in import_candidates_by_file {
        if !path.exists() {
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
        for candidate in candidates {
            if remove_import_span(&mut source, &candidate) {
                report.removed_imports += 1;
            }
        }
        fs::write(path, source)?;
    }

    if has_structural_repairs {
        report.deferred_dead_code_allows = allow_candidates_by_file
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
            for candidate in candidates {
                if add_dead_code_allow_for_enclosing_type(&mut source, &candidate) {
                    report.added_dead_code_allows += 1;
                }
            }
            fs::write(path, source)?;
        }
    }

    Ok(report)
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

fn dead_candidate_name(message: &str, span: &CheckSpan) -> Option<String> {
    if !dead_code_message_is_item(message) {
        return None;
    }
    name_from_span_text(span).or_else(|| name_from_backticks(message).map(str::to_string))
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
        CheckDiagnostic, CheckSpan,
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
        let source = fs::read_to_string(file).unwrap();
        assert!(!source.contains("std::fmt"));
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
