pub struct DeadMutexIntoInnerUnwrapMapItem;

pub struct DeadMutexIntoInnerUnwrapMapPayload {
    value: String,
}

impl DeadMutexIntoInnerUnwrapMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-mutex-into-inner-unwrap-map:{}", self.value)
    }
}

pub fn dead_mutex_into_inner_unwrap_map(raw: &str) -> String {
    DeadMutexIntoInnerUnwrapMapPayload::new(raw).dead_method()
}
