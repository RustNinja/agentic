pub struct DeadTypedSliceTuplePatternItem;

pub struct DeadTypedSliceTuplePatternPayload {
    value: String,
}

impl DeadTypedSliceTuplePatternPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-typed-slice-tuple-pattern:{}", self.value)
    }
}

pub fn dead_typed_slice_tuple_pattern(raw: &str) -> String {
    DeadTypedSliceTuplePatternPayload::new(raw).dead_method()
}
