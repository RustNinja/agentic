pub struct OptionFilterPayload {
    value: String,
}

impl OptionFilterPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn accepts(&self) -> bool {
        self.render_label().contains("ready")
    }

    pub fn render_label(&self) -> String {
        format!("option-filter:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-filter:{}", self.value)
    }
}

fn option_filter_payload(raw: &str) -> Option<OptionFilterPayload> {
    if raw.trim().is_empty() {
        None
    } else {
        Some(OptionFilterPayload::new(raw))
    }
}

pub fn selected_option_filter(raw: &str) -> String {
    option_filter_payload(raw)
        .filter(|payload| payload.accepts())
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "option-filter:missing".to_string())
}

pub fn dead_live_option_filter(raw: &str) -> String {
    OptionFilterPayload::new(raw).dead_method()
}
