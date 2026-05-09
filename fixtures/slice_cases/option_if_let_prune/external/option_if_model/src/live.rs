pub struct OptionIfPayload {
    value: String,
}

impl OptionIfPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("option-if:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-if:{}", self.value)
    }
}

fn option_if_payload(raw: &str) -> Option<OptionIfPayload> {
    if raw.trim().is_empty() {
        None
    } else {
        Some(OptionIfPayload::new(raw))
    }
}

pub fn selected_option_if(raw: &str) -> String {
    if let Some(payload) = option_if_payload(raw) {
        payload.render_label()
    } else {
        "option-if:missing".to_string()
    }
}

pub fn dead_live_option_if(raw: &str) -> String {
    OptionIfPayload::new(raw).dead_method()
}
