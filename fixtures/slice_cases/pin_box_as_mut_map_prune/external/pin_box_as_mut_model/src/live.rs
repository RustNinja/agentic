use std::pin::Pin;

pub struct PinBoxAsMutMapPayload {
    value: String,
}

impl PinBoxAsMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("pin-box-as-mut-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("pin-box-as-mut-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-pin-box-as-mut-map:{}", self.value)
    }
}

pub fn selected_pin_box_as_mut_map(raw: &str) -> String {
    let mut payload = Pin::new(Box::new(PinBoxAsMutMapPayload::new(raw)));
    payload.as_mut().get_mut().bump_and_render()
}

pub fn dead_live_pin_box_as_mut_map(raw: &str) -> String {
    PinBoxAsMutMapPayload::new(raw).dead_method()
}
