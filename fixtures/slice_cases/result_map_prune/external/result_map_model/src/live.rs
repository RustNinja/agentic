pub struct ResultMapPayload {
    value: String,
}

impl ResultMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("result-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-map:{}", self.value)
    }
}

pub struct ResultMapError {
    code: String,
}

impl ResultMapError {
    pub fn new(raw: &str) -> Self {
        Self {
            code: raw.to_string(),
        }
    }

    pub fn render_error(&self) -> String {
        format!("result-map-error:{}", self.code)
    }
}

fn result_map_payload(raw: &str) -> Result<ResultMapPayload, ResultMapError> {
    if raw.trim().is_empty() {
        Err(ResultMapError::new(raw))
    } else {
        Ok(ResultMapPayload::new(raw))
    }
}

pub fn selected_result_map(raw: &str) -> String {
    result_map_payload(raw)
        .map(|payload| payload.render_label())
        .unwrap_or_else(|err| err.render_error())
}

pub fn dead_live_result_map(raw: &str) -> String {
    ResultMapPayload::new(raw).dead_method()
}
