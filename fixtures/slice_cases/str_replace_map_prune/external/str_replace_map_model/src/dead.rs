pub struct DeadStrReplaceMapItem {
    value: String,
}

impl DeadStrReplaceMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-str-replace-map:{}", self.value)
    }
}

pub fn dead_str_replace_map(raw: &str) -> String {
    DeadStrReplaceMapItem::new(raw).dead_method()
}
