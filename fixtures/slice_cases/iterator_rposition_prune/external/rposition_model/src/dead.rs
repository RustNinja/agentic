pub struct DeadRPositionItem {
    value: String,
}

impl DeadRPositionItem {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.to_string() }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-rposition:{}", self.value)
    }
}

pub fn dead_rposition(raw: &str) -> String {
    DeadRPositionItem::new(raw).dead_method()
}
