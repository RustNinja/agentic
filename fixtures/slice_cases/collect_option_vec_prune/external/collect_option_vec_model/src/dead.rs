pub struct DeadCollectOptionVecItem {
    value: String,
}

impl DeadCollectOptionVecItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-collect-option-vec:{}", self.value)
    }
}

pub fn dead_collect_option_vec(raw: &str) -> String {
    DeadCollectOptionVecItem::new(raw).dead_method()
}
