pub struct DeadForVecRefPayloadPayload {
    value: String,
}

impl DeadForVecRefPayloadPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-for-vec-ref-payload:{}", self.value)
    }
}

pub fn dead_for_vec_ref_payload(raw: &str) -> String {
    DeadForVecRefPayloadPayload::new(raw).dead_method()
}
