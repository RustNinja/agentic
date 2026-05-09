pub struct DeadBtreemapValuesFilterMapItem {
    value: String,
}

impl DeadBtreemapValuesFilterMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-values-filter-map:{}", self.value)
    }
}

pub fn dead_btreemap_values_filter_map(raw: &str) -> String {
    DeadBtreemapValuesFilterMapItem::new(raw).dead_method()
}
