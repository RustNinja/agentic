pub struct DeadStringInsertStrMapItem {
    value: String,
}

impl DeadStringInsertStrMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-string-insert-str-map:{}", self.value)
    }
}

pub fn dead_string_insert_str_map(raw: &str) -> String {
    DeadStringInsertStrMapItem::new(raw).dead_method()
}
