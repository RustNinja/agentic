pub struct DeadLinkedlistAppendBackMapItem {
    value: String,
}

impl DeadLinkedlistAppendBackMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-linkedlist-append-back-map:{}", self.value)
    }
}

pub fn dead_linkedlist_append_back_map(raw: &str) -> String {
    DeadLinkedlistAppendBackMapItem::new(raw).dead_method()
}
