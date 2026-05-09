pub struct DeadPathFileNameMapItem {
    value: String,
}

impl DeadPathFileNameMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-path-file-name-map:{}", self.value)
    }
}

pub fn dead_path_file_name_map(raw: &str) -> String {
    DeadPathFileNameMapItem::new(raw).dead_method()
}
