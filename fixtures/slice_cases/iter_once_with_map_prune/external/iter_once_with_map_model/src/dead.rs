pub struct DeadIterOnceWithMapItem;

pub struct DeadIterOnceWithMapPayload {
    value: String,
}

impl DeadIterOnceWithMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iter-once-with-map:{}", self.value)
    }
}

pub fn dead_iter_once_with_map(raw: &str) -> String {
    DeadIterOnceWithMapPayload::new(raw).dead_method()
}
