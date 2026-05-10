pub struct DeadRwlockTryWriteMapItem;

pub struct DeadRwlockTryWriteMapPayload {
    value: String,
}

impl DeadRwlockTryWriteMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-rwlock-try-write-map:{}", self.value)
    }
}

pub fn dead_rwlock_try_write_map(raw: &str) -> String {
    DeadRwlockTryWriteMapPayload::new(raw).dead_method()
}
