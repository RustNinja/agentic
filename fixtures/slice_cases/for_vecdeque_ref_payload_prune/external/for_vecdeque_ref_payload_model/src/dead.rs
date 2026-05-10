pub struct DeadForVecdequeRefPayloadPayload {
    value: String,
}

impl DeadForVecdequeRefPayloadPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-for-vecdeque-ref-payload:{}", self.value)
    }
}

pub fn dead_for_vecdeque_ref_payload(raw: &str) -> String {
    DeadForVecdequeRefPayloadPayload::new(raw).dead_method()
}
