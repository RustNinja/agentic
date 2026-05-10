pub struct DeadOptionUnwrapUncheckedMapItem;

pub struct DeadOptionUnwrapUncheckedMapPayload {
    value: String,
}

impl DeadOptionUnwrapUncheckedMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-unwrap-unchecked-map:{}", self.value)
    }
}

pub fn dead_option_unwrap_unchecked_map(raw: &str) -> String {
    DeadOptionUnwrapUncheckedMapPayload::new(raw).dead_method()
}
