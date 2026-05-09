use std::path::{Component, Path};

pub struct PathComponentsFilterMapPayload {
    value: String,
}

impl PathComponentsFilterMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("path-components-filter-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-path-components-filter-map:{}", self.value)
    }
}

pub fn selected_path_components_filter_map(raw: &str) -> String {
    Path::new(raw)
        .components()
        .filter_map(|component| match component {
            Component::Normal(part) => {
                Some(PathComponentsFilterMapPayload::new(&part.to_string_lossy()).render_label())
            }
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_path_components_filter_map(raw: &str) -> String {
    PathComponentsFilterMapPayload::new(raw).unused_label()
}
