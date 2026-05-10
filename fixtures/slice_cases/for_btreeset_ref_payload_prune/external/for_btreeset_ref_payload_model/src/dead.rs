pub struct DeadForBtreesetRefPayloadPayload {
    value: String,
}

impl DeadForBtreesetRefPayloadPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-for-btreeset-ref-payload:{}", self.value)
    }
}

pub fn dead_for_btreeset_ref_payload(raw: &str) -> String {
    DeadForBtreesetRefPayloadPayload::new(raw).dead_method()
}
