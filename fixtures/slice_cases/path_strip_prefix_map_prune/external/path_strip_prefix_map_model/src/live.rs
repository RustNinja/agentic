use std::path::Path;
pub struct PathStripPrefixMapPayload {
    value: String,
}

impl PathStripPrefixMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("path-strip-prefix-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("path-strip-prefix-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-path-strip-prefix-map:{}", self.value)
    }
}

pub fn selected_path_strip_prefix_map(raw: &str) -> String {
    Path::new(raw)
        .strip_prefix("/tmp")
        .ok()
        .and_then(|path| path.to_str())
        .map(|part| PathStripPrefixMapPayload::new(part).render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_path_strip_prefix_map(raw: &str) -> String {
    PathStripPrefixMapPayload::new(raw).unused_label()
}
