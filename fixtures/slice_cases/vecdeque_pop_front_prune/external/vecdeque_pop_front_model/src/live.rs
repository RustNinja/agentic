use std::collections::VecDeque;

pub struct VecdequePopFrontPayload {
    value: String,
}

impl VecdequePopFrontPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vecdeque-pop-front:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vecdeque-pop-front:{}", self.value)
    }
}

fn vecdeque_pop_front_items(raw: &str) -> VecDeque<VecdequePopFrontPayload> {
    let mut payloads = VecDeque::new();
    payloads.push_back(VecdequePopFrontPayload::new(raw));
    payloads.push_back(VecdequePopFrontPayload::new("tail"));
    payloads
}

pub fn selected_vecdeque_pop_front(raw: &str) -> String {
    let mut payloads = vecdeque_pop_front_items(raw);
    payloads
        .pop_front()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "vecdeque-pop-front:missing".to_string())
}

pub fn dead_live_vecdeque_pop_front(raw: &str) -> String {
    VecdequePopFrontPayload::new(raw).dead_method()
}
