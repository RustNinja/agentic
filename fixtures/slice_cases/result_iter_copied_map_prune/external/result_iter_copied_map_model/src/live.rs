#[derive(Clone, Copy)]
pub struct ResultIterCopiedMapPayload {
    value: usize,
}

impl ResultIterCopiedMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().len(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("result-iter-copied-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-iter-copied-map:{}", self.value)
    }
}

pub struct ResultIterCopiedMapError {
    value: String,
}

impl ResultIterCopiedMapError {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_error(&self) -> String {
        format!("result-iter-copied-map-error:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-iter-copied-map-error:{}", self.value)
    }
}

pub fn selected_result_iter_copied_map(raw: &str) -> String {
    let result: Result<ResultIterCopiedMapPayload, ResultIterCopiedMapError> =
        Ok(ResultIterCopiedMapPayload::new(raw));
    result
        .iter()
        .copied()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| format!("result-iter-copied-map:missing"))
}

pub fn dead_live_result_iter_copied_map(raw: &str) -> String {
    ResultIterCopiedMapPayload::new(raw).dead_method()
}
