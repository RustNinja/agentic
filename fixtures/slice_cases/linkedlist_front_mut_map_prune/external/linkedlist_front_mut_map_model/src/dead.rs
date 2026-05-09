pub struct DeadLinkedlistFrontMutMapItem {
    value: String,
}

impl DeadLinkedlistFrontMutMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-linkedlist-front-mut-map:{}", self.value)
    }
}

pub fn dead_linkedlist_front_mut_map(raw: &str) -> String {
    DeadLinkedlistFrontMutMapItem::new(raw).dead_method()
}
