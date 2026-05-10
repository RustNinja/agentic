pub struct DeadArcTryUnwrapUnwrapMapItem;

pub struct DeadArcTryUnwrapUnwrapMapPayload {
    value: String,
}

impl DeadArcTryUnwrapUnwrapMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-arc-try-unwrap-unwrap-map:{}", self.value)
    }
}

pub fn dead_arc_try_unwrap_unwrap_map(raw: &str) -> String {
    DeadArcTryUnwrapUnwrapMapPayload::new(raw).dead_method()
}
