pub struct DeadStrRsplitMapItem {
    value: String,
}

impl DeadStrRsplitMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-str-rsplit-map:{}", self.value)
    }
}

pub fn dead_str_rsplit_map(raw: &str) -> String {
    DeadStrRsplitMapItem::new(raw).dead_method()
}
