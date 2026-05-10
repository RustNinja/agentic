pub struct DeadVecdequeMakeContiguousFirstMapItem;

pub struct DeadVecdequeMakeContiguousFirstMapPayload {
    value: String,
}

impl DeadVecdequeMakeContiguousFirstMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vecdeque-make-contiguous-first-map:{}", self.value)
    }
}

pub fn dead_vecdeque_make_contiguous_first_map(raw: &str) -> String {
    DeadVecdequeMakeContiguousFirstMapPayload::new(raw).dead_method()
}
