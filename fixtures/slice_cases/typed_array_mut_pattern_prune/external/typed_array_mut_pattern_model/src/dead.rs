pub struct DeadTypedArrayMutPatternItem;

pub struct DeadTypedArrayMutPatternPayload {
    value: String,
}

impl DeadTypedArrayMutPatternPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-typed-array-mut-pattern:{}", self.value)
    }
}

pub fn dead_typed_array_mut_pattern(raw: &str) -> String {
    DeadTypedArrayMutPatternPayload::new(raw).dead_method()
}
