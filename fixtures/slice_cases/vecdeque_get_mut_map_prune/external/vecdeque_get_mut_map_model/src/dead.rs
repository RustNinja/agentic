pub struct DeadVecdequeGetMutMapItem {
    value: String,
}

impl DeadVecdequeGetMutMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vecdeque-get-mut-map:{}", self.value)
    }
}

pub fn dead_vecdeque_get_mut_map(raw: &str) -> String {
    DeadVecdequeGetMutMapItem::new(raw).dead_method()
}
