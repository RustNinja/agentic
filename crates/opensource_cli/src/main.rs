use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use opensource_core::{
    generate, generate_compiler_prune_workspace, generate_lint_audit_workspace,
    CompilerPruneOptions, GenerateOptions, LintAuditOptions,
};
use serde_json::{json, Value};

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut run_check = false;
    let mut lint_audit = false;
    let mut compiler_prune = false;
    let mut rust_analyzer = RustAnalyzerOptions::default();
    let mut positional = Vec::new();
    let mut args = std::env::args_os().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_os_str() {
            value if value == OsStr::new("--check") => run_check = true,
            value if value == OsStr::new("--lint-audit") => lint_audit = true,
            value if value == OsStr::new("--compiler-prune") => compiler_prune = true,
            value if value == OsStr::new("--ra-audit") => rust_analyzer.enabled = true,
            value if value == OsStr::new("--ra-disable-build-scripts") => {
                rust_analyzer.disable_build_scripts = true
            }
            value if value == OsStr::new("--ra-disable-proc-macros") => {
                rust_analyzer.disable_proc_macros = true
            }
            value if value == OsStr::new("--rust-analyzer") => {
                let Some(path) = args.next() else {
                    return Err("--rust-analyzer requires a path".into());
                };
                rust_analyzer.binary = Some(PathBuf::from(path));
            }
            value if value == OsStr::new("--ra-proc-macro-srv") => {
                let Some(path) = args.next() else {
                    return Err("--ra-proc-macro-srv requires a path".into());
                };
                rust_analyzer.proc_macro_srv = Some(PathBuf::from(path));
            }
            value if value == OsStr::new("--help") || value == OsStr::new("-h") => {
                println!("{}", usage());
                return Ok(());
            }
            _ => positional.push(PathBuf::from(arg)),
        }
    }

    let [workspace_root, output_root] = positional.as_slice() else {
        return Err(usage().into());
    };

    if same_path(workspace_root, output_root) {
        return Err("output root must be different from workspace root".into());
    }

    if lint_audit {
        run_lint_audit(
            workspace_root.clone(),
            output_root.clone(),
            rust_analyzer.clone(),
        )?;
        return Ok(());
    }

    if compiler_prune {
        run_compiler_prune(
            workspace_root.clone(),
            output_root.clone(),
            rust_analyzer.clone(),
        )?;
        return Ok(());
    }

    let report = generate(GenerateOptions {
        workspace_root: workspace_root.clone(),
        output_root: output_root.clone(),
    })?;

    println!("root: {}", report.root);
    println!("files written: {}", report.files_written);
    println!("packages: {}", report.packages.join(", "));
    println!("reachable callables:");
    for callable in &report.reachable {
        println!("  {callable}");
    }
    println!("reachable items:");
    for item in &report.reachable_items {
        println!("  {item}");
    }

    if run_check {
        let status = Command::new("cargo")
            .arg("check")
            .arg("--manifest-path")
            .arg(output_root.join("Cargo.toml"))
            .status()?;
        if !status.success() {
            return Err(format!("generated workspace failed cargo check with {status}").into());
        }
    }

    if rust_analyzer.enabled {
        run_rust_analyzer_audit(&output_root, &rust_analyzer)?;
    }

    Ok(())
}

fn run_compiler_prune(
    workspace_root: PathBuf,
    output_root: PathBuf,
    rust_analyzer: RustAnalyzerOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let report = generate_compiler_prune_workspace(CompilerPruneOptions {
        workspace_root,
        output_root: output_root.clone(),
    })?;

    println!("compiler-prune root: {}", report.root);
    println!("compiler-prune files written: {}", report.files_written);
    println!("compiler-prune packages: {}", report.packages.join(", "));
    println!(
        "compiler-prune demoted visibilities: {}",
        report.demoted_visibilities
    );
    println!("compiler-prune linted crate roots:");
    for path in &report.linted_roots {
        println!("  {}", path.display());
    }

    let mut rounds = Vec::new();
    let mut final_output = run_cargo_check_json(&output_root)?;
    let mut diagnostics = compiler_prune_diagnostics(&final_output.stdout, &output_root);
    for round in 0..12 {
        let prune_report = prune_dead_items_from_diagnostics(&output_root, &diagnostics)?;
        let redemoted = 0;
        rounds.push(json!({
            "round": round,
            "cargo_status": final_output.status.code(),
            "diagnostic_count": diagnostics.len(),
            "diagnostics": diagnostics,
            "removed_dead_items": prune_report.removed_items,
            "removed_imports": prune_report.removed_imports,
            "field_dead_code_allows": prune_report.field_dead_code_allows,
            "deferred_field_dead_code_allows": prune_report.deferred_field_dead_code_allows,
            "total_changes": prune_report.total_changes(),
            "redemoted_visibilities": redemoted,
        }));
        if prune_report.total_changes() == 0 && redemoted == 0 {
            break;
        }
        final_output = run_cargo_check_json(&output_root)?;
        diagnostics = compiler_prune_diagnostics(&final_output.stdout, &output_root);
    }

    let report_path = output_root.join("slicers-compiler-prune-report.json");
    let payload = json!({
        "root": report.root.to_string(),
        "packages": report.packages,
        "files_written": report.files_written,
        "demoted_visibilities": report.demoted_visibilities,
        "cargo_status": final_output.status.code(),
        "diagnostic_count": diagnostics.len(),
        "diagnostics": diagnostics,
        "rounds": rounds,
    });
    fs::write(&report_path, serde_json::to_string_pretty(&payload)?)?;

    println!(
        "compiler-prune diagnostics: {} ({})",
        payload["diagnostic_count"],
        report_path.display()
    );

    if !final_output.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&final_output.stderr));
    }

    if rust_analyzer.enabled {
        run_rust_analyzer_audit(&output_root, &rust_analyzer)?;
    }

    if !final_output.status.success() || !diagnostics.is_empty() {
        return Err(format!(
            "compiler-prune did not converge: cargo status {:?}, {} local diagnostics remain; see {}",
            final_output.status.code(),
            diagnostics.len(),
            report_path.display()
        )
        .into());
    }

    Ok(())
}

fn run_cargo_check_json(output_root: &Path) -> Result<Output, Box<dyn std::error::Error>> {
    let target_dir = std::env::var_os("SLICERS_CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or(std::env::current_dir()?.join("target/slicers-compiler-prune"));
    Ok(Command::new("cargo")
        .arg("check")
        .arg("--message-format=json")
        .arg("--manifest-path")
        .arg(output_root.join("Cargo.toml"))
        .env("CARGO_TARGET_DIR", target_dir)
        .output()?)
}

fn prune_dead_items_from_diagnostics(
    output_root: &Path,
    diagnostics: &[Value],
) -> Result<PruneReport, Box<dyn std::error::Error>> {
    let mut prune_ops_by_file: BTreeMap<PathBuf, Vec<DeadItemCandidate>> = BTreeMap::new();
    let mut field_allows_by_file: BTreeMap<PathBuf, Vec<AllowDeadCodeCandidate>> = BTreeMap::new();
    let mut import_spans_by_file: BTreeMap<PathBuf, Vec<ImportSpanCandidate>> = BTreeMap::new();
    for diagnostic in diagnostics {
        let code = diagnostic
            .get("code")
            .and_then(|code| code.get("code"))
            .and_then(Value::as_str);
        let Some(message) = diagnostic.get("message").and_then(Value::as_str) else {
            continue;
        };

        for span in diagnostic
            .get("spans")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let Some(file_name) = span.get("file_name").and_then(Value::as_str) else {
                continue;
            };
            let Some(line_start) = span.get("line_start").and_then(Value::as_u64) else {
                continue;
            };
            if code == Some("unused_imports") {
                let span_text = span_text(span);
                let remove_line = span_text
                    .is_some_and(|text| unused_import_span_removes_whole_use(text, message));
                let column_start = span
                    .get("column_start")
                    .and_then(Value::as_u64)
                    .unwrap_or(1);
                let column_end = span
                    .get("column_end")
                    .and_then(Value::as_u64)
                    .unwrap_or(column_start);
                let Some(path) = diagnostic_path(output_root, file_name) else {
                    continue;
                };
                import_spans_by_file
                    .entry(path)
                    .or_default()
                    .push(ImportSpanCandidate {
                        line_start: line_start as usize,
                        column_start: column_start as usize,
                        column_end: column_end as usize,
                        remove_line,
                        message: message.to_string(),
                    });
                continue;
            }
            let Some(path) = diagnostic_path(output_root, file_name) else {
                continue;
            };
            if code == Some("dead_code") && dead_code_message_is_field(message) {
                field_allows_by_file
                    .entry(path)
                    .or_default()
                    .push(AllowDeadCodeCandidate {
                        line_start: line_start as usize,
                    });
                continue;
            }
            let Some(name) = dead_candidate_name(code, message, span) else {
                continue;
            };
            prune_ops_by_file
                .entry(path)
                .or_default()
                .push(DeadItemCandidate {
                    name,
                    line_start: line_start as usize,
                });
        }
    }

    let mut report = PruneReport::default();
    for (path, mut candidates) in prune_ops_by_file {
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
    for (path, mut candidates) in import_spans_by_file {
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

    if report.removed_items == 0 && report.removed_imports == 0 {
        for (path, mut candidates) in field_allows_by_file {
            if !path.exists() {
                continue;
            }
            candidates.sort_by(|left, right| right.line_start.cmp(&left.line_start));
            candidates.dedup();
            let mut source = fs::read_to_string(&path)?;
            for candidate in candidates {
                if allow_dead_code_on_enclosing_item(&mut source, &candidate) {
                    report.field_dead_code_allows += 1;
                }
            }
            fs::write(path, source)?;
        }
    } else {
        report.deferred_field_dead_code_allows = field_allows_by_file.values().map(Vec::len).sum();
    }

    Ok(report)
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

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct PruneReport {
    removed_items: usize,
    removed_imports: usize,
    field_dead_code_allows: usize,
    deferred_field_dead_code_allows: usize,
}

impl PruneReport {
    fn total_changes(&self) -> usize {
        self.removed_items + self.removed_imports + self.field_dead_code_allows
    }
}

fn diagnostic_path(output_root: &Path, file_name: &str) -> Option<PathBuf> {
    let path = PathBuf::from(file_name);
    if path.is_absolute() {
        path_is_inside_output_root(&path, output_root).then_some(path)
    } else {
        Some(output_root.join(path))
    }
}

fn name_from_backticks(message: &str) -> Option<&str> {
    let (_, rest) = message.split_once('`')?;
    let (name, _) = rest.split_once('`')?;
    Some(name)
}

fn dead_candidate_name(code: Option<&str>, message: &str, span: &Value) -> Option<String> {
    if code == Some("dead_code") && dead_code_message_is_item(message) {
        let message_name = name_from_backticks(message)?;
        return Some(name_from_span_text(span).unwrap_or_else(|| message_name.to_string()));
    }

    if matches!(code, Some("E0405" | "E0412" | "E0425"))
        && message.starts_with("cannot find")
        && span_text(span)?.trim_start().starts_with("impl ")
    {
        return Some("impl".to_string());
    }

    if matches!(code, Some("unused_imports" | "E0432" | "E0603"))
        && is_use_statement_start(span_text(span)?)
    {
        return Some("use".to_string());
    }

    None
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

fn dead_code_message_is_field(message: &str) -> bool {
    message.contains("field `") || message.contains("fields `")
}

fn name_from_span_text(span: &Value) -> Option<String> {
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

fn span_text(span: &Value) -> Option<&str> {
    span.get("text")
        .and_then(Value::as_array)?
        .first()?
        .get("text")
        .and_then(Value::as_str)
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
        *source = lines.join("\n");
        source.push('\n');
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
    *source = lines.join("\n");
    source.push('\n');
    true
}

fn allow_dead_code_on_enclosing_item(
    source: &mut String,
    candidate: &AllowDeadCodeCandidate,
) -> bool {
    let mut lines = source.lines().map(str::to_string).collect::<Vec<_>>();
    let Some(mut index) = candidate.line_start.checked_sub(1) else {
        return false;
    };
    if index >= lines.len() {
        return false;
    }

    loop {
        let trimmed = lines[index].trim_start();
        if item_header_can_own_field_dead_code(trimmed) {
            let mut insert_at = index;
            while insert_at > 0 {
                let previous = lines[insert_at - 1].trim_start();
                if previous.starts_with("#[") || previous.starts_with("///") {
                    insert_at -= 1;
                } else {
                    break;
                }
            }
            if lines[insert_at..=index]
                .iter()
                .any(|line| line.contains("allow(dead_code)"))
            {
                return false;
            }
            let indentation = &lines[index][..lines[index].len() - trimmed.len()];
            lines.insert(insert_at, format!("{indentation}#[allow(dead_code)]"));
            *source = lines.join("\n");
            source.push('\n');
            return true;
        }
        if index == 0 {
            return false;
        }
        index -= 1;
    }
}

fn item_header_can_own_field_dead_code(trimmed: &str) -> bool {
    let without_visibility = trimmed
        .strip_prefix("pub ")
        .or_else(|| trimmed.strip_prefix("pub(crate) "))
        .or_else(|| trimmed.strip_prefix("pub(super) "))
        .or_else(|| trimmed.strip_prefix("pub(in "))
        .unwrap_or(trimmed);
    without_visibility.starts_with("struct ")
        || without_visibility.starts_with("enum ")
        || without_visibility.starts_with("union ")
}

fn remove_import_span(source: &mut String, candidate: &ImportSpanCandidate) -> bool {
    let mut lines = source.lines().map(str::to_string).collect::<Vec<_>>();
    let Some(line_index) = candidate.line_start.checked_sub(1) else {
        return false;
    };
    let Some(line) = lines.get_mut(line_index) else {
        return false;
    };

    if candidate.remove_line {
        if !remove_use_statement_at_line(&mut lines, line_index) {
            lines.remove(line_index);
        }
        *source = lines.join("\n");
        source.push('\n');
        return true;
    }

    if !remove_column_range(line, candidate.column_start, candidate.column_end) {
        return false;
    }
    *line = cleanup_import_line(line);
    if line.trim().is_empty() || line.contains("::{}") {
        lines.remove(line_index);
    }

    *source = lines.join("\n");
    source.push('\n');
    true
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
            .replace("{ }", "{}");
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

#[derive(Clone, Debug, Default)]
struct RustAnalyzerOptions {
    enabled: bool,
    binary: Option<PathBuf>,
    proc_macro_srv: Option<PathBuf>,
    disable_build_scripts: bool,
    disable_proc_macros: bool,
}

fn run_lint_audit(
    workspace_root: PathBuf,
    output_root: PathBuf,
    rust_analyzer: RustAnalyzerOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let report = generate_lint_audit_workspace(LintAuditOptions {
        workspace_root,
        output_root: output_root.clone(),
    })?;

    println!("lint-audit files written: {}", report.files_written);
    println!("lint-audit packages: {}", report.packages.join(", "));
    println!("lint-audit injected crate roots:");
    for path in &report.linted_roots {
        println!("  {}", path.display());
    }

    let output = Command::new("cargo")
        .arg("check")
        .arg("--message-format=json")
        .arg("--manifest-path")
        .arg(output_root.join("Cargo.toml"))
        .output()?;

    let diagnostics = lint_audit_diagnostics(&output.stdout);
    let report_path = output_root.join("slicers-lint-audit-report.json");
    let payload = json!({
        "cargo_status": output.status.code(),
        "diagnostic_count": diagnostics.len(),
        "diagnostics": diagnostics,
    });
    fs::write(&report_path, serde_json::to_string_pretty(&payload)?)?;

    println!(
        "lint-audit diagnostics: {} ({})",
        payload["diagnostic_count"],
        report_path.display()
    );

    if !output.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }

    if rust_analyzer.enabled {
        run_rust_analyzer_audit(&output_root, &rust_analyzer)?;
    }

    Ok(())
}

fn run_rust_analyzer_audit(
    output_root: &PathBuf,
    options: &RustAnalyzerOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let binary = rust_analyzer_binary(options);
    let availability = Command::new(&binary).arg("--version").output();
    let Ok(availability) = availability else {
        write_rust_analyzer_report(
            output_root,
            &binary,
            json!({
                "available": false,
                "error": "failed to start rust-analyzer",
            }),
        )?;
        println!(
            "rust-analyzer audit unavailable: failed to start {}",
            binary.display()
        );
        return Ok(());
    };

    if !availability.status.success() {
        write_rust_analyzer_report(
            output_root,
            &binary,
            json!({
                "available": false,
                "status": availability.status.code(),
                "stdout": String::from_utf8_lossy(&availability.stdout),
                "stderr": String::from_utf8_lossy(&availability.stderr),
            }),
        )?;
        println!(
            "rust-analyzer audit unavailable: {} --version failed",
            binary.display()
        );
        return Ok(());
    }

    let diagnostics = run_rust_analyzer_command(&binary, "diagnostics", output_root, options)?;
    let unresolved =
        run_rust_analyzer_command(&binary, "unresolved-references", output_root, options)?;
    let payload = json!({
        "available": true,
        "binary": path_to_json_string(binary.as_os_str()),
        "version": String::from_utf8_lossy(&availability.stdout).trim(),
        "diagnostics": diagnostics,
        "unresolved_references": unresolved,
    });
    write_rust_analyzer_report(output_root, &binary, payload)?;
    println!(
        "rust-analyzer audit written: {}",
        output_root
            .join("slicers-rust-analyzer-report.json")
            .display()
    );
    Ok(())
}

fn run_rust_analyzer_command(
    binary: &PathBuf,
    subcommand: &str,
    output_root: &PathBuf,
    options: &RustAnalyzerOptions,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut command = Command::new(binary);
    command.arg(subcommand).arg(output_root);
    if options.disable_build_scripts {
        command.arg("--disable-build-scripts");
    }
    if options.disable_proc_macros {
        command.arg("--disable-proc-macros");
    }
    if !options.disable_proc_macros {
        if let Some(proc_macro_srv) = rust_analyzer_proc_macro_srv(options) {
            command.arg("--proc-macro-srv").arg(proc_macro_srv);
        }
    }

    let output = command.output()?;
    Ok(json!({
        "status": output.status.code(),
        "stdout": String::from_utf8_lossy(&output.stdout),
        "stderr": String::from_utf8_lossy(&output.stderr),
    }))
}

fn rust_analyzer_binary(options: &RustAnalyzerOptions) -> PathBuf {
    if let Some(binary) = &options.binary {
        return binary.clone();
    }
    if let Some(binary) = std::env::var_os("SLICERS_RUST_ANALYZER") {
        return PathBuf::from(binary);
    }
    PathBuf::from("rust-analyzer")
}

fn rust_analyzer_proc_macro_srv(options: &RustAnalyzerOptions) -> Option<PathBuf> {
    if let Some(proc_macro_srv) = &options.proc_macro_srv {
        return Some(proc_macro_srv.clone());
    }
    std::env::var_os("SLICERS_RA_PROC_MACRO_SRV").map(PathBuf::from)
}

fn write_rust_analyzer_report(
    output_root: &PathBuf,
    binary: &PathBuf,
    mut payload: Value,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(object) = payload.as_object_mut() {
        object.insert(
            "binary".to_string(),
            Value::String(path_to_json_string(binary.as_os_str())),
        );
    }
    fs::write(
        output_root.join("slicers-rust-analyzer-report.json"),
        serde_json::to_string_pretty(&payload)?,
    )?;
    Ok(())
}

fn path_to_json_string(path: &OsStr) -> String {
    path.to_string_lossy().into_owned()
}

fn compiler_prune_diagnostics(stdout: &[u8], output_root: &Path) -> Vec<Value> {
    lint_audit_diagnostics(stdout)
        .into_iter()
        .filter(|diagnostic| diagnostic_is_inside_output_root(diagnostic, output_root))
        .collect()
}

fn lint_audit_diagnostics(stdout: &[u8]) -> Vec<Value> {
    let interesting_codes = [
        "dead_code",
        "unused_imports",
        "unused_macros",
        "unreachable_pub",
    ];
    String::from_utf8_lossy(stdout)
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter(|value| value.get("reason").and_then(Value::as_str) == Some("compiler-message"))
        .filter_map(|value| {
            let message = value.get("message")?.clone();
            Some((value, message))
        })
        .filter(|(_, message)| {
            let level = message.get("level").and_then(Value::as_str);
            let code = message
                .get("code")
                .and_then(|code| code.get("code"))
                .and_then(Value::as_str);
            level == Some("error")
                || (level == Some("warning")
                    && code.is_some_and(|code| interesting_codes.contains(&code)))
        })
        .map(|(value, message)| {
            let primary_spans = message
                .get("spans")
                .and_then(Value::as_array)
                .map(|spans| {
                    spans
                        .iter()
                        .filter(|span| {
                            span.get("is_primary").and_then(Value::as_bool) == Some(true)
                        })
                        .cloned()
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            json!({
                "code": message.get("code").cloned().unwrap_or(Value::Null),
                "level": message.get("level").cloned().unwrap_or(Value::Null),
                "manifest_path": value.get("manifest_path").cloned().unwrap_or(Value::Null),
                "package_id": value.get("package_id").cloned().unwrap_or(Value::Null),
                "message": message.get("message").cloned().unwrap_or(Value::Null),
                "spans": primary_spans,
            })
        })
        .collect()
}

fn diagnostic_is_inside_output_root(diagnostic: &Value, output_root: &Path) -> bool {
    if let Some(manifest_path) = diagnostic.get("manifest_path").and_then(Value::as_str) {
        return path_is_inside_output_root(Path::new(manifest_path), output_root);
    }

    let Some(spans) = diagnostic.get("spans").and_then(Value::as_array) else {
        return true;
    };
    if spans.is_empty() {
        return true;
    }

    spans.iter().any(|span| {
        span.get("file_name")
            .and_then(Value::as_str)
            .is_some_and(|file_name| diagnostic_path(output_root, file_name).is_some())
    })
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

    let Some(parent) = path.parent() else {
        return path.to_path_buf();
    };
    let Ok(canonical_parent) = parent.canonicalize() else {
        return path.to_path_buf();
    };
    match path.file_name() {
        Some(file_name) => canonical_parent.join(file_name),
        None => canonical_parent,
    }
}

fn usage() -> String {
    [
        "usage: slicers [--check] [--lint-audit] [--ra-audit]",
        "               [--compiler-prune]",
        "               [--rust-analyzer <path>]",
        "               [--ra-proc-macro-srv <path>]",
        "               [--ra-disable-build-scripts] [--ra-disable-proc-macros]",
        "               <workspace-root> <output-root>",
    ]
    .join("\n")
}

fn same_path(left: &PathBuf, right: &PathBuf) -> bool {
    let Ok(left) = left.canonicalize() else {
        return false;
    };
    let Ok(right) = right.canonicalize() else {
        return false;
    };
    left == right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleanup_import_line_removes_leading_group_comma() {
        assert_eq!(cleanup_import_line("    , Envelope,"), "    Envelope,");
        assert_eq!(cleanup_import_line("    ,"), "    ");
    }

    #[test]
    fn remove_import_span_removes_whole_multiline_use_statement() {
        let mut source = r#"use crate::protocol::params::{
    InitializeParams,
    TypedBroadcast,
};

pub struct Kept;
"#
        .to_string();

        let removed = remove_import_span(
            &mut source,
            &ImportSpanCandidate {
                line_start: 1,
                column_start: 1,
                column_end: 29,
                remove_line: true,
                message: String::new(),
            },
        );

        assert!(removed);
        assert_eq!(source, "\npub struct Kept;\n");
    }

    #[test]
    fn prune_imports_ignores_nested_spans_inside_removed_multiline_use() {
        let root = std::env::temp_dir().join(format!(
            "slicers-cli-import-overlap-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let source_path = root.join("src/lib.rs");
        std::fs::create_dir_all(source_path.parent().unwrap()).unwrap();
        std::fs::write(
            &source_path,
            r#"use crate::protocol::params::{
    InitializeParams,
    TypedBroadcast,
};

pub struct Kept {
    pub request_timeout: Duration,
}
"#,
        )
        .unwrap();

        let diagnostics = vec![
            json!({
                "code": { "code": "unused_imports" },
                "message": "unused import",
                "spans": [{
                    "file_name": "src/lib.rs",
                    "line_start": 1,
                    "column_start": 1,
                    "column_end": 31,
                    "text": [{ "text": "use crate::protocol::params::{" }]
                }]
            }),
            json!({
                "code": { "code": "unused_imports" },
                "message": "unused import",
                "spans": [{
                    "file_name": "src/lib.rs",
                    "line_start": 2,
                    "column_start": 5,
                    "column_end": 21,
                    "text": [{ "text": "    InitializeParams," }]
                }]
            }),
        ];

        let report = prune_dead_items_from_diagnostics(&root, &diagnostics).unwrap();
        let source = std::fs::read_to_string(source_path).unwrap();

        assert_eq!(report.total_changes(), 1);
        assert_eq!(report.removed_imports, 1);
        assert!(!source.contains("use crate::protocol::params"));
        assert!(source.contains("pub struct Kept {"));
        assert!(source.contains("pub request_timeout: Duration"));
    }

    #[test]
    fn prune_imports_promotes_fully_unused_multiline_group_from_inner_spans() {
        let root = std::env::temp_dir().join(format!(
            "slicers-cli-import-inner-only-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let source_path = root.join("src/lib.rs");
        std::fs::create_dir_all(source_path.parent().unwrap()).unwrap();
        std::fs::write(
            &source_path,
            r#"use crate::protocol::params::{
    InitializeParams,
    TypedBroadcast,
};

pub struct Kept {
    pub request_timeout: Duration,
}
"#,
        )
        .unwrap();

        let diagnostics = vec![
            json!({
                "code": { "code": "unused_imports" },
                "message": "unused import",
                "spans": [{
                    "file_name": "src/lib.rs",
                    "line_start": 2,
                    "column_start": 5,
                    "column_end": 21,
                    "text": [{ "text": "    InitializeParams," }]
                }]
            }),
            json!({
                "code": { "code": "unused_imports" },
                "message": "unused import",
                "spans": [{
                    "file_name": "src/lib.rs",
                    "line_start": 3,
                    "column_start": 5,
                    "column_end": 19,
                    "text": [{ "text": "    TypedBroadcast," }]
                }]
            }),
        ];

        let report = prune_dead_items_from_diagnostics(&root, &diagnostics).unwrap();
        let source = std::fs::read_to_string(source_path).unwrap();

        assert_eq!(report.total_changes(), 1);
        assert_eq!(report.removed_imports, 1);
        assert!(!source.contains("use crate::protocol::params"));
        assert!(!source.contains("InitializeParams"));
        assert!(!source.contains("TypedBroadcast"));
        assert!(source.contains("pub struct Kept {"));
        assert!(source.contains("pub request_timeout: Duration"));
    }

    #[test]
    fn prune_imports_promotes_multiline_group_from_message_names() {
        let root = std::env::temp_dir().join(format!(
            "slicers-cli-import-message-names-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let source_path = root.join("src/lib.rs");
        std::fs::create_dir_all(source_path.parent().unwrap()).unwrap();
        std::fs::write(
            &source_path,
            r#"use crate::protocol::params::{
    ThreadFollowerCommandApprovalDecisionParams, ThreadFollowerEditLastUserTurnParams,
    ThreadFollowerFileApprovalDecisionParams,
    TypedBroadcast,
};

pub struct Kept {
    inner: Arc<Inner>,
}
"#,
        )
        .unwrap();

        let diagnostics = vec![json!({
            "code": { "code": "unused_imports" },
            "message": "unused imports: `ThreadFollowerCommandApprovalDecisionParams`, `ThreadFollowerEditLastUserTurnParams`, `ThreadFollowerFileApprovalDecisionParams`, and `TypedBroadcast`",
            "spans": [{
                "file_name": "src/lib.rs",
                "line_start": 2,
                "column_start": 5,
                "column_end": 48,
                "text": [{
                    "text": "    ThreadFollowerCommandApprovalDecisionParams, ThreadFollowerEditLastUserTurnParams,"
                }]
            }]
        })];

        let report = prune_dead_items_from_diagnostics(&root, &diagnostics).unwrap();
        let source = std::fs::read_to_string(source_path).unwrap();

        assert_eq!(report.total_changes(), 1);
        assert_eq!(report.removed_imports, 1);
        assert!(!source.contains("use crate::protocol::params"));
        assert!(!source.contains("ThreadFollower"));
        assert!(!source.contains("TypedBroadcast"));
        assert!(source.contains("pub struct Kept {"));
        assert!(source.contains("inner: Arc<Inner>"));
    }

    #[test]
    fn prune_imports_removes_duplicate_whole_use_spans_once() {
        let root = std::env::temp_dir().join(format!(
            "slicers-cli-import-duplicate-whole-use-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let source_path = root.join("src/lib.rs");
        std::fs::create_dir_all(source_path.parent().unwrap()).unwrap();
        std::fs::write(
            &source_path,
            r#"use tokio::io::{AsyncRead, AsyncWrite};
use crate::client::connection::IpcConnection;

pub struct Kept {
    connection: IpcConnection,
}
"#,
        )
        .unwrap();

        let diagnostics = vec![json!({
            "code": { "code": "unused_imports" },
            "message": "unused imports: `AsyncRead` and `AsyncWrite`",
            "spans": [
                {
                    "file_name": "src/lib.rs",
                    "line_start": 1,
                    "column_start": 17,
                    "column_end": 26,
                    "text": [{ "text": "use tokio::io::{AsyncRead, AsyncWrite};" }]
                },
                {
                    "file_name": "src/lib.rs",
                    "line_start": 1,
                    "column_start": 28,
                    "column_end": 38,
                    "text": [{ "text": "use tokio::io::{AsyncRead, AsyncWrite};" }]
                }
            ]
        })];

        let report = prune_dead_items_from_diagnostics(&root, &diagnostics).unwrap();
        let source = std::fs::read_to_string(source_path).unwrap();

        assert_eq!(report.total_changes(), 1);
        assert_eq!(report.removed_imports, 1);
        assert!(!source.contains("tokio::io"));
        assert!(source.contains("use crate::client::connection::IpcConnection;"));
        assert!(source.contains("connection: IpcConnection"));
    }

    #[test]
    fn prune_imports_keeps_used_member_in_single_line_group() {
        let root = std::env::temp_dir().join(format!(
            "slicers-cli-import-partial-single-line-group-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let source_path = root.join("src/lib.rs");
        std::fs::create_dir_all(source_path.parent().unwrap()).unwrap();
        std::fs::write(
            &source_path,
            r#"use crate::protocol::params::{TypedBroadcast, TypedRequest};

pub struct Kept {
    broadcast_tx: TypedBroadcast,
}
"#,
        )
        .unwrap();

        let diagnostics = vec![json!({
            "code": { "code": "unused_imports" },
            "message": "unused import: `TypedRequest`",
            "spans": [{
                "file_name": "src/lib.rs",
                "line_start": 1,
                "column_start": 47,
                "column_end": 59,
                "text": [{
                    "text": "use crate::protocol::params::{TypedBroadcast, TypedRequest};"
                }]
            }]
        })];

        let report = prune_dead_items_from_diagnostics(&root, &diagnostics).unwrap();
        let source = std::fs::read_to_string(source_path).unwrap();

        assert_eq!(report.total_changes(), 1);
        assert_eq!(report.removed_imports, 1);
        assert!(source.contains("use crate::protocol::params::{TypedBroadcast};"));
        assert!(!source.contains("TypedRequest"));
        assert!(source.contains("broadcast_tx: TypedBroadcast"));
    }

    #[test]
    fn prune_imports_removes_visibility_prefixed_single_use() {
        let root = std::env::temp_dir().join(format!(
            "slicers-cli-import-pub-crate-use-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let source_path = root.join("src/lib.rs");
        std::fs::create_dir_all(source_path.parent().unwrap()).unwrap();
        std::fs::write(
            &source_path,
            r#"pub(crate) use snapshot::QueuedFollowUpDraft;
pub use snapshot::AppSnapshot;
"#,
        )
        .unwrap();

        let diagnostics = vec![json!({
            "code": { "code": "unused_imports" },
            "message": "unused import: `snapshot::QueuedFollowUpDraft`",
            "spans": [{
                "file_name": "src/lib.rs",
                "line_start": 1,
                "column_start": 16,
                "column_end": 47,
                "text": [{ "text": "pub(crate) use snapshot::QueuedFollowUpDraft;" }]
            }]
        })];

        let report = prune_dead_items_from_diagnostics(&root, &diagnostics).unwrap();
        let source = std::fs::read_to_string(source_path).unwrap();

        assert_eq!(report.total_changes(), 1);
        assert_eq!(report.removed_imports, 1);
        assert!(!source.contains("QueuedFollowUpDraft"));
        assert!(!source.contains("pub(crate) use ;"));
        assert!(source.contains("pub use snapshot::AppSnapshot;"));
    }

    #[test]
    fn prune_dead_items_keeps_item_lines_stable_when_imports_are_removed() {
        let root = std::env::temp_dir().join(format!(
            "slicers-cli-prune-order-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let source_path = root.join("src/lib.rs");
        std::fs::create_dir_all(source_path.parent().unwrap()).unwrap();
        std::fs::write(
            &source_path,
            r#"use crate::foo::{
    A,
};

pub struct Config {
    socket_path: PathBuf,
    request_timeout: Duration,
}
"#,
        )
        .unwrap();

        let diagnostics = vec![
            json!({
                "code": { "code": "unused_imports" },
                "message": "unused import",
                "spans": [{
                    "file_name": "src/lib.rs",
                    "line_start": 1,
                    "column_start": 1,
                    "column_end": 18,
                    "text": [{ "text": "use crate::foo::{" }]
                }]
            }),
            json!({
                "code": { "code": "dead_code" },
                "message": "field `socket_path` is never read",
                "spans": [{
                    "file_name": "src/lib.rs",
                    "line_start": 6,
                    "text": [{ "text": "    socket_path: PathBuf," }]
                }]
            }),
        ];

        let report = prune_dead_items_from_diagnostics(&root, &diagnostics).unwrap();
        let source = std::fs::read_to_string(source_path).unwrap();

        assert_eq!(report.total_changes(), 1);
        assert_eq!(report.removed_imports, 1);
        assert_eq!(report.deferred_field_dead_code_allows, 1);
        assert!(!source.contains("#[allow(dead_code)]"));
        assert!(source.contains("pub struct Config {"));
        assert!(source.contains("socket_path: PathBuf"));
        assert!(source.contains("request_timeout: Duration"));
        assert!(!source.contains("use crate::foo"));
    }

    #[test]
    fn prune_field_dead_code_only_after_structural_pruning_is_exhausted() {
        let root = std::env::temp_dir().join(format!(
            "slicers-cli-field-allow-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let source_path = root.join("src/lib.rs");
        std::fs::create_dir_all(source_path.parent().unwrap()).unwrap();
        std::fs::write(
            &source_path,
            r#"pub struct Config {
    socket_path: PathBuf,
    request_timeout: Duration,
}
"#,
        )
        .unwrap();

        let diagnostics = vec![json!({
            "code": { "code": "dead_code" },
            "message": "field `socket_path` is never read",
            "spans": [{
                "file_name": "src/lib.rs",
                "line_start": 2,
                "text": [{ "text": "    socket_path: PathBuf," }]
            }]
        })];

        let report = prune_dead_items_from_diagnostics(&root, &diagnostics).unwrap();
        let source = std::fs::read_to_string(source_path).unwrap();

        assert_eq!(report.total_changes(), 1);
        assert_eq!(report.field_dead_code_allows, 1);
        assert!(source.contains("#[allow(dead_code)]\npub struct Config {"));
        assert!(source.contains("socket_path: PathBuf"));
        assert!(source.contains("request_timeout: Duration"));
    }

    #[test]
    fn compiler_prune_does_not_preallow_reachable_dependency_items() {
        let workspace = std::env::temp_dir().join(format!(
            "slicers-cli-compiler-prune-fixture-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let output = std::env::temp_dir().join(format!(
            "slicers-cli-compiler-prune-output-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let target = std::env::temp_dir().join(format!(
            "slicers-cli-compiler-prune-target-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _target_guard = EnvVarGuard::set_path("SLICERS_CARGO_TARGET_DIR", &target);
        let opensourced_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("opensourced");
        let opensourced_path = opensourced_path.to_string_lossy();

        std::fs::create_dir_all(workspace.join("app/src")).unwrap();
        std::fs::create_dir_all(workspace.join("helper/src")).unwrap();
        std::fs::write(
            workspace.join("Cargo.toml"),
            format!(
                r#"[workspace]
members = ["app", "helper"]
resolver = "2"

[workspace.dependencies]
opensourced = {{ path = "{}" }}
"#,
                opensourced_path
            ),
        )
        .unwrap();
        std::fs::write(
            workspace.join("app/Cargo.toml"),
            r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
helper = { path = "../helper" }
opensourced.workspace = true
"#,
        )
        .unwrap();
        std::fs::write(
            workspace.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;

#[opensourced]
pub fn exported() -> helper::Kept {
    helper::make()
}
"#,
        )
        .unwrap();
        std::fs::write(
            workspace.join("helper/Cargo.toml"),
            r#"[package]
name = "helper"
version = "0.1.0"
edition = "2021"
"#,
        )
        .unwrap();
        std::fs::write(
            workspace.join("helper/src/lib.rs"),
            r#"use std::{fmt::Debug, marker::PhantomData};

pub struct Kept {
    value: i32,
    marker: PhantomData<()>,
}

pub fn make() -> Kept {
    Kept {
        value: 7,
        marker: PhantomData,
    }
}
"#,
        )
        .unwrap();

        run_compiler_prune(workspace, output.clone(), RustAnalyzerOptions::default())
            .expect("compiler-prune should converge");

        let helper_source = std::fs::read_to_string(output.join("helper/src/lib.rs")).unwrap();
        assert!(
            helper_source.contains("#[allow(dead_code)]\npub struct Kept"),
            "field-level dead_code should be allowed only on the remaining necessary type:\n{helper_source}"
        );
        assert!(
            !helper_source.contains("#[allow(dead_code)]\npub fn make"),
            "reachable functions must not be pre-allowed:\n{helper_source}"
        );

        let report: Value = serde_json::from_str(
            &std::fs::read_to_string(output.join("slicers-compiler-prune-report.json")).unwrap(),
        )
        .unwrap();
        let rounds = report
            .get("rounds")
            .and_then(Value::as_array)
            .expect("report should contain rounds");
        assert!(
            rounds.iter().any(|round| round
                .get("field_dead_code_allows")
                .and_then(Value::as_u64)
                .unwrap_or(0)
                > 0),
            "later pass should restore only necessary field dead_code allows:\n{report:#}"
        );
    }

    struct EnvVarGuard {
        key: &'static str,
        previous: Option<std::ffi::OsString>,
    }

    impl EnvVarGuard {
        fn set_path(key: &'static str, value: &Path) -> Self {
            let previous = std::env::var_os(key);
            std::env::set_var(key, value);
            Self { key, previous }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            if let Some(previous) = &self.previous {
                std::env::set_var(self.key, previous);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }
}
