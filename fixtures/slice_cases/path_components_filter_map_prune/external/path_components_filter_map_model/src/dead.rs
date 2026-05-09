pub struct DeadPathComponentsFilterMapItem {
    value: String,
}

impl DeadPathComponentsFilterMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-path-components-filter-map:{}", self.value)
    }
}

pub fn dead_path_components_filter_map(raw: &str) -> String {
    DeadPathComponentsFilterMapItem::new(raw).dead_method()
}
