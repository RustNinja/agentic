pub struct DeadBtreemapIntoKeysMapItem {
    value: String,
}

impl DeadBtreemapIntoKeysMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-into-keys-map:{}", self.value)
    }
}

pub fn dead_btreemap_into_keys_map(raw: &str) -> String {
    DeadBtreemapIntoKeysMapItem::new(raw).dead_method()
}
