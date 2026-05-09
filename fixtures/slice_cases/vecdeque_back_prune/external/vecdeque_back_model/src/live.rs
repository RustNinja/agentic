use std::collections::VecDeque;

pub struct VecdequeBackPayload {
    value: String,
}

impl VecdequeBackPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vecdeque-back:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vecdeque-back:{}", self.value)
    }
}

fn vecdeque_back_items(raw: &str) -> VecDeque<VecdequeBackPayload> {
    let mut payloads = VecDeque::new();
    payloads.push_back(VecdequeBackPayload::new(raw));
    payloads.push_back(VecdequeBackPayload::new("tail"));
    payloads
}

pub fn selected_vecdeque_back(raw: &str) -> String {
    vecdeque_back_items(raw)
        .back()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "vecdeque-back:missing".to_string())
}

pub fn dead_live_vecdeque_back(raw: &str) -> String {
    VecdequeBackPayload::new(raw).dead_method()
}
