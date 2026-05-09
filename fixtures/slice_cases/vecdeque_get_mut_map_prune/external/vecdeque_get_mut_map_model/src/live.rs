use std::collections::VecDeque;
pub struct VecdequeGetMutMapPayload {
    value: String,
}

impl VecdequeGetMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vecdeque-get-mut-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vecdeque-get-mut-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vecdeque-get-mut-map:{}", self.value)
    }
}

pub fn selected_vecdeque_get_mut_map(raw: &str) -> String {
    let mut values = VecDeque::from([VecdequeGetMutMapPayload::new(raw)]);
    values
        .get_mut(0)
        .map(|payload| payload.bump_and_render())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_vecdeque_get_mut_map(raw: &str) -> String {
    VecdequeGetMutMapPayload::new(raw).unused_label()
}
