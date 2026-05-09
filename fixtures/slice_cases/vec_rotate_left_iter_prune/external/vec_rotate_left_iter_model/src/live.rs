#[derive(Clone)]
pub struct VecRotateLeftIterPayload {
    value: String,
}

impl VecRotateLeftIterPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-rotate-left-iter:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("vec-rotate-left-iter:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-rotate-left-iter:{}", self.value)
    }
}

pub fn selected_vec_rotate_left_iter(raw: &str) -> String {
    let mut payloads = vec![
        VecRotateLeftIterPayload::new(raw),
        VecRotateLeftIterPayload::new("middle"),
        VecRotateLeftIterPayload::new("tail"),
    ];
    payloads.rotate_left(1);
    payloads
        .iter()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "vec-rotate-left-iter:missing".to_string())
}

pub fn dead_live_vec_rotate_left_iter(raw: &str) -> String {
    VecRotateLeftIterPayload::new(raw).dead_method()
}
