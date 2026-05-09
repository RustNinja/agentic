pub struct RevLastPayload {
    value: String,
}

impl RevLastPayload {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.trim().to_string() }
    }

    pub fn render_label(&self) -> String {
        format!("rev-last:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-rev-last:{}", self.value)
    }
}

fn rev_last_items(raw: &str) -> Vec<RevLastPayload> {
    vec![RevLastPayload::new(raw), RevLastPayload::new("tail")]
}

pub fn selected_rev_last(raw: &str) -> String {
    rev_last_items(raw)
        .into_iter()
        .rev()
        .last()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "rev-last:missing".to_string())
}

pub fn dead_live_rev_last(raw: &str) -> String {
    RevLastPayload::new(raw).dead_method()
}
