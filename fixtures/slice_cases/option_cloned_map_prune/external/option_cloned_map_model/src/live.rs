#[derive(Clone)]
pub struct OptionClonedMapPayload {
    value: String,
}

impl OptionClonedMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("option-cloned-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("option-cloned-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-cloned-map:{}", self.value)
    }
}

pub fn selected_option_cloned_map(raw: &str) -> String {
    let payload = OptionClonedMapPayload::new(raw);
    Some(&payload)
        .cloned()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| format!("option-cloned-map:missing"))
}

pub fn dead_live_option_cloned_map(raw: &str) -> String {
    let mut payload = OptionClonedMapPayload::new(raw);
    payload.bump_and_render()
}
