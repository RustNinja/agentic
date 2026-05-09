pub struct IterRepeatWithTakeMapPayload {
    value: String,
}

impl IterRepeatWithTakeMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iter-repeat-with-take-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-iter-repeat-with-take-map:{}", self.value)
    }
}

pub fn selected_iter_repeat_with_take_map(raw: &str) -> String {
    std::iter::repeat_with(|| IterRepeatWithTakeMapPayload::new(raw))
        .take(2)
        .map(|payload| payload.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_iter_repeat_with_take_map(raw: &str) -> String {
    IterRepeatWithTakeMapPayload::new(raw).unused_label()
}
