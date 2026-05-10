pub struct DeadSliceLetElsePatternItem;

pub struct DeadSliceLetElsePatternPayload {
    value: String,
}

impl DeadSliceLetElsePatternPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-let-else-pattern:{}", self.value)
    }
}

pub fn dead_slice_let_else_pattern(raw: &str) -> String {
    DeadSliceLetElsePatternPayload::new(raw).dead_method()
}
