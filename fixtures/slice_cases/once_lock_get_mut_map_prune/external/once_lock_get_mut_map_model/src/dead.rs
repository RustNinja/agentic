pub struct DeadOnceLockGetMutMapItem;

pub struct DeadOnceLockGetMutMapPayload {
    value: String,
}

impl DeadOnceLockGetMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-once-lock-get-mut-map:{}", self.value)
    }
}

pub fn dead_once_lock_get_mut_map(raw: &str) -> String {
    DeadOnceLockGetMutMapPayload::new(raw).dead_method()
}
