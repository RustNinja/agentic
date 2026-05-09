use std::ffi::OsStr;
use std::path::Path;
pub struct PathFileStemMapPayload {
    value: String,
}

impl PathFileStemMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("path-file-stem-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("path-file-stem-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-path-file-stem-map:{}", self.value)
    }
}

pub fn selected_path_file_stem_map(raw: &str) -> String {
    Path::new(raw)
        .file_stem()
        .and_then(OsStr::to_str)
        .map(|part| PathFileStemMapPayload::new(part).render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_path_file_stem_map(raw: &str) -> String {
    PathFileStemMapPayload::new(raw).unused_label()
}
