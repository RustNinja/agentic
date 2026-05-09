pub struct DeadBtreemapKeysMapItem {
    value: String,
}

impl DeadBtreemapKeysMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-keys-map:{}", self.value)
    }
}

pub fn dead_btreemap_keys_map(raw: &str) -> String {
    DeadBtreemapKeysMapItem::new(raw).dead_method()
}
