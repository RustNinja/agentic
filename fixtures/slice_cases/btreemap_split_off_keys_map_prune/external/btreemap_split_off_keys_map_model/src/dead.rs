pub struct DeadBtreemapSplitOffKeysMapItem {
    value: String,
}

impl DeadBtreemapSplitOffKeysMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-split-off-keys-map:{}", self.value)
    }
}

pub fn dead_btreemap_split_off_keys_map(raw: &str) -> String {
    DeadBtreemapSplitOffKeysMapItem::new(raw).dead_method()
}
