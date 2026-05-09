pub struct DeadMinByKeyItem {
    value: String,
}

impl DeadMinByKeyItem {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.to_string() }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-min-by-key:{}", self.value)
    }
}

pub fn dead_min_by_key(raw: &str) -> String {
    DeadMinByKeyItem::new(raw).dead_method()
}
