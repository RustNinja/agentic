use std::pin::Pin;

pub fn selected_option_as_pin_ref_map(raw: &str) -> String {
    let slot = Some(OptionAsPinRefMapPayload::new(raw));
    Pin::new(&slot)
        .as_pin_ref()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "empty".to_string())
}

pub struct OptionAsPinRefMapPayload {
    value: String,
}

impl OptionAsPinRefMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("option_as_pin_ref_map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("option_as_pin_ref_map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-option-as-pin-ref-map:{}", self.value)
    }
}

pub fn dead_live_option_as_pin_ref_map(raw: &str) -> String {
    OptionAsPinRefMapPayload::new(raw).unused_label()
}
