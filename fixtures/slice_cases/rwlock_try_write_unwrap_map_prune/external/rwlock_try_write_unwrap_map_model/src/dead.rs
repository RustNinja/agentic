pub struct DeadRwlockTryWriteUnwrapMapItem;

pub struct DeadRwlockTryWriteUnwrapMapPayload {
    value: String,
}

impl DeadRwlockTryWriteUnwrapMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-rwlock-try-write-unwrap-map:{}", self.value)
    }
}

pub fn dead_rwlock_try_write_unwrap_map(raw: &str) -> String {
    DeadRwlockTryWriteUnwrapMapPayload::new(raw).dead_method()
}
