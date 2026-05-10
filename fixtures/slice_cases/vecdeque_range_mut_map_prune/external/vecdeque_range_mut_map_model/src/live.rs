use std::collections::VecDeque;

pub fn selected_vecdeque_range_mut_map(raw: &str) -> String {
    let mut payloads = VecDeque::from(vecdeque_range_mut_map_items(raw));
    payloads
        .range_mut(0..2)
        .map(|payload| payload.bump_and_render())
        .collect::<Vec<_>>()
        .join("|")
}

pub struct VecdequeRangeMutMapPayload {
    value: String,
}

impl VecdequeRangeMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vecdeque_range_mut_map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vecdeque_range_mut_map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vecdeque-range-mut-map:{}", self.value)
    }
}

fn vecdeque_range_mut_map_items(raw: &str) -> Vec<VecdequeRangeMutMapPayload> {
    vec![
        VecdequeRangeMutMapPayload::new(raw),
        VecdequeRangeMutMapPayload::new("middle"),
        VecdequeRangeMutMapPayload::new("tail"),
    ]
}

pub fn dead_live_vecdeque_range_mut_map(raw: &str) -> String {
    VecdequeRangeMutMapPayload::new(raw).unused_label()
}
