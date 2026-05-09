pub struct DeadBtreemapAppendValuesItem {
    value: String,
}

impl DeadBtreemapAppendValuesItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-btreemap-append-values:{}", self.value)
    }
}

pub fn dead_btreemap_append_values(raw: &str) -> String {
    DeadBtreemapAppendValuesItem::new(raw).render()
}
