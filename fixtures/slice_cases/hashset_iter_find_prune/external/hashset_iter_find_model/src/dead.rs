pub struct DeadHashsetIterFindItem {
    value: String,
}

impl DeadHashsetIterFindItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashset-iter-find:{}", self.value)
    }
}

pub fn dead_hashset_iter_find(raw: &str) -> String {
    DeadHashsetIterFindItem::new(raw).dead_method()
}
