use std::collections::VecDeque;
pub struct VecdequePushFrontBackMapPayload {
    value: String,
}

impl VecdequePushFrontBackMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vecdeque-push-front-back-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vecdeque-push-front-back-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vecdeque-push-front-back-map:{}", self.value)
    }
}

pub fn selected_vecdeque_push_front_back_map(raw: &str) -> String {
    let mut values = VecDeque::new();
    values.push_front(VecdequePushFrontBackMapPayload::new(raw));
    values
        .back()
        .map(|payload| payload.render_label())
        .unwrap_or_default()
}

pub fn dead_live_vecdeque_push_front_back_map(raw: &str) -> String {
    VecdequePushFrontBackMapPayload::new(raw).unused_label()
}
