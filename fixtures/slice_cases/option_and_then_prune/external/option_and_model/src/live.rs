pub struct OptionAndPayload {
    value: String,
}

impl OptionAndPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn expand(&self) -> Option<String> {
        Some(format!("option-and:{}", self.render_label()))
    }

    pub fn render_label(&self) -> String {
        format!("label:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-and:{}", self.value)
    }
}

fn option_and_payload(raw: &str) -> Option<OptionAndPayload> {
    if raw.trim().is_empty() {
        None
    } else {
        Some(OptionAndPayload::new(raw))
    }
}

pub fn selected_option_and_then(raw: &str) -> String {
    option_and_payload(raw)
        .and_then(|payload| payload.expand())
        .unwrap_or_else(|| "option-and:missing".to_string())
}

pub fn dead_live_option_and_then(raw: &str) -> String {
    OptionAndPayload::new(raw).dead_method()
}
