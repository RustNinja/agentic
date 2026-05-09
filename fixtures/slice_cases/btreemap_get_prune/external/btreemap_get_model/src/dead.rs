pub struct DeadBtreemapGetItem {
    value: String,
}

impl DeadBtreemapGetItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-get:{}", self.value)
    }
}

pub fn dead_btreemap_get(raw: &str) -> String {
    DeadBtreemapGetItem::new(raw).dead_method()
}
