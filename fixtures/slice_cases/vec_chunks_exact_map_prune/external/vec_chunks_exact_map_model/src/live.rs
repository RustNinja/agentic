pub struct VecChunksExactMapPayload {
    value: String,
}

impl VecChunksExactMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-chunks-exact-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vec-chunks-exact-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vec-chunks-exact-map:{}", self.value)
    }
}

pub fn selected_vec_chunks_exact_map(raw: &str) -> String {
    let values = vec![
        VecChunksExactMapPayload::new(raw),
        VecChunksExactMapPayload::new("tail"),
    ];
    values
        .chunks_exact(1)
        .filter_map(|chunk| chunk.first())
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_vec_chunks_exact_map(raw: &str) -> String {
    VecChunksExactMapPayload::new(raw).unused_label()
}
