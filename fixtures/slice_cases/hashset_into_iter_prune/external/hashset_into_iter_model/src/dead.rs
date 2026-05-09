pub struct DeadHashsetIntoIterItem {
    value: String,
}

impl DeadHashsetIntoIterItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashset-into-iter:{}", self.value)
    }
}

pub fn dead_hashset_into_iter(raw: &str) -> String {
    DeadHashsetIntoIterItem::new(raw).dead_method()
}
