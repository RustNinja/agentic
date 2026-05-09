#[derive(Clone)]
pub struct OptionAsMutMapPayload {
    value: String,
}

impl OptionAsMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("option-as-mut-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("option-as-mut-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-as-mut-map:{}", self.value)
    }
}

fn option_as_mut_map_payload(raw: &str) -> Option<OptionAsMutMapPayload> {
    Some(OptionAsMutMapPayload::new(raw))
}

pub fn selected_option_as_mut_map(raw: &str) -> String {
    let mut payload = option_as_mut_map_payload(raw);
    payload
        .as_mut()
        .map(|payload| payload.bump_and_render())
        .unwrap_or_else(|| "option-as-mut-map:missing".to_string())
}

pub fn dead_live_option_as_mut_map(raw: &str) -> String {
    OptionAsMutMapPayload::new(raw).dead_method()
}
