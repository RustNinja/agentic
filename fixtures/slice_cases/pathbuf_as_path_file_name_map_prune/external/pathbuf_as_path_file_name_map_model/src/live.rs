use std::path::PathBuf;
pub struct PathbufAsPathFileNameMapPayload {
    value: String,
}

impl PathbufAsPathFileNameMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("pathbuf-as-path-file-name-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("pathbuf-as-path-file-name-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-pathbuf-as-path-file-name-map:{}", self.value)
    }
}

pub fn selected_pathbuf_as_path_file_name_map(raw: &str) -> String {
    let path = PathBuf::from(raw).join("file.txt");
    path.as_path()
        .file_name()
        .and_then(|name| name.to_str())
        .map(|value| PathbufAsPathFileNameMapPayload::new(value).render_label())
        .unwrap_or_default()
}

pub fn dead_live_pathbuf_as_path_file_name_map(raw: &str) -> String {
    PathbufAsPathFileNameMapPayload::new(raw).unused_label()
}
