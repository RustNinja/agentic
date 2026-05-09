pub struct DeadCollectAnnotatedVecDequeItem {
    value: String,
}

impl DeadCollectAnnotatedVecDequeItem {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.to_string() }
    }

    pub fn render(&self) -> String {
        format!("dead-model-collect-annotated-vecdeque:{}", self.value)
    }
}

pub fn dead_collect_annotated_vecdeque_model(raw: &str) -> String {
    DeadCollectAnnotatedVecDequeItem::new(raw).render()
}
