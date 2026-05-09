pub struct DeadOptionRefItem {
    value: String,
}

impl DeadOptionRefItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-option-ref:{}", self.value)
    }
}

pub fn dead_option_ref(raw: &str) -> String {
    DeadOptionRefItem::new(raw).render()
}
