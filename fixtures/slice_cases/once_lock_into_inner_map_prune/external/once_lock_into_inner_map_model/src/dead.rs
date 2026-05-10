pub struct DeadOnceLockIntoInnerMapItem;

pub struct DeadOnceLockIntoInnerMapPayload {
    value: String,
}

impl DeadOnceLockIntoInnerMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-once-lock-into-inner-map:{}", self.value)
    }
}

pub fn dead_once_lock_into_inner_map(raw: &str) -> String {
    DeadOnceLockIntoInnerMapPayload::new(raw).dead_method()
}
