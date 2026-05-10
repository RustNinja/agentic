use std::rc::Rc;
pub struct RcTryUnwrapOkMapPayload {
    value: String,
}

impl RcTryUnwrapOkMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("rc-try-unwrap-ok-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("rc-try-unwrap-ok-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-rc-try-unwrap-ok-map:{}", self.value)
    }
}

pub fn selected_rc_try_unwrap_ok_map(raw: &str) -> String {
    let payload = Rc::new(RcTryUnwrapOkMapPayload::new(raw));
    Rc::try_unwrap(payload)
        .ok()
        .map(|payload| payload.render_label())
        .unwrap_or_default()
}

pub fn dead_live_rc_try_unwrap_ok_map(raw: &str) -> String {
    RcTryUnwrapOkMapPayload::new(raw).unused_label()
}
