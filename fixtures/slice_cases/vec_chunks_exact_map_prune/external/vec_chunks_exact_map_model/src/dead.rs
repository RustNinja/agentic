pub struct DeadVecChunksExactMapItem {
    value: String,
}

impl DeadVecChunksExactMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-chunks-exact-map:{}", self.value)
    }
}

pub fn dead_vec_chunks_exact_map(raw: &str) -> String {
    DeadVecChunksExactMapItem::new(raw).dead_method()
}
