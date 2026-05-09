pub struct NestedIfPayload {
    value: String,
}

impl NestedIfPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("nested-if:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-nested-if-payload:{}", self.value)
    }
}

pub struct NestedIfError {
    value: String,
}

impl NestedIfError {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_error(&self) -> String {
        format!("nested-if-error:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-nested-if-error:{}", self.value)
    }
}

fn nested_if_payload(raw: &str) -> Option<Result<NestedIfPayload, NestedIfError>> {
    if raw.trim().is_empty() {
        None
    } else if raw.trim().starts_with("err") {
        Some(Err(NestedIfError::new(raw)))
    } else {
        Some(Ok(NestedIfPayload::new(raw)))
    }
}

pub fn selected_nested_if(raw: &str) -> String {
    if let Some(Ok(payload)) = nested_if_payload(raw) {
        payload.render_label()
    } else if let Some(Err(err)) = nested_if_payload(raw) {
        err.render_error()
    } else {
        "nested-if:missing".to_string()
    }
}

pub fn dead_live_nested_if(raw: &str) -> String {
    NestedIfPayload::new(raw).dead_method()
}
