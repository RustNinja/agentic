pub struct DeadBtreemapIterMutPairsItem {
    value: String,
}

impl DeadBtreemapIterMutPairsItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-iter-mut-pairs:{}", self.value)
    }
}

pub fn dead_btreemap_iter_mut_pairs(raw: &str) -> String {
    DeadBtreemapIterMutPairsItem::new(raw).dead_method()
}
