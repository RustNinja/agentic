use std::path::Path;
pub struct PathAncestorsFindMapPayload {
    value: String,
}

impl PathAncestorsFindMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("path-ancestors-find-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("path-ancestors-find-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-path-ancestors-find-map:{}", self.value)
    }
}

pub fn selected_path_ancestors_find_map(raw: &str) -> String {
    Path::new(raw)
        .ancestors()
        .find_map(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|part| PathAncestorsFindMapPayload::new(part).render_label())
        })
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_path_ancestors_find_map(raw: &str) -> String {
    PathAncestorsFindMapPayload::new(raw).unused_label()
}
