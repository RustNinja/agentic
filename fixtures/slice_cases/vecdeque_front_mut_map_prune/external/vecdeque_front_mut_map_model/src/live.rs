use std::collections::VecDeque;
pub struct VecdequeFrontMutMapPayload {
    value: String,
}

impl VecdequeFrontMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vecdeque-front-mut-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vecdeque-front-mut-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vecdeque-front-mut-map:{}", self.value)
    }
}

pub fn selected_vecdeque_front_mut_map(raw: &str) -> String {
    let mut values = VecDeque::from([VecdequeFrontMutMapPayload::new(raw)]);
    values
        .front_mut()
        .map(|payload| payload.bump_and_render())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_vecdeque_front_mut_map(raw: &str) -> String {
    VecdequeFrontMutMapPayload::new(raw).unused_label()
}
