pub struct OptionIntoIterNextMapPayload {
    value: String,
}

impl OptionIntoIterNextMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("option-into-iter-next-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("option-into-iter-next-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-option-into-iter-next-map:{}", self.value)
    }
}

pub fn selected_option_into_iter_next_map(raw: &str) -> String {
    Some(OptionIntoIterNextMapPayload::new(raw))
        .into_iter()
        .next()
        .map(|payload| payload.render_label())
        .unwrap_or_default()
}

pub fn dead_live_option_into_iter_next_map(raw: &str) -> String {
    OptionIntoIterNextMapPayload::new(raw).unused_label()
}
