pub struct OptionCheckPayload {
    value: String,
}

impl OptionCheckPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn accepts(&self) -> bool {
        self.render_label().contains("ready")
    }

    pub fn render_label(&self) -> String {
        format!("option-check:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-check:{}", self.value)
    }
}

fn option_check_payload(raw: &str) -> Option<OptionCheckPayload> {
    if raw.trim().is_empty() {
        None
    } else {
        Some(OptionCheckPayload::new(raw))
    }
}

pub fn selected_option_check(raw: &str) -> String {
    if option_check_payload(raw).is_some_and(|payload| payload.accepts()) {
        "option-check:accepted".to_string()
    } else {
        "option-check:rejected".to_string()
    }
}

pub fn dead_live_option_check(raw: &str) -> String {
    OptionCheckPayload::new(raw).dead_method()
}
