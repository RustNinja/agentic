pub struct DeadBtreemapGetMutItem {
    value: String,
}

impl DeadBtreemapGetMutItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-get-mut:{}", self.value)
    }
}

pub fn dead_btreemap_get_mut(raw: &str) -> String {
    DeadBtreemapGetMutItem::new(raw).dead_method()
}
