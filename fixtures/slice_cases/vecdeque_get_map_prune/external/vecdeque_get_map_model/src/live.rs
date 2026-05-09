use std::collections::VecDeque;
pub struct VecdequeGetMapPayload {
    value: String,
}

impl VecdequeGetMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vecdeque-get-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vecdeque-get-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vecdeque-get-map:{}", self.value)
    }
}

pub fn selected_vecdeque_get_map(raw: &str) -> String {
    let values = VecDeque::from([VecdequeGetMapPayload::new(raw)]);
    values
        .get(0)
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_vecdeque_get_map(raw: &str) -> String {
    VecdequeGetMapPayload::new(raw).unused_label()
}
