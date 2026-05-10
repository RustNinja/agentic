use std::rc::Rc;
pub struct RcIntoInnerMapPayload {
    value: String,
}

impl RcIntoInnerMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("rc-into-inner-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("rc-into-inner-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-rc-into-inner-map:{}", self.value)
    }
}

pub fn selected_rc_into_inner_map(raw: &str) -> String {
    let payload = Rc::new(RcIntoInnerMapPayload::new(raw));
    Rc::into_inner(payload)
        .map(|payload| payload.render_label())
        .unwrap_or_default()
}

pub fn dead_live_rc_into_inner_map(raw: &str) -> String {
    RcIntoInnerMapPayload::new(raw).unused_label()
}
