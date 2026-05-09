#[derive(Clone)]
pub struct OptionGetOrInsertDefaultMapPayload {
    value: String,
}

impl OptionGetOrInsertDefaultMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("option-get-or-insert-default-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("option-get-or-insert-default-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-get-or-insert-default-map:{}", self.value)
    }
}

impl Default for OptionGetOrInsertDefaultMapPayload {
    fn default() -> Self {
        Self::new("default")
    }
}

pub fn selected_option_get_or_insert_default_map(raw: &str) -> String {
    let _ = raw;
    let mut slot: Option<OptionGetOrInsertDefaultMapPayload> = None;
    slot.get_or_insert_default().bump_and_render()
}

pub fn dead_live_option_get_or_insert_default_map(raw: &str) -> String {
    let mut payload = OptionGetOrInsertDefaultMapPayload::new(raw);
    payload.bump_and_render()
}
