pub struct DeadRwlockTryReadMapItem;

pub struct DeadRwlockTryReadMapPayload {
    value: String,
}

impl DeadRwlockTryReadMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-rwlock-try-read-map:{}", self.value)
    }
}

pub fn dead_rwlock_try_read_map(raw: &str) -> String {
    DeadRwlockTryReadMapPayload::new(raw).dead_method()
}
