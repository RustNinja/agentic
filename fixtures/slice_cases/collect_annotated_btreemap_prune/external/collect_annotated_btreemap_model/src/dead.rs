pub struct DeadCollectAnnotatedBTreeMapItem {
    value: String,
}

impl DeadCollectAnnotatedBTreeMapItem {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.to_string() }
    }

    pub fn render(&self) -> String {
        format!("dead-model-collect-annotated-btreemap:{}", self.value)
    }
}

pub fn dead_collect_annotated_btreemap_model(raw: &str) -> String {
    DeadCollectAnnotatedBTreeMapItem::new(raw).render()
}
