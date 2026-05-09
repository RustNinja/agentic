pub struct DeadVecDequeRetainItem {
    value: String,
}

impl DeadVecDequeRetainItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vecdeque-retain:{}", self.value)
    }
}

pub fn dead_vecdeque_retain(raw: &str) -> String {
    DeadVecDequeRetainItem::new(raw).dead_method()
}
