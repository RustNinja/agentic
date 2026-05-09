pub struct DeadHashmapDrainPairsItem {
    value: String,
}

impl DeadHashmapDrainPairsItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-drain-pairs:{}", self.value)
    }
}

pub fn dead_hashmap_drain_pairs(raw: &str) -> String {
    DeadHashmapDrainPairsItem::new(raw).dead_method()
}
