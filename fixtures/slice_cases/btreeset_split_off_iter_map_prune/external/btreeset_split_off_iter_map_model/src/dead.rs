pub struct DeadBtreesetSplitOffIterMapItem {
    value: String,
}

impl DeadBtreesetSplitOffIterMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreeset-split-off-iter-map:{}", self.value)
    }
}

pub fn dead_btreeset_split_off_iter_map(raw: &str) -> String {
    DeadBtreesetSplitOffIterMapItem::new(raw).dead_method()
}
