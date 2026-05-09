#[derive(Clone)]
pub struct OptionTakeIfMapPayload {
    value: String,
}

impl OptionTakeIfMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("option-take-if-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("option-take-if-map:{}", self.value)
    }

    pub fn allow(&self) -> bool {
        !self.value.is_empty()
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-take-if-map:{}", self.value)
    }
}

pub fn selected_option_take_if_map(raw: &str) -> String {
    let mut payload = Some(OptionTakeIfMapPayload::new(raw));
    payload
        .take_if(|payload| payload.allow())
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "option-take-if-map:missing".to_string())
}

pub fn dead_live_option_take_if_map(raw: &str) -> String {
    OptionTakeIfMapPayload::new(raw).dead_method()
}
