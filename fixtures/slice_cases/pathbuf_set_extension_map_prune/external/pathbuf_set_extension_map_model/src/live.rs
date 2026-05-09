use std::path::PathBuf;
pub struct PathbufSetExtensionMapPayload {
    value: String,
}

impl PathbufSetExtensionMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("pathbuf-set-extension-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("pathbuf-set-extension-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-pathbuf-set-extension-map:{}", self.value)
    }
}

pub fn selected_pathbuf_set_extension_map(raw: &str) -> String {
    let mut path = PathBuf::from(raw);
    path.set_extension("log");
    path.to_str()
        .map(PathbufSetExtensionMapPayload::new)
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_pathbuf_set_extension_map(raw: &str) -> String {
    PathbufSetExtensionMapPayload::new(raw).unused_label()
}
