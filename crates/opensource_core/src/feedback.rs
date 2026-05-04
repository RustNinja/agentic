use std::{
    ffi::OsStr,
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    thread,
    time::{Duration, Instant},
};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct CheckOptions {
    pub manifest_path: PathBuf,
    pub target_dir: Option<PathBuf>,
    pub timeout: Option<Duration>,
    pub cargo_args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckReport {
    pub manifest_path: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<PathBuf>,
    pub target_dir: Option<PathBuf>,
    pub timeout_ms: Option<u64>,
    #[serde(default)]
    pub cargo_args: Vec<String>,
    pub success: bool,
    pub timed_out: bool,
    pub exit_code: i32,
    pub duration_ms: u64,
    pub diagnostics: Vec<CheckDiagnostic>,
    #[serde(default)]
    pub widening: FeedbackWideningReport,
    pub stderr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckDiagnostic {
    pub level: String,
    pub message: String,
    pub code: Option<String>,
    #[serde(default)]
    pub package_id: Option<String>,
    #[serde(default)]
    pub target: Option<CheckTarget>,
    pub rendered: Option<String>,
    pub spans: Vec<CheckSpan>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suggestions: Vec<CheckSuggestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckTarget {
    pub name: String,
    #[serde(default)]
    pub kind: Vec<String>,
    #[serde(default)]
    pub src_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckSpan {
    pub file_name: String,
    pub line_start: u64,
    pub line_end: u64,
    pub column_start: u64,
    pub column_end: u64,
    pub is_primary: bool,
    pub text: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckSuggestion {
    pub level: String,
    pub message: String,
    pub file_name: String,
    pub line_start: u64,
    pub line_end: u64,
    pub column_start: u64,
    pub column_end: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub byte_start: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub byte_end: Option<u64>,
    pub suggested_replacement: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggestion_applicability: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FeedbackWideningReport {
    pub candidates: Vec<FeedbackWideningCandidate>,
    pub hazards: Vec<FeedbackHazard>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackWideningCandidate {
    pub kind: String,
    pub confidence: String,
    pub code: Option<String>,
    pub symbol: Option<String>,
    pub file_name: Option<String>,
    pub line_start: Option<u64>,
    #[serde(default)]
    pub suggestions: usize,
    #[serde(default)]
    pub machine_applicable_suggestions: usize,
    pub action: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackHazard {
    pub kind: String,
    pub severity: String,
    pub code: Option<String>,
    pub message: String,
}

impl CheckReport {
    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.level == "error")
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.level == "warning")
            .count()
    }
}

pub fn check_workspace(options: CheckOptions) -> Result<CheckReport, Box<dyn std::error::Error>> {
    check_workspace_with_program(OsStr::new("cargo"), options)
}

fn check_workspace_with_program(
    program: &OsStr,
    options: CheckOptions,
) -> Result<CheckReport, Box<dyn std::error::Error>> {
    let manifest_path_for_cargo = absolute_path(&options.manifest_path)?;
    let working_dir = manifest_working_dir(&manifest_path_for_cargo);
    let target_dir_for_cargo = options
        .target_dir
        .as_ref()
        .map(|target_dir| absolute_path(target_dir))
        .transpose()?;
    let mut command = Command::new(program);
    command
        .arg("check")
        .arg("--manifest-path")
        .arg(&manifest_path_for_cargo)
        .arg("--message-format=json");
    command.current_dir(&working_dir);
    command.args(&options.cargo_args);

    if let Some(target_dir) = &target_dir_for_cargo {
        command.env("CARGO_TARGET_DIR", target_dir);
    }

    let outcome = run_command(command, options.timeout)?;
    let output = outcome.output;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let mut diagnostics = parse_cargo_messages(&stdout);
    if outcome.timed_out {
        diagnostics.push(timeout_failure_diagnostic(options.timeout));
    }
    if !output.status.success() && diagnostics.is_empty() {
        if let Some(diagnostic) = stderr_failure_diagnostic(&stderr) {
            diagnostics.push(diagnostic);
        }
    }

    let widening = classify_feedback(&diagnostics);

    Ok(CheckReport {
        manifest_path: options.manifest_path,
        working_dir: Some(working_dir),
        target_dir: target_dir_for_cargo,
        timeout_ms: options
            .timeout
            .map(|timeout| timeout.as_millis().try_into().unwrap_or(u64::MAX)),
        cargo_args: options.cargo_args,
        success: output.status.success(),
        timed_out: outcome.timed_out,
        exit_code: output.status.code().unwrap_or(-1),
        duration_ms: outcome.duration_ms,
        diagnostics,
        widening,
        stderr,
    })
}

fn absolute_path(path: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    Ok(std::env::current_dir()?.join(path))
}

fn manifest_working_dir(manifest_path: &Path) -> PathBuf {
    manifest_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}

pub fn write_report(report: &CheckReport, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(report)?)?;
    Ok(())
}

fn parse_cargo_messages(stdout: &str) -> Vec<CheckDiagnostic> {
    stdout
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter(|message| message.get("reason").and_then(Value::as_str) == Some("compiler-message"))
        .filter_map(|message| diagnostic_from_compiler_message(&message))
        .collect()
}

fn classify_feedback(diagnostics: &[CheckDiagnostic]) -> FeedbackWideningReport {
    let mut candidates = diagnostics
        .iter()
        .filter_map(classify_widening_candidate)
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        left.file_name
            .cmp(&right.file_name)
            .then_with(|| left.line_start.cmp(&right.line_start))
            .then_with(|| left.kind.cmp(&right.kind))
            .then_with(|| left.symbol.cmp(&right.symbol))
            .then_with(|| left.message.cmp(&right.message))
    });
    candidates.dedup_by(|left, right| {
        left.kind == right.kind
            && left.code == right.code
            && left.symbol == right.symbol
            && left.file_name == right.file_name
            && left.line_start == right.line_start
            && left.message == right.message
    });

    let mut hazards = diagnostics
        .iter()
        .filter_map(classify_feedback_hazard)
        .collect::<Vec<_>>();
    hazards.sort_by(|left, right| {
        hazard_severity_rank(&left.severity)
            .cmp(&hazard_severity_rank(&right.severity))
            .then_with(|| left.kind.cmp(&right.kind))
            .then_with(|| left.code.cmp(&right.code))
            .then_with(|| left.message.cmp(&right.message))
    });
    hazards.dedup_by(|left, right| {
        left.kind == right.kind && left.code == right.code && left.message == right.message
    });

    FeedbackWideningReport {
        candidates,
        hazards,
    }
}

fn classify_widening_candidate(diagnostic: &CheckDiagnostic) -> Option<FeedbackWideningCandidate> {
    if diagnostic.level != "error" {
        return None;
    }
    let code = diagnostic.code.as_deref()?;
    let (kind, confidence, action) = match code {
        "E0405" => (
            "missing-trait",
            "high",
            "widen the retained trait, trait import, or dependency feature that defines the missing trait",
        ),
        "E0412" | "E0422" => (
            "missing-type",
            "high",
            "widen the retained type definition, type re-export, or dependency feature referenced by this path",
        ),
        "E0425" => (
            "missing-value",
            "high",
            "widen the retained function, const, static, local module item, or import referenced by this expression",
        ),
        "E0432" | "E0433" => (
            "unresolved-path",
            "high",
            "widen the retained import/module path or restore the dependency package providing this path",
        ),
        "E0463" => (
            "missing-crate",
            "high",
            "restore the dependency package, manifest entry, feature, or target-specific dependency for this crate",
        ),
        "E0583" => (
            "missing-module-file",
            "high",
            "restore the external module file or remove the module declaration if the module is truly unreachable",
        ),
        "E0599" => (
            "missing-method-or-associated-item",
            "medium",
            "widen receiver type impls, extension trait imports, inherent impls, or trait bounds needed by this call",
        ),
        "E0277" if diagnostic.message.contains(": From<")
            || diagnostic.message.contains(": Into<")
            || diagnostic.message.contains(": TryFrom<")
            || diagnostic.message.contains(": TryInto<") =>
        {
            (
                "missing-conversion-trait-impl",
                "medium",
                "widen the retained conversion trait impl required by this .into(), .try_into(), or conversion call",
            )
        }
        _ => return None,
    };
    let primary = diagnostic.spans.iter().find(|span| span.is_primary);
    Some(FeedbackWideningCandidate {
        kind: kind.to_string(),
        confidence: confidence.to_string(),
        code: diagnostic.code.clone(),
        symbol: diagnostic_symbol(diagnostic),
        file_name: primary.map(|span| span.file_name.clone()),
        line_start: primary.map(|span| span.line_start),
        suggestions: diagnostic.suggestions.len(),
        machine_applicable_suggestions: diagnostic
            .suggestions
            .iter()
            .filter(|suggestion| {
                suggestion.suggestion_applicability.as_deref() == Some("MachineApplicable")
            })
            .count(),
        action: action.to_string(),
        message: diagnostic.message.clone(),
    })
}

fn classify_feedback_hazard(diagnostic: &CheckDiagnostic) -> Option<FeedbackHazard> {
    if semantic_warning_is_hazard(diagnostic) {
        return Some(FeedbackHazard {
            kind: "semantic-warning".to_string(),
            severity: "high".to_string(),
            code: diagnostic.code.clone(),
            message: diagnostic.message.clone(),
        });
    }

    let code = diagnostic.code.as_deref()?;
    let (kind, severity) = match code {
        "cargo-timeout" => ("feedback-timeout", "blocker"),
        "cargo-stderr" => ("cargo-shape-failure", "blocker"),
        "E0463" => ("missing-crate", "blocker"),
        "E0583" => ("missing-module-file", "blocker"),
        "E0277" | "E0432" | "E0433" | "E0405" | "E0412" | "E0422" | "E0425" | "E0599" => {
            ("needs-widening", "high")
        }
        _ if diagnostic.level == "error" => ("unclassified-compiler-error", "medium"),
        _ => return None,
    };

    Some(FeedbackHazard {
        kind: kind.to_string(),
        severity: severity.to_string(),
        code: diagnostic.code.clone(),
        message: diagnostic.message.clone(),
    })
}

fn semantic_warning_is_hazard(diagnostic: &CheckDiagnostic) -> bool {
    if diagnostic.level != "warning" {
        return false;
    }
    matches!(
        diagnostic.code.as_deref(),
        Some("unreachable_patterns" | "irrefutable_let_patterns" | "bindings_with_variant_name")
    ) || (diagnostic.code.as_deref() == Some("non_snake_case")
        && diagnostic.message.contains("variable `"))
        || diagnostic.message.contains("unreachable pattern")
        || diagnostic.message.contains("irrefutable")
}

fn hazard_severity_rank(severity: &str) -> usize {
    match severity {
        "blocker" => 0,
        "high" => 1,
        "medium" => 2,
        "low" => 3,
        _ => 4,
    }
}

fn diagnostic_symbol(diagnostic: &CheckDiagnostic) -> Option<String> {
    extract_backticked_symbol(&diagnostic.message).or_else(|| {
        diagnostic
            .rendered
            .as_deref()
            .and_then(extract_backticked_symbol)
    })
}

fn extract_backticked_symbol(text: &str) -> Option<String> {
    let start = text.find('`')?;
    let tail = &text[start + 1..];
    let end = tail.find('`')?;
    Some(tail[..end].to_string()).filter(|symbol| !symbol.is_empty())
}

struct CommandOutcome {
    output: std::process::Output,
    timed_out: bool,
    duration_ms: u64,
}

fn run_command(
    mut command: Command,
    timeout: Option<Duration>,
) -> Result<CommandOutcome, Box<dyn std::error::Error>> {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    configure_timeout_process_group(&mut command);
    let mut child = command.spawn()?;
    let stdout_reader = child.stdout.take().map(read_output_pipe);
    let stderr_reader = child.stderr.take().map(read_output_pipe);
    let started = Instant::now();
    let mut timed_out = false;
    loop {
        if let Some(status) = child.try_wait()? {
            let output = Output {
                status,
                stdout: join_output_reader(stdout_reader)?,
                stderr: join_output_reader(stderr_reader)?,
            };
            return Ok(CommandOutcome {
                output,
                timed_out,
                duration_ms: elapsed_ms(started),
            });
        }
        if timeout.is_some_and(|timeout| started.elapsed() >= timeout) {
            kill_timed_out_child(&mut child);
            let status = child.wait()?;
            timed_out = true;
            let output = Output {
                status,
                stdout: join_output_reader(stdout_reader)?,
                stderr: join_output_reader(stderr_reader)?,
            };
            return Ok(CommandOutcome {
                output,
                timed_out,
                duration_ms: elapsed_ms(started),
            });
        }
        thread::sleep(Duration::from_millis(50));
    }
}

fn read_output_pipe<R: Read + Send + 'static>(
    mut pipe: R,
) -> thread::JoinHandle<std::io::Result<Vec<u8>>> {
    thread::spawn(move || {
        let mut output = Vec::new();
        pipe.read_to_end(&mut output)?;
        Ok(output)
    })
}

fn join_output_reader(
    reader: Option<thread::JoinHandle<std::io::Result<Vec<u8>>>>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let Some(reader) = reader else {
        return Ok(Vec::new());
    };
    Ok(reader
        .join()
        .map_err(|_| "cargo output reader panicked".to_string())??)
}

fn elapsed_ms(started: Instant) -> u64 {
    started.elapsed().as_millis().try_into().unwrap_or(u64::MAX)
}

#[cfg(unix)]
fn configure_timeout_process_group(command: &mut Command) {
    command.process_group(0);
}

#[cfg(not(unix))]
fn configure_timeout_process_group(_command: &mut Command) {}

#[cfg(unix)]
fn kill_timed_out_child(child: &mut std::process::Child) {
    let process_group = -(child.id() as libc::pid_t);
    // Kill the process group so build scripts do not outlive the timed-out cargo process.
    let killed_group = unsafe { libc::kill(process_group, libc::SIGKILL) == 0 };
    if !killed_group {
        let _ = child.kill();
    }
}

#[cfg(not(unix))]
fn kill_timed_out_child(child: &mut std::process::Child) {
    let _ = child.kill();
}

fn diagnostic_from_compiler_message(message: &Value) -> Option<CheckDiagnostic> {
    let diagnostic = message.get("message")?;
    let level = diagnostic.get("level")?.as_str()?.to_string();
    let message_text = diagnostic.get("message")?.as_str()?.to_string();
    let code = diagnostic
        .get("code")
        .and_then(|code| code.get("code"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let package_id = message
        .get("package_id")
        .and_then(Value::as_str)
        .map(str::to_string);
    let target = message.get("target").and_then(target_from_value);
    let rendered = diagnostic
        .get("rendered")
        .and_then(Value::as_str)
        .map(str::to_string);
    let spans = diagnostic
        .get("spans")
        .and_then(Value::as_array)
        .map(|spans| spans.iter().filter_map(span_from_value).collect())
        .unwrap_or_default();
    let mut suggestions = Vec::new();
    collect_suggestions(diagnostic, &mut suggestions);

    Some(CheckDiagnostic {
        level,
        message: message_text,
        code,
        package_id,
        target,
        rendered,
        spans,
        suggestions,
    })
}

fn target_from_value(target: &Value) -> Option<CheckTarget> {
    Some(CheckTarget {
        name: target.get("name")?.as_str()?.to_string(),
        kind: target
            .get("kind")
            .and_then(Value::as_array)
            .map(|kinds| {
                kinds
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default(),
        src_path: target
            .get("src_path")
            .and_then(Value::as_str)
            .map(str::to_string),
    })
}

fn timeout_failure_diagnostic(timeout: Option<Duration>) -> CheckDiagnostic {
    let seconds = timeout.map(|timeout| timeout.as_secs()).unwrap_or_default();
    CheckDiagnostic {
        level: "error".to_string(),
        message: format!("cargo check exceeded feedback timeout of {seconds}s"),
        code: Some("cargo-timeout".to_string()),
        package_id: None,
        target: None,
        rendered: Some(format!(
            "cargo check was terminated after exceeding the feedback timeout of {seconds}s\n"
        )),
        spans: Vec::new(),
        suggestions: Vec::new(),
    }
}

fn stderr_failure_diagnostic(stderr: &str) -> Option<CheckDiagnostic> {
    let first_nonempty = stderr
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())?
        .to_string();
    let message = stderr
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with("error:"))
        .unwrap_or(&first_nonempty)
        .to_string();

    Some(CheckDiagnostic {
        level: "error".to_string(),
        message,
        code: Some("cargo-stderr".to_string()),
        package_id: None,
        target: None,
        rendered: Some(stderr.to_string()),
        spans: Vec::new(),
        suggestions: Vec::new(),
    })
}

fn span_from_value(span: &Value) -> Option<CheckSpan> {
    let text = span
        .get("text")
        .and_then(Value::as_array)
        .map(|lines| {
            lines
                .iter()
                .filter_map(|line| line.get("text").and_then(Value::as_str))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();

    Some(CheckSpan {
        file_name: span.get("file_name")?.as_str()?.to_string(),
        line_start: span.get("line_start")?.as_u64()?,
        line_end: span.get("line_end")?.as_u64()?,
        column_start: span.get("column_start")?.as_u64()?,
        column_end: span.get("column_end")?.as_u64()?,
        is_primary: span.get("is_primary")?.as_bool()?,
        text,
    })
}

fn collect_suggestions(diagnostic: &Value, suggestions: &mut Vec<CheckSuggestion>) {
    let level = diagnostic
        .get("level")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let message = diagnostic
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if let Some(spans) = diagnostic.get("spans").and_then(Value::as_array) {
        suggestions.extend(
            spans
                .iter()
                .filter_map(|span| suggestion_from_span(level, message, span)),
        );
    }
    if let Some(children) = diagnostic.get("children").and_then(Value::as_array) {
        for child in children {
            collect_suggestions(child, suggestions);
        }
    }
}

fn suggestion_from_span(level: &str, message: &str, span: &Value) -> Option<CheckSuggestion> {
    let suggested_replacement = span.get("suggested_replacement")?.as_str()?.to_string();
    Some(CheckSuggestion {
        level: level.to_string(),
        message: message.to_string(),
        file_name: span.get("file_name")?.as_str()?.to_string(),
        line_start: span.get("line_start")?.as_u64()?,
        line_end: span.get("line_end")?.as_u64()?,
        column_start: span.get("column_start")?.as_u64()?,
        column_end: span.get("column_end")?.as_u64()?,
        byte_start: span.get("byte_start").and_then(Value::as_u64),
        byte_end: span.get("byte_end").and_then(Value::as_u64),
        suggested_replacement,
        suggestion_applicability: span
            .get("suggestion_applicability")
            .and_then(Value::as_str)
            .map(str::to_string),
    })
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        sync::{Mutex, MutexGuard},
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

    use super::{
        check_workspace_with_program, classify_feedback, parse_cargo_messages,
        stderr_failure_diagnostic, timeout_failure_diagnostic, CheckDiagnostic, CheckOptions,
        CheckReport, CheckSpan, FeedbackWideningReport,
    };

    static FAKE_CARGO_TEST_LOCK: Mutex<()> = Mutex::new(());

    fn fake_cargo_test_lock() -> MutexGuard<'static, ()> {
        FAKE_CARGO_TEST_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[test]
    fn parses_compiler_message_diagnostics() {
        let json = r#"{"reason":"compiler-message","package_id":"broken 0.1.0 (path+file:///tmp/broken)","target":{"kind":["lib"],"crate_types":["lib"],"name":"broken","src_path":"/tmp/broken/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"message":{"rendered":"error[E0425]: cannot find value `missing` in this scope\n --> src/lib.rs:2:5\n","children":[{"children":[],"code":null,"level":"help","message":"consider importing this unit struct","rendered":null,"spans":[{"byte_end":0,"byte_start":0,"column_end":1,"column_start":1,"expansion":null,"file_name":"src/lib.rs","is_primary":true,"label":null,"line_end":1,"line_start":1,"suggested_replacement":"use crate::support::Worker;\n\n","suggestion_applicability":"MaybeIncorrect","text":[]}]}],"code":{"code":"E0425","explanation":null},"level":"error","message":"cannot find value `missing` in this scope","spans":[{"byte_end":32,"byte_start":25,"column_end":12,"column_start":5,"expansion":null,"file_name":"src/lib.rs","is_primary":true,"label":"not found in this scope","line_end":2,"line_start":2,"suggested_replacement":null,"suggestion_applicability":null,"text":[{"highlight_end":12,"highlight_start":5,"text":"    missing"}]}]}}"#;

        let diagnostics = parse_cargo_messages(json);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].level, "error");
        assert_eq!(diagnostics[0].code.as_deref(), Some("E0425"));
        assert_eq!(
            diagnostics[0].package_id.as_deref(),
            Some("broken 0.1.0 (path+file:///tmp/broken)")
        );
        let target = diagnostics[0]
            .target
            .as_ref()
            .expect("compiler-message target should be captured");
        assert_eq!(target.name, "broken");
        assert_eq!(target.kind, ["lib"]);
        assert_eq!(target.src_path.as_deref(), Some("/tmp/broken/src/lib.rs"));
        assert_eq!(
            diagnostics[0].message,
            "cannot find value `missing` in this scope"
        );
        assert_eq!(diagnostics[0].spans[0].file_name, "src/lib.rs");
        assert!(diagnostics[0].spans[0].is_primary);
        assert_eq!(diagnostics[0].suggestions.len(), 1);
        assert_eq!(
            diagnostics[0].suggestions[0].suggested_replacement,
            "use crate::support::Worker;\n\n"
        );
        assert_eq!(
            diagnostics[0].suggestions[0]
                .suggestion_applicability
                .as_deref(),
            Some("MaybeIncorrect")
        );
    }

    #[test]
    fn classifies_unresolved_compiler_errors_as_widening_candidates() {
        let diagnostics = vec![
            diagnostic(
                "E0432",
                "unresolved import `crate::worker::Task`",
                "src/lib.rs",
                7,
            ),
            diagnostic(
                "E0599",
                "no method named `run` found for struct `Worker` in the current scope",
                "src/lib.rs",
                11,
            ),
        ];

        let widening = classify_feedback(&diagnostics);

        assert_eq!(widening.candidates.len(), 2);
        assert!(widening.candidates.iter().any(|candidate| {
            candidate.kind == "unresolved-path"
                && candidate.symbol.as_deref() == Some("crate::worker::Task")
                && candidate.file_name.as_deref() == Some("src/lib.rs")
                && candidate.line_start == Some(7)
                && candidate.suggestions == 0
        }));
        assert!(widening.candidates.iter().any(|candidate| {
            candidate.kind == "missing-method-or-associated-item"
                && candidate.symbol.as_deref() == Some("run")
                && candidate.confidence == "medium"
        }));
        assert!(widening
            .hazards
            .iter()
            .any(|hazard| hazard.kind == "needs-widening"));
    }

    #[test]
    fn classifies_missing_conversion_impls_as_widening_candidates() {
        let diagnostics = vec![diagnostic(
            "E0277",
            "the trait bound `Target: From<Source>` is not satisfied",
            "src/lib.rs",
            17,
        )];

        let widening = classify_feedback(&diagnostics);

        assert_eq!(widening.candidates.len(), 1);
        assert_eq!(widening.candidates[0].kind, "missing-conversion-trait-impl");
        assert_eq!(
            widening.candidates[0].symbol.as_deref(),
            Some("Target: From<Source>")
        );
        assert!(widening.hazards.iter().any(
            |hazard| hazard.kind == "needs-widening" && hazard.code.as_deref() == Some("E0277")
        ));
    }

    #[test]
    fn classifies_timeout_and_cargo_stderr_as_blocking_hazards() {
        let diagnostics = vec![
            timeout_failure_diagnostic(Some(Duration::from_secs(5))),
            stderr_failure_diagnostic("error: failed to load manifest").unwrap(),
        ];

        let widening = classify_feedback(&diagnostics);

        assert!(widening.candidates.is_empty());
        assert_eq!(widening.hazards.len(), 2);
        assert!(widening
            .hazards
            .iter()
            .all(|hazard| hazard.severity == "blocker"));
    }

    #[test]
    fn classifies_semantic_warning_feedback_hazards() {
        let diagnostics = vec![
            warning(
                "unreachable_patterns",
                "unreachable pattern",
                "src/lib.rs",
                9,
            ),
            warning(
                "non_snake_case",
                "variable `HTTP_OK` should have a snake case name",
                "src/lib.rs",
                12,
            ),
        ];

        let widening = classify_feedback(&diagnostics);

        assert!(widening.candidates.is_empty());
        assert_eq!(widening.hazards.len(), 2);
        assert!(widening
            .hazards
            .iter()
            .all(|hazard| { hazard.kind == "semantic-warning" && hazard.severity == "high" }));
    }

    #[test]
    fn turns_stderr_only_cargo_failure_into_error_diagnostic() {
        let diagnostic = stderr_failure_diagnostic(
            "\nerror: failed to load manifest for workspace member `/tmp/out/member`\n\nCaused by:\n  missing dependency\n",
        )
        .expect("stderr should produce a diagnostic");

        assert_eq!(diagnostic.level, "error");
        assert_eq!(diagnostic.code.as_deref(), Some("cargo-stderr"));
        assert_eq!(
            diagnostic.message,
            "error: failed to load manifest for workspace member `/tmp/out/member`"
        );
        assert!(diagnostic.rendered.unwrap().contains("missing dependency"));
        assert!(diagnostic.spans.is_empty());
    }

    #[test]
    fn prefers_stderr_error_line_over_leading_warnings() {
        let diagnostic = stderr_failure_diagnostic(
            "warning: /tmp/out/Cargo.toml: unused manifest key: patch.crates-io.ohttp.revision\nerror: failed to select a version for `rc_crypto`\n",
        )
        .expect("stderr should produce a diagnostic");

        assert_eq!(
            diagnostic.message,
            "error: failed to select a version for `rc_crypto`"
        );
    }

    #[test]
    fn reports_feedback_timeout_as_structured_diagnostic() {
        let diagnostic = timeout_failure_diagnostic(Some(Duration::from_secs(42)));

        assert_eq!(diagnostic.level, "error");
        assert_eq!(diagnostic.code.as_deref(), Some("cargo-timeout"));
        assert_eq!(
            diagnostic.message,
            "cargo check exceeded feedback timeout of 42s"
        );
        assert!(diagnostic.spans.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn terminates_feedback_command_after_timeout() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = fake_cargo_test_lock();
        let root = std::env::temp_dir().join(format!(
            "opensourced-feedback-timeout-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let fake_cargo = root.join("fake-cargo");
        fs::write(
            &fake_cargo,
            "#!/bin/sh\nsleep 5\necho '{\"reason\":\"build-finished\",\"success\":true}'\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&fake_cargo).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&fake_cargo, permissions).unwrap();

        let report = check_workspace_with_program(
            fake_cargo.as_os_str(),
            CheckOptions {
                manifest_path: root.join("Cargo.toml"),
                target_dir: None,
                timeout: Some(Duration::from_millis(50)),
                cargo_args: Vec::new(),
            },
        )
        .expect("fake cargo should run");

        assert!(report.timed_out);
        assert!(!report.success);
        assert_eq!(report.diagnostics[0].code.as_deref(), Some("cargo-timeout"));

        let _ = fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn passes_extra_cargo_check_arguments_to_command() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = fake_cargo_test_lock();
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("opensourced-feedback-args-{unique}"));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let fake_cargo = root.join("fake-cargo");
        fs::write(
            &fake_cargo,
            "#!/bin/sh\nprintf 'ARG:%s\\n' \"$@\" >&2\necho '{\"reason\":\"build-finished\",\"success\":true}'\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&fake_cargo).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&fake_cargo, permissions).unwrap();

        let report = check_workspace_with_program(
            fake_cargo.as_os_str(),
            CheckOptions {
                manifest_path: root.join("Cargo.toml"),
                target_dir: None,
                timeout: Some(Duration::from_secs(1)),
                cargo_args: vec![
                    "--all-features".to_string(),
                    "--target".to_string(),
                    "wasm32-unknown-unknown".to_string(),
                ],
            },
        )
        .expect("fake cargo should run");

        assert!(report.success);
        assert_eq!(
            report.cargo_args,
            ["--all-features", "--target", "wasm32-unknown-unknown"]
        );
        assert_eq!(report.timeout_ms, Some(1_000));
        assert!(report.stderr.contains("ARG:--all-features"));
        assert!(report.stderr.contains("ARG:--target"));
        assert!(report.stderr.contains("ARG:wasm32-unknown-unknown"));

        let _ = fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn drains_large_cargo_json_output_while_waiting() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = fake_cargo_test_lock();
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("opensourced-feedback-drain-{unique}"));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let fake_cargo = root.join("fake-cargo");
        fs::write(
            &fake_cargo,
            "#!/bin/sh\n\
             i=0\n\
             while [ \"$i\" -lt 20000 ]; do\n\
             echo '{\"reason\":\"compiler-artifact\",\"package_id\":\"pkg 0.1.0\"}'\n\
             i=$((i + 1))\n\
             done\n\
             echo '{\"reason\":\"build-finished\",\"success\":true}'\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&fake_cargo).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&fake_cargo, permissions).unwrap();

        let report = check_workspace_with_program(
            fake_cargo.as_os_str(),
            CheckOptions {
                manifest_path: root.join("Cargo.toml"),
                target_dir: None,
                timeout: Some(Duration::from_secs(5)),
                cargo_args: Vec::new(),
            },
        )
        .expect("fake cargo should run");

        assert!(report.success);
        assert!(!report.timed_out);
        assert!(report.diagnostics.is_empty());

        let _ = fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn runs_cargo_check_from_manifest_parent() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = fake_cargo_test_lock();
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("opensourced-feedback-cwd-{unique}"));
        let workspace = root.join("workspace");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&workspace).unwrap();
        fs::write(workspace.join("Cargo.toml"), "[workspace]\n").unwrap();
        let fake_cargo = root.join("fake-cargo");
        fs::write(
            &fake_cargo,
            "#!/bin/sh\npwd >&2\necho '{\"reason\":\"build-finished\",\"success\":true}'\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&fake_cargo).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&fake_cargo, permissions).unwrap();

        let report = check_workspace_with_program(
            fake_cargo.as_os_str(),
            CheckOptions {
                manifest_path: workspace.join("Cargo.toml"),
                target_dir: None,
                timeout: Some(Duration::from_secs(1)),
                cargo_args: Vec::new(),
            },
        )
        .expect("fake cargo should run");

        let actual = PathBuf::from(report.stderr.trim()).canonicalize();
        let expected = workspace.canonicalize();
        assert_eq!(actual.unwrap(), expected.unwrap());
        assert_eq!(report.working_dir.as_deref(), Some(workspace.as_path()));

        let _ = fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn absolutizes_relative_target_dir_before_changing_working_dir() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = fake_cargo_test_lock();
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("opensourced-feedback-target-{unique}"));
        let workspace = root.join("workspace");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&workspace).unwrap();
        fs::write(workspace.join("Cargo.toml"), "[workspace]\n").unwrap();
        let fake_cargo = root.join("fake-cargo");
        fs::write(
            &fake_cargo,
            "#!/bin/sh\nprintf '%s' \"$CARGO_TARGET_DIR\" >&2\necho '{\"reason\":\"build-finished\",\"success\":true}'\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&fake_cargo).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&fake_cargo, permissions).unwrap();

        let relative_target = PathBuf::from("relative-feedback-target");
        let expected_target = std::env::current_dir().unwrap().join(&relative_target);
        let report = check_workspace_with_program(
            fake_cargo.as_os_str(),
            CheckOptions {
                manifest_path: workspace.join("Cargo.toml"),
                target_dir: Some(relative_target),
                timeout: Some(Duration::from_secs(1)),
                cargo_args: Vec::new(),
            },
        )
        .expect("fake cargo should run");

        assert_eq!(report.stderr, expected_target.display().to_string());
        assert_eq!(
            report.target_dir.as_deref(),
            Some(expected_target.as_path())
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn serializes_feedback_invocation_context() {
        let report = CheckReport {
            manifest_path: PathBuf::from("/tmp/slice/Cargo.toml"),
            working_dir: Some(PathBuf::from("/tmp/slice")),
            target_dir: Some(PathBuf::from("/tmp/slice-target")),
            timeout_ms: Some(600_000),
            cargo_args: vec!["--all-targets".to_string()],
            success: true,
            timed_out: false,
            exit_code: 0,
            duration_ms: 12,
            diagnostics: Vec::new(),
            widening: FeedbackWideningReport::default(),
            stderr: String::new(),
        };

        let value: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&report).unwrap())
                .expect("report should serialize");

        assert_eq!(value["target_dir"], "/tmp/slice-target");
        assert_eq!(value["working_dir"], "/tmp/slice");
        assert_eq!(value["timeout_ms"], 600_000);
        assert_eq!(value["cargo_args"][0], "--all-targets");
    }

    fn diagnostic(code: &str, message: &str, file_name: &str, line_start: u64) -> CheckDiagnostic {
        check_diagnostic("error", code, message, file_name, line_start)
    }

    fn warning(code: &str, message: &str, file_name: &str, line_start: u64) -> CheckDiagnostic {
        check_diagnostic("warning", code, message, file_name, line_start)
    }

    fn check_diagnostic(
        level: &str,
        code: &str,
        message: &str,
        file_name: &str,
        line_start: u64,
    ) -> CheckDiagnostic {
        CheckDiagnostic {
            level: level.to_string(),
            message: message.to_string(),
            code: Some(code.to_string()),
            package_id: None,
            target: None,
            rendered: None,
            spans: vec![CheckSpan {
                file_name: file_name.to_string(),
                line_start,
                line_end: line_start,
                column_start: 1,
                column_end: 1,
                is_primary: true,
                text: Vec::new(),
            }],
            suggestions: Vec::new(),
        }
    }
}
