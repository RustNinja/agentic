pub struct OptionXorPayload {
    value: String,
}

impl OptionXorPayload {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.trim().to_string() }
    }

    pub fn render_label(&self) -> String {
        format!("option-xor:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-xor:{}", self.value)
    }
}

fn option_xor_left(raw: &str) -> Option<OptionXorPayload> {
    (!raw.trim().is_empty()).then(|| OptionXorPayload::new(raw))
}

fn option_xor_right(raw: &str) -> Option<OptionXorPayload> {
    raw.trim().is_empty().then(|| OptionXorPayload::new("fallback"))
}

pub fn selected_option_xor(raw: &str) -> String {
    option_xor_left(raw)
        .xor(option_xor_right(raw))
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "option-xor:missing".to_string())
}

pub fn dead_live_option_xor(raw: &str) -> String {
    OptionXorPayload::new(raw).dead_method()
}
