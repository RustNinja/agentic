use std::pin::Pin;

pub struct PinBoxAsRefMapPayload {
    value: String,
}

impl PinBoxAsRefMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("pin-box-as-ref-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("pin-box-as-ref-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-pin-box-as-ref-map:{}", self.value)
    }
}

pub fn selected_pin_box_as_ref_map(raw: &str) -> String {
    let payload = Pin::new(Box::new(PinBoxAsRefMapPayload::new(raw)));
    payload.as_ref().get_ref().render_label()
}

pub fn dead_live_pin_box_as_ref_map(raw: &str) -> String {
    PinBoxAsRefMapPayload::new(raw).dead_method()
}
