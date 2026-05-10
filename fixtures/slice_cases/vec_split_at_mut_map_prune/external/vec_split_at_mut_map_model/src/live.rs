pub struct VecSplitAtMutMapPayload {
    value: String,
}

impl VecSplitAtMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-split-at-mut-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vec-split-at-mut-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vec-split-at-mut-map:{}", self.value)
    }
}

pub fn selected_vec_split_at_mut_map(raw: &str) -> String {
    let mut values = vec![
        VecSplitAtMutMapPayload::new(raw),
        VecSplitAtMutMapPayload::new("tail"),
    ];
    let (head, _tail) = values.split_at_mut(1);
    head.iter_mut()
        .map(|payload| payload.bump_and_render())
        .next()
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_vec_split_at_mut_map(raw: &str) -> String {
    VecSplitAtMutMapPayload::new(raw).unused_label()
}
