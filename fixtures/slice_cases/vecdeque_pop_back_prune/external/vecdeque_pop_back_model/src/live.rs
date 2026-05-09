use std::collections::VecDeque;

pub struct VecdequePopBackPayload {
    value: String,
}

impl VecdequePopBackPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vecdeque-pop-back:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vecdeque-pop-back:{}", self.value)
    }
}

fn vecdeque_pop_back_items(raw: &str) -> VecDeque<VecdequePopBackPayload> {
    let mut payloads = VecDeque::new();
    payloads.push_back(VecdequePopBackPayload::new(raw));
    payloads.push_back(VecdequePopBackPayload::new("tail"));
    payloads
}

pub fn selected_vecdeque_pop_back(raw: &str) -> String {
    let mut payloads = vecdeque_pop_back_items(raw);
    payloads
        .pop_back()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "vecdeque-pop-back:missing".to_string())
}

pub fn dead_live_vecdeque_pop_back(raw: &str) -> String {
    VecdequePopBackPayload::new(raw).dead_method()
}
