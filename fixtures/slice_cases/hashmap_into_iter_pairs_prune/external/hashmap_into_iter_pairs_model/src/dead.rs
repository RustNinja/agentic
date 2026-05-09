pub struct DeadHashmapIntoIterPairsItem {
    value: String,
}

impl DeadHashmapIntoIterPairsItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-into-iter-pairs:{}", self.value)
    }
}

pub fn dead_hashmap_into_iter_pairs(raw: &str) -> String {
    DeadHashmapIntoIterPairsItem::new(raw).dead_method()
}
