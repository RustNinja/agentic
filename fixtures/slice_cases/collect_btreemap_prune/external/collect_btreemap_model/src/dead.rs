pub struct DeadCollectBTreeMapItem {
    value: String,
}

impl DeadCollectBTreeMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-collect-btreemap:{}", self.value)
    }
}

pub fn dead_collect_btreemap(raw: &str) -> String {
    DeadCollectBTreeMapItem::new(raw).dead_method()
}
