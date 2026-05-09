#[derive(Clone)]
pub struct OptionReplaceMapPayload {
    value: String,
}

impl OptionReplaceMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("option-replace-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("option-replace-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-replace-map:{}", self.value)
    }
}

pub fn selected_option_replace_map(raw: &str) -> String {
    let mut slot = Some(OptionReplaceMapPayload::new(raw));
    slot.replace(OptionReplaceMapPayload::new("fresh"))
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "option-replace-map:missing".to_string())
}

pub fn dead_live_option_replace_map(raw: &str) -> String {
    OptionReplaceMapPayload::new(raw).dead_method()
}
