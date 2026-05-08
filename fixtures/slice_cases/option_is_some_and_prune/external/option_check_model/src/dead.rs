pub struct DeadOptionCheckItem {
    value: String,
}

impl DeadOptionCheckItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-option-check:{}", self.value)
    }
}

pub fn dead_option_check(raw: &str) -> String {
    DeadOptionCheckItem::new(raw).render()
}
