#[derive(Clone)]
pub struct VecResizeCloneMapPayload {
    value: String,
}

impl VecResizeCloneMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-resize-clone-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vec-resize-clone-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vec-resize-clone-map:{}", self.value)
    }
}

pub fn selected_vec_resize_clone_map(raw: &str) -> String {
    let mut values = vec![VecResizeCloneMapPayload::new("head")];
    values.resize(2, VecResizeCloneMapPayload::new(raw));
    values
        .last()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_vec_resize_clone_map(raw: &str) -> String {
    VecResizeCloneMapPayload::new(raw).unused_label()
}
