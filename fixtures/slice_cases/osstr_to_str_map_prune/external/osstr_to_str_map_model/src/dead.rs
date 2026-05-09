pub struct DeadOsstrToStrMapItem {
    value: String,
}

impl DeadOsstrToStrMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-osstr-to-str-map:{}", self.value)
    }
}

pub fn dead_osstr_to_str_map(raw: &str) -> String {
    DeadOsstrToStrMapItem::new(raw).dead_method()
}
