use std::collections::VecDeque;
pub struct VecdequeMakeContiguousFirstMutMapPayload {
    value: String,
}

impl VecdequeMakeContiguousFirstMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vecdeque-make-contiguous-first-mut-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vecdeque-make-contiguous-first-mut-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vecdeque-make-contiguous-first-mut-map:{}", self.value)
    }
}

pub fn selected_vecdeque_make_contiguous_first_mut_map(raw: &str) -> String {
    let mut entries = VecDeque::from([VecdequeMakeContiguousFirstMutMapPayload::new(raw)]);
    entries
        .make_contiguous()
        .first_mut()
        .map(|payload| payload.bump_and_render())
        .unwrap_or_default()
}

pub fn dead_live_vecdeque_make_contiguous_first_mut_map(raw: &str) -> String {
    VecdequeMakeContiguousFirstMutMapPayload::new(raw).unused_label()
}
