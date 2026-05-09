pub struct DeadOptionIfItem {
    value: String,
}

impl DeadOptionIfItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-option-if:{}", self.value)
    }
}

pub fn dead_option_if(raw: &str) -> String {
    DeadOptionIfItem::new(raw).render()
}
