pub struct DeadBtreemapIntoValuesNextItem {
    value: String,
}

impl DeadBtreemapIntoValuesNextItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-into-values-next:{}", self.value)
    }
}

pub fn dead_btreemap_into_values_next(raw: &str) -> String {
    DeadBtreemapIntoValuesNextItem::new(raw).dead_method()
}
