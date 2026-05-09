use std::collections::VecDeque;

#[derive(Clone)]
pub struct VecdequeSplitOffIntoIterPayload {
    value: String,
}

impl VecdequeSplitOffIntoIterPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vecdeque-split-off-into-iter:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("vecdeque-split-off-into-iter:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vecdeque-split-off-into-iter:{}", self.value)
    }
}

fn vecdeque_split_off_into_iter_payloads(raw: &str) -> VecDeque<VecdequeSplitOffIntoIterPayload> {
    let mut payloads = VecDeque::new();
    payloads.push_back(VecdequeSplitOffIntoIterPayload::new(raw));
    payloads.push_back(VecdequeSplitOffIntoIterPayload::new("tail"));
    payloads
}

pub fn selected_vecdeque_split_off_into_iter(raw: &str) -> String {
    let mut payloads = vecdeque_split_off_into_iter_payloads(raw);
    payloads
        .split_off(1)
        .into_iter()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "vecdeque-split-off-into-iter:missing".to_string())
}

pub fn dead_live_vecdeque_split_off_into_iter(raw: &str) -> String {
    VecdequeSplitOffIntoIterPayload::new(raw).dead_method()
}
