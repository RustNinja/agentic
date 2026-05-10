pub struct DeadOsstrToStringLossyMapItem {
    value: String,
}

impl DeadOsstrToStringLossyMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-osstr-to-string-lossy-map:{}", self.value)
    }
}

pub fn dead_osstr_to_string_lossy_map(raw: &str) -> String {
    DeadOsstrToStringLossyMapItem::new(raw).dead_method()
}
