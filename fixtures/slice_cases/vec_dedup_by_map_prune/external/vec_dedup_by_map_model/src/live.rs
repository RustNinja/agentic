pub struct VecDedupByMapPayload {
    value: String,
}

impl VecDedupByMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-dedup-by-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vec-dedup-by-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vec-dedup-by-map:{}", self.value)
    }
}

pub fn selected_vec_dedup_by_map(raw: &str) -> String {
    let mut values = vec![
        VecDedupByMapPayload::new(raw),
        VecDedupByMapPayload::new(raw),
        VecDedupByMapPayload::new("tail"),
    ];
    values.dedup_by(|left, right| left.value == right.value);
    values
        .iter()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_vec_dedup_by_map(raw: &str) -> String {
    VecDedupByMapPayload::new(raw).unused_label()
}
