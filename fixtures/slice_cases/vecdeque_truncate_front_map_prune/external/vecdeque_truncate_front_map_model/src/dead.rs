pub struct DeadVecdequeTruncateFrontMapItem {
    value: String,
}

impl DeadVecdequeTruncateFrontMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vecdeque-truncate-front-map:{}", self.value)
    }
}

pub fn dead_vecdeque_truncate_front_map(raw: &str) -> String {
    DeadVecdequeTruncateFrontMapItem::new(raw).dead_method()
}
