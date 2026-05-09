pub struct DeadLinkedlistPopFrontItem {
    value: String,
}

impl DeadLinkedlistPopFrontItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-linkedlist-pop-front:{}", self.value)
    }
}

pub fn dead_linkedlist_pop_front(raw: &str) -> String {
    DeadLinkedlistPopFrontItem::new(raw).dead_method()
}
