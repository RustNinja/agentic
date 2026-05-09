#[derive(Clone)]
pub struct ResultTransposeUnwrapMapPayload {
    value: String,
}

impl ResultTransposeUnwrapMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("result-transpose-unwrap-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("result-transpose-unwrap-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-transpose-unwrap-map:{}", self.value)
    }
}

pub struct ResultTransposeUnwrapMapError {
    value: String,
}

impl ResultTransposeUnwrapMapError {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_error(&self) -> String {
        format!("result-transpose-unwrap-map-error:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-transpose-unwrap-map-error:{}", self.value)
    }
}

pub fn selected_result_transpose_unwrap_map(raw: &str) -> String {
    let result: Result<Option<ResultTransposeUnwrapMapPayload>, ResultTransposeUnwrapMapError> =
        Ok(Some(ResultTransposeUnwrapMapPayload::new(raw)));
    result
        .transpose()
        .unwrap_or_else(|| Ok(ResultTransposeUnwrapMapPayload::new("fallback")))
        .map(|payload| payload.render_label())
        .unwrap_or_else(|err| err.render_error())
}

pub fn dead_live_result_transpose_unwrap_map(raw: &str) -> String {
    let mut payload = ResultTransposeUnwrapMapPayload::new(raw);
    payload.bump_and_render()
}
