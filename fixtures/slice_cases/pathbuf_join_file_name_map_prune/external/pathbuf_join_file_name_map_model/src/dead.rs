pub struct DeadPathbufJoinFileNameMapItem {
    value: String,
}

impl DeadPathbufJoinFileNameMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-pathbuf-join-file-name-map:{}", self.value)
    }
}

pub fn dead_pathbuf_join_file_name_map(raw: &str) -> String {
    DeadPathbufJoinFileNameMapItem::new(raw).dead_method()
}
