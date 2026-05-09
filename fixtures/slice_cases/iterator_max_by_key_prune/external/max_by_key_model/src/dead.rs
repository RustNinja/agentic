pub struct DeadMaxByKeyItem {
    value: String,
}

impl DeadMaxByKeyItem {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.to_string() }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-max-by-key:{}", self.value)
    }
}

pub fn dead_max_by_key(raw: &str) -> String {
    DeadMaxByKeyItem::new(raw).dead_method()
}
