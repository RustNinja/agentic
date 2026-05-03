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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckReport {
    pub manifest_path: PathBuf,
    pub success: bool,
    pub timed_out: bool,
    pub diagnostics: Vec<CheckDiagnostic>,
    pub stderr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckDiagnostic {
    pub level: String,
    pub message: String,
    pub code: Option<String>,
    pub rendered: Option<String>,
    pub spans: Vec<CheckSpan>,
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

    if let Some(target_dir) = &options.target_dir {
        command.env("CARGO_TARGET_DIR", target_dir);
    }

    let (output, timed_out) = run_command(command, options.timeout)?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let mut diagnostics = parse_cargo_messages(&stdout);
    if timed_out {
        diagnostics.push(timeout_failure_diagnostic(options.timeout));
    }
    if !output.status.success() && diagnostics.is_empty() {
        if let Some(diagnostic) = stderr_failure_diagnostic(&stderr) {
            diagnostics.push(diagnostic);
        }
    }

    Ok(CheckReport {
        manifest_path: options.manifest_path,
        success: output.status.success(),
        timed_out,
        diagnostics,
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
        .filter_map(|message| diagnostic_from_value(message.get("message")?))
        .collect()
}

fn run_command(
    mut command: Command,
    timeout: Option<Duration>,
) -> Result<(std::process::Output, bool), Box<dyn std::error::Error>> {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    configure_timeout_process_group(&mut command);
    let mut child = command.spawn()?;
    let Some(timeout) = timeout else {
        return Ok((child.wait_with_output()?, false));
    };
    let started = Instant::now();

    loop {
        if child.try_wait()?.is_some() {
            return Ok((child.wait_with_output()?, false));
        }
        if started.elapsed() >= timeout {
            kill_timed_out_child(&mut child);
            return Ok((child.wait_with_output()?, true));
        }
        thread::sleep(Duration::from_millis(50));
    }
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

fn diagnostic_from_value(message: &Value) -> Option<CheckDiagnostic> {
    let level = message.get("level")?.as_str()?.to_string();
    let message_text = message.get("message")?.as_str()?.to_string();
    let code = message
        .get("code")
        .and_then(|code| code.get("code"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let rendered = message
        .get("rendered")
        .and_then(Value::as_str)
        .map(str::to_string);
    let spans = message
        .get("spans")
        .and_then(Value::as_array)
        .map(|spans| spans.iter().filter_map(span_from_value).collect())
        .unwrap_or_default();

    Some(CheckDiagnostic {
        level,
        message: message_text,
        code,
        rendered,
        spans,
    })
}

fn timeout_failure_diagnostic(timeout: Option<Duration>) -> CheckDiagnostic {
    let seconds = timeout.map(|timeout| timeout.as_secs()).unwrap_or_default();
    CheckDiagnostic {
        level: "error".to_string(),
        message: format!("cargo check exceeded feedback timeout of {seconds}s"),
        code: Some("cargo-timeout".to_string()),
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
    use std::{fs, time::Duration};

    use super::{
        check_workspace_with_program, parse_cargo_messages, stderr_failure_diagnostic,
        timeout_failure_diagnostic, CheckOptions,
    };

    #[test]
    fn parses_compiler_message_diagnostics() {
        let json = r#"{"reason":"compiler-message","package_id":"broken 0.1.0 (path+file:///tmp/broken)","target":{"kind":["lib"],"crate_types":["lib"],"name":"broken","src_path":"/tmp/broken/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"message":{"rendered":"error[E0425]: cannot find value `missing` in this scope\n --> src/lib.rs:2:5\n","children":[],"code":{"code":"E0425","explanation":null},"level":"error","message":"cannot find value `missing` in this scope","spans":[{"byte_end":32,"byte_start":25,"column_end":12,"column_start":5,"expansion":null,"file_name":"src/lib.rs","is_primary":true,"label":"not found in this scope","line_end":2,"line_start":2,"suggested_replacement":null,"suggestion_applicability":null,"text":[{"highlight_end":12,"highlight_start":5,"text":"    missing"}]}]}}"#;

        let diagnostics = parse_cargo_messages(json);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].level, "error");
        assert_eq!(diagnostics[0].code.as_deref(), Some("E0425"));
        assert_eq!(
            diagnostics[0].message,
            "cannot find value `missing` in this scope"
        );
        assert_eq!(diagnostics[0].spans[0].file_name, "src/lib.rs");
        assert!(diagnostics[0].spans[0].is_primary);
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
            },
        )
        .expect("fake cargo should run");

        assert!(report.timed_out);
        assert!(!report.success);
        assert_eq!(report.diagnostics[0].code.as_deref(), Some("cargo-timeout"));

        let _ = fs::remove_dir_all(root);
    }
}
