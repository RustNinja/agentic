use std::collections::VecDeque;
pub struct VecdequeBackMutMapPayload {
    value: String,
}

impl VecdequeBackMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vecdeque-back-mut-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vecdeque-back-mut-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vecdeque-back-mut-map:{}", self.value)
    }
}

pub fn selected_vecdeque_back_mut_map(raw: &str) -> String {
    let mut values = VecDeque::from([VecdequeBackMutMapPayload::new(raw)]);
    values
        .back_mut()
        .map(|payload| payload.bump_and_render())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_vecdeque_back_mut_map(raw: &str) -> String {
    VecdequeBackMutMapPayload::new(raw).unused_label()
}
