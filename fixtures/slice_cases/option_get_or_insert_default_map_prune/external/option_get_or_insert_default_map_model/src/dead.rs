pub struct DeadOptionGetOrInsertDefaultMapItem {
    value: String,
}

impl DeadOptionGetOrInsertDefaultMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-option-get-or-insert-default-map:{}", self.value)
    }
}

pub fn dead_option_get_or_insert_default_map(raw: &str) -> String {
    DeadOptionGetOrInsertDefaultMapItem::new(raw).render()
}
