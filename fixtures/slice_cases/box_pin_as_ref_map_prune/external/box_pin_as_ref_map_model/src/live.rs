use std::pin::Pin;

#[derive(Clone)]
pub struct BoxPinAsRefMapPayload {
    value: String,
}

impl BoxPinAsRefMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("box-pin-as-ref-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("box-pin-as-ref-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-box-pin-as-ref-map:{}", self.value)
    }
}

pub fn selected_box_pin_as_ref_map(raw: &str) -> String {
    let pinned: Pin<Box<BoxPinAsRefMapPayload>> = Box::pin(BoxPinAsRefMapPayload::new(raw));
    pinned.as_ref().get_ref().render_label()
}

pub fn dead_live_box_pin_as_ref_map(raw: &str) -> String {
    BoxPinAsRefMapPayload::new(raw).dead_method()
}
