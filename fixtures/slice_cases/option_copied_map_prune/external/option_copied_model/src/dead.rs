pub struct DeadOptionCopiedMapItem {
    value: String,
}

impl DeadOptionCopiedMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-option-copied-map:{}", self.value)
    }
}

pub fn dead_option_copied_map(raw: &str) -> String {
    DeadOptionCopiedMapItem::new(raw).render()
}
