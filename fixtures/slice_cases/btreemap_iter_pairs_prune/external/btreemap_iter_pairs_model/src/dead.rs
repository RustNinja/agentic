pub struct DeadBtreemapIterPairsItem {
    value: String,
}

impl DeadBtreemapIterPairsItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-iter-pairs:{}", self.value)
    }
}

pub fn dead_btreemap_iter_pairs(raw: &str) -> String {
    DeadBtreemapIterPairsItem::new(raw).dead_method()
}
