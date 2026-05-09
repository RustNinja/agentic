pub struct DeadBTreeMapRetainItem {
    value: String,
}

impl DeadBTreeMapRetainItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-retain:{}", self.value)
    }
}

pub fn dead_btreemap_retain(raw: &str) -> String {
    DeadBTreeMapRetainItem::new(raw).dead_method()
}
