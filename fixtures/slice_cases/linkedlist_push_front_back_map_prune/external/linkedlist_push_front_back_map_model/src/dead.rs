pub struct DeadLinkedlistPushFrontBackMapItem {
    value: String,
}

impl DeadLinkedlistPushFrontBackMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-linkedlist-push-front-back-map:{}", self.value)
    }
}

pub fn dead_linkedlist_push_front_back_map(raw: &str) -> String {
    DeadLinkedlistPushFrontBackMapItem::new(raw).dead_method()
}
