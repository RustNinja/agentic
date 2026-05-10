pub struct DeadForLinkedlistRefPayloadPayload {
    value: String,
}

impl DeadForLinkedlistRefPayloadPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-for-linkedlist-ref-payload:{}", self.value)
    }
}

pub fn dead_for_linkedlist_ref_payload(raw: &str) -> String {
    DeadForLinkedlistRefPayloadPayload::new(raw).dead_method()
}
