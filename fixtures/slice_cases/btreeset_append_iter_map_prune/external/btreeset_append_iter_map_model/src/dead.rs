pub struct DeadBtreesetAppendIterMapItem {
    value: String,
}

impl DeadBtreesetAppendIterMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreeset-append-iter-map:{}", self.value)
    }
}

pub fn dead_btreeset_append_iter_map(raw: &str) -> String {
    DeadBtreesetAppendIterMapItem::new(raw).dead_method()
}
