pub struct DeadHashsetReplaceItem {
    value: String,
}

impl DeadHashsetReplaceItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashset-replace:{}", self.value)
    }
}

pub fn dead_hashset_replace(raw: &str) -> String {
    DeadHashsetReplaceItem::new(raw).dead_method()
}
