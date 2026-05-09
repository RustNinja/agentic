pub struct ResultUnwrapDirectPayload {
    value: String,
}

impl ResultUnwrapDirectPayload {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.trim().to_string() }
    }

    pub fn render_label(&self) -> String {
        format!("result-unwrap-direct:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-unwrap-direct:{}", self.value)
    }
}

#[derive(Debug)]
pub struct ResultUnwrapDirectError {
    value: String,
}

impl ResultUnwrapDirectError {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.to_string() }
    }
}

fn result_unwrap_direct_payload(raw: &str) -> Result<ResultUnwrapDirectPayload, ResultUnwrapDirectError> {
    Ok(ResultUnwrapDirectPayload::new(raw))
}

pub fn selected_result_unwrap_direct(raw: &str) -> String {
    result_unwrap_direct_payload(raw).unwrap().render_label()
}

pub fn dead_live_result_unwrap_direct(raw: &str) -> String {
    ResultUnwrapDirectPayload::new(raw).dead_method()
}
