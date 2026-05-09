pub struct DeadBtreemapLastKeyValueItem {
    value: String,
}

impl DeadBtreemapLastKeyValueItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-btreemap-last-key-value:{}", self.value)
    }
}

pub fn dead_btreemap_last_key_value(raw: &str) -> String {
    DeadBtreemapLastKeyValueItem::new(raw).render()
}
