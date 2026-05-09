pub struct OptionAsDerefMapPayload {
    value: String,
}

impl OptionAsDerefMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("option-as-deref-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("option-as-deref-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-as-deref-map:{}", self.value)
    }
}

fn option_as_deref_map_payload(raw: &str) -> Option<Box<OptionAsDerefMapPayload>> {
    if raw.trim().is_empty() {
        None
    } else {
        Some(Box::new(OptionAsDerefMapPayload::new(raw)))
    }
}

pub fn selected_option_as_deref_map(raw: &str) -> String {
    option_as_deref_map_payload(raw)
        .as_deref()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "option-as-deref-map:missing".to_string())
}

pub fn dead_live_option_as_deref_map(raw: &str) -> String {
    OptionAsDerefMapPayload::new(raw).dead_method()
}
