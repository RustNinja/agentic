use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
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
    let mut command = Command::new(program);
    command
        .arg("check")
        .arg("--manifest-path")
        .arg(&options.manifest_path)
        .arg("--message-format=json");
    command.args(&options.cargo_args);

    if let Some(target_dir) = &options.target_dir {
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
        target_dir: options.target_dir,
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
        "E0432" | "E0433" | "E0405" | "E0412" | "E0422" | "E0425" | "E0599" => {
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
    let started = Instant::now();
    let Some(timeout) = timeout else {
        let output = child.wait_with_output()?;
        return Ok(CommandOutcome {
            output,
            timed_out: false,
            duration_ms: elapsed_ms(started),
        });
    };

    loop {
        if child.try_wait()?.is_some() {
            let output = child.wait_with_output()?;
            return Ok(CommandOutcome {
                output,
                timed_out: false,
                duration_ms: elapsed_ms(started),
            });
        }
        if started.elapsed() >= timeout {
            kill_timed_out_child(&mut child);
            let output = child.wait_with_output()?;
            return Ok(CommandOutcome {
                output,
                timed_out: true,
                duration_ms: elapsed_ms(started),
            });
        }
        thread::sleep(Duration::from_millis(50));
    }
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

    Some(CheckDiagnostic {
        level,
        message: message_text,
        code,
        package_id,
        target,
        rendered,
        spans,
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

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

    use super::{
        check_workspace_with_program, classify_feedback, parse_cargo_messages,
        stderr_failure_diagnostic, timeout_failure_diagnostic, CheckDiagnostic, CheckOptions,
        CheckReport, CheckSpan, FeedbackWideningReport,
    };

    #[test]
    fn parses_compiler_message_diagnostics() {
        let json = r#"{"reason":"compiler-message","package_id":"broken 0.1.0 (path+file:///tmp/broken)","target":{"kind":["lib"],"crate_types":["lib"],"name":"broken","src_path":"/tmp/broken/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"message":{"rendered":"error[E0425]: cannot find value `missing` in this scope\n --> src/lib.rs:2:5\n","children":[],"code":{"code":"E0425","explanation":null},"level":"error","message":"cannot find value `missing` in this scope","spans":[{"byte_end":32,"byte_start":25,"column_end":12,"column_start":5,"expansion":null,"file_name":"src/lib.rs","is_primary":true,"label":"not found in this scope","line_end":2,"line_start":2,"suggested_replacement":null,"suggestion_applicability":null,"text":[{"highlight_end":12,"highlight_start":5,"text":"    missing"}]}]}}"#;

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

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("opensourced-feedback-args-{unique}"));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let fake_cargo = root.join("fake-cargo");
        let args_path = root.join("args.txt");
        fs::write(
            &fake_cargo,
            format!(
                "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\necho '{{\"reason\":\"build-finished\",\"success\":true}}'\n",
                args_path.display()
            ),
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

        let args = fs::read_to_string(&args_path).unwrap();
        assert!(report.success);
        assert_eq!(
            report.cargo_args,
            ["--all-features", "--target", "wasm32-unknown-unknown"]
        );
        assert_eq!(report.timeout_ms, Some(1_000));
        assert!(args.contains("--all-features"));
        assert!(args.contains("--target\nwasm32-unknown-unknown"));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn serializes_feedback_invocation_context() {
        let report = CheckReport {
            manifest_path: PathBuf::from("/tmp/slice/Cargo.toml"),
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
        }
    }
}
