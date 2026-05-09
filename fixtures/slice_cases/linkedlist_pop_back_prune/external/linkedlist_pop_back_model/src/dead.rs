pub struct DeadLinkedlistPopBackItem {
    value: String,
}

impl DeadLinkedlistPopBackItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-linkedlist-pop-back:{}", self.value)
    }
}

pub fn dead_linkedlist_pop_back(raw: &str) -> String {
    DeadLinkedlistPopBackItem::new(raw).dead_method()
}
