pub struct DeadBtreesetIterNextBackMapItem {
    value: String,
}

impl DeadBtreesetIterNextBackMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreeset-iter-next-back-map:{}", self.value)
    }
}

pub fn dead_btreeset_iter_next_back_map(raw: &str) -> String {
    DeadBtreesetIterNextBackMapItem::new(raw).dead_method()
}
