pub struct DeadLinkedlistBackMutMapItem {
    value: String,
}

impl DeadLinkedlistBackMutMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-linkedlist-back-mut-map:{}", self.value)
    }
}

pub fn dead_linkedlist_back_mut_map(raw: &str) -> String {
    DeadLinkedlistBackMutMapItem::new(raw).dead_method()
}
