pub struct DeadForHashsetRefPayloadPayload {
    value: String,
}

impl DeadForHashsetRefPayloadPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-for-hashset-ref-payload:{}", self.value)
    }
}

pub fn dead_for_hashset_ref_payload(raw: &str) -> String {
    DeadForHashsetRefPayloadPayload::new(raw).dead_method()
}
