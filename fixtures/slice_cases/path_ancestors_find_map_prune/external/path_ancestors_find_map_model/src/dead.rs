pub struct DeadPathAncestorsFindMapItem {
    value: String,
}

impl DeadPathAncestorsFindMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-path-ancestors-find-map:{}", self.value)
    }
}

pub fn dead_path_ancestors_find_map(raw: &str) -> String {
    DeadPathAncestorsFindMapItem::new(raw).dead_method()
}
