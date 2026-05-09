pub struct DeadOptionOkItem {
    value: String,
}

impl DeadOptionOkItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-option-ok:{}", self.value)
    }
}

pub fn dead_option_ok(raw: &str) -> String {
    DeadOptionOkItem::new(raw).render()
}
