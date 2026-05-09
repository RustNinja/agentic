use std::collections::VecDeque;

#[derive(Clone)]
pub struct VecdequeAsSlicesIterPayload {
    value: String,
}

impl VecdequeAsSlicesIterPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vecdeque-as-slices-iter:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("vecdeque-as-slices-iter:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vecdeque-as-slices-iter:{}", self.value)
    }
}

pub fn selected_vecdeque_as_slices_iter(raw: &str) -> String {
    let mut payloads = VecDeque::new();
    payloads.push_back(VecdequeAsSlicesIterPayload::new(raw));
    payloads.push_back(VecdequeAsSlicesIterPayload::new("tail"));
    let (front, _back) = payloads.as_slices();
    front
        .iter()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "vecdeque-as-slices-iter:missing".to_string())
}

pub fn dead_live_vecdeque_as_slices_iter(raw: &str) -> String {
    VecdequeAsSlicesIterPayload::new(raw).dead_method()
}
