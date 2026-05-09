pub struct ResultErrMapPayload {
    value: String,
}

impl ResultErrMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-err-map:{}", self.value)
    }
}

pub struct ResultErrMapError {
    code: String,
}

impl ResultErrMapError {
    pub fn new(raw: &str) -> Self {
        Self {
            code: raw.to_string(),
        }
    }

    pub fn render_error(&self) -> String {
        format!("result-err-map-error:{}", self.code)
    }
}

fn result_err_map_payload(raw: &str) -> Result<ResultErrMapPayload, ResultErrMapError> {
    if raw.trim().is_empty() {
        Err(ResultErrMapError::new(raw))
    } else {
        Ok(ResultErrMapPayload::new(raw))
    }
}

pub fn selected_result_err_map(raw: &str) -> String {
    result_err_map_payload(raw)
        .err()
        .map(|err| err.render_error())
        .unwrap_or_else(|| "result-err-map:ok".to_string())
}

pub fn dead_live_result_err_map(raw: &str) -> String {
    ResultErrMapPayload::new(raw).dead_method()
}
