pub struct DeadPathbufSetFileNameMapItem {
    value: String,
}

impl DeadPathbufSetFileNameMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-pathbuf-set-file-name-map:{}", self.value)
    }
}

pub fn dead_pathbuf_set_file_name_map(raw: &str) -> String {
    DeadPathbufSetFileNameMapItem::new(raw).dead_method()
}
