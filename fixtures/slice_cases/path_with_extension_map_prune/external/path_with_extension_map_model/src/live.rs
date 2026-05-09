use std::path::Path;
pub struct PathWithExtensionMapPayload {
    value: String,
}

impl PathWithExtensionMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("path-with-extension-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("path-with-extension-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-path-with-extension-map:{}", self.value)
    }
}

pub fn selected_path_with_extension_map(raw: &str) -> String {
    let renamed = Path::new(raw).with_extension("slice");
    renamed
        .to_str()
        .map(|part| PathWithExtensionMapPayload::new(part).render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_path_with_extension_map(raw: &str) -> String {
    PathWithExtensionMapPayload::new(raw).unused_label()
}
