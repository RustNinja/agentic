pub struct DeadTypedSliceEnumStructPatternItem;

pub struct DeadTypedSliceEnumStructPatternPayload {
    value: String,
}

impl DeadTypedSliceEnumStructPatternPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-typed-slice-enum-struct-pattern:{}", self.value)
    }
}

pub fn dead_typed_slice_enum_struct_pattern(raw: &str) -> String {
    DeadTypedSliceEnumStructPatternPayload::new(raw).dead_method()
}
