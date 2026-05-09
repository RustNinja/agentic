use std::path::PathBuf;
pub struct PathbufSetFileNameMapPayload {
    value: String,
}

impl PathbufSetFileNameMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("pathbuf-set-file-name-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("pathbuf-set-file-name-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-pathbuf-set-file-name-map:{}", self.value)
    }
}

pub fn selected_pathbuf_set_file_name_map(raw: &str) -> String {
    let mut path = PathBuf::from(raw);
    path.set_file_name("renamed.txt");
    path.to_str()
        .map(PathbufSetFileNameMapPayload::new)
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_pathbuf_set_file_name_map(raw: &str) -> String {
    PathbufSetFileNameMapPayload::new(raw).unused_label()
}
