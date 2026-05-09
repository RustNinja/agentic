use std::path::Path;

pub struct PathFileNameMapPayload {
    value: String,
}

impl PathFileNameMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("path-file-name-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-path-file-name-map:{}", self.value)
    }
}

pub fn selected_path_file_name_map(raw: &str) -> String {
    Path::new(raw)
        .file_name()
        .and_then(|name| name.to_str())
        .map(PathFileNameMapPayload::new)
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_path_file_name_map(raw: &str) -> String {
    PathFileNameMapPayload::new(raw).unused_label()
}
