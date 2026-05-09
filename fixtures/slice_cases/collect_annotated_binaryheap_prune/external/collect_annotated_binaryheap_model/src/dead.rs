pub struct DeadCollectAnnotatedBinaryHeapItem {
    value: String,
}

impl DeadCollectAnnotatedBinaryHeapItem {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.to_string() }
    }

    pub fn render(&self) -> String {
        format!("dead-model-collect-annotated-binaryheap:{}", self.value)
    }
}

pub fn dead_collect_annotated_binaryheap_model(raw: &str) -> String {
    DeadCollectAnnotatedBinaryHeapItem::new(raw).render()
}
