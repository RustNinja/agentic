pub struct DeadVecdequeMakeContiguousLastMapItem;

pub struct DeadVecdequeMakeContiguousLastMapPayload {
    value: String,
}

impl DeadVecdequeMakeContiguousLastMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vecdeque-make-contiguous-last-map:{}", self.value)
    }
}

pub fn dead_vecdeque_make_contiguous_last_map(raw: &str) -> String {
    DeadVecdequeMakeContiguousLastMapPayload::new(raw).dead_method()
}
