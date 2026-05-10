pub struct DeadRwlockIntoInnerMapItem;

pub struct DeadRwlockIntoInnerMapPayload {
    value: String,
}

impl DeadRwlockIntoInnerMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-rwlock-into-inner-map:{}", self.value)
    }
}

pub fn dead_rwlock_into_inner_map(raw: &str) -> String {
    DeadRwlockIntoInnerMapPayload::new(raw).dead_method()
}
