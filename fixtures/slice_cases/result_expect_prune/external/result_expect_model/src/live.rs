pub struct ResultExpectPayload {
    value: String,
}

impl ResultExpectPayload {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.trim().to_string() }
    }

    pub fn render_label(&self) -> String {
        format!("result-expect:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-expect:{}", self.value)
    }
}

#[derive(Debug)]
pub struct ResultExpectError {
    value: String,
}

impl ResultExpectError {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.to_string() }
    }
}

fn result_expect_payload(raw: &str) -> Result<ResultExpectPayload, ResultExpectError> {
    if raw.trim().is_empty() {
        Err(ResultExpectError::new(raw))
    } else {
        Ok(ResultExpectPayload::new(raw))
    }
}

pub fn selected_result_expect(raw: &str) -> String {
    result_expect_payload(raw)
        .expect("fixture payload should exist")
        .render_label()
}

pub fn dead_live_result_expect(raw: &str) -> String {
    ResultExpectPayload::new(raw).dead_method()
}
