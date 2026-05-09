#[derive(Clone)]
pub struct OptionGetOrInsertMapPayload {
    value: String,
}

impl OptionGetOrInsertMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("option-get-or-insert-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("option-get-or-insert-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-get-or-insert-map:{}", self.value)
    }
}

pub fn selected_option_get_or_insert_map(raw: &str) -> String {
    let mut slot = None;
    slot.get_or_insert(OptionGetOrInsertMapPayload::new(raw))
        .bump_and_render()
}

pub fn dead_live_option_get_or_insert_map(raw: &str) -> String {
    let mut payload = OptionGetOrInsertMapPayload::new(raw);
    payload.bump_and_render()
}
