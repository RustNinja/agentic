pub struct ResultMapOrValuePayload {
    value: String,
}

impl ResultMapOrValuePayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("result-map-or-value:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-map-or-value:{}", self.value)
    }
}

pub struct ResultMapOrValueError {
    code: String,
}

impl ResultMapOrValueError {
    pub fn new(raw: &str) -> Self {
        Self {
            code: raw.to_string(),
        }
    }
}

pub struct ResultMapOrValueDefault {
    value: String,
}

impl ResultMapOrValueDefault {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render_missing(&self) -> String {
        format!("result-map-or-value-missing:{}", self.value)
    }
}

fn result_map_or_value_payload(raw: &str) -> Result<ResultMapOrValuePayload, ResultMapOrValueError> {
    if raw.trim().is_empty() {
        Err(ResultMapOrValueError::new(raw))
    } else {
        Ok(ResultMapOrValuePayload::new(raw))
    }
}

pub fn selected_result_map_or_value(raw: &str) -> String {
    result_map_or_value_payload(raw).map_or(
        ResultMapOrValueDefault::new(raw).render_missing(),
        |payload| payload.render_label(),
    )
}

pub fn dead_live_result_map_or_value(raw: &str) -> String {
    ResultMapOrValuePayload::new(raw).dead_method()
}
