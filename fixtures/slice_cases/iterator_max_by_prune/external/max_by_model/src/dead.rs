pub struct DeadMaxByItem {
    value: String,
}

impl DeadMaxByItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-max-by:{}", self.value)
    }
}

pub fn dead_max_by(raw: &str) -> String {
    DeadMaxByItem::new(raw).dead_method()
}
