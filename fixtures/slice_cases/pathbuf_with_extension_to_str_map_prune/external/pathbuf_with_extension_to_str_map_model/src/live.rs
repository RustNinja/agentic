use std::path::PathBuf;
pub struct PathbufWithExtensionToStrMapPayload {
    value: String,
}

impl PathbufWithExtensionToStrMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("pathbuf-with-extension-to-str-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("pathbuf-with-extension-to-str-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-pathbuf-with-extension-to-str-map:{}", self.value)
    }
}

pub fn selected_pathbuf_with_extension_to_str_map(raw: &str) -> String {
    let path = PathBuf::from(raw).with_extension("log");
    path.to_str()
        .map(PathbufWithExtensionToStrMapPayload::new)
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_pathbuf_with_extension_to_str_map(raw: &str) -> String {
    PathbufWithExtensionToStrMapPayload::new(raw).unused_label()
}
