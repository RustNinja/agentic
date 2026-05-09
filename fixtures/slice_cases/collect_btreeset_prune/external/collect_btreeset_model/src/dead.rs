pub struct DeadCollectBTreeSetItem {
    value: String,
}

impl DeadCollectBTreeSetItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-collect-btreeset:{}", self.value)
    }
}

pub fn dead_collect_btreeset(raw: &str) -> String {
    DeadCollectBTreeSetItem::new(raw).dead_method()
}
