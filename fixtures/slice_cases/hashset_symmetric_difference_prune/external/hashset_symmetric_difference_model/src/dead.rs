pub struct DeadHashsetSymmetricDifferenceItem {
    value: String,
}

impl DeadHashsetSymmetricDifferenceItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-hashset-symmetric-difference:{}", self.value)
    }
}

pub fn dead_hashset_symmetric_difference(raw: &str) -> String {
    DeadHashsetSymmetricDifferenceItem::new(raw).render()
}
