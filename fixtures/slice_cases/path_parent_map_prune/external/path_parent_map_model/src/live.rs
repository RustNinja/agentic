use std::path::Path;
pub struct PathParentMapPayload {
    value: String,
}

impl PathParentMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("path-parent-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("path-parent-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-path-parent-map:{}", self.value)
    }
}

pub fn selected_path_parent_map(raw: &str) -> String {
    Path::new(raw)
        .parent()
        .and_then(|path| path.to_str())
        .map(|part| PathParentMapPayload::new(part).render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_path_parent_map(raw: &str) -> String {
    PathParentMapPayload::new(raw).unused_label()
}
