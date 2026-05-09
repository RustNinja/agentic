pub struct DeadVecdequeRemoveMapItem {
    value: String,
}

impl DeadVecdequeRemoveMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vecdeque-remove-map:{}", self.value)
    }
}

pub fn dead_vecdeque_remove_map(raw: &str) -> String {
    DeadVecdequeRemoveMapItem::new(raw).dead_method()
}
