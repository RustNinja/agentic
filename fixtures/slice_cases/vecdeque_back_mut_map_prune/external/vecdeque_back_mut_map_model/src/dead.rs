pub struct DeadVecdequeBackMutMapItem {
    value: String,
}

impl DeadVecdequeBackMutMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vecdeque-back-mut-map:{}", self.value)
    }
}

pub fn dead_vecdeque_back_mut_map(raw: &str) -> String {
    DeadVecdequeBackMutMapItem::new(raw).dead_method()
}
