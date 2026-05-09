use std::ffi::OsStr;
use std::path::Path;
pub struct PathExtensionMapPayload {
    value: String,
}

impl PathExtensionMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("path-extension-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("path-extension-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-path-extension-map:{}", self.value)
    }
}

pub fn selected_path_extension_map(raw: &str) -> String {
    Path::new(raw)
        .extension()
        .and_then(OsStr::to_str)
        .map(|part| PathExtensionMapPayload::new(part).render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_path_extension_map(raw: &str) -> String {
    PathExtensionMapPayload::new(raw).unused_label()
}
