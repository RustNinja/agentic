pub struct DeadLinkedlistPushBackFrontMapItem {
    value: String,
}

impl DeadLinkedlistPushBackFrontMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-linkedlist-push-back-front-map:{}", self.value)
    }
}

pub fn dead_linkedlist_push_back_front_map(raw: &str) -> String {
    DeadLinkedlistPushBackFrontMapItem::new(raw).dead_method()
}
