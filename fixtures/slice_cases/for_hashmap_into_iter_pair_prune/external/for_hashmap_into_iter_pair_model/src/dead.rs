pub struct DeadForHashmapIntoIterPairPayload {
    value: String,
}

impl DeadForHashmapIntoIterPairPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-for-hashmap-into-iter-pair:{}", self.value)
    }
}

pub fn dead_for_hashmap_into_iter_pair(raw: &str) -> String {
    DeadForHashmapIntoIterPairPayload::new(raw).dead_method()
}
