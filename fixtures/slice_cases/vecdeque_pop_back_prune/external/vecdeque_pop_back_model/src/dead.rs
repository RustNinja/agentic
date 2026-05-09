pub struct DeadVecdequePopBackItem {
    value: String,
}

impl DeadVecdequePopBackItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vecdeque-pop-back:{}", self.value)
    }
}

pub fn dead_vecdeque_pop_back(raw: &str) -> String {
    DeadVecdequePopBackItem::new(raw).dead_method()
}
