use std::rc::Rc;

pub struct RcAsRefMapPayload {
    value: String,
}

impl RcAsRefMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("rc-as-ref-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("rc-as-ref-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-rc-as-ref-map:{}", self.value)
    }
}

pub fn selected_rc_as_ref_map(raw: &str) -> String {
    let payload = Rc::new(RcAsRefMapPayload::new(raw));
    payload.as_ref().render_label()
}

pub fn dead_live_rc_as_ref_map(raw: &str) -> String {
    RcAsRefMapPayload::new(raw).dead_method()
}
