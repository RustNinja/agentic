pub struct DeadCollectBinaryHeapItem {
    value: String,
}

impl DeadCollectBinaryHeapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-collect-binaryheap:{}", self.value)
    }
}

pub fn dead_collect_binaryheap(raw: &str) -> String {
    DeadCollectBinaryHeapItem::new(raw).dead_method()
}
