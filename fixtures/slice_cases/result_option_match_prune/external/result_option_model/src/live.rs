pub struct ResultOptionPayload {
    value: String,
}

impl ResultOptionPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("result-option:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-option:{}", self.value)
    }
}

pub struct ResultOptionError {
    value: String,
}

impl ResultOptionError {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render_error(&self) -> String {
        format!("result-option-error:{}", self.value)
    }
}

fn result_option_payload(raw: &str) -> Result<Option<ResultOptionPayload>, ResultOptionError> {
    if raw == "err" {
        Err(ResultOptionError::new(raw))
    } else if raw.trim().is_empty() {
        Ok(None)
    } else {
        Ok(Some(ResultOptionPayload::new(raw)))
    }
}

pub fn selected_result_option(raw: &str) -> String {
    match result_option_payload(raw) {
        Ok(Some(payload)) => payload.render_label(),
        Ok(None) => "result-option:empty".to_string(),
        Err(err) => err.render_error(),
    }
}

pub fn dead_live_result_option(raw: &str) -> String {
    ResultOptionPayload::new(raw).dead_method()
}
