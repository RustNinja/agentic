pub struct DeadSliceIfLetPatternItem;

pub struct DeadSliceIfLetPatternPayload {
    value: String,
}

impl DeadSliceIfLetPatternPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-if-let-pattern:{}", self.value)
    }
}

pub fn dead_slice_if_let_pattern(raw: &str) -> String {
    DeadSliceIfLetPatternPayload::new(raw).dead_method()
}
