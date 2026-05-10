pub struct DeadForOptionIterPayloadPayload {
    value: String,
}

impl DeadForOptionIterPayloadPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-for-option-iter-payload:{}", self.value)
    }
}

pub fn dead_for_option_iter_payload(raw: &str) -> String {
    DeadForOptionIterPayloadPayload::new(raw).dead_method()
}
