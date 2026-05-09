pub struct DeadCollectAnnotatedHashSetItem {
    value: String,
}

impl DeadCollectAnnotatedHashSetItem {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.to_string() }
    }

    pub fn render(&self) -> String {
        format!("dead-model-collect-annotated-hashset:{}", self.value)
    }
}

pub fn dead_collect_annotated_hashset_model(raw: &str) -> String {
    DeadCollectAnnotatedHashSetItem::new(raw).render()
}
