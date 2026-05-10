pub struct DeadForVecMutPayloadPayload {
    value: String,
}

impl DeadForVecMutPayloadPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-for-vec-mut-payload:{}", self.value)
    }
}

pub fn dead_for_vec_mut_payload(raw: &str) -> String {
    DeadForVecMutPayloadPayload::new(raw).dead_method()
}
