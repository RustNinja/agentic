pub struct DeadNestedMatchItem {
    value: String,
}

impl DeadNestedMatchItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-nested-match:{}", self.value)
    }
}

pub fn dead_nested_match(raw: &str) -> String {
    DeadNestedMatchItem::new(raw).render()
}
