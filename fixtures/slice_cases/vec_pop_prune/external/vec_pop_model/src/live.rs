pub struct VecPopPayload {
    value: String,
}

impl VecPopPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-pop:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-pop:{}", self.value)
    }
}

fn vec_pop_items(raw: &str) -> Vec<VecPopPayload> {
    vec![VecPopPayload::new(raw), VecPopPayload::new("tail")]
}

pub fn selected_vec_pop(raw: &str) -> String {
    let mut payloads = vec_pop_items(raw);
    payloads
        .pop()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "vec-pop:missing".to_string())
}

pub fn dead_live_vec_pop(raw: &str) -> String {
    VecPopPayload::new(raw).dead_method()
}
