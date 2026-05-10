pub struct DeadArcIntoInnerMapItem;

pub struct DeadArcIntoInnerMapPayload {
    value: String,
}

impl DeadArcIntoInnerMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-arc-into-inner-map:{}", self.value)
    }
}

pub fn dead_arc_into_inner_map(raw: &str) -> String {
    DeadArcIntoInnerMapPayload::new(raw).dead_method()
}
