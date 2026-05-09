pub struct DeadResultMapOrItem {
    value: String,
}

impl DeadResultMapOrItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-map-or:{}", self.value)
    }
}

pub fn dead_result_map_or(raw: &str) -> String {
    DeadResultMapOrItem::new(raw).dead_method()
}
