pub struct DeadOptionAndItem {
    value: String,
}

impl DeadOptionAndItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-option-and:{}", self.value)
    }
}

pub fn dead_option_and_then(raw: &str) -> String {
    DeadOptionAndItem::new(raw).render()
}
