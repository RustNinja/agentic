pub struct DeadStringInsertMapItem {
    value: String,
}

impl DeadStringInsertMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-string-insert-map:{}", self.value)
    }
}

pub fn dead_string_insert_map(raw: &str) -> String {
    DeadStringInsertMapItem::new(raw).dead_method()
}
