pub struct DeadForResultIterMutPayloadPayload {
    value: String,
}

impl DeadForResultIterMutPayloadPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-for-result-iter-mut-payload:{}", self.value)
    }
}

pub fn dead_for_result_iter_mut_payload(raw: &str) -> String {
    DeadForResultIterMutPayloadPayload::new(raw).dead_method()
}
