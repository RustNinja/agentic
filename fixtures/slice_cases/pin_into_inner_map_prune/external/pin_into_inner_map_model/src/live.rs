use std::pin::Pin;
pub struct PinIntoInnerMapPayload {
    value: String,
}

impl PinIntoInnerMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("pin-into-inner-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("pin-into-inner-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-pin-into-inner-map:{}", self.value)
    }
}

pub fn selected_pin_into_inner_map(raw: &str) -> String {
    let pinned = Box::pin(PinIntoInnerMapPayload::new(raw));
    Pin::into_inner(pinned).render_label()
}

pub fn dead_live_pin_into_inner_map(raw: &str) -> String {
    PinIntoInnerMapPayload::new(raw).unused_label()
}
