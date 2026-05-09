pub struct DeadLinkedlistBackItem {
    value: String,
}

impl DeadLinkedlistBackItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-linkedlist-back:{}", self.value)
    }
}

pub fn dead_linkedlist_back(raw: &str) -> String {
    DeadLinkedlistBackItem::new(raw).dead_method()
}
