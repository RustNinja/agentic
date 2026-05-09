pub struct DeadResultErrMapItem {
    value: String,
}

impl DeadResultErrMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-err-map:{}", self.value)
    }
}

pub fn dead_result_err_map(raw: &str) -> String {
    DeadResultErrMapItem::new(raw).dead_method()
}
