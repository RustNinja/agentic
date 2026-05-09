pub struct VecLastPayload {
    value: String,
}

impl VecLastPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-last:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-last:{}", self.value)
    }
}

fn vec_last_items(raw: &str) -> Vec<VecLastPayload> {
    vec![VecLastPayload::new(raw), VecLastPayload::new("tail")]
}

pub fn selected_vec_last(raw: &str) -> String {
    vec_last_items(raw)
        .last()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "vec-last:missing".to_string())
}

pub fn dead_live_vec_last(raw: &str) -> String {
    VecLastPayload::new(raw).dead_method()
}
