pub struct DeadArrayIterItem {
    value: String,
}

impl DeadArrayIterItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-array-iter:{}", self.value)
    }
}

pub fn dead_array_iter(raw: &str) -> String {
    DeadArrayIterItem::new(raw).dead_method()
}
