pub struct DeadRcTryUnwrapUnwrapMapItem;

pub struct DeadRcTryUnwrapUnwrapMapPayload {
    value: String,
}

impl DeadRcTryUnwrapUnwrapMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-rc-try-unwrap-unwrap-map:{}", self.value)
    }
}

pub fn dead_rc_try_unwrap_unwrap_map(raw: &str) -> String {
    DeadRcTryUnwrapUnwrapMapPayload::new(raw).dead_method()
}
