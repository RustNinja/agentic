pub struct DeadTypedArrayRefPatternItem;

pub struct DeadTypedArrayRefPatternPayload {
    value: String,
}

impl DeadTypedArrayRefPatternPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-typed-array-ref-pattern:{}", self.value)
    }
}

pub fn dead_typed_array_ref_pattern(raw: &str) -> String {
    DeadTypedArrayRefPatternPayload::new(raw).dead_method()
}
