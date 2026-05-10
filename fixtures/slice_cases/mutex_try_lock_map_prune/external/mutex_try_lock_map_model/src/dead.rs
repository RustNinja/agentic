pub struct DeadMutexTryLockMapItem;

pub struct DeadMutexTryLockMapPayload {
    value: String,
}

impl DeadMutexTryLockMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-mutex-try-lock-map:{}", self.value)
    }
}

pub fn dead_mutex_try_lock_map(raw: &str) -> String {
    DeadMutexTryLockMapPayload::new(raw).dead_method()
}
