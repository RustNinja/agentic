pub struct DeadTypedSliceTupleMutPatternItem;

pub struct DeadTypedSliceTupleMutPatternPayload {
    value: String,
}

impl DeadTypedSliceTupleMutPatternPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-typed-slice-tuple-mut-pattern:{}", self.value)
    }
}

pub fn dead_typed_slice_tuple_mut_pattern(raw: &str) -> String {
    DeadTypedSliceTupleMutPatternPayload::new(raw).dead_method()
}
