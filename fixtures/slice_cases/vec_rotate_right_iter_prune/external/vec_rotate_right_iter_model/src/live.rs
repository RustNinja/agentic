#[derive(Clone)]
pub struct VecRotateRightIterPayload {
    value: String,
}

impl VecRotateRightIterPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-rotate-right-iter:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("vec-rotate-right-iter:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-rotate-right-iter:{}", self.value)
    }
}

pub fn selected_vec_rotate_right_iter(raw: &str) -> String {
    let mut payloads = vec![
        VecRotateRightIterPayload::new(raw),
        VecRotateRightIterPayload::new("middle"),
        VecRotateRightIterPayload::new("tail"),
    ];
    payloads.rotate_right(1);
    payloads
        .iter()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "vec-rotate-right-iter:missing".to_string())
}

pub fn dead_live_vec_rotate_right_iter(raw: &str) -> String {
    VecRotateRightIterPayload::new(raw).dead_method()
}
