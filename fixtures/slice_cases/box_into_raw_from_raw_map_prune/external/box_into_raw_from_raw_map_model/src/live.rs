pub struct BoxIntoRawFromRawMapPayload {
    value: String,
}

impl BoxIntoRawFromRawMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("box-into-raw-from-raw-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("box-into-raw-from-raw-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-box-into-raw-from-raw-map:{}", self.value)
    }
}

pub fn selected_box_into_raw_from_raw_map(raw: &str) -> String {
    let ptr = Box::into_raw(Box::new(BoxIntoRawFromRawMapPayload::new(raw)));
    unsafe { Box::from_raw(ptr).render_label() }
}

pub fn dead_live_box_into_raw_from_raw_map(raw: &str) -> String {
    BoxIntoRawFromRawMapPayload::new(raw).unused_label()
}
