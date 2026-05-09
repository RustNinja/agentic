use std::collections::VecDeque;
pub struct VecdequeRotateRightBackMapPayload {
    value: String,
}

impl VecdequeRotateRightBackMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vecdeque-rotate-right-back-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vecdeque-rotate-right-back-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vecdeque-rotate-right-back-map:{}", self.value)
    }
}

pub fn selected_vecdeque_rotate_right_back_map(raw: &str) -> String {
    let mut values = VecDeque::from([
        VecdequeRotateRightBackMapPayload::new(raw),
        VecdequeRotateRightBackMapPayload::new("last"),
    ]);
    values.rotate_right(1);
    values
        .back()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_vecdeque_rotate_right_back_map(raw: &str) -> String {
    VecdequeRotateRightBackMapPayload::new(raw).unused_label()
}
