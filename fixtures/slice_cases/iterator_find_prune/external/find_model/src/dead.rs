pub struct DeadFindItem {
    value: String,
}

impl DeadFindItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-find:{}", self.value)
    }
}

pub fn dead_find(raw: &str) -> String {
    DeadFindItem::new(raw).dead_method()
}
