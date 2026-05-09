pub struct DeadBtreesetPopLastItem {
    value: String,
}

impl DeadBtreesetPopLastItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreeset-pop-last:{}", self.value)
    }
}

pub fn dead_btreeset_pop_last(raw: &str) -> String {
    DeadBtreesetPopLastItem::new(raw).dead_method()
}
