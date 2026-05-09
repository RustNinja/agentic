pub struct DeadVecdequeFrontItem {
    value: String,
}

impl DeadVecdequeFrontItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vecdeque-front:{}", self.value)
    }
}

pub fn dead_vecdeque_front(raw: &str) -> String {
    DeadVecdequeFrontItem::new(raw).dead_method()
}
