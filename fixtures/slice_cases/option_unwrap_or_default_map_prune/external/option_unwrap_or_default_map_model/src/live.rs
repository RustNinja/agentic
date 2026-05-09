#[derive(Clone)]
pub struct OptionUnwrapOrDefaultMapPayload {
    value: String,
}

impl OptionUnwrapOrDefaultMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("option-unwrap-or-default-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("option-unwrap-or-default-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-unwrap-or-default-map:{}", self.value)
    }
}

impl Default for OptionUnwrapOrDefaultMapPayload {
    fn default() -> Self {
        Self::new("default")
    }
}

pub fn selected_option_unwrap_or_default_map(raw: &str) -> String {
    let slot = if raw.trim().is_empty() {
        None
    } else {
        Some(OptionUnwrapOrDefaultMapPayload::new(raw))
    };
    slot.unwrap_or_default().render_label()
}

pub fn dead_live_option_unwrap_or_default_map(raw: &str) -> String {
    let mut payload = OptionUnwrapOrDefaultMapPayload::new(raw);
    payload.bump_and_render()
}
