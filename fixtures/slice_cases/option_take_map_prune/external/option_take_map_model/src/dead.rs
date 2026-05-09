pub struct DeadOptionTakeMapItem {
    value: String,
}

impl DeadOptionTakeMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-option-take-map:{}", self.value)
    }
}

pub fn dead_option_take_map(raw: &str) -> String {
    DeadOptionTakeMapItem::new(raw).render()
}
