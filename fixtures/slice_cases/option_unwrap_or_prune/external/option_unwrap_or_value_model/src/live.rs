pub struct OptionUnwrapOrValuePayload {
    value: String,
}

impl OptionUnwrapOrValuePayload {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.trim().to_string() }
    }

    pub fn fallback(raw: &str) -> Self {
        Self { value: format!("fallback-{raw}") }
    }

    pub fn render_label(&self) -> String {
        format!("option-unwrap-or-value:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-unwrap-or-value:{}", self.value)
    }
}

fn option_unwrap_or_value_payload(raw: &str) -> Option<OptionUnwrapOrValuePayload> {
    (!raw.trim().is_empty()).then(|| OptionUnwrapOrValuePayload::new(raw))
}

pub fn selected_option_unwrap_or_value(raw: &str) -> String {
    option_unwrap_or_value_payload(raw)
        .unwrap_or(OptionUnwrapOrValuePayload::fallback(raw))
        .render_label()
}

pub fn dead_live_option_unwrap_or_value(raw: &str) -> String {
    OptionUnwrapOrValuePayload::new(raw).dead_method()
}
