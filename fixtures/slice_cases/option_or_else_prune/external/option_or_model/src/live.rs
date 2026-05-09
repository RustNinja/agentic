pub struct OptionOrPayload {
    value: String,
}

impl OptionOrPayload {
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
        format!("option-or:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-or:{}", self.value)
    }
}

fn option_or_payload(raw: &str) -> Option<OptionOrPayload> {
    (!raw.trim().is_empty()).then(|| OptionOrPayload::new(raw))
}

pub fn selected_option_or(raw: &str) -> String {
    option_or_payload(raw)
        .or_else(|| Some(OptionOrPayload::fallback(raw)))
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "option-or:missing".to_string())
}

pub fn dead_live_option_or(raw: &str) -> String {
    OptionOrPayload::new(raw).dead_method()
}
