use std::rc::Rc;
pub struct RcGetMutMapPayload {
    value: String,
}

impl RcGetMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("rc-get-mut-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("rc-get-mut-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-rc-get-mut-map:{}", self.value)
    }
}

pub fn selected_rc_get_mut_map(raw: &str) -> String {
    let mut payload = Rc::new(RcGetMutMapPayload::new(raw));
    Rc::get_mut(&mut payload)
        .map(|payload| payload.bump_and_render())
        .unwrap_or_default()
}

pub fn dead_live_rc_get_mut_map(raw: &str) -> String {
    RcGetMutMapPayload::new(raw).unused_label()
}
