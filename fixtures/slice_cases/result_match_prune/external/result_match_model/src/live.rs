pub struct ResultMatchPayload {
    value: String,
}

impl ResultMatchPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("result-match:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-match-payload:{}", self.value)
    }
}

pub struct ResultMatchError {
    value: String,
}

impl ResultMatchError {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_error(&self) -> String {
        format!("result-match-error:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-match-error:{}", self.value)
    }
}

fn result_match_payload(raw: &str) -> Result<ResultMatchPayload, ResultMatchError> {
    if raw.trim().starts_with("err") {
        Err(ResultMatchError::new(raw))
    } else {
        Ok(ResultMatchPayload::new(raw))
    }
}

pub fn selected_result_match(raw: &str) -> String {
    match result_match_payload(raw) {
        Ok(payload) => payload.render_label(),
        Err(err) => err.render_error(),
    }
}

pub fn dead_live_result_match(raw: &str) -> String {
    ResultMatchPayload::new(raw).dead_method()
}
