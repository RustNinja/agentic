pub struct BoxLeakMutMapPayload {
    value: String,
}

impl BoxLeakMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("box-leak-mut-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("box-leak-mut-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-box-leak-mut-map:{}", self.value)
    }
}

pub fn selected_box_leak_mut_map(raw: &str) -> String {
    let payload = Box::leak(Box::new(BoxLeakMutMapPayload::new(raw)));
    payload.bump_and_render()
}

pub fn dead_live_box_leak_mut_map(raw: &str) -> String {
    BoxLeakMutMapPayload::new(raw).unused_label()
}
