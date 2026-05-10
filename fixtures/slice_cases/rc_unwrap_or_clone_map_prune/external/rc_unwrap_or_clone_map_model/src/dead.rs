pub struct DeadRcUnwrapOrCloneMapItem;

pub struct DeadRcUnwrapOrCloneMapPayload {
    value: String,
}

impl DeadRcUnwrapOrCloneMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-rc-unwrap-or-clone-map:{}", self.value)
    }
}

pub fn dead_rc_unwrap_or_clone_map(raw: &str) -> String {
    DeadRcUnwrapOrCloneMapPayload::new(raw).dead_method()
}
