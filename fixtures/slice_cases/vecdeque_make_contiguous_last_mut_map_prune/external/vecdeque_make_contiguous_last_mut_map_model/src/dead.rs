pub struct DeadVecdequeMakeContiguousLastMutMapItem;

pub struct DeadVecdequeMakeContiguousLastMutMapPayload {
    value: String,
}

impl DeadVecdequeMakeContiguousLastMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vecdeque-make-contiguous-last-mut-map:{}", self.value)
    }
}

pub fn dead_vecdeque_make_contiguous_last_mut_map(raw: &str) -> String {
    DeadVecdequeMakeContiguousLastMutMapPayload::new(raw).dead_method()
}
