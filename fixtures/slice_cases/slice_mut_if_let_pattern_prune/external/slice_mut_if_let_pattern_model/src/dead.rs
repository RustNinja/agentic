pub struct DeadSliceMutIfLetPatternItem;

pub struct DeadSliceMutIfLetPatternPayload {
    value: String,
}

impl DeadSliceMutIfLetPatternPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-mut-if-let-pattern:{}", self.value)
    }
}

pub fn dead_slice_mut_if_let_pattern(raw: &str) -> String {
    DeadSliceMutIfLetPatternPayload::new(raw).dead_method()
}
