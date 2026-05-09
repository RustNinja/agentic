pub struct DeadMinByItem {
    value: String,
}

impl DeadMinByItem {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.to_string() }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-min-by:{}", self.value)
    }
}

pub fn dead_min_by(raw: &str) -> String {
    DeadMinByItem::new(raw).dead_method()
}
