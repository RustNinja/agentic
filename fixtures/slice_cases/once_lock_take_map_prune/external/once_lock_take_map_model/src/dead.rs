pub struct DeadOnceLockTakeMapItem;

pub struct DeadOnceLockTakeMapPayload {
    value: String,
}

impl DeadOnceLockTakeMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-once-lock-take-map:{}", self.value)
    }
}

pub fn dead_once_lock_take_map(raw: &str) -> String {
    DeadOnceLockTakeMapPayload::new(raw).dead_method()
}
