pub struct DeadResultUnwrapErrUncheckedMapItem;

pub struct DeadResultUnwrapErrUncheckedMapPayload {
    value: String,
}

impl DeadResultUnwrapErrUncheckedMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-unwrap-err-unchecked-map:{}", self.value)
    }
}

pub fn dead_result_unwrap_err_unchecked_map(raw: &str) -> String {
    DeadResultUnwrapErrUncheckedMapPayload::new(raw).dead_method()
}
