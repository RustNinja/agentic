#[derive(Clone, Copy)]
pub struct ResultCopiedMapPayload {
    value: usize,
}

impl ResultCopiedMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().len(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("result-copied-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-copied-map:{}", self.value)
    }
}

pub struct ResultCopiedMapError {
    value: String,
}

impl ResultCopiedMapError {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_error(&self) -> String {
        format!("result-copied-map-error:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-copied-map-error:{}", self.value)
    }
}

pub fn selected_result_copied_map(raw: &str) -> String {
    let payload = ResultCopiedMapPayload::new(raw);
    let result: Result<&ResultCopiedMapPayload, ResultCopiedMapError> = Ok(&payload);
    result
        .copied()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|err| err.render_error())
}

pub fn dead_live_result_copied_map(raw: &str) -> String {
    ResultCopiedMapPayload::new(raw).dead_method()
}
