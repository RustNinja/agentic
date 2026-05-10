pub struct DeadForHashmapIterPairPayload {
    value: String,
}

impl DeadForHashmapIterPairPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-for-hashmap-iter-pair:{}", self.value)
    }
}

pub fn dead_for_hashmap_iter_pair(raw: &str) -> String {
    DeadForHashmapIterPairPayload::new(raw).dead_method()
}
