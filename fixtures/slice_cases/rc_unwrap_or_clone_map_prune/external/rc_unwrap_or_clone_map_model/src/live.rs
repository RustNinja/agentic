use std::rc::Rc;
#[derive(Clone)]
pub struct RcUnwrapOrCloneMapPayload {
    value: String,
}

impl RcUnwrapOrCloneMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("rc-unwrap-or-clone-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("rc-unwrap-or-clone-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-rc-unwrap-or-clone-map:{}", self.value)
    }
}

pub fn selected_rc_unwrap_or_clone_map(raw: &str) -> String {
    let payload = Rc::new(RcUnwrapOrCloneMapPayload::new(raw));
    Rc::unwrap_or_clone(payload).render_label()
}

pub fn dead_live_rc_unwrap_or_clone_map(raw: &str) -> String {
    RcUnwrapOrCloneMapPayload::new(raw).unused_label()
}
