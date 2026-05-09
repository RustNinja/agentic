#[derive(Clone)]
pub struct OptionIterMutMapPayload {
    value: String,
}

impl OptionIterMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("option-iter-mut-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("option-iter-mut-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-iter-mut-map:{}", self.value)
    }
}

fn option_iter_mut_map_payload(raw: &str) -> Option<OptionIterMutMapPayload> {
    Some(OptionIterMutMapPayload::new(raw))
}

pub fn selected_option_iter_mut_map(raw: &str) -> String {
    let mut payload = option_iter_mut_map_payload(raw);
    payload
        .iter_mut()
        .map(|payload| payload.bump_and_render())
        .next()
        .unwrap_or_else(|| "option-iter-mut-map:missing".to_string())
}

pub fn dead_live_option_iter_mut_map(raw: &str) -> String {
    OptionIterMutMapPayload::new(raw).dead_method()
}
