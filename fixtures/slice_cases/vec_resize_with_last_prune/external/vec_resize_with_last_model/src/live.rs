#[derive(Clone)]
pub struct VecResizeWithLastPayload {
    value: String,
}

impl VecResizeWithLastPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-resize-with-last:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("vec-resize-with-last:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-resize-with-last:{}", self.value)
    }
}

pub fn selected_vec_resize_with_last(raw: &str) -> String {
    let mut payloads = vec![VecResizeWithLastPayload::new(raw)];
    payloads.resize_with(2, || VecResizeWithLastPayload::new("fresh"));
    payloads
        .last()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "vec-resize-with-last:missing".to_string())
}

pub fn dead_live_vec_resize_with_last(raw: &str) -> String {
    VecResizeWithLastPayload::new(raw).dead_method()
}
