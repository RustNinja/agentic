pub struct DeadIterOnceMapItem;

pub struct DeadIterOnceMapPayload {
    value: String,
}

impl DeadIterOnceMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iter-once-map:{}", self.value)
    }
}

pub fn dead_iter_once_map(raw: &str) -> String {
    DeadIterOnceMapPayload::new(raw).dead_method()
}
