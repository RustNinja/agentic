use std::collections::VecDeque;

#[derive(Clone)]
pub struct VecdequeAppendIterPayload {
    value: String,
}

impl VecdequeAppendIterPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vecdeque-append-iter:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("vecdeque-append-iter:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vecdeque-append-iter:{}", self.value)
    }
}

fn vecdeque_append_iter_payloads(raw: &str) -> VecDeque<VecdequeAppendIterPayload> {
    let mut payloads = VecDeque::new();
    payloads.push_back(VecdequeAppendIterPayload::new(raw));
    payloads
}

pub fn selected_vecdeque_append_iter(raw: &str) -> String {
    let mut payloads = vecdeque_append_iter_payloads(raw);
    let mut extras = vecdeque_append_iter_payloads("tail");
    payloads.append(&mut extras);
    payloads
        .iter()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "vecdeque-append-iter:missing".to_string())
}

pub fn dead_live_vecdeque_append_iter(raw: &str) -> String {
    VecdequeAppendIterPayload::new(raw).dead_method()
}
