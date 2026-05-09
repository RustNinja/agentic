pub struct DeadResultMapItem {
    value: String,
}

impl DeadResultMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-map:{}", self.value)
    }
}

pub fn dead_result_map(raw: &str) -> String {
    DeadResultMapItem::new(raw).dead_method()
}
