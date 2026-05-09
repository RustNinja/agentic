pub struct DeadBtreesetGetItem {
    value: String,
}

impl DeadBtreesetGetItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreeset-get:{}", self.value)
    }
}

pub fn dead_btreeset_get(raw: &str) -> String {
    DeadBtreesetGetItem::new(raw).dead_method()
}
