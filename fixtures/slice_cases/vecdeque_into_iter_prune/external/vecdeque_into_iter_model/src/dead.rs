pub struct DeadVecdequeIntoIterItem {
    value: String,
}

impl DeadVecdequeIntoIterItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vecdeque-into-iter:{}", self.value)
    }
}

pub fn dead_vecdeque_into_iter(raw: &str) -> String {
    DeadVecdequeIntoIterItem::new(raw).dead_method()
}
