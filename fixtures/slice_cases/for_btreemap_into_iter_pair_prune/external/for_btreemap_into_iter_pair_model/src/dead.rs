pub struct DeadForBtreemapIntoIterPairPayload {
    value: String,
}

impl DeadForBtreemapIntoIterPairPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-for-btreemap-into-iter-pair:{}", self.value)
    }
}

pub fn dead_for_btreemap_into_iter_pair(raw: &str) -> String {
    DeadForBtreemapIntoIterPairPayload::new(raw).dead_method()
}
