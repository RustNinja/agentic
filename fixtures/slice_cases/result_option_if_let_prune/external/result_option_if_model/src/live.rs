pub struct ResultOptionIfPayload {
    value: String,
}

impl ResultOptionIfPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("result-option-if:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-option-if:{}", self.value)
    }
}

pub struct ResultOptionIfError {
    value: String,
}

impl ResultOptionIfError {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render_error(&self) -> String {
        format!("result-option-if-error:{}", self.value)
    }
}

fn result_option_if_payload(raw: &str) -> Result<Option<ResultOptionIfPayload>, ResultOptionIfError> {
    if raw == "err" {
        Err(ResultOptionIfError::new(raw))
    } else if raw.trim().is_empty() {
        Ok(None)
    } else {
        Ok(Some(ResultOptionIfPayload::new(raw)))
    }
}

pub fn selected_result_option_if(raw: &str) -> String {
    if let Ok(Some(payload)) = result_option_if_payload(raw) {
        payload.render_label()
    } else if let Err(err) = result_option_if_payload(raw) {
        err.render_error()
    } else {
        "result-option-if:empty".to_string()
    }
}

pub fn dead_live_result_option_if(raw: &str) -> String {
    ResultOptionIfPayload::new(raw).dead_method()
}
