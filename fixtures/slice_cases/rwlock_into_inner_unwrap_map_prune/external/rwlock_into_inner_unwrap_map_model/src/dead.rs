pub struct DeadRwlockIntoInnerUnwrapMapItem;

pub struct DeadRwlockIntoInnerUnwrapMapPayload {
    value: String,
}

impl DeadRwlockIntoInnerUnwrapMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-rwlock-into-inner-unwrap-map:{}", self.value)
    }
}

pub fn dead_rwlock_into_inner_unwrap_map(raw: &str) -> String {
    DeadRwlockIntoInnerUnwrapMapPayload::new(raw).dead_method()
}
