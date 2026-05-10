pub struct DeadResultUnwrapErrMapItem;

pub struct DeadResultUnwrapErrMapPayload {
    value: String,
}

impl DeadResultUnwrapErrMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-unwrap-err-map:{}", self.value)
    }
}

pub fn dead_result_unwrap_err_map(raw: &str) -> String {
    DeadResultUnwrapErrMapPayload::new(raw).dead_method()
}
