pub struct DeadBtreesetLastMapItem {
    value: String,
}

impl DeadBtreesetLastMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreeset-last-map:{}", self.value)
    }
}

pub fn dead_btreeset_last_map(raw: &str) -> String {
    DeadBtreesetLastMapItem::new(raw).dead_method()
}
