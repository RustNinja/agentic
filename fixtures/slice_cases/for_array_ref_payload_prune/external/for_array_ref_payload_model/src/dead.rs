pub struct DeadForArrayRefPayloadPayload {
    value: String,
}

impl DeadForArrayRefPayloadPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-for-array-ref-payload:{}", self.value)
    }
}

pub fn dead_for_array_ref_payload(raw: &str) -> String {
    DeadForArrayRefPayloadPayload::new(raw).dead_method()
}
