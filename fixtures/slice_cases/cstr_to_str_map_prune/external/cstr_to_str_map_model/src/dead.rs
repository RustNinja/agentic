pub struct DeadCstrToStrMapItem {
    value: String,
}

impl DeadCstrToStrMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-cstr-to-str-map:{}", self.value)
    }
}

pub fn dead_cstr_to_str_map(raw: &str) -> String {
    DeadCstrToStrMapItem::new(raw).dead_method()
}
