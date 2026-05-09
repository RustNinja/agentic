pub struct OptionRefPayload {
    value: String,
}

impl OptionRefPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("option-ref:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-ref:{}", self.value)
    }
}

fn option_ref_payload(raw: &str) -> Option<OptionRefPayload> {
    if raw.trim().is_empty() {
        None
    } else {
        Some(OptionRefPayload::new(raw))
    }
}

pub fn selected_option_ref(raw: &str) -> String {
    let payload = option_ref_payload(raw);
    payload
        .as_ref()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "option-ref:missing".to_string())
}

pub fn dead_live_option_ref(raw: &str) -> String {
    OptionRefPayload::new(raw).dead_method()
}
