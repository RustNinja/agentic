pub struct DeadOptionReplaceMapItem {
    value: String,
}

impl DeadOptionReplaceMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-option-replace-map:{}", self.value)
    }
}

pub fn dead_option_replace_map(raw: &str) -> String {
    DeadOptionReplaceMapItem::new(raw).render()
}
