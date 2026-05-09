use std::collections::VecDeque;

pub struct VecdequeFrontPayload {
    value: String,
}

impl VecdequeFrontPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vecdeque-front:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vecdeque-front:{}", self.value)
    }
}

fn vecdeque_front_items(raw: &str) -> VecDeque<VecdequeFrontPayload> {
    let mut payloads = VecDeque::new();
    payloads.push_back(VecdequeFrontPayload::new(raw));
    payloads.push_back(VecdequeFrontPayload::new("tail"));
    payloads
}

pub fn selected_vecdeque_front(raw: &str) -> String {
    vecdeque_front_items(raw)
        .front()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "vecdeque-front:missing".to_string())
}

pub fn dead_live_vecdeque_front(raw: &str) -> String {
    VecdequeFrontPayload::new(raw).dead_method()
}
