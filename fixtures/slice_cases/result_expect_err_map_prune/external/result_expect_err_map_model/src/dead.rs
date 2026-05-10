pub struct DeadResultExpectErrMapItem;

pub struct DeadResultExpectErrMapPayload {
    value: String,
}

impl DeadResultExpectErrMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-expect-err-map:{}", self.value)
    }
}

pub fn dead_result_expect_err_map(raw: &str) -> String {
    DeadResultExpectErrMapPayload::new(raw).dead_method()
}
