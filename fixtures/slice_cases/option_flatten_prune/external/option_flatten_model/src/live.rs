pub struct OptionFlattenPayload {
    value: String,
}

impl OptionFlattenPayload {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.trim().to_string() }
    }

    pub fn render_label(&self) -> String {
        format!("option-flatten:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-flatten:{}", self.value)
    }
}

fn nested_option_flatten_payload(raw: &str) -> Option<Option<OptionFlattenPayload>> {
    Some((!raw.trim().is_empty()).then(|| OptionFlattenPayload::new(raw)))
}

pub fn selected_option_flatten(raw: &str) -> String {
    nested_option_flatten_payload(raw)
        .flatten()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "option-flatten:missing".to_string())
}

pub fn dead_live_option_flatten(raw: &str) -> String {
    OptionFlattenPayload::new(raw).dead_method()
}
