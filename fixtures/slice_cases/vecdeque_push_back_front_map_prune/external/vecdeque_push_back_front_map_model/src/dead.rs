pub struct DeadVecdequePushBackFrontMapItem {
    value: String,
}

impl DeadVecdequePushBackFrontMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vecdeque-push-back-front-map:{}", self.value)
    }
}

pub fn dead_vecdeque_push_back_front_map(raw: &str) -> String {
    DeadVecdequePushBackFrontMapItem::new(raw).dead_method()
}
