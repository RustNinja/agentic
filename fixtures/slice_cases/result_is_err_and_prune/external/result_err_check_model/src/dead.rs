pub struct DeadResultErrCheckItem {
    value: String,
}

impl DeadResultErrCheckItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-err-check:{}", self.value)
    }
}

pub fn dead_result_err_check(raw: &str) -> String {
    DeadResultErrCheckItem::new(raw).dead_method()
}
