#[derive(Clone)]
pub struct VecReverseIterPayload {
    value: String,
}

impl VecReverseIterPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-reverse-iter:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("vec-reverse-iter:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-reverse-iter:{}", self.value)
    }
}

pub fn selected_vec_reverse_iter(raw: &str) -> String {
    let mut payloads = vec![
        VecReverseIterPayload::new(raw),
        VecReverseIterPayload::new("tail"),
    ];
    payloads.reverse();
    payloads
        .iter()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "vec-reverse-iter:missing".to_string())
}

pub fn dead_live_vec_reverse_iter(raw: &str) -> String {
    VecReverseIterPayload::new(raw).dead_method()
}
