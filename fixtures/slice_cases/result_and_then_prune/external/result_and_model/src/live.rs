pub struct ResultAndPayload {
    value: String,
}

impl ResultAndPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn expand(&self) -> Result<String, ResultAndError> {
        Ok(format!("result-and:{}", self.render_label()))
    }

    pub fn render_label(&self) -> String {
        format!("label:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-and-payload:{}", self.value)
    }
}

pub struct ResultAndError {
    value: String,
}

impl ResultAndError {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_error(&self) -> String {
        format!("result-and-error:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-and-error:{}", self.value)
    }
}

fn result_and_payload(raw: &str) -> Result<ResultAndPayload, ResultAndError> {
    if raw.trim().starts_with("err") {
        Err(ResultAndError::new(raw))
    } else {
        Ok(ResultAndPayload::new(raw))
    }
}

pub fn selected_result_and(raw: &str) -> String {
    result_and_payload(raw)
        .and_then(|payload| payload.expand())
        .unwrap_or_else(|err| err.render_error())
}

pub fn dead_live_result_and(raw: &str) -> String {
    ResultAndPayload::new(raw).dead_method()
}
