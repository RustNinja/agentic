pub struct DeadArcUnwrapOrCloneMapItem;

pub struct DeadArcUnwrapOrCloneMapPayload {
    value: String,
}

impl DeadArcUnwrapOrCloneMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-arc-unwrap-or-clone-map:{}", self.value)
    }
}

pub fn dead_arc_unwrap_or_clone_map(raw: &str) -> String {
    DeadArcUnwrapOrCloneMapPayload::new(raw).dead_method()
}
