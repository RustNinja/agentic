pub struct DeadCollectVecTupleItem {
    value: String,
}

impl DeadCollectVecTupleItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-collect-vec-tuple:{}", self.value)
    }
}

pub fn dead_collect_vec_tuple(raw: &str) -> String {
    DeadCollectVecTupleItem::new(raw).dead_method()
}
