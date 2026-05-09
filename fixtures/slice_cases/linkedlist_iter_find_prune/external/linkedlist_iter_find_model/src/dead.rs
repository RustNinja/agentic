pub struct DeadLinkedlistIterFindItem {
    value: String,
}

impl DeadLinkedlistIterFindItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-linkedlist-iter-find:{}", self.value)
    }
}

pub fn dead_linkedlist_iter_find(raw: &str) -> String {
    DeadLinkedlistIterFindItem::new(raw).dead_method()
}
