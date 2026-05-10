pub struct DeadTypedArrayPatternItem;

pub struct DeadTypedArrayPatternPayload {
    value: String,
}

impl DeadTypedArrayPatternPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-typed-array-pattern:{}", self.value)
    }
}

pub fn dead_typed_array_pattern(raw: &str) -> String {
    DeadTypedArrayPatternPayload::new(raw).dead_method()
}
