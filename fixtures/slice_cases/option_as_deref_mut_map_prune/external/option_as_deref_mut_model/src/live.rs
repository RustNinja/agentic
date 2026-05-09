pub struct OptionAsDerefMutMapPayload {
    value: String,
}

impl OptionAsDerefMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("option-as-deref-mut-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("option-as-deref-mut-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-as-deref-mut-map:{}", self.value)
    }
}

fn option_as_deref_mut_map_payload(raw: &str) -> Option<Box<OptionAsDerefMutMapPayload>> {
    if raw.trim().is_empty() {
        None
    } else {
        Some(Box::new(OptionAsDerefMutMapPayload::new(raw)))
    }
}

pub fn selected_option_as_deref_mut_map(raw: &str) -> String {
    let mut payload = option_as_deref_mut_map_payload(raw);
    payload
        .as_deref_mut()
        .map(|payload| payload.bump_and_render())
        .unwrap_or_else(|| "option-as-deref-mut-map:missing".to_string())
}

pub fn dead_live_option_as_deref_mut_map(raw: &str) -> String {
    OptionAsDerefMutMapPayload::new(raw).dead_method()
}
