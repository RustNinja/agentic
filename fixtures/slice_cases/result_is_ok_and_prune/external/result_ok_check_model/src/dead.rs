pub struct DeadResultOkCheckItem {
    value: String,
}

impl DeadResultOkCheckItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-ok-check:{}", self.value)
    }
}

pub fn dead_result_ok_check(raw: &str) -> String {
    DeadResultOkCheckItem::new(raw).dead_method()
}
