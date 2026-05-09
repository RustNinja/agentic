pub struct DeadCollectAnnotatedHashMapItem {
    value: String,
}

impl DeadCollectAnnotatedHashMapItem {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.to_string() }
    }

    pub fn render(&self) -> String {
        format!("dead-model-collect-annotated-hashmap:{}", self.value)
    }
}

pub fn dead_collect_annotated_hashmap_model(raw: &str) -> String {
    DeadCollectAnnotatedHashMapItem::new(raw).render()
}
