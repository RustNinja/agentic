pub struct DeadForHashmapIterMutPairPayload {
    value: String,
}

impl DeadForHashmapIterMutPairPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-for-hashmap-iter-mut-pair:{}", self.value)
    }
}

pub fn dead_for_hashmap_iter_mut_pair(raw: &str) -> String {
    DeadForHashmapIterMutPairPayload::new(raw).dead_method()
}
