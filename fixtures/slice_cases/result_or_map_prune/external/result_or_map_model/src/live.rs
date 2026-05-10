pub struct ResultOrMapPayload {
    value: String,
}

impl ResultOrMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("result-or-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("result-or-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-result-or-map:{}", self.value)
    }
}

pub fn selected_result_or_map(raw: &str) -> String {
    let value: Result<ResultOrMapPayload, ResultOrMapPayload> = Ok(ResultOrMapPayload::new(raw));
    value
        .or(Ok::<ResultOrMapPayload, ResultOrMapPayload>(
            ResultOrMapPayload::new("fallback"),
        ))
        .map(|payload| payload.render_label())
        .unwrap_or_default()
}

pub fn dead_live_result_or_map(raw: &str) -> String {
    ResultOrMapPayload::new(raw).unused_label()
}
