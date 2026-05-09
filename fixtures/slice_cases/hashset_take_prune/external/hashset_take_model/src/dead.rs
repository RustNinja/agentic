pub struct DeadHashsetTakeItem {
    value: String,
}

impl DeadHashsetTakeItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashset-take:{}", self.value)
    }
}

pub fn dead_hashset_take(raw: &str) -> String {
    DeadHashsetTakeItem::new(raw).dead_method()
}
