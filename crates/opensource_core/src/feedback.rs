use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct CheckOptions {
    pub manifest_path: PathBuf,
    pub target_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckReport {
    pub manifest_path: PathBuf,
    pub success: bool,
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
    let mut command = Command::new("cargo");
    command
        .arg("check")
        .arg("--manifest-path")
        .arg(&options.manifest_path)
        .arg("--message-format=json");

    if let Some(target_dir) = &options.target_dir {
        command.env("CARGO_TARGET_DIR", target_dir);
    }

    let output = command.output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let diagnostics = parse_cargo_messages(&stdout);

    Ok(CheckReport {
        manifest_path: options.manifest_path,
        success: output.status.success(),
        diagnostics,
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
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
    use super::parse_cargo_messages;

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
}
