#[derive(Clone)]
pub struct OptionIterClonedMapPayload {
    value: String,
}

impl OptionIterClonedMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("option-iter-cloned-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("option-iter-cloned-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-iter-cloned-map:{}", self.value)
    }
}

pub fn selected_option_iter_cloned_map(raw: &str) -> String {
    let option = Some(OptionIterClonedMapPayload::new(raw));
    option
        .iter()
        .cloned()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| format!("option-iter-cloned-map:missing"))
}

pub fn dead_live_option_iter_cloned_map(raw: &str) -> String {
    let mut payload = OptionIterClonedMapPayload::new(raw);
    payload.bump_and_render()
}
