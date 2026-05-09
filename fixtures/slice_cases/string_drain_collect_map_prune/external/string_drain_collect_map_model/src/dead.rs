pub struct DeadStringDrainCollectMapItem {
    value: String,
}

impl DeadStringDrainCollectMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-string-drain-collect-map:{}", self.value)
    }
}

pub fn dead_string_drain_collect_map(raw: &str) -> String {
    DeadStringDrainCollectMapItem::new(raw).dead_method()
}
