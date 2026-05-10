pub struct DeadTypedSliceNamedStructPatternItem;

pub struct DeadTypedSliceNamedStructPatternPayload {
    value: String,
}

impl DeadTypedSliceNamedStructPatternPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-typed-slice-named-struct-pattern:{}", self.value)
    }
}

pub fn dead_typed_slice_named_struct_pattern(raw: &str) -> String {
    DeadTypedSliceNamedStructPatternPayload::new(raw).dead_method()
}
