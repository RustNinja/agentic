#[derive(Clone)]
pub struct VecLeakIterPayload {
    value: String,
}

impl VecLeakIterPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-leak-iter:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("vec-leak-iter:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-leak-iter:{}", self.value)
    }
}

pub fn selected_vec_leak_iter(raw: &str) -> String {
    let payloads = vec![
        VecLeakIterPayload::new(raw),
        VecLeakIterPayload::new("tail"),
    ];
    payloads
        .leak()
        .iter()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "vec-leak-iter:missing".to_string())
}

pub fn dead_live_vec_leak_iter(raw: &str) -> String {
    VecLeakIterPayload::new(raw).dead_method()
}
