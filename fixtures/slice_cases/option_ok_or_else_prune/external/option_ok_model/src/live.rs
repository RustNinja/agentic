pub struct OptionOkPayload {
    value: String,
}

impl OptionOkPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("option-ok:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-ok-payload:{}", self.value)
    }
}

pub struct OptionOkError {
    value: String,
}

impl OptionOkError {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_error(&self) -> String {
        format!("option-ok-error:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-ok-error:{}", self.value)
    }
}

fn option_ok_payload(raw: &str) -> Option<OptionOkPayload> {
    if raw.trim().is_empty() {
        None
    } else {
        Some(OptionOkPayload::new(raw))
    }
}

pub fn selected_option_ok(raw: &str) -> String {
    option_ok_payload(raw)
        .ok_or_else(|| OptionOkError::new(raw))
        .map(|payload| payload.render_label())
        .unwrap_or_else(|err| err.render_error())
}

pub fn dead_live_option_ok(raw: &str) -> String {
    OptionOkError::new(raw).dead_method()
}
