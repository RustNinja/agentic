pub struct DeadBtreemapAppendValuesLastMapItem {
    value: String,
}

impl DeadBtreemapAppendValuesLastMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-append-values-last-map:{}", self.value)
    }
}

pub fn dead_btreemap_append_values_last_map(raw: &str) -> String {
    DeadBtreemapAppendValuesLastMapItem::new(raw).dead_method()
}
