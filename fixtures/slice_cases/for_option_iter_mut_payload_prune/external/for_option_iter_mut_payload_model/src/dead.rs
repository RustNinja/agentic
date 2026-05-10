pub struct DeadForOptionIterMutPayloadPayload {
    value: String,
}

impl DeadForOptionIterMutPayloadPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-for-option-iter-mut-payload:{}", self.value)
    }
}

pub fn dead_for_option_iter_mut_payload(raw: &str) -> String {
    DeadForOptionIterMutPayloadPayload::new(raw).dead_method()
}
