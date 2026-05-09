pub struct ResultErrCheckPayload {
    value: String,
}

impl ResultErrCheckPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("result-err-check:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-err-check:{}", self.value)
    }
}

pub struct ResultErrCheckError {
    retryable: bool,
}

impl ResultErrCheckError {
    pub fn new(raw: &str) -> Self {
        Self {
            retryable: raw.contains("retry"),
        }
    }

    pub fn is_retryable(&self) -> bool {
        self.retryable
    }
}

fn result_err_check_payload(raw: &str) -> Result<ResultErrCheckPayload, ResultErrCheckError> {
    if raw.contains("err") {
        Err(ResultErrCheckError::new(raw))
    } else {
        Ok(ResultErrCheckPayload::new(raw))
    }
}

pub fn selected_result_err_check(raw: &str) -> String {
    if result_err_check_payload(raw).is_err_and(|err| err.is_retryable()) {
        "result-err-check:retry".to_string()
    } else {
        ResultErrCheckPayload::new(raw).render_label()
    }
}

pub fn dead_live_result_err_check(raw: &str) -> String {
    ResultErrCheckPayload::new(raw).dead_method()
}
