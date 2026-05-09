use std::rc::Rc;

#[derive(Clone)]
pub struct RcMakeMutMapPayload {
    value: String,
}

impl RcMakeMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("rc-make-mut-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("rc-make-mut-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-rc-make-mut-map:{}", self.value)
    }
}

pub fn selected_rc_make_mut_map(raw: &str) -> String {
    let mut payload = Rc::new(RcMakeMutMapPayload::new(raw));
    Rc::make_mut(&mut payload).bump_and_render()
}

pub fn dead_live_rc_make_mut_map(raw: &str) -> String {
    RcMakeMutMapPayload::new(raw).dead_method()
}
