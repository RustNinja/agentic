pub struct NestedMatchPayload {
    value: String,
}

impl NestedMatchPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("nested-match:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-nested-match-payload:{}", self.value)
    }
}

pub struct NestedMatchError {
    value: String,
}

impl NestedMatchError {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_error(&self) -> String {
        format!("nested-match-error:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-nested-match-error:{}", self.value)
    }
}

fn nested_match_payload(raw: &str) -> Option<Result<NestedMatchPayload, NestedMatchError>> {
    if raw.trim().is_empty() {
        None
    } else if raw.trim().starts_with("err") {
        Some(Err(NestedMatchError::new(raw)))
    } else {
        Some(Ok(NestedMatchPayload::new(raw)))
    }
}

pub fn selected_nested_match(raw: &str) -> String {
    match nested_match_payload(raw) {
        Some(Ok(payload)) => payload.render_label(),
        Some(Err(err)) => err.render_error(),
        None => "nested-match:missing".to_string(),
    }
}

pub fn dead_live_nested_match(raw: &str) -> String {
    NestedMatchPayload::new(raw).dead_method()
}
