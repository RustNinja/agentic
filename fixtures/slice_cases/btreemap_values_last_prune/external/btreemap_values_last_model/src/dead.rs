pub struct DeadBtreemapValuesLastItem {
    value: String,
}

impl DeadBtreemapValuesLastItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-values-last:{}", self.value)
    }
}

pub fn dead_btreemap_values_last(raw: &str) -> String {
    DeadBtreemapValuesLastItem::new(raw).dead_method()
}
