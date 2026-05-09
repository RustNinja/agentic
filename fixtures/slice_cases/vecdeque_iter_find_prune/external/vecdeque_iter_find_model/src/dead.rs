pub struct DeadVecdequeIterFindItem {
    value: String,
}

impl DeadVecdequeIterFindItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vecdeque-iter-find:{}", self.value)
    }
}

pub fn dead_vecdeque_iter_find(raw: &str) -> String {
    DeadVecdequeIterFindItem::new(raw).dead_method()
}
