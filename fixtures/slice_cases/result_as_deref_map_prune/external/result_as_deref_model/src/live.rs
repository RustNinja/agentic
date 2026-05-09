pub struct ResultAsDerefMapPayload {
    value: String,
}

impl ResultAsDerefMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("result-as-deref-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("result-as-deref-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-as-deref-map:{}", self.value)
    }
}

pub struct ResultAsDerefMapError {
    code: String,
}

impl ResultAsDerefMapError {
    pub fn new(raw: &str) -> Self {
        Self {
            code: raw.to_string(),
        }
    }

    pub fn render_error(&self) -> String {
        format!("result-as-deref-map-error:{}", self.code)
    }

    pub fn dead_error_method(&self) -> String {
        format!("dead-result-as-deref-map-error:{}", self.code)
    }
}

fn result_as_deref_map_payload(raw: &str) -> Result<Box<ResultAsDerefMapPayload>, ResultAsDerefMapError> {
    if raw.trim().is_empty() {
        Err(ResultAsDerefMapError::new(raw))
    } else {
        Ok(Box::new(ResultAsDerefMapPayload::new(raw)))
    }
}

pub fn selected_result_as_deref_map(raw: &str) -> String {
    result_as_deref_map_payload(raw)
        .as_deref()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|err| err.render_error())
}

pub fn dead_live_result_as_deref_map(raw: &str) -> String {
    ResultAsDerefMapPayload::new(raw).dead_method()
}
