pub struct DeadForBtreemapIterMutPairPayload {
    value: String,
}

impl DeadForBtreemapIterMutPairPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-for-btreemap-iter-mut-pair:{}", self.value)
    }
}

pub fn dead_for_btreemap_iter_mut_pair(raw: &str) -> String {
    DeadForBtreemapIterMutPairPayload::new(raw).dead_method()
}
