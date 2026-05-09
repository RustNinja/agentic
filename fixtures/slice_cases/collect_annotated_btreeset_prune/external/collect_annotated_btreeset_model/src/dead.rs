pub struct DeadCollectAnnotatedBTreeSetItem {
    value: String,
}

impl DeadCollectAnnotatedBTreeSetItem {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.to_string() }
    }

    pub fn render(&self) -> String {
        format!("dead-model-collect-annotated-btreeset:{}", self.value)
    }
}

pub fn dead_collect_annotated_btreeset_model(raw: &str) -> String {
    DeadCollectAnnotatedBTreeSetItem::new(raw).render()
}
