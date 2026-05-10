pub struct DeadForBtreemapIterPairPayload {
    value: String,
}

impl DeadForBtreemapIterPairPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-for-btreemap-iter-pair:{}", self.value)
    }
}

pub fn dead_for_btreemap_iter_pair(raw: &str) -> String {
    DeadForBtreemapIterPairPayload::new(raw).dead_method()
}
