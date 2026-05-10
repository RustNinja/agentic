pub struct DeadOnceLockIntoInnerUnwrapMapItem;

pub struct DeadOnceLockIntoInnerUnwrapMapPayload {
    value: String,
}

impl DeadOnceLockIntoInnerUnwrapMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-once-lock-into-inner-unwrap-map:{}", self.value)
    }
}

pub fn dead_once_lock_into_inner_unwrap_map(raw: &str) -> String {
    DeadOnceLockIntoInnerUnwrapMapPayload::new(raw).dead_method()
}
