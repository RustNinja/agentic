use std::collections::VecDeque;
pub struct VecdequeTruncateFrontMapPayload {
    value: String,
}

impl VecdequeTruncateFrontMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vecdeque-truncate-front-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vecdeque-truncate-front-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vecdeque-truncate-front-map:{}", self.value)
    }
}

pub fn selected_vecdeque_truncate_front_map(raw: &str) -> String {
    let mut values = VecDeque::from([
        VecdequeTruncateFrontMapPayload::new(raw),
        VecdequeTruncateFrontMapPayload::new("tail"),
    ]);
    values.truncate(1);
    values
        .front()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_vecdeque_truncate_front_map(raw: &str) -> String {
    VecdequeTruncateFrontMapPayload::new(raw).unused_label()
}
