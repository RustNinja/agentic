use std::collections::VecDeque;

#[derive(Clone)]
pub struct VecdequeAsMutSlicesIterPayload {
    value: String,
}

impl VecdequeAsMutSlicesIterPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vecdeque-as-mut-slices-iter:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("vecdeque-as-mut-slices-iter:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vecdeque-as-mut-slices-iter:{}", self.value)
    }
}

pub fn selected_vecdeque_as_mut_slices_iter(raw: &str) -> String {
    let mut payloads = VecDeque::new();
    payloads.push_back(VecdequeAsMutSlicesIterPayload::new(raw));
    payloads.push_back(VecdequeAsMutSlicesIterPayload::new("tail"));
    let (front, _back) = payloads.as_mut_slices();
    front
        .iter_mut()
        .map(|payload| payload.bump_and_render())
        .next()
        .unwrap_or_else(|| "vecdeque-as-mut-slices-iter:missing".to_string())
}

pub fn dead_live_vecdeque_as_mut_slices_iter(raw: &str) -> String {
    VecdequeAsMutSlicesIterPayload::new(raw).dead_method()
}
