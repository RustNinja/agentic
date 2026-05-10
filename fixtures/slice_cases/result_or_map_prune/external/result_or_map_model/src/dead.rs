pub struct DeadResultOrMapItem {
    value: String,
}

impl DeadResultOrMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-or-map:{}", self.value)
    }
}

pub fn dead_result_or_map(raw: &str) -> String {
    DeadResultOrMapItem::new(raw).dead_method()
}
