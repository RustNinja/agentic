pub struct LastPayload {
    value: String,
}

impl LastPayload {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.trim().to_string() }
    }

    pub fn render_label(&self) -> String {
        format!("last:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-last:{}", self.value)
    }
}

fn last_items(raw: &str) -> Vec<LastPayload> {
    vec![LastPayload::new(raw), LastPayload::new("tail")]
}

pub fn selected_last(raw: &str) -> String {
    last_items(raw)
        .into_iter()
        .last()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "last:missing".to_string())
}

pub fn dead_live_last(raw: &str) -> String {
    LastPayload::new(raw).dead_method()
}
