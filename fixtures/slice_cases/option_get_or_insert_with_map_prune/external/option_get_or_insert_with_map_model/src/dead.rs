pub struct DeadOptionGetOrInsertWithMapItem {
    value: String,
}

impl DeadOptionGetOrInsertWithMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-option-get-or-insert-with-map:{}", self.value)
    }
}

pub fn dead_option_get_or_insert_with_map(raw: &str) -> String {
    DeadOptionGetOrInsertWithMapItem::new(raw).render()
}
