use std::pin::Pin;

pub fn selected_option_as_pin_mut_map(raw: &str) -> String {
    let mut slot = Some(OptionAsPinMutMapPayload::new(raw));
    Pin::new(&mut slot)
        .as_pin_mut()
        .map(|mut payload| payload.bump_and_render())
        .unwrap_or_else(|| "empty".to_string())
}

pub struct OptionAsPinMutMapPayload {
    value: String,
}

impl OptionAsPinMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("option_as_pin_mut_map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("option_as_pin_mut_map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-option-as-pin-mut-map:{}", self.value)
    }
}

pub fn dead_live_option_as_pin_mut_map(raw: &str) -> String {
    OptionAsPinMutMapPayload::new(raw).unused_label()
}
