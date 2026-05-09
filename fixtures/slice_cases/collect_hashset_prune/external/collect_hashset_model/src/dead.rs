pub struct DeadCollectHashSetItem {
    value: String,
}

impl DeadCollectHashSetItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-collect-hashset:{}", self.value)
    }
}

pub fn dead_collect_hashset(raw: &str) -> String {
    DeadCollectHashSetItem::new(raw).dead_method()
}
