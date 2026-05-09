pub struct DeadBtreesetFirstMapItem {
    value: String,
}

impl DeadBtreesetFirstMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreeset-first-map:{}", self.value)
    }
}

pub fn dead_btreeset_first_map(raw: &str) -> String {
    DeadBtreesetFirstMapItem::new(raw).dead_method()
}
