pub struct DeadHashmapIterPairsItem {
    value: String,
}

impl DeadHashmapIterPairsItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-iter-pairs:{}", self.value)
    }
}

pub fn dead_hashmap_iter_pairs(raw: &str) -> String {
    DeadHashmapIterPairsItem::new(raw).dead_method()
}
