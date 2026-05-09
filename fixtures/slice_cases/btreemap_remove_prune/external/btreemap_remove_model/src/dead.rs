pub struct DeadBtreemapRemoveItem {
    value: String,
}

impl DeadBtreemapRemoveItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-remove:{}", self.value)
    }
}

pub fn dead_btreemap_remove(raw: &str) -> String {
    DeadBtreemapRemoveItem::new(raw).dead_method()
}
