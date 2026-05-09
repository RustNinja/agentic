pub struct DeadPathParentMapItem {
    value: String,
}

impl DeadPathParentMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-path-parent-map:{}", self.value)
    }
}

pub fn dead_path_parent_map(raw: &str) -> String {
    DeadPathParentMapItem::new(raw).dead_method()
}
