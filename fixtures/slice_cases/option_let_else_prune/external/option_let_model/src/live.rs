pub struct OptionLetPayload {
    value: String,
}

impl OptionLetPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("option-let:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-let:{}", self.value)
    }
}

fn option_let_payload(raw: &str) -> Option<OptionLetPayload> {
    if raw.trim().is_empty() {
        None
    } else {
        Some(OptionLetPayload::new(raw))
    }
}

pub fn selected_option_let(raw: &str) -> String {
    let Some(payload) = option_let_payload(raw) else {
        return "option-let:missing".to_string();
    };
    payload.render_label()
}

pub fn dead_live_option_let(raw: &str) -> String {
    OptionLetPayload::new(raw).dead_method()
}
