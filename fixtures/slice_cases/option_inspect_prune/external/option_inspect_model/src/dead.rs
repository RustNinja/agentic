pub struct DeadOptionInspectItem {
    value: String,
}

impl DeadOptionInspectItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-option-inspect:{}", self.value)
    }
}

pub fn dead_option_inspect(raw: &str) -> String {
    DeadOptionInspectItem::new(raw).render()
}
