pub struct DeadBtreemapAppendKeysMapItem {
    value: String,
}

impl DeadBtreemapAppendKeysMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-append-keys-map:{}", self.value)
    }
}

pub fn dead_btreemap_append_keys_map(raw: &str) -> String {
    DeadBtreemapAppendKeysMapItem::new(raw).dead_method()
}
