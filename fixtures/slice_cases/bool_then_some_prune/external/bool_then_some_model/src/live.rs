pub struct BoolThenSomePayload {
    value: String,
}

impl BoolThenSomePayload {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.trim().to_string() }
    }

    pub fn render_label(&self) -> String {
        format!("bool-then-some:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-bool-then-some:{}", self.value)
    }
}

pub fn selected_bool_then_some(raw: &str) -> String {
    (!raw.trim().is_empty())
        .then_some(BoolThenSomePayload::new(raw))
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "bool-then-some:missing".to_string())
}

pub fn dead_live_bool_then_some(raw: &str) -> String {
    BoolThenSomePayload::new(raw).dead_method()
}
