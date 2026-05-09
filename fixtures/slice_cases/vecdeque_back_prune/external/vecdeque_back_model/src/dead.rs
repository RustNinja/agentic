pub struct DeadVecdequeBackItem {
    value: String,
}

impl DeadVecdequeBackItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vecdeque-back:{}", self.value)
    }
}

pub fn dead_vecdeque_back(raw: &str) -> String {
    DeadVecdequeBackItem::new(raw).dead_method()
}
