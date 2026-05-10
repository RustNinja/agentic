pub struct DeadResultUnwrapUncheckedMapItem;

pub struct DeadResultUnwrapUncheckedMapPayload {
    value: String,
}

impl DeadResultUnwrapUncheckedMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-unwrap-unchecked-map:{}", self.value)
    }
}

pub fn dead_result_unwrap_unchecked_map(raw: &str) -> String {
    DeadResultUnwrapUncheckedMapPayload::new(raw).dead_method()
}
