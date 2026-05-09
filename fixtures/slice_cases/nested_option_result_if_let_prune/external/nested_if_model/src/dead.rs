pub struct DeadNestedIfItem {
    value: String,
}

impl DeadNestedIfItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-nested-if:{}", self.value)
    }
}

pub fn dead_nested_if(raw: &str) -> String {
    DeadNestedIfItem::new(raw).render()
}
