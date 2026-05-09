pub struct DeadLinkedlistIntoIterItem {
    value: String,
}

impl DeadLinkedlistIntoIterItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-linkedlist-into-iter:{}", self.value)
    }
}

pub fn dead_linkedlist_into_iter(raw: &str) -> String {
    DeadLinkedlistIntoIterItem::new(raw).dead_method()
}
