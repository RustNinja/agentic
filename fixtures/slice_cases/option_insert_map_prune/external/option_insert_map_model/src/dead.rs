pub struct DeadOptionInsertMapItem {
    value: String,
}

impl DeadOptionInsertMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-option-insert-map:{}", self.value)
    }
}

pub fn dead_option_insert_map(raw: &str) -> String {
    DeadOptionInsertMapItem::new(raw).render()
}
