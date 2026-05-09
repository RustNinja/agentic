pub struct DeadLinkedlistIterMutMapItem {
    value: String,
}

impl DeadLinkedlistIterMutMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-linkedlist-iter-mut-map:{}", self.value)
    }
}

pub fn dead_linkedlist_iter_mut_map(raw: &str) -> String {
    DeadLinkedlistIterMutMapItem::new(raw).dead_method()
}
