pub struct DeadBtreesetSymmetricDifferenceItem {
    value: String,
}

impl DeadBtreesetSymmetricDifferenceItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-btreeset-symmetric-difference:{}", self.value)
    }
}

pub fn dead_btreeset_symmetric_difference(raw: &str) -> String {
    DeadBtreesetSymmetricDifferenceItem::new(raw).render()
}
