#[derive(Clone)]
pub struct OptionInsertMapPayload {
    value: String,
}

impl OptionInsertMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("option-insert-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("option-insert-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-insert-map:{}", self.value)
    }
}

pub struct OptionInsertMapSlot {
    fallback: String,
    payload: Option<OptionInsertMapPayload>,
}

impl OptionInsertMapSlot {
    pub fn new(raw: &str) -> Self {
        Self {
            fallback: raw.trim().to_string(),
            payload: None,
        }
    }

    pub fn insert_render(&mut self) -> String {
        self.payload
            .insert(OptionInsertMapPayload::new(&self.fallback))
            .render_label()
    }

    pub fn dead_slot_method(&self) -> String {
        format!("dead-option-insert-map-slot:{}", self.fallback)
    }
}
pub fn selected_option_insert_map(raw: &str) -> String {
    let mut slot = OptionInsertMapSlot::new(raw);
    slot.insert_render()
}

pub fn dead_live_option_insert_map(raw: &str) -> String {
    OptionInsertMapPayload::new(raw).dead_method()
}
