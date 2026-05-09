pub struct NthPayload {
    value: String,
}

impl NthPayload {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.trim().to_string() }
    }

    pub fn render_label(&self) -> String {
        format!("nth:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-nth:{}", self.value)
    }
}

fn nth_items(raw: &str) -> Vec<NthPayload> {
    vec![NthPayload::new("head"), NthPayload::new(raw)]
}

pub fn selected_nth(raw: &str) -> String {
    nth_items(raw)
        .into_iter()
        .nth(1)
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "nth:missing".to_string())
}

pub fn dead_live_nth(raw: &str) -> String {
    NthPayload::new(raw).dead_method()
}
