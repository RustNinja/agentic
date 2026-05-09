pub struct RPositionPayload {
    value: String,
}

impl RPositionPayload {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.trim().to_string() }
    }

    pub fn accepts(&self) -> bool {
        !self.value.is_empty()
    }

    pub fn render_label(&self) -> String {
        format!("rposition:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-rposition:{}", self.value)
    }
}

fn rposition_items(raw: &str) -> Vec<RPositionPayload> {
    vec![RPositionPayload::new(raw)]
}

pub fn selected_rposition(raw: &str) -> String {
    rposition_items(raw)
        .iter()
        .rposition(|payload| payload.accepts())
        .map(|index| RPositionPayload::new(&index.to_string()).render_label())
        .unwrap_or_else(|| "rposition:missing".to_string())
}

pub fn dead_live_rposition(raw: &str) -> String {
    RPositionPayload::new(raw).dead_method()
}
