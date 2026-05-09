pub struct ResultOkMapPayload {
    value: String,
}

impl ResultOkMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("result-ok-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-ok-map:{}", self.value)
    }
}

pub struct ResultOkMapError {
    code: String,
}

impl ResultOkMapError {
    pub fn new(raw: &str) -> Self {
        Self {
            code: raw.to_string(),
        }
    }
}

fn result_ok_map_payload(raw: &str) -> Result<ResultOkMapPayload, ResultOkMapError> {
    if raw.trim().is_empty() {
        Err(ResultOkMapError::new(raw))
    } else {
        Ok(ResultOkMapPayload::new(raw))
    }
}

pub fn selected_result_ok_map(raw: &str) -> String {
    result_ok_map_payload(raw)
        .ok()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "result-ok-map:missing".to_string())
}

pub fn dead_live_result_ok_map(raw: &str) -> String {
    ResultOkMapPayload::new(raw).dead_method()
}
