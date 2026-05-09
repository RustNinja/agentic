pub struct DeadResultOkMapItem {
    value: String,
}

impl DeadResultOkMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-ok-map:{}", self.value)
    }
}

pub fn dead_result_ok_map(raw: &str) -> String {
    DeadResultOkMapItem::new(raw).dead_method()
}
