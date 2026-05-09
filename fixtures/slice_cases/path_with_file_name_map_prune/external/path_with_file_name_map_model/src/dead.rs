pub struct DeadPathWithFileNameMapItem {
    value: String,
}

impl DeadPathWithFileNameMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-path-with-file-name-map:{}", self.value)
    }
}

pub fn dead_path_with_file_name_map(raw: &str) -> String {
    DeadPathWithFileNameMapItem::new(raw).dead_method()
}
