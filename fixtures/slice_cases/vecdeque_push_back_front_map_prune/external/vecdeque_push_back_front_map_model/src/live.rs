use std::collections::VecDeque;
pub struct VecdequePushBackFrontMapPayload {
    value: String,
}

impl VecdequePushBackFrontMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vecdeque-push-back-front-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vecdeque-push-back-front-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vecdeque-push-back-front-map:{}", self.value)
    }
}

pub fn selected_vecdeque_push_back_front_map(raw: &str) -> String {
    let mut values = VecDeque::new();
    values.push_back(VecdequePushBackFrontMapPayload::new(raw));
    values
        .front()
        .map(|payload| payload.render_label())
        .unwrap_or_default()
}

pub fn dead_live_vecdeque_push_back_front_map(raw: &str) -> String {
    VecdequePushBackFrontMapPayload::new(raw).unused_label()
}
