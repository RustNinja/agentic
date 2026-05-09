pub struct ResultOkCheckPayload {
    value: String,
}

impl ResultOkCheckPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn accepts(&self) -> bool {
        !self.value.is_empty()
    }

    pub fn render_label(&self) -> String {
        format!("result-ok-check:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-ok-check:{}", self.value)
    }
}

fn result_ok_check_payload(raw: &str) -> Result<ResultOkCheckPayload, String> {
    Ok(ResultOkCheckPayload::new(raw))
}

pub fn selected_result_ok_check(raw: &str) -> String {
    if result_ok_check_payload(raw).is_ok_and(|payload| payload.accepts()) {
        ResultOkCheckPayload::new(raw).render_label()
    } else {
        "result-ok-check:missing".to_string()
    }
}

pub fn dead_live_result_ok_check(raw: &str) -> String {
    ResultOkCheckPayload::new(raw).dead_method()
}
