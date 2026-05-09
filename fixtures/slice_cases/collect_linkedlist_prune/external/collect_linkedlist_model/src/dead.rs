pub struct DeadCollectLinkedListItem {
    value: String,
}

impl DeadCollectLinkedListItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-collect-linkedlist:{}", self.value)
    }
}

pub fn dead_collect_linkedlist(raw: &str) -> String {
    DeadCollectLinkedListItem::new(raw).dead_method()
}
