pub struct DeadBTreeSetRetainItem {
    value: String,
}

impl DeadBTreeSetRetainItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreeset-retain:{}", self.value)
    }
}

pub fn dead_btreeset_retain(raw: &str) -> String {
    DeadBTreeSetRetainItem::new(raw).dead_method()
}
