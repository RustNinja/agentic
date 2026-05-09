pub struct DeadCollectAnnotatedLinkedListItem {
    value: String,
}

impl DeadCollectAnnotatedLinkedListItem {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.to_string() }
    }

    pub fn render(&self) -> String {
        format!("dead-model-collect-annotated-linkedlist:{}", self.value)
    }
}

pub fn dead_collect_annotated_linkedlist_model(raw: &str) -> String {
    DeadCollectAnnotatedLinkedListItem::new(raw).render()
}
