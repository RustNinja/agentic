pub struct DeadResultMapOrValueItem {
    value: String,
}

impl DeadResultMapOrValueItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-map-or-value:{}", self.value)
    }
}

pub fn dead_result_map_or_value(raw: &str) -> String {
    DeadResultMapOrValueItem::new(raw).dead_method()
}
