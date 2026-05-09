pub struct BoolThenPayload {
    value: String,
}

impl BoolThenPayload {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.trim().to_string() }
    }

    pub fn render_label(&self) -> String {
        format!("bool-then:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-bool-then:{}", self.value)
    }
}

pub fn selected_bool_then(raw: &str) -> String {
    (!raw.trim().is_empty())
        .then(|| BoolThenPayload::new(raw))
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "bool-then:missing".to_string())
}

pub fn dead_live_bool_then(raw: &str) -> String {
    BoolThenPayload::new(raw).dead_method()
}
