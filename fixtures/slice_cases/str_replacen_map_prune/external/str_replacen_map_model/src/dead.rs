pub struct DeadStrReplacenMapItem {
    value: String,
}

impl DeadStrReplacenMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-str-replacen-map:{}", self.value)
    }
}

pub fn dead_str_replacen_map(raw: &str) -> String {
    DeadStrReplacenMapItem::new(raw).dead_method()
}
