pub struct DeadPathFileStemMapItem {
    value: String,
}

impl DeadPathFileStemMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-path-file-stem-map:{}", self.value)
    }
}

pub fn dead_path_file_stem_map(raw: &str) -> String {
    DeadPathFileStemMapItem::new(raw).dead_method()
}
