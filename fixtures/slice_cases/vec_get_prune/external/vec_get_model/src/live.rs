pub struct VecGetPayload {
    value: String,
}

impl VecGetPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-get:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-get:{}", self.value)
    }
}

fn vec_get_items(raw: &str) -> Vec<VecGetPayload> {
    vec![VecGetPayload::new(raw), VecGetPayload::new("tail")]
}

pub fn selected_vec_get(raw: &str) -> String {
    vec_get_items(raw)
        .get(1)
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "vec-get:missing".to_string())
}

pub fn dead_live_vec_get(raw: &str) -> String {
    VecGetPayload::new(raw).dead_method()
}
