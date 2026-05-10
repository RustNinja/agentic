use std::collections::VecDeque;
pub struct VecdequeMakeContiguousLastMutMapPayload {
    value: String,
}

impl VecdequeMakeContiguousLastMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vecdeque-make-contiguous-last-mut-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vecdeque-make-contiguous-last-mut-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vecdeque-make-contiguous-last-mut-map:{}", self.value)
    }
}

pub fn selected_vecdeque_make_contiguous_last_mut_map(raw: &str) -> String {
    let mut entries = VecDeque::from([VecdequeMakeContiguousLastMutMapPayload::new(raw)]);
    entries
        .make_contiguous()
        .last_mut()
        .map(|payload| payload.bump_and_render())
        .unwrap_or_default()
}

pub fn dead_live_vecdeque_make_contiguous_last_mut_map(raw: &str) -> String {
    VecdequeMakeContiguousLastMutMapPayload::new(raw).unused_label()
}
