pub struct DeadTypedSliceTupleStructPatternItem;

pub struct DeadTypedSliceTupleStructPatternPayload {
    value: String,
}

impl DeadTypedSliceTupleStructPatternPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-typed-slice-tuple-struct-pattern:{}", self.value)
    }
}

pub fn dead_typed_slice_tuple_struct_pattern(raw: &str) -> String {
    DeadTypedSliceTupleStructPatternPayload::new(raw).dead_method()
}
