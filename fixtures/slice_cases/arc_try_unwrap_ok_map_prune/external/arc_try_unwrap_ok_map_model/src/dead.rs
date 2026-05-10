pub struct DeadArcTryUnwrapOkMapItem;

pub struct DeadArcTryUnwrapOkMapPayload {
    value: String,
}

impl DeadArcTryUnwrapOkMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-arc-try-unwrap-ok-map:{}", self.value)
    }
}

pub fn dead_arc_try_unwrap_ok_map(raw: &str) -> String {
    DeadArcTryUnwrapOkMapPayload::new(raw).dead_method()
}
