use std::collections::VecDeque;
pub struct VecdequeIterNextBackMapPayload {
    value: String,
}

impl VecdequeIterNextBackMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vecdeque-iter-next-back-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vecdeque-iter-next-back-map:{}", self.value)
    }

    pub fn is_match(&self) -> bool {
        !self.value.is_empty()
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vecdeque-iter-next-back-map:{}", self.value)
    }
}

fn vecdeque_iter_next_back_map_items(raw: &str) -> VecDeque<VecdequeIterNextBackMapPayload> {
    let mut values = VecDeque::new();
    values.push_back(VecdequeIterNextBackMapPayload::new("head"));
    values.push_back(VecdequeIterNextBackMapPayload::new(raw));
    values
}

pub fn selected_vecdeque_iter_next_back_map(raw: &str) -> String {
    vecdeque_iter_next_back_map_items(raw)
        .iter()
        .next_back()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "vecdeque-iter-next-back-map:missing".to_string())
}

pub fn dead_live_vecdeque_iter_next_back_map(raw: &str) -> String {
    VecdequeIterNextBackMapPayload::new(raw).unused_label()
}
