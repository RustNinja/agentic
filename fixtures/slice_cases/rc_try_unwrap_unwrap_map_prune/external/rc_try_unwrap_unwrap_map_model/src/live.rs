use std::rc::Rc;
#[derive(Debug)]
pub struct RcTryUnwrapUnwrapMapPayload {
    value: String,
}

impl RcTryUnwrapUnwrapMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("rc-try-unwrap-unwrap-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("rc-try-unwrap-unwrap-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-rc-try-unwrap-unwrap-map:{}", self.value)
    }
}

pub fn selected_rc_try_unwrap_unwrap_map(raw: &str) -> String {
    let payload = Rc::new(RcTryUnwrapUnwrapMapPayload::new(raw));
    Rc::try_unwrap(payload).unwrap().render_label()
}

pub fn dead_live_rc_try_unwrap_unwrap_map(raw: &str) -> String {
    RcTryUnwrapUnwrapMapPayload::new(raw).unused_label()
}
