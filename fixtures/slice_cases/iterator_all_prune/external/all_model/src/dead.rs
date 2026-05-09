pub struct DeadAllItem {
    value: String,
}

impl DeadAllItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-all:{}", self.value)
    }
}

pub fn dead_all(raw: &str) -> String {
    DeadAllItem::new(raw).dead_method()
}
