pub struct DeadOptionTakeIfMapItem {
    value: String,
}

impl DeadOptionTakeIfMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-option-take-if-map:{}", self.value)
    }
}

pub fn dead_option_take_if_map(raw: &str) -> String {
    DeadOptionTakeIfMapItem::new(raw).render()
}
