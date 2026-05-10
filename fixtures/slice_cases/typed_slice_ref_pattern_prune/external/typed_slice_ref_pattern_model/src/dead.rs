pub struct DeadTypedSliceRefPatternItem;

pub struct DeadTypedSliceRefPatternPayload {
    value: String,
}

impl DeadTypedSliceRefPatternPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-typed-slice-ref-pattern:{}", self.value)
    }
}

pub fn dead_typed_slice_ref_pattern(raw: &str) -> String {
    DeadTypedSliceRefPatternPayload::new(raw).dead_method()
}
