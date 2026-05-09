pub struct DeadOptionGetOrInsertMapItem {
    value: String,
}

impl DeadOptionGetOrInsertMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-option-get-or-insert-map:{}", self.value)
    }
}

pub fn dead_option_get_or_insert_map(raw: &str) -> String {
    DeadOptionGetOrInsertMapItem::new(raw).render()
}
