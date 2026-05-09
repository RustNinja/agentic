use std::collections::VecDeque;
pub struct VecdequeRotateLeftFrontMapPayload {
    value: String,
}

impl VecdequeRotateLeftFrontMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vecdeque-rotate-left-front-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vecdeque-rotate-left-front-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vecdeque-rotate-left-front-map:{}", self.value)
    }
}

pub fn selected_vecdeque_rotate_left_front_map(raw: &str) -> String {
    let mut values = VecDeque::from([
        VecdequeRotateLeftFrontMapPayload::new("first"),
        VecdequeRotateLeftFrontMapPayload::new(raw),
    ]);
    values.rotate_left(1);
    values
        .front()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_vecdeque_rotate_left_front_map(raw: &str) -> String {
    VecdequeRotateLeftFrontMapPayload::new(raw).unused_label()
}
