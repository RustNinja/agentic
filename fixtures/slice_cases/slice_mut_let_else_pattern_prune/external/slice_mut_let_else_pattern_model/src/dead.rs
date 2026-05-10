pub struct DeadSliceMutLetElsePatternItem;

pub struct DeadSliceMutLetElsePatternPayload {
    value: String,
}

impl DeadSliceMutLetElsePatternPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-mut-let-else-pattern:{}", self.value)
    }
}

pub fn dead_slice_mut_let_else_pattern(raw: &str) -> String {
    DeadSliceMutLetElsePatternPayload::new(raw).dead_method()
}
