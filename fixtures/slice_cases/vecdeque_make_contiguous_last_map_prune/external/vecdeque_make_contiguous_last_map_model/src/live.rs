use std::collections::VecDeque;
pub struct VecdequeMakeContiguousLastMapPayload {
    value: String,
}

impl VecdequeMakeContiguousLastMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vecdeque-make-contiguous-last-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vecdeque-make-contiguous-last-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vecdeque-make-contiguous-last-map:{}", self.value)
    }
}

pub fn selected_vecdeque_make_contiguous_last_map(raw: &str) -> String {
    let mut entries = VecDeque::from([VecdequeMakeContiguousLastMapPayload::new(raw)]);
    entries
        .make_contiguous()
        .last()
        .map(|payload| payload.render_label())
        .unwrap_or_default()
}

pub fn dead_live_vecdeque_make_contiguous_last_map(raw: &str) -> String {
    VecdequeMakeContiguousLastMapPayload::new(raw).unused_label()
}
