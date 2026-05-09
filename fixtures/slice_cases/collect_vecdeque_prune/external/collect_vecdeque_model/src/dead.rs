pub struct DeadCollectVecDequeItem {
    value: String,
}

impl DeadCollectVecDequeItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-collect-vecdeque:{}", self.value)
    }
}

pub fn dead_collect_vecdeque(raw: &str) -> String {
    DeadCollectVecDequeItem::new(raw).dead_method()
}
