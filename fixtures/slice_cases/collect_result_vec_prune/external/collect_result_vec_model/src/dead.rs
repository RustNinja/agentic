pub struct DeadCollectResultVecItem {
    value: String,
}

impl DeadCollectResultVecItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-collect-result-vec:{}", self.value)
    }
}

pub fn dead_collect_result_vec(raw: &str) -> String {
    DeadCollectResultVecItem::new(raw).dead_method()
}
