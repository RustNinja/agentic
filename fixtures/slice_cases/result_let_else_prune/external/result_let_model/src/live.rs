pub struct ResultLetPayload {
    value: String,
}

impl ResultLetPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("result-let:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-let-payload:{}", self.value)
    }
}

pub struct ResultLetError {
    value: String,
}

impl ResultLetError {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_error(&self) -> String {
        format!("result-let-error:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-let-error:{}", self.value)
    }
}

fn result_let_payload(raw: &str) -> Result<ResultLetPayload, ResultLetError> {
    if raw.trim().starts_with("err") {
        Err(ResultLetError::new(raw))
    } else {
        Ok(ResultLetPayload::new(raw))
    }
}

pub fn selected_result_let(raw: &str) -> String {
    let Ok(payload) = result_let_payload(raw) else {
        return ResultLetError::new(raw).render_error();
    };
    payload.render_label()
}

pub fn dead_live_result_let(raw: &str) -> String {
    ResultLetPayload::new(raw).dead_method()
}
