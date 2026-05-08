pub struct OptionInspectPayload {
    value: String,
}

impl OptionInspectPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn audit(&self) -> String {
        format!("audit:{}", self.render_label())
    }

    pub fn render_label(&self) -> String {
        format!("option-inspect:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-inspect:{}", self.value)
    }
}

fn option_inspect_payload(raw: &str) -> Option<OptionInspectPayload> {
    if raw.trim().is_empty() {
        None
    } else {
        Some(OptionInspectPayload::new(raw))
    }
}

pub fn selected_option_inspect(raw: &str) -> String {
    let mut audit = Vec::new();
    let rendered = option_inspect_payload(raw)
        .inspect(|payload| audit.push(payload.audit()))
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "option-inspect:missing".to_string());
    format!("{}:{}", audit.len(), rendered)
}

pub fn dead_live_option_inspect(raw: &str) -> String {
    OptionInspectPayload::new(raw).dead_method()
}
