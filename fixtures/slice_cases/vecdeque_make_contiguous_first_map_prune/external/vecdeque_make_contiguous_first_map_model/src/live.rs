use std::collections::VecDeque;
pub struct VecdequeMakeContiguousFirstMapPayload {
    value: String,
}

impl VecdequeMakeContiguousFirstMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vecdeque-make-contiguous-first-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vecdeque-make-contiguous-first-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vecdeque-make-contiguous-first-map:{}", self.value)
    }
}

pub fn selected_vecdeque_make_contiguous_first_map(raw: &str) -> String {
    let mut entries = VecDeque::from([VecdequeMakeContiguousFirstMapPayload::new(raw)]);
    entries
        .make_contiguous()
        .first()
        .map(|payload| payload.render_label())
        .unwrap_or_default()
}

pub fn dead_live_vecdeque_make_contiguous_first_map(raw: &str) -> String {
    VecdequeMakeContiguousFirstMapPayload::new(raw).unused_label()
}
