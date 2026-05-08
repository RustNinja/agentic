pub struct ResultOrPayload {
    value: String,
}

impl ResultOrPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("result-or:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-or-payload:{}", self.value)
    }
}

pub struct ResultOrError {
    value: String,
}

impl ResultOrError {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn recover(&self) -> Result<ResultOrPayload, String> {
        Ok(ResultOrPayload::new(&format!("recovered-{}", self.value)))
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-or-error:{}", self.value)
    }
}

fn result_or_payload(raw: &str) -> Result<ResultOrPayload, ResultOrError> {
    if raw.trim().starts_with("err") {
        Err(ResultOrError::new(raw))
    } else {
        Ok(ResultOrPayload::new(raw))
    }
}

pub fn selected_result_or(raw: &str) -> String {
    result_or_payload(raw)
        .or_else(|err| err.recover())
        .map(|payload| payload.render_label())
        .unwrap_or_else(|err| err)
}

pub fn dead_live_result_or(raw: &str) -> String {
    ResultOrError::new(raw).dead_method()
}
