pub struct ResultAsRefMapPayload {
    value: String,
}

impl ResultAsRefMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("result-as-ref-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("result-as-ref-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-as-ref-map:{}", self.value)
    }
}

pub struct ResultAsRefMapError {
    code: String,
}

impl ResultAsRefMapError {
    pub fn new(raw: &str) -> Self {
        Self {
            code: raw.to_string(),
        }
    }

    pub fn render_error(&self) -> String {
        format!("result-as-ref-map-error:{}", self.code)
    }

    pub fn dead_error_method(&self) -> String {
        format!("dead-result-as-ref-map-error:{}", self.code)
    }
}

fn result_as_ref_map_payload(raw: &str) -> Result<ResultAsRefMapPayload, ResultAsRefMapError> {
    if raw.trim().is_empty() {
        Err(ResultAsRefMapError::new(raw))
    } else {
        Ok(ResultAsRefMapPayload::new(raw))
    }
}

pub fn selected_result_as_ref_map(raw: &str) -> String {
    result_as_ref_map_payload(raw)
        .as_ref()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|err| err.render_error())
}

pub fn dead_live_result_as_ref_map(raw: &str) -> String {
    ResultAsRefMapPayload::new(raw).dead_method()
}
