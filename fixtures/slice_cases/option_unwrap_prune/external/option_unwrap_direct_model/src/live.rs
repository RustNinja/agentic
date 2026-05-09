pub struct OptionUnwrapDirectPayload {
    value: String,
}

impl OptionUnwrapDirectPayload {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.trim().to_string() }
    }

    pub fn render_label(&self) -> String {
        format!("option-unwrap-direct:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-unwrap-direct:{}", self.value)
    }
}

fn option_unwrap_direct_payload(raw: &str) -> Option<OptionUnwrapDirectPayload> {
    Some(OptionUnwrapDirectPayload::new(raw))
}

pub fn selected_option_unwrap_direct(raw: &str) -> String {
    option_unwrap_direct_payload(raw).unwrap().render_label()
}

pub fn dead_live_option_unwrap_direct(raw: &str) -> String {
    OptionUnwrapDirectPayload::new(raw).dead_method()
}
