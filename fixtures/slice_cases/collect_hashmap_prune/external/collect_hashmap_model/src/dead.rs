pub struct DeadCollectHashMapItem {
    value: String,
}

impl DeadCollectHashMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-collect-hashmap:{}", self.value)
    }
}

pub fn dead_collect_hashmap(raw: &str) -> String {
    DeadCollectHashMapItem::new(raw).dead_method()
}
