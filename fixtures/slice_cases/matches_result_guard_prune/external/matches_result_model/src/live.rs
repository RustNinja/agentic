pub struct MatchesResultPayload {
    value: String,
}

impl MatchesResultPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn accepts(&self) -> bool {
        self.render_label().contains("ready")
    }

    pub fn render_label(&self) -> String {
        format!("matches-result:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-matches-result-payload:{}", self.value)
    }
}

pub struct MatchesResultError {
    value: String,
}

impl MatchesResultError {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_error(&self) -> String {
        format!("matches-result-error:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-matches-result-error:{}", self.value)
    }
}

fn matches_result_payload(raw: &str) -> Result<MatchesResultPayload, MatchesResultError> {
    if raw.trim().starts_with("err") {
        Err(MatchesResultError::new(raw))
    } else {
        Ok(MatchesResultPayload::new(raw))
    }
}

pub fn selected_matches_result(raw: &str) -> String {
    let result = matches_result_payload(raw);
    if matches!(result.as_ref(), Ok(payload) if payload.accepts()) {
        result
            .map(|payload| payload.render_label())
            .unwrap_or_else(|err| err.render_error())
    } else {
        "matches-result:rejected".to_string()
    }
}

pub fn dead_live_matches_result(raw: &str) -> String {
    MatchesResultPayload::new(raw).dead_method()
}
