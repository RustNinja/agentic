pub struct DeadTypedSliceMutPatternItem;

pub struct DeadTypedSliceMutPatternPayload {
    value: String,
}

impl DeadTypedSliceMutPatternPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-typed-slice-mut-pattern:{}", self.value)
    }
}

pub fn dead_typed_slice_mut_pattern(raw: &str) -> String {
    DeadTypedSliceMutPatternPayload::new(raw).dead_method()
}
