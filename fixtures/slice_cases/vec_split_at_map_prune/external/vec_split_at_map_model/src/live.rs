pub struct VecSplitAtMapPayload {
    value: String,
}

impl VecSplitAtMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-split-at-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vec-split-at-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vec-split-at-map:{}", self.value)
    }
}

pub fn selected_vec_split_at_map(raw: &str) -> String {
    let values = vec![
        VecSplitAtMapPayload::new(raw),
        VecSplitAtMapPayload::new("tail"),
    ];
    let (head, _tail) = values.split_at(1);
    head.iter()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_vec_split_at_map(raw: &str) -> String {
    VecSplitAtMapPayload::new(raw).unused_label()
}
