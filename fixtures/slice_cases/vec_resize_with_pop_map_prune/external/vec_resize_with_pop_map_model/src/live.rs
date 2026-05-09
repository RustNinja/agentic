pub struct VecResizeWithPopMapPayload {
    value: String,
}

impl VecResizeWithPopMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-resize-with-pop-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vec-resize-with-pop-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vec-resize-with-pop-map:{}", self.value)
    }
}

pub fn selected_vec_resize_with_pop_map(raw: &str) -> String {
    let mut values = Vec::new();
    values.resize_with(2, || VecResizeWithPopMapPayload::new(raw));
    values
        .pop()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_vec_resize_with_pop_map(raw: &str) -> String {
    VecResizeWithPopMapPayload::new(raw).unused_label()
}
