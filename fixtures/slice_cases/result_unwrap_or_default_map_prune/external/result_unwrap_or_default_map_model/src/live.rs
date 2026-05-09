#[derive(Clone)]
pub struct ResultUnwrapOrDefaultMapPayload {
    value: String,
}

impl ResultUnwrapOrDefaultMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("result-unwrap-or-default-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("result-unwrap-or-default-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-unwrap-or-default-map:{}", self.value)
    }
}

impl Default for ResultUnwrapOrDefaultMapPayload {
    fn default() -> Self {
        Self::new("default")
    }
}

pub struct ResultUnwrapOrDefaultMapError {
    value: String,
}

impl ResultUnwrapOrDefaultMapError {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_error(&self) -> String {
        format!("result-unwrap-or-default-map-error:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-unwrap-or-default-map-error:{}", self.value)
    }
}

pub fn selected_result_unwrap_or_default_map(raw: &str) -> String {
    let result: Result<ResultUnwrapOrDefaultMapPayload, ResultUnwrapOrDefaultMapError> =
        if raw.trim().is_empty() {
            Err(ResultUnwrapOrDefaultMapError::new(raw))
        } else {
            Ok(ResultUnwrapOrDefaultMapPayload::new(raw))
        };
    result.unwrap_or_default().render_label()
}

pub fn dead_live_result_unwrap_or_default_map(raw: &str) -> String {
    let mut payload = ResultUnwrapOrDefaultMapPayload::new(raw);
    payload.bump_and_render()
}
