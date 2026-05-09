pub struct DeadVecdequeGetMapItem {
    value: String,
}

impl DeadVecdequeGetMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vecdeque-get-map:{}", self.value)
    }
}

pub fn dead_vecdeque_get_map(raw: &str) -> String {
    DeadVecdequeGetMapItem::new(raw).dead_method()
}
