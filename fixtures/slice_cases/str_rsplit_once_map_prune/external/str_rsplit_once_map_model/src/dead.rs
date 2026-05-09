pub struct DeadStrRsplitOnceMapItem {
    value: String,
}

impl DeadStrRsplitOnceMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-str-rsplit-once-map:{}", self.value)
    }
}

pub fn dead_str_rsplit_once_map(raw: &str) -> String {
    DeadStrRsplitOnceMapItem::new(raw).dead_method()
}
