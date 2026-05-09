pub struct OptionMapPayload {
    value: String,
}

impl OptionMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("option-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-map:{}", self.value)
    }
}

fn option_map_payload(raw: &str) -> Option<OptionMapPayload> {
    (!raw.trim().is_empty()).then(|| OptionMapPayload::new(raw))
}

pub fn selected_option_map(raw: &str) -> String {
    option_map_payload(raw)
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "option-map:missing".to_string())
}

pub fn dead_live_option_map(raw: &str) -> String {
    OptionMapPayload::new(raw).dead_method()
}
