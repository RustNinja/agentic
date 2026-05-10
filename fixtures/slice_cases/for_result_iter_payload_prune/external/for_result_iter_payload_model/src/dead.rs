pub struct DeadForResultIterPayloadPayload {
    value: String,
}

impl DeadForResultIterPayloadPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-for-result-iter-payload:{}", self.value)
    }
}

pub fn dead_for_result_iter_payload(raw: &str) -> String {
    DeadForResultIterPayloadPayload::new(raw).dead_method()
}
