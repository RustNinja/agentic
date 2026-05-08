pub struct DeadEnumMatchItem {
    value: String,
}

impl DeadEnumMatchItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-enum-match:{}", self.value)
    }
}

pub fn dead_enum_match(raw: &str) -> String {
    DeadEnumMatchItem::new(raw).render()
}
