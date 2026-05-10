pub struct DeadRcTryUnwrapOkMapItem;

pub struct DeadRcTryUnwrapOkMapPayload {
    value: String,
}

impl DeadRcTryUnwrapOkMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-rc-try-unwrap-ok-map:{}", self.value)
    }
}

pub fn dead_rc_try_unwrap_ok_map(raw: &str) -> String {
    DeadRcTryUnwrapOkMapPayload::new(raw).dead_method()
}
