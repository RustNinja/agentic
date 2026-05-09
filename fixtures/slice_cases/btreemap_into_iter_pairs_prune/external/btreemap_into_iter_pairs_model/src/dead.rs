pub struct DeadBtreemapIntoIterPairsItem {
    value: String,
}

impl DeadBtreemapIntoIterPairsItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-into-iter-pairs:{}", self.value)
    }
}

pub fn dead_btreemap_into_iter_pairs(raw: &str) -> String {
    DeadBtreemapIntoIterPairsItem::new(raw).dead_method()
}
