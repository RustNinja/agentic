pub struct DeadOptionFilterItem {
    value: String,
}

impl DeadOptionFilterItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-option-filter:{}", self.value)
    }
}

pub fn dead_option_filter(raw: &str) -> String {
    DeadOptionFilterItem::new(raw).render()
}
