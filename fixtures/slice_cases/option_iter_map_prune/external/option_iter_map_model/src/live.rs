#[derive(Clone)]
pub struct OptionIterMapPayload {
    value: String,
}

impl OptionIterMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("option-iter-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("option-iter-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-iter-map:{}", self.value)
    }
}

fn option_iter_map_payload(raw: &str) -> Option<OptionIterMapPayload> {
    Some(OptionIterMapPayload::new(raw))
}

pub fn selected_option_iter_map(raw: &str) -> String {
    option_iter_map_payload(raw)
        .iter()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "option-iter-map:missing".to_string())
}

pub fn dead_live_option_iter_map(raw: &str) -> String {
    OptionIterMapPayload::new(raw).dead_method()
}
