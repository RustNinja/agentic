pub struct DeadLastItem {
    value: String,
}

impl DeadLastItem {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.to_string() }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-last:{}", self.value)
    }
}

pub fn dead_last(raw: &str) -> String {
    DeadLastItem::new(raw).dead_method()
}
