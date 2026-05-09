pub struct VecGetMutMapPayload {
    value: String,
}

impl VecGetMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("vec-get-mut-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("vec-get-mut-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-vec-get-mut-map:{}", self.value)
    }
}

pub fn selected_vec_get_mut_map(raw: &str) -> String {
    let mut values = vec![VecGetMutMapPayload::new(raw)];
    values
        .get_mut(0)
        .map(|payload| payload.bump_and_render())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_vec_get_mut_map(raw: &str) -> String {
    VecGetMutMapPayload::new(raw).unused_label()
}
