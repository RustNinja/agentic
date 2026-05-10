pub struct DeadResultIntoIterNextMapItem {
    value: String,
}

impl DeadResultIntoIterNextMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-into-iter-next-map:{}", self.value)
    }
}

pub fn dead_result_into_iter_next_map(raw: &str) -> String {
    DeadResultIntoIterNextMapItem::new(raw).dead_method()
}
