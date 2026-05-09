pub struct DeadBtreesetDifferenceItem {
    value: String,
}

impl DeadBtreesetDifferenceItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-btreeset-difference:{}", self.value)
    }
}

pub fn dead_btreeset_difference(raw: &str) -> String {
    DeadBtreesetDifferenceItem::new(raw).render()
}
