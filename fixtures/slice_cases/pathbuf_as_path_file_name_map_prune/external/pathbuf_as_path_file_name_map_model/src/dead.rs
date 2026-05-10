pub struct DeadPathbufAsPathFileNameMapItem {
    value: String,
}

impl DeadPathbufAsPathFileNameMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-pathbuf-as-path-file-name-map:{}", self.value)
    }
}

pub fn dead_pathbuf_as_path_file_name_map(raw: &str) -> String {
    DeadPathbufAsPathFileNameMapItem::new(raw).dead_method()
}
