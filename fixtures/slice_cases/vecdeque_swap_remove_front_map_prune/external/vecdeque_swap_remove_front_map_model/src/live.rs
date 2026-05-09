use std::collections::VecDeque;
pub struct VecdequeSwapRemoveFrontMapPayload {
    value: String,
}

impl VecdequeSwapRemoveFrontMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vecdeque-swap-remove-front-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vecdeque-swap-remove-front-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vecdeque-swap-remove-front-map:{}", self.value)
    }
}

pub fn selected_vecdeque_swap_remove_front_map(raw: &str) -> String {
    let mut values = VecDeque::from([
        VecdequeSwapRemoveFrontMapPayload::new(raw),
        VecdequeSwapRemoveFrontMapPayload::new("tail"),
    ]);
    values
        .swap_remove_front(0)
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_vecdeque_swap_remove_front_map(raw: &str) -> String {
    VecdequeSwapRemoveFrontMapPayload::new(raw).unused_label()
}
