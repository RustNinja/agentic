#[derive(Clone)]
pub struct OptionGetOrInsertWithMapPayload {
    value: String,
}

impl OptionGetOrInsertWithMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("option-get-or-insert-with-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("option-get-or-insert-with-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-get-or-insert-with-map:{}", self.value)
    }
}

pub struct OptionGetOrInsertWithMapSlot {
    fallback: String,
    payload: Option<OptionGetOrInsertWithMapPayload>,
}

impl OptionGetOrInsertWithMapSlot {
    pub fn new(raw: &str) -> Self {
        Self {
            fallback: raw.trim().to_string(),
            payload: None,
        }
    }

    pub fn ensure_render(&mut self) -> String {
        let fallback = self.fallback.clone();
        self.payload
            .get_or_insert_with(|| OptionGetOrInsertWithMapPayload::new(&fallback))
            .render_label()
    }

    pub fn dead_slot_method(&self) -> String {
        format!("dead-option-get-or-insert-with-map-slot:{}", self.fallback)
    }
}
pub fn selected_option_get_or_insert_with_map(raw: &str) -> String {
    let mut slot = OptionGetOrInsertWithMapSlot::new(raw);
    slot.ensure_render()
}

pub fn dead_live_option_get_or_insert_with_map(raw: &str) -> String {
    OptionGetOrInsertWithMapPayload::new(raw).dead_method()
}
