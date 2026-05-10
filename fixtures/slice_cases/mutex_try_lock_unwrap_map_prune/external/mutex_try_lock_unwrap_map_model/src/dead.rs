pub struct DeadMutexTryLockUnwrapMapItem;

pub struct DeadMutexTryLockUnwrapMapPayload {
    value: String,
}

impl DeadMutexTryLockUnwrapMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-mutex-try-lock-unwrap-map:{}", self.value)
    }
}

pub fn dead_mutex_try_lock_unwrap_map(raw: &str) -> String {
    DeadMutexTryLockUnwrapMapPayload::new(raw).dead_method()
}
