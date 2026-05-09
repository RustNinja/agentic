pub struct OptionUnwrapPayload {
    value: String,
}

impl OptionUnwrapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn fallback(raw: &str) -> Self {
        Self {
            value: format!("fallback-{raw}"),
        }
    }

    pub fn render_label(&self) -> String {
        format!("option-unwrap:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-unwrap:{}", self.value)
    }
}

fn option_unwrap_payload(raw: &str) -> Option<OptionUnwrapPayload> {
    (!raw.trim().is_empty()).then(|| OptionUnwrapPayload::new(raw))
}

pub fn selected_option_unwrap(raw: &str) -> String {
    option_unwrap_payload(raw)
        .unwrap_or_else(|| OptionUnwrapPayload::fallback(raw))
        .render_label()
}

pub fn dead_live_option_unwrap(raw: &str) -> String {
    OptionUnwrapPayload::new(raw).dead_method()
}
