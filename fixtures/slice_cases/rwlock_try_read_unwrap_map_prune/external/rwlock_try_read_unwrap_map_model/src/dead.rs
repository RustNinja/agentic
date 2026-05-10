pub struct DeadRwlockTryReadUnwrapMapItem;

pub struct DeadRwlockTryReadUnwrapMapPayload {
    value: String,
}

impl DeadRwlockTryReadUnwrapMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-rwlock-try-read-unwrap-map:{}", self.value)
    }
}

pub fn dead_rwlock_try_read_unwrap_map(raw: &str) -> String {
    DeadRwlockTryReadUnwrapMapPayload::new(raw).dead_method()
}
