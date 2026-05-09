pub struct DeadBtreesetTakeItem {
    value: String,
}

impl DeadBtreesetTakeItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreeset-take:{}", self.value)
    }
}

pub fn dead_btreeset_take(raw: &str) -> String {
    DeadBtreesetTakeItem::new(raw).dead_method()
}
