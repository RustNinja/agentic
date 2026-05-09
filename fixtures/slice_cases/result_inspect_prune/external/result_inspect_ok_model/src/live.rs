pub struct ResultInspectOkPayload {
    value: String,
}

impl ResultInspectOkPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn audit(&self) -> String {
        format!("audit:{}", self.render_label())
    }

    pub fn render_label(&self) -> String {
        format!("result-inspect-ok:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-inspect-ok-payload:{}", self.value)
    }
}

pub struct ResultInspectOkError {
    value: String,
}

impl ResultInspectOkError {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_error(&self) -> String {
        format!("result-inspect-ok-error:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-inspect-ok-error:{}", self.value)
    }
}

fn result_inspect_ok_payload(raw: &str) -> Result<ResultInspectOkPayload, ResultInspectOkError> {
    if raw.trim().starts_with("err") {
        Err(ResultInspectOkError::new(raw))
    } else {
        Ok(ResultInspectOkPayload::new(raw))
    }
}

pub fn selected_result_inspect_ok(raw: &str) -> String {
    let mut audit = Vec::new();
    result_inspect_ok_payload(raw)
        .inspect(|payload| audit.push(payload.audit()))
        .map(|payload| payload.render_label())
        .unwrap_or_else(|err| err.render_error())
}

pub fn dead_live_result_inspect_ok(raw: &str) -> String {
    ResultInspectOkError::new(raw).dead_method()
}
