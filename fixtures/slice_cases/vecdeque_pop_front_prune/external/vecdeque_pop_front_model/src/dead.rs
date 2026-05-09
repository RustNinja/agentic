pub struct DeadVecdequePopFrontItem {
    value: String,
}

impl DeadVecdequePopFrontItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vecdeque-pop-front:{}", self.value)
    }
}

pub fn dead_vecdeque_pop_front(raw: &str) -> String {
    DeadVecdequePopFrontItem::new(raw).dead_method()
}
