pub struct DeadRcIntoInnerMapItem;

pub struct DeadRcIntoInnerMapPayload {
    value: String,
}

impl DeadRcIntoInnerMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-rc-into-inner-map:{}", self.value)
    }
}

pub fn dead_rc_into_inner_map(raw: &str) -> String {
    DeadRcIntoInnerMapPayload::new(raw).dead_method()
}
