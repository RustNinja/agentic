use std::collections::VecDeque;
pub struct VecdequeResizeWithBackMapPayload {
    value: String,
}

impl VecdequeResizeWithBackMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vecdeque-resize-with-back-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vecdeque-resize-with-back-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vecdeque-resize-with-back-map:{}", self.value)
    }
}

pub fn selected_vecdeque_resize_with_back_map(raw: &str) -> String {
    let mut values = VecDeque::new();
    values.resize_with(2, || VecdequeResizeWithBackMapPayload::new(raw));
    values
        .back()
        .map(|payload| payload.render_label())
        .unwrap_or_default()
}

pub fn dead_live_vecdeque_resize_with_back_map(raw: &str) -> String {
    VecdequeResizeWithBackMapPayload::new(raw).unused_label()
}
