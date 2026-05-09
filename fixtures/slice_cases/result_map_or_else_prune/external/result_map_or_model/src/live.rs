pub struct ResultMapOrPayload {
    value: String,
}

impl ResultMapOrPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("result-map-or:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-map-or:{}", self.value)
    }
}

pub struct ResultMapOrError {
    value: String,
}

impl ResultMapOrError {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render_error(&self) -> String {
        format!("result-map-or-error:{}", self.value)
    }
}

fn result_map_or_payload(raw: &str) -> Result<ResultMapOrPayload, ResultMapOrError> {
    if raw == "err" {
        Err(ResultMapOrError::new(raw))
    } else {
        Ok(ResultMapOrPayload::new(raw))
    }
}

pub fn selected_result_map_or(raw: &str) -> String {
    result_map_or_payload(raw).map_or_else(|err| err.render_error(), |payload| payload.render_label())
}

pub fn dead_live_result_map_or(raw: &str) -> String {
    ResultMapOrPayload::new(raw).dead_method()
}
