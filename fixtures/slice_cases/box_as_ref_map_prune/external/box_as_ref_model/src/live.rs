pub struct BoxAsRefMapPayload {
    value: String,
}

impl BoxAsRefMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("box-as-ref-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("box-as-ref-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-box-as-ref-map:{}", self.value)
    }
}

pub fn selected_box_as_ref_map(raw: &str) -> String {
    let payload = Box::new(BoxAsRefMapPayload::new(raw));
    payload.as_ref().render_label()
}

pub fn dead_live_box_as_ref_map(raw: &str) -> String {
    BoxAsRefMapPayload::new(raw).dead_method()
}
