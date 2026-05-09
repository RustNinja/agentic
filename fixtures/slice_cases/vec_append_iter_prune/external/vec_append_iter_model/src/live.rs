#[derive(Clone)]
pub struct VecAppendIterPayload {
    value: String,
}

impl VecAppendIterPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-append-iter:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("vec-append-iter:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-append-iter:{}", self.value)
    }
}

pub fn selected_vec_append_iter(raw: &str) -> String {
    let mut payloads = vec![VecAppendIterPayload::new(raw)];
    let mut extras = vec![VecAppendIterPayload::new("tail")];
    payloads.append(&mut extras);
    payloads
        .iter()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "vec-append-iter:missing".to_string())
}

pub fn dead_live_vec_append_iter(raw: &str) -> String {
    VecAppendIterPayload::new(raw).dead_method()
}
