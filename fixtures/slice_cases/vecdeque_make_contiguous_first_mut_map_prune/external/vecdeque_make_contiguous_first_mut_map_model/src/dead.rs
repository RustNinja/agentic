pub struct DeadVecdequeMakeContiguousFirstMutMapItem;

pub struct DeadVecdequeMakeContiguousFirstMutMapPayload {
    value: String,
}

impl DeadVecdequeMakeContiguousFirstMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vecdeque-make-contiguous-first-mut-map:{}", self.value)
    }
}

pub fn dead_vecdeque_make_contiguous_first_mut_map(raw: &str) -> String {
    DeadVecdequeMakeContiguousFirstMutMapPayload::new(raw).dead_method()
}
