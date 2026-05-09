pub struct DeadBtreemapFirstKeyValueItem {
    value: String,
}

impl DeadBtreemapFirstKeyValueItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-btreemap-first-key-value:{}", self.value)
    }
}

pub fn dead_btreemap_first_key_value(raw: &str) -> String {
    DeadBtreemapFirstKeyValueItem::new(raw).render()
}
