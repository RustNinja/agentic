use std::ffi::OsStr;
use std::path::Path;
pub struct PathIterFilterMapPayload {
    value: String,
}

impl PathIterFilterMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("path-iter-filter-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("path-iter-filter-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-path-iter-filter-map:{}", self.value)
    }
}

pub fn selected_path_iter_filter_map(raw: &str) -> String {
    Path::new(raw)
        .iter()
        .filter_map(OsStr::to_str)
        .map(|part| PathIterFilterMapPayload::new(part).render_label())
        .next()
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_path_iter_filter_map(raw: &str) -> String {
    PathIterFilterMapPayload::new(raw).unused_label()
}
