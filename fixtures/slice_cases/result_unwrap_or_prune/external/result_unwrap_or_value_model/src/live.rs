pub struct ResultUnwrapOrValuePayload {
    value: String,
}

impl ResultUnwrapOrValuePayload {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.trim().to_string() }
    }

    pub fn fallback(raw: &str) -> Self {
        Self { value: format!("fallback-{raw}") }
    }

    pub fn render_label(&self) -> String {
        format!("result-unwrap-or-value:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-unwrap-or-value:{}", self.value)
    }
}

#[derive(Debug)]
pub struct ResultUnwrapOrValueError {
    value: String,
}

impl ResultUnwrapOrValueError {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.to_string() }
    }
}

fn result_unwrap_or_value_payload(raw: &str) -> Result<ResultUnwrapOrValuePayload, ResultUnwrapOrValueError> {
    if raw.trim().is_empty() {
        Err(ResultUnwrapOrValueError::new(raw))
    } else {
        Ok(ResultUnwrapOrValuePayload::new(raw))
    }
}

pub fn selected_result_unwrap_or_value(raw: &str) -> String {
    result_unwrap_or_value_payload(raw)
        .unwrap_or(ResultUnwrapOrValuePayload::fallback(raw))
        .render_label()
}

pub fn dead_live_result_unwrap_or_value(raw: &str) -> String {
    ResultUnwrapOrValuePayload::new(raw).dead_method()
}
