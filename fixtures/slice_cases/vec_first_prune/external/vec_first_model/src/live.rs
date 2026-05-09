pub struct VecFirstPayload {
    value: String,
}

impl VecFirstPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-first:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-first:{}", self.value)
    }
}

fn vec_first_items(raw: &str) -> Vec<VecFirstPayload> {
    vec![VecFirstPayload::new(raw), VecFirstPayload::new("tail")]
}

pub fn selected_vec_first(raw: &str) -> String {
    vec_first_items(raw)
        .first()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "vec-first:missing".to_string())
}

pub fn dead_live_vec_first(raw: &str) -> String {
    VecFirstPayload::new(raw).dead_method()
}
