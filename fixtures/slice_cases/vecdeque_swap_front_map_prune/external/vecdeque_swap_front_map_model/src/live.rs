use std::collections::VecDeque;
pub struct VecdequeSwapFrontMapPayload {
    value: String,
}

impl VecdequeSwapFrontMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vecdeque-swap-front-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vecdeque-swap-front-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vecdeque-swap-front-map:{}", self.value)
    }
}

pub fn selected_vecdeque_swap_front_map(raw: &str) -> String {
    let mut values = VecDeque::from([
        VecdequeSwapFrontMapPayload::new("left"),
        VecdequeSwapFrontMapPayload::new(raw),
    ]);
    values.swap(0, 1);
    values
        .front()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_vecdeque_swap_front_map(raw: &str) -> String {
    VecdequeSwapFrontMapPayload::new(raw).unused_label()
}
