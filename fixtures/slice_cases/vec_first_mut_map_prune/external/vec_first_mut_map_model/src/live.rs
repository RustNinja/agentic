pub struct VecFirstMutMapPayload {
    value: String,
}

impl VecFirstMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-first-mut-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vec-first-mut-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vec-first-mut-map:{}", self.value)
    }
}

pub fn selected_vec_first_mut_map(raw: &str) -> String {
    let mut values = vec![VecFirstMutMapPayload::new(raw)];
    values
        .first_mut()
        .map(|payload| payload.bump_and_render())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_vec_first_mut_map(raw: &str) -> String {
    VecFirstMutMapPayload::new(raw).unused_label()
}
