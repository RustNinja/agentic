pub struct VecSplitOffLastMapPayload {
    value: String,
}

impl VecSplitOffLastMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-split-off-last-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vec-split-off-last-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vec-split-off-last-map:{}", self.value)
    }
}

pub fn selected_vec_split_off_last_map(raw: &str) -> String {
    let mut values = vec![
        VecSplitOffLastMapPayload::new("head"),
        VecSplitOffLastMapPayload::new(raw),
    ];
    let tail = values.split_off(1);
    tail.last()
        .map(|payload| payload.render_label())
        .unwrap_or_default()
}

pub fn dead_live_vec_split_off_last_map(raw: &str) -> String {
    VecSplitOffLastMapPayload::new(raw).unused_label()
}
