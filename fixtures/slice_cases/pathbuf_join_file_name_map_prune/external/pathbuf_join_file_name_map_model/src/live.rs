use std::path::PathBuf;
pub struct PathbufJoinFileNameMapPayload {
    value: String,
}

impl PathbufJoinFileNameMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("pathbuf-join-file-name-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("pathbuf-join-file-name-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-pathbuf-join-file-name-map:{}", self.value)
    }
}

pub fn selected_pathbuf_join_file_name_map(raw: &str) -> String {
    let path = PathBuf::from(raw).join("tail.txt");
    path.file_name()
        .and_then(|name| name.to_str())
        .map(PathbufJoinFileNameMapPayload::new)
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_pathbuf_join_file_name_map(raw: &str) -> String {
    PathbufJoinFileNameMapPayload::new(raw).unused_label()
}
