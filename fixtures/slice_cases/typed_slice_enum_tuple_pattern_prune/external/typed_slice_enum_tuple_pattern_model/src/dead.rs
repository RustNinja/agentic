pub struct DeadTypedSliceEnumTuplePatternItem;

pub struct DeadTypedSliceEnumTuplePatternPayload {
    value: String,
}

impl DeadTypedSliceEnumTuplePatternPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-typed-slice-enum-tuple-pattern:{}", self.value)
    }
}

pub fn dead_typed_slice_enum_tuple_pattern(raw: &str) -> String {
    DeadTypedSliceEnumTuplePatternPayload::new(raw).dead_method()
}
