#[derive(Clone)]
pub struct VecFillWithIterPayload {
    value: String,
}

impl VecFillWithIterPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-fill-with-iter:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("vec-fill-with-iter:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-fill-with-iter:{}", self.value)
    }
}

pub fn selected_vec_fill_with_iter(raw: &str) -> String {
    let mut payloads = vec![
        VecFillWithIterPayload::new(raw),
        VecFillWithIterPayload::new("tail"),
    ];
    payloads.fill_with(|| VecFillWithIterPayload::new("filled"));
    payloads
        .iter()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "vec-fill-with-iter:missing".to_string())
}

pub fn dead_live_vec_fill_with_iter(raw: &str) -> String {
    VecFillWithIterPayload::new(raw).dead_method()
}
