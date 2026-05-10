pub struct DeadForHashmapDrainPairPayload {
    value: String,
}

impl DeadForHashmapDrainPairPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-for-hashmap-drain-pair:{}", self.value)
    }
}

pub fn dead_for_hashmap_drain_pair(raw: &str) -> String {
    DeadForHashmapDrainPairPayload::new(raw).dead_method()
}
