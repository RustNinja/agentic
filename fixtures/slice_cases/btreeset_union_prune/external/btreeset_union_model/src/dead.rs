pub struct DeadBtreesetUnionItem {
    value: String,
}

impl DeadBtreesetUnionItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-btreeset-union:{}", self.value)
    }
}

pub fn dead_btreeset_union(raw: &str) -> String {
    DeadBtreesetUnionItem::new(raw).render()
}
