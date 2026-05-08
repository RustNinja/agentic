pub struct ResultValue {
    label: String,
}

impl ResultValue {
    pub fn render(self) -> String {
        format!("result:{}", self.label)
    }

    pub fn dead_method(self) -> String {
        format!("dead-result:{}", self.label)
    }
}

pub struct ResultError {
    message: String,
}

impl ResultError {
    pub fn render(self) -> String {
        format!("result-error:{}", self.message)
    }

    pub fn dead_method(self) -> String {
        format!("dead-result-error:{}", self.message)
    }
}

pub fn parse_result(raw: &str) -> Result<ResultValue, ResultError> {
    if raw.trim().is_empty() {
        Err(ResultError {
            message: "empty".to_string(),
        })
    } else {
        Ok(ResultValue {
            label: raw.trim().to_string(),
        })
    }
}

pub fn dead_live_result(raw: &str) -> String {
    parse_result(raw)
        .map(|value| value.dead_method())
        .map_err(|err| err.dead_method())
        .unwrap_or_else(|message| message)
}
