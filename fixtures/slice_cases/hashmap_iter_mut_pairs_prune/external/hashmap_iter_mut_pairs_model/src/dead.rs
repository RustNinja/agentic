pub struct DeadHashmapIterMutPairsItem {
    value: String,
}

impl DeadHashmapIterMutPairsItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-iter-mut-pairs:{}", self.value)
    }
}

pub fn dead_hashmap_iter_mut_pairs(raw: &str) -> String {
    DeadHashmapIterMutPairsItem::new(raw).dead_method()
}
