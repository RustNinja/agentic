#[derive(Clone)]
pub struct OptionTakeMapPayload {
    value: String,
}

impl OptionTakeMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("option-take-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("option-take-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-take-map:{}", self.value)
    }
}

pub struct OptionTakeMapSlot {
    payload: Option<OptionTakeMapPayload>,
}

impl OptionTakeMapSlot {
    pub fn new(raw: &str) -> Self {
        Self {
            payload: Some(OptionTakeMapPayload::new(raw)),
        }
    }

    pub fn take_render(&mut self) -> String {
        self.payload
            .take()
            .map(|payload| payload.render_label())
            .unwrap_or_else(|| "option-take-map:missing".to_string())
    }

    pub fn dead_slot_method(&self) -> String {
        "dead-option-take-map-slot".to_string()
    }
}
pub fn selected_option_take_map(raw: &str) -> String {
    let mut slot = OptionTakeMapSlot::new(raw);
    slot.take_render()
}

pub fn dead_live_option_take_map(raw: &str) -> String {
    OptionTakeMapPayload::new(raw).dead_method()
}
