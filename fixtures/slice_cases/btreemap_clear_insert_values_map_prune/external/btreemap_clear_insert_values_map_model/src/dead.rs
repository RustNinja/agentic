pub struct DeadBtreemapClearInsertValuesMapItem {
    value: String,
}

impl DeadBtreemapClearInsertValuesMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-clear-insert-values-map:{}", self.value)
    }
}

pub fn dead_btreemap_clear_insert_values_map(raw: &str) -> String {
    DeadBtreemapClearInsertValuesMapItem::new(raw).dead_method()
}
