pub struct DeadHashsetGetItem {
    value: String,
}

impl DeadHashsetGetItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashset-get:{}", self.value)
    }
}

pub fn dead_hashset_get(raw: &str) -> String {
    DeadHashsetGetItem::new(raw).dead_method()
}
