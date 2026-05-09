#[derive(Clone)]
pub struct VecSpliceMapPayload {
    value: String,
}

impl VecSpliceMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-splice-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("vec-splice-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-splice-map:{}", self.value)
    }
}

pub fn selected_vec_splice_map(raw: &str) -> String {
    let mut payloads = vec![
        VecSpliceMapPayload::new(raw),
        VecSpliceMapPayload::new("tail"),
    ];
    let label = payloads
        .splice(0..1, vec![VecSpliceMapPayload::new("replacement")])
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "vec-splice-map:missing".to_string());
    label
}

pub fn dead_live_vec_splice_map(raw: &str) -> String {
    VecSpliceMapPayload::new(raw).dead_method()
}
