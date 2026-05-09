pub struct DeadHashsetDifferenceItem {
    value: String,
}

impl DeadHashsetDifferenceItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-hashset-difference:{}", self.value)
    }
}

pub fn dead_hashset_difference(raw: &str) -> String {
    DeadHashsetDifferenceItem::new(raw).render()
}
