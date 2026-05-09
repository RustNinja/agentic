pub struct DeadPathIterFilterMapItem {
    value: String,
}

impl DeadPathIterFilterMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-path-iter-filter-map:{}", self.value)
    }
}

pub fn dead_path_iter_filter_map(raw: &str) -> String {
    DeadPathIterFilterMapItem::new(raw).dead_method()
}
