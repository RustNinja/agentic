pub struct DeadBtreesetPopFirstItem {
    value: String,
}

impl DeadBtreesetPopFirstItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreeset-pop-first:{}", self.value)
    }
}

pub fn dead_btreeset_pop_first(raw: &str) -> String {
    DeadBtreesetPopFirstItem::new(raw).dead_method()
}
