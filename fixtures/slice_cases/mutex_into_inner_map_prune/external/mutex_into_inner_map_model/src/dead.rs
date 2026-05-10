pub struct DeadMutexIntoInnerMapItem;

pub struct DeadMutexIntoInnerMapPayload {
    value: String,
}

impl DeadMutexIntoInnerMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-mutex-into-inner-map:{}", self.value)
    }
}

pub fn dead_mutex_into_inner_map(raw: &str) -> String {
    DeadMutexIntoInnerMapPayload::new(raw).dead_method()
}
