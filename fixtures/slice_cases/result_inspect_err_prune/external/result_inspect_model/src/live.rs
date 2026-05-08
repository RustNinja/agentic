pub struct ResultInspectPayload {
    value: String,
}

impl ResultInspectPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("result-inspect:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-inspect-payload:{}", self.value)
    }
}

pub struct ResultInspectError {
    value: String,
}

impl ResultInspectError {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn audit(&self) -> String {
        format!("audit:{}", self.render_error())
    }

    pub fn render_error(&self) -> String {
        format!("result-inspect-error:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-inspect-error:{}", self.value)
    }
}

fn result_inspect_payload(raw: &str) -> Result<ResultInspectPayload, ResultInspectError> {
    if raw.trim().starts_with("err") {
        Err(ResultInspectError::new(raw))
    } else {
        Ok(ResultInspectPayload::new(raw))
    }
}

pub fn selected_result_inspect(raw: &str) -> String {
    let mut audit = Vec::new();
    result_inspect_payload(raw)
        .inspect_err(|err| audit.push(err.audit()))
        .map(|payload| payload.render_label())
        .unwrap_or_else(|err| format!("{}:{}", audit.len(), err.render_error()))
}

pub fn dead_live_result_inspect(raw: &str) -> String {
    ResultInspectPayload::new(raw).dead_method()
}
