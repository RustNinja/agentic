use std::collections::VecDeque;
pub struct VecdequeRangeMapPayload {
    value: String,
}

impl VecdequeRangeMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vecdeque-range-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vecdeque-range-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vecdeque-range-map:{}", self.value)
    }
}

pub fn selected_vecdeque_range_map(raw: &str) -> String {
    let values = VecDeque::from([
        VecdequeRangeMapPayload::new(raw),
        VecdequeRangeMapPayload::new("tail"),
    ]);
    values
        .range(0..1)
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_vecdeque_range_map(raw: &str) -> String {
    VecdequeRangeMapPayload::new(raw).unused_label()
}
