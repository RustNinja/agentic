pub struct DeadLinkedlistFrontItem {
    value: String,
}

impl DeadLinkedlistFrontItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-linkedlist-front:{}", self.value)
    }
}

pub fn dead_linkedlist_front(raw: &str) -> String {
    DeadLinkedlistFrontItem::new(raw).dead_method()
}
